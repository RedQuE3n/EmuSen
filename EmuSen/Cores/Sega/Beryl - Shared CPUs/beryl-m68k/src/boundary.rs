//! What happens between instructions, which the single-step suites do not reach: interrupts, the reset exception and
//! trace, each on a bus that records its cycles and clocks (Beryl_M68k.md §5).

use std::collections::BTreeMap;

use crate::{Access, Bus, M68000, Size, Step};

#[derive(Default)]
struct Rig {
    ram: BTreeMap<u32, u8>,
    level: u8,
    vector: Option<u8>,
    log: Vec<String>,
    clocks: u32,
}

impl Rig {
    fn word(&self, a: u32) -> u16 {
        (self.ram.get(&a).copied().unwrap_or(0) as u16) << 8 | self.ram.get(&(a + 1)).copied().unwrap_or(0) as u16
    }

    fn put(&mut self, a: u32, v: u16) {
        self.ram.insert(a, (v >> 8) as u8);
        self.ram.insert(a + 1, v as u8);
    }

    fn long(&mut self, a: u32, v: u32) {
        self.put(a, (v >> 16) as u16);
        self.put(a + 2, v as u16);
    }
}

impl Bus for Rig {
    fn read(&mut self, a: Access) -> u16 {
        self.clocks += 4;
        self.log.push(format!("r{} {:X}", a.function, a.address));
        match a.size {
            Size::Word => self.word(a.address),
            Size::Byte => self.ram.get(&a.address).copied().unwrap_or(0) as u16,
        }
    }

    fn write(&mut self, a: Access, v: u16) {
        self.clocks += 4;
        self.log.push(format!("w {:X}={v:X}", a.address));
        self.put(a.address, v);
    }

    fn idle(&mut self, clocks: u32) {
        self.clocks += clocks;
        self.log.push(format!("n{clocks}"));
    }

    fn interrupt_level(&mut self) -> u8 {
        self.level
    }

    fn acknowledge(&mut self, level: u8) -> Option<u8> {
        self.clocks += 4;
        self.log.push(format!("iack{level}"));
        self.vector
    }
}

/// A processor at 0x1000 in user mode with mask `mask`, a NOP before it and another after, SSP 0x8000.
fn at_nop(mask: u16) -> (M68000, Rig) {
    let mut rig = Rig::default();
    rig.put(0x1000, 0x4E71);
    rig.put(0x1002, 0x4E71);
    rig.put(0x1004, 0x4E71);
    let mut cpu = M68000::new();
    cpu.regs.sr = mask << 8 | 0x0015;
    cpu.regs.a[7] = 0x3000;
    cpu.regs.other_sp = 0x8000;
    cpu.regs.pc = 0x1000;
    cpu.regs.prefetch = [0x4E71, 0x4E71];
    (cpu, rig)
}

#[test]
fn an_autovectored_interrupt_stacks_and_vectors_in_44_clocks() {
    let (mut cpu, mut rig) = at_nop(3);
    rig.long(0x70, 0x2000);
    rig.put(0x2000, 0x4E71);
    rig.level = 4;
    assert_eq!(cpu.step(&mut rig), Step::Exception(28));
    assert_eq!(
        rig.log,
        ["n6", "w 7FFE=1000", "iack4", "n4", "w 7FFA=315", "w 7FFC=0", "r5 70", "r5 72", "r6 2000", "n2", "r6 2002"]
    );
    assert_eq!(rig.clocks, 44);
    assert_eq!((cpu.regs.sr, cpu.regs.a[7], cpu.regs.other_sp, cpu.regs.pc), (0x2415, 0x7FFA, 0x3000, 0x2000));
}

#[test]
fn a_masked_level_waits_and_a_vectored_one_takes_its_device_vector() {
    let (mut cpu, mut rig) = at_nop(4);
    rig.level = 4;
    assert_eq!(cpu.step(&mut rig), Step::Instruction);
    rig.level = 5;
    rig.vector = Some(0x40);
    rig.long(0x100, 0x2400);
    assert_eq!(cpu.step(&mut rig), Step::Exception(0x40));
    assert_eq!(cpu.regs.pc, 0x2400);
    assert_eq!(rig.word(0x7FFE), 0x1002, "the PC stacked is the next instruction's");
}

#[test]
fn level_seven_is_taken_on_its_rise_and_by_comparison_once_the_mask_is_lowered() {
    let (mut cpu, mut rig) = at_nop(7);
    rig.long(0x7C, 0x2000);
    rig.put(0x2000, 0x4E71);
    rig.put(0x2002, 0x46FC);
    rig.put(0x2004, 0x2600);
    rig.put(0x2006, 0x4E71);
    rig.level = 7;
    assert_eq!(cpu.step(&mut rig), Step::Exception(31));
    assert_eq!(cpu.step(&mut rig), Step::Instruction, "a held level 7 does not interrupt again at mask 7");
    assert_eq!(cpu.step(&mut rig), Step::Instruction, "MOVE #$2600,SR lowers the mask");
    assert_eq!(cpu.step(&mut rig), Step::Exception(31), "the held level 7 now wins the comparison");
    rig.level = 0;
    let (mut cpu, mut rig2) = at_nop(7);
    rig2.long(0x7C, 0x2000);
    assert_eq!(cpu.step(&mut rig2), Step::Instruction);
    rig2.level = 7;
    assert_eq!(cpu.step(&mut rig2), Step::Exception(31), "a rise to 7 at mask 7");
}

#[test]
fn an_interrupt_ends_stop_and_stacks_the_instruction_after_it() {
    let mut rig = Rig::default();
    rig.put(0x1000, 0x4E72);
    rig.put(0x1002, 0x2000);
    rig.long(0x74, 0x2000);
    rig.put(0x2000, 0x4E71);
    let mut cpu = M68000::new();
    cpu.regs.sr = 0x2700;
    cpu.regs.a[7] = 0x8000;
    cpu.regs.pc = 0x1000;
    cpu.regs.prefetch = [0x4E72, 0x2000];
    assert_eq!(cpu.step(&mut rig), Step::Stopped);
    assert_eq!(cpu.regs.sr, 0x2000);
    assert_eq!(cpu.step(&mut rig), Step::Stopped);
    rig.level = 5;
    assert_eq!(cpu.step(&mut rig), Step::Exception(29));
    assert_eq!(rig.word(0x7FFE), 0x1004);
    assert!(!cpu.stopped);
}

#[test]
fn reset_loads_the_stack_pointer_and_pc_in_40_clocks() {
    let mut rig = Rig::default();
    rig.long(0, 0x00FF_FE00);
    rig.long(4, 0x0000_0200);
    rig.put(0x200, 0x4E71);
    rig.put(0x202, 0x4E75);
    let mut cpu = M68000::new();
    cpu.regs.sr = 0x8015;
    cpu.halted = true;
    cpu.reset(&mut rig);
    assert_eq!(rig.log, ["n14", "r6 0", "r6 2", "r6 4", "r6 6", "r6 200", "n2", "r6 202"]);
    assert_eq!(rig.clocks, 40);
    assert_eq!((cpu.regs.sr, cpu.regs.a[7], cpu.regs.pc, cpu.regs.prefetch, cpu.halted), (0x2715, 0x00FF_FE00, 0x200, [0x4E71, 0x4E75], false));
}

#[test]
fn trace_follows_the_traced_instruction_with_vector_9() {
    let (mut cpu, mut rig) = at_nop(0);
    cpu.regs.sr |= 0x8000;
    rig.long(0x24, 0x2000);
    rig.put(0x2000, 0x4E71);
    assert_eq!(cpu.step(&mut rig), Step::Instruction);
    assert_eq!(cpu.trace_pending, Some(0x1002));
    rig.log.clear();
    rig.clocks = 0;
    assert_eq!(cpu.step(&mut rig), Step::Exception(9));
    assert_eq!(rig.log, ["n4", "w 7FFE=1002", "w 7FFA=8015", "w 7FFC=0", "r5 24", "r5 26", "r6 2000", "n2", "r6 2002"]);
    assert_eq!(rig.clocks, 34);
    assert_eq!(cpu.regs.sr & 0xA000, 0x2000, "supervisor, trace off");
    assert_eq!(cpu.step(&mut rig), Step::Instruction, "the handler is not traced");
}

#[test]
fn trace_does_not_follow_an_illegal_instruction_and_does_follow_a_trap() {
    let (mut cpu, mut rig) = at_nop(0);
    cpu.regs.sr |= 0x8000;
    cpu.regs.prefetch = [0x4AFC, 0x4E71];
    rig.long(0x10, 0x2000);
    assert_eq!(cpu.step(&mut rig), Step::Exception(4));
    assert_eq!(cpu.trace_pending, None);
    let (mut cpu, mut rig) = at_nop(0);
    cpu.regs.sr |= 0x8000;
    cpu.regs.prefetch = [0x4E41, 0x4E71];
    rig.long(0x84, 0x2000);
    assert_eq!(cpu.step(&mut rig), Step::Exception(33));
    assert_eq!(cpu.trace_pending, Some(0x2000), "traced into the handler's first instruction");
}
