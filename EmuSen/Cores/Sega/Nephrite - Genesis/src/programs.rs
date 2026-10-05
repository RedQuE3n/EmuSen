//! The console test programs of Nephrite_Plan.md §3.1 on the Genesis, each verdict read where its source says the
//! program leaves it, before there is a picture: the BCD verifier's failure counts, the opcode-size test's verdict
//! tiles, the illegal-instruction test's background colour, and VDPFIFOTesting's records. The programs are found
//! through `EMUSEN_NEPHRITE_ROMS` (`~/.cache/emusen/probe/nephrite/roms`); without it they pass unrun.
//! Nephrite_Native.md §9 and §13 are the record.

use crate::machine::Machine;
use crate::media::Media;

pub const ROMS_VARIABLE: &str = "EMUSEN_NEPHRITE_ROMS";

fn program(path: &str) -> Option<Machine> {
    let Some(root) = std::env::var_os(ROMS_VARIABLE) else {
        eprintln!("{ROMS_VARIABLE} is not set: {path} not run");
        return None;
    };
    let image = std::fs::read(std::path::Path::new(&root).join(path)).expect("the program");
    Some(Machine::new(&image, Media::read(&image)))
}

/// Frames until `done`, at most `limit`; the frames it took.
fn run_until(m: &mut Machine, limit: u32, done: impl Fn(&Machine) -> bool) -> Option<u32> {
    for f in 1..=limit {
        m.advance();
        if done(m) {
            return Some(f);
        }
    }
    None
}

fn word(b: &[u8], a: usize) -> u16 {
    u16::from_be_bytes([b[a], b[a + 1]])
}

/// Flamewing's verifier: six longs of failures at `$FFFF00` (flags and results of ABCD, SBCD and NBCD), the report
/// written to its text buffer at `$FF0000` once NBCD, the last, is done.
#[test]
fn the_bcd_verifier_counts_no_failure() {
    let Some(mut m) = program("exodus-techdocs/bcd_verifier/bcd-verifier-u1.bin") else { return };
    let finished = |m: &Machine| {
        let ram = &m.genesis.hw.wram;
        ram[0xFEFF] == b'n' && ram[0x0090..0x00C0].iter().any(|&b| b != 0)
    };
    let frames = run_until(&mut m, 6000, finished).expect("the verifier finished");
    let ram = &m.genesis.hw.wram;
    let counts: Vec<u32> = (0..6).map(|i| u32::from_be_bytes(ram[0xFF00 + 4 * i..][..4].try_into().unwrap())).collect();
    eprintln!("BCD verifier: finished at frame {frames}; failures (flags, results) abcd {:?} sbcd {:?} nbcd {:?}", &counts[0..2], &counts[2..4], &counts[4..6]);
    assert_eq!(counts, [0; 6]);
}

/// realmonster's opcode sizes: for each of its two CRCs, the fourth tile of four written to VRAM from `$8000` is
/// `$66` bytes for a pass and `$99` for a failure.
#[test]
fn the_opcode_sizes_match_both_crcs() {
    let Some(mut m) = program("exodus-techdocs/opcode_sizes/m68k_opcode_sizes.bin") else { return };
    let verdict = |m: &Machine, tile: usize| {
        let v = &m.genesis.hw.vdp.vram[0x8000 + 32 * tile..][..32];
        if v.iter().all(|&b| b == 0x66) { Some(true) } else if v.iter().all(|&b| b == 0x99) { Some(false) } else { None }
    };
    let frames = run_until(&mut m, 3000, |m| verdict(m, 7).is_some()).expect("both checks reported");
    eprintln!("opcode sizes: reported at frame {frames}: lengths {:?}, skipped opcodes {:?}", verdict(&m, 3), verdict(&m, 7));
    assert_eq!((verdict(&m, 3), verdict(&m, 7)), (Some(true), Some(true)));
}

/// MacDonald's illegal-instruction test: CRAM's first entry $0E00 while it works, $00E0 when every illegal,
/// line A and line F opcode took its exception, $000E when anything else was taken.
#[test]
fn every_illegal_opcode_takes_its_exception() {
    let Some(mut m) = program("exodus-techdocs/illegal_opcodes/itest.BIN") else { return };
    let colour = |m: &Machine| word(&m.genesis.hw.vdp.cram, 0);
    let frames = run_until(&mut m, 3000, |m| matches!(colour(m), 0x00E0 | 0x000E)).expect("the test finished");
    eprintln!("illegal instructions: colour ${:04X} at frame {frames}", colour(&m));
    assert_eq!(colour(&m), 0x00E0);
}

/// Nemesis's VDPFIFOTesting, its 122 records read where the program leaves them once A has run it to its last page:
/// all pass but one of FIFO Wait States' samples (Nephrite_Native.md §13.3).
#[test]
fn vdpfifotesting_passes_all_but_one() {
    let Some(mut m) = program("exodus-techdocs/vdp_port_access/VDPFIFOTesting.bin") else { return };
    while m.frames < 3000 {
        m.pads[0] = crate::fifo_records::press(m.frames);
        m.advance();
    }
    let rs = crate::fifo_records::records(&m.genesis.hw.wram);
    let failed: Vec<&str> = rs.iter().filter(|r| !r.passed()).map(|r| r.name.as_str()).collect();
    eprintln!("VDPFIFOTesting: {} of {} pass; failing {failed:?}", rs.len() - failed.len(), rs.len());
    assert_eq!((rs.len(), failed), (122, vec!["FIFO Wait States"]));
}
