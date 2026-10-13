//! The crate on a real device: enumeration with a driver and without, buffers, a program, the three kinds of submission and a device
//! chosen by name. Each test runs on the first device by rank and stands down with none, which is a CI runner's case. See
//! Hardware/Shared/EmuSen_Gpu.md §7.

use std::process::Command;

use emusen_gpu::{AHEAD, GpuDevice, Submission, Where};

/// `tests/shaders/triple.comp`: each word becomes three times itself plus the push constant, 64 invocations a group.
const TRIPLE: &[u8] = include_bytes!("shaders/triple.spv");
const CHILD: &str = "EMUSEN_GPU_TEST_CHILD";

fn device() -> Option<GpuDevice> {
    let names = GpuDevice::device_names();
    let Some(first) = names.first() else {
        eprintln!("no Vulkan device: not run");
        return None;
    };
    Some(GpuDevice::try_create(Some(first)).unwrap_or_else(|report| panic!("{report}")))
}

fn tripled(seed: u32, add: u32) -> u32 {
    seed.wrapping_mul(3).wrapping_add(add)
}

fn seeds(count: usize, salt: u32) -> Vec<u32> {
    (0..count as u32).map(|i| i.wrapping_mul(0x9E37_79B9) ^ salt).collect()
}

#[test]
fn the_devices_are_listed_in_rank_order_and_the_first_opens_with_its_report() {
    let names = GpuDevice::device_names();
    let Some(device) = device() else {
        let report = GpuDevice::try_create(None).err().expect("no device listed, and one opened");
        assert!(report.starts_with("Vulkan is not available") || report.starts_with("no Vulkan device"), "{report}");
        return;
    };
    assert_eq!(device.name(), names[0]);
    let report = device.report();
    assert!(report.starts_with(&format!("{}, Vulkan 1.", names[0])), "{report}");
    assert!(device.max_buffer_bytes() >= 1 << 27, "a binding of {} bytes is below the specification's least", device.max_buffer_bytes());
    eprintln!("{report}; {} devices: {names:?}", names.len());
}

/// Run alone as a child with every driver hidden from the loader: nothing is listed and the device is refused with a reason.
#[test]
fn child_without_a_driver() {
    if std::env::var(CHILD).is_err() {
        return;
    }
    assert!(GpuDevice::device_names().is_empty(), "the loader listed {:?} with every driver hidden", GpuDevice::device_names());
    let report = GpuDevice::try_create(None).err().expect("a device opened with every driver hidden");
    assert!(report.starts_with("Vulkan is not available") || report == "no Vulkan device offers a compute queue", "{report}");
    println!("refused: {report}");
}

#[test]
fn without_a_driver_nothing_is_listed_and_the_device_is_refused_with_a_reason() {
    let exe = std::env::current_exe().expect("the test binary");
    let out = Command::new(exe)
        .args(["--exact", "child_without_a_driver", "--nocapture", "--test-threads=1"])
        .env(CHILD, "1")
        .env("VK_LOADER_DRIVERS_DISABLE", "*")
        .output()
        .expect("the child ran");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "the child failed: {text}{}", String::from_utf8_lossy(&out.stderr));
    assert!(text.contains("refused: "), "the child did not run its test: {text}");
    eprintln!("{}", text.lines().find(|l| l.starts_with("refused: ")).unwrap_or_default());
}

#[test]
fn a_buffer_round_trips_through_device_memory_and_is_filled() {
    let Some(device) = device() else { return };
    let words = 4099;
    let source = device.create_buffer(words as u64 * 4, Where::Host).expect("a host buffer");
    let middle = device.create_buffer(words as u64 * 4, Where::Device).expect("a device buffer");
    let back = device.create_buffer(words as u64 * 4, Where::Host).expect("a host buffer");
    assert_eq!(source.bytes(), words as u64 * 4);
    let sent = seeds(words, 0x5A5A_0001);
    source.slice_mut::<u32>().copy_from_slice(&sent);
    device.submit(|c| {
        c.copy(&source, &middle, 0, 0, 0);
        c.copy(&middle, &back, 0, 0, 0);
    });
    assert_eq!(back.slice::<u32>(), &sent[..], "host to device to host");

    // A part at offsets, then a fill of the whole.
    device.submit(|c| {
        c.fill(&middle, 0xDEAD_BEEF);
        c.copy(&source, &middle, 64, 16, 128);
        c.copy(&middle, &back, 0, 0, 0);
    });
    let got = back.slice::<u32>();
    for (i, &word) in got.iter().enumerate() {
        let expected = if (32..48).contains(&i) { sent[i - 32 + 4] } else { 0xDEAD_BEEF };
        assert_eq!(word, expected, "word {i}");
    }
    for buffer in [source, middle, back] {
        device.destroy_buffer(buffer);
    }
}

#[test]
fn a_program_runs_over_a_buffer_with_its_push_constants() {
    let Some(device) = device() else { return };
    assert!(device.create_program(&TRIPLE[..TRIPLE.len() - 1], 1, 4).is_err(), "a part of a word of SPIR-V was taken");
    let program = device.create_program(TRIPLE, 1, 4).expect("the program");
    // Not a whole number of groups, so the last group's bound check is reached.
    let words = 1000;
    let values = device.create_buffer(words as u64 * 4, Where::Host).expect("a host buffer");
    let sent = seeds(words, 7);
    values.slice_mut::<u32>().copy_from_slice(&sent);
    device.submit(|c| {
        assert_eq!(c.submission(), Submission::Now);
        c.dispatch(&program, &[&values], &7u32, (words as u32).div_ceil(64), 1, 1);
    });
    let expected: Vec<u32> = sent.iter().map(|&s| tripled(s, 7)).collect();
    assert_eq!(values.slice::<u32>(), &expected[..]);
    device.destroy_buffer(values);
    device.destroy_program(program);
}

#[test]
fn submissions_ahead_go_out_without_waiting_and_are_waited_for_by_slot() {
    let Some(device) = device() else { return };
    let program = device.create_program_with_sets(TRIPLE, 1, 4, AHEAD).expect("the program");
    let words = 1 << 16;
    let buffers: Vec<_> = (0..AHEAD).map(|_| device.create_buffer(words as u64 * 4, Where::Host).expect("a host buffer")).collect();
    let mut expected = Vec::new();
    for round in 0..3u32 {
        for (slot, buffer) in buffers.iter().enumerate() {
            // The buffer is written only before its slot's first submission; each later one waits for the slot's last itself.
            if round == 0 {
                let sent = seeds(words, slot as u32);
                buffer.slice_mut::<u32>().copy_from_slice(&sent);
                expected.push(sent);
            }
            let add = round * 10 + slot as u32;
            expected[slot].iter_mut().for_each(|e| *e = tripled(*e, add));
            device.submit_ahead(slot, |c| {
                assert_eq!(c.submission(), Submission::Ahead(slot));
                c.dispatch_in(&program, slot, &[buffer], &add, (words as u32).div_ceil(64), 1, 1);
            });
        }
    }
    for (slot, buffer) in buffers.iter().enumerate() {
        device.wait_ahead(slot);
        assert!(device.finished(Submission::Ahead(slot)));
        assert_eq!(buffer.slice::<u32>(), &expected[slot][..], "slot {slot}");
    }
    for buffer in buffers {
        device.destroy_buffer(buffer);
    }
    device.destroy_program(program);
}

#[test]
fn a_pending_submission_is_waited_for_by_a_thread_that_does_not_submit() {
    let Some(device) = device() else { return };
    let program = device.create_program(TRIPLE, 1, 4).expect("the program");
    let words = 1 << 18;
    let values = device.create_buffer(words as u64 * 4, Where::Host).expect("a host buffer");
    let sent = seeds(words, 3);
    values.slice_mut::<u32>().copy_from_slice(&sent);
    device.submit_pending(|c| {
        assert_eq!(c.submission(), Submission::Pending);
        c.dispatch(&program, &[&values], &11u32, (words as u32).div_ceil(64), 1, 1);
    });
    let pending = device.pending();
    std::thread::spawn(move || pending.wait()).join().expect("the waiting thread");
    assert!(device.finished(Submission::Pending));
    let expected: Vec<u32> = sent.iter().map(|&s| tripled(s, 11)).collect();
    assert_eq!(values.slice::<u32>(), &expected[..]);

    // The next pending submission waits for the last before it records; a waited one leaves it alone.
    device.submit_pending(|c| c.dispatch(&program, &[&values], &0u32, (words as u32).div_ceil(64), 1, 1));
    device.submit(|c| c.fill(&values, 1));
    device.wait_for_pending();
    assert!(values.slice::<u32>().iter().all(|&v| v == 1), "the fill after the pending dispatch was not last");
    device.destroy_buffer(values);
    device.destroy_program(program);
}

#[test]
fn a_device_is_chosen_by_any_part_of_its_name_in_any_case_and_refused_by_one_it_lacks() {
    let names = GpuDevice::device_names();
    let Some(first) = names.first() else {
        eprintln!("no Vulkan device: not run");
        return;
    };
    let device = GpuDevice::try_create(Some(&first.to_uppercase())).unwrap_or_else(|report| panic!("{report}"));
    assert_eq!(device.name(), first);
    drop(device);
    let wanted = "no device is called this";
    let report = GpuDevice::try_create(Some(wanted)).err().expect("a device opened by a name none has");
    assert_eq!(report, format!("no Vulkan device is named like \"{wanted}\""));
}
