extern crate bindgen;

use std::path::PathBuf;
use std::{env, fs};

fn main() {
    // We need to find the right header to include - with function declarations,
    // e.g. inotify_init1
    let include_dir = find_header_dir();
    println!("cargo:warning=Including {}", include_dir);
    let bindings = bindgen::Builder::default()
        .header("c/headers_wrapper.h")
        .clang_arg(format!("-I{}", include_dir))
        .generate()
        .expect("Unable to generate bindings");

    let out_file_name = "bindings.rs";
    let out_path = PathBuf::from(env::current_dir().unwrap())
        .join("src")
        .join(out_file_name);
    println!(
        "cargo:warning=Binding C system lib, output in: {}",
        out_path
            .as_path()
            .to_string_lossy()
    );

    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}

fn find_header_dir() -> String {
    // First check where are the inotify.h headers
    use walkdir::WalkDir;
    let root_dir = "/usr";
    let file_name = "inotify.h";

    for entry in WalkDir::new(root_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry
            .path()
            .to_string_lossy()
            .contains("sys")
        {
            continue;
        }
        if entry.file_name() == file_name {
            // Check if contains the func declarations
            let contents =
                fs::read_to_string(entry.path()).expect("Failed to read the header file");
            if contents.contains("inotify_init") {
                return entry
                    .path()
                    .to_string_lossy()
                    .strip_suffix(file_name)
                    .unwrap()
                    .to_string();
            }
        }
    }
    String::new()
}
