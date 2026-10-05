//! The step's boundary, which the single-step suite never reaches: HALT, NMI, INT in each mode, EI's delay and the
//! P/V of LD A,I when an interrupt follows it, each against a bus that counts T-states (Beryl_Z80.md §5).

use crate::exec::PV;
use crate::{Bus, Step, Z80};
use emusen_native::debug::kind;

struct Rig {
    ram: Vec<u8>,
    t: u32,
    int: bool,
    nmi: bool,
    vector: u8,
}

impl Rig {
    fn new(code: &[u8]) -> Rig {
        let mut ram = vec![0u8; 0x10000];
        ram[..code.len()].copy_from_slice(code);
        Rig { ram, t: 0, int: false, nmi: false, vector: 0xFF }
    }
}

impl Bus for Rig {
    fn fetch(&mut self, a: u16, _: u16) -> u8 {
        self.t += 4;
        self.ram[a as usize]
    }
    fn read(&mut self, a: u16) -> u8 {
        self.t += 3;
        self.ram[a as usize]
    }
    fn write(&mut self, a: u16, v: u8) {
        self.t += 3;
        self.ram[a as usize] = v;
    }
    fn input(&mut self, _: u16) -> u8 {
        self.t += 4;
        0xFF
    }
    fn output(&mut self, _: u16, _: u8) {
        self.t += 4;
    }
    fn idle(&mut self, t: u32) {
        self.t += t;
    }
    fn int_line(&mut self) -> bool {
        self.int
    }
    fn nmi_edge(&mut self) -> bool {
        std::mem::take(&mut self.nmi)
    }
    fn acknowledge(&mut self) -> u8 {
        self.t += 6;
        self.vector
    }
}

fn cpu() -> Z80 {
    let mut z = Z80::new();
    z.regs.sp = 0x8000;
    z
}

/// Steps one, returning what it did and the T-states it took.
fn step(z: &mut Z80, b: &mut Rig) -> (Step, u32) {
    let t = b.t;
    let s = z.step(b);
    (s, b.t - t)
}

fn stacked(b: &Rig, z: &Z80) -> u16 {
    let sp = z.regs.sp as usize;
    b.ram[sp] as u16 | (b.ram[sp + 1] as u16) << 8
}

#[test]
fn ei_delays_the_interrupt_by_one_instruction_and_mode_1_takes_thirteen_t_states() {
    let mut z = cpu();
    z.regs.im = 1;
    let mut b = Rig::new(&[0xFB, 0x00, 0x00]);
    b.int = true;
    assert_eq!(step(&mut z, &mut b), (Step::Instruction, 4), "EI");
    assert_eq!(step(&mut z, &mut b), (Step::Instruction, 4), "the instruction after EI runs");
    assert_eq!(step(&mut z, &mut b), (Step::Interrupt(kind::IRQ), 13));
    assert_eq!((z.regs.pc, z.regs.wz, stacked(&b, &z), z.regs.iff1, z.regs.iff2), (0x38, 0x38, 2, false, false));
}

#[test]
fn mode_2_reads_the_vector_table_in_nineteen_t_states() {
    let mut z = cpu();
    z.regs.im = 2;
    z.regs.i = 0x40;
    z.regs.iff1 = true;
    z.regs.iff2 = true;
    let mut b = Rig::new(&[0x00]);
    b.ram[0x4010] = 0x34;
    b.ram[0x4011] = 0x12;
    b.vector = 0x10;
    b.int = true;
    assert_eq!(step(&mut z, &mut b), (Step::Interrupt(kind::IRQ), 19));
    assert_eq!((z.regs.pc, stacked(&b, &z)), (0x1234, 0));
}

#[test]
fn mode_0_executes_the_rst_on_the_bus() {
    let mut z = cpu();
    z.regs.iff1 = true;
    let mut b = Rig::new(&[0x00]);
    b.vector = 0xEF;
    b.int = true;
    assert_eq!(step(&mut z, &mut b), (Step::Interrupt(kind::IRQ), 13));
    assert_eq!(z.regs.pc, 0x28);
}

#[test]
fn di_masks_int_and_nmi_takes_priority_and_retn_restores_iff1() {
    let mut z = cpu();
    z.regs.im = 1;
    z.regs.iff1 = true;
    z.regs.iff2 = true;
    let mut b = Rig::new(&[0x00]);
    b.ram[0x66] = 0xED;
    b.ram[0x67] = 0x45;
    b.int = true;
    b.nmi = true;
    assert_eq!(step(&mut z, &mut b), (Step::Interrupt(kind::NMI), 11));
    assert_eq!((z.regs.pc, z.regs.iff1, z.regs.iff2, stacked(&b, &z)), (0x66, false, true, 0));
    b.int = true;
    assert_eq!(step(&mut z, &mut b), (Step::Instruction, 14), "RETN, INT masked since NMI cleared IFF1");
    assert_eq!((z.regs.pc, z.regs.iff1), (0, true));
    let mut z = cpu();
    z.regs.im = 1;
    let mut b = Rig::new(&[0xF3, 0x00]);
    z.regs.iff1 = true;
    assert_eq!(step(&mut z, &mut b).0, Step::Instruction, "DI");
    b.int = true;
    assert_eq!(step(&mut z, &mut b).0, Step::Instruction, "no interrupt after DI");
}

#[test]
fn halt_repeats_its_nop_until_an_interrupt_which_stacks_the_next_instruction() {
    let mut z = cpu();
    z.regs.im = 1;
    z.regs.iff1 = true;
    let mut b = Rig::new(&[0x76, 0x00]);
    assert_eq!(step(&mut z, &mut b), (Step::Halted, 4));
    let r = z.regs.r;
    for _ in 0..3 {
        assert_eq!(step(&mut z, &mut b), (Step::Halted, 4));
        assert_eq!(z.regs.pc, 1);
    }
    assert_eq!(z.regs.r, r + 3, "each NOP is an M1 with its refresh");
    b.int = true;
    assert_eq!(step(&mut z, &mut b), (Step::Interrupt(kind::IRQ), 13));
    assert_eq!((z.halted, stacked(&b, &z)), (false, 1));
}

#[test]
fn an_interrupt_after_ld_a_i_clears_its_p_v() {
    let mut z = cpu();
    z.regs.im = 1;
    z.regs.iff1 = true;
    z.regs.iff2 = true;
    let mut b = Rig::new(&[0xED, 0x57]);
    assert_eq!(step(&mut z, &mut b), (Step::Instruction, 9));
    assert!(z.regs.p && z.regs.af as u8 & PV != 0, "LD A,I copies IFF2 to P/V");
    b.int = true;
    assert_eq!(step(&mut z, &mut b).0, Step::Interrupt(kind::IRQ));
    assert_eq!(z.regs.af as u8 & PV, 0);
}
