//! Stage S15, the GIF (Sapphire_Native.md §33): the project's programs sending GS packets down each path and
//! interleaving them, run on the interpreter with the board and graded by the records `s15::manual` gives from the EE
//! User's Manual's chapter 7 over S16's and S17's models of the GS; S16's, S17's and S18's own programs, until
//! now graded by driving the GS's host interface directly, run whole through PATH3 and VIF1; and what no program can
//! show a referee (each packing conversion's output, the FIFO's depth against the DMAC, the paths' cost), on the units.

mod s6;
mod s11;
mod s14;
mod s15;
mod s16;
mod s17;
mod s18;

use sapphire::board::System;

use s6::{DONE, DONE_AT, ORIGIN, Program, RECORD};

fn assemble(source: &str) -> Vec<u32> {
    emusen_mips::assemble(source, ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}"))
}

/// Runs a program on the interpreter and the board, and returns the system with every record word graded.
fn grade(p: Program) -> System {
    let name = p.name;
    let (source, expected) = p.finish();
    let mut sys = System::new(ORIGIN);
    sys.board.map.load(ORIGIN, &assemble(&source));
    let limit = 400_000_000;
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

#[test]
fn s16s_programs_run_whole_through_path3_and_vif1() {
    for script in s16::all() {
        grade(s16::program(&script));
    }
}

#[test]
fn s17s_programs_draw_through_path3() {
    for script in s17::all() {
        grade(s17::program(&script));
    }
}

#[test]
fn s18s_programs_texture_through_path3() {
    for script in s18::all() {
        grade(s17::program(&script));
    }
}

#[test]
fn packed_mode_writes_each_descriptor_as_section_7_3_draws_it() {
    grade(s15::packed());
}

#[test]
fn reglist_and_image_modes_follow_sections_7_4_and_7_5() {
    grade(s15::reglist());
}

#[test]
fn every_path_carries_the_same_packets_to_the_gs() {
    grade(s15::paths());
}

#[test]
fn the_paths_are_arbitrated_at_packet_ends_and_path3_masked_and_interrupted_as_section_7_1_says() {
    grade(s15::arbitration());
}

#[test]
fn the_intermittent_mode_lets_direct_and_xgkick_overtake_a_running_image_transfer() {
    grade(s15::intermittent());
}

#[test]
fn the_gif_registers_pause_reset_and_report_as_section_7_6_says() {
    grade(s15::regs());
}

/// The probe of the second DIRECT that PCSX2 does not take (Sapphire_Native.md §33.8): every form takes both here.
#[test]
fn a_second_direct_reaches_the_gs_in_every_form_of_the_probe() {
    for form in 0..4 {
        grade(s15::second_direct(form));
    }
}

/// The GS's host interface as a list of what reached it.
#[derive(Default)]
struct Recorder(Vec<(u8, u64)>);

impl sapphire::gif::packing::Out for Recorder {
    fn write(&mut self, r: u8, data: u64) {
        self.0.push((r, data));
    }
}

impl sapphire::gs::HostInterface for Recorder {
    fn write(&mut self, r: u8, data: u64) {
        self.0.push((r, data));
    }
    fn read(&mut self) -> Option<u64> {
        None
    }
    fn stalled(&self) -> bool {
        false
    }
    fn reversed(&self) -> bool {
        false
    }
}

/// Generated packets in every format, every descriptor among their registers, random bits in every qword: the source
/// writes exactly what chapter 7's figures give (`s15::manual`), XYZF2's fog byte, ADC, A+D's address, Q from ST and
/// its reset at each GIFtag among them, and ends each packet where the manual does.
#[test]
fn every_packing_conversion_writes_what_section_7_3_draws() {
    use sapphire::gif::packing::{Packet, Q_ONE};
    let mut x = 0x1357_9BDFu64;
    let mut next = || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };
    for round in 0..2000 {
        let mut packet = Vec::new();
        let prims = 1 + next() % 4;
        for k in 0..prims {
            let flg = (next() % 4) as u32;
            let nreg = 1 + (next() % 16) as usize;
            let regs: Vec<u8> = (0..nreg).map(|_| (next() % 16) as u8).collect();
            let nloop = if round % 7 == 0 && k == 0 { 0 } else { (next() % 4) as u32 };
            let pre = (next() % 2 == 0).then(|| (next() & 0x7FF) as u32);
            packet.push(s15::manual::tag(nloop, k == prims - 1, pre, flg, &regs) | u128::from(next() & 0x0000_3FFF_FFFF_0000));
            let data = match flg {
                0 => nloop as usize * nreg,
                1 => (nloop as usize * nreg).div_ceil(2),
                _ => nloop as usize,
            };
            packet.extend((0..data).map(|_| u128::from(next()) | u128::from(next()) << 64));
        }
        let (want, n) = s15::manual::Gif::default().packet(&packet);
        assert_eq!(n, packet.len());
        let (mut p, mut q, mut out) = (Packet::default(), Q_ONE, Recorder::default());
        for (i, &w) in packet.iter().enumerate() {
            let s = p.take(w, &mut q, &mut out);
            assert_eq!(s.end, i == packet.len() - 1, "round {round}: the packet ends at its EOP primitive's end, qword {i}");
        }
        assert_eq!(out.0, want, "round {round}: {packet:x?}");
    }
}

/// On the board: PATH3 masked by GIF_MODE takes 16 qwords into the FIFO and holds the DMA there (QWC 24 of 40 left),
/// and runs once unmasked; an IMAGE packet takes two bus cycles a qword through the GIF and a PACKED one one, so the
/// DMA the GIF paces takes about twice as long (Sapphire_Native.md §33.4, measured).
#[test]
fn the_fifo_holds_sixteen_qwords_and_the_gif_paces_the_dma() {
    use sapphire::board::Board;
    use sapphire::ee::Bus;
    use sapphire::gs::HostInterface;
    let mut spr = vec![0u8; 16 << 10];
    let send = |b: &mut Board, spr: &mut Vec<u8>, packet: &[u128], mask: bool| -> u64 {
        for (i, &q) in packet.iter().enumerate() {
            b.write(0x0040_0000 + 16 * i as u32, 16, q);
        }
        b.write(s15::GIF_MODE, 4, u128::from(mask));
        b.write(0x1000_E000, 4, 1);
        b.write(0x1000_A010, 4, 0x0040_0000);
        b.write(0x1000_A020, 4, packet.len() as u128);
        let start = b.now;
        b.write(0x1000_A000, 4, 0x101);
        let mut t = b.now;
        while b.read(0x1000_A000, 4) & 0x100 != 0 && t < start + 200_000 {
            t += 2;
            b.run_to(t, spr);
            if mask && t > start + 2_000 {
                break;
            }
        }
        t - start
    };
    let mut b = Board::new(sapphire::clock::NTSC);
    let mut long = vec![s15::manual::tag(39, true, None, 0, &[0xE])];
    long.extend((0..39).map(|k| s15::manual::ad(0x62, k as u64 | 0xFFFF_FFFF << 32)));
    send(&mut b, &mut spr, &long, true);
    let stat = b.read(s15::GIF_STAT, 4) as u32;
    assert_eq!((stat >> 24, b.read(0x1000_A020, 4)), (16, 24), "16 qwords in the FIFO, the DMA held with 24 left");
    assert_eq!(b.gs.siglblid() >> 32, 0, "nothing reached the GS");
    b.write(s15::GIF_MODE, 4, 0);
    b.run_to(b.now + 4_000, &mut spr);
    assert_eq!(b.gs.siglblid() >> 32, 38, "unmasked, every LABEL arrived");
    let mut long = vec![s15::manual::tag(199, true, None, 0, &[0xE])];
    long.extend((0..199).map(|k| s15::manual::ad(0x62, k as u64 | 0xFFFF_FFFF << 32)));
    let packed = send(&mut b, &mut spr, &long, false);
    let mut image = vec![s15::manual::tag(199, true, None, 2, &[])];
    image.extend((0..199).map(|_| 0u128));
    b.gs.write(0x50, 0);
    b.gs.write(0x52, 40 << 32 | 40);
    b.gs.write(0x53, 0);
    let imaged = send(&mut b, &mut spr, &image, false);
    assert!(imaged > packed * 3 / 2 && imaged < packed * 5 / 2, "IMAGE {imaged} cycles against PACKED {packed}");
}

/// PATH1's port tells VU1 its packet's end from the packet in VU1's memory (an arbitration cycle, a cycle a GIFtag and
/// a PACKED qword), a second kick starting behind it; a packet with no EOP in the memory is never done.
#[test]
fn an_xgkick_is_told_its_packets_end_and_a_second_starts_behind_it() {
    use sapphire::clock::BUS;
    use sapphire::gif::{ENDLESS, Path1Port};
    use sapphire::vu::Path1;
    let mut data = vec![0u8; 16 << 10];
    let put = |data: &mut Vec<u8>, a: usize, q: u128| data[16 * a..16 * a + 16].copy_from_slice(&q.to_le_bytes());
    put(&mut data, 10, s15::manual::tag(3, true, None, 0, &[0xE]));
    let mut port = Path1Port::default();
    port.set_free(0);
    assert_eq!(port.kick(&data, 10, 100), 100 + BUS * (1 + 1 + 3));
    assert_eq!(port.kick(&data, 10, 101), 100 + 2 * BUS * 5, "behind the first");
    assert_eq!(port.take_kick(), Some((10, 100, 100 + BUS * 5)));
    let mut endless = vec![0u8; 16 << 10];
    put(&mut endless, 0, s15::manual::tag(1, false, None, 0, &[0xE]));
    port.set_free(0);
    assert_eq!(port.kick(&endless, 1, 0), ENDLESS, "no EOP anywhere");
}

/// On the GIF directly: with PATH3's packet, a kick and a DIRECT all waiting at once, PATH1 goes first, then PATH2,
/// then PATH3 (§7.1.2); masked by GIF_MODE, PATH3 waits behind both and then for the mask.
#[test]
fn the_arbitration_grants_path1_then_path2_then_path3() {
    use sapphire::gif::Gif;
    use sapphire::vu::{VU1_MEM, Vu};
    for masked in [false, true] {
        let mut vu1 = Vu::new(VU1_MEM);
        vu1.set_mem_q(0, s15::manual::tag(1, true, None, 0, &[0xE]));
        vu1.set_mem_q(1, s15::manual::ad(0x62, 1));
        let mut g = Gif::new();
        let mut gs = Recorder::default();
        if masked {
            g.write(0x10, 1);
        }
        for q in s15::label_packet(3) {
            g.put(q, false, 20);
        }
        g.kick(0, 10, 20);
        let tag = s15::manual::tag(1, true, None, 0, &[0xE]);
        assert_eq!(g.path2(tag, false, 10, &mut gs), None, "PATH1 waits first");
        g.run_to(100, &vu1, &mut gs);
        assert!(g.path2(tag, false, 102, &mut gs).is_some(), "PATH2 next");
        assert!(g.path2(s15::manual::ad(0x62, 2), false, 104, &mut gs).is_some());
        g.run_to(200, &vu1, &mut gs);
        let labels = |gs: &Recorder| gs.0.iter().filter(|w| w.0 == 0x62).map(|w| w.1 & 0xFFFF).collect::<Vec<_>>();
        if masked {
            assert_eq!(labels(&gs), [1, 2], "PATH3 masked");
            g.write(0x10, 0);
            g.run_to(300, &vu1, &mut gs);
        }
        assert_eq!(labels(&gs), [1, 2, 3]);
    }
}

/// ps2autotests' `dma/vif/mskpath3` console text (its `.expected`, a console's; the program's source not read), whose
/// values follow from GIF_STAT read whole with M3P its only bit set [A]: MSKPATH3 through VIF1's FIFO sets and clears
/// it. PCSX2 reproduces the text (Sapphire_Native.md §33.7); Sapphire runs the program itself with S10.
#[test]
fn the_mskpath3_console_text_follows_from_gif_stat() {
    use sapphire::board::Board;
    use sapphire::ee::Bus;
    let text = "-- TEST BEGIN\nmask value (initial): 00000000\nmask value (after mskpath3(true))): 00000002\nmask value (after mskpath3(false))): 00000000\n-- TEST END\n";
    let mut b = Board::new(sapphire::clock::NTSC);
    let mut spr = vec![0u8; 16 << 10];
    let mut lines = vec!["-- TEST BEGIN".to_string(), format!("mask value (initial): {:08x}", b.read(s15::GIF_STAT, 4))];
    for (on, what) in [(true, "true"), (false, "false")] {
        b.write(0x1000_5000, 16, u128::from(0x0600_0000u32 | if on { 0x8000 } else { 0 }));
        b.run_to(b.now + 100, &mut spr);
        lines.push(format!("mask value (after mskpath3({what}))): {:08x}", b.read(s15::GIF_STAT, 4)));
    }
    lines.push("-- TEST END".into());
    assert_eq!(lines.join("\n") + "\n", text);
}
