use std::{time::Duration, thread::sleep};

use sick_scan_xd::SickScanApiHandle;

fn main() {
    println!("Test");
    SickScanApiHandle::load().unwrap();
    let api_handle = SickScanApiHandle::create();
    api_handle.initialize_from_command_line().unwrap();
    loop {
        let message = api_handle
            .wait_for_next_cartesian_point_cloud_message(Duration::from_secs(1))
            .unwrap();
        let data = message.get_data();
        for item in data {
            println!("data: {:?}", item);
        }
        // sleep(Duration::from_secs(1));
    }
}
