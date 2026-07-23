use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").expect("TARGET must be provided by Cargo");

    if target != "x86_64-pc-windows-msvc" {
        panic!(
            "whitebase-asm-adapter currently supports only \
             x86_64-pc-windows-msvc; target was {target}"
        );
    }

    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be provided by Cargo"),
    );

    let profile = env::var("PROFILE").expect("PROFILE must be provided by Cargo");

    let configuration = match profile.as_str() {
        "debug" => "Debug",
        "release" => "Release",
        other => panic!("unsupported Cargo profile: {other}"),
    };

    let library_directory = manifest_dir
        .join("..")
        .join("..")
        .join("native")
        .join("Whitebase.Cpp")
        .join("x64")
        .join(configuration);

    let library_file = library_directory.join("Whitebase.Assembly.lib");

    if !library_file.exists() {
        panic!(
            "Assembly library was not found at {}. \
             Build Whitebase.Assembly for x64/{configuration} first.",
            library_file.display()
        );
    }

    println!(
        "cargo:rustc-link-search=native={}",
        library_directory.display()
    );
    println!("cargo:rustc-link-lib=static=Whitebase.Assembly");

    println!("cargo:rerun-if-changed={}", library_file.display());
    println!("cargo:rerun-if-env-changed=PROFILE");
}
