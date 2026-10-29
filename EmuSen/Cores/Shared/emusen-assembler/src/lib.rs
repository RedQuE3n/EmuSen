//! The tailored recompiler's macro-assembler (EmuSen_Recompiler.md §3.3). See EmuSen_Recompiler.md §14.

pub mod x64;

/// The `Assembler` trait's version: (major, minor).
pub const VERSION: (u16, u16) = (1, 1);

/// A host register, numbered as the backend numbers them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Reg(pub u8);

/// An operation's width: a 32-bit result is zero-extended into the register.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width {
    W32,
    W64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    And,
    Or,
    Xor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShiftOp {
    Shl,
    Shr,
    Sar,
}

/// A second operand: a register or an immediate sign-extended to the width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Src {
    Reg(Reg),
    Imm(i32),
}

/// A shift's count: a constant, or a register's low bits (5 at 32 bits, 6 at 64).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum By {
    Imm(u8),
    Reg(Reg),
}

/// A condition on the flags the last compare or test set, or an arithmetic operation's overflow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cond {
    Eq,
    Ne,
    Lt,
    Ge,
    Le,
    Gt,
    Ltu,
    Geu,
    Leu,
    Gtu,
    Overflow,
}

impl Cond {
    pub fn not(self) -> Option<Cond> {
        use Cond::*;
        Some(match self {
            Eq => Ne,
            Ne => Eq,
            Lt => Ge,
            Ge => Lt,
            Le => Gt,
            Gt => Le,
            Ltu => Geu,
            Geu => Ltu,
            Leu => Gtu,
            Gtu => Leu,
            Overflow => return None,
        })
    }
}

/// `[base + disp]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mem {
    pub base: Reg,
    pub disp: i32,
}

pub const fn at(base: Reg, disp: i32) -> Mem {
    Mem { base, disp }
}

/// Where code is laid: the hot arena, which a block's fast paths fill, or the cold one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Hot,
    Cold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Label(pub u32);

/// The purposes bytes may be counted under; the lowering names them.
pub const PURPOSES: usize = 16;

/// The macro-assembler a lowering writes a block through. See EmuSen_Recompiler.md §14.
pub trait Assembler {
    fn label(&mut self) -> Label;
    /// The label's place: here, in the current section.
    fn bind(&mut self, l: Label);
    fn section(&mut self, s: Section);
    fn current(&self) -> Section;
    /// The purpose the bytes from here on are counted under, below `PURPOSES`.
    fn purpose(&mut self, p: usize);

    fn mov(&mut self, w: Width, d: Reg, s: Reg);
    fn mov_imm(&mut self, d: Reg, v: i64);
    /// `d = a op b`.
    fn alu(&mut self, op: Op, w: Width, d: Reg, a: Reg, b: Src);
    /// `[m] op= b`, `bytes` 1, 4 or 8.
    fn alu_mem(&mut self, op: Op, bytes: u8, m: Mem, b: Src);
    fn shift(&mut self, op: ShiftOp, w: Width, d: Reg, a: Reg, by: By);
    fn not(&mut self, w: Width, d: Reg, a: Reg);
    fn neg(&mut self, w: Width, d: Reg, a: Reg);
    /// `d` the low 32 bits of `s` sign-extended.
    fn sext32(&mut self, d: Reg, s: Reg);
    /// `d = a * b`, the product's low 64 bits (1.1).
    fn mul(&mut self, d: Reg, a: Reg, b: Reg);
    /// `q = a / b` and `r = a % b` at `w`, with `b` nonzero, not the signed minimum over -1, and none of the backend's `DIV_REGS`, which it may clobber (1.1).
    fn divrem(&mut self, signed: bool, w: Width, q: Reg, r: Reg, a: Reg, b: Reg);
    fn cmp(&mut self, w: Width, a: Reg, b: Src);
    /// `bytes` 1, 4 or 8 of memory compared with `b`.
    fn cmp_mem(&mut self, bytes: u8, m: Mem, b: Src);
    fn test(&mut self, w: Width, a: Reg, b: Src);
    /// `d` 1 where `c` holds, else 0.
    fn set(&mut self, c: Cond, d: Reg);
    /// `d = s` where `c` holds.
    fn select(&mut self, c: Cond, d: Reg, s: Reg);
    /// `bytes` 1, 2, 4 or 8, sign- or zero-extended to 64 bits.
    fn load(&mut self, bytes: u8, signed: bool, d: Reg, m: Mem);
    fn store(&mut self, bytes: u8, s: Reg, m: Mem);
    fn store_imm(&mut self, bytes: u8, v: i32, m: Mem);
    fn lea(&mut self, w: Width, d: Reg, m: Mem);

    fn jump(&mut self, l: Label);
    fn branch(&mut self, c: Cond, l: Label);
    /// A jump or a call to shared stub `id`, which the runtime places once an arena.
    fn jump_stub(&mut self, id: u32);
    fn call_stub(&mut self, id: u32);
    /// `d` the address of a label.
    fn lea_label(&mut self, d: Reg, l: Label);
    /// `d` the address of the block's data at `offset`.
    fn lea_data(&mut self, d: Reg, offset: u32);
    /// A jump through the 8-byte word at the block's data `offset`.
    fn jump_via_data(&mut self, offset: u32);
    /// Bytes appended to the block's data, 8-aligned; their offset.
    fn data(&mut self, bytes: &[u8]) -> u32;
    /// The offset in its section a bound label will have once laid out, for `entry`.
    fn finish(self, entry: Label) -> Code;
}

/// What a relocation's 32-bit field, relative to the field's end, must reach.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Hot(u32),
    Cold(u32),
    Data(u32),
    Stub(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reloc {
    pub section: Section,
    /// The field's offset in its section.
    pub at: u32,
    pub target: Target,
}

/// A block's code: its sections' bytes, relocations, its entry, and its hot bytes by purpose.
#[derive(Clone, Debug, Default)]
pub struct Code {
    pub hot: Vec<u8>,
    pub cold: Vec<u8>,
    pub data: Vec<u8>,
    pub relocs: Vec<Reloc>,
    pub entry: u32,
    pub entry_cold: bool,
    pub hot_by_purpose: [u32; PURPOSES],
}

impl Code {
    /// The relocations applied for sections placed at `hot`, `cold` and `data`, stub `id` at `stub(id)`; false when a field cannot reach its target.
    pub fn link(&mut self, hot: u64, cold: u64, data: u64, stub: &dyn Fn(u32) -> u64) -> bool {
        for r in self.relocs.clone() {
            let (base, bytes) = match r.section {
                Section::Hot => (hot, &mut self.hot),
                Section::Cold => (cold, &mut self.cold),
            };
            let to = match r.target {
                Target::Hot(o) => hot + u64::from(o),
                Target::Cold(o) => cold + u64::from(o),
                Target::Data(o) => data + u64::from(o),
                Target::Stub(id) => stub(id),
            };
            let end = base + u64::from(r.at) + 4;
            let Ok(d) = i32::try_from(to.wrapping_sub(end) as i64) else { return false };
            bytes[r.at as usize..r.at as usize + 4].copy_from_slice(&d.to_le_bytes());
        }
        true
    }
}
