//! The Z80's bus (Endou_Native.md §9.3, §9.6): the memory map with port $3E's enables, the I/O ports decoded by A7, A6
//! and A0, and the clock. The bus owns the time: every machine cycle the Z80 reports advances it, and the devices are
//! brought to it when an access reaches them. The VDP's ports are a stand-in until stage 3 builds `beryl-sms-vdp`.

use beryl_sn76489::Sn76489;

use crate::cart::Cartridge;
use crate::io::Io;
use crate::model::Model;

/// The Z80's clock over the PSG's internal one [R SMS Power!'s "SN76489"; `Beryl_SN76489.md` §2].
pub const PSG_DIVIDER: u64 = 16;

/// Port $3E's bits, each enabling its part when clear [R MacDonald's `smstech` §4; Sega's Mark III manual, "Memory
/// enables"].
pub const CARTRIDGE_OFF: u8 = 1 << 6;
pub const WORK_RAM_OFF: u8 = 1 << 4;
pub const IO_OFF: u8 = 1 << 2;

#[derive(Clone, Debug)]
pub struct Bus {
    pub model: Model,
    /// Z80 clocks since power-on; the master clock is fifteen times it.
    pub clock: u64,
    pub ram: Vec<u8>,
    pub cart: Cartridge,
    pub io: Io,
    pub psg: Sn76489,
    /// The PSG's internal clocks run so far.
    pub psg_ticks: u64,
    /// Port $3E as last written.
    pub memory_control: u8,
    /// The last byte on the data bus, which an access nothing drives reads on the consoles that float it.
    pub data_bus: u8,
    /// A falling edge on NMI not yet taken.
    pub nmi: bool,
    pub vram: Vec<u8>,
    pub cram: Vec<u8>,
}

impl Bus {
    pub fn new(model: Model, cart: Cartridge, ram: usize, psg: Sn76489, cram: usize) -> Bus {
        Bus {
            model,
            clock: 0,
            ram: vec![0; ram],
            cart,
            io: Io::default(),
            psg,
            psg_ticks: 0,
            memory_control: model.memory_control_at_start(),
            data_bus: 0xFF,
            nmi: false,
            vram: vec![0; 0x4000],
            cram: vec![0; cram],
        }
    }

    /// A read nothing drives: the bus's last byte on the consoles that float it, $FF on the Master System II.
    fn undriven(&self) -> u8 {
        if self.model.floating_bus() { self.data_bus } else { 0xFF }
    }

    fn enabled(&self, bit: u8) -> bool {
        !self.model.has_control_ports() || self.memory_control & bit == 0
    }

    /// The byte at `a`, as the CPU reads it.
    pub fn peek(&self, a: u16) -> u8 {
        if a >= 0xC000 {
            if self.cart.ram_over_work_ram() {
                return self.cart.read_high(a);
            }
            return if self.enabled(WORK_RAM_OFF) { self.ram[a as usize & (self.ram.len() - 1)] } else { self.undriven() };
        }
        // The BIOS, the card and the expansion slot hold nothing until stage 6; only the cartridge drives the slot area.
        if self.enabled(CARTRIDGE_OFF) {
            if let Some(v) = self.cart.read(a) {
                return v;
            }
        }
        self.undriven()
    }

    fn store(&mut self, a: u16, v: u8) {
        if a >= 0xC000 {
            if self.enabled(WORK_RAM_OFF) {
                let n = self.ram.len();
                self.ram[a as usize & (n - 1)] = v;
            }
            self.cart.write(a, v);
        } else if self.enabled(CARTRIDGE_OFF) {
            self.cart.write(a, v);
        }
    }

    /// The PSG brought to the present, its output discarded until stage 5 mixes it.
    fn catch_up_psg(&mut self) {
        let now = self.clock / PSG_DIVIDER;
        while self.psg_ticks < now {
            let n = (now - self.psg_ticks).min(u32::MAX as u64) as u32;
            self.psg.advance(n, |_, _| {});
            self.psg_ticks += n as u64;
        }
    }

    /// The machine's end: every device at the bus's time.
    pub fn settle(&mut self) {
        self.catch_up_psg();
    }

    fn port_in(&mut self, port: u8) -> u8 {
        let odd = port & 1 != 0;
        match port >> 6 {
            0 => self.undriven(),
            1 if self.model == Model::Sg1000 => self.undriven(),
            // The V and H counters, and the VDP's data and status, read $FF until stage 3 [C].
            1 | 2 => 0xFF,
            _ if !self.enabled(IO_OFF) => self.undriven(),
            _ if odd => self.io.port_dd(self.model),
            _ => self.io.port_dc(self.model),
        }
    }

    fn port_out(&mut self, port: u8, v: u8) {
        match port >> 6 {
            0 if self.model.has_control_ports() => {
                if port & 1 == 0 {
                    self.memory_control = v;
                } else {
                    self.io.control = v;
                }
            }
            1 => {
                self.catch_up_psg();
                self.psg.write(v);
            }
            // The VDP's data and control ports are taken by stage 3; ports $C0-$FF take no write before stage 9's FM.
            _ => {}
        }
    }
}

/// Each call is a machine cycle of the Z80 CPU User Manual's length, its time added before its access.
impl beryl_z80::Bus for Bus {
    fn fetch(&mut self, address: u16, _refresh: u16) -> u8 {
        self.clock += 4;
        let v = self.peek(address);
        self.data_bus = v;
        v
    }

    fn read(&mut self, address: u16) -> u8 {
        self.clock += 3;
        let v = self.peek(address);
        self.data_bus = v;
        v
    }

    fn write(&mut self, address: u16, value: u8) {
        self.clock += 3;
        self.data_bus = value;
        self.store(address, value);
    }

    /// Only the port address's low byte is decoded [R `smstech` §3].
    fn input(&mut self, port: u16) -> u8 {
        self.clock += 4;
        let v = self.port_in(port as u8);
        self.data_bus = v;
        v
    }

    fn output(&mut self, port: u16, value: u8) {
        self.clock += 4;
        self.data_bus = value;
        self.port_out(port as u8, value);
    }

    fn idle(&mut self, t_states: u32) {
        self.clock += t_states as u64;
    }

    /// The VDP's interrupt arrives at stage 3.
    fn int_line(&mut self) -> bool {
        false
    }

    fn nmi_edge(&mut self) -> bool {
        std::mem::take(&mut self.nmi)
    }

    /// No device answers the acknowledge: $FF on the Master System II, the floating bus on the others [R `smstech` §5].
    fn acknowledge(&mut self) -> u8 {
        self.clock += 6;
        self.undriven()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, Mapper, SEGA_RAM, Why};
    use beryl_sn76489::Variant;
    use beryl_z80::Bus as _;

    fn bus(model: Model, mapper: Mapper, rom: Vec<u8>) -> Bus {
        let ram = if model == Model::Sg1000 { 0x400 } else { 0x2000 };
        let board = Board { mapper, ram: if mapper == Mapper::Sega { SEGA_RAM } else { 0 }, battery: false, why: Why::Size };
        Bus::new(model, Cartridge::new(board, rom), ram, Sn76489::new(Variant::SEGA), 32)
    }

    #[test]
    fn work_ram_mirrors_and_keeps_the_mapper_s_registers() {
        let mut b = bus(Model::Sms2, Mapper::Sega, vec![0; 0x40000]);
        b.write(0xC123, 0x5A);
        assert_eq!((b.read(0xE123), b.read(0xC123)), (0x5A, 0x5A));
        b.write(0xFFFF, 7);
        assert_eq!((b.read(0xDFFF), b.cart.slots[2]), (7, 7 * 0x4000), "the register is written and RAM keeps its copy");
        let mut sg = bus(Model::Sg1000, Mapper::Plain, vec![0; 0x8000]);
        sg.write(0xC3FF, 9);
        assert_eq!((sg.read(0xFFFF), sg.read(0xC7FF)), (9, 9), "the SG-1000's 1 KiB mirrors every kilobyte");
    }

    #[test]
    fn port_3e_disables_the_cartridge_and_the_ram() {
        let rom: Vec<u8> = (0..0x8000).map(|i| (i >> 8) as u8).collect();
        for (model, undriven) in [(Model::Sms1, true), (Model::Sms2, false)] {
            let mut b = bus(model, Mapper::Plain, rom.clone());
            assert_eq!(b.memory_control, 0xAB);
            b.write(0xC000, 0x42);
            b.output(0x3E, 0xAB | CARTRIDGE_OFF | WORK_RAM_OFF);
            b.data_bus = 0x3E;
            let want = if undriven { 0x3E } else { 0xFF };
            assert_eq!((b.peek(0x1234), b.peek(0xC000)), (want, want), "{model:?}");
            b.write(0xC000, 0x99);
            b.output(0x3E, 0xAB);
            assert_eq!((b.peek(0x1234), b.peek(0xC000)), (0x12, 0x42), "{model:?}: a write to disabled RAM is lost");
        }
        let mut sg = bus(Model::Sg1000, Mapper::Plain, rom);
        sg.output(0x3E, 0xFF);
        assert_eq!(sg.peek(0x1234), 0x12, "the SG-1000 has no port $3E");
    }

    #[test]
    fn ports_decode_by_a7_a6_and_a0() {
        let mut b = bus(Model::Sms2, Mapper::Plain, vec![0; 0x8000]);
        b.io.pads = [crate::io::UP, crate::io::LEFT];
        for p in (0xC0..=0xFFu16).step_by(2) {
            assert_eq!((b.input(p), b.input(p + 1)), (0xFE, 0xFE), "{p:02X}");
        }
        assert_eq!(b.input(0x00DC | 0xAB00), 0xFE, "the high byte is not decoded");
        b.output(0x01, 0x55);
        assert_eq!(b.io.control, 0x55, "any odd port below $40 is $3F");
        b.output(0x3C, 0xAB | IO_OFF);
        assert_eq!(b.input(0xDC), 0xFF, "I/O disabled reads $FF on the Master System II");
        assert_eq!(b.input(0x3E), 0xFF);
        let mut s1 = bus(Model::Sms1, Mapper::Plain, vec![0; 0x8000]);
        s1.read(0x0000);
        s1.data_bus = 0x3F;
        assert_eq!(s1.input(0x3F), 0x3F, "the first Master System floats the bus");
        let t = b.clock;
        b.output(0x7F, 0x9F);
        assert_eq!(b.clock - t, 4);
        b.output(0x40, 0x8F);
        assert_ne!(b.psg, Sn76489::new(Variant::SEGA), "the PSG is written through any port $40-$7F");
    }

    #[test]
    fn each_machine_cycle_takes_its_length() {
        let mut b = bus(Model::Sms2, Mapper::Plain, vec![0; 0x8000]);
        b.fetch(0, 0);
        b.read(0);
        b.write(0xC000, 0);
        b.input(0xDC);
        b.output(0x3F, 0xFF);
        b.idle(5);
        b.acknowledge();
        assert_eq!(b.clock, 4 + 3 + 3 + 4 + 4 + 5 + 6);
        let mut s1 = bus(Model::Sms1, Mapper::Plain, vec![0; 0x8000]);
        s1.write(0xC000, 0x5A);
        assert_eq!((s1.acknowledge(), b.acknowledge()), (0x5A, 0xFF), "a write leaves its byte on the bus the first Master System floats");
        b.clock = 1000;
        b.settle();
        assert_eq!(b.psg_ticks, 1000 / 16);
    }
}
