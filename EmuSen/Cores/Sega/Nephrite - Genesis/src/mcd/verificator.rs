//! mcd-verificator (krikzz, GPL-3.0) in the Sega CD's boot ROM, the 240p Test Suite's Sega CD disc in the drive, its
//! verdicts read from plane A in VRAM, where its console writes one character a word (Nephrite_Native.md §49.1). Both
//! are found through `EMUSEN_NEPHRITE_ROMS`; without it the tests pass unrun.

use crate::machine::Machine;
use crate::media::Media;

const PROGRAM: &str = "mcd-verificator/mcd-verificator/mcd-verificator.bin";
const DISC: &str = "240p-scd/240p_SegaCD_USA.iso";

/// The machine with mcd-verificator as its boot ROM and the disc in its drive.
pub fn machine() -> Option<Machine> {
    let Some(root) = std::env::var_os(crate::genesis::programs::ROMS_VARIABLE) else {
        eprintln!("{} is not set: mcd-verificator not run", crate::genesis::programs::ROMS_VARIABLE);
        return None;
    };
    let root = std::path::Path::new(&root);
    let bios = std::fs::read(root.join(PROGRAM)).expect("mcd-verificator");
    let disc = std::fs::read(root.join(DISC)).expect("the 240p Sega CD disc");
    let media = Media::read(&disc);
    let model = crate::machine::default_model(&media);
    Some(Machine::with_bios(&disc, media, model, Some(bios)))
}

/// Plane A's rows as text: 28 rows of 40 characters from VRAM `$C000`, 64 words a row.
pub fn screen(m: &Machine) -> Vec<String> {
    let vram = &m.hw().vdp.vram;
    (0..28)
        .map(|r| {
            (0..40)
                .map(|c| {
                    let b = vram[0xC000 + r * 128 + c * 2 + 1];
                    if (0x20..0x7F).contains(&b) { b as char } else { ' ' }
                })
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

/// Each test's name and its verdict: "OK", "ERROR: nn", or what the row holds after the name's dots.
pub fn verdicts(rows: &[String]) -> Vec<(String, String)> {
    rows.iter()
        .filter_map(|r| {
            let (name, rest) = r.split_once("..")?;
            Some((name.trim().to_string(), rest.trim_start_matches('.').trim().to_string()))
        })
        .collect()
}

/// Frames until the program prints "Diagnostics complete.", at most `limit`.
pub fn run(m: &mut Machine, limit: u32) -> Option<u32> {
    for f in 1..=limit {
        m.advance();
        if screen(m).iter().any(|r| r.contains("Diagnostics complete")) {
            return Some(f);
        }
    }
    None
}

/// Every test of the program's list (Nephrite_Native.md §49.1), in its order.
pub const TESTS: [&str; 18] = ["RAM CART", "COLOR CALC", "VAR TESTS", "IRQ TEST", "REG X000", "REG X002", "REG 2006", "REG X00C", "REG 8030", "CDC REGS", "PROG RAM", "WORD RAM", "WRAM PMOD", "CDC INIT", "CDC FLAGS", "CDC DMA2", "CDC DMA3", "CDC DMA1"];

#[test]
fn mcd_verificator_passes_every_test() {
    let Some(mut m) = machine() else { return };
    let done = run(&mut m, 3000);
    let rows = screen(&m);
    for r in rows.iter().filter(|r| !r.is_empty()) {
        eprintln!("| {r}");
    }
    eprintln!("finished at frame {done:?}");
    assert!(done.is_some(), "Diagnostics complete.");
    assert!(rows.iter().any(|r| r.contains("CD hardware detected at 0x00000000")), "mode 2: the program found itself at $000000");
    let v = verdicts(&rows);
    for name in TESTS {
        let got = v.iter().find(|(n, _)| n == name).map(|(_, r)| r.as_str());
        assert!(got.is_some_and(|r| r.ends_with("OK")), "{name}: {got:?}");
    }
}

/// A disc's machine with no boot ROM runs, reading the open bus where the ROM would be, and draws black.
#[test]
fn a_disc_with_no_boot_rom_runs_and_stands() {
    let mut disc = vec![0u8; 0x8000];
    disc[..16].copy_from_slice(b"SEGADISCSYSTEM  ");
    let media = Media::read(&disc);
    assert_eq!(media.system, crate::media::System::Mcd);
    let mut m = Machine::with_bios(&disc, media.clone(), crate::machine::default_model(&media), None);
    for _ in 0..30 {
        m.advance();
    }
    assert_eq!(m.frames, 30);
    assert!(m.mcd().unwrap().hw.g.reset_held, "the sub CPU stays held as at power-on");
    assert_eq!(m.hw().io.version & 0x20, 0, "the version register shows the expansion unit");
    let cart = Machine::new(&[0u8; 0x400], Media::read(&[0u8; 0x400]));
    assert_eq!(cart.hw().io.version & 0x20, 0x20, "and a cartridge's console has none");
}

/// With Sega's backup RAM cartridge plugged in, RAM CART finds it, its size, its protection and its memory.
#[test]
fn mcd_verificator_finds_the_ram_cartridge() {
    let Some(mut m) = machine() else { return };
    m.mcd_mut().unwrap().hw.cart = Some(crate::mcd::RamCart::new(crate::mcd::RamCart::SEGA));
    let mut row = None;
    for _ in 0..600 {
        m.advance();
        row = verdicts(&screen(&m)).into_iter().find(|(n, _)| n == "RAM CART").map(|(_, r)| r);
        if row.as_deref().is_some_and(|r| r.contains("OK") || r.contains("ERROR")) {
            break;
        }
    }
    assert_eq!(row.as_deref(), Some("OK"));
}
