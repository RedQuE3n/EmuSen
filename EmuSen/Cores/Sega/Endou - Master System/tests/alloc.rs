//! A frame with its sound drained allocates nothing once the machine's buffers have their size (Endou_Native.md §12.5).
//! A test binary of its own, so that the count is this test's alone.

#[path = "../programs/images.rs"]
#[allow(dead_code)]
mod images;

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use emusen_native::core::{Core, Create, Settings, sys};
use endou::machine::Machine;

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

#[test]
fn a_sounding_frame_allocates_nothing() {
    for (program, model) in [(&images::VOICES, "sms2"), (&images::VOICES_SG, "sg1000"), (&images::VOICES, "mark3")] {
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
        let before = ALLOCATIONS.load(Ordering::Relaxed);
        for i in 4..724 {
            frame(&mut m, i);
        }
        assert_eq!(ALLOCATIONS.load(Ordering::Relaxed) - before, 0, "{model}: allocations in 720 frames of the voices program");
    }
}
