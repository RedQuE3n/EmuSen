//! Conformance C1-C25 over 120 frames against the seven active native cores, the libretro host and the brokered proxy.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use emusen_core_conform::{Options, run};

static BUILD_LOCK: Mutex<()> = Mutex::new(());
static RUNNER: OnceLock<PathBuf> = OnceLock::new();

fn cores_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn target_root() -> PathBuf {
    let base = if let Some(dir) = std::env::var_os("CARGO_TARGET_DIR") {
        PathBuf::from(dir)
    } else {
        let exe = std::env::current_exe().expect("test executable path");
        exe.parent().unwrap().parent().unwrap().parent().unwrap().to_path_buf()
    };
    base.join("conform-artifacts")
}

fn active_profile_name() -> String {
    let exe = std::env::current_exe().expect("test executable path");
    exe.parent().unwrap().parent().unwrap().file_name().unwrap().to_string_lossy().into_owned()
}

fn build_artifact(rel_crate: &str, cargo_args: &[&str], subdir: Option<&str>, file_name: &str) -> PathBuf {
    let _guard = BUILD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let crate_dir = cores_root().join(rel_crate);
    let manifest = std::fs::read_to_string(crate_dir.join("Cargo.toml")).expect("crate Cargo.toml");
    let want = active_profile_name();
    let (profile_args, out_dir): (Vec<&str>, &str) = if want == "test-fast" && manifest.contains("[profile.test-fast]") {
        (vec!["--profile", "test-fast"], "test-fast")
    } else if (want == "test-checked" || want == "test-fast") && manifest.contains("[profile.test-checked]") {
        (vec!["--profile", "test-checked"], "test-checked")
    } else if want == "release" || want == "test-checked" || want == "test-fast" {
        (vec!["--release"], "release")
    } else {
        (Vec::new(), "debug")
    };
    let target = target_root();
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = Command::new(cargo)
        .current_dir(&crate_dir)
        .env("CARGO_TARGET_DIR", &target)
        .args(["build", "--quiet", "-j", "4"])
        .args(&profile_args)
        .args(cargo_args)
        .arg("--target-dir")
        .arg(&target)
        .status()
        .expect("cargo build runs");
    assert!(status.success(), "{rel_crate} build failed: {status}");
    let mut out = target.join(out_dir);
    if let Some(s) = subdir {
        out = out.join(s);
    }
    out.join(file_name)
}

fn lib_filename(stem: &str) -> String {
    if cfg!(windows) {
        format!("{stem}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{stem}.dylib")
    } else {
        format!("lib{stem}.so")
    }
}

fn run_conform(name: &str, library: &Path, image: Vec<u8>) -> Vec<(String, Vec<String>)> {
    let report = run(library, &Options { self_delimiting: false, image: image.into(), frames: 120, settings: String::new(), files: Vec::new(), script: None, on_disk: None });
    let mut failures = Vec::new();
    let mut evidences = Vec::new();
    for case in &report.cases {
        evidences.push((case.id.to_owned(), case.evidence.clone()));
        if !case.passed {
            failures.push(format!("{} {}: {:?}", case.id, case.name, case.evidence));
        }
    }
    assert!(failures.is_empty(), "{name} conformance failures:\n{}", failures.join("\n"));
    evidences
}

fn verify_native(rel_crate: &str, lib_stem: &str, image: Vec<u8>) -> Vec<(String, Vec<String>)> {
    let lib = build_artifact(rel_crate, &["--lib"], None, &lib_filename(lib_stem));
    run_conform(lib_stem, &lib, image)
}

fn nes() -> Vec<u8> {
    let mut prg = vec![0u8; 0x4000];
    prg[..3].copy_from_slice(&[0x4C, 0x00, 0xC0]);
    for v in [0x3FFA, 0x3FFC, 0x3FFE] {
        prg[v..v + 2].copy_from_slice(&0xC000u16.to_le_bytes());
    }
    let mut rom = b"NES\x1a\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    rom.extend(prg);
    rom.extend(vec![0u8; 0x2000]);
    rom
}

fn gb() -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0x100..0x104].copy_from_slice(&[0x00, 0xC3, 0x50, 0x01]);
    rom[0x134..0x13F].copy_from_slice(b"CONFORM GB ");
    rom[0x150..0x152].copy_from_slice(&[0x18, 0xFE]);
    let sum = rom[0x134..0x14D].iter().fold(0u8, |s, &b| s.wrapping_sub(b).wrapping_sub(1));
    rom[0x14D] = sum;
    rom
}

fn snes() -> Vec<u8> {
    let mut rom = vec![0u8; 0x20000];
    rom[..5].copy_from_slice(&[0x78, 0x18, 0xFB, 0x80, 0xFE]);
    rom[0x7FC0..0x7FD5].copy_from_slice(b"CONFORM SNES         ");
    rom[0x7FD5..0x7FD9].copy_from_slice(&[0x20, 0x00, 0x07, 0x00]);
    rom[0x7FFC..0x7FFE].copy_from_slice(&0x8000u16.to_le_bytes());
    rom[0x7FDC..0x7FE0].copy_from_slice(&[0xFF, 0xFF, 0x00, 0x00]);
    let sum = rom.iter().fold(0u16, |s, &b| s.wrapping_add(u16::from(b)));
    rom[0x7FDC..0x7FDE].copy_from_slice(&(!sum).to_le_bytes());
    rom[0x7FDE..0x7FE0].copy_from_slice(&sum.to_le_bytes());
    rom
}

fn sms() -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[..3].copy_from_slice(&[0xF3, 0x18, 0xFE]);
    rom[0x7FF0..0x7FF8].copy_from_slice(b"TMR SEGA");
    rom[0x7FFF] = 0x4C;
    let sum = rom[..0x7FF0].iter().fold(0u16, |s, &b| s.wrapping_add(u16::from(b)));
    rom[0x7FFA..0x7FFC].copy_from_slice(&sum.to_le_bytes());
    rom
}

fn genesis() -> Vec<u8> {
    let mut rom = vec![0u8; 0x20000];
    rom[0..4].copy_from_slice(&0x00FF_FE00u32.to_be_bytes());
    rom[4..8].copy_from_slice(&0x0000_0200u32.to_be_bytes());
    rom[0x100..0x110].copy_from_slice(b"SEGA MEGA DRIVE ");
    rom[0x120..0x12B].copy_from_slice(b"CONFORM MD ");
    rom[0x1F0..0x1F3].copy_from_slice(b"JUE");
    rom[0x200..0x204].copy_from_slice(&[0x4E, 0x71, 0x60, 0xFC]);
    rom
}

fn n64() -> Vec<u8> {
    let mut rom = vec![0u8; 0x10_0000];
    rom[0..4].copy_from_slice(&0x8037_1240u32.to_be_bytes());
    rom[8..12].copy_from_slice(&0x8000_0400u32.to_be_bytes());
    rom[0x20..0x2B].copy_from_slice(b"CONFORM N64");
    rom[0x3E] = b'E';
    for (i, w) in [0x1000_FFFFu32, 0].into_iter().enumerate() {
        rom[0x40 + i * 4..0x44 + i * 4].copy_from_slice(&w.to_be_bytes());
    }
    rom
}

fn psx() -> Vec<u8> {
    let mut exe = vec![0u8; 0x1000];
    exe[..8].copy_from_slice(b"PS-X EXE");
    for (at, v) in [(0x10, 0x8001_0000u32), (0x18, 0x8001_0000), (0x1C, 0x800), (0x30, 0x801F_FFF0)] {
        exe[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
    let region = b"Sony Computer Entertainment Inc. for North America area";
    exe[0x4C..0x4C + region.len()].copy_from_slice(region);
    for (i, w) in [0x1000_FFFFu32, 0].into_iter().enumerate() {
        exe[0x800 + i * 4..0x804 + i * 4].copy_from_slice(&w.to_le_bytes());
    }
    exe
}

fn libretro_layout(name: &str, entry_lib: &str, built_lib: &Path, extra_proxy: bool) -> (PathBuf, PathBuf) {
    let root = target_root().join("real-cores-libretro").join(format!("{name}-{}", std::process::id()));
    let entry = root.join("libretro").join("testcore");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&entry).unwrap();
    let suffix = if cfg!(windows) { "dll" } else if cfg!(target_os = "macos") { "dylib" } else { "so" };
    let host_lib = build_artifact("Libretro/emusen-libretro-host", &["--lib"], None, &lib_filename("emusen_libretro_host"));
    let core_lib = build_artifact("Libretro/emusen-libretro-testcore", &["--lib"], None, &lib_filename("emusen_libretro_testcore"));
    std::fs::copy(&host_lib, entry.join(format!("emusen-libretro-host.{suffix}"))).unwrap();
    std::fs::copy(&core_lib, entry.join(format!("testcore_libretro.{suffix}"))).unwrap();
    std::fs::write(entry.join("testcore_libretro.info"), "display_name = \"EmuSen - Test Core\"\ncorename = \"TestCore\"\nlicense = \"GPLv3\"\n").unwrap();
    let target = entry.join(format!("{entry_lib}.{suffix}"));
    if extra_proxy {
        std::fs::copy(built_lib, &target).unwrap();
        let runner = RUNNER.get_or_init(|| {
            let bin = if cfg!(windows) { "emusen-core-runner.exe" } else { "emusen-core-runner" };
            build_artifact("Shared/emusen-core-broker", &["--bin", "emusen-core-runner"], None, bin)
        });
        unsafe { std::env::set_var("EMUSEN_CORE_RUNNER", runner) };
    }
    (root, target)
}

#[test]
fn moonrt_conforms() {
    verify_native("Nintendo/MoonRT - NES", "moonrt", nes());
}

#[test]
fn mercuryrt_conforms() {
    verify_native("Nintendo/MercuryRT - GB", "mercuryrt", gb());
}

#[test]
fn venusrt_conforms() {
    verify_native("Nintendo/VenusRT - SNES", "venusrt", snes());
}

#[test]
fn marsrt_conforms() {
    verify_native("Nintendo/MarsRT - N64", "marsrt", n64());
}

#[test]
fn endou_conforms() {
    verify_native("Sega/Endou - Master System", "endou", sms());
}

#[test]
fn nephrite_conforms() {
    verify_native("Sega/Nephrite - Genesis", "nephrite", genesis());
}

#[test]
fn libretro_proxy_with_testcore_conforms() {
    let proxy_lib = build_artifact("Libretro/emusen-libretro-proxy", &["--lib"], None, &lib_filename("emusen_libretro_proxy"));
    let (root, entry) = libretro_layout("proxy", "emusen-core-broker", &proxy_lib, true);
    run_conform("emusen-libretro-proxy", &entry, b"TCG1 a test game".to_vec());
    let _ = std::fs::remove_dir_all(&root);
}

mod vulkan_gpu {
    use super::*;

    #[test]
    fn diamond_conforms() {
        let evidences = verify_native("Sony/Diamond - PlayStation/v1", "diamond", psx());
        let c20 = evidences.into_iter().find(|(id, _)| id == "C20").expect("C20 case");
        assert!(!c20.1.iter().any(|e| e.contains("not lent a device")), "Diamond C20 must test Vulkan device lending: {:?}", c20.1);
    }
}
