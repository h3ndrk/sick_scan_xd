use std::{env::var, path::PathBuf};

use cc::Build;

fn main() {
    let out_path = PathBuf::from(var("OUT_DIR").unwrap());

    Build::new()
        .file("../test/src/sick_scan_xd_api/sick_scan_xd_api_wrapper.c")
        .include("../include")
        .compile("wrapper");

    let bindings = bindgen::Builder::default()
        .header("../include/sick_scan_xd_api/sick_scan_api.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Failed to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Failed to write bindings");
}
