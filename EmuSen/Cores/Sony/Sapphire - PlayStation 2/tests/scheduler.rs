//! The scheduler's step (Sapphire_Scheduler.md §54): the board's calls the step answers without making them leave every
//! program where the plain run left it. The records and the states' digests were taken from the tree before the step
//! (WiseMan `f6c51d5b9`) running the same programs; the rests the step keeps are not state, so a machine loaded from a
//! state every few steps, which keeps none, must run alike.

mod s54;

use emusen_native::StateReader;
use sapphire::blocks;
use sapphire::board::System;
use sapphire::machine::{Machine, Market};
use sapphire::media::Media;

fn digest(sys: &System) -> u64 {
    blocks::state_of(sys).iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01B3))
}

fn words_digest(w: &[u32]) -> u64 {
    w.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &x| (h ^ u64::from(x)).wrapping_mul(0x0000_0100_0000_01B3))
}

fn run(sys: &mut System, steps: u64) {
    for _ in 0..steps {
        sys.step().unwrap_or_else(|h| panic!("{h:?}"));
    }
}

/// A fresh machine holding `sys`'s state, and so none of the rests `sys` keeps.
fn reloaded(sys: &System) -> System {
    let mut fresh = System::new(0);
    fresh.read_state(&mut StateReader::new(&blocks::state_of(sys))).expect("the state loads");
    fresh
}

/// `make`'s machine run `steps` steps beside one loaded from its own state every `every` steps, the two compared
/// whole at each load.
fn alike_when_reloaded(name: &str, make: fn() -> System, steps: u64, every: u64) {
    let (mut plain, mut loaded) = (make(), make());
    let mut done = 0;
    while done < steps {
        run(&mut plain, every);
        run(&mut loaded, every);
        done += every;
        loaded = reloaded(&loaded);
        assert!(blocks::state_of(&plain) == blocks::state_of(&loaded), "{name}: the machines differ after {done} steps");
    }
}

#[test]
fn the_iop_is_brought_to_each_instruction_as_the_plain_run_brings_it() {
    let mut sys = s54::lockstep();
    run(&mut sys, 20_000);
    let records = s54::ee_records(&sys, 1024);
    assert!(records.iter().step_by(2).zip(records.iter().step_by(2).skip(1)).all(|(a, b)| a <= b), "the count read rises");
    assert_eq!((words_digest(&records), digest(&sys)), (LOCKSTEP.0, LOCKSTEP.1), "the records and the state as the plain run left them");
}

#[test]
fn a_sif_kick_reaches_the_iop_at_its_next_service() {
    let mut sys = s54::sif_kick();
    run(&mut sys, 300_000);
    let counts = s54::iop_records(&sys, 8);
    assert!(counts.iter().all(|&c| c > 0), "every packet landed: {counts:?}");
    assert_eq!((counts, digest(&sys)), (SIF_KICK.0.to_vec(), SIF_KICK.1), "the IOP's counts and the state as the plain run left them");
}

#[test]
fn a_cop2_instruction_ends_the_units_rest() {
    let mut sys = s54::vpu();
    run(&mut sys, 10_000);
    let records = s54::ee_records(&sys, 4 * 48);
    assert!(records.chunks(4).all(|r| r[0] == 0x4040_0000), "every quotient 3.0");
    assert_eq!((words_digest(&records), digest(&sys)), (VPU.0, VPU.1), "the records and the state as the plain run left them");
    alike_when_reloaded("vpu", s54::vpu, 10_000, 37);
}

#[test]
fn every_program_runs_alike_from_a_state_saved_between_any_two_steps() {
    alike_when_reloaded("lockstep", s54::lockstep, 20_000, 97);
    alike_when_reloaded("sif-kick", s54::sif_kick, 60_000, 499);
}

/// The console's program run three frames with the device answering `device`: the text kept, the calls caught, the
/// state's digest.
fn console(device: bool) -> (Vec<u8>, u64, u64) {
    let mut m = Machine::new(Media::read(&sapphire::media::program(&[0], sapphire::media::PROGRAM_ORIGIN)).unwrap(), Market::Americas, [None, None]);
    m.sys = Box::new(s54::console());
    m.sys.board.iop.tty.device = device;
    m.booted = true;
    for _ in 0..3 {
        m.advance();
    }
    let t = &m.sys.board.iop.tty;
    (t.text[..t.len].to_vec(), t.calls[1], digest(&m.sys))
}

#[test]
fn the_console_hook_is_met_at_every_call_while_the_iop_runs_on_between_them() {
    for device in [false, true] {
        let (text, calls, state) = console(device);
        assert!(calls > 10 && text == s54::TEXT.repeat(calls as usize), "device {device}: {calls} calls, every one kept whole");
        assert_eq!((calls, state), CONSOLE[usize::from(device)], "device {device}: the calls and the state as the plain run left them");
    }
}

/// Taken on the tree before the step, 2026-10-26.
const LOCKSTEP: (u64, u64) = (11529424670664699004, 4295713031045649312);
const SIF_KICK: ([u32; 8], u64) = ([37, 33, 28, 25, 20, 17, 12, 9], 15252193040340453850);
const VPU: (u64, u64) = (6838493890734116509, 15479450278842447035);
const CONSOLE: [(u64, u64); 2] = [(547, 9424842234595010237), (547, 6188472412445505377)];
