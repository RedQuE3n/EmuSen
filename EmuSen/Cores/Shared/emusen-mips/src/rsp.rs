//! The N64's signal processor as it runs: the scalar unit's MIPS subset and its vector unit on coprocessor 2, written in
//! the VR4300's spelling with the SGI names for the vector unit (`Mars_Disassembler.md` §5). A pure decode of one word;
//! every address it computes is an offset into IMEM (a branch or jump, wrapped at 4 KiB) or DMEM (a load or store from
//! register zero). Moved from the C# `RspDisassembler`, which it matches word for word (EmuSen_Mips.md §7).

use crate::vr4300::{GPR, Instruction, Reference, hex, word};

const IMEM_MASK: u32 = 0xFFC;
const DMEM_MASK: u32 = 0xFFF;

/// The libultra names of the sixteen registers `mfc0` and `mtc0` reach, by the four bits the processor reads.
pub const COP0: [&str; 16] = [
    "SP_MEM_ADDR", "SP_DRAM_ADDR", "SP_RD_LEN", "SP_WR_LEN", "SP_STATUS", "SP_DMA_FULL", "SP_DMA_BUSY", "SP_SEMAPHORE", "DPC_START", "DPC_END",
    "DPC_CURRENT", "DPC_STATUS", "DPC_CLOCK", "DPC_BUFBUSY", "DPC_PIPEBUSY", "DPC_TMEM",
];

/// The vector unit's control registers by the two bits read: 2 and 3 both reach VCE (`Mars_RspVector.md` §3).
const CONTROL: [&str; 4] = ["vco", "vcc", "vce", "vce"];

/// The vector operations by function; none where the references agree on no name (`Mars_Disassembler.md` §7.2).
const VECTOR: [Option<&str>; 64] = [
    Some("vmulf"), Some("vmulu"), Some("vrndp"), Some("vmulq"), Some("vmudl"), Some("vmudm"), Some("vmudn"), Some("vmudh"),
    Some("vmacf"), Some("vmacu"), Some("vrndn"), Some("vmacq"), Some("vmadl"), Some("vmadm"), Some("vmadn"), Some("vmadh"),
    Some("vadd"), Some("vsub"), Some("vsut"), Some("vabs"), Some("vaddc"), Some("vsubc"), Some("vaddb"), Some("vsubb"),
    Some("vaccb"), Some("vsucb"), Some("vsad"), Some("vsac"), Some("vsum"), Some("vsar"), None, None,
    Some("vlt"), Some("veq"), Some("vne"), Some("vge"), Some("vcl"), Some("vch"), Some("vcr"), Some("vmrg"),
    Some("vand"), Some("vnand"), Some("vor"), Some("vnor"), Some("vxor"), Some("vnxor"), None, None,
    Some("vrcp"), Some("vrcpl"), Some("vrcph"), Some("vmov"), Some("vrsq"), Some("vrsql"), Some("vrsqh"), Some("vnop"),
    Some("vextt"), Some("vextq"), Some("vextn"), None, Some("vinst"), Some("vinsq"), Some("vinsn"), Some("vnull"),
];

/// The vector loads and stores by format; the load of format 10 does nothing and has no name (`Mars_RspVector.md` §4).
const LOADS: [Option<&str>; 12] =
    [Some("lbv"), Some("lsv"), Some("llv"), Some("ldv"), Some("lqv"), Some("lrv"), Some("lpv"), Some("luv"), Some("lhv"), Some("lfv"), None, Some("ltv")];
const STORES: [Option<&str>; 12] =
    [Some("sbv"), Some("ssv"), Some("slv"), Some("sdv"), Some("sqv"), Some("srv"), Some("spv"), Some("suv"), Some("shv"), Some("sfv"), Some("swv"), Some("stv")];

/// How far each format shifts its seven-bit offset (`Mars_Disassembler.md` §5.3).
const OFFSET_SCALE: [u32; 12] = [0, 1, 2, 3, 4, 4, 3, 3, 4, 4, 4, 4];

fn rs(w: u32) -> usize {
    ((w >> 21) & 0x1F) as usize
}
fn rt(w: u32) -> usize {
    ((w >> 16) & 0x1F) as usize
}
fn rd(w: u32) -> usize {
    ((w >> 11) & 0x1F) as usize
}
fn sa(w: u32) -> usize {
    ((w >> 6) & 0x1F) as usize
}
fn simm(w: u32) -> i64 {
    w as u16 as i16 as i64
}

fn plain(mnemonic: &str, operands: impl Into<String>) -> Instruction {
    Instruction { mnemonic: mnemonic.to_string(), operands: operands.into(), reference: None }
}

/// A software code field is shown only when something was put in it.
fn code(c: u32) -> String {
    if c == 0 { String::new() } else { hex(c as i64) }
}

/// One RSP word at `address` in IMEM.
pub fn decode(w: u32, address: u32) -> Instruction {
    let (s, t) = (rs(w), rt(w));
    match w >> 26 {
        0x00 => special(w),
        0x01 => regimm(w, address),
        0x02 => jump("j", w),
        0x03 => jump("jal", w),
        0x04 if s == 0 && t == 0 => branch("b", "", w, address),
        0x04 if t == 0 => branch("beqz", &format!("{}, ", GPR[s]), w, address),
        0x04 => branch("beq", &format!("{}, {}, ", GPR[s], GPR[t]), w, address),
        0x05 if t == 0 => branch("bnez", &format!("{}, ", GPR[s]), w, address),
        0x05 => branch("bne", &format!("{}, {}, ", GPR[s], GPR[t]), w, address),
        0x06 => branch("blez", &format!("{}, ", GPR[s]), w, address),
        0x07 => branch("bgtz", &format!("{}, ", GPR[s]), w, address),
        0x08 => plain("addi", format!("{}, {}, {}", GPR[t], GPR[s], hex(simm(w)))),
        0x09 => plain("addiu", format!("{}, {}, {}", GPR[t], GPR[s], hex(simm(w)))),
        0x0A => plain("slti", format!("{}, {}, {}", GPR[t], GPR[s], hex(simm(w)))),
        0x0B => plain("sltiu", format!("{}, {}, {}", GPR[t], GPR[s], hex(simm(w)))),
        0x0C => plain("andi", format!("{}, {}, {}", GPR[t], GPR[s], hex((w & 0xFFFF) as i64))),
        0x0D => plain("ori", format!("{}, {}, {}", GPR[t], GPR[s], hex((w & 0xFFFF) as i64))),
        0x0E => plain("xori", format!("{}, {}, {}", GPR[t], GPR[s], hex((w & 0xFFFF) as i64))),
        0x0F => plain("lui", format!("{}, {}", GPR[t], hex((w & 0xFFFF) as i64))),
        0x10 if s == 0x00 => plain("mfc0", format!("{}, {}", GPR[t], COP0[rd(w) & 0xF])),
        0x10 if s == 0x04 => plain("mtc0", format!("{}, {}", GPR[t], COP0[rd(w) & 0xF])),
        0x12 => cop2(w),
        0x20 => scalar("lb", w, Reference::Read),
        0x21 => scalar("lh", w, Reference::Read),
        0x23 => scalar("lw", w, Reference::Read),
        0x24 => scalar("lbu", w, Reference::Read),
        0x25 => scalar("lhu", w, Reference::Read),
        0x27 => scalar("lwu", w, Reference::Read),
        0x28 => scalar("sb", w, Reference::Write),
        0x29 => scalar("sh", w, Reference::Write),
        0x2B => scalar("sw", w, Reference::Write),
        0x32 => vector(&LOADS, w, Reference::Read),
        0x3A => vector(&STORES, w, Reference::Write),
        _ => word(w),
    }
}

fn special(w: u32) -> Instruction {
    let (s, t, d) = (GPR[rs(w)], GPR[rt(w)], GPR[rd(w)]);
    let shift = || format!("{d}, {t}, {}", hex(sa(w) as i64));
    let variable = || format!("{d}, {t}, {s}");
    let three = || format!("{d}, {s}, {t}");
    match w & 0x3F {
        0x00 if w == 0 => plain("nop", ""),
        0x00 => plain("sll", shift()),
        0x02 => plain("srl", shift()),
        0x03 => plain("sra", shift()),
        0x04 => plain("sllv", variable()),
        0x06 => plain("srlv", variable()),
        0x07 => plain("srav", variable()),
        0x08 => plain("jr", s),
        0x09 => plain("jalr", if rd(w) == 31 { s.to_string() } else { format!("{d}, {s}") }),
        0x0D => plain("break", code((w >> 6) & 0xF_FFFF)),
        0x20 => plain("add", three()),
        0x21 if rt(w) == 0 => plain("move", format!("{d}, {s}")),
        0x21 => plain("addu", three()),
        0x22 if rs(w) == 0 => plain("neg", format!("{d}, {t}")),
        0x22 => plain("sub", three()),
        0x23 if rs(w) == 0 => plain("negu", format!("{d}, {t}")),
        0x23 => plain("subu", three()),
        0x24 => plain("and", three()),
        0x25 if rt(w) == 0 => plain("move", format!("{d}, {s}")),
        0x25 => plain("or", three()),
        0x26 => plain("xor", three()),
        0x27 if rt(w) == 0 => plain("not", format!("{d}, {s}")),
        0x27 => plain("nor", three()),
        0x2A => plain("slt", three()),
        0x2B => plain("sltu", three()),
        _ => word(w),
    }
}

/// Four selectors are named; the interpreter branches on the others too (`Mars_Disassembler.md` §3.1).
fn regimm(w: u32, address: u32) -> Instruction {
    let s = rs(w);
    match rt(w) {
        0x00 => branch("bltz", &format!("{}, ", GPR[s]), w, address),
        0x01 => branch("bgez", &format!("{}, ", GPR[s]), w, address),
        0x10 => branch("bltzal", &format!("{}, ", GPR[s]), w, address),
        0x11 if s == 0 => branch("bal", "", w, address),
        0x11 => branch("bgezal", &format!("{}, ", GPR[s]), w, address),
        _ => word(w),
    }
}

fn cop2(w: u32) -> Instruction {
    if w & (1 << 25) != 0 {
        return vector_operation(w);
    }
    let (t, d) = (GPR[rt(w)], rd(w));
    // The transfers' element is a byte index into the register, not a selector (`Mars_RspVector.md` §3).
    let element = format!("v{d}[{}]", (w >> 7) & 0xF);
    match rs(w) {
        0x00 => plain("mfc2", format!("{t}, {element}")),
        0x02 => plain("cfc2", format!("{t}, {}", CONTROL[d & 3])),
        0x04 => plain("mtc2", format!("{t}, {element}")),
        0x06 => plain("ctc2", format!("{t}, {}", CONTROL[d & 3])),
        _ => word(w),
    }
}

fn vector_operation(w: u32) -> Instruction {
    let funct = (w & 0x3F) as usize;
    let Some(name) = VECTOR[funct] else { return word(w) };
    let (vd, vs, vt) = (sa(w), rd(w), rt(w));
    let selected = format!("v{vt}{}", selector((w >> 21) & 0xF));
    match funct {
        0x37 | 0x3F => plain(name, ""),
        // One lane in, one lane out, and the destination lane lives in the vs field (`Mars_Disassembler.md` §5.2).
        0x30..=0x36 => plain(name, format!("v{vd}[{}], {selected}", vs & 7)),
        _ => plain(name, format!("v{vd}, v{vs}, {selected}")),
    }
}

/// The SGI spelling: nothing for the whole register, then quarters, halves and single elements (`Mars_Disassembler.md` §5.1).
fn selector(e: u32) -> String {
    match e {
        0 | 1 => String::new(),
        2 | 3 => format!("[{}q]", e - 2),
        4..=7 => format!("[{}h]", e - 4),
        _ => format!("[{}]", e - 8),
    }
}

fn vector(names: &[Option<&str>; 12], w: u32, kind: Reference) -> Instruction {
    let format = rd(w);
    let Some(name) = names.get(format).copied().flatten() else { return word(w) };
    let offset = (((w << 25) as i32) >> 25) << OFFSET_SCALE[format];
    let operands = format!("v{}[{}], {}({})", rt(w), (w >> 7) & 0xF, hex(offset as i64), GPR[rs(w)]);
    let reference = (rs(w) == 0).then_some((kind, offset as u32 & DMEM_MASK));
    Instruction { mnemonic: name.to_string(), operands, reference }
}

fn scalar(mnemonic: &str, w: u32, kind: Reference) -> Instruction {
    let operands = format!("{}, {}({})", GPR[rt(w)], hex(simm(w)), GPR[rs(w)]);
    let reference = (rs(w) == 0).then_some((kind, simm(w) as u32 & DMEM_MASK));
    Instruction { mnemonic: mnemonic.to_string(), operands, reference }
}

/// Counted from the delay slot and wrapped inside instruction memory (`Mars_Rsp.md` §2).
fn branch(mnemonic: &str, registers: &str, w: u32, address: u32) -> Instruction {
    let target = address.wrapping_add(4).wrapping_add(((w as u16 as i16 as i32) << 2) as u32) & IMEM_MASK;
    Instruction { mnemonic: mnemonic.to_string(), operands: format!("{registers}0x{target:03X}"), reference: Some((Reference::Call, target)) }
}

fn jump(mnemonic: &str, w: u32) -> Instruction {
    let target = (w << 2) & IMEM_MASK;
    Instruction { mnemonic: mnemonic.to_string(), operands: format!("0x{target:03X}"), reference: Some((Reference::Call, target)) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(w: u32, address: u32) -> String {
        let i = decode(w, address);
        format!("{} {}", i.mnemonic, i.operands).trim_end().to_string()
    }

    #[test]
    fn the_scalar_unit_reads_as_the_vr4300_s_spelling_with_imem_targets() {
        assert_eq!(text(0, 0), "nop");
        assert_eq!(text(0x1000_FFFF, 0x040), "b 0x040");
        assert_eq!(text(0x0800_0010, 0), "j 0x040");
        assert_eq!(text(0x4084_2000, 0), "mtc0 a0, SP_STATUS");
        assert_eq!(decode(0x8C01_0010, 0).reference, Some((Reference::Read, 0x010)));
        assert_eq!(decode(0x8C41_0010, 0).reference, None);
    }

    #[test]
    fn the_vector_unit_reads_with_the_sgi_names() {
        assert_eq!(text(0x4A00_0037, 0), "vnop");
        assert_eq!(text(0x4B01_1100 | 0x10, 0), "vadd v4, v2, v1[0]");
        assert_eq!(text(0xC801_2000, 0), "lqv v1[0], 0x0(zero)");
        assert_eq!(text(0xE801_207F, 0), "sqv v1[0], -0x10(zero)");
        assert_eq!(text(0xC801_5000, 0), ".word 0xC8015000");
    }
}
