//! The Z80's bus (Endou_Native.md §9.3, §9.6): the memory map with port $3E's enables, the BIOS's slot beside the
//! cartridge's (Endou_OpenBoot.md §4), the I/O ports decoded by A7, A6 and A0, and the clock. The bus owns the time:
//! every machine cycle the Z80 reports advances it, and the devices are brought to it when an access reaches them. The
//! VDP is Beryl's `beryl-sms-vdp`: the 315-5124 or 315-5246 on the Master System's consoles, the TMS9918A on the SG-1000
//! (Endou_Native.md §10, §11); the PSG and the FM unit's YM2413 at ports $F0-$F2 are `sound`'s (§12, §16).

use beryl_sms_vdp::Vdp;

use crate::cart::Cartridge;
use crate::io::Io;
use crate::machine::{CARTRAM_ID, RAM_ID};
use crate::model::Model;
use crate::sound::Sound;

/// The VDP's clock, the master clock over five, against the Z80's over fifteen (Endou_Plan.md §5.1).
pub const VDP_PER_Z80: u64 = 3;

/// Port $3E's bits, each enabling its part when clear [R MacDonald's `smstech` §4; Sega's Mark III manual, "Memory
/// enables"].
pub const CARTRIDGE_OFF: u8 = 1 << 6;
pub const WORK_RAM_OFF: u8 = 1 << 4;
pub const BIOS_OFF: u8 = 1 << 3;
pub const IO_OFF: u8 = 1 << 2;

/// The FM unit's audio control port (SMS Power!'s "Audio Control Port").
pub const AUDIO_CONTROL: u8 = 0xF2;

/// The highest address a Game Genie patches: the cartridge's slot area (SMS Power!'s "Game Genie").
pub const PATCH_HIGH: u16 = 0xBFFF;

/// A Game Genie's patch: the byte the cartridge gives at `address` replaced by `value`, where it equals `compare` if one
/// is given. The host sends them; they are not the machine's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Patch {
    pub address: u16,
    pub value: u8,
    pub compare: Option<u8>,
}

#[derive(Clone, Debug)]
pub struct Bus {
    pub model: Model,
    /// Z80 clocks since power-on; the master clock is fifteen times it.
    pub clock: u64,
    pub ram: Vec<u8>,
    pub cart: Cartridge,
    pub io: Io,
    pub sound: Sound,
    /// Port $3E as last written.
    pub memory_control: u8,
    /// The last byte on the data bus, which an access nothing drives reads on the consoles that float it.
    pub data_bus: u8,
    /// A falling edge on NMI not yet taken.
    pub nmi: bool,
    pub vdp: Vdp,
    /// The BIOS's ROM, empty on a console without one or a machine started with none (`boot::Boot`).
    pub bios: Vec<u8>,
    /// The cheat device's patches on the cartridge's reads, empty with none.
    pub patches: Vec<Patch>,
    /// While a debugger watches stores: each byte a store left in work RAM or the cartridge's RAM, as (space, offset,
    /// value), for the machine to report with the instruction's address (Endou_Native.md §15.3).
    pub watch: bool,
    pub landed: Vec<(u32, u32, u8)>,
}

impl Bus {
    /// A bus with no BIOS: port $3E as a BIOS leaves it for a cartridge.
    pub fn new(model: Model, cart: Cartridge, ram: usize, sound: Sound, vdp: Vdp) -> Bus {
        Bus::with_bios(model, cart, ram, sound, vdp, Vec::new())
    }

    /// A bus whose BIOS slot holds `bios`, port $3E at its reset value with the BIOS alone in the slot area
    /// (Endou_OpenBoot.md §2, U1); with none, as `new`.
    pub fn with_bios(model: Model, cart: Cartridge, ram: usize, sound: Sound, vdp: Vdp, bios: Vec<u8>) -> Bus {
        assert!(bios.is_empty() || bios.len().is_power_of_two(), "a BIOS mirrors by its size, a power of two");
        let memory_control = if bios.is_empty() { model.memory_control_at_start() } else { crate::boot::RESET_MEMORY_CONTROL };
        Bus { model, clock: 0, ram: vec![0; ram], cart, io: Io::default(), sound, memory_control, data_bus: 0xFF, nmi: false, vdp, bios, patches: Vec::new(), watch: false, landed: Vec::with_capacity(4) }
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
        // The card and the expansion slots hold nothing; the BIOS's 8 KiB mirror through the slot area [A]. With both on,
        // the cartridge wins on the first Master System and the two are ANDed on the second [R `smstech` §4].
        let cart = if self.enabled(CARTRIDGE_OFF) { self.cart.read(a).map(|v| self.patched(a, v)) } else { None };
        if self.bios.is_empty() || !self.enabled(BIOS_OFF) {
            return cart.unwrap_or_else(|| self.undriven());
        }
        let b = self.bios[a as usize & (self.bios.len() - 1)];
        match cart {
            Some(c) if self.model.bios_and_cartridge_are_anded() => b & c,
            Some(c) => c,
            None => b,
        }
    }

    /// The cartridge's byte as a Game Genie between it and the console gives it: the first patch of the address whose
    /// reference matches, or that has none.
    fn patched(&self, a: u16, v: u8) -> u8 {
        if self.patches.is_empty() {
            return v;
        }
        self.patches.iter().find(|p| p.address == a && p.compare.is_none_or(|c| c == v)).map_or(v, |p| p.value)
    }

    fn store(&mut self, a: u16, v: u8) {
        let mut cart = None;
        if a >= 0xC000 {
            if self.enabled(WORK_RAM_OFF) {
                let at = a as usize & (self.ram.len() - 1);
                self.ram[at] = v;
                if self.watch {
                    self.landed.push((RAM_ID, at as u32, v));
                }
            }
            cart = self.cart.write(a, v);
        } else if self.enabled(CARTRIDGE_OFF) {
            cart = self.cart.write(a, v);
        }
        if let Some(at) = cart.filter(|_| self.watch) {
            self.landed.push((CARTRAM_ID, at as u32, v));
        }
    }

    /// The VDP brought to the present.
    fn catch_up_vdp(&mut self) {
        self.vdp.run(self.clock * VDP_PER_Z80);
    }

    /// The machine's end: every device at the bus's time.
    pub fn settle(&mut self) {
        self.sound.run(self.clock);
        self.catch_up_vdp();
    }

    fn port_in(&mut self, port: u8) -> u8 {
        let odd = port & 1 != 0;
        match port >> 6 {
            0 => self.undriven(),
            1 if !self.model.has_counters() => self.undriven(),
            1 => {
                self.catch_up_vdp();
                if odd { self.vdp.read_h() } else { self.vdp.read_v() }
            }
            2 if odd => {
                self.catch_up_vdp();
                // The Sega VDPs' status leaves its low five bits undriven [R MacDonald's `msvdp` §4] (Endou_Native.md §10.4).
                let (undriven, driven) = (self.undriven(), self.vdp.status_bits());
                (self.vdp.read_control() & driven) | (undriven & !driven)
            }
            2 => {
                self.catch_up_vdp();
                self.vdp.read_data()
            }
            _ if !self.enabled(IO_OFF) && port == AUDIO_CONTROL && self.model.fm().is_some() => self.audio_control(),
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
                    let before = self.io.th_levels(self.model);
                    self.io.control = v;
                    let after = self.io.th_levels(self.model);
                    // TH rising on either port latches the H counter [R MacDonald's `msvdp` §11].
                    if (0..2).any(|p| !before[p] && after[p]) {
                        self.catch_up_vdp();
                        self.vdp.latch_h();
                    }
                }
            }
            1 => self.sound.write(v, self.clock),
            2 => {
                self.catch_up_vdp();
                if port & 1 != 0 { self.vdp.write_control(v) } else { self.vdp.write_data(v) }
            }
            // Ports $C0-$FF take no write but the FM unit's three; with no unit, $F0-$F2 are nothing (Endou_Native.md
            // §12.4, §16.3, D-13, D-17).
            _ if self.model.fm().is_none() => {}
            _ => match port {
                0xF0 | 0xF1 => self.sound.write_fm(port == 0xF1, v, self.clock),
                AUDIO_CONTROL => self.sound.write_control(v, self.clock),
                _ => {}
            },
        }
    }

    /// Port $F2 read with the I/O chip off on a console with an FM unit: bits 7-5 bits 11, 7 and 3 of a counter of the
    /// lines' sync pulses since power-on, taken as one a line; bits 4-2 clear; bits 1-0 as written [R SMS Power!'s
    /// "Audio Control Port"] (Endou_Disputes.md D-17).
    fn audio_control(&self) -> u8 {
        let lines = self.clock / crate::machine::LINE;
        let counter = ((lines >> 11 & 1) << 7 | (lines >> 7 & 1) << 6 | (lines >> 3 & 1) << 5) as u8;
        counter | self.sound.fm.as_ref().map_or(0, |f| f.control)
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

    /// The VDP's INT, as it stands at the instruction's end.
    fn int_line(&mut self) -> bool {
        self.catch_up_vdp();
        self.vdp.irq()
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
    use beryl_sn76489::{Sn76489, Variant};
    use beryl_z80::Bus as _;

    fn bus(model: Model, mapper: Mapper, rom: Vec<u8>) -> Bus {
        let ram = if model == Model::Sg1000 { 0x400 } else { 0x2000 };
        let board = Board { mapper, ram: if mapper == Mapper::Sega { SEGA_RAM } else { 0 }, battery: false, why: Why::Size };
        Bus::new(model, Cartridge::new(board, rom), ram, Sound::new(Sn76489::new(Variant::SEGA), 53_693_175, 15), Vdp::new(beryl_sms_vdp::Variant::SMS2, false))
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
        assert_ne!(b.sound.psg, Sn76489::new(Variant::SEGA), "the PSG is written through any port $40-$7F");
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
        assert_eq!(b.sound.ticks, 1000 / 16);
    }
}
