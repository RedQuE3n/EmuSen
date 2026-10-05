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
/// all pass (Nephrite_Native.md §13.3, §14.3).
#[test]
fn vdpfifotesting_passes_every_test() {
    let Some(mut m) = program("exodus-techdocs/vdp_port_access/VDPFIFOTesting.bin") else { return };
    while m.frames < 3000 {
        m.pads[0] = crate::fifo_records::press(m.frames);
        m.advance();
    }
    let rs = crate::fifo_records::records(&m.genesis.hw.wram);
    let failed: Vec<&str> = rs.iter().filter(|r| !r.passed()).map(|r| r.name.as_str()).collect();
    eprintln!("VDPFIFOTesting: {} of {} pass; failing {failed:?}", rs.len() - failed.len(), rs.len());
    assert_eq!((rs.len(), failed), (122, Vec::<&str>::new()));
}

/// Nemesis's sprite masking and overflow test in both widths, C switching from H32 to H40 (its "Start" is read with
/// TH an input, which the pad answers with C): every one of its nine verdicts green, as on his console's photographs
/// (Nephrite_Native.md §14.3).
#[test]
fn the_sprite_masking_test_passes_in_both_widths() {
    let Some(mut m) = program("exodus-techdocs/sprite_masking/SpriteMaskingTestRom.bin") else { return };
    let verdicts = |m: &Machine| {
        let mut colours = std::collections::BTreeMap::new();
        for p in m.picture.chunks(4) {
            *colours.entry((p[0], p[1], p[2])).or_insert(0) += 1;
        }
        colours
    };
    while m.frames < 120 {
        m.advance();
    }
    let h32 = verdicts(&m);
    while m.frames < 300 {
        m.pads[0] = if (150..156).contains(&m.frames) { 1 << 5 } else { 0 };
        m.advance();
    }
    let h40 = verdicts(&m);
    eprintln!("sprite masking: H32 {h32:?}; H40 {h40:?}");
    for (c, w) in [(h32, 256), (h40, 320)] {
        assert_eq!(c.keys().copied().collect::<Vec<_>>(), vec![(0, 0, 144), (0, 255, 0), (255, 255, 255)], "only the screen's blue, white and the verdicts' green");
        assert_eq!((c[&(0, 255, 0)], c.values().sum::<usize>()), (1574, w * 224));
    }
}

/// MacDonald's window bug program, its scroll stepped every frame (its `lsr.w #4` made a `nop`): each frame, the
/// fine scroll's partial column after the window shows plane A's data from 16 pixels on and the rest of the line its
/// own, and line 111, where the program turns the window off, still shows it, as the board does (Nephrite_Disputes.md
/// D-5). The window's text rows repeat every 8 lines, so the bottom half, the window off, is the reference.
#[test]
fn the_window_bug_takes_the_partial_column_from_the_next() {
    let Some(root) = std::env::var_os(ROMS_VARIABLE) else { return };
    let mut image = std::fs::read(std::path::Path::new(&root).join("exodus-techdocs/window_distortion/Window distortion bug.BIN")).expect("the program");
    let at = image.windows(4).position(|p| p == [0xE8, 0x48, 0x02, 0x40]).expect("the scroll's shift");
    image[at..at + 2].copy_from_slice(&[0x4E, 0x71]);
    let mut m = Machine::new(&image, Media::read(&image));
    let px = |m: &Machine, x: usize, y: usize| m.genesis.hw.vdp.frame.rgba[(y * crate::render::MAX_W + x) * 4..][..3].to_vec();
    for _ in 0..24 {
        m.advance();
    }
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..16 {
        m.advance();
        let text_at = (0..320).find(|&x| px(&m, x, 200) != px(&m, 300, 200)).expect("the text");
        let s = text_at - 81;
        seen.insert(s);
        assert_ne!(px(&m, 50, 111), px(&m, 50, 112), "line 111 still in the window");
        for y in 0..104 {
            for x in 112..128 {
                let from = if x < 112 + s { x + 16 } else { x };
                assert_eq!(px(&m, x, y), px(&m, from, y + 112), "scroll {s}, pixel {x}, line {y}");
            }
        }
    }
    assert_eq!(seen.len(), 16, "every fine scroll: {seen:?}");
}
