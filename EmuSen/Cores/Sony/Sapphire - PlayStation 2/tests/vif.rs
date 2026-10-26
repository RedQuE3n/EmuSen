//! Stage S14's gate (Sapphire_Native.md §28): the project's programs on VIF0 and VIF1, run on the interpreter with the
//! board and graded by the records `s14::manual` gives from the EE User's Manual's chapter 6; ps2autotests' `dma/vif`
//! console texts reproduced on the VIF directly, in the shape their lines show; and what no program can show a
//! referee (PATH2's stand-in, the reversed FIFO, the DMAC waiting on a full FIFO), checked on the units directly.

mod s6;
mod s11;
mod s14;

use sapphire::board::{Board, System};
use sapphire::clock;
use sapphire::ee::Bus;
use sapphire::vif::{self, reg};

use s6::{DONE, DONE_AT, ORIGIN, Program, RECORD};
use s14::console;

fn assemble(source: &str, origin: u32) -> Vec<u32> {
    emusen_mips::assemble(source, origin, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}"))
}

/// Runs a program after `setup`, and returns the system with every record word graded.
fn grade_with(p: Program, setup: impl FnOnce(&mut System)) -> System {
    let name = p.name;
    let (source, expected) = p.finish();
    let mut sys = System::new(ORIGIN);
    sys.board.map.load(ORIGIN, &assemble(&source, ORIGIN));
    setup(&mut sys);
    let limit = 80_000_000;
    let n = sys.run(limit, &mut |s| s.peek(DONE_AT, 4) as u32 == DONE).unwrap_or_else(|h| panic!("{h:?}"));
    assert!(n < limit, "{name}: no done word in {limit} instructions, pc {:08x}", sys.ee.pc);
    let mut wrong = Vec::new();
    for (i, want) in expected.iter().enumerate() {
        let got = sys.peek(RECORD + 4 * i as u32, 4) as u32;
        if let Some(w) = want
            && *w != got
        {
            wrong.push(format!("word {i} (record {}): {got:08x}, expected {w:08x}", i / 2));
        }
    }
    assert!(wrong.is_empty(), "{name}: {} of {} words differ, first {:?}", wrong.len(), expected.len(), &wrong[..wrong.len().min(8)]);
    sys
}

fn grade(p: Program) -> System {
    grade_with(p, |_| {})
}

#[test]
fn every_unpack_format_equals_section_6_3_4() {
    grade(s14::unpack());
}

#[test]
fn the_mask_and_the_addition_modes_equal_sections_6_3_6_and_6_3_7() {
    grade(s14::mask());
}

#[test]
fn skipping_and_filling_writes_equal_section_6_3_5() {
    grade(s14::cycle());
}

#[test]
fn mpg_and_the_mscal_family_start_and_wait_for_the_vus() {
    grade(s14::micro_program());
}

#[test]
fn vif1_s_double_buffer_equals_section_6_3_8() {
    grade(s14::dbuf());
}

#[test]
fn the_registers_the_interrupt_stall_fbrst_and_the_error_masks() {
    grade(s14::regs());
}

/// DIRECT's and DIRECTHL's packets reach the GS through PATH2 in order (each a LABEL, whose ID the program reads back
/// from SIGLBLID), MSKPATH3 reaches the GIF, and the FLUSH family and MSCALF go on once the paths are idle (from S15
/// the GIF's own, Sapphire_Native.md §33).
#[test]
fn direct_reaches_path2_and_the_flush_family_waits_for_the_paths() {
    let mut sys = grade(s14::direct());
    let label = |s: &mut System, record: u32| s.peek(RECORD + 8 * record, 4) as u32 & 0xFFFF;
    assert_eq!([label(&mut sys, 2), label(&mut sys, 5), label(&mut sys, 13)], [0x1234, 0x5678, 0x9ABC], "each DIRECT's LABEL in turn");
    let stat = sys.board.gif.status(sys.board.now, false);
    assert_eq!(stat & sapphire::gif::stat::M3P, 0, "MSKPATH3 set and then cleared");
}

/// A board driven through the EE's map alone, as the console texts' programs drive it.
struct Rig {
    board: Board,
    spram: Vec<u8>,
    t: u64,
}

impl Rig {
    fn new() -> Rig {
        Rig { board: Board::new(clock::NTSC), spram: vec![0; 16 << 10], t: 0 }
    }

    fn run(&mut self, cycles: u64) {
        self.t += cycles;
        self.board.run_to(self.t, &mut self.spram);
    }

    fn reg(&mut self, n: usize, off: u32) -> u32 {
        self.board.read(vif::REGS[n] + off, 4) as u32
    }

    fn set(&mut self, n: usize, off: u32, v: u32) {
        self.board.write(vif::REGS[n] + off, 4, u128::from(v));
        self.run(2);
    }

    /// Words into VIF `n`'s FIFO, padded with NOPs to a qword, then time for the VIF to take them.
    fn send(&mut self, n: usize, words: &[u32]) {
        for c in words.chunks(4) {
            let q = (0..4).rev().fold(0u128, |a, i| a << 32 | u128::from(c.get(i).copied().unwrap_or(0)));
            self.board.write(vif::FIFO[n], 16, q);
            self.run(4);
        }
        self.run(4_000);
    }

    fn qword(&mut self, n: usize, a: u32) -> [u32; 4] {
        let base = [0x1100_4000, 0x1100_C000][n] + 16 * a;
        core::array::from_fn(|f| self.board.read(base + 4 * f as u32, 4) as u32)
    }

    fn fill(&mut self, n: usize, a: u32, v: u32) {
        let base = [0x1100_4000, 0x1100_C000][n] + 16 * a;
        self.board.write(base, 16, u128::from(v) * 0x0000_0001_0000_0001_0000_0001_0000_0001);
    }

    fn reset(&mut self, n: usize) {
        self.set(n, reg::FBRST, 1);
        self.set(n, reg::ERR, 6);
    }
}

/// The elements 88h, 99h, AAh, BBh (or their 16- and 32-bit widenings) in a format, packed as §6.3.4 draws it.
fn console_data(f: u32) -> Vec<u32> {
    let (vn, vl) = (f >> 2 & 3, f & 3);
    if vl == 3 {
        return vec![0xFFFF];
    }
    let bits = 32 >> vl;
    let mut stream: Vec<u64> = Vec::new();
    for i in 0..=vn {
        let byte = [0x88u64, 0x99, 0xAA, 0xBB][i as usize];
        stream.push(byte * match bits {
            32 => 0x0101_0101,
            16 => 0x0101,
            _ => 1,
        });
    }
    let mut words = vec![0u32; ((bits * (vn + 1)) as usize).div_ceil(32)];
    for (i, e) in stream.iter().enumerate() {
        let bit = i * bits as usize;
        words[bit / 32] |= (*e as u32) << (bit % 32);
    }
    words
}

/// `dma/vif/unpack`'s lines: each format's first vector, with a field V3 leaves indeterminate written 0 over a memory
/// that held something else (D-180), and the "No data" lines' VPS, CODE and NUM, the VIFcode the last word of its
/// qword.
#[test]
fn the_unpack_console_text_holds() {
    for n in 0..2 {
        let mut rig = Rig::new();
        rig.reset(n);
        for (f, usn, want) in console::UNPACK {
            rig.fill(n, 5, 0x5A5A_5A5A);
            let mut w = vec![0x01 << 24 | 0x0101, (0x60 | f) << 24 | 1 << 16 | u32::from(usn) << 14 | 5];
            w.extend(console_data(f));
            rig.send(n, &w);
            assert_eq!(rig.qword(n, 5), want, "VIF{n} format {f:x} usn {usn}");
        }
        for (num, before, after, code, num0, num1) in console::NO_DATA {
            rig.reset(n);
            assert_eq!(rig.reg(n, reg::STAT) & 3, before);
            rig.send(n, &[0, 0, 0, 0x62 << 24 | num << 16]);
            assert_eq!(rig.reg(n, reg::STAT) & 3, after, "VIF{n} NUM {num}");
            assert_eq!(rig.reg(n, reg::CODE), code);
            assert_eq!(rig.reg(n, reg::NUM), [num0, num1][n], "VIF{n} NUM {num}");
        }
    }
}

/// `dma/vif/unpack`'s "Set 4096 bytes" line on both VIFs, its packet as the program's source builds it (read at S10):
/// Col FFEEDDCCh, BBAA9988h, 77665544h, 33221100h, MASK AAAAAAAAh (every field from Col), CL 1 WL 0, and an S-8 UNPACK
/// with the m bit and NUM 256 in a packet of four qwords. WL 0 counts as 256, so the cycle fills: 256 qwords are written
/// from one element, the first from Col 0 and the 256th, in the fourth cycle, from Col 3 (Sapphire_Disputes.md D-181).
#[test]
fn the_unpack_set_line_holds() {
    for n in 0..2 {
        let mut rig = Rig::new();
        rig.reset(n);
        for a in 0..256 {
            rig.fill(n, a, 0xFFFF_FFFF);
        }
        let mut w = vec![0x31 << 24, 0xFFEE_DDCC, 0xBBAA_9988, 0x7766_5544, 0x3322_1100, 0x20 << 24, 0xAAAA_AAAA, 0x01 << 24 | 1, 0x72 << 24];
        w.resize(16, 0);
        rig.send(n, &w);
        let (first, last) = (rig.qword(n, 0), rig.qword(n, 255));
        assert_eq!([first[0], first[1], last[2], last[3]], [0xFFEE_DDCC, 0xFFEE_DDCC, 0x3322_1100, 0x3322_1100], "VIF{n}");
    }
}

/// `dma/vif/stmod`'s eight cases on both VIFs: Row 1000h, MODE 0 to 3, V4-32 of 0 to 15 with CL = WL = 4, without the
/// m bit and with the diagonal mask.
#[test]
fn the_stmod_console_text_holds() {
    for n in 0..2 {
        let mut rig = Rig::new();
        rig.reset(n);
        for (k, (name, qwords, row, mode)) in console::STMOD.iter().enumerate() {
            let m = if k % 2 == 1 { 0x10 } else { 0 };
            let mut w = vec![0x01 << 24 | 0x0404, 0x30 << 24, 0x1000, 0x1000, 0x1000, 0x1000, 0x05 << 24 | mode, 0x20 << 24, s14::DIAGONAL];
            w.push((0x6C | m) << 24 | 4 << 16);
            w.extend(0..16);
            rig.send(n, &w);
            for (a, q) in qwords.iter().enumerate() {
                assert_eq!(rig.qword(n, a as u32), *q, "VIF{n} {name}, qword {a}");
            }
            let got_row: Vec<u32> = (0..4).map(|f| rig.reg(n, reg::R0 + 16 * f)).collect();
            assert_eq!(got_row, row, "VIF{n} {name}, Row");
            assert_eq!(rig.reg(n, reg::MODE), *mode);
        }
    }
}

/// `dma/vif/stcycl`'s lines on both VIFs: 64 qwords from address 0 cleared, Col 1 to 4, every specifier 2, then an
/// S-32 UNPACK with the m bit and NUM 8 under each CL and WL: each qword shows the write cycle that wrote it, and the
/// addresses wrap at the memory's size (VIF0's 256 qwords; VIF1's 1,024 do not wrap there).
#[test]
#[allow(clippy::needless_range_loop)]
fn the_stcycl_console_text_holds() {
    for n in 0..2 {
        let mut rig = Rig::new();
        rig.reset(n);
        for &(cl, wl, digits) in console::STCYCL_VIF0.iter() {
            let digits = console::STCYCL_VIF1.iter().find(|x| n == 1 && x.0 == cl && x.1 == wl).map_or(digits, |x| x.2);
            for a in (0..64).chain(QWORDS_TAIL[n].clone()) {
                rig.fill(n, a, 0);
            }
            let mut w = vec![0x31 << 24, 1, 2, 3, 4, 0x20 << 24, 0xAAAA_AAAA, 0x01 << 24 | wl << 8 | cl, 0x70 << 24 | 8 << 16];
            let words = s14::manual::data_words(0x70, 8, cl, wl);
            w.extend((0..words).map(|i| 0x100 + i));
            rig.send(n, &w);
            assert_eq!(rig.reg(n, reg::CYCLE), wl << 8 | cl);
            let got: String = (0..64).map(|a| char::from_digit(rig.qword(n, a)[0], 10).unwrap()).collect();
            assert_eq!(got.trim_end_matches('0'), digits, "VIF{n} cl {cl} wl {wl}");
        }
    }
}

/// The qwords past the 64 the text prints that a wrapped line writes, cleared too.
const QWORDS_TAIL: [std::ops::Range<u32>; 2] = [192..256, 960..1024];

/// The DMAC waits on a full FIFO and goes on as the VIF frees it: an S-32 UNPACK of 256 words through VIF1's 16-qword
/// FIFO, which the VIF empties a qword every four writes (D-194: a bus cycle a write) while the DMAC fills it a qword
/// a bus cycle, ends when the VIF has taken all but the last 16 qwords, and every word arrives.
#[test]
fn the_dmac_waits_on_a_full_fifo() {
    let mut rig = Rig::new();
    rig.reset(1);
    let mut words = vec![0x01 << 24 | 0x0101, 0x60 << 24];
    words.extend(0..256u32);
    while words.len() % 4 != 0 {
        words.push(0);
    }
    for (i, c) in words.chunks(4).enumerate() {
        let q = (0..4).rev().fold(0u128, |a, j| a << 32 | u128::from(c[j]));
        rig.board.map.write(0x0010_0000 + 16 * i as u32, 16, q);
    }
    let qwc = words.len() as u64 / 4;
    rig.board.write(0x1000_E000, 4, 1);
    rig.board.write(0x1000_9010, 4, 0x0010_0000);
    rig.board.write(0x1000_9020, 4, u128::from(qwc));
    rig.board.write(0x1000_9000, 4, 0x101);
    let start = rig.t;
    let mut end = None;
    for _ in 0..20_000 {
        rig.run(1);
        if end.is_none() && rig.board.read(0x1000_9000, 4) as u32 & 0x100 == 0 {
            end = Some(rig.t - start);
        }
    }
    let end = end.expect("the transfer ends");
    let taken = (qwc - vif::DEPTH[1] as u64) * 4 * clock::BUS;
    assert!(end > qwc * clock::BUS + 100, "held by the full FIFO: {end} cycles");
    assert!(end.abs_diff(taken) <= 16, "the VIF's pace, {taken} cycles: {end}");
    for a in 0..256u32 {
        assert_eq!(rig.qword(1, a), [a; 4], "qword {a}");
    }
}

/// VIF1 reversed (FDR set, D1_CHCR.DIR clear) carries the GS's local-to-host data to memory (EE User's Manual chapter
/// 5's channel table, GS User's Manual §4.2.2).
#[test]
fn vif1_reversed_carries_local_to_host_data() {
    use sapphire::gs::psm::Psm;
    use sapphire::gs::{HostInterface, privileged, reg as gs_reg};
    let mut rig = Rig::new();
    let gs = &mut rig.board.gs;
    for i in 0..64u32 {
        gs.local.write(Psm::Ct32, 0, 1, i % 8, i / 8, 0xC000_0000 | i);
    }
    gs.write(gs_reg::BITBLTBUF, 0x0001_0000);
    gs.write(gs_reg::TRXPOS, 0);
    gs.write(gs_reg::TRXREG, 8 | 8 << 32);
    gs.write(gs_reg::TRXDIR, 1);
    rig.board.write(privileged::address(privileged::BUSDIR), 8, 1);
    rig.set(1, reg::STAT, vif::stat::FDR);
    rig.board.write(0x1000_E000, 4, 1);
    rig.board.write(0x1000_9010, 4, 0x0020_0000);
    rig.board.write(0x1000_9020, 4, 16);
    rig.board.write(0x1000_9000, 4, 0x100);
    rig.run(10_000);
    assert_eq!(rig.board.read(0x1000_9000, 4) as u32 & 0x100, 0, "the transfer ends");
    for i in 0..64u32 {
        assert_eq!(rig.board.map.read(0x0020_0000 + 4 * i, 4) as u32, 0xC000_0000 | i, "word {i}");
    }
}

/// The GIF as VIF1 reaches it, refusing PATH2 until opened (D-462).
struct Refusing {
    open: bool,
    taken: usize,
}

impl vif::Gif for Refusing {
    fn path2(&mut self, _q: u128, _hl: bool, at: u64) -> Option<u64> {
        self.open.then(|| {
            self.taken += 1;
            at
        })
    }
    fn idle_at(&mut self, _path3: bool, at: u64) -> Option<u64> {
        Some(at)
    }
    fn mask_path3(&mut self, _masked: bool, _at: u64) {}
    fn host_read(&mut self) -> Option<u128> {
        None
    }
}

/// A DIRECT whose data the GIF refuses asks the board for no event while the DMAC waits on the full FIFO, since the VIF
/// can take nothing until the GIF moves; before, it asked for its own time again and again and the board never left it
/// (Jak and Daxter's Naughty Dog logo, D-462). Once the GIF takes PATH2 the data goes on and VGW clears.
#[test]
fn a_direct_refused_by_the_gif_asks_for_no_event() {
    let mut v = vif::Vif::new(1);
    let mut vpu = sapphire::vu::Vpu::new();
    let mut gif = Refusing { open: false, taken: 0 };
    v.put(u128::from(0x50u32 << 24 | 20) << 96, false);
    for i in 0..15u128 {
        v.put(i, false);
    }
    assert_eq!(v.request(1, false, &mut gif), 0, "the FIFO is full");
    v.run_to(1_000, &mut vif::Reach { vpu: &mut vpu, gif: &mut gif });
    assert_ne!(v.read(reg::STAT) & 0x8, 0, "VGW: waiting for the GIF");
    v.put(15, false);
    v.put(16, false);
    assert_eq!(v.request(1, false, &mut gif), 0, "the DMAC has filled the FIFO again and waits on it");
    assert_eq!(v.next_event(), u64::MAX, "nothing the board can run");
    gif.open = true;
    v.run_to(2_000, &mut vif::Reach { vpu: &mut vpu, gif: &mut gif });
    assert_eq!((gif.taken, v.read(reg::STAT) & 0x8), (17, 0), "the data taken once the GIF has PATH2");
}
