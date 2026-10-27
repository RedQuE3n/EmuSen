//! Stage S34's gate (Sapphire_Native.md §53): the IOP's decoded fetch held to its fetch and decode, word, time and fault,
//! over every segment, the cache off and on, misses, hits, isolated stores, the cache control changed under valid lines
//! and a state's words; and the project's programs that write code into IOP RAM by a store, by the IOP's DMAC and by
//! the SIF's transfer, then run it from the cache and past it.

mod s21;

use blackmoon_r3000::{Bus, Size};
use emusen_native::{StateReader, StateWriter};
use sapphire::board::Board;
use sapphire::clock;
use sapphire::iop::bus::{CACHE_CONTROL, IopBus, ROM_BASE};
use sapphire::iop::icache::{LINES, bcc};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 16) as u32
    }

    fn below(&mut self, n: u32) -> u32 {
        self.next() % n
    }
}

/// A board whose IOP RAM's first 64 KiB and boot ROM's first 16 KiB hold the same random words for every seed.
fn board(seed: u64) -> Board {
    let mut b = Board::new(clock::NTSC);
    let mut r = Rng(seed);
    let words: Vec<u32> = (0..16 << 10).map(|_| r.next()).collect();
    b.iop.bus.load(0, &words);
    b.iop.bus.rom = (0..16 << 10).flat_map(|_| r.next().to_le_bytes()).collect();
    b
}

/// An address a fetch may be made at: RAM's first 4 KiB (oftenest, so that lines hit) or 16 KiB, its mirror at 2 MiB, the boot ROM, the scratchpad, an I/O
/// register, the region past RAM, or no device; through kuseg, kseg0, kseg1, kuseg past 512 MiB or kseg2.
fn address(r: &mut Rng) -> u32 {
    let offset = r.below(4096) * 4;
    let physical = match r.below(8) {
        0..=1 => offset & 0xFFC,
        2 => offset,
        3 => (2 << 20) + offset,
        4 => ROM_BASE + offset,
        5 => 0x1F80_0000 + (offset & 0x3FC),
        6 => 0x1F80_1000 + (offset & 0xFC),
        _ => 0x0100_0000 + offset,
    };
    match r.below(8) {
        0..=2 => physical,
        3..=5 => 0x8000_0000 | physical,
        6 => 0xA000_0000 | physical,
        _ if r.below(2) == 0 => 0x4000_0000 | physical,
        _ => 0xFFFE_0000 | (offset & 0x1FC),
    }
}

fn state(bus: &IopBus) -> Vec<u8> {
    let mut c = StateWriter::counter();
    bus.write_own(&mut c);
    let mut v = vec![0; c.len()];
    bus.write_own(&mut StateWriter::new(&mut v));
    v
}

/// What both buses must agree on after each access.
fn same(a: &IopBus, b: &IopBus, n: usize) {
    assert_eq!(a.now, b.now, "access {n}: the clock");
    assert_eq!(a.fills, b.fills, "access {n}: the refills");
    assert_eq!(a.icache, b.icache, "access {n}: the cache, its decoded words with it");
}

/// `fetch_decoded` is `fetch` and `decode`: the word, its operation, the time, the fault and the cache after it, through
/// every segment and target, the cache switched on and off and its block size changed under valid lines, isolated loads
/// and stores of words and tags, stores to code in RAM past the cache, and a state's words loaded into a cache whose
/// decoded words were left from other code.
#[test]
fn a_decoded_fetch_is_a_fetch_and_its_decoding() {
    for seed in 1..=6u64 {
        let (mut plain, mut decoded) = (board(seed), board(seed));
        let (a, b) = (&mut plain.iop.bus, &mut decoded.iop.bus);
        let mut r = Rng(seed * 0x9E37_79B9);
        let mut hits = 0u64;
        for n in 0..200_000 {
            match r.below(100) {
                0 => {
                    let v = match r.below(4) {
                        0 => 0,
                        1 => bcc::KERNEL & !bcc::IS1,
                        2 => bcc::KERNEL & !(3 << bcc::IBLKSZ_SHIFT),
                        _ => bcc::KERNEL,
                    };
                    a.write(CACHE_CONTROL, Size::Word, v);
                    b.write(CACHE_CONTROL, Size::Word, v);
                }
                1 => {
                    let (at, v) = (address(&mut r), r.next());
                    let tag = r.below(2) == 0;
                    for bus in [&mut *a, &mut *b] {
                        let saved = bus.icache.bcc;
                        bus.icache.bcc = if tag { saved | bcc::TAG } else { saved & !bcc::TAG };
                        bus.isolated_write(at & 0x1FFF_FFFC, Size::Word, v);
                        bus.icache.bcc = saved;
                    }
                }
                2..=4 => {
                    let (at, v) = (r.below(4096) * 4, r.next());
                    a.write(at, Size::Word, v);
                    b.write(at, Size::Word, v);
                    a.bus_error();
                    b.bus_error();
                }
                5 => {
                    let saved = state(b);
                    b.icache.ops = [emusen_mips::decode(r.next()); LINES * 4];
                    b.read_own(&mut StateReader::new(&saved)).expect("the state loads");
                }
                _ => {
                    let at = address(&mut r);
                    let w = a.fetch(at);
                    let op = emusen_mips::decode(w);
                    let fault = a.bus_error();
                    let cached = (at < 0x2000_0000 || (0x8000_0000..0xA000_0000).contains(&at)) && b.icache.enabled();
                    hits += u64::from(cached && b.icache.hit(at & 0x1FFF_FFFF).is_some());
                    let (w2, op2) = b.fetch_decoded(at);
                    assert_eq!((w2, op2, b.bus_error()), (w, op, fault), "access {n}: fetch at {at:08x}");
                }
            }
            same(a, b, n);
        }
        assert!(hits > 5_000, "seed {seed}: the hits taken ({hits})");
    }
}

/// The routine every program runs: `jr ra`, and in its delay slot the value into v0, which tells which code ran.
const RETURN: u32 = 0x03E0_0008;
const OLD: u32 = 0x3402_AAAA;
/// `lui v0, BBBBh`, an operation other than the old word's, so that a word run by the old word's decode gives another
/// value (`ori` with the new word's fields would give BBBBh).
const NEW: u32 = 0x3C02_BBBB;
const AT: u32 = s21::AREA + 0x2_0000;

fn call(p: &mut s21::Program, address: u32, want: u32) {
    p.li("t0", address);
    p.emit("jalr t0");
    p.emit("nop");
    p.record("v0", Some(want));
}

/// A program that runs the old routine cached, writes the new one over it by `write`, and calls it: cached (the line
/// stale, as the console's cache is, Sapphire_Native.md §22.3), through kseg1, with the cache control's IS1 clear, with
/// it set again (still stale), and after the line is flushed and refilled.
fn rewrite(name: &'static str, write: impl Fn(&mut s21::Program)) -> s21::Program {
    use s21::reg::BCC;
    let mut p = s21::Program::new(name);
    p.poke(s21::K1 | AT, RETURN);
    p.poke(s21::K1 | (AT + 4), OLD);
    call(&mut p, s21::K0 | AT, 0xAAAA);
    write(&mut p);
    call(&mut p, s21::K0 | AT, 0xAAAA);
    call(&mut p, s21::K1 | AT, 0xBBBB_0000);
    p.poke(BCC, bcc::KERNEL & !bcc::IS1);
    call(&mut p, s21::K0 | AT, 0xBBBB_0000);
    p.poke(BCC, bcc::KERNEL);
    call(&mut p, s21::K0 | AT, 0xAAAA);
    p.poke(BCC, bcc::KERNEL | bcc::TAG);
    p.li("t0", 0x0001_0000);
    p.emit("mtc0 t0, SR");
    p.emit("nop");
    p.li("t6", AT & 0xFF0);
    p.emit("sw zero, 0(t6)");
    p.emit("mtc0 zero, SR");
    p.emit("nop");
    p.poke(BCC, bcc::KERNEL);
    call(&mut p, s21::K0 | AT, 0xBBBB_0000);
    p
}

/// Runs an IOP program in a board until its done word, `setup` given the board first, and grades its record.
fn grade(p: s21::Program, setup: impl FnOnce(&mut Board)) {
    let name = p.name;
    let (source, expected) = p.finish();
    let mut b = Board::new(clock::NTSC);
    b.iop.bus.load(s21::ORIGIN, &s21::assemble(&source));
    b.iop.bus.load(0x80, &s21::vector());
    setup(&mut b);
    b.iop.enter(s21::K1 | s21::ORIGIN);
    let mut spr = vec![0u8; 16 << 10];
    while b.iop.bus.word(s21::DONE_AT) != s21::DONE {
        assert!(b.now < clock::IOP * 100_000_000, "{name}: no done word");
        let t = b.now + clock::IOP * 20_000;
        b.run_to(t, &mut spr);
    }
    let got: Vec<u32> = (0..expected.len()).map(|i| b.iop.bus.word(s21::RECORD + 4 * i as u32)).collect();
    let wrong: Vec<String> = expected.iter().zip(&got).enumerate().filter(|(_, (w, g))| w.is_some_and(|w| w != **g)).map(|(i, (w, g))| format!("word {i}: {g:08x}, expected {:08x}", w.unwrap())).collect();
    assert!(wrong.is_empty(), "{name}: {wrong:?}");
}

/// Code rewritten by the IOP's own store.
#[test]
fn code_rewritten_by_a_store_runs_as_the_console_runs_it() {
    grade(rewrite("s34-store", |p| p.poke(s21::K1 | (AT + 4), NEW)), |_| {});
}

/// Code rewritten by the IOP's DMAC: channel 1's peripheral gives the two words, a burst into RAM.
#[test]
fn code_rewritten_by_the_iops_dma_runs_as_the_console_runs_it() {
    use s21::reg::{DPCR, bcr, chcr, madr};
    let p = rewrite("s34-dma", |p| {
        p.poke(DPCR, 0x0777_77F7);
        p.poke(madr(1), AT);
        p.poke(bcr(1), 2);
        p.poke(chcr(1), 0x1100_0000);
        p.wait(chcr(1), 0x0100_0000, false, 10_000);
        p.record_is("t5", 10_000);
    });
    grade(p, |b| b.iop.bus.ports.ports[1].give.extend([RETURN, NEW]));
}

/// Code rewritten by a SIF1 transfer: the IOP's channel 10 takes a packet from the SIF's FIFO, its tag naming the
/// routine's address, into RAM.
#[test]
fn code_rewritten_by_a_sif_transfer_runs_as_the_console_runs_it() {
    use s21::reg::{DPCR2, chcr};
    let p = rewrite("s34-sif", |p| {
        p.poke(DPCR2, 0x0000_F000);
        p.poke(chcr(10), 0x0100_0600);
        p.wait(chcr(10), 0x0100_0000, false, 10_000);
        p.rec(chcr(10), Some(0x0000_0600));
    });
    grade(p, |b| {
        for w in [AT | 1 << 31, 2, 0, 0, RETURN, NEW] {
            b.iop.bus.sif.fifo[sapphire::sif::SIF1].push(w);
        }
    });
}
