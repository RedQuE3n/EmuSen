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
    /// The clock at which a run must end, and the clock's bit that drives the interrupt line when it is not held.
    until: u64,
    line_bit: Option<u32>,
}

fn at(address: u32) -> usize {
    address as usize & (RAM - 1)
}

impl Rig {
    fn new() -> Rig {
        Rig { ram: vec![0; RAM], clock: 0, idled: 0, line: false, commands: Vec::new(), cop2: [0; 32], isolated: Vec::new(), writes: Vec::new(), absent: u32::MAX, faulted: false, until: u64::MAX, line_bit: None }
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
        self.line || self.line_bit.is_some_and(|b| self.clock >> b & 1 != 0)
    }
    fn running(&mut self) -> bool {
        self.clock < self.until
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
/// before it is taken (BlackMoon_R3000.md D-17).
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
/// instruction after the slot (D-18).
#[test]
fn an_exception_in_a_not_taken_branchs_slot_sets_tar_to_the_fall_through() {
    let (mut cpu, mut rig) = machine("bne zero, zero, 0x80001100\nlw t0, 1(zero)");
    cpu.step(&mut rig);
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::ADEL as u8));
    let r = &cpu.regs;
    assert_eq!((r.epc, r.cause >> 28, r.tar), (ORIGIN, 0b1011, ORIGIN + 8), "BD, BT clear, CE from the LW's bits");
}

/// `psxtest_cpx` BREAK on a console: a load or store that meets no device, and a break of the debug unit, leave CE
/// zero where an address error leaves the opcode's bits (D-19).
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
    r.dcic = dcic::DE | dcic::DAE | dcic::DR | dcic::UD | dcic::TR;
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
    r.dcic = dcic::DE | dcic::DAE | dcic::DW | dcic::UD | dcic::TR;
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::BP as u8));
    assert_eq!((cpu.regs.pc, cpu.regs.epc, cpu.regs.dcic & 0x1F), (0x8000_0040, ORIGIN + 4, dcic::DB | dcic::DA | dcic::W));
    assert!(rig.writes.is_empty());
}

/// `psxtest_cpx` BREAK's model (D-20): KD answers for an address with bit 31 set and UD for one without, whatever the
/// mode; in kernel mode a store to 200h breaks only with UD, and the PC at 80001000h only with KD.
#[test]
fn the_debug_unit_answers_to_kd_or_ud_by_the_address_segment() {
    for (mode, breaks) in [(dcic::KD, false), (dcic::UD, true)] {
        let (mut cpu, mut rig) = machine("sw t0, 0x200(zero)");
        let r = &mut cpu.regs;
        r.bda = 0x200;
        r.bdam = u32::MAX;
        r.dcic = dcic::DE | dcic::DAE | dcic::DW | mode | dcic::TR;
        let want = if breaks { Step::Exception(code::BP as u8) } else { Step::Instruction };
        assert_eq!((cpu.step(&mut rig), cpu.regs.dcic & 0x1F != 0), (want, breaks), "a user address, {mode:08X}");
    }
    for (mode, breaks) in [(dcic::KD, true), (dcic::UD, false)] {
        let (mut cpu, mut rig) = machine("sw t0, 0x200(zero)\nnop");
        let r = &mut cpu.regs;
        r.r[8] = 0x8000_0000;
        r.bpc = ORIGIN;
        r.bpcm = u32::MAX;
        r.dcic = dcic::DE | dcic::PCE | mode;
        cpu.step(&mut rig);
        assert_eq!(cpu.regs.dcic & 3 != 0, breaks, "a kernel PC, {mode:08X}");
    }
}

/// D-20: the data break compares the address the access puts on the bus, LWL's and SWL's aligned word and LWR's and
/// SWR's own address, as `psxtest_cpx`'s model has it.
#[test]
fn lwl_and_swl_match_their_aligned_word_and_lwr_and_swr_their_address() {
    for (source, write, bda, matches) in [
        ("lwl t0, 0x201(zero)", false, 0x200, true),
        ("lwl t0, 0x201(zero)", false, 0x201, false),
        ("lwr t0, 0x201(zero)", false, 0x201, true),
        ("lwr t0, 0x201(zero)", false, 0x200, false),
        ("swl t0, 0x203(zero)", true, 0x200, true),
        ("swr t0, 0x203(zero)", true, 0x203, true),
        ("swr t0, 0x203(zero)", true, 0x200, false),
    ] {
        let (mut cpu, mut rig) = machine(source);
        let r = &mut cpu.regs;
        r.bda = bda;
        r.bdam = u32::MAX;
        r.dcic = dcic::DE | dcic::DAE | if write { dcic::DW } else { dcic::DR } | dcic::UD;
        cpu.step(&mut rig);
        assert_eq!(cpu.regs.dcic & dcic::DA != 0, matches, "{source} against {bda:X}");
    }
}

/// D-20: a misaligned access that matches sets the status bits and takes the address error, TR or not.
#[test]
fn a_misaligned_access_that_matches_sets_the_status_bits_and_takes_the_address_error() {
    let (mut cpu, mut rig) = machine("lw t0, 0x201(zero)");
    let r = &mut cpu.regs;
    r.bda = 0x201;
    r.bdam = u32::MAX;
    r.dcic = dcic::DE | dcic::DAE | dcic::DR | dcic::UD | dcic::TR;
    assert_eq!(cpu.step(&mut rig), Step::Exception(code::ADEL as u8));
    assert_eq!((cpu.regs.pc, cpu.regs.dcic & 0x1F), (0x8000_0080, dcic::DB | dcic::DA | dcic::R));
}

/// D-21: SWC0, SWC1 and SWC3 with their CU bit set store a word on the bus, which can meet no device.
#[test]
fn swc0_swc1_and_swc3_store_on_the_bus() {
    for z in [0, 1, 3] {
        let (mut cpu, mut rig) = machine(&format!(".word {:#X}", 0xE000_0000u32 | z << 26 | 1 << 16 | 0x200));
        cpu.regs.sr = sr::CU0 << z;
        rig.absent = 0x200;
        assert_eq!(cpu.step(&mut rig), Step::Exception(code::DBE as u8), "SWC{z}");
        assert_eq!(rig.writes.iter().map(|w| (w.0, w.1)).collect::<Vec<_>>(), [(0x200, Size::Word)], "SWC{z}");
    }
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
    read: Vec<(u32, u32, u8, u32)>,
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
    fn read(&mut self, space: u32, address: u32, value: u8, pc: u32) {
        self.read.push((space, address, value, pc));
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

/// Each byte a load reads is reported once it has been read, LWL's and LWR's at the bytes of the word they reach; and
/// `load_ahead`, asked in front of every load in every mode the step distinguishes, names exactly the bytes the step then
/// reports, except where only the read can tell: a load that meets no device.
#[test]
fn every_load_reports_the_bytes_it_read_and_load_ahead_names_them_first() {
    let (mut cpu, mut rig) = machine("lwl t6, 0x10B(zero)\nlwr t7, 0x109(zero)");
    rig.put(0x108, 0x4433_2211);
    cpu.space = 7;
    let mut w = Watch::default();
    cpu.step_observed(&mut rig, &mut w);
    cpu.step_observed(&mut rig, &mut w);
    assert_eq!(w.read, [(7, 0x108, 0x11, ORIGIN), (7, 0x109, 0x22, ORIGIN), (7, 0x10A, 0x33, ORIGIN), (7, 0x10B, 0x44, ORIGIN), (7, 0x109, 0x22, ORIGIN + 4), (7, 0x10A, 0x33, ORIGIN + 4), (7, 0x10B, 0x44, ORIGIN + 4)]);

    let loads = ["lb", "lbu", "lh", "lhu", "lw", "lwl", "lwr", "lwc2"];
    let modes: [(&str, u32, u32); 5] = [("kernel", sr::CU2, 0), ("isolated", sr::CU2 | sr::ISC, 0), ("user, KSEG0", sr::CU2 | sr::KUC, 0x8000_0000), ("CU2 off", 0, 0), ("no device", sr::CU2, 0x0008_0000)];
    let (mut checked, mut unknowable) = (0, 0);
    for op in loads {
        for (mode, status, base) in modes {
            for offset in 0..4u32 {
                let reg = if op == "lwc2" { "$0" } else { "t1" };
                let (mut cpu, mut rig) = machine(&format!("{op} {reg}, {:#x}(t0)", 0x200 + offset));
                for i in 0..16 {
                    rig.ram[0x200 + i] = 0xA0 + i as u8;
                }
                rig.absent = if mode == "no device" { 0x0008_0000 } else { u32::MAX };
                cpu.regs.r[8] = base;
                cpu.regs.sr = status;
                cpu.space = 3;
                let ahead = cpu.load_ahead(rig.word(ORIGIN));
                let mut w = Watch::default();
                cpu.step_observed(&mut rig, &mut w);
                let seen = w.read.first().map(|r| (r.1, w.read.len() as u32));
                assert!(w.read.iter().enumerate().all(|(i, r)| r.1 == w.read[0].1 + i as u32 && r.3 == ORIGIN && (mode == "user, KSEG0" || r.2 == rig.ram[at(r.1)])), "{op} {mode} +{offset}: {:x?}", w.read);
                if mode == "no device" {
                    assert_eq!(seen, None, "{op} +{offset}: a load that met no device reports nothing");
                    unknowable += ahead.is_some() as u32;
                } else {
                    assert_eq!(ahead, seen, "{op} in {mode} at +{offset}");
                    checked += 1;
                }
            }
        }
    }
    assert_eq!((checked, unknowable), (128, 22), "every load that would reach no device is a case only the read can tell");
    let (cpu, _) = machine("addiu t0, t0, 4");
    assert_eq!(cpu.load_ahead(0x2508_0004), None, "no load");
}

// --- Runs (BlackMoon_R3000.md §13) ---

/// What `R3000::run` promises, made from `step`: steps until one is not an instruction, the limit, or the bus's no.
fn stepped(cpu: &mut R3000, rig: &mut Rig, limit: u32) -> (u32, Step) {
    let mut n = 0;
    loop {
        let s = cpu.step(rig);
        n += 1;
        if s != Step::Instruction || n >= limit || !rig.running() {
            return (n, s);
        }
    }
}

/// A handler at each of the four vectors that returns past the instruction: `mfc0 k0, EPC`, `addiu k0, k0, 4`,
/// `jr k0`, `rfe`.
fn handlers(rig: &mut Rig) {
    for vector in [0x8000_0080, 0x8000_0040, 0xBFC0_0180, 0xBFC0_0140] {
        rig.program(vector, "mfc0 k0, EPC\naddiu k0, k0, 4\njr k0\nrfe");
    }
}

#[test]
fn a_bus_that_does_not_say_makes_a_run_one_step() {
    let mut bus = crate::singlestep::FlatBus::new(Vec::new());
    let mut cpu = R3000::new();
    cpu.regs.pc = ORIGIN;
    assert_eq!(cpu.run(&mut bus, 10), (1, Step::Instruction));
    assert_eq!(cpu.regs.pc, ORIGIN + 4);
}

#[test]
fn a_run_ends_at_its_limit_at_the_buss_no_and_at_an_exception() {
    let source = "addiu t0, t0, 1\naddiu t0, t0, 1\naddiu t0, t0, 1\naddiu t0, t0, 1\naddiu t0, t0, 1\nsyscall\naddiu t0, t0, 1";
    let (mut cpu, mut rig) = machine(source);
    assert_eq!((cpu.run(&mut rig, 3), cpu.regs.r[8], cpu.regs.pc), ((3, Step::Instruction), 3, ORIGIN + 12), "the limit");
    let (mut cpu, mut rig) = machine(source);
    rig.until = 4;
    assert_eq!((cpu.run(&mut rig, 100), cpu.regs.r[8], rig.clock), ((4, Step::Instruction), 4, 4), "the bus's no, after the step that reached it");
    let (mut cpu, mut rig) = machine(source);
    assert_eq!((cpu.run(&mut rig, 100), cpu.regs.r[8], cpu.regs.epc), ((6, Step::Exception(code::SYS as u8)), 5, ORIGIN + 20), "an exception ends it");
}

/// Inside a run the boundary is the step's: an interrupt whose line rises is taken where the steps take it, one that
/// MTC0 enables waits one instruction (D-17), a PC break armed by MTC0 breaks, and a jump to an address that cannot be
/// fetched from faults.
#[test]
fn a_runs_boundary_is_the_steps() {
    let nops = "addiu t0, t0, 1\n".repeat(12);
    let (mut cpu, mut rig) = machine(&nops);
    cpu.regs.sr = sr::IEC | 0x0400;
    rig.line_bit = Some(3);
    assert_eq!((cpu.run(&mut rig, 100), cpu.regs.r[8], cpu.regs.epc), ((9, Step::Exception(code::INT as u8)), 8, ORIGIN + 32), "the line rises at the eighth cycle");

    let (mut cpu, mut rig) = machine("addiu t0, zero, 0x401\naddiu t1, t1, 1\nmtc0 t0, SR\naddiu t1, t1, 1\naddiu t1, t1, 1\naddiu t1, t1, 1");
    rig.line = true;
    assert_eq!((cpu.run(&mut rig, 100), cpu.regs.r[9], cpu.regs.epc), ((5, Step::Exception(code::INT as u8)), 2, ORIGIN + 16), "one instruction after the MTC0");

    let (mut cpu, mut rig) = machine("addiu t1, t1, 1\naddiu t1, t1, 1\nmtc0 t0, DCIC\naddiu t1, t1, 1\naddiu t1, t1, 1\naddiu t1, t1, 1");
    cpu.regs.r[8] = dcic::DE | dcic::PCE | dcic::KD | dcic::TR;
    (cpu.regs.bpc, cpu.regs.bpcm) = (ORIGIN + 16, u32::MAX);
    assert_eq!((cpu.run(&mut rig, 100), cpu.regs.r[9], cpu.regs.pc), ((5, Step::Exception(code::BP as u8)), 3, 0x8000_0040), "the PC break MTC0 armed");

    let (mut cpu, mut rig) = machine("addiu t1, t1, 1\naddiu t1, t1, 1\njr t0\naddiu t1, t1, 1\naddiu t1, t1, 1");
    cpu.regs.r[8] = ORIGIN + 0x102;
    assert_eq!((cpu.run(&mut rig, 100), cpu.regs.r[9], cpu.regs.bad_vaddr), ((5, Step::Exception(code::ADEL as u8)), 3, ORIGIN + 0x102), "a PC that is not a word's");

    let (mut cpu, mut rig) = machine("addiu t1, t1, 1\naddiu t1, t1, 1\njr t0\naddiu t1, t1, 1\naddiu t1, t1, 1");
    (cpu.regs.pc, cpu.regs.sr, cpu.regs.r[8]) = (ORIGIN & 0x1FFF_FFFF, sr::KUC, ORIGIN + 0x100);
    assert_eq!((cpu.run(&mut rig, 100), cpu.regs.r[9], cpu.regs.bad_vaddr), ((5, Step::Exception(code::ADEL as u8)), 3, ORIGIN + 0x100), "kernel space in user mode");
}

struct Random(u64);

impl Random {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 16) as u32
    }
    fn below(&mut self, n: u32) -> u32 {
        self.next() % n
    }
    fn of(&mut self, choices: &[u32]) -> u32 {
        choices[self.below(choices.len() as u32) as usize]
    }
}

/// A random instruction over registers 8-15, which hold numbers and addresses, 16-19, which hold values for SR, and
/// 20-21, which hold values for DCIC: mostly the common instructions, with every other kind among them.
fn random_word(g: &mut Random) -> u32 {
    let t = |g: &mut Random| 8 + g.below(8);
    let (rs, rt, rd, sa, imm) = (t(g), t(g), t(g), g.below(32), g.next() & 0xFFFF);
    let small = (g.below(32) as i32 - 16) as u32 & 0xFFFF;
    let r_type = |funct: u32| rs << 21 | rt << 16 | rd << 11 | sa << 6 | funct;
    let i_type = |op: u32, imm: u32| op << 26 | rs << 21 | rt << 16 | imm;
    let cop = |z: u32, sub: u32, rt: u32, rd: u32| (0x10 + z) << 26 | sub << 21 | rt << 16 | rd << 11;
    match g.below(64) {
        0..=17 => r_type(g.of(&[0x00, 0x02, 0x03, 0x04, 0x06, 0x07, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x2A, 0x2B])),
        18..=27 => i_type(8 + g.below(8), if g.below(4) == 0 { imm } else { imm & 0xFF }),
        28..=33 => i_type(4 + g.below(4), small),
        34 => 1 << 26 | rs << 21 | g.of(&[0, 1, 16, 17]) << 16 | small,
        35 => g.of(&[2, 3]) << 26 | (ORIGIN >> 2 & 0x03FF_FFFF) + g.below(200),
        36 => rs << 21 | g.of(&[0, 31, rd]) << 11 | g.of(&[8, 9]),
        37..=44 => i_type(g.of(&[0x20, 0x21, 0x23, 0x24, 0x25, 0x23, 0x22, 0x26]), g.below(0x200)),
        45..=50 => i_type(g.of(&[0x28, 0x29, 0x2B, 0x2B, 0x2A, 0x2E]), g.below(0x200)),
        51..=53 => r_type(g.of(&[0x10, 0x11, 0x12, 0x13, 0x18, 0x19, 0x1A, 0x1B])) & !(31 << 6),
        54 => cop(0, 0, t(g), g.of(&[3, 5, 6, 7, 8, 9, 11, 12, 13, 14, 15, 2, 16])),
        55..=56 => cop(0, 4, 16 + g.below(4), 12),
        57 => match g.below(4) {
            0 => cop(0, 4, 20 + g.below(2), 7),
            1 => cop(0, 4, t(g), g.of(&[13, 3, 5, 9, 11])),
            2 => 0x4200_0010,
            _ => cop(0, g.of(&[2, 6]), t(g), 0),
        },
        58..=59 => cop(2, g.of(&[0, 2, 4, 6]), t(g), g.below(32)),
        60 => {
            let (command, offset) = (g.next() & 0x00FF_FFF0, g.below(0x200));
            g.of(&[0x4A00_0001 | command, 0x32u32 << 26 | rs << 21 | rt << 16 | offset, 0x3Au32 << 26 | rs << 21 | rt << 16 | offset])
        }
        61 => g.of(&[0x0000_000C, 0x0000_000D, 0xFC00_0000, 0x4500_0000 | small, 0x4900_0000 | small]),
        _ => 0,
    }
}

/// Runs against the steps they stand for, over random programs: every kind of instruction, exceptions returned from by
/// handlers at the four vectors, an interrupt line that rises and falls with the clock, SR written with interrupts on
/// and off, user mode, the cache isolated and BEV, DCIC armed on a PC and on data, loads and stores that are unaligned
/// or meet no device; runs of random length and limit. After each run the processor and the bus must be where the
/// steps left a copy, and the run must have counted the same steps and ended in the same one.
#[test]
fn a_run_is_its_steps_on_random_programs() {
    let mut runs = 0u64;
    let (mut longest, mut exceptions, mut interrupts) = (0, 0u32, 0u32);
    for seed in 1..=400u64 {
        let mut g = Random(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let mut rig = Rig::new();
        handlers(&mut rig);
        for i in 0..400 {
            rig.put(ORIGIN + 4 * i, random_word(&mut g));
        }
        for i in 0..0x400 {
            rig.ram[0x4_0000 + i] = g.next() as u8;
        }
        rig.absent = if seed % 5 == 0 { 0x8004_0100 } else { u32::MAX };
        rig.line_bit = [None, Some(5), Some(7), Some(9)][(seed % 4) as usize];
        let mut cpu = R3000::new();
        cpu.regs.pc = ORIGIN;
        for i in 8..16 {
            cpu.regs.r[i] = if i < 12 { 0x8004_0000 + g.below(0x80) * 4 + if g.below(8) == 0 { g.below(4) } else { 0 } } else { g.next() };
        }
        let interrupts_on = sr::IEC | 0x0400 | sr::CU2;
        cpu.regs.r[16..22].copy_from_slice(&[interrupts_on, sr::CU2, interrupts_on | sr::BEV, if seed % 7 == 0 { sr::KUC | sr::IEC | 0x0400 } else if seed % 11 == 0 { sr::ISC | sr::CU2 } else { 0x0401 }, dcic::DE | dcic::PCE | dcic::KD | dcic::TR, dcic::DE | dcic::DAE | dcic::DR | dcic::DW | dcic::KD | dcic::TR]);
        (cpu.regs.bpc, cpu.regs.bpcm, cpu.regs.bda, cpu.regs.bdam) = (ORIGIN + 4 * g.below(400), !0, 0x8004_0000 + 4 * g.below(0x80), !3);
        cpu.regs.sr = if seed % 3 == 0 { interrupts_on } else { sr::CU2 };
        let (mut twin, mut twin_rig) = (cpu.clone(), Rig { ram: rig.ram.clone(), ..Rig::new() });
        (twin_rig.absent, twin_rig.line_bit) = (rig.absent, rig.line_bit);
        let mut steps = 0;
        while steps < 4000 {
            let (limit, until) = (1 + g.below(300), rig.clock + 1 + g.below(400) as u64);
            (rig.until, twin_rig.until) = (until, until);
            let ran = cpu.run(&mut rig, limit);
            let made = stepped(&mut twin, &mut twin_rig, limit);
            assert_eq!(ran, made, "seed {seed} after {steps} steps: the run's count and end against the steps'");
            assert_eq!((cpu.regs, cpu.hilo_ready), (twin.regs, twin.hilo_ready), "seed {seed} after {steps} steps");
            assert!(rig.clock == twin_rig.clock && rig.idled == twin_rig.idled && rig.writes == twin_rig.writes && rig.isolated == twin_rig.isolated && rig.commands == twin_rig.commands && rig.cop2 == twin_rig.cop2, "seed {seed} after {steps} steps: the bus");
            steps += ran.0;
            runs += 1;
            longest = longest.max(ran.0);
            if let Step::Exception(c) = ran.1 {
                exceptions |= 1 << c;
                interrupts += (c == 0) as u32;
            }
            (rig.writes.clear(), twin_rig.writes.clear(), rig.isolated.clear(), twin_rig.isolated.clear());
        }
        assert!(rig.ram == twin_rig.ram, "seed {seed}: memory");
    }
    eprintln!("{runs} runs, the longest {longest} steps; exception codes {exceptions:#b}, {interrupts} interrupts");
    assert!(longest >= 100 && interrupts > 100, "the programs run and are interrupted: {longest}, {interrupts}");
    for c in [code::INT, code::ADEL, code::ADES, code::DBE, code::SYS, code::BP, code::RI, code::CPU, code::OV] {
        assert!(exceptions & 1 << c != 0, "exception {c} was met");
    }
}
