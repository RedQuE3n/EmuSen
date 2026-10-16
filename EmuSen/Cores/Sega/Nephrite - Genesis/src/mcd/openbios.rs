//! The open BIOS (Nephrite_OpenBios.md) running the 240p Test Suite's Sega CD disc, found through
//! `EMUSEN_NEPHRITE_ROMS`; without it the test passes unrun.

use crate::machine::Machine;
use crate::media::Media;

const DISC: &str = "240p-scd/240p_SegaCD_USA.iso";
/// Where the sub CPU's program keeps a bit for each function `_CDBIOS` was called with (`bios/sub.s`, CALLSEEN).
const CALLSEEN: usize = 0x5644;

fn suite() -> Option<Machine> {
    let Some(root) = std::env::var_os(crate::genesis::programs::ROMS_VARIABLE) else {
        eprintln!("{} is not set: the open BIOS's disc not run", crate::genesis::programs::ROMS_VARIABLE);
        return None;
    };
    let disc = std::fs::read(std::path::Path::new(&root).join(DISC)).expect("the 240p Sega CD disc");
    let media = Media::read(&disc);
    let model = crate::machine::default_model(&media);
    Some(Machine::with_bios(&disc, media, model, Some(super::bios::OPEN_BIOS.to_vec())))
}

/// Whether `_CDBIOS` was called with function `f`.
fn called(m: &Machine, f: u16) -> bool {
    let k = if f >= 0x80 { f as usize - 0x80 + 40 } else { f as usize };
    m.mcd().unwrap().hw.prg[CALLSEEN + k / 8] & (1 << (k % 8)) != 0
}

// The boot on both CPUs, the drive's TOC, the data reads and `$0368` (D-41 to D-44): the suite's menu runs, its own V-INT handler in Word RAM.
#[test]
fn the_open_bios_boots_the_240p_suite_to_its_menu() {
    let Some(mut m) = suite() else { return };
    for _ in 0..1200 {
        m.advance();
    }
    let pc = m.cpu().regs.pc;
    assert!((0x20_0000..0x24_0000).contains(&pc), "the main program runs from Word RAM, at {pc:06X}");
    let w = &m.hw().wram;
    let vint = u32::from_be_bytes([w[0xFD08], w[0xFD09], w[0xFD0A], w[0xFD0B]]);
    assert!((0x20_0000..0x24_0000).contains(&vint), "its V-INT handler is set through $0368: {vint:06X}");
    assert!(m.hw().vdp.regs[1] & 0x40 != 0, "its picture is on");
    for f in [0x10, 0x20, 0x80, 0x81, 0x89, 0x8A, 0x8B, 0x8C, 0x8D] {
        assert!(called(&m, f), "the suite called function {f:02X}");
    }
}
