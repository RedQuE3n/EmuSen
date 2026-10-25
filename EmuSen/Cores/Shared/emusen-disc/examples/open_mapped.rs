//! Opens a disc image mapped read-only, as a host lends it, and times each step (EmuSen_Disc.md §14.5, Sapphire_Plan.md
//! P13): `open_mapped <image> [dvd|cd] [copy]`. Prints the map, the open (the layer break found), the first sector and
//! a thousand sectors read, and with `copy` the time to read the whole file into memory instead. Never writes the file.

use std::ffi::c_void;
use std::fs::File;
use std::os::fd::AsRawFd;
use std::time::Instant;

use emusen_disc::{IsoForm, Kind, Lone, Medium, open_medium};

unsafe extern "C" {
    fn mmap(addr: *mut c_void, len: usize, prot: i32, flags: i32, fd: i32, offset: i64) -> *mut c_void;
    fn munmap(addr: *mut c_void, len: usize) -> i32;
}

const PROT_READ: i32 = 1;
const MAP_PRIVATE: i32 = 2;

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = &args[0];
    let kind = if args.get(1).map(String::as_str) == Some("cd") { Kind::Cd } else { Kind::Dvd };
    let t = Instant::now();
    let file = File::open(path).expect("the image");
    let len = file.metadata().unwrap().len() as usize;
    // SAFETY: a private read-only mapping of a file opened for reading, unmapped below after its last use.
    let at = unsafe { mmap(std::ptr::null_mut(), len, PROT_READ, MAP_PRIVATE, file.as_raw_fd(), 0) };
    assert!(at as isize != -1, "mmap refused");
    // SAFETY: the mapping is `len` bytes, readable, and lives until `munmap` after the disc is dropped.
    let bytes: &[u8] = unsafe { std::slice::from_raw_parts(at as *const u8, len) };
    let mapped = ms(t);
    let t = Instant::now();
    let mut medium = open_medium(bytes, Vec::new(), IsoForm::Mode2Form1, Lone::Detect, kind).expect("a disc");
    let opened = ms(t);
    let (what, first, thousand) = match &mut medium {
        Medium::Dvd(d) => {
            let what = format!("DVD, {} sectors, {} layer(s), layer break {:?}", d.sectors(), d.layers(), d.layer_break());
            let mut u = [0u8; 2048];
            let t = Instant::now();
            d.read_user(16, &mut u);
            let first = ms(t);
            let t = Instant::now();
            let step = (d.sectors() / 1000).max(1);
            (0..1000).for_each(|k| {
                d.read_user(k * step % d.sectors(), &mut u);
            });
            (what, first, ms(t))
        }
        Medium::Cd(d) => {
            let what = format!("CD, lead-out at {}", d.lead_out());
            let mut r = [0u8; 2352];
            let t = Instant::now();
            d.read(166, &mut r);
            let first = ms(t);
            let t = Instant::now();
            (0..1000).for_each(|k| {
                d.read(150 + k, &mut r);
            });
            (what, first, ms(t))
        }
    };
    drop(medium);
    // SAFETY: nothing borrows the mapping any more.
    unsafe { munmap(at, len) };
    println!("{path}: {len} bytes, {what}");
    println!("map {mapped:.3} ms, open {opened:.3} ms, first sector {first:.3} ms, 1,000 sectors spread over the disc {thousand:.3} ms");
    if args.iter().any(|a| a == "copy") {
        let t = Instant::now();
        let all = std::fs::read(path).expect("read");
        println!("copy of {} bytes into memory {:.1} ms", all.len(), ms(t));
    }
}
