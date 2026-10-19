//! A frame through the host allocates nothing once the machine's buffers have their size (EmuSen_Backend.md §27.3).
//! The core's own allocations are its standard library's, not this binary's, so the count is the host's alone.

use std::alloc::{GlobalAlloc, Layout, System};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use emusen_core_host::{Create, InputLatch, Library, Machine};

struct Counting;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn example(name: &str) -> PathBuf {
    let profile = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    profile.join("examples").join(if cfg!(windows) { format!("{name}.dll") } else if cfg!(target_os = "macos") { format!("lib{name}.dylib") } else { format!("lib{name}.so") })
}

#[test]
fn a_frame_allocates_nothing() {
    let lib = Library::load(&example("host_test_core")).unwrap();
    lib.check().unwrap();
    let image = [b'V', b'1', b'T', b'C', 3, 0, 1, 2, 3];
    let mut m = Machine::create(lib, &Create::new(&image)).unwrap();
    let latch = InputLatch::new(2, 1);
    let (mut picture, mut sound, mut registers, mut counters, mut ram) = (vec![0u8; 1 << 20], vec![0i16; 8192], vec![0i64; 64], vec![0i64; 64], [0u8; 16]);
    let mut frame = |m: &mut Machine, i: u32| {
        latch.hold(0, i & 1, 1);
        latch.move_axis(0, 0, i as f64 / 100.0);
        latch.send(m);
        m.advance().unwrap();
        let fi = m.frame_info().unwrap();
        m.frame_copy(&mut picture[..fi.bytes as usize]).unwrap();
        m.present().unwrap();
        m.audio_drain(&mut sound).unwrap();
        m.events().unwrap();
        m.registers(0, &mut registers).unwrap();
        m.telemetry(&mut counters).unwrap();
        m.space_read(0, 0, &mut ram).unwrap();
    };
    for i in 0..4 {
        frame(&mut m, i);
    }
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    for i in 4..124 {
        frame(&mut m, i);
    }
    assert_eq!(ALLOCATIONS.load(Ordering::Relaxed) - before, 0, "allocations in 120 frames");
}
