//! The debugger's points of a frame allocate nothing once warm with nothing armed, and the seeded fault of a registry
//! cloned each frame is seen to (EmuSen_Backend.md §29.4, §29.34). The core's own allocations are its standard
//! library's, not this binary's, so the count is the host's and the debugger's alone.

use std::alloc::{GlobalAlloc, Layout, System};
use std::path::PathBuf;
use std::cell::Cell;

use emusen_core_host::{Create, Library, Machine};
use emusen_crystal_debug::{Debugger, Ran};

struct Counting;

thread_local! {
    /// This thread's count, so a test running beside another counts its own.
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

fn count() -> usize {
    ALLOCATIONS.with(Cell::get)
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.with(|c| c.set(c.get() + 1));
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.with(|c| c.set(c.get() + 1));
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

fn example(name: &str) -> PathBuf {
    let profile = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    profile.join("examples").join(if cfg!(windows) { format!("{name}.dll") } else if cfg!(target_os = "macos") { format!("lib{name}.dylib") } else { format!("lib{name}.so") })
}

/// The frame's (b) points in §29.4's order, the machine's advance between them.
fn frame(d: &mut Debugger, m: &mut Machine) {
    d.refresh_cheats(m);
    let ran = if d.armed() { d.run_frame(m, false).unwrap() } else { m.advance().map(|_| Ran::Completed).unwrap() };
    if ran == Ran::Completed {
        d.frame_end(m);
    }
    d.refresh(m, 1.0);
}

#[test]
fn the_debuggers_points_allocate_nothing_in_a_frame() {
    let lib = Library::load(&example("debug_test_core")).unwrap();
    lib.check().unwrap();
    let image = [b'V', b'1', b'T', b'C', 3, 0, 1, 2, 3];
    let mut m = Machine::create(lib, &Create::new(&image)).unwrap();
    let mut d = Debugger::new();
    d.attach(&mut m);
    d.cheats.add_ram_poke("RAM", 8, 0x99, "poke", true).unwrap();
    // A poke the core cannot gate, which the host writes at each frame's end.
    let wide = emusen_systems::cheats::Write { width: 2, ..emusen_systems::cheats::Write::poke("RAM", 9, 0x1234, 2) };
    d.cheats.add(emusen_systems::cheats::Kind::RamPoke, vec![wide], None, "wide", true).unwrap();
    for _ in 0..200 {
        frame(&mut d, &mut m);
    }
    let before = count();
    for _ in 0..1000 {
        frame(&mut d, &mut m);
    }
    assert_eq!(count() - before, 0, "allocations in 1,000 frames");
}

#[test]
fn a_cloned_registry_a_frame_is_seen() {
    let lib = Library::load(&example("debug_test_core")).unwrap();
    lib.check().unwrap();
    let image = [b'V', b'1', b'T', b'C', 3, 0, 1, 2, 3];
    let mut m = Machine::create(lib, &Create::new(&image)).unwrap();
    let mut d = Debugger::new();
    d.attach(&mut m);
    d.cheats.add_ram_poke("RAM", 8, 0x99, "poke", true).unwrap();
    for _ in 0..200 {
        frame(&mut d, &mut m);
    }
    let before = count();
    for _ in 0..10 {
        frame(&mut d, &mut m);
        std::hint::black_box(d.cheats.clone());
    }
    assert!(count() - before >= 10, "the count sees a clone a frame");
}
