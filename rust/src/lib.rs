use std::{
    env::args, ffi::CString, mem::ManuallyDrop, os::raw::c_int, ptr::null_mut,
    slice::from_raw_parts, thread::sleep, time::Duration,
};

use byteorder::{LittleEndian, ReadBytesExt};

mod bindings;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to load shared library: SickScanApiLoadLibrary returned {internal}")]
    LibraryNotLoaded { internal: i32 },
    #[error("failed to initialize from command line: SickScanApiInitByCli returned {internal}")]
    NotInitialized { internal: i32 },
    #[error("failed to get cloud message: SickScanApiWaitNextCartesianPointCloudMsg returned {internal}")]
    NoCloudMessage { internal: i32 },
}

pub struct SickScanApiHandle {
    api_handle: bindings::SickScanApiHandle,
}

impl SickScanApiHandle {
    pub fn load() -> Result<(), Error> {
        let file_path = CString::new("libsick_scan_shared_lib.so").unwrap();
        let result = unsafe { bindings::SickScanApiLoadLibrary(file_path.as_ptr()) };
        match result {
            0 => Ok(()),
            _ => Err(Error::LibraryNotLoaded { internal: result }),
        }
    }

    pub fn create() -> SickScanApiHandle {
        let argv_strings: Vec<_> = args()
            .map(|argument| CString::new(argument).unwrap())
            .collect();
        let mut argv: ManuallyDrop<Vec<_>> = ManuallyDrop::new(
            argv_strings
                .into_iter()
                .map(|argument| argument.into_raw())
                .collect(),
        );
        unsafe {
            let api_handle = SickScanApiHandle {
                api_handle: bindings::SickScanApiCreate(argv.len() as c_int, argv.as_mut_ptr()),
            };
            // for argument in argv {
            //     let string = CString::from_raw(argument);
            //     println!("string: {string:?}");
            // }
            api_handle
        }
    }

    pub fn initialize_from_command_line(&self) -> Result<(), Error> {
        let argv_strings: Vec<_> = args()
            .map(|argument| CString::new(argument).unwrap())
            .collect();
        let mut argv: ManuallyDrop<Vec<_>> = ManuallyDrop::new(
            argv_strings
                .into_iter()
                .map(|argument| argument.into_raw())
                .collect(),
        );
        unsafe {
            let result = bindings::SickScanApiInitByCli(
                self.api_handle,
                argv.len() as c_int,
                argv.as_mut_ptr(),
            );
            // for argument in argv {
            //     drop(CString::from_raw(argument));
            // }
            match result {
                0 => Ok(()),
                _ => Err(Error::NotInitialized { internal: result }),
            }
        }
    }

    pub fn wait_for_next_cartesian_point_cloud_message(
        &self,
        timeout: Duration,
    ) -> Result<CloudMessage, Error> {
        let mut message = Box::new(bindings::SickScanPointCloudMsg {
            header: bindings::SickScanHeaderType {
                seq: 0,
                timestamp_sec: 0,
                timestamp_nsec: 0,
                frame_id: [0; 256],
            },
            height: 0,
            width: 0,
            fields: bindings::SickScanPointFieldArrayType {
                capacity: 0,
                size: 0,
                buffer: null_mut() as *mut bindings::SickScanPointFieldMsg,
            },
            is_bigendian: 0,
            point_step: 0,
            row_step: 0,
            data: bindings::SickScanUint8Array {
                capacity: 0,
                size: 0,
                buffer: null_mut() as *mut u8,
            },
            is_dense: 0,
            num_echos: 0,
            segment_idx: 0,
        });
        let result = unsafe {
            bindings::SickScanApiWaitNextCartesianPointCloudMsg(
                self.api_handle,
                &mut *message as *mut _,
                timeout.as_secs_f64(),
            )
        };
        match result {
            0 => Ok(CloudMessage::Cartesian {
                api_handle: self.api_handle,
                message,
            }),
            5 => {
                println!("timeout");
                Ok(CloudMessage::Cartesian {
                    api_handle: self.api_handle,
                    message,
                })
            }
            _ => Err(Error::NotInitialized { internal: result }),
        }
    }
}

pub enum CloudMessage {
    Cartesian {
        api_handle: bindings::SickScanApiHandle,
        message: Box<bindings::SickScanPointCloudMsg>,
    },
}

impl Drop for CloudMessage {
    fn drop(&mut self) {
        // let (api_handle, message) = match self {
        //     CloudMessage::Cartesian {
        //         api_handle,
        //         message,
        //     } => (*api_handle, message),
        // };
        // unsafe {
        //     bindings::SickScanApiFreePointCloudMsg(api_handle, &mut **message as *mut _);
        // }
    }
}

#[derive(Debug)]
pub struct CartesianPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub intensity: f32,
}

impl CloudMessage {
    pub fn get_data(&self) -> Vec<CartesianPoint> {
        match self {
            CloudMessage::Cartesian { message, .. } => {
                if message.fields.size != 4 {
                    println!("leer");
                    return vec![];
                }

                assert_eq!(message.fields.size, 4);
                assert_ne!(message.fields.buffer, null_mut());

                let field_x = unsafe { *message.fields.buffer.add(0) };
                assert_eq!(field_x.name[0], 'x' as i8);
                assert_eq!(field_x.name[1], '\0' as i8);
                assert_eq!(field_x.offset, 0);
                assert_eq!(
                    field_x.datatype,
                    bindings::SickScanNativeDataType_SICK_SCAN_POINTFIELD_DATATYPE_FLOAT32 as u8
                );
                assert_eq!(field_x.count, 1);

                let field_y = unsafe { *message.fields.buffer.add(1) };
                assert_eq!(field_y.name[0], 'y' as i8);
                assert_eq!(field_y.name[1], '\0' as i8);
                assert_eq!(field_y.offset, 4);
                assert_eq!(
                    field_y.datatype,
                    bindings::SickScanNativeDataType_SICK_SCAN_POINTFIELD_DATATYPE_FLOAT32 as u8
                );
                assert_eq!(field_y.count, 1);

                let field_z = unsafe { *message.fields.buffer.add(2) };
                assert_eq!(field_z.name[0], 'z' as i8);
                assert_eq!(field_z.name[1], '\0' as i8);
                assert_eq!(field_z.offset, 8);
                assert_eq!(
                    field_z.datatype,
                    bindings::SickScanNativeDataType_SICK_SCAN_POINTFIELD_DATATYPE_FLOAT32 as u8
                );
                assert_eq!(field_z.count, 1);

                let field_intensity = unsafe { *message.fields.buffer.add(3) };
                assert_eq!(field_intensity.name[0], 'i' as i8);
                assert_eq!(field_intensity.name[1], 'n' as i8);
                assert_eq!(field_intensity.name[2], 't' as i8);
                assert_eq!(field_intensity.name[3], 'e' as i8);
                assert_eq!(field_intensity.name[4], 'n' as i8);
                assert_eq!(field_intensity.name[5], 's' as i8);
                assert_eq!(field_intensity.name[6], 'i' as i8);
                assert_eq!(field_intensity.name[7], 't' as i8);
                assert_eq!(field_intensity.name[8], 'y' as i8);
                assert_eq!(field_intensity.name[9], '\0' as i8);
                assert_eq!(field_intensity.offset, 12);
                assert_eq!(
                    field_intensity.datatype,
                    bindings::SickScanNativeDataType_SICK_SCAN_POINTFIELD_DATATYPE_FLOAT32 as u8
                );
                assert_eq!(field_intensity.count, 1);

                assert_eq!(message.is_bigendian, 0);

                assert_ne!(message.data.buffer, null_mut());

                dbg!(message);

                (0..(message.data.size as usize / message.point_step as usize))
                    .map(|index| {
                        let mut data = unsafe {
                            from_raw_parts(
                                message.data.buffer.add(index * message.point_step as usize),
                                4 * 4,
                            )
                        };
                        // sleep(Duration::from_millis(1));
                        let x = data.read_f32::<LittleEndian>().unwrap();
                        let y = data.read_f32::<LittleEndian>().unwrap();
                        let z = data.read_f32::<LittleEndian>().unwrap();
                        let intensity = data.read_f32::<LittleEndian>().unwrap();
                        CartesianPoint { x, y, z, intensity }
                    })
                    .collect()
            }
        }
    }
}
