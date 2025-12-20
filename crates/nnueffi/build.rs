use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Get the manifest directory (where Cargo.toml is)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let lib_path = PathBuf::from(&manifest_dir).join("assets/nnue-probe/src");

    // Convert to absolute path
    let lib_path_absolute = fs::canonicalize(&lib_path).unwrap_or_else(|_| lib_path.clone());

    // Tell cargo to look for shared libraries in the specified directory
    println!(
        "cargo:rustc-link-search=native={}",
        lib_path_absolute.display()
    );

    // Tell cargo to link the nnueprobe library
    println!("cargo:rustc-link-lib=dylib=nnueprobe");

    // Hardcode the ABSOLUTE rpath so we don't need LD_LIBRARY_PATH
    // This embeds the library path directly into the binary
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}",
        lib_path_absolute.display()
    );

    // Tell cargo to invalidate the built crate whenever the library changes
    println!("cargo:rerun-if-changed=assets/nnue-probe/src/libnnueprobe.so");
    println!("cargo:rerun-if-changed=assets/nnue-probe/src/nnue.cpp");
    println!("cargo:rerun-if-changed=assets/nnue-probe/src/misc.cpp");

    // Debug output
    eprintln!("NNUE library rpath set to: {}", lib_path_absolute.display());
}
