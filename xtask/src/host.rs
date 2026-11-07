//! The machine the build runs on, and the file names a platform gives a library or a program.

use std::env::consts::{ARCH, OS};

/// Rust's name for this machine's target, as the PGO profiles and the reuse cache name it.
pub fn rust_target() -> String {
    match OS {
        "windows" => format!("{ARCH}-pc-windows-msvc"),
        "macos" => format!("{ARCH}-apple-darwin"),
        _ => format!("{ARCH}-unknown-linux-gnu"),
    }
}

/// .NET's portable name for this machine (linux-x64), under which a prebuilt folder and the chdman toolchain keep files.
pub fn portable_rid() -> String {
    let os = match OS {
        "windows" => "win",
        "macos" => "osx",
        _ => "linux",
    };
    let arch = match ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    format!("{os}-{arch}")
}

/// The library file `name` is on the platform `rid` names (win-x64, osx-arm64, linux-x64).
pub fn library_file(name: &str, rid: &str) -> String {
    if rid.starts_with("win") {
        format!("{name}.dll")
    } else if rid.starts_with("osx") {
        format!("lib{name}.dylib")
    } else {
        format!("lib{name}.so")
    }
}

/// The program file `name` is on the platform `rid` names.
pub fn program_file(name: &str, rid: &str) -> String {
    if rid.starts_with("win") {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

pub fn is_linux() -> bool {
    OS == "linux"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_follow_the_platform() {
        assert_eq!(library_file("marsrt", "linux-x64"), "libmarsrt.so");
        assert_eq!(library_file("marsrt", "win-x64"), "marsrt.dll");
        assert_eq!(library_file("marsrt", "osx-arm64"), "libmarsrt.dylib");
        assert_eq!(program_file("nano", "win-x64"), "nano.exe");
        assert_eq!(program_file("nano", "linux-x64"), "nano");
    }
}
