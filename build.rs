use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=third-party/sdl3/bin/SDL3.dll");
    println!("cargo:rerun-if-changed=third-party/sdl3/lib/libSDL3.dll.a");
    if env::var_os("CARGO_CFG_WINDOWS").is_none() {
        return;
    }
    let manifest_directory = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let sdl_library_directory = manifest_directory.join("third-party/sdl3/lib");
    let sdl_dll = manifest_directory.join("third-party/sdl3/bin/SDL3.dll");
    println!(
        "cargo:rustc-link-search=native={}",
        sdl_library_directory.display()
    );
    copy_sdl_dll(&sdl_dll);
}

fn copy_sdl_dll(sdl_dll: &Path) {
    let out_directory = PathBuf::from(env::var("OUT_DIR").unwrap());
    let Some(profile_directory) = out_directory.ancestors().nth(3) else {
        return;
    };
    for target_directory in [
        profile_directory.to_path_buf(),
        profile_directory.join("deps"),
    ] {
        if fs::create_dir_all(&target_directory).is_ok() {
            let _ = fs::copy(sdl_dll, target_directory.join("SDL3.dll"));
        }
    }
}
