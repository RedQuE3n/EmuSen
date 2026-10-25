//! Stage S30's gate on the project's programs (Sapphire_Native.md §38.6): each program run once to its done word, then
//! again with its state saved at evenly spaced instants of its run; each state is loaded into a machine just made and
//! held to the machine that saved it until the next instant, their states and sound equal there. The programs reach
//! every unit: DMA chains, VU microprograms, VIF, the GIF's paths, GS drawing, the PCRTC, the SIF, SIO2, SPU2 voices and
//! the IPU.

mod s6;
mod s9;
mod s11;
mod s12;
mod s13;
mod s14;
mod s15;
mod s16;
mod s17;
mod s20;
mod s21;
mod s22;
mod s25;
mod s26;
mod s27;

use emusen_native::{StateReader, StateWriter};
use sapphire::board::{Board, System};
use sapphire::clock::{self, IOP};
use sapphire::sio2::card::MemoryCard;
use sapphire::sio2::pad::DualShock2;

/// The instants a run is cut at.
const CUTS: u64 = 4;

fn state(sys: &System) -> Vec<u8> {
    let mut c = StateWriter::counter();
    sys.write_state(&mut c);
    let mut v = vec![0; c.len()];
    sys.write_state(&mut StateWriter::new(&mut v));
    v
}

/// The listing's fields whose bytes differ, for a failure's words.
fn differing(sys: &System, a: &[u8], b: &[u8]) -> Vec<String> {
    let mut w = StateWriter::layout();
    sys.write_state(&mut w);
    w.into_layout()
        .lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split(' ').collect();
            let (at, len) = (f[0].parse::<usize>().ok()?, f[1].parse::<usize>().ok()?);
            (a[at..at + len] != b[at..at + len]).then(|| f[3].to_string())
        })
        .collect()
}

/// A program as the gate runs it: how its machine is made, one step of it, and its end.
struct Run {
    name: String,
    make: Box<dyn Fn() -> System>,
    step: fn(&mut System),
    done: fn(&mut System) -> bool,
}

fn drain(sys: &mut System, into: &mut Vec<i16>) {
    let mut chunk = [0i16; 4096];
    loop {
        let n = sys.board.iop.bus.spu2.out.drain(&mut chunk, 2048);
        into.extend_from_slice(&chunk[..2 * n]);
        if n == 0 {
            break;
        }
    }
}

/// The media kept apart from a state, which a host keeps as they stand across a load: the cards' images and the
/// EEPROM's bytes (Sapphire_Native.md §38.3).
fn keep_media(from: &System, to: &mut System) {
    let (f, t) = (&from.board.iop.bus, &mut to.board.iop.bus);
    for (fs, ts) in f.sio2.slots.iter().zip(&mut t.sio2.slots) {
        for (fc, tc) in fs.cards.iter().zip(&mut ts.cards) {
            if let (Some(fc), Some(tc)) = (fc, tc) {
                tc.image.clone_from(&fc.image);
            }
        }
    }
    t.cdvd.nvm.bytes.clone_from(&f.cdvd.nvm.bytes);
}

fn round_trips(run: &Run) {
    let mut a = (run.make)();
    let mut n = 0u64;
    while !(run.done)(&mut a) {
        (run.step)(&mut a);
        n += 1;
        assert!(n < 400_000_000, "{}: no done word", run.name);
    }
    let end = n + n / 16 + 64;
    let points: Vec<u64> = (1..=CUTS).map(|k| n * k / (CUTS + 1)).filter(|&p| p > 0).collect();
    let mut a = (run.make)();
    let mut at = 0;
    let (mut heard_a, mut heard_b) = (Vec::new(), Vec::new());
    for (i, &p) in points.iter().enumerate() {
        while at < p {
            (run.step)(&mut a);
            drain(&mut a, &mut heard_a);
            at += 1;
        }
        let saved = state(&a);
        let mut b = (run.make)();
        b.read_state(&mut StateReader::new(&saved)).unwrap();
        keep_media(&a, &mut b);
        assert!(state(&b) == saved, "{}: save, load, save differs at step {p}", run.name);
        let next = points.get(i + 1).copied().unwrap_or(end);
        heard_a.clear();
        heard_b.clear();
        while at < next {
            (run.step)(&mut a);
            (run.step)(&mut b);
            drain(&mut a, &mut heard_a);
            drain(&mut b, &mut heard_b);
            at += 1;
        }
        let (sa, sb) = (state(&a), state(&b));
        assert!(sa == sb, "{}: loaded at step {p} of {n}, at step {next} its state differs in {:?}", run.name, differing(&a, &sa, &sb));
        assert!(heard_a == heard_b, "{}: loaded at step {p}, its sound differs by step {next}", run.name);
    }
}

fn assemble(source: &str, origin: u32) -> Vec<u32> {
    emusen_mips::assemble(source, origin, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}"))
}

fn ee_step(s: &mut System) {
    s.step().unwrap();
}

fn ee_done(s: &mut System) -> bool {
    s.peek(s6::DONE_AT, 4) as u32 == s6::DONE
}

/// An EE program of S6's form at `ORIGIN`.
fn ee(p: s6::Program) -> Run {
    let name = p.name.to_string();
    let (source, _) = p.finish();
    let words = assemble(&source, s6::ORIGIN);
    Run {
        name,
        make: Box::new(move || {
            let mut sys = System::new(s6::ORIGIN);
            sys.board.map.load(s6::ORIGIN, &words);
            sys
        }),
        step: ee_step,
        done: ee_done,
    }
}

/// A bare IOP program of S21's form in a board whose EE does not run, `setup` given the board first.
fn iop(p: s21::Program, setup: fn(&mut Board)) -> Run {
    let name = p.name.to_string();
    let (source, _) = p.finish();
    let words = s21::assemble(&source);
    Run {
        name,
        make: Box::new(move || {
            let mut sys = System::new(0);
            let b = &mut sys.board;
            b.iop.bus.load(s21::ORIGIN, &words);
            b.iop.bus.load(0x80, &s21::vector());
            setup(b);
            b.iop.enter(s21::K1 | s21::ORIGIN);
            sys
        }),
        step: |s| {
            let t = s.board.now + IOP * 3_001;
            let System { ee, board } = s;
            board.run_to(t, &mut ee.spram[..]);
        },
        done: |s| s.board.iop.bus.word(s21::DONE_AT) == s21::DONE,
    }
}

#[test]
fn dma_chains_round_trip_mid_run() {
    for p in [s11::chain(), s11::callret(), s11::interleave(), s11::stall(), s11::mfifo()] {
        round_trips(&ee(p));
    }
}

#[test]
fn vu_microprograms_round_trip_mid_run() {
    for p in s13::all().into_iter().take(3) {
        round_trips(&ee(p));
    }
}

#[test]
fn vif_unpacks_and_microprograms_it_starts_round_trip_mid_run() {
    for p in [s14::unpack(), s14::micro_program(), s14::dbuf()] {
        round_trips(&ee(p));
    }
}

#[test]
fn the_gifs_paths_round_trip_mid_run() {
    for p in [s15::paths(), s15::arbitration(), s15::intermittent()] {
        round_trips(&ee(p));
    }
}

#[test]
fn gs_drawing_round_trips_mid_run() {
    for script in [s17::triangles(), s17::gouraud(), s17::sprites()] {
        round_trips(&ee(s17::program(&script)));
    }
}

#[test]
fn the_pcrtcs_timing_round_trips_mid_run() {
    round_trips(&ee(s20::timing()));
}

#[test]
fn the_sifs_messages_round_trip_mid_run() {
    for p in [s22::sif1(), s22::sif0(), s22::sif2()] {
        let image = p.image.clone();
        round_trips(&Run {
            name: p.name.to_string(),
            make: Box::new(move || {
                let mut sys = System::new(s22::EE_ORIGIN);
                sys.board.map.load(s22::EE_ORIGIN, &image);
                sys
            }),
            step: |s| {
                if s.board.iop.held && s.board.iop.bus.word(0x88) == s21::vector()[2] {
                    s.board.iop.enter(s21::K1 | s21::ORIGIN);
                }
                s.step().unwrap();
            },
            done: |s| s.peek(s22::EE_DONE_AT, 4) as u32 == s22::DONE,
        });
    }
}

#[test]
fn sio2s_pads_cards_and_multitap_round_trip_mid_run() {
    round_trips(&iop(s25::pad_program(), |_| {}));
    round_trips(&iop(s25::card_program(), |b| {
        for s in &mut b.iop.bus.sio2.slots {
            s.cards[0] = Some(MemoryCard::erased());
        }
    }));
    round_trips(&iop(s25::tap_program(), |b| {
        let s = &mut b.iop.bus.sio2.slots[0];
        s.tap = true;
        s.cards[0] = Some(MemoryCard::erased());
        s.pads[0] = Some(DualShock2::default());
    }));
}

#[test]
fn spu2_voices_round_trip_mid_run_with_their_sound() {
    for g in [s26::gate::voices(), s26::gate::reverb(), s26::gate::adma(), s26::gate::irq()] {
        round_trips(&iop(g.finish().0, |_| {}));
    }
}

#[test]
fn the_ipus_decoding_round_trips_mid_run() {
    for p in s27::programs().into_iter().take(2) {
        let (name, image) = (p.name.to_string(), p.image.clone());
        round_trips(&Run {
            name,
            make: Box::new(move || {
                let mut sys = System::new(s27::program::ORIGIN);
                sys.board.map.load(s27::program::ORIGIN, &image);
                sys
            }),
            step: ee_step,
            done: |s| s.peek(s27::program::DONE_AT, 4) as u32 == s27::program::DONE,
        });
    }
}

#[test]
fn the_board_alone_round_trips_at_the_pcrtcs_fields() {
    round_trips(&Run {
        name: "no program".into(),
        make: Box::new(|| System { ee: sapphire::ee::Ee::reset(), board: Board::new(clock::PAL) }),
        step: |s| {
            let t = s.board.now + 70_001;
            let System { ee, board } = s;
            board.run_to(t, &mut ee.spram[..]);
        },
        done: |s| s.board.pcrtc.fields >= 6,
    });
}
