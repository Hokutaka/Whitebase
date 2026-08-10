use std::env;
use std::path::{Path, PathBuf};

const WINDOWS_TARGET: &str = "x86_64-pc-windows-msvc";
const LINUX_TARGET: &str = "x86_64-unknown-linux-gnu";

fn main() {
    let target = env::var("TARGET").expect("TARGET must be provided by Cargo");
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be provided by Cargo"),
    );
    let configuration = cargo_configuration();

    match target.as_str() {
        WINDOWS_TARGET => link_windows_library(&manifest_dir, configuration),
        LINUX_TARGET => link_linux_library(&manifest_dir, configuration),
        _ => {
            println!("cargo:warning=whitebase-asm-adapter is unavailable for target {target}");
        }
    }

    println!("cargo:rerun-if-env-changed=PROFILE");
}

fn cargo_configuration() -> &'static str {
    let profile = env::var("PROFILE").expect("PROFILE must be provided by Cargo");

    match profile.as_str() {
        "debug" => "Debug",
        "release" => "Release",
        other => panic!("unsupported Cargo profile: {other}"),
    }
}

fn link_windows_library(manifest_dir: &Path, configuration: &str) {
    let library_directory = repository_root(manifest_dir)
        .join("native")
        .join("Whitebase.Cpp")
        .join("x64")
        .join(configuration);
    let library_file = library_directory.join("Whitebase.Assembly.lib");

    require_library(
        &library_file,
        &format!("Build Whitebase.Assembly for x64/{configuration} first."),
    );

    println!(
        "cargo:rustc-link-search=native={}",
        library_directory.display()
    );
    println!("cargo:rustc-link-lib=static=Whitebase.Assembly");
    println!("cargo:rerun-if-changed={}", library_file.display());
}

fn link_linux_library(manifest_dir: &Path, configuration: &str) {
    let library_directory = repository_root(manifest_dir)
        .join("native")
        .join("Whitebase.Linux")
        .join("build")
        .join(configuration);
    let library_file = library_directory.join("libwhitebase_assembly.a");

    require_library(
        &library_file,
        &format!(
            "Run ./scripts/linux-native.sh {} first.",
            linux_command(configuration)
        ),
    );

    println!(
        "cargo:rustc-link-search=native={}",
        library_directory.display()
    );
    println!("cargo:rustc-link-lib=static=whitebase_assembly");
    println!("cargo:rerun-if-changed={}", library_file.display());
}

fn repository_root(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .join("..")
        .join("..")
        .join("..")
        .join("..")
}

fn require_library(library_file: &Path, instruction: &str) {
    if !library_file.exists() {
        panic!(
            "Assembly library was not found at {}. {instruction}",
            library_file.display()
        );
    }
}

fn linux_command(configuration: &str) -> &'static str {
    match configuration {
        "Debug" => "build",
        "Release" => "release",
        _ => unreachable!("validated Cargo configuration"),
    }
}
