use std::{env, fs, path::PathBuf};

fn main(){
    let sdl2_lib_dir = "SDL2/lib/x64";
    let sdl2_dll_path = "SDL2/lib/x64/SDL2.dll";

    println!("cargo:rustc-link-search=native={}", sdl2_lib_dir);
    println!("cargo:rustc-link-lib=SDL2");

    let out_dir = env::var("OUT_DIR").unwrap();
    let target_dir = PathBuf::from(out_dir).ancestors().nth(4).unwrap().join(env::var("PROFILE").unwrap());

    fs::create_dir_all(&target_dir).unwrap();
    println!("target dir {}", target_dir.display());
    fs::copy(sdl2_dll_path, target_dir.join("SDL2.dll")).expect("Failed to copy dll to output directory");
}