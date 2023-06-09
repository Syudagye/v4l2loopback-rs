use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=v4l2loopback.h");

    let bindings = bindgen::Builder::default()
        .header("v4l2loopback/v4l2loopback.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("v4l2loopback.rs"))
        .expect("Couldn't write bindings for v4l2 loopback types");
}
