//! The debugger's view of the machine (Endou_Native.md §15): the Z80's observer, its bus as a space read without side
//! effects, its code listed by `beryl-z80`'s disassembler, and the parts that run no code as units: the VDP, the PSG,
//! the cartridge's board and the I/O chip's two ports.

use emusen_native::core::{Instruction, UNIT_FIRST_ID, Unit};
use emusen_native::debug::{Hooks, Observer, stop};

use crate::board::Mapper;
use crate::machine::{Machine, Z80BUS_ID};

/// The Z80's bus as a space: 64 KiB.
pub const Z80BUS_SIZE: usize = 1 << 16;

pub const VDP: u32 = UNIT_FIRST_ID;
pub const PSG: u32 = UNIT_FIRST_ID + 1;
pub const BOARD: u32 = UNIT_FIRST_ID + 2;
pub const IO: u32 = UNIT_FIRST_ID + 3;

/// The Z80's observer in the observed frame: the stop asked before each instruction but the one a resume runs unasked,
/// then its coverage and profile recorded; its stores on the bus and its calls into the hooks. The stores in the
/// memories they land in are the bus's to report (`Bus::landed`).
pub struct Watch<'a> {
    pub hooks: &'a mut Hooks,
    pub unchecked: bool,
}

impl Observer for Watch<'_> {
    fn before(&mut self, _processor: usize, pc: u32) -> u32 {
        if !std::mem::take(&mut self.unchecked) {
            let why = self.hooks.stop_before(pc);
            if why != stop::FRAME {
                return why;
            }
        }
        self.hooks.record(pc);
        stop::FRAME
    }

    fn wrote(&mut self, _space: u32, address: u32, value: u8, pc: u32) {
        if self.hooks.writes {
            self.hooks.note_write(Z80BUS_ID, address & 0xFFFF, value, pc);
        }
    }

    fn called(&mut self, source: u32, target: u32, kind: u32) {
        self.hooks.note_call(source, target, kind);
    }

    fn returned(&mut self) {
        self.hooks.note_return();
    }
}

/// The spaces whose stores are reported: the bus, and the two RAMs a store through it lands in.
pub fn reports_stores(space: u32) -> bool {
    matches!(space, Z80BUS_ID | crate::machine::RAM_ID | crate::machine::CARTRAM_ID)
}

/// The board's number as the `Board` unit's `Mapper` register gives it, which the `sms` pack names (Endou_Native.md §15.4).
pub fn mapper_number(m: Mapper) -> i64 {
    match m {
        Mapper::Plain => 0,
        Mapper::Sega => 1,
        Mapper::Codemasters => 2,
        Mapper::SgRam => 3,
    }
}

type Registers = Vec<(&'static str, u32, i64)>;

/// One unit's name, kind and registers with their values, read from the part that holds it; none for an id the machine
/// has no unit for. Machine info's list and `debug_registers` both read it, so the two cannot disagree.
fn unit(m: &Machine, id: u32) -> Option<(&'static str, &'static str, Registers)> {
    let b = &m.bus;
    Some(match id {
        VDP => {
            let v = &b.vdp;
            let mut r: Registers = (0..11).map(|i| (["R0", "R1", "R2", "R3", "R4", "R5", "R6", "R7", "R8", "R9", "R10"][i], 8, v.regs[i] as i64)).collect();
            let low = if v.variant.tms { v.fifth & 0x1F } else { 0 };
            let status = (v.frame_flag as u8) << 7 | (v.overflow as u8) << 6 | (v.collision as u8) << 5 | low;
            r.extend([
                ("Status", 8, status as i64),
                ("LineFlag", 1, v.line_flag as i64),
                ("Address", 14, v.address as i64),
                ("Code", 2, v.code as i64),
                ("Second", 1, v.second as i64),
                ("Buffer", 8, v.buffer as i64),
                ("LineCounter", 8, v.line_counter as i64),
                ("VCounter", 8, v.read_v() as i64),
                ("HLatch", 8, v.h_latch as i64),
                ("Line", 9, v.line as i64),
                ("Lines", 8, v.active_lines() as i64),
                ("Mode4", 1, v.mode4() as i64),
                ("IRQ", 1, v.irq() as i64),
                ("Variant", 3, v.variant.id as i64),
            ]);
            ("VDP", "video", r)
        }
        PSG => {
            let p = &b.sound.psg;
            ("PSG", "audio", vec![
                ("Tone0", 10, p.tone[0] as i64),
                ("Tone1", 10, p.tone[1] as i64),
                ("Tone2", 10, p.tone[2] as i64),
                ("Noise", 3, p.noise as i64),
                ("Volume0", 4, p.volume[0] as i64),
                ("Volume1", 4, p.volume[1] as i64),
                ("Volume2", 4, p.volume[2] as i64),
                ("Volume3", 4, p.volume[3] as i64),
                ("Latch", 3, p.latch as i64),
            ])
        }
        BOARD if !b.cart.rom.is_empty() => {
            let c = &b.cart;
            ("Board", "cartridge", vec![
                ("Mapper", 2, mapper_number(c.board.mapper)),
                ("Control", 8, c.regs[0] as i64),
                ("Bank0", 8, c.regs[1] as i64),
                ("Bank1", 8, c.regs[2] as i64),
                ("Bank2", 8, c.regs[3] as i64),
                ("Ram", 16, c.ram.len() as i64),
                ("Battery", 1, c.board.battery as i64),
            ])
        }
        IO if m.model().has_control_ports() => ("IO", "io", vec![("MemoryControl", 8, b.memory_control as i64), ("IoControl", 8, b.io.control as i64)]),
        _ => return None,
    })
}

/// Machine info's `units`, in id order: those the machine has.
pub fn units(m: &Machine) -> Vec<Unit> {
    (VDP..=IO)
        .filter_map(|id| unit(m, id).map(|(name, kind, r)| Unit { id, name: name.to_owned(), kind: kind.to_owned(), registers: r.into_iter().map(|(n, bits, _)| (n.to_owned(), bits)).collect() }))
        .collect()
}

/// A unit's values in its registers' order, or none for an id the machine has no unit for.
pub fn unit_values(m: &Machine, id: u32) -> Option<Vec<i64>> {
    unit(m, id).map(|(_, _, r)| r.into_iter().map(|(_, _, v)| v).collect())
}

impl Machine {
    /// A byte of a space as a debugger reads it: the bus as the Z80 would read it, with no side effect; past a space's end 0.
    pub fn debug_byte(&self, space: u32, address: u32) -> Option<u8> {
        if space == Z80BUS_ID {
            return Some(self.bus.peek(address as u16));
        }
        self.space(space).map(|m| m.get(address as usize).copied().unwrap_or(0))
    }

    /// `count` instructions from `address` of a space, at most 4,096, decoded by `beryl-z80`: on the bus at the Z80's
    /// addresses, in another space at its offsets; none for a space the machine has not.
    pub fn debug_disassemble(&self, space: u32, address: u32, count: u32) -> Option<Vec<Instruction>> {
        self.debug_byte(space, 0)?;
        let mut out = Vec::with_capacity(count.min(4096) as usize);
        let mut at = if space == Z80BUS_ID { address & 0xFFFF } else { address };
        for _ in 0..count.min(4096) {
            let base = at;
            // The decoder counts in the Z80's 16 bits; each byte is read at its distance from the instruction's start.
            let read = |a: u16| self.debug_byte(space, if space == Z80BUS_ID { a as u32 } else { base.wrapping_add(a.wrapping_sub(base as u16) as u32) }).unwrap_or(0);
            let mut i = beryl_z80::disasm::disassemble(&read, base as u16);
            i.address = base;
            at = base.wrapping_add(i.bytes.len().max(1) as u32);
            if space == Z80BUS_ID {
                at &= 0xFFFF;
            }
            out.push(i);
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::{CARTRAM_ID, RAM_ID, ROM_ID};
    use crate::media::{Market, Media};
    use crate::model::Model;
    use emusen_native::debug::{drain_coverage, drain_profile, flag, kind, run};
    use emusen_native::ffi::StateMachine;

    const RESUME: u32 = run::UNCHECKED | run::CONTINUE;

    /// The frame interrupt on, its handler at $38 reading the status and returning; a loop that calls a routine counting
    /// in $C000 through HL. The labels are the assembler's.
    const CALLS: &str = "
        di
        jp main
        org $38
irq:    push af
        in a,($BF)
        pop af
        ei
        ret
        org $66
nmi:    retn
main:   im 1
        ld sp,$DFF0
        ld a,$20
        out ($BF),a
        ld a,$81
        out ($BF),a
        ld a,$9F
        out ($7F),a
        ei
loop:   call count
        jr loop
count:  ld hl,$C000
        inc (hl)
        ret
";

    struct Program {
        machine: Machine,
        labels: std::collections::BTreeMap<String, i64>,
    }

    impl Program {
        fn at(&self, label: &str) -> u32 {
            self.labels[label] as u32
        }
    }

    /// The source at $0000 in a 32 KiB export cartridge with Sega's header, or of `banks` banks on Sega's mapper, run
    /// from reset on a Master System II with no BIOS.
    fn program(source: &str, banks: usize) -> Program {
        let code = beryl_z80::asm::assemble(source, 0).unwrap();
        let mut rom = crate::media::sample::cartridge(banks * 0x4000, 4);
        rom[..code.bytes.len()].copy_from_slice(&code.bytes);
        let media = Media::read(&rom).unwrap();
        Program { machine: Machine::new(media, Market::Americas, Model::Sms2, &rom), labels: code.symbols }
    }

    fn state(m: &Machine) -> Vec<u8> {
        let mut v = vec![0; m.state_size()];
        let n = m.save_state(&mut v).unwrap();
        v.truncate(n);
        v
    }

    fn sound(m: &mut Machine) -> Vec<i16> {
        let mut out = vec![0; 8192];
        let n = m.audio.drain(&mut out, usize::MAX);
        out.truncate(n);
        out
    }

    fn run_out(m: &mut Machine) -> u32 {
        let mut stops = 0;
        while m.run_frame_debug(RESUME) != 0 {
            stops += 1;
        }
        stops
    }

    // The kit's C15 on the core: every table armed with nothing to hit gives the plain frame's state, picture and sound.
    #[test]
    fn an_armed_frame_with_nothing_to_hit_is_the_plain_frame() {
        let (mut a, mut b) = (program(CALLS, 2).machine, program(CALLS, 2).machine);
        b.hooks.configure(flag::CALLS | flag::WRITES | flag::PROFILING | 1 << flag::COVERAGE, i32::MIN, -1);
        b.hooks.set_breakpoints(&[-2, -2]);
        b.hooks.set_ranges(0, &[RAM_ID, 0, 0x1FFF, Z80BUS_ID, 0, 0xFFFF]);
        b.hooks.set_ranges(1, &[RAM_ID, 0x1FF0, 0x1FF0]);
        for _ in 0..4 {
            a.advance();
            assert_eq!(b.run_frame_debug(0), 0);
            let (sa, sb) = (sound(&mut a), sound(&mut b));
            assert_eq!((state(&b), b.frame_bytes(), sb), (state(&a), a.frame_bytes(), sa));
            b.hooks.writes_log.clear();
            b.hooks.calls_log.clear();
        }
        let mut covered = 0;
        assert!(drain_coverage(&mut b.hooks, 0, &mut vec![0; 1 << 13], &mut covered) > 0 && covered > 0);
    }

    // A breakpoint stops in front of its instruction with the frame left open, and an unchecked resume runs past it once.
    #[test]
    fn a_breakpoint_stops_in_front_of_its_instruction() {
        let p = program(CALLS, 2);
        let (count, mut m) = (p.at("count"), p.machine);
        m.hooks.set_breakpoints(&[count as i32, count as i32]);
        assert_eq!(m.run_frame_debug(0), stop::BREAKPOINT);
        assert_eq!((m.cpu.regs.pc as u32, m.frames, m.debug_open), (count, 0, true));
        let before = m.bus.ram[0];
        assert_eq!(m.run_frame_debug(RESUME), stop::BREAKPOINT);
        assert_eq!((m.cpu.regs.pc as u32, m.bus.ram[0]), (count, before.wrapping_add(1)), "one pass of the routine between the stops");
        assert_eq!(m.frames, 0, "the frame stays open");
    }

    // Stops through a frame, each resumed, leave it as a plain one makes it: the devices brought to time at each stop change nothing.
    #[test]
    fn a_frame_stopped_and_resumed_is_the_plain_frame() {
        let p = program(CALLS, 2);
        let (count, irq) = (p.at("count"), p.at("irq"));
        let (mut a, mut b) = (program(CALLS, 2).machine, p.machine);
        b.hooks.set_breakpoints(&[count as i32, count as i32, irq as i32, irq as i32]);
        for _ in 0..3 {
            a.advance();
            assert!(run_out(&mut b) > 100);
            let (sa, sb) = (sound(&mut a), sound(&mut b));
            assert_eq!((state(&b), b.frame_bytes(), sb), (state(&a), a.frame_bytes(), sa));
        }
    }

    // A stop brings the VDP and the PSG to the bus's time, so a unit read there is the present's: a seeded fault that left them behind passed every test (Endou_Native.md §15.8).
    #[test]
    fn a_stop_brings_every_device_to_its_time() {
        let p = program(CALLS, 2);
        let (count, mut m) = (p.at("count"), p.machine);
        m.hooks.set_breakpoints(&[count as i32, count as i32]);
        for _ in 0..300 {
            assert_eq!(m.run_frame_debug(RESUME), stop::BREAKPOINT);
        }
        let (vdp, psg) = (unit_values(&m, VDP).unwrap(), m.bus.sound.ticks);
        m.bus.settle();
        assert_eq!((unit_values(&m, VDP).unwrap(), m.bus.sound.ticks), (vdp, psg));
        assert!((1..200).contains(&m.bus.vdp.line), "mid-frame: line {}", m.bus.vdp.line);
    }

    // EACH stops in front of every instruction, and each unchecked resume moves the machine by one.
    #[test]
    fn each_stops_in_front_of_every_instruction() {
        let p = program(CALLS, 2);
        let (looping, count, mut m) = (p.at("loop"), p.at("count"), p.machine);
        m.hooks.configure(flag::EACH, i32::MIN, -1);
        assert_eq!(m.run_frame_debug(0), stop::EACH);
        let mut seen = Vec::new();
        for _ in 0..16 {
            seen.push(m.cpu.regs.pc as u32);
            assert_eq!(m.run_frame_debug(RESUME), stop::EACH);
        }
        let start = seen.iter().position(|&pc| pc == looping).unwrap();
        assert_eq!(&seen[start..start + 5], [looping, count, count + 3, count + 4, looping + 3], "CALL, LD HL, INC (HL), RET, JR");
    }

    // A CALL pushes and RET pops; a depth target steps out of the routine; the frame interrupt stops in front of its handler with an IRQ in the log, and Pause's NMI with an NMI.
    #[test]
    fn the_call_stack_follows_calls_interrupts_and_returns() {
        let p = program(CALLS, 2);
        let (looping, count, irq, nmi, mut m) = (p.at("loop"), p.at("count"), p.at("irq"), p.at("nmi"), p.machine);
        m.hooks.configure(flag::CALLS, i32::MIN, -1);
        m.hooks.set_breakpoints(&[count as i32 + 3, count as i32 + 3]);
        assert_eq!(m.run_frame_debug(0), stop::BREAKPOINT);
        assert_eq!(m.hooks.stack, [(looping, count)]);
        m.hooks.set_breakpoints(&[]);
        m.hooks.configure(flag::CALLS, 0, -1);
        assert_eq!(m.run_frame_debug(RESUME), stop::DEPTH);
        assert_eq!((m.cpu.regs.pc as u32, m.hooks.stack.len()), (looping + 3, 0), "out of the routine at the JR");
        m.hooks.configure(flag::CALLS | flag::INTERRUPTS, i32::MIN, -1);
        assert_eq!(m.run_frame_debug(RESUME), stop::INTERRUPT);
        assert_eq!(m.cpu.regs.pc as u32, irq);
        let top = *m.hooks.calls_log.last().unwrap();
        assert_eq!((top.kind, top.target), (kind::IRQ, irq));
        while m.run_frame_debug(RESUME) != 0 {}
        m.pads[0] = crate::io::PAUSE as u32;
        assert_eq!(m.run_frame_debug(0), stop::INTERRUPT);
        let top = *m.hooks.calls_log.last().unwrap();
        assert_eq!((top.kind, top.target, m.cpu.regs.pc as u32), (kind::NMI, nmi, nmi));
    }

    // A watch logs the routine's store on the bus and in work RAM with the storing instruction's address; a data breakpoint stops in front of the next instruction.
    #[test]
    fn watches_and_data_breakpoints_see_stores_on_the_bus_and_in_ram() {
        let p = program(CALLS, 2);
        let (count, mut m) = (p.at("count"), p.machine);
        m.hooks.configure(flag::WRITES, i32::MIN, -1);
        m.hooks.set_ranges(0, &[RAM_ID, 0, 0, Z80BUS_ID, 0xC000, 0xC000]);
        assert_eq!(m.run_frame_debug(0), 0);
        let log = std::mem::take(&mut m.hooks.writes_log);
        let ram: Vec<_> = log.iter().filter(|w| w.space == RAM_ID).collect();
        assert!(!ram.is_empty() && ram.iter().all(|w| (w.address, w.pc) == (0, count + 3)));
        assert_eq!(ram.last().unwrap().value, m.bus.ram[0] as u32);
        assert_eq!(log.iter().filter(|w| w.space == Z80BUS_ID && w.address == 0xC000).count(), ram.len(), "the same stores on the bus");
        assert!(log.iter().all(|w| w.space != CARTRAM_ID));
        m.hooks.set_ranges(0, &[]);
        m.hooks.set_ranges(1, &[RAM_ID, 0, 0]);
        assert_eq!(m.run_frame_debug(0), stop::DATA);
        assert_eq!(m.cpu.regs.pc as u32, count + 4, "in front of the RET after the store");
        assert!(!m.bus.watch && m.bus.landed.is_empty(), "the bus reports nothing between debugged frames");
    }

    // Sega's mapper with its RAM in slot 2: a store at $8000 is reported in CARTRAM at its offset, and not in work RAM.
    #[test]
    fn a_store_in_the_cartridge_s_ram_is_reported_there() {
        let p = program("    ld a,$08\n    ld ($FFFC),a\n    ld a,$5A\n    ld ($8001),a\n    halt\n", 4);
        let mut m = p.machine;
        assert!(m.bus.cart.ram.len() > 0);
        m.hooks.configure(flag::WRITES, i32::MIN, -1);
        m.hooks.set_ranges(0, &[CARTRAM_ID, 0, 0x7FFF, RAM_ID, 0, 0x1FFF]);
        assert_eq!(m.run_frame_debug(0), 0);
        let log: Vec<_> = m.hooks.writes_log.iter().map(|w| (w.space, w.address, w.value, w.pc)).collect();
        assert_eq!(log, [(RAM_ID, 0x1FFC, 0x08, 2), (CARTRAM_ID, 1, 0x5A, 7)], "$FFFC lands in work RAM's copy, $8001 in the cartridge's RAM");
    }

    // Coverage marks each instruction run and nothing else; the profile charges the routine's three instructions to it.
    #[test]
    fn coverage_and_the_profile_charge_what_ran() {
        let p = program(CALLS, 2);
        let (looping, count, mut m) = (p.at("loop"), p.at("count"), p.machine);
        m.hooks.configure(flag::CALLS | flag::PROFILING | 1 << flag::COVERAGE, i32::MIN, -1);
        assert_eq!(m.run_frame_debug(0), 0);
        let mut bits = vec![0u8; 1 << 13];
        let mut n = 0;
        drain_coverage(&mut m.hooks, 0, &mut bits, &mut n);
        let hit = |at: u32| bits[(at >> 3) as usize] >> (at & 7) & 1 != 0;
        assert!([0, 1, looping, looping + 3, count, count + 3, count + 4].iter().all(|&at| hit(at)));
        assert!(!hit(count + 1) && !hit(looping + 1) && !hit(0x66), "operand bytes and the NMI's handler never ran");
        let mut pairs = vec![0i64; 64];
        let k = drain_profile(&mut m.hooks, &mut pairs);
        let routine = pairs[..2 * k].chunks(2).find(|p| p[0] == count as i64).map(|p| p[1]).unwrap();
        let calls = m.hooks.calls_log.iter().filter(|c| c.target == count && c.kind == kind::CALL).count() as i64;
        assert!(calls > 0 && (3 * calls - 3..=3 * calls).contains(&routine), "three instructions a call, the frame's last perhaps unfinished: {routine} for {calls}");
    }

    // The registers are beryl-z80's list; the units are the VDP, the PSG, the board and the I/O chip, each answered with its count, and the id past them refused.
    #[test]
    fn the_registers_and_units_answer_as_machine_info_names_them() {
        use emusen_native::core::Core;
        let mut m = program(CALLS, 2).machine;
        m.advance();
        let info = m.machine_info();
        assert_eq!(info.processors[0].registers.len(), 18);
        assert_eq!((info.processors[0].registers[11].0.as_str(), info.processors[0].code_space), ("PC", Some(Z80BUS_ID)));
        let r = m.debug_registers(0).unwrap();
        assert_eq!((r[10], r[11], r[15]), (m.cpu.regs.sp as i64, m.cpu.regs.pc as i64, 1), "SP, PC and IM 1");
        let units = m.units();
        assert_eq!(units.iter().map(|u| (u.id, u.name.as_str(), u.kind.as_str())).collect::<Vec<_>>(), [(256, "VDP", "video"), (257, "PSG", "audio"), (258, "Board", "cartridge"), (259, "IO", "io")]);
        for u in &units {
            assert_eq!(m.debug_registers(u.id).unwrap().len(), u.registers.len(), "{}", u.name);
        }
        assert_eq!(m.debug_registers(260), Err(emusen_native::core::status::NOT_SUPPORTED));
        let value = |id: u32, name: &str| m.debug_registers(id).unwrap()[units.iter().find(|u| u.id == id).unwrap().registers.iter().position(|r| r.0 == name).unwrap()];
        assert_eq!((value(VDP, "R1"), value(VDP, "Variant"), value(VDP, "Lines")), (0x20, 2, 192));
        assert_eq!((value(PSG, "Volume0"), value(PSG, "Latch")), (0xF, 1), "the latch byte $9F: tone 0's attenuation, silent");
        assert_eq!((value(BOARD, "Mapper"), value(IO, "MemoryControl")), (0, 0xAB));
        let request = emusen_native::core::Create { image: &[], settings: Default::default(), files: Vec::new(), pixel_formats: 1, host_abi_version: emusen_native::core::sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None };
        let empty = Machine::create_empty("sms", &request).unwrap();
        assert_eq!(empty.units().iter().map(|u| u.name.as_str()).collect::<Vec<_>>(), ["VDP", "PSG", "IO"], "an empty slot has no board, a seeded fault's listing of one passed every test (Endou_Native.md §15.8)");
        assert_eq!(empty.debug_registers(BOARD), Err(emusen_native::core::status::NOT_SUPPORTED));
        let sg = Machine::new(Media::read(&[0u8; 0x2000]).unwrap(), Market::Japan, Model::Sg1000, &[0u8; 0x2000]);
        assert_eq!(sg.units().iter().map(|u| u.name.as_str()).collect::<Vec<_>>(), ["VDP", "PSG", "Board"], "the SG-1000 has no ports $3E and $3F");
        assert_eq!(sg.debug_registers(VDP).unwrap()[sg.units()[0].registers.iter().position(|r| r.0 == "Variant").unwrap()], 4);
    }

    // The bus reads as the Z80 sees it, mirrors and all, with no side effect, and takes no write.
    #[test]
    fn the_bus_reads_as_the_z80_sees_it_and_changes_nothing() {
        use emusen_native::core::Core;
        let mut m = program(CALLS, 2).machine;
        m.advance();
        m.bus.ram[0x10] = 0xAB;
        let before = state(&m);
        let mut out = [0u8; 4];
        m.space_read(Z80BUS_ID, 0xC010, &mut out[..1]).unwrap();
        m.space_read(Z80BUS_ID, 0xE010, &mut out[1..2]).unwrap();
        m.space_read(Z80BUS_ID, 0xFFFF, &mut out[2..4]).unwrap();
        assert_eq!(out, [0xAB, 0xAB, m.bus.ram[0x1FFF], m.bus.cart.rom[0]], "work RAM, its mirror, and the wrap to $0000");
        assert_eq!(state(&m), before);
        assert_eq!((m.space_size(Z80BUS_ID), m.space_write(Z80BUS_ID, 0xC000, &[1])), (Ok(0x10000), Err(emusen_native::core::status::READ_ONLY)));
        let info = m.machine_info();
        let bus = &info.spaces[0];
        assert_eq!((bus.id, bus.name.as_str(), bus.read_only, bus.view, bus.reports_stores), (Z80BUS_ID, "Z80BUS", true, true, true));
        assert_eq!(info.spaces.iter().filter(|s| s.reports_stores).map(|s| s.name.as_str()).collect::<Vec<_>>(), ["Z80BUS", "RAM"]);
    }

    // The code reads in the Z80's instruction set on the bus, and in the ROM at its offsets past 64 KiB.
    #[test]
    fn the_code_lists_on_the_bus_and_in_a_space_at_its_offsets() {
        use emusen_native::core::Core;
        let p = program(CALLS, 16);
        let (main, mut m) = (p.at("main"), p.machine);
        let text = |list: Vec<Instruction>| list.iter().map(|i| (i.address, format!("{} {}", i.mnemonic, i.operands).trim().to_owned())).collect::<Vec<_>>();
        assert_eq!(text(Core::debug_disassemble(&mut m, 0, Z80BUS_ID, 0, 2).unwrap()), [(0, "DI".to_owned()), (1, format!("JP ${main:04X}"))]);
        m.bus.cart.rom[0x1_FFFE..0x2_0001].copy_from_slice(&[0x3E, 0x05, 0xC9]);
        assert_eq!(text(Core::debug_disassemble(&mut m, 0, ROM_ID, 0x1_FFFE, 2).unwrap()), [(0x1_FFFE, "LD A,$05".to_owned()), (0x2_0000, "RET".to_owned())]);
        assert_eq!(Core::debug_disassemble(&mut m, 0, Z80BUS_ID, 0xFFFF, 2).unwrap()[1].address, 0, "the bus wraps");
        assert_eq!(Core::debug_disassemble(&mut m, 1, Z80BUS_ID, 0, 1).err(), Some(emusen_native::core::status::NOT_SUPPORTED));
        assert_eq!(Core::debug_disassemble(&mut m, 0, 99, 0, 1).err(), Some(emusen_native::core::status::NO_SUCH_SPACE));
        assert_eq!(Core::debug_disassemble(&mut m, 0, Z80BUS_ID, 0, 10_000).unwrap().len(), 4096);
    }
}
