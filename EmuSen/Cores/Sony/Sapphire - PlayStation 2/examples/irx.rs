//! The EE programs that load IOP modules (Sapphire_Native.md §40, S10's question 1), written as ELFs from `tests/irx`'s
//! loader.
//!
//! `cargo run --release --example irx -- probe <out.elf>` writes the dispute step's probe (§40.3): calls to LOADFILE's
//! function 0 with the module's path, `cdrom0:\TEST.IRX;1`, at each of a series of offsets in an argument block of
//! 512 zero bytes, each reply kept in the record for a referee's RAM. `probe2 <out.elf>` writes its second part: the
//! path at offset 8 with the block's first two words set in turn, and a path naming no file. `loader <out.elf>` writes the
//! loader the 23 IOP programs run by: one call loading `cdrom0:\TEST.IRX;1` as D-480 found the layout.

#[path = "../tests/irx/mod.rs"]
mod irx;

/// The module's path on the synthetic disc, as the IOP's file system names a file of the disc's root.
pub const MODULE: &str = "cdrom0:\\TEST.IRX;1";

/// The probe's calls: the path at offsets 0 to 64 in steps of 4, then 128, 252 and 256, in a block of 512 bytes.
fn probe() -> Vec<irx::Call> {
    let offsets: Vec<usize> = (0..=64).step_by(4).chain([128, 252, 256]).collect();
    offsets
        .into_iter()
        .map(|k| {
            let mut args = vec![0u8; 512];
            args[k..k + MODULE.len()].copy_from_slice(MODULE.as_bytes());
            irx::Call { function: irx::LOAD_MODULE, args, recv: 512 }
        })
        .collect()
}

/// The second part: the path at 8, as the first part found; a file the disc lacks; then each of the block's first two
/// words set to 16, to all ones and to 512, the first with a string of 15 characters at offset 260.
fn probe2() -> Vec<irx::Call> {
    let block = |path: &str, w0: u32, w1: u32, tail: Option<(usize, &str)>| {
        let mut args = vec![0u8; 512];
        args[0..4].copy_from_slice(&w0.to_le_bytes());
        args[4..8].copy_from_slice(&w1.to_le_bytes());
        args[8..8 + path.len()].copy_from_slice(path.as_bytes());
        if let Some((at, s)) = tail {
            args[at..at + s.len()].copy_from_slice(s.as_bytes());
        }
        irx::Call { function: irx::LOAD_MODULE, args, recv: 512 }
    };
    vec![
        block(MODULE, 0, 0, None),
        block("cdrom0:\\NONE.IRX;1", 0, 0, None),
        block(MODULE, 16, 0, Some((260, "abcdefghijklmno"))),
        block(MODULE, 0, 16, None),
        block(MODULE, 0xFFFF_FFFF, 0, None),
        block(MODULE, 0, 0xFFFF_FFFF, None),
        block(MODULE, 512, 0, None),
        block(MODULE, 0, 512, None),
    ]
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let calls = match a.first().map(String::as_str) {
        Some("probe") => probe(),
        Some("probe2") => probe2(),
        Some("loader") => vec![irx::module_load(MODULE)],
        _ => panic!("irx probe|probe2|loader <out.elf>"),
    };
    let words = irx::image(&calls);
    std::fs::write(&a[1], sapphire::media::program(&words, irx::ORIGIN)).expect("written");
    println!("{} calls, {} words; record at {:08X}, done word at {:08X}", calls.len(), words.len(), irx::RECORD, irx::DONE_AT);
}
