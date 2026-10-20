//! An armed frame with the profile drained each frame allocates nothing once the hooks have their size
//! (EmuSen_NativeCores.md §12.8). Each thread counts its own allocations, so tests running beside each other do not add
//! to each other's counts.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use emusen_native::debug::{Call, Hooks, drain, drain_profile, flag, kind};

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

/// A frame of `routines` calls, each charged its own count with one nested call in every fourth, entered in an order
/// that is not the owners' so that new owners land in the middle of the profile; then the host's drain.
fn frame(h: &mut Hooks, routines: u32, calls: &mut [u32], pairs: &mut [i64]) -> usize {
    h.record(0);
    for r in 0..routines {
        let target = 0x1000 + (r * 37 % routines) * 0x10;
        h.note_call(target - 3, target, kind::CALL);
        for _ in 0..=r % 5 {
            h.record(target);
        }
        if r % 4 == 0 {
            h.note_call(target + 4, 0x8000 + r, kind::CALL);
            h.record(0x8000 + r);
            h.note_return();
        }
        h.note_return();
        h.record(0);
    }
    drain(&mut h.calls_log, calls, |c: &Call| [c.kind, c.source, c.target]);
    drain_profile(h, pairs)
}

fn armed_frames(routines: u32) -> usize {
    let mut h = Hooks::new(&[16]);
    h.configure(flag::CALLS | flag::PROFILING, i32::MIN, -1);
    let (mut calls, mut pairs) = (vec![0u32; 1 << 16], vec![0i64; 1 << 12]);
    for _ in 0..4 {
        frame(&mut h, routines, &mut calls, &mut pairs);
    }
    let before = allocations();
    for _ in 0..720 {
        let owners = 1 + routines as usize + routines.div_ceil(4) as usize;
        assert_eq!(frame(&mut h, routines, &mut calls, &mut pairs), owners);
    }
    allocations() - before
}

#[test]
fn a_profiled_frame_allocates_nothing() {
    for routines in [1, 40, 300] {
        assert_eq!(armed_frames(routines), 0, "allocations in 720 profiled frames of {routines} routines");
    }
}
