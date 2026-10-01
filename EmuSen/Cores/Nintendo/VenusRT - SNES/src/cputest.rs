//! gilyon's cputest-full and cputest-basic on the 65816 over a minimal bus: LoROM, WRAM, the VRAM port the ROM reports
//! through, $4210's vblank flag and the pad it waits on after a failure. See VenusRT_Native.md §11.5.

use crate::cpu::{Bus, Cpu, Interrupt, pin};

const FRAME: u64 = 357_368;
const VBLANK_START: u64 = 225 * 1364;

struct Minimal {
    rom: Vec<u8>,
    wram: Vec<u8>,
    vram: Vec<u16>,
    vaddr: u16,
    vmain: u8,
    clock: u64,
    nmi_flag: bool,
    pad: u8,
}

impl Minimal {
    fn new(rom: Vec<u8>) -> Minimal {
        Minimal { rom, wram: vec![0; 0x20000], vram: vec![0; 0x8000], vaddr: 0, vmain: 0, clock: 0, nmi_flag: false, pad: 0 }
    }

    fn tick(&mut self, clocks: u64) {
        let before = self.clock % FRAME;
        self.clock += clocks;
        let after = self.clock % FRAME;
        if (before < VBLANK_START && after >= VBLANK_START) || (after < before && VBLANK_START > before) {
            self.nmi_flag = true;
        }
    }

    fn vram_step(&mut self, high: bool) {
        if high == (self.vmain & 0x80 != 0) {
            self.vaddr = self.vaddr.wrapping_add([1, 32, 128, 128][(self.vmain & 3) as usize]);
        }
    }

    fn text(&self, word: usize, n: usize) -> String {
        (0..n).map(|i| (self.vram[word + i] & 0xFF) as u8 as char).collect()
    }
}

impl Bus for Minimal {
    fn read(&mut self, address: u32, _pins: u8) -> u8 {
        self.tick(8);
        let (bank, offset) = ((address >> 16) as u8, address as u16);
        match (bank, offset) {
            (0x7E | 0x7F, _) => self.wram[(address & 0x1FFFF) as usize],
            (b, 0x0000..=0x1FFF) if b & 0x40 == 0 => self.wram[offset as usize],
            (b, 0x4210) if b & 0x40 == 0 => {
                let v = if self.nmi_flag { 0x80 } else { 0 };
                self.nmi_flag = false;
                v
            }
            (b, 0x4212) if b & 0x40 == 0 => 0,
            (b, 0x4218) if b & 0x40 == 0 => self.pad,
            (_, 0x8000..=0xFFFF) => self.rom[(((bank as usize & 0x7F) << 15) | (offset as usize & 0x7FFF)) % self.rom.len()],
            _ => 0,
        }
    }

    fn write(&mut self, address: u32, value: u8, pins: u8) {
        self.tick(8);
        if pins & (pin::VDA | pin::VPA) == 0 {
            return;
        }
        let (bank, offset) = ((address >> 16) as u8, address as u16);
        match (bank, offset) {
            (0x7E | 0x7F, _) => self.wram[(address & 0x1FFFF) as usize] = value,
            (b, 0x0000..=0x1FFF) if b & 0x40 == 0 => self.wram[offset as usize] = value,
            (b, 0x2115) if b & 0x40 == 0 => self.vmain = value,
            (b, 0x2116) if b & 0x40 == 0 => self.vaddr = (self.vaddr & 0xFF00) | value as u16,
            (b, 0x2117) if b & 0x40 == 0 => self.vaddr = (self.vaddr & 0x00FF) | (value as u16) << 8,
            (b, 0x2118) if b & 0x40 == 0 => {
                let w = &mut self.vram[(self.vaddr & 0x7FFF) as usize];
                *w = (*w & 0xFF00) | value as u16;
                self.vram_step(false);
            }
            (b, 0x2119) if b & 0x40 == 0 => {
                let w = &mut self.vram[(self.vaddr & 0x7FFF) as usize];
                *w = (*w & 0x00FF) | (value as u16) << 8;
                self.vram_step(true);
            }
            _ => {}
        }
    }

    fn idle(&mut self, _address: u32, _pins: u8) {
        self.tick(6);
    }

    fn halted(&mut self) {
        self.tick(6);
    }
}

/// The run's end: the final verdict and every test that failed on the way, pressing A past each.
#[derive(Debug)]
pub struct Verdict {
    pub success: bool,
    pub last: String,
    pub failed: Vec<String>,
    pub frames: u64,
    pub unimplemented: bool,
    pub stopped: bool,
}

pub fn run(rom: Vec<u8>, frames: u64) -> Verdict {
    let mut bus = Minimal::new(rom);
    let mut cpu = Cpu::default();
    cpu.interrupt(&mut bus, Interrupt::Reset);
    let mut failed: Vec<String> = Vec::new();
    let mut pressing_until = 0u64;
    while bus.clock < frames * FRAME {
        cpu.step(&mut bus);
        if cpu.unimplemented || cpu.stopped {
            break;
        }
        if bus.text(0x32, 7) == "Success" {
            break;
        }
        if bus.text(0x341, 5) == "Press" && pressing_until == 0 {
            failed.push(bus.text(0x6E, 4));
            for w in 0x341..0x341 + 25 {
                bus.vram[w] = 0;
            }
            pressing_until = bus.clock + FRAME;
            bus.pad = 0x80;
        }
        if pressing_until != 0 && bus.clock > pressing_until {
            bus.pad = 0;
            pressing_until = 0;
        }
    }
    Verdict {
        success: bus.text(0x32, 7) == "Success",
        last: bus.text(0x6E, 4),
        failed,
        frames: bus.clock / FRAME,
        unimplemented: cpu.unimplemented,
        stopped: cpu.stopped,
    }
}

/// The ROM on the machine's own bus, to its verdict: the text at word $0032, the test number at $006E, the frames.
pub fn run_on_machine(image: &[u8], frames: u64) -> (String, String, u64) {
    let mut m = crate::machine::Machine::load_rom(image).expect("an image");
    let text = |m: &crate::machine::Machine, word: usize, n: usize| (0..n).map(|i| (m.sys.ppu.vram[word + i] & 0xFF) as u8 as char).collect::<String>();
    while (m.total_frames() as u64) < frames {
        m.run_frame();
        let verdict = text(&m, 0x32, 7);
        if verdict.starts_with("Success") || verdict.starts_with("Failed") {
            break;
        }
    }
    (text(&m, 0x32, 7), text(&m, 0x6E, 4), m.total_frames() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The same ROMs through the cartridge's LoROM map, the access speeds, the refresh and the real I/O decode.
    #[test]
    fn gilyons_cpu_tests_on_the_machine() {
        for (name, tests) in [("cputest-basic.sfc", "0452"), ("cputest-full.sfc", "0649")] {
            let Some(image) = rom(name) else {
                eprintln!("EMUSEN_VENUSRT_CORPUS unset, not run");
                return;
            };
            let (verdict, number, frames) = run_on_machine(&image, 1000);
            eprintln!("{name} on the machine: {verdict} at {number}, frame {frames}");
            assert_eq!((verdict.as_str(), number.as_str()), ("Success", tests), "{name}");
        }
    }

    fn rom(name: &str) -> Option<Vec<u8>> {
        let root = std::env::var_os("EMUSEN_VENUSRT_CORPUS")?;
        std::fs::read(std::path::Path::new(&root).join("roms/gilyon-snes-tests-v1.4/cputest").join(name)).ok()
    }

    #[test]
    fn gilyons_cpu_tests_on_the_minimal_bus() {
        for (name, tests) in [("cputest-basic.sfc", "0452"), ("cputest-full.sfc", "0649")] {
            let Some(image) = rom(name) else {
                eprintln!("EMUSEN_VENUSRT_CORPUS unset, not run");
                return;
            };
            let v = run(image, 6000);
            eprintln!("{name}: {v:?}");
            assert!(!v.unimplemented && !v.stopped, "{name}: {v:?}");
            assert!(v.success && v.failed.is_empty() && v.last == tests && v.frames < 1000, "{name}: {v:?}");
        }
    }
}
