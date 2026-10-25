//! Stage S21's gate (Sapphire_Native.md §22.8): the project's bare IOP programs on the memory map, the I-cache and
//! scratchpad, the INTC, the DMAC and the timers, run on `blackmoon-r3000` with Sapphire's IOP bus inside the board and
//! graded by the records the documents give; and what no program can show a referee, checked on the units directly.

mod s21;

use sapphire::board::{Board, System};
use sapphire::clock::{self, IOP};
use sapphire::iop::dma::{self, Dmac, Memory, Peripherals, StandIns};
use sapphire::iop::{self, icache};
use sapphire::sif;

/// Runs an IOP program in a board until its done word, `setup` given the board first; the board and the expected record.
fn run(p: s21::Program, setup: impl FnOnce(&mut Board)) -> (Board, Vec<Option<u32>>) {
    let name = p.name;
    let (source, expected) = p.finish();
    let words = s21::assemble(&source);
    let mut b = Board::new(clock::NTSC);
    b.iop.bus.load(s21::ORIGIN, &words);
    b.iop.bus.load(0x80, &s21::vector());
    setup(&mut b);
    b.iop.enter(s21::K1 | s21::ORIGIN);
    let mut spr = vec![0u8; 16 << 10];
    let limit = IOP * 400_000_000;
    while b.iop.bus.word(s21::DONE_AT) != s21::DONE {
        assert!(b.now < limit, "{name}: no done word by EE cycle {limit}, IOP pc {:08x}", b.iop.cpu.regs.pc);
        let t = b.now + IOP * 20_000;
        b.run_to(t, &mut spr);
    }
    (b, expected)
}

/// Runs and grades a program; the board, for further checks.
fn grade(p: s21::Program, setup: impl FnOnce(&mut Board)) -> Board {
    let name = p.name;
    let (b, expected) = run(p, setup);
    let mut wrong = Vec::new();
    for (i, want) in expected.iter().enumerate() {
        let got = b.iop.bus.word(s21::RECORD + 4 * i as u32);
        if let Some(w) = want
            && *w != got
        {
            wrong.push(format!("word {i}: {got:08x}, expected {w:08x}"));
        }
    }
    assert!(wrong.is_empty(), "{name}: {} of {} words differ, first {:?}", wrong.len(), expected.len(), &wrong[..wrong.len().min(8)]);
    b
}

fn record(b: &Board, i: usize) -> u32 {
    b.iop.bus.word(s21::RECORD + 4 * i as u32)
}

#[test]
fn the_memory_map_scratchpad_and_bus_errors_equal_the_documents() {
    let b = grade(s21::map(), |_| {});
    assert_eq!(record(&b, 0), iop::PRID, "PRId is D-60's");
}

#[test]
fn a_fetch_from_the_scratchpad_is_an_instruction_bus_error() {
    grade(s21::fetch(), |_| {});
}

#[test]
fn the_icache_holds_stale_code_until_flushed_and_conflicts_at_4_kib() {
    let b = grade(s21::cache(), |_| {});
    let n = s21::cache().expected.len();
    let (cached, uncached, disabled) = (record(&b, n - 5), record(&b, n - 4), record(&b, n - 3));
    assert!((3000..3100).contains(&cached), "1,000 passes of three hits, a cycle each: {cached}");
    assert!((15_000..15_100).contains(&uncached), "1,000 passes of three uncached fetches, five cycles each: {uncached}");
    assert_eq!(disabled, uncached, "IS1 clear runs kseg0 uncached");
}

#[test]
fn the_intc_masks_latches_and_drives_ip2() {
    grade(s21::intc(), |_| {});
}

#[test]
fn otc_and_the_dmac_registers_equal_the_documents() {
    grade(s21::dma(), |_| {});
}

/// Since S26 channels 4 and 7 reach the SPU2's cores in place of their stand-ins, each writing at its own transfer address.
#[test]
fn each_dma_mode_reaches_its_stand_in() {
    let b = grade(s21::dma_modes(), |b| {
        for w in s21::sif1_queue() {
            b.iop.bus.sif.fifo[sif::SIF1].push(w);
        }
    });
    let fifo = |f: usize| b.iop.bus.sif.fifo[f].words().copied().collect::<Vec<_>>();
    let spu2 = &b.iop.bus.spu2;
    let words = |n: usize| (0..n).map(|i| u32::from_le_bytes(spu2.ram[4 * i..4 * i + 4].try_into().unwrap())).collect::<Vec<_>>();
    let w = |i: u32| 0x7000_0000 | i;
    assert_eq!(spu2.cores[1].tsa, 24, "slice: three blocks of four into core 1, a halfword each step of its address");
    let mut want: Vec<u32> = (0..5).map(w).collect();
    assert_eq!(&words(5), &want[..], "the requested burst of five into core 0, over the slice's first words");
    want.extend((0..12).map(w));
    assert_eq!(words(17)[5..], want[5..], "the chopped burst's first words after it");
    assert_eq!(fifo(sif::SIF2), [0x2000_0000, 0x2000_0001, 0x2000_0002], "the list's words, no headers, in SIF2's FIFO");
    assert_eq!(
        fifo(sif::SIF0),
        [0x9000_0000, 0x9000_0001, 0x9100_0000, 0x9100_0001, 0, 0, 0x9000_0002, 0x9000_0003, 0x9200_0000, 0x9200_0001, 0x9200_0002, 0],
        "each tag's two words before its data, in SIF0's FIFO, the data padded with zeros to whole quadwords (D-345)"
    );
    assert!(b.iop.bus.sif.fifo[sif::SIF1].is_empty(), "the tag and its words taken from SIF1's FIFO");
    assert_eq!(spu2.cores[0].tsa, 2 * (5 + 32), "the requested burst and the chopped one");
}

#[test]
fn the_timers_widths_clocks_flags_requests_and_gates_equal_the_documents() {
    grade(s21::timers(), |_| {});
}

/// The IOP runs at an eighth of the EE's clock: a loop of known cycles takes eight times as many EE cycles.
#[test]
fn the_iop_keeps_its_ratio_to_the_ee_clock() {
    let words = s21::assemble("top: addiu t0, t0, 1\nbne zero, zero, top\nnop\nbeq zero, zero, top\nnop\n");
    let mut b = Board::new(clock::NTSC);
    b.iop.bus.load(s21::ORIGIN, &words);
    b.iop.enter(s21::K1 | s21::ORIGIN);
    let mut spr = vec![0u8; 16 << 10];
    b.run_to(IOP * 50_000, &mut spr);
    let passes = b.iop.cpu.regs.r[8];
    assert!(b.iop.bus.now >= 50_000 && b.iop.bus.now < 50_010, "the IOP's clock {}", b.iop.bus.now);
    assert_eq!(passes, 50_000 / 25, "five uncached instructions of five cycles a pass");
}

/// A held IOP costs nothing and keeps the board's time; the EE reaches IOP RAM through 1C000000h.
#[test]
fn the_ee_writes_iop_ram_through_its_window_and_a_held_iop_follows_the_clock() {
    let ee = "lui t0, 0xbc01\nlui t1, 0x1234\nori t1, t1, 0x5678\nsw t1, 0x10(t0)\nlw t2, 0x10(t0)\nend: beq zero, zero, end\nnop\n";
    let words = emusen_mips::assemble(ee, 0x0010_0000, &emusen_mips::Dialect::R5900).unwrap();
    let mut sys = System::new(0x0010_0000);
    sys.board.map.load(0x0010_0000, &words);
    for _ in 0..20 {
        sys.step().unwrap();
    }
    assert_eq!(sys.board.iop.bus.word(0x1_0010), 0x1234_5678);
    assert_eq!(sys.ee.gpr[10] as u32, 0x1234_5678);
    assert!(sys.board.iop.held);
    assert_eq!(sys.board.iop.bus.now, sys.board.now / IOP);
}

/// Words a test memory gives the DMAC.
struct Flat(Vec<u32>);

impl Memory for Flat {
    fn load(&mut self, addr: u32) -> Option<u32> {
        self.0.get(addr as usize / 4).copied()
    }

    fn store(&mut self, addr: u32, w: u32) -> bool {
        match self.0.get_mut(addr as usize / 4) {
            Some(v) => {
                *v = w;
                true
            }
            None => false,
        }
    }
}

fn run_dmac(d: &mut Dmac, mem: &mut Flat, dev: &mut StandIns, until: u64) {
    let mut now = 0;
    while now < until {
        d.step(now, mem, dev);
        now = d.next_event().max(now + 1);
    }
}

/// DPCR's priorities decide which of two ready channels goes first, the higher channel on a tie; a channel whose
/// peripheral asserts no request waits, scheduling nothing, until woken.
#[test]
fn priorities_order_the_channels_and_a_channel_without_dreq_waits() {
    let mut mem = Flat(vec![0; 0x1000]);
    let mut dev = StandIns::default();
    let mut d = Dmac::default();
    d.write(dma::DMACEN, 1);
    d.write(dma::DPCR, 0x000B_0900);
    for n in [dma::SIF2, dma::SPU] {
        d.write(dma::base(n) + 4, 4);
        d.write(dma::base(n) + 8, 0x0100_0001);
    }
    d.step(0, &mut mem, &mut dev);
    assert_ne!(d.read(dma::base(dma::SPU) + 8) & 0x0100_0000, 0, "SPU at priority 3 waits");
    assert_eq!(dev.ports[dma::SIF2].count, 4, "SIF2 at priority 1 goes first");
    run_dmac(&mut d, &mut mem, &mut dev, 100);
    assert_eq!(dev.ports[dma::SPU].count, 4);
    d.write(dma::DPCR, 0x000B_0B00);
    for n in [dma::SIF2, dma::SPU] {
        d.write(dma::base(n) + 8, 0x0100_0001);
    }
    d.step(200, &mut mem, &mut dev);
    assert_eq!((dev.ports[dma::SPU].count, dev.ports[dma::SIF2].count), (8, 4), "on a tie the higher channel first");
    run_dmac(&mut d, &mut mem, &mut dev, 400);
    d.write(dma::DPCR2, 0x8);
    d.write(dma::base(dma::SPU2) + 4, 2);
    d.write(dma::base(dma::SPU2) + 8, 0x0100_0000);
    d.step(500, &mut mem, &mut dev);
    assert_eq!(d.next_event(), u64::MAX, "no request, nothing scheduled");
    dev.ports[dma::SPU2].give.extend([7, 8]);
    d.wake();
    run_dmac(&mut d, &mut mem, &mut dev, 600);
    assert_eq!(&mem.0[..2], [7, 8]);
    assert_eq!(d.read(dma::base(dma::SPU2) + 8) & 0x0100_0000, 0);
}

/// A burst holds the processor for a cycle a word; chopping splits it into windows with the processor's cycles between.
#[test]
fn a_burst_holds_the_bus_and_chopping_opens_windows() {
    let mut mem = Flat(vec![0; 0x1000]);
    let mut dev = StandIns::default();
    let mut d = Dmac::default();
    d.write(dma::DMACEN, 1);
    d.write(dma::DPCR, 0x000F_0000);
    d.write(dma::base(dma::SPU) + 4, 64);
    d.write(dma::base(dma::SPU) + 8, 0x1100_0001);
    d.step(10, &mut mem, &mut dev);
    assert_eq!(d.hold, 10 + 64 * dma::WORD);
    run_dmac(&mut d, &mut mem, &mut dev, 1000);
    d.write(dma::base(dma::SPU) + 4, 64);
    d.write(dma::base(dma::SPU) + 8, 0x1153_0101);
    d.step(1000, &mut mem, &mut dev);
    assert_eq!(d.hold, 1000 + 8, "a window of eight words");
    assert_eq!(d.next_event(), 1000 + 8 + 32, "then 32 cycles for the processor");
}

/// The cache's refill rules as psx-spx's "i-Cache" gives them.
#[test]
fn a_miss_fills_to_the_lines_end_and_iblksz_0_fills_two_words() {
    let mut c = icache::ICache { bcc: icache::bcc::KERNEL, ..Default::default() };
    assert_eq!(c.lookup(0x1000_0008), icache::Lookup::Miss { first: 2, count: 2 });
    c.fill(0x1000_0008, 2, &[1, 2]);
    assert_eq!(c.lookup(0x1000_000C), icache::Lookup::Hit(2));
    assert_eq!(c.lookup(0x1000_0000), icache::Lookup::Miss { first: 0, count: 4 }, "tag matches, word invalid");
    c.bcc &= !(3 << icache::bcc::IBLKSZ_SHIFT);
    assert_eq!(c.lookup(0x2000_0000), icache::Lookup::Miss { first: 0, count: 2 });
}

/// The DMAC's peripherals interface, as a stage implements it.
#[test]
fn the_peripherals_interface_is_the_stand_ins() {
    fn takes(_: &mut dyn Peripherals) {}
    takes(&mut StandIns::default());
}
