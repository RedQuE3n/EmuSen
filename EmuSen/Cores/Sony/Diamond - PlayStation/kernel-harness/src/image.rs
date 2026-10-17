//! The kernel's image: the ELF the pinned toolchain links, flattened to 512 KiB by its loadable segments' physical
//! addresses, and the rebuild that checks it against the committed bytes (Diamond_OpenKernel.md §6).

use std::path::{Path, PathBuf};
use std::process::Command;

pub const BASE: u32 = 0x1FC0_0000;
pub const SIZE: usize = 512 * 1024;

pub fn kernel_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../kernel")
}

/// The shell's crate, whose PS-EXE the kernel carries (Diamond_Shell.md §1.3).
pub fn shell_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../kernel-shell")
}

pub const SHELL_EXE: &str = "diamond-shell.exe";

/// The shell's committed PS-EXE.
pub fn committed_shell() -> Vec<u8> {
    std::fs::read(shell_dir().join(SHELL_EXE)).expect("the shell's PS-EXE")
}

pub fn committed_shell_digest() -> String {
    let text = std::fs::read_to_string(shell_dir().join(format!("{SHELL_EXE}.sha256"))).expect("diamond-shell.exe.sha256");
    text.split_whitespace().next().unwrap_or("").to_string()
}

/// Where the shell's stack starts, its own in user RAM; the kernel's shell stack is left to the kernel.
pub const SHELL_STACK: u32 = 0x801F_FF00;

/// A PS-EXE from the shell's ELF (psx-spx "CDROM File Playstation EXE and SYSTEM.CNF"): its one loadable segment's
/// bytes as the body at the segment's address, padded to 800h, the entry as PC, GP 0, the variables from __bss_start to
/// __bss_end as the region Exec zeroes, and the shell's own stack.
pub fn psexe(elf: &[u8]) -> Result<Vec<u8>, String> {
    if elf.len() < 52 || &elf[..4] != b"\x7FELF" || elf[4] != 1 || elf[5] != 1 {
        return Err("not a 32-bit little-endian ELF".into());
    }
    let (phoff, phentsize, phnum) = (u32_at(elf, 28) as usize, u16_at(elf, 42), u16_at(elf, 44));
    let loads: Vec<usize> = (0..phnum).map(|i| phoff + i * phentsize).filter(|&p| u32_at(elf, p) == 1 && u32_at(elf, p + 16) > 0).collect();
    let [p] = loads[..] else { return Err(format!("{} loadable segments with bytes, not one", loads.len())) };
    let (offset, vaddr, filesz) = (u32_at(elf, p + 4) as usize, u32_at(elf, p + 8), u32_at(elf, p + 16) as usize);
    let size = filesz.div_ceil(0x800) * 0x800;
    let (bss, end) = (symbol(elf, "__bss_start").ok_or("no __bss_start")?, symbol(elf, "__bss_end").ok_or("no __bss_end")?);
    let mut exe = vec![0u8; 0x800 + size];
    exe[..8].copy_from_slice(b"PS-X EXE");
    let mut put = |at: usize, v: u32| exe[at..at + 4].copy_from_slice(&v.to_le_bytes());
    put(0x10, u32_at(elf, 24));
    put(0x14, 0);
    put(0x18, vaddr);
    put(0x1C, size as u32);
    put(0x28, bss);
    put(0x2C, end - bss);
    put(0x30, SHELL_STACK);
    put(0x34, 0);
    let marker = b"Diamond open kernel shell";
    exe[0x4C..0x4C + marker.len()].copy_from_slice(marker);
    exe[0x800..0x800 + filesz].copy_from_slice(&elf[offset..offset + filesz]);
    Ok(exe)
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
    Some(rebuild_elf()?.and_then(|elf| flatten(&elf)))
}

/// The kernel's ELF as the pinned toolchain links it, over the committed shell; None when that toolchain is absent.
pub fn rebuild_elf() -> Option<Result<Vec<u8>, String>> {
    build(&kernel_dir(), "kernel-rebuild", "diamond-kernel")
}

/// The shell's ELF as the pinned toolchain links it; None when that toolchain is absent.
pub fn rebuild_shell_elf() -> Option<Result<Vec<u8>, String>> {
    build(&shell_dir(), "shell-rebuild", "diamond-shell")
}

/// The shell's PS-EXE rebuilt; None when the toolchain is absent.
pub fn rebuild_shell() -> Option<Result<Vec<u8>, String>> {
    Some(rebuild_shell_elf()?.and_then(|elf| psexe(&elf)))
}

fn build(dir: &Path, target_name: &str, binary: &str) -> Option<Result<Vec<u8>, String>> {
    let mut cargo = cargo()?;
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join(target_name);
    let output = cargo
        .current_dir(dir)
        .args(["build", "--release", "-Z", "build-std=core", "--target", "mipsel-sony-psx", "--target-dir"])
        .arg(&target)
        .output();
    Some(match output {
        Err(e) => Err(format!("cargo did not run: {e}")),
        Ok(o) if !o.status.success() => Err(String::from_utf8_lossy(&o.stderr).into_owned()),
        Ok(_) => std::fs::read(target.join(format!("mipsel-sony-psx/release/{binary}.exe"))).map_err(|e| e.to_string()),
    })
}

fn section_headers(elf: &[u8]) -> impl Iterator<Item = [u32; 10]> + '_ {
    let (shoff, shentsize, shnum) = (u32_at(elf, 32) as usize, u16_at(elf, 46), u16_at(elf, 48));
    (0..shnum).map(move |i| core::array::from_fn(|k| u32_at(elf, shoff + i * shentsize + 4 * k)))
}

/// A symbol's value from the ELF's symbol table.
pub fn symbol(elf: &[u8], name: &str) -> Option<u32> {
    let headers: Vec<[u32; 10]> = section_headers(elf).collect();
    let table = headers.iter().find(|h| h[1] == 2)?;
    let strings = headers[table[6] as usize][4] as usize;
    (0..table[5] as usize / 16).map(|k| table[4] as usize + 16 * k).find_map(|e| {
        let at = strings + u32_at(elf, e) as usize;
        let end = at + elf[at..].iter().position(|&b| b == 0)?;
        (&elf[at..end] == name.as_bytes()).then(|| u32_at(elf, e + 4))
    })
}

/// The words of the ELF's bytes from `from` to `to` by their run-time addresses, in the section holding them.
pub fn words(elf: &[u8], from: u32, to: u32) -> Option<Vec<u32>> {
    let h = section_headers(elf).find(|h| h[1] == 1 && h[3] <= from && to <= h[3] + h[5])?;
    let start = (h[4] + from - h[3]) as usize;
    Some((0..(to - from) as usize / 4).map(|i| u32_at(elf, start + 4 * i)).collect())
}
