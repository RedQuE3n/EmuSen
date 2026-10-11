//! Four of ps1-tests' TTY programs rewritten to record in RAM, assembled from the project's own part in
//! `programs/records/` and the release build 158's compiled code, carried from the local corpus named by
//! `EMUSEN_DIAMOND_PS1TESTS` at each file's `@release` line and pinned by SHA-256 (Diamond_Records.md §1.1). Without
//! the corpus nothing is built; the release's code is never kept in the tree.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

pub const CORPUS_VARIABLE: &str = "EMUSEN_DIAMOND_PS1TESTS";

/// One rewrite: its release program in the corpus, its own part, the carried words' digest, and where its calls into
/// PSn00bSDK go.
pub struct Record {
    pub name: &'static str,
    pub release: &'static str,
    pub own: &'static str,
    pub sha256: &'static str,
    pub calls: &'static [(u32, &'static str)],
}

pub const RECORDS: [Record; 4] = [
    Record {
        name: "access-time",
        release: "cpu/access-time/access-time.exe",
        own: include_str!("../../programs/records/access-time.s"),
        sha256: "05a055d9eb950cc0ebc3ce02c921135621a59539e093d8d4d6b6731d7b19c2a2",
        calls: &[(0x8001_50D4, "record"), (0x8001_5110, "critical"), (0x8001_5120, "critical")],
    },
    Record {
        name: "chopping",
        release: "dma/chopping/chopping.exe",
        own: include_str!("../../programs/records/chopping.s"),
        sha256: "7c5bce9e1eadb19ee6968d32ab5308ae41efd51be0c469681139ebf10eebbea7",
        calls: &[
            (0x8001_0BDC, "none"), (0x8001_0C10, "none"), (0x8001_0C48, "drawprim"), (0x8001_0CA8, "drawsync"),
            (0x8001_0D44, "putdisp"), (0x8001_0F1C, "putdraw"), (0x8001_1090, "gpustat"), (0x8001_10A0, "resetgraph"),
            (0x8001_1210, "finish"), (0x8001_1400, "dispmask"), (0x8001_1420, "videomode"), (0x8001_165C, "record"),
            (0x8001_1698, "none"), (0x8001_16A8, "none"), (0x8001_1BD0, "malloc"),
        ],
    },
    Record {
        name: "bandwidth",
        release: "gpu/bandwidth/bandwidth.exe",
        own: include_str!("../../programs/records/bandwidth.s"),
        sha256: "36c6d03672bac32d7bfb4377cd2f7039a5d3cf5c94ba31f8cd867d29c4f0b753",
        calls: &[
            (0x8001_0BD0, "none"), (0x8001_0C04, "none"), (0x8001_0C3C, "drawprim"), (0x8001_0C9C, "drawsync"),
            (0x8001_0D38, "putdisp"), (0x8001_0F10, "putdraw"), (0x8001_1084, "gpustat"), (0x8001_1094, "resetgraph"),
            (0x8001_1204, "finish"), (0x8001_13F4, "dispmask"), (0x8001_1414, "videomode"), (0x8001_1650, "record"),
            (0x8001_168C, "none"), (0x8001_169C, "none"), (0x8001_1D9C, "rand"),
        ],
    },
    Record {
        name: "chain-looping",
        release: "dma/chain-looping/chain-looping.exe",
        own: include_str!("../../programs/records/chain-looping.s"),
        sha256: "66a9d148d1e4419863a8bd7067ee6bacb919eaabba26fb5ecee4a228c997a4a8",
        calls: &[
            (0x8001_0D4C, "resetgraph"), (0x8001_0888, "none"), (0x8001_08BC, "none"), (0x8001_09F0, "putdisp"),
            (0x8001_0BC8, "putdraw"), (0x8001_10AC, "dispmask"), (0x8001_08F4, "drawprim"), (0x8001_0EBC, "finish"),
            (0x8001_128C, "record"), (0x8001_12C8, "none"), (0x8001_12D8, "none"),
        ],
    },
];

pub fn corpus() -> Option<PathBuf> {
    std::env::var_os(CORPUS_VARIABLE).map(PathBuf::from)
}

pub fn named(name: &str) -> &'static Record {
    RECORDS.iter().find(|r| r.name == name).expect("a rewrite of that name")
}

/// The program's source: its own part with the release's code, disassembled, at the `@release` line.
pub fn source(record: &Record, corpus: &Path) -> Result<String, String> {
    let path = corpus.join(record.release);
    let image = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let marker = record.own.lines().find(|l| l.starts_with("# @release ")).ok_or("no @release line")?;
    let mut range = marker.split_whitespace().skip(2).take(2).map(|v| u32::from_str_radix(v.trim_end_matches(':'), 16));
    let (Some(Ok(start)), Some(Ok(end))) = (range.next(), range.next()) else { return Err("a bad @release line".into()) };
    let load = u32::from_le_bytes(image.get(0x18..0x1C).ok_or("not a PS-EXE")?.try_into().unwrap());
    let at = |a: u32| 0x800 + a.wrapping_sub(load) as usize;
    let bytes = image.get(at(start)..at(end)).ok_or("the release is too short")?;
    let digest = emusen_core_conform::digest::sha256_hex(bytes);
    if digest != record.sha256 {
        return Err(format!("{}: its code is not build 158's (SHA-256 {digest})", path.display()));
    }
    let dialect = blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR));
    let mut code = String::new();
    for (i, word) in bytes.chunks_exact(4).enumerate() {
        let pc = start + 4 * i as u32;
        let w = u32::from_le_bytes(word.try_into().unwrap());
        let target = (pc.wrapping_add(4) & 0xF000_0000) | (w & 0x03FF_FFFF) << 2;
        if matches!(w >> 26, 2 | 3) && !(0x8001_0000..end).contains(&target) {
            let Some(&(_, label)) = record.calls.iter().find(|c| c.0 == target) else {
                return Err(format!("{}: a call at {pc:08X} to {target:08X} has no stand-in", record.name));
            };
            code += &format!("        {} {label}\n", if w >> 26 == 3 { "jal" } else { "j" });
        } else {
            code += &format!("        {}\n", emusen_mips::disassemble(w, pc, &dialect));
        }
    }
    Ok(record.own.replacen(marker, code.trim_end(), 1))
}
