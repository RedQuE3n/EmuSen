//! What the single-step suite never reaches, each against a bus that counts cycles: interrupts, the exceptions and
//! their Cause, EPC, BadVaddr and mode stack, RFE, coprocessor usability, cache isolation, the debug unit, the
//! multiplier's interlock and the observer (BlackMoon_R3000.md §7). The programs are written with `emusen-mips`.

use emusen_native::debug::{Observer, kind, stop};

use crate::cop0::{cause, code, dcic, sr};
use crate::{Bus, R3000, Size, Step};

const RAM: usize = 1 << 20;

struct Rig {
    ram: Vec<u8>,
    clock: u64,
    idled: u64,
    line: bool,
    commands: Vec<u32>,
    cop2: [u32; 32],
    isolated: Vec<(bool, u32, Size, u32)>,
    writes: Vec<(u32, Size, u32)>,
    /// Addresses from here up meet no device, and the access that just met none.
    absent: u32,
    faulted: bool,
}

fn at(address: u32) -> usize {
    address as usize & (RAM - 1)
}

impl Rig {
    fn new() -> Rig {
        Rig { ram: vec![0; RAM], clock: 0, idled: 0, line: false, commands: Vec::new(), cop2: [0; 32], isolated: Vec::new(), writes: Vec::new(), absent: u32::MAX, faulted: false }
    }

    fn word(&self, address: u32) -> u32 {
        let a = at(address);
        u32::from_le_bytes(self.ram[a..a + 4].try_into().unwrap())
    }

    fn put(&mut self, address: u32, v: u32) {
        let a = at(address);
        self.ram[a..a + 4].copy_from_slice(&v.to_le_bytes());
    }

    fn program(&mut self, origin: u32, source: &str) {
        let words = emusen_mips::assemble(source, origin, &crate::dialect(None)).unwrap_or_else(|e| panic!("{e}"));
        for (i, w) in words.into_iter().enumerate() {
            self.put(origin + 4 * i as u32, w);
        }
    }
}

impl Bus for Rig {
    fn fetch(&mut self, address: u32) -> u32 {
        self.clock += 1;
        self.word(address)
    }
    fn read(&mut self, address: u32, size: Size) -> u32 {
        self.clock += 1;
        self.faulted = address >= self.absent;
        (0..size.bytes()).fold(0, |v, i| v | (self.ram[at(address + i)] as u32) << (8 * i))
    }
    fn write(&mut self, address: u32, size: Size, value: u32) {
        self.clock += 1;
        self.faulted = address >= self.absent;
        self.writes.push((address, size, value));
        for i in 0..size.bytes() {
            self.ram[at(address + i)] = (value >> (8 * i)) as u8;
        }
    }
    fn isolated_read(&mut self, address: u32, size: Size) -> u32 {
        self.clock += 1;
        self.isolated.push((false, address, size, 0));
        0
    }
    fn isolated_write(&mut self, address: u32, size: Size, value: u32) {
        self.clock += 1;
        self.isolated.push((true, address, size, value));
    }
    fn idle(&mut self, cycles: u32) {
        self.clock += cycles as u64;
        self.idled += cycles as u64;
    }
    fn cycles(&mut self) -> u64 {
        self.clock
    }
    fn interrupt(&mut self) -> bool {
        self.line
    }
    fn cop2_data(&mut self, register: u8) -> u32 {
        self.cop2[register as usize]
    }
    fn set_cop2_data(&mut self, register: u8, value: u32) {
        self.cop2[register as usize] = value;
    }
    fn cop2_control(&mut self, register: u8) -> u32 {
        !self.cop2[register as usize]
    }
    fn set_cop2_control(&mut self, _register: u8, _value: u32) {}
    fn cop2_command(&mut self, command: u32) {
        self.commands.push(command);
    }
    fn bus_error(&mut self) -> bool {
        std::mem::take(&mut self.faulted)
    }
}

const ORIGIN: u32 = 0x8000_1000;

fn machine(source: &str) -> (R3000, Rig) {
    let mut rig = Rig::new();
    rig.program(ORIGIN, source);
    let mut cpu = R3000::new();
    cpu.regs.pc = ORIGIN;
    (cpu, rig)
}

fn run(cpu: &mut R3000, rig: &mut Rig, steps: usize) -> Vec<Step> {
    (0..steps).map(|_| cpu.step(rig)).collect()
}

#[test]
fn an_interrupt_is_taken_at_the_boundary_with_the_mode_stack_pushed_and_the_pending_load_landed() {
    let (mut cpu, mut rig) = machine("lw t0, 0x100(zero)\naddiu t1, zero, 1\naddiu t2, zero, 2");
    rig.put(0x100, 0x1234_5678);
    cpu.regs.sr = sr::IEC | 0x0400 | 0b01_00;
    assert_eq!(cpu.step(&mut rig), Step::Instruction);
    rig.line = true;
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::INT as u8));
    let r = &cpu.regs;
    assert_eq!((r.pc, r.epc, r.r[8], r.r[9]), (0x8000_0080, ORIGIN + 4, 0x1234_5678, 0));
    assert_eq!(r.cause & cause::EXCEPTION, 1 << 28, "no ExcCode, no BD, and CE from the interrupted ADDIU's bits 27-26");
    assert_ne!(r.cause & cause::IP2, 0);
    assert_eq!(r.sr & 0x3F, 0b01_01_00, "IEc pushed to IEp, IEp to IEo, the current pair cleared");
    assert_eq!(r.load.register, None);
}

#[test]
fn a_masked_or_disabled_interrupt_is_not_taken_but_ip2_follows_the_line() {
    for sr_value in [0x0401 & !sr::IEC, 0x0001] {
        let (mut cpu, mut rig) = machine("nop\nnop");
        cpu.regs.sr = sr_value;
        rig.line = true;
        assert_eq!(cpu.step(&mut rig), Step::Instruction);
        assert_ne!(cpu.regs.cause & cause::IP2, 0);
        rig.line = false;
        cpu.step(&mut rig);
        assert_eq!(cpu.regs.cause & cause::IP2, 0);
    }
}

#[test]
fn a_software_interrupt_from_cause_and_an_interrupt_in_a_delay_slot() {
    let (mut cpu, mut rig) = machine("addiu t0, zero, 0x100\nmtc0 t0, CAUSE\nnop");
    cpu.regs.sr = sr::IEC | 0x0100;
    run(&mut cpu, &mut rig, 2);
    assert_eq!(cpu.regs.cause & cause::SW, 0x100);
    assert_eq!(cpu.step(&mut rig), Step::Exception(0));
    assert_eq!(cpu.regs.epc, ORIGIN + 8);

    let (mut cpu, mut rig) = machine("j 0x80002000\nnop");
    cpu.regs.sr = sr::IEC | 0x0400;
    cpu.step(&mut rig);
    rig.line = true;
    assert_eq!(cpu.step(&mut rig), Step::Exception(0));
    let r = &cpu.regs;
    assert_eq!((r.epc, r.cause >> 30, r.tar), (ORIGIN, 0b11, 0x8000_2000), "BD and BT set, EPC at the jump, TAR its target");
}

#[test]
fn a_gte_command_meeting_an_interrupt_runs_and_epc_still_points_at_it() {
    let (mut cpu, mut rig) = machine("cop2 0x0180001");
    cpu.regs.sr = sr::IEC | 0x0400 | sr::CU2;
    rig.line = true;
    assert_eq!(cpu.step(&mut rig), Step::Exception(0));
    assert_eq!((rig.commands.as_slice(), cpu.regs.epc), (&[0x0180001][..], ORIGIN));
    assert_eq!(cpu.regs.cause >> 28 & 3, 2, "CE from the interrupted instruction");
}

/// amidog's `psxtest_cpx` INTR on a console: after an MTC0 that enables an interrupt in SR, one more instruction runs
/// before it is taken (BlackMoon_R3000.md D-4).
#[test]
fn an_interrupt_enabled_by_mtc0_sr_waits_one_instruction() {
    let (mut cpu, mut rig) = machine("addiu t0, zero, 0x101\nmtc0 t0, SR\nmflo t1\nnop");
    cpu.regs.cause = 0x100;
    cpu.regs.lo = 0x55;
    assert_eq!(run(&mut cpu, &mut rig, 3), [Step::Instruction; 3], "the MFLO after the MTC0 runs");
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::INT as u8));
    assert_eq!((cpu.regs.epc, cpu.regs.r[9]), (ORIGIN + 12, 0x55));

    let (mut cpu, mut rig) = machine("mtc0 zero, SR\nnop\nnop");
    cpu.regs.sr = sr::IEC | 0x400;
    assert_eq!(cpu.step(&mut rig), Step::Instruction);
    rig.line = true;
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::INT as u8), "the boundary after the MTC0 still sees SR as it was");
    assert_eq!(cpu.regs.epc, ORIGIN + 4);
}

/// `psxtest_cpx` SLOT on a console: an exception in the delay slot of a branch not taken leaves TAR at the
/// instruction after the slot (D-5).
#[test]
fn an_exception_in_a_not_taken_branchs_slot_sets_tar_to_the_fall_through() {
    let (mut cpu, mut rig) = machine("bne zero, zero, 0x80001100\nlw t0, 1(zero)");
    cpu.step(&mut rig);
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::ADEL as u8));
    let r = &cpu.regs;
    assert_eq!((r.epc, r.cause >> 28, r.tar), (ORIGIN, 0b1011, ORIGIN + 8), "BD, BT clear, CE from the LW's bits");
}

/// `psxtest_cpx` BREAK on a console: a load or store that meets no device, and a break of the debug unit, leave CE
/// zero where an address error leaves the opcode's bits (D-6).
#[test]
fn a_bus_error_and_a_debug_break_leave_ce_zero() {
    let (mut cpu, mut rig) = machine("lw t0, 0x200(zero)\nnop\nsh t0, 0x200(zero)\nnop\nlw t0, 0x100(zero)");
    rig.absent = 0x200;
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::DBE as u8));
    assert_eq!(cpu.regs.cause >> 28, 0, "LW's bits are 3");
    cpu.regs.pc = ORIGIN + 8;
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::DBE as u8));
    assert_eq!(cpu.regs.cause >> 28, 0, "SH's bits are 1");
    cpu.regs.pc = ORIGIN + 16;
    let r = &mut cpu.regs;
    r.bda = 0x100;
    r.bdam = u32::MAX;
    r.dcic = dcic::DE | dcic::DAE | dcic::DR | dcic::KD | dcic::TR;
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::BP as u8));
    assert_eq!((cpu.regs.pc, cpu.regs.cause >> 28), (0x8000_0040, 0));
}

#[test]
fn rfe_pops_the_mode_stack_and_leaves_the_old_pair() {
    let (mut cpu, mut rig) = machine("rfe");
    cpu.regs.sr = 0xF000_0000 | 0b11_10_01;
    cpu.step(&mut rig);
    assert_eq!(cpu.regs.sr, 0xF000_0000 | 0b11_11_10);
}

#[test]
fn bev_moves_the_vectors_to_rom() {
    let (mut cpu, mut rig) = machine("syscall");
    cpu.regs.sr = sr::BEV;
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::SYS as u8));
    assert_eq!(cpu.regs.pc, 0xBFC0_0180);
}

#[test]
fn address_errors_on_a_fetch_a_load_and_a_store_in_user_mode() {
    let (mut cpu, mut rig) = machine("jr t0\nnop");
    cpu.regs.r[8] = 0x8000_2002;
    run(&mut cpu, &mut rig, 2);
    let fetches = rig.clock;
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::ADEL as u8));
    assert_eq!((cpu.regs.epc, cpu.regs.bad_vaddr, rig.clock), (0x8000_2002, 0x8000_2002, fetches), "no fetch");
    assert_eq!(cpu.regs.cause >> 31, 0, "the target is not in a delay slot");

    let (mut cpu, mut rig) = machine("lw t1, 0(t0)\nsw t1, 0(t0)");
    rig.put(0x1000_0000 & (RAM as u32 - 1), 0);
    cpu.regs.r[8] = 0x8000_0100;
    cpu.regs.sr = sr::KUC;
    cpu.regs.pc = 0x0000_1000;
    rig.program(0x1000, "lw t1, 0(t0)\nsw t1, 0(t0)");
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::ADEL as u8));
    assert_eq!(cpu.regs.bad_vaddr, 0x8000_0100);
    cpu.regs.pc = 0x0000_1004;
    cpu.regs.sr = sr::KUC;
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::ADES as u8));
    assert!(rig.writes.is_empty());
}

#[test]
fn reserved_and_unusable_instructions() {
    let cases: [(&str, u32, Option<u8>); 12] = [
        (".word 0xFC000000", 0, Some(code::RI as u8)),
        (".word 0x0000003F", 0, Some(code::RI as u8)),
        ("mfc2 t0, $1", 0, Some(code::CPU as u8)),
        ("mfc2 t0, $1", sr::CU2, None),
        ("mfc0 t0, $0", 0, Some(code::RI as u8)),
        ("tlbr", 0, Some(code::RI as u8)),
        ("cop0 0x1e00000", 0, None),
        ("cfc0 t0, $3", 0, Some(code::RI as u8)),
        ("cop1 0x0000000", 0, Some(code::CPU as u8)),
        ("cop1 0x0000000", sr::CU0 << 1, None),
        ("swc0 $1, 0(zero)", 0, Some(code::CPU as u8)),
        ("swc0 $1, 0(zero)", sr::CU0, None),
    ];
    for (source, sr_value, want) in cases {
        let (mut cpu, mut rig) = machine(source);
        cpu.regs.sr = sr_value;
        let got = cpu.step(&mut rig);
        assert_eq!(got, want.map_or(Step::Instruction, Step::Exception), "{source}");
        if want == Some(code::CPU as u8) {
            let z = emusen_mips::Word(rig.word(ORIGIN)).z() as u32;
            assert_eq!(cpu.regs.cause >> 28 & 3, z, "{source}: CE");
        }
    }
}

#[test]
fn coprocessor_0_in_user_mode_is_unusable_but_its_garbage_registers_read() {
    let (mut cpu, mut rig) = machine("");
    rig.program(0x1000, "mfc0 t0, SR\nmfc0 t0, $16");
    cpu.regs.pc = 0x1000;
    cpu.regs.sr = sr::KUC;
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::CPU as u8));
    cpu.regs.pc = 0x1004;
    cpu.regs.sr = sr::KUC;
    assert_eq!(cpu.step(&mut rig), Step::Instruction);
}

#[test]
fn mfc0_reads_through_the_load_delay_and_mtc0_keeps_to_the_writable_bits() {
    let (mut cpu, mut rig) = machine("mfc0 t0, PRID\nor t1, t0, zero\nor t2, t0, zero\nmtc0 t3, CAUSE\nmtc0 t4, SR\nmtc0 t3, DCIC");
    cpu.regs.r[11] = u32::MAX;
    cpu.regs.r[12] = !(sr::KUC | sr::IEC);
    run(&mut cpu, &mut rig, 6);
    let r = &cpu.regs;
    assert_eq!((r.r[9], r.r[10]), (0, 2));
    assert_eq!((r.sr, r.cause, r.dcic), (sr::WRITABLE & !(sr::KUC | sr::IEC), cause::SW, dcic::WRITABLE));
}

#[test]
fn bc2f_is_always_taken_and_bc2t_never() {
    let (mut cpu, mut rig) = machine("bc2t 0x80001100\nnop\nbc2f 0x80001200\nnop");
    cpu.regs.sr = sr::CU2;
    run(&mut cpu, &mut rig, 4);
    assert_eq!(cpu.regs.pc, 0x8000_1200);
}

#[test]
fn overflow_in_a_delay_slot_records_the_branch() {
    let (mut cpu, mut rig) = machine("beq zero, zero, 0x80001100\nadd t0, t1, t1");
    cpu.regs.r[9] = 0x4000_0000;
    cpu.regs.r[8] = 7;
    cpu.step(&mut rig);
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::OV as u8));
    let r = &cpu.regs;
    assert_eq!((r.epc, r.cause >> 30, r.tar, r.r[8]), (ORIGIN, 0b11, 0x8000_1100, 7));
}

#[test]
fn the_multiplier_holds_mfhi_and_mflo_by_rs() {
    let cases: [(&str, u32, u32, u64); 6] = [
        ("multu", 0x7FF, 3, 5),
        ("multu", 0x800, 3, 8),
        ("multu", 0x10_0000, 3, 12),
        ("mult", 0xFFFF_F800, 3, 5),
        ("mult", 0xFFF0_0000, 3, 8),
        ("div", 0x1234_5678, 7, 35),
    ];
    for (op, a, b, idle) in cases {
        let (mut cpu, mut rig) = machine(&format!("{op} t0, t1\nmflo t2"));
        cpu.regs.r[8] = a;
        cpu.regs.r[9] = b;
        run(&mut cpu, &mut rig, 2);
        assert_eq!(rig.idled, idle, "{op} {a:x}");
    }
    let (mut cpu, mut rig) = machine("mult t0, t1\nnop\nnop\nnop\nmfhi t2\ndiv t0, zero\nmflo t3\nmfhi t4");
    cpu.regs.r[8] = 0x8000_0000;
    cpu.regs.r[9] = 2;
    run(&mut cpu, &mut rig, 8);
    assert_eq!(rig.idled, 9 + 35);
    assert_eq!((cpu.regs.r[10], cpu.regs.r[11], cpu.regs.r[12]), (u32::MAX, 1, 0x8000_0000));
}

#[test]
fn an_isolated_cache_takes_loads_and_stores_and_memory_sees_none() {
    let (mut cpu, mut rig) = machine("sw t0, 0x100(zero)\nlw t1, 0x100(zero)\nsb t0, 0x104(zero)");
    cpu.regs.sr = sr::ISC;
    cpu.regs.r[8] = 0xAABB_CCDD;
    run(&mut cpu, &mut rig, 3);
    assert!(rig.writes.is_empty());
    assert_eq!(rig.isolated, [(true, 0x100, Size::Word, 0xAABB_CCDD), (false, 0x100, Size::Word, 0), (true, 0x104, Size::Byte, 0xAABB_CCDD)]);
}

#[test]
fn the_debug_unit_breaks_on_the_pc_and_on_a_store() {
    let (mut cpu, mut rig) = machine("nop\nsw t0, 0x200(zero)");
    let r = &mut cpu.regs;
    r.bpc = ORIGIN;
    r.bpcm = u32::MAX;
    r.dcic = dcic::DE | dcic::PCE | dcic::KD;
    assert_eq!(cpu.step(&mut rig), Step::Instruction, "without TR only the status bits");
    assert_eq!(cpu.regs.dcic & 3, dcic::DB | dcic::PC);
    let r = &mut cpu.regs;
    r.bda = 0x200;
    r.bdam = !3;
    r.dcic = dcic::DE | dcic::DAE | dcic::DW | dcic::KD | dcic::TR;
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::BP as u8));
    assert_eq!((cpu.regs.pc, cpu.regs.epc, cpu.regs.dcic & 0x1F), (0x8000_0040, ORIGIN + 4, dcic::DB | dcic::DA | dcic::W));
    assert!(rig.writes.is_empty());
}

#[test]
fn the_load_delay_corners_the_suite_holds_in_sequence() {
    let (mut cpu, mut rig) = machine(
        "lw t0, 0x100(zero)\nor t1, t0, zero\nor t2, t0, zero\n\
         lw t3, 0x100(zero)\nlw t3, 0x104(zero)\nor t4, t3, zero\nor t5, t3, zero\n\
         lwl t6, 0x10B(zero)\nlwr t6, 0x108(zero)\nnop\nor t7, t6, zero",
    );
    rig.put(0x100, 0x1111_1111);
    rig.put(0x104, 0x2222_2222);
    rig.put(0x108, 0x4433_2211);
    cpu.regs.r[8] = 5;
    run(&mut cpu, &mut rig, 11);
    let r = &cpu.regs.r;
    assert_eq!((r[9], r[10]), (5, 0x1111_1111));
    assert_eq!((r[12], r[13]), (0, 0x2222_2222), "the first load to t3 never lands");
    assert_eq!(r[15], 0x4433_2211, "LWL and LWR merge with no delay between them");
}

#[derive(Default)]
struct Watch {
    stop_at: Option<u32>,
    wrote: Vec<(u32, u32, u8, u32)>,
    calls: Vec<(u32, u32, u32)>,
    returns: usize,
}

impl Observer for Watch {
    fn before(&mut self, _processor: usize, pc: u32) -> u32 {
        if self.stop_at == Some(pc) { stop::BREAKPOINT } else { stop::FRAME }
    }
    fn wrote(&mut self, space: u32, address: u32, value: u8, pc: u32) {
        self.wrote.push((space, address, value, pc));
    }
    fn called(&mut self, source: u32, target: u32, kind: u32) {
        self.calls.push((source, target, kind));
    }
    fn returned(&mut self) {
        self.returns += 1;
    }
}

#[test]
fn the_observer_sees_the_boundary_each_stored_byte_calls_returns_and_exceptions() {
    let (mut cpu, mut rig) = machine("jal 0x80001100\nnop\nsyscall");
    rig.program(0x8000_1100, "swl t0, 0x201(zero)\njr ra\nnop");
    cpu.space = 7;
    cpu.regs.r[8] = 0xAABB_CCDD;
    let mut w = Watch { stop_at: Some(ORIGIN + 8), ..Watch::default() };
    let steps: Vec<Step> = (0..6).map(|_| cpu.step_observed(&mut rig, &mut w)).collect();
    assert_eq!(steps[5], Step::Observed(stop::BREAKPOINT));
    assert_eq!(cpu.regs.pc, ORIGIN + 8, "a stop runs nothing");
    assert_eq!(w.wrote, [(7, 0x200, 0xBB, 0x8000_1100), (7, 0x201, 0xAA, 0x8000_1100)]);
    assert_eq!((w.calls.as_slice(), w.returns), (&[(ORIGIN, 0x8000_1100, kind::CALL)][..], 1));
    w.stop_at = None;
    assert_eq!(cpu.step_observed(&mut rig, &mut w), Step::Exception(code::SYS as u8));
    assert_eq!(w.calls[1], (ORIGIN + 8, 0x8000_0080, kind::BRK));
}
