use std::env;
use std::path::PathBuf;

fn main() {
    // Running tests concurrently makes `tests::device_query_infos` fail sometimes
    // Reducing the tests thread count to one fixes the issue
    println!("cargo:rustc-env=RUST_TEST_THREADS=1");

    // Generating bindings

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
