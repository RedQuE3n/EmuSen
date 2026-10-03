//! The state's bytes hashed every `step` frames, for proving a change leaves the machine where it was.
//! `state_hash <rom> <frames> [step]` prints one FNV-1a hash a line.
use emusen_native::ffi::StateMachine;

#[path = "common/load.rs"]
mod load;

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let frames: u32 = a[2].parse().unwrap();
    let step: u32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(60);
    let mut m = load::machine(&a[1]);
    let mut state = vec![0u8; m.state_size()];
    for f in 1..=frames {
        m.run_frame();
        if f % step == 0 {
            m.save_state(&mut state).unwrap();
            let h = state.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3));
            println!("{f} {h:016x}");
        }
    }
}
