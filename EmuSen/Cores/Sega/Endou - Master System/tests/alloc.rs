//! A frame with its sound drained allocates nothing once the machine's buffers have their size (Endou_Native.md §12.5),
//! the FM unit's included (§16.6), nor one through the armed debugger, its profile included (§15.6). Each thread
//! counts its own allocations, so that tests running beside each other do not add to each other's counts.

#[path = "../programs/images.rs"]
#[allow(dead_code)]
mod images;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use emusen_native::core::{Core, Create, Settings, sys};
use endou::machine::Machine;

struct Counting;

thread_local! {
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

fn count() {
    let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
}

fn allocations() -> usize {
    ALLOCATIONS.with(Cell::get)
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count();
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        count();
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

#[test]
fn a_sounding_frame_allocates_nothing() {
    for (program, model) in [(&images::VOICES, "sms2"), (&images::VOICES_SG, "sg1000"), (&images::VOICES, "mark3"), (&images::OPLL, "jsms"), (&images::OPLL, "mark3fm")] {
        let image = images::image(program);
        let settings = Settings::from_pairs(vec![("model".to_owned(), model.to_owned())]);
        let request = Create { image: &image, settings, files: Vec::new(), pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
        let mut m = Machine::create(&request).unwrap();
        let mut out = vec![0i16; 8192];
        let mut frame = |m: &mut Machine, i: u32| {
            m.set_buttons(0, i & 1, 1).unwrap();
            Core::advance(m, &mut 0).unwrap();
            let _ = m.frame();
            m.drain_audio(&mut out, usize::MAX);
        };
        for i in 0..4 {
            frame(&mut m, i);
        }
        let before = allocations();
        for i in 4..724 {
            frame(&mut m, i);
        }
        assert_eq!(allocations() - before, 0, "{model}: allocations in 720 frames of the {} program", program.name);
    }
}

/// Frames through the debugger's observer with its tables armed and nothing to stop for, the logs drained into a host's
/// buffers each frame as the Rust debugger drains them: the watches' stores on the bus and in RAM, the calls, and the
/// coverage (Endou_Native.md §15.6). `profiling` adds the profile, drained the same way.
fn armed_frames(profiling: bool) -> usize {
    use emusen_native::debug::{Call, Write, drain, drain_coverage, drain_profile, flag};
    let image = images::image(&images::VOICES);
    let request = Create { image: &image, settings: Settings::default(), files: Vec::new(), pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
    let mut m = Machine::create(&request).unwrap();
    let flags = flag::CALLS | flag::WRITES | 1 << flag::COVERAGE | if profiling { flag::PROFILING } else { 0 };
    m.hooks.configure(flags, i32::MIN, -1);
    m.hooks.set_breakpoints(&[-2, -2]);
    m.hooks.set_ranges(0, &[1, 0, 0x1FFF, 0, 0xC000, 0xFFFF]);
    m.hooks.set_ranges(1, &[3, 0, 0]);
    let (mut out, mut events, mut bits, mut profile) = (vec![0i16; 8192], vec![0u32; 1 << 20], vec![0u8; 1 << 13], vec![0i64; 1 << 12]);
    let mut frame = |m: &mut Machine| {
        assert_eq!(m.run_frame_debug(0), 0);
        let _ = m.frame();
        m.drain_audio(&mut out, usize::MAX);
        drain(&mut m.hooks.writes_log, &mut events, |w: &Write| [w.space, w.address, w.value, w.pc]);
        drain(&mut m.hooks.calls_log, &mut events, |c: &Call| [c.kind, c.source, c.target]);
        drain_coverage(&mut m.hooks, 0, &mut bits, &mut 0);
        drain_profile(&mut m.hooks, &mut profile);
    };
    // A second to warm: the logs grow to the largest frame's stores and calls of the program's first moments, at frame 22.
    for _ in 0..60 {
        frame(&mut m);
    }
    let before = allocations();
    for _ in 60..780 {
        frame(&mut m);
    }
    allocations() - before
}

#[test]
fn an_armed_debugger_s_frame_allocates_nothing() {
    assert_eq!(armed_frames(false), 0, "allocations in 720 armed frames of the voices program");
}

/// The profile is cleared in place by each drain, so it allocates nothing either (EmuSen_NativeCores.md §12.8).
#[test]
fn a_profiled_frame_allocates_nothing() {
    assert_eq!(armed_frames(true), 0, "allocations in 720 armed frames of the voices program with the profile drained");
}
