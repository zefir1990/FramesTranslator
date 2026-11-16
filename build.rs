use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const CPP_RUNTIME_LIBRARIES: [&str; 3] = [
    "libstdc++-6.dll",
    "libgcc_s_seh-1.dll",
    "libwinpthread-1.dll",
];

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
    copy_library_to_target_directories(&sdl_dll);
    copy_cpp_runtime_libraries();
}

fn copy_library_to_target_directories(library: &Path) {
    let out_directory = PathBuf::from(env::var("OUT_DIR").unwrap());
    let Some(profile_directory) = out_directory.ancestors().nth(3) else {
        return;
    };
    let Some(file_name) = library.file_name() else {
        return;
    };
    for target_directory in [
        profile_directory.to_path_buf(),
        profile_directory.join("deps"),
    ] {
        if fs::create_dir_all(&target_directory).is_ok() {
            let _ = fs::copy(library, target_directory.join(file_name));
        }
    }
}

fn copy_cpp_runtime_libraries() {
    let Ok(compiler) = cc::Build::new().cpp(true).try_get_compiler() else {
        return;
    };
    if !compiler.is_like_gnu() {
        return;
    }
    let Some(compiler_directory) = resolve_compiler_directory(compiler.path()) else {
        return;
    };
    for library_name in CPP_RUNTIME_LIBRARIES {
        let library = compiler_directory.join(library_name);
        if library.is_file() {
            copy_library_to_target_directories(&library);
        }
    }
}

fn resolve_compiler_directory(compiler: &Path) -> Option<PathBuf> {
    if let Some(directory) = compiler
        .parent()
        .filter(|directory| !directory.as_os_str().is_empty())
    {
        return Some(directory.to_path_buf());
    }
    let executable_name = if compiler.extension().is_none() {
        compiler.with_extension("exe")
    } else {
        compiler.to_path_buf()
    };
    let path_variable = env::var_os("PATH")?;
    env::split_paths(&path_variable).find(|directory| directory.join(&executable_name).is_file())
}
