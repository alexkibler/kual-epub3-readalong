use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    // Tell cargo to link FBInk
    // On the Kindle, FBInk might be in /mnt/us/extensions/FBInk or bundled
    println!("cargo:rustc-link-search=native=/mnt/us/extensions/FBInk/lib");
    println!("cargo:rustc-link-lib=fbink");

    // Generate bindings for FBInk
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Allow FBInk types
        .allowlist_function("fbink_.*")
        .allowlist_type("FBInk.*")
        .allowlist_var("FBINK_.*")
        // Generate bindings
        .generate()
        .expect("Unable to generate FBInk bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("fbink_bindings.rs"))
        .expect("Couldn't write bindings!");
}
