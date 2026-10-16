//! MarsRT on the core ABI v1: what the conformance kit asks of a run, on a game's copy behind `EMUSEN_MARSRT_STATES`. See Mars_Native.md §7.

use std::sync::Arc;

use emusen_native::ffi::StateMachine;

use crate::ffi::Core;
use crate::machine::Machine;
use crate::rom::RomImage;

fn game(name: &str) -> Option<Core> {
    let folder = std::env::var("EMUSEN_MARSRT_STATES").ok()?;
    let image = RomImage::from_image(&std::fs::read(std::path::Path::new(&folder).join(name)).ok()?).unwrap();
    let mut core = Core::new(Machine::load_rom(Arc::new(image), true, None, None));
    core.apply(crate::ffi::v1::Values::defaults());
    Some(core)
}

fn frames(core: &mut Core, n: usize) {
    for _ in 0..n {
        core.advance();
        core.present_profiled();
    }
}

fn state(core: &mut Core) -> Vec<u8> {
    core.machine.settle();
    let mut out = vec![0; core.state_size()];
    let n = core.save_state(&mut out).unwrap();
    out.truncate(n);
    out
}

/// C8's question: a machine loaded with a state of the middle of a run ends where the run ends, byte for byte, since a load leaves
/// the battery files' `Dirty` as the state says (Mars_Native.md §7.4).
#[test]
fn a_state_loaded_in_the_middle_of_a_run_ends_where_the_run_ends() {
    let (Some(mut a), Some(mut d)) = (game("sm64.z64"), game("sm64.z64")) else { return };
    frames(&mut a, 300);
    let middle = state(&mut a);
    frames(&mut a, 300);
    d.load_state(&middle).unwrap();
    assert_eq!(state(&mut d), middle, "save, load, save is not the state saved");
    frames(&mut d, 300);
    assert!(state(&mut a) == state(&mut d), "the loaded run ends elsewhere");
}
