//! The kernel's image: the ELF the pinned toolchain links, flattened to 512 KiB by its loadable segments' physical
//! addresses, and the rebuild that checks it against the committed bytes (Diamond_OpenKernel.md §6).

use std::path::{Path, PathBuf};
use std::process::Command;

pub const BASE: u32 = 0x1FC0_0000;
pub const SIZE: usize = 512 * 1024;

pub fn kernel_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../kernel")
}

/// The committed image, or the one DIAMOND_KERNEL_IMAGE names, for running the suite against a spoiled kernel.
pub fn committed() -> Vec<u8> {
    let path =
        std::env::var_os("DIAMOND_KERNEL_IMAGE").map_or_else(|| kernel_dir().join("diamond-kernel.bin"), PathBuf::from);
    std::fs::read(path).expect("the kernel's image")
}

pub fn committed_digest() -> String {
    let text =
        std::fs::read_to_string(kernel_dir().join("diamond-kernel.bin.sha256")).expect("diamond-kernel.bin.sha256");
    text.split_whitespace().next().unwrap_or("").to_string()
}

fn u16_at(b: &[u8], o: usize) -> usize {
    u16::from_le_bytes([b[o], b[o + 1]]) as usize
}

fn u32_at(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

/// Each PT_LOAD segment's file bytes at its physical address less 1FC00000h; a segment with bytes outside the ROM is
/// an error, one with none (the RAM's .bss) is skipped.
pub fn flatten(elf: &[u8]) -> Result<Vec<u8>, String> {
    if elf.len() < 52 || &elf[..4] != b"\x7FELF" || elf[4] != 1 || elf[5] != 1 {
        return Err("not a 32-bit little-endian ELF".into());
    }
    let (phoff, phentsize, phnum) = (u32_at(elf, 28) as usize, u16_at(elf, 42), u16_at(elf, 44));
    let mut image = vec![0u8; SIZE];
    for i in 0..phnum {
        let p = phoff + i * phentsize;
        let (kind, offset, paddr, filesz) =
            (u32_at(elf, p), u32_at(elf, p + 4) as usize, u32_at(elf, p + 12), u32_at(elf, p + 16) as usize);
        if kind != 1 || filesz == 0 {
            continue;
        }
        let at = paddr.wrapping_sub(BASE) as usize;
        if at + filesz > SIZE {
            return Err(format!("segment at {paddr:08X}, {filesz} bytes, lies outside the ROM"));
        }
        image[at..at + filesz].copy_from_slice(&elf[offset..offset + filesz]);
    }
    Ok(image)
}

pub fn channel() -> String {
    let text = std::fs::read_to_string(kernel_dir().join("rust-toolchain.toml")).expect("rust-toolchain.toml");
    text.lines()
        .find_map(|l| {
            l.trim()
                .strip_prefix("channel")
                .map(|v| v.trim_start_matches([' ', '=']).trim().trim_matches('"').to_string())
        })
        .expect("a channel")
}

/// A cargo that has the pinned toolchain: the project's toolchain cache, then rustup's cargo on PATH.
fn cargo() -> Option<Command> {
    let channel = channel();
    let home = std::env::var("HOME").unwrap_or_default();
    let cache = Path::new(&home).join(".cache/emusen/toolchains");
    let mut candidates = Vec::new();
    if cache.join("cargo/bin/cargo").exists() {
        candidates.push((cache.join("cargo/bin/cargo"), Some((cache.join("rustup"), cache.join("cargo")))));
    }
    candidates.push((PathBuf::from("cargo"), None));
    for (program, homes) in candidates {
        let make = || {
            let mut c = Command::new(&program);
            for (k, _) in std::env::vars() {
                if (k.starts_with("CARGO") && k != "CARGO_HOME") || k.starts_with("RUST") && k != "RUSTUP_HOME" {
                    c.env_remove(&k);
                }
            }
            if let Some((rustup, cargo)) = &homes {
                c.env("RUSTUP_HOME", rustup).env("CARGO_HOME", cargo);
                let path = std::env::var("PATH").unwrap_or_default();
                c.env("PATH", format!("{}:{path}", cargo.join("bin").display()));
            }
            c.arg(format!("+{channel}"));
            c
        };
        if make().arg("--version").output().is_ok_and(|o| o.status.success()) {
            return Some(make());
        }
    }
    None
}

/// The kernel built by the pinned toolchain and flattened; None when that toolchain is absent.
pub fn rebuild() -> Option<Result<Vec<u8>, String>> {
    let mut cargo = cargo()?;
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/kernel-rebuild");
    let output = cargo
        .current_dir(kernel_dir())
        .args(["build", "--release", "-Z", "build-std=core", "--target", "mipsel-sony-psx", "--target-dir"])
        .arg(&target)
        .output();
    Some(match output {
        Err(e) => Err(format!("cargo did not run: {e}")),
        Ok(o) if !o.status.success() => Err(String::from_utf8_lossy(&o.stderr).into_owned()),
        Ok(_) => std::fs::read(target.join("mipsel-sony-psx/release/diamond-kernel.exe"))
            .map_err(|e| e.to_string())
            .and_then(|elf| flatten(&elf)),
    })
}
