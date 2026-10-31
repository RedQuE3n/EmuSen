//! The GS drawn on the Vulkan device (Sapphire_Gs.md): generated streams of every primitive, environment, texture (from
//! the frame's own pages too), CLUT load and transfer drawn by the software rasteriser and by the device, the local
//! memory and every value read back equal at every check; and a state saved mid-stream loading into a device GS and
//! continuing as the software one does, both with records that read where they draw given to the processor; and with
//! them read from a snapshot, the same memory from two runs. Each test stands down when no Vulkan device can be made.

use emusen_native::{State, StateReader, StateWriter};
use sapphire::gs::Gs;

mod gsgen;

use gsgen::{run, stream};

fn device() -> Option<Gs> {
    let mut gs = Gs::new();
    gs.set_workers(1);
    match gs.set_device(false) {
        Ok(()) => {
            gs.set_exact_feedback(true);
            Some(gs)
        }
        Err(e) => {
            eprintln!("no device, the test stands down: {e}");
            None
        }
    }
}

fn software() -> Gs {
    let mut gs = Gs::new();
    gs.set_workers(1);
    gs
}

#[test]
fn generated_streams_leave_the_memory_and_every_read_back_as_the_software_rasteriser_does() {
    let Some(_) = device() else { return };
    let (mut records, mut stretches, mut fallbacks) = (0, 0, 0);
    for seed in 0..16u64 {
        let steps = stream(seed, 700);
        let want = run(&mut software(), &steps);
        let mut gs = device().expect("a device");
        let got = run(&mut gs, &steps);
        let first = want.iter().zip(&got).position(|(a, b)| a != b);
        assert!(first.is_none() && want.len() == got.len(), "seed {seed}: the first difference at entry {first:?} of {}", want.len());
        let (c, _) = gs.device_counts().expect("the device's counts");
        records += c.plan.records;
        stretches += c.plan.stretches;
        fallbacks += c.plan.fallback.iter().sum::<u64>() + c.plan.snapshot[0];
    }
    assert!(records > 800 && stretches > 300, "records {records}, stretches {stretches}");
    assert!(fallbacks > 0, "some primitives were drawn by the processor or read a snapshot");
}

fn save(gs: &Gs) -> Vec<u8> {
    let mut c = StateWriter::counter();
    gs.write_state(&mut c);
    let mut v = vec![0; c.len()];
    gs.write_state(&mut StateWriter::new(&mut v));
    v
}

#[test]
fn a_state_saved_mid_stream_loads_into_the_device_and_continues_as_the_software_rasteriser_does() {
    let Some(_) = device() else { return };
    for seed in 20..23u64 {
        let steps = stream(seed, 600);
        let (a, b) = steps.split_at(300);
        let mut one = software();
        let mut want = run(&mut one, a);
        want.extend(run(&mut one, b));
        let mut gs = device().expect("a device");
        let mut got = run(&mut gs, a);
        let bytes = save(&gs);
        let mut loaded = device().expect("a device");
        loaded.read_state(&mut StateReader::new(&bytes)).expect("the state loads");
        got.extend(run(&mut loaded, b));
        assert_eq!(want, got, "seed {seed}");
    }
}

#[test]
fn with_snapshots_two_runs_leave_the_same_memory_and_read_backs() {
    let Some(_) = device() else { return };
    let mut snapshots = 0;
    for seed in 0..6u64 {
        let steps = stream(seed, 700);
        let runs: Vec<(Vec<u64>, u64)> = (0..2)
            .map(|_| {
                let mut gs = device().expect("a device");
                gs.set_exact_feedback(false);
                let got = run(&mut gs, &steps);
                (got, gs.device_counts().expect("counts").0.plan.snapshot[0])
            })
            .collect();
        assert_eq!(runs[0], runs[1], "seed {seed}");
        snapshots += runs[0].1;
    }
    assert!(snapshots > 0, "some records read a snapshot");
}
