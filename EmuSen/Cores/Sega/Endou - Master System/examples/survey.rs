//! The library's boards, measured (Endou_Native.md §9.10): each image in a folder, read in place, its board chosen as
//! the core chooses it, then run for a number of frames with every write the Z80 makes to the cartridge's registers or
//! the slot area tallied. Nothing is written back. `cargo run --release --example survey -- <folder> <frames>`; one
//! tab-separated line an image: its CRC-32, size, console, header region, the board and why, then the tallies.

use std::collections::BTreeSet;

use emusen_native::core::{Core, Create, Settings, sys};
use endou::board::crc32;
use endou::bus::Bus;
use endou::machine::Machine;

#[derive(Default)]
struct Tally {
    sega: BTreeSet<u16>,
    max_bank: u8,
    ram_enabled: bool,
    ram_written: usize,
    /// The highest cartridge RAM offset written, as the mapper's bank bit and the slot's address make it.
    ram_top: usize,
    /// The highest slot-area address written with no RAM mapped there.
    slot_top: u16,
    slot_writes: BTreeSet<u16>,
    low_writes: usize,
    high_ram_on: bool,
}

/// The bus with every write tallied before it is made.
struct Spy<'a> {
    bus: &'a mut Bus,
    t: &'a mut Tally,
}

impl beryl_z80::Bus for Spy<'_> {
    fn fetch(&mut self, a: u16, r: u16) -> u8 {
        self.bus.fetch(a, r)
    }
    fn read(&mut self, a: u16) -> u8 {
        self.bus.read(a)
    }
    fn write(&mut self, a: u16, v: u8) {
        match a {
            0xFFFC => {
                self.t.sega.insert(a);
                self.t.ram_enabled |= v & 0x08 != 0;
                self.t.high_ram_on |= v & 0x10 != 0;
            }
            0xFFFD..=0xFFFF => {
                self.t.sega.insert(a);
                self.t.max_bank = self.t.max_bank.max(v);
            }
            0x8000..=0xBFFF if self.bus.cart.regs[0] & 0x08 != 0 => {
                self.t.ram_written += 1;
                let bank = if self.bus.cart.regs[0] & 0x04 != 0 { 0x4000 } else { 0 };
                self.t.ram_top = self.t.ram_top.max(bank + (a as usize & 0x3FFF));
            }
            0x8000..=0xBFFF => {
                self.t.slot_writes.insert(a & 0xF000);
                self.t.slot_top = self.t.slot_top.max(a);
            }
            0x0000..=0x7FFF => {
                self.t.low_writes += 1;
                self.t.slot_writes.insert(a & 0xF000);
            }
            _ => {}
        }
        self.bus.write(a, v)
    }
    fn input(&mut self, p: u16) -> u8 {
        self.bus.input(p)
    }
    fn output(&mut self, p: u16, v: u8) {
        self.bus.output(p, v)
    }
    fn idle(&mut self, t: u32) {
        self.bus.idle(t)
    }
    fn int_line(&mut self) -> bool {
        self.bus.int_line()
    }
    fn nmi_edge(&mut self) -> bool {
        self.bus.nmi_edge()
    }
    fn acknowledge(&mut self) -> u8 {
        self.bus.acknowledge()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let frames: u64 = args.get(2).map_or(600, |f| f.parse().expect("a frame count"));
    let mut names: Vec<_> = std::fs::read_dir(&args[1]).expect("the folder").filter_map(|e| e.ok()).map(|e| e.path()).collect();
    names.sort();
    println!("crc\tsize\tconsole\tregion\tmapper\twhy\tsega_regs\tmax_bank\tram_on\tram_writes\tram_top\tslot_top\tc000_ram\tlow_writes\tslot_writes\tname");
    for path in names {
        let Ok(image) = std::fs::read(&path) else { continue };
        let request = Create { image: &image, settings: Settings::default(), files: Vec::new(), pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
        let Ok(mut m) = Machine::create(&request) else {
            println!("refused\t{}\t\t\t\t\t\t\t\t\t\t\t\t{}", image.len(), path.display());
            continue;
        };
        let mut t = Tally::default();
        let end = m.frame_clocks() * frames;
        while m.bus.clock < end {
            m.cpu.step(&mut Spy { bus: &mut m.bus, t: &mut t });
        }
        let board = m.bus.cart.board;
        let region = m.media.header.map_or("-".to_owned(), |h| format!("{:X}", h.region));
        let slots: Vec<String> = t.slot_writes.iter().map(|a| format!("{a:04X}")).collect();
        let sega: Vec<String> = t.sega.iter().map(|a| format!("{a:04X}")).collect();
        println!(
            "{:08X}\t{}\t{}\t{}\t{:?}\t{:?}\t{}\t{}\t{}\t{}\t{:04X}\t{:04X}\t{}\t{}\t{}\t{}",
            crc32(&m.bus.cart.rom),
            image.len(),
            m.console().id(),
            region,
            board.mapper,
            board.why,
            sega.join(","),
            t.max_bank,
            t.ram_enabled,
            t.ram_written,
            t.ram_top,
            t.slot_top,
            t.high_ram_on,
            t.low_writes,
            slots.join(","),
            path.file_name().unwrap().to_string_lossy()
        );
        let _ = Core::frame_count(&m);
    }
}
