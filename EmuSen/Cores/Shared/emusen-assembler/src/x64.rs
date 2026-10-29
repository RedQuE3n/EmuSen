//! The x86-64 backend (EmuSen_Recompiler.md §3.3, §14.2). See EmuSen_Recompiler.md §14.

use cranelift_assembler_x64 as asm;
use asm::{Amode, AmodeOffset, AmodeOffsetPlusKnownOffset, DeferredTarget, GprMem, Inst, inst as i};

use crate::{Assembler, By, Code, Cond, Label, Mem, Op, PURPOSES, Reg, Reloc, Section, ShiftOp, Src, Target, Width};

pub const RAX: Reg = Reg(0);
pub const RCX: Reg = Reg(1);
pub const RDX: Reg = Reg(2);
pub const RBX: Reg = Reg(3);
pub const RSP: Reg = Reg(4);
pub const RBP: Reg = Reg(5);
pub const RSI: Reg = Reg(6);
pub const RDI: Reg = Reg(7);
pub const R8: Reg = Reg(8);
pub const R9: Reg = Reg(9);
pub const R10: Reg = Reg(10);
pub const R11: Reg = Reg(11);
pub const R12: Reg = Reg(12);
pub const R13: Reg = Reg(13);
pub const R14: Reg = Reg(14);
pub const R15: Reg = Reg(15);

/// Every general register but rsp.
pub const ALL: [Reg; 15] = [RAX, RCX, RDX, RBX, RBP, RSI, RDI, R8, R9, R10, R11, R12, R13, R14, R15];

/// The helper convention, version 1 (EmuSen_Recompiler.md §3.5). See EmuSen_Recompiler.md §14.
pub const HELPER_ARGS: [Reg; 3] = [RDI, RSI, RDX];

/// The trampoline that calls `helper`, an `extern "C" fn(context, a, b, c) -> u64`, in the helper convention; entered by a call from code whose stack is 16-byte aligned.
pub fn trampoline(a: &mut X64, context: Reg, helper: u64) {
    const SAVED: [Reg; 8] = [RCX, RDX, RSI, RDI, R8, R9, R10, R11];
    for x in SAVED {
        a.push(x);
    }
    a.alu(Op::Sub, Width::W64, RSP, RSP, Src::Imm(8));
    a.mov(Width::W64, RCX, RDX);
    a.mov(Width::W64, RDX, RSI);
    a.mov(Width::W64, RSI, RDI);
    a.mov(Width::W64, RDI, context);
    a.mov_imm(RAX, helper as i64);
    a.call_reg(RAX);
    a.alu(Op::Add, Width::W64, RSP, RSP, Src::Imm(8));
    for x in SAVED.iter().rev() {
        a.pop(*x);
    }
    a.ret();
}

#[derive(Clone, Copy, Debug)]
pub struct Regs;
impl asm::Registers for Regs {
    type ReadGpr = u8;
    type ReadWriteGpr = u8;
    type WriteGpr = u8;
    type ReadXmm = u8;
    type ReadWriteXmm = u8;
    type WriteXmm = u8;
}

/// A code sink that notes where a rip-relative field begins.
#[derive(Default)]
struct Sink {
    bytes: Vec<u8>,
    field: Option<usize>,
}

impl asm::CodeSink for Sink {
    fn put1(&mut self, v: u8) {
        self.bytes.push(v);
    }
    fn put2(&mut self, v: u16) {
        self.bytes.extend_from_slice(&v.to_le_bytes());
    }
    fn put4(&mut self, v: u32) {
        self.bytes.extend_from_slice(&v.to_le_bytes());
    }
    fn put8(&mut self, v: u64) {
        self.bytes.extend_from_slice(&v.to_le_bytes());
    }
    fn add_trap(&mut self, _: asm::TrapCode) {}
    fn use_target(&mut self, _: DeferredTarget) {
        self.field = Some(self.bytes.len());
    }
    fn known_offset(&self, o: asm::KnownOffset) -> i32 {
        panic!("no known offset {o}")
    }
}

fn mem(m: Mem) -> Amode<u8> {
    Amode::ImmReg { base: m.base.0, simm32: AmodeOffsetPlusKnownOffset { simm32: AmodeOffset::new(m.disp), offset: None }, trap: None }
}

fn rip() -> Amode<u8> {
    Amode::RipRelative { target: DeferredTarget::None }
}

fn gm(m: Mem) -> GprMem<u8, u8> {
    GprMem::Mem(mem(m))
}

fn r(x: Reg) -> u8 {
    x.0
}

/// The condition's code in a Jcc, SETcc or CMOVcc.
fn cc(c: Cond) -> u8 {
    match c {
        Cond::Overflow => 0x0,
        Cond::Ltu => 0x2,
        Cond::Geu => 0x3,
        Cond::Eq => 0x4,
        Cond::Ne => 0x5,
        Cond::Leu => 0x6,
        Cond::Gtu => 0x7,
        Cond::Lt => 0xC,
        Cond::Ge => 0xD,
        Cond::Le => 0xE,
        Cond::Gt => 0xF,
    }
}

/// What a field in a run of bytes reaches: a target outside the block's sections, or a label.
#[derive(Clone, Copy)]
enum Fix {
    Out(Target),
    At(Label),
}

enum Item {
    Bytes { purpose: usize, bytes: Vec<u8>, relocs: Vec<(u32, Fix)> },
    Jump { cond: Option<Cond>, label: Label, purpose: usize, long: bool },
    Bind,
}

#[derive(Default)]
struct Lane {
    items: Vec<Item>,
    starts: Vec<u32>,
    size: u32,
}

/// The x86-64 macro-assembler.
pub struct X64 {
    lanes: [Lane; 2],
    now: Section,
    purpose: usize,
    labels: Vec<Option<(Section, usize)>>,
    data: Vec<u8>,
}

impl Default for X64 {
    fn default() -> Self {
        X64::new()
    }
}

fn lane_of(s: Section) -> usize {
    match s {
        Section::Hot => 0,
        Section::Cold => 1,
    }
}

impl X64 {
    pub fn new() -> X64 {
        X64 { lanes: [Lane::default(), Lane::default()], now: Section::Hot, purpose: 0, labels: Vec::new(), data: Vec::new() }
    }

    fn lane(&mut self) -> &mut Lane {
        &mut self.lanes[lane_of(self.now)]
    }

    fn put_bytes(&mut self, bytes: &[u8], reloc: Option<(u32, Fix)>) {
        let purpose = self.purpose;
        let lane = self.lane();
        if !matches!(lane.items.last(), Some(Item::Bytes { purpose: p, .. }) if *p == purpose) {
            lane.items.push(Item::Bytes { purpose, bytes: Vec::new(), relocs: Vec::new() });
        }
        let Some(Item::Bytes { bytes: b, relocs, .. }) = lane.items.last_mut() else { unreachable!("a run of bytes is last") };
        if let Some((at, t)) = reloc {
            relocs.push((b.len() as u32 + at, t));
        }
        b.extend_from_slice(bytes);
    }

    fn put(&mut self, inst: impl Into<Inst<Regs>>) {
        let mut s = Sink::default();
        inst.into().encode(&mut s);
        debug_assert!(s.field.is_none(), "a rip-relative operand goes through `put_rip`");
        self.put_bytes(&s.bytes, None);
    }

    /// An instruction with a rip-relative operand reaching `t`; the field must end the instruction.
    fn put_rip(&mut self, inst: impl Into<Inst<Regs>>, t: Fix) {
        let mut s = Sink::default();
        inst.into().encode(&mut s);
        let at = s.field.expect("a rip-relative operand");
        assert_eq!(at + 4, s.bytes.len(), "the field ends the instruction");
        self.put_bytes(&s.bytes, Some((at as u32, t)));
    }

    /// The host's own sequences, for stubs and trampolines: no guest's lowering calls them.
    pub fn push(&mut self, x: Reg) {
        self.put(i::pushq_o::new(r(x)));
    }

    pub fn pop(&mut self, x: Reg) {
        self.put(i::popq_o::new(r(x)));
    }

    pub fn ret(&mut self) {
        self.put(i::retq_zo::new());
    }

    pub fn call_reg(&mut self, x: Reg) {
        self.put(i::callq_m::new(r(x)));
    }

    pub fn jump_reg(&mut self, x: Reg) {
        self.put(i::jmpq_m::new(r(x)));
    }

    fn sizes(&mut self) {
        for lane in &mut self.lanes {
            lane.starts.clear();
            let mut at = 0u32;
            for item in &lane.items {
                lane.starts.push(at);
                at += match item {
                    Item::Bytes { bytes, .. } => bytes.len() as u32,
                    Item::Jump { long: false, .. } => 2,
                    Item::Jump { cond: None, .. } => 5,
                    Item::Jump { .. } => 6,
                    Item::Bind => 0,
                };
            }
            lane.size = at;
        }
    }

    fn place(&self, l: Label) -> (Section, u32) {
        let (s, k) = self.labels[l.0 as usize].expect("every label used is bound");
        (s, self.lanes[lane_of(s)].starts[k])
    }

    /// Jumps grown to rel32 until every short one reaches.
    fn relax(&mut self) {
        loop {
            self.sizes();
            let mut grow = Vec::new();
            for (n, lane) in self.lanes.iter().enumerate() {
                for (k, item) in lane.items.iter().enumerate() {
                    if let Item::Jump { label, long: false, .. } = item {
                        let (s, to) = self.place(*label);
                        let from = i64::from(lane.starts[k]) + 2;
                        if lane_of(s) != n || !(-128..=127).contains(&(i64::from(to) - from)) {
                            grow.push((n, k));
                        }
                    }
                }
            }
            if grow.is_empty() {
                return;
            }
            for (n, k) in grow {
                if let Item::Jump { long, .. } = &mut self.lanes[n].items[k] {
                    *long = true;
                }
            }
        }
    }
}

impl Assembler for X64 {
    fn label(&mut self) -> Label {
        self.labels.push(None);
        Label(self.labels.len() as u32 - 1)
    }

    fn bind(&mut self, l: Label) {
        let s = self.now;
        let lane = self.lane();
        lane.items.push(Item::Bind);
        let k = lane.items.len() - 1;
        assert!(self.labels[l.0 as usize].is_none(), "a label is bound once");
        self.labels[l.0 as usize] = Some((s, k));
    }

    fn section(&mut self, s: Section) {
        self.now = s;
    }

    fn current(&self) -> Section {
        self.now
    }

    fn purpose(&mut self, p: usize) {
        assert!(p < PURPOSES, "a purpose below PURPOSES");
        self.purpose = p;
    }

    fn mov(&mut self, w: Width, d: Reg, s: Reg) {
        match w {
            Width::W64 if d != s => self.put(i::movq_mr::new(r(d), r(s))),
            Width::W32 => self.put(i::movl_mr::new(r(d), r(s))),
            _ => {}
        }
    }

    fn mov_imm(&mut self, d: Reg, v: i64) {
        if (0..=i64::from(u32::MAX)).contains(&v) {
            self.put(i::movl_oi::new(r(d), v as u32));
        } else if let Ok(v32) = i32::try_from(v) {
            self.put(i::movq_mi_sxl::new(r(d), v32));
        } else {
            self.put(i::movabsq_oi::new(r(d), v as u64));
        }
    }

    fn alu(&mut self, op: Op, w: Width, d: Reg, a: Reg, b: Src) {
        let commutes = op != Op::Sub;
        let b = match b {
            Src::Reg(x) if x == d && d != a => {
                if commutes {
                    return self.alu(op, w, d, d, Src::Reg(a));
                }
                // d = a - d.
                self.neg(w, d, d);
                return self.alu(Op::Add, w, d, d, Src::Reg(a));
            }
            b => b,
        };
        if d != a {
            self.mov(w, d, a);
        }
        let d = r(d);
        macro_rules! go {
            ($q_rm:ident, $l_rm:ident, $q_sxb:ident, $q_sxl:ident, $l_sxb:ident, $l_i:ident) => {
                match (w, b) {
                    (Width::W64, Src::Reg(x)) => self.put(i::$q_rm::new(d, r(x))),
                    (Width::W32, Src::Reg(x)) => self.put(i::$l_rm::new(d, r(x))),
                    (Width::W64, Src::Imm(k)) if i8::try_from(k).is_ok() => self.put(i::$q_sxb::new(d, k as i8)),
                    (Width::W64, Src::Imm(k)) => self.put(i::$q_sxl::new(d, k)),
                    (Width::W32, Src::Imm(k)) if i8::try_from(k).is_ok() => self.put(i::$l_sxb::new(d, k as i8)),
                    (Width::W32, Src::Imm(k)) => self.put(i::$l_i::new(d, k as u32)),
                }
            };
        }
        match op {
            Op::Add => go!(addq_rm, addl_rm, addq_mi_sxb, addq_mi_sxl, addl_mi_sxb, addl_mi),
            Op::Sub => go!(subq_rm, subl_rm, subq_mi_sxb, subq_mi_sxl, subl_mi_sxb, subl_mi),
            Op::And => go!(andq_rm, andl_rm, andq_mi_sxb, andq_mi_sxl, andl_mi_sxb, andl_mi),
            Op::Or => go!(orq_rm, orl_rm, orq_mi_sxb, orq_mi_sxl, orl_mi_sxb, orl_mi),
            Op::Xor => go!(xorq_rm, xorl_rm, xorq_mi_sxb, xorq_mi_sxl, xorl_mi_sxb, xorl_mi),
        }
    }

    fn alu_mem(&mut self, op: Op, bytes: u8, m: Mem, b: Src) {
        macro_rules! go {
            ($b_mr:ident, $l_mr:ident, $q_mr:ident, $b_mi:ident, $l_mi:ident, $q_sxl:ident) => {
                match (bytes, b) {
                    (1, Src::Reg(x)) => self.put(i::$b_mr::new(gm(m), r(x))),
                    (4, Src::Reg(x)) => self.put(i::$l_mr::new(gm(m), r(x))),
                    (8, Src::Reg(x)) => self.put(i::$q_mr::new(gm(m), r(x))),
                    (1, Src::Imm(k)) => self.put(i::$b_mi::new(gm(m), k as u8)),
                    (4, Src::Imm(k)) => self.put(i::$l_mi::new(gm(m), k as u32)),
                    (8, Src::Imm(k)) => self.put(i::$q_sxl::new(gm(m), k)),
                    _ => panic!("alu_mem of {bytes} bytes"),
                }
            };
        }
        match op {
            Op::Add => go!(addb_mr, addl_mr, addq_mr, addb_mi, addl_mi, addq_mi_sxl),
            Op::Sub => go!(subb_mr, subl_mr, subq_mr, subb_mi, subl_mi, subq_mi_sxl),
            Op::And => go!(andb_mr, andl_mr, andq_mr, andb_mi, andl_mi, andq_mi_sxl),
            Op::Or => go!(orb_mr, orl_mr, orq_mr, orb_mi, orl_mi, orq_mi_sxl),
            Op::Xor => go!(xorb_mr, xorl_mr, xorq_mr, xorb_mi, xorl_mi, xorq_mi_sxl),
        }
    }

    fn shift(&mut self, op: ShiftOp, w: Width, d: Reg, a: Reg, by: By) {
        match by {
            By::Reg(n) => {
                let (d, a, n) = (r(d), r(a), r(n));
                match (op, w) {
                    (ShiftOp::Shl, Width::W64) => self.put(i::shlxq_rmv::new(d, a, n)),
                    (ShiftOp::Shr, Width::W64) => self.put(i::shrxq_rmv::new(d, a, n)),
                    (ShiftOp::Sar, Width::W64) => self.put(i::sarxq_rmv::new(d, a, n)),
                    (ShiftOp::Shl, Width::W32) => self.put(i::shlxl_rmv::new(d, a, n)),
                    (ShiftOp::Shr, Width::W32) => self.put(i::shrxl_rmv::new(d, a, n)),
                    (ShiftOp::Sar, Width::W32) => self.put(i::sarxl_rmv::new(d, a, n)),
                }
            }
            By::Imm(k) => {
                self.mov(w, d, a);
                if k == 0 {
                    self.mov(w, d, d);
                    return;
                }
                let d = r(d);
                match (op, w) {
                    (ShiftOp::Shl, Width::W64) => self.put(i::shlq_mi::new(d, k)),
                    (ShiftOp::Shr, Width::W64) => self.put(i::shrq_mi::new(d, k)),
                    (ShiftOp::Sar, Width::W64) => self.put(i::sarq_mi::new(d, k)),
                    (ShiftOp::Shl, Width::W32) => self.put(i::shll_mi::new(d, k)),
                    (ShiftOp::Shr, Width::W32) => self.put(i::shrl_mi::new(d, k)),
                    (ShiftOp::Sar, Width::W32) => self.put(i::sarl_mi::new(d, k)),
                }
            }
        }
    }

    fn not(&mut self, w: Width, d: Reg, a: Reg) {
        self.mov(w, d, a);
        match w {
            Width::W64 => self.put(i::notq_m::new(r(d))),
            Width::W32 => self.put(i::notl_m::new(r(d))),
        }
    }

    fn neg(&mut self, w: Width, d: Reg, a: Reg) {
        self.mov(w, d, a);
        match w {
            Width::W64 => self.put(i::negq_m::new(r(d))),
            Width::W32 => self.put(i::negl_m::new(r(d))),
        }
    }

    fn sext32(&mut self, d: Reg, s: Reg) {
        self.put(i::movslq_rm::new(r(d), r(s)));
    }

    fn cmp(&mut self, w: Width, a: Reg, b: Src) {
        let a = r(a);
        match (w, b) {
            (Width::W64, Src::Reg(x)) => self.put(i::cmpq_rm::new(a, r(x))),
            (Width::W32, Src::Reg(x)) => self.put(i::cmpl_rm::new(a, r(x))),
            (Width::W64, Src::Imm(k)) if i8::try_from(k).is_ok() => self.put(i::cmpq_mi_sxb::new(a, k as i8)),
            (Width::W64, Src::Imm(k)) => self.put(i::cmpq_mi::new(a, k)),
            (Width::W32, Src::Imm(k)) if i8::try_from(k).is_ok() => self.put(i::cmpl_mi_sxb::new(a, k as i8)),
            (Width::W32, Src::Imm(k)) => self.put(i::cmpl_mi::new(a, k as u32)),
        }
    }

    fn cmp_mem(&mut self, bytes: u8, m: Mem, b: Src) {
        match (bytes, b) {
            (1, Src::Reg(x)) => self.put(i::cmpb_mr::new(gm(m), r(x))),
            (4, Src::Reg(x)) => self.put(i::cmpl_mr::new(gm(m), r(x))),
            (8, Src::Reg(x)) => self.put(i::cmpq_mr::new(gm(m), r(x))),
            (1, Src::Imm(k)) => self.put(i::cmpb_mi::new(gm(m), k as u8)),
            (4, Src::Imm(k)) if i8::try_from(k).is_ok() => self.put(i::cmpl_mi_sxb::new(gm(m), k as i8)),
            (4, Src::Imm(k)) => self.put(i::cmpl_mi::new(gm(m), k as u32)),
            (8, Src::Imm(k)) if i8::try_from(k).is_ok() => self.put(i::cmpq_mi_sxb::new(gm(m), k as i8)),
            (8, Src::Imm(k)) => self.put(i::cmpq_mi::new(gm(m), k)),
            _ => panic!("cmp_mem of {bytes} bytes"),
        }
    }

    fn test(&mut self, w: Width, a: Reg, b: Src) {
        let a = r(a);
        match (w, b) {
            (Width::W64, Src::Reg(x)) => self.put(i::testq_mr::new(a, r(x))),
            (Width::W32, Src::Reg(x)) => self.put(i::testl_mr::new(a, r(x))),
            (Width::W64, Src::Imm(k)) => self.put(i::testq_mi::new(a, k)),
            (Width::W32, Src::Imm(k)) => self.put(i::testl_mi::new(a, k as u32)),
        }
    }

    fn set(&mut self, c: Cond, d: Reg) {
        let x = r(d);
        match c {
            Cond::Overflow => self.put(i::seto_m::new(x)),
            Cond::Ltu => self.put(i::setb_m::new(x)),
            Cond::Geu => self.put(i::setae_m::new(x)),
            Cond::Eq => self.put(i::sete_m::new(x)),
            Cond::Ne => self.put(i::setne_m::new(x)),
            Cond::Leu => self.put(i::setbe_m::new(x)),
            Cond::Gtu => self.put(i::seta_m::new(x)),
            Cond::Lt => self.put(i::setl_m::new(x)),
            Cond::Ge => self.put(i::setge_m::new(x)),
            Cond::Le => self.put(i::setle_m::new(x)),
            Cond::Gt => self.put(i::setg_m::new(x)),
        }
        self.put(i::movzbl_rm::new(x, x));
    }

    fn select(&mut self, c: Cond, d: Reg, s: Reg) {
        let (d, s) = (r(d), r(s));
        match c {
            Cond::Overflow => self.put(i::cmovoq_rm::new(d, s)),
            Cond::Ltu => self.put(i::cmovbq_rm::new(d, s)),
            Cond::Geu => self.put(i::cmovaeq_rm::new(d, s)),
            Cond::Eq => self.put(i::cmoveq_rm::new(d, s)),
            Cond::Ne => self.put(i::cmovneq_rm::new(d, s)),
            Cond::Leu => self.put(i::cmovbeq_rm::new(d, s)),
            Cond::Gtu => self.put(i::cmovaq_rm::new(d, s)),
            Cond::Lt => self.put(i::cmovlq_rm::new(d, s)),
            Cond::Ge => self.put(i::cmovgeq_rm::new(d, s)),
            Cond::Le => self.put(i::cmovleq_rm::new(d, s)),
            Cond::Gt => self.put(i::cmovgq_rm::new(d, s)),
        }
    }

    fn load(&mut self, bytes: u8, signed: bool, d: Reg, m: Mem) {
        let (d, m) = (r(d), gm(m));
        match (bytes, signed) {
            (1, true) => self.put(i::movsbq_rm::new(d, m)),
            (1, false) => self.put(i::movzbl_rm::new(d, m)),
            (2, true) => self.put(i::movswq_rm::new(d, m)),
            (2, false) => self.put(i::movzwl_rm::new(d, m)),
            (4, true) => self.put(i::movslq_rm::new(d, m)),
            (4, false) => self.put(i::movl_rm::new(d, m)),
            (8, _) => self.put(i::movq_rm::new(d, m)),
            _ => panic!("a load of {bytes} bytes"),
        }
    }

    fn store(&mut self, bytes: u8, s: Reg, m: Mem) {
        let (s, m) = (r(s), gm(m));
        match bytes {
            1 => self.put(i::movb_mr::new(m, s)),
            2 => self.put(i::movw_mr::new(m, s)),
            4 => self.put(i::movl_mr::new(m, s)),
            8 => self.put(i::movq_mr::new(m, s)),
            _ => panic!("a store of {bytes} bytes"),
        }
    }

    fn store_imm(&mut self, bytes: u8, v: i32, m: Mem) {
        let m = gm(m);
        match bytes {
            1 => self.put(i::movb_mi::new(m, v as u8)),
            2 => self.put(i::movw_mi::new(m, v as u16)),
            4 => self.put(i::movl_mi::new(m, v as u32)),
            8 => self.put(i::movq_mi_sxl::new(m, v)),
            _ => panic!("a store of {bytes} bytes"),
        }
    }

    fn lea(&mut self, w: Width, d: Reg, m: Mem) {
        match w {
            Width::W64 => self.put(i::leaq_rm::new(r(d), mem(m))),
            Width::W32 => self.put(i::leal_rm::new(r(d), mem(m))),
        }
    }

    fn jump(&mut self, l: Label) {
        let purpose = self.purpose;
        self.lane().items.push(Item::Jump { cond: None, label: l, purpose, long: false });
    }

    fn branch(&mut self, c: Cond, l: Label) {
        let purpose = self.purpose;
        self.lane().items.push(Item::Jump { cond: Some(c), label: l, purpose, long: false });
    }

    fn jump_stub(&mut self, id: u32) {
        self.put_bytes(&[0xE9, 0, 0, 0, 0], Some((1, Fix::Out(Target::Stub(id)))));
    }

    fn call_stub(&mut self, id: u32) {
        self.put_bytes(&[0xE8, 0, 0, 0, 0], Some((1, Fix::Out(Target::Stub(id)))));
    }

    fn lea_label(&mut self, d: Reg, l: Label) {
        self.put_rip(i::leaq_rm::new(r(d), rip()), Fix::At(l));
    }

    fn lea_data(&mut self, d: Reg, offset: u32) {
        self.put_rip(i::leaq_rm::new(r(d), rip()), Fix::Out(Target::Data(offset)));
    }

    fn jump_via_data(&mut self, offset: u32) {
        self.put_rip(i::jmpq_m::new(GprMem::Mem(rip())), Fix::Out(Target::Data(offset)));
    }

    fn data(&mut self, bytes: &[u8]) -> u32 {
        while self.data.len() % 8 != 0 {
            self.data.push(0);
        }
        let at = self.data.len() as u32;
        self.data.extend_from_slice(bytes);
        at
    }

    fn finish(mut self, entry: Label) -> Code {
        self.relax();
        let (es, eo) = self.place(entry);
        let mut code = Code { entry: eo, entry_cold: es == Section::Cold, data: std::mem::take(&mut self.data), ..Code::default() };
        for n in 0..2 {
            let section = if n == 0 { Section::Hot } else { Section::Cold };
            let mut out = Vec::with_capacity(self.lanes[n].size as usize);
            for (k, item) in self.lanes[n].items.iter().enumerate() {
                let start = self.lanes[n].starts[k];
                let purpose = match item {
                    Item::Bytes { bytes, relocs, purpose } => {
                        let first = out.len();
                        out.extend_from_slice(bytes);
                        for &(at, f) in relocs {
                            let field = start + at;
                            match f {
                                Fix::Out(t) => code.relocs.push(Reloc { section, at: field, target: t }),
                                Fix::At(l) => {
                                    let (s, to) = self.place(l);
                                    if s == section {
                                        let d = (i64::from(to) - i64::from(field) - 4) as i32;
                                        let k = first + at as usize;
                                        out[k..k + 4].copy_from_slice(&d.to_le_bytes());
                                    } else {
                                        let t = if s == Section::Hot { Target::Hot(to) } else { Target::Cold(to) };
                                        code.relocs.push(Reloc { section, at: field, target: t });
                                    }
                                }
                            }
                        }
                        *purpose
                    }
                    Item::Jump { cond, label, long, purpose } => {
                        let (s, to) = self.place(*label);
                        let len = match (long, cond) {
                            (false, _) => 2,
                            (true, None) => 5,
                            (true, Some(_)) => 6,
                        };
                        match (long, cond) {
                            (false, None) => out.push(0xEB),
                            (false, Some(c)) => out.push(0x70 | cc(*c)),
                            (true, None) => out.push(0xE9),
                            (true, Some(c)) => out.extend_from_slice(&[0x0F, 0x80 | cc(*c)]),
                        }
                        let end = i64::from(start) + len;
                        if *long {
                            if lane_of(s) == n {
                                out.extend_from_slice(&((i64::from(to) - end) as i32).to_le_bytes());
                            } else {
                                let t = if s == Section::Hot { Target::Hot(to) } else { Target::Cold(to) };
                                code.relocs.push(Reloc { section, at: start + len as u32 - 4, target: t });
                                out.extend_from_slice(&[0; 4]);
                            }
                        } else {
                            out.push((i64::from(to) - end) as i8 as u8);
                        }
                        *purpose
                    }
                    Item::Bind => continue,
                };
                if n == 0 {
                    code.hot_by_purpose[purpose] += out.len() as u32 - start;
                }
            }
            if n == 0 {
                code.hot = out;
            } else {
                code.cold = out;
            }
        }
        code
    }
}
