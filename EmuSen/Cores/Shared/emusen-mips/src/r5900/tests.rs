//! The R5900's table against the TX79 manual's opcode tables and ps2tek's, each transcribed here cell for cell from
//! the documents (Toshiba, TX79 Core Architecture, rev. 2.0, A.5, B.5, C.2, D.4; ps2tek "EE Instruction Decoding"),
//! not from the table in `r5900.rs`. A cell is a name, a class the next table decodes, or reserved (`*`, `η`, `ϕ`).

use super::*;
use crate::decode::tests::mix;

/// A.5, the OpCode field. B.5, C.2 and D.4 print the same map again, MMI, LQ and SQ named in each.
const A5_PRIMARY: [&str; 64] = [
    "SPECIAL", "REGIMM", "J", "JAL", "BEQ", "BNE", "BLEZ", "BGTZ", //
    "ADDI", "ADDIU", "SLTI", "SLTIU", "ANDI", "ORI", "XORI", "LUI", //
    "COP0", "COP1", "*", "*", "BEQL", "BNEL", "BLEZL", "BGTZL", //
    "DADDI", "DADDIU", "LDL", "LDR", "MMI", "*", "LQ", "SQ", //
    "LB", "LH", "LWL", "LW", "LBU", "LHU", "LWR", "LWU", //
    "SB", "SH", "SWL", "SW", "SDL", "SDR", "SWR", "CACHE", //
    "η", "LWC1", "η", "PREF", "η", "LDC1", "η", "LD", //
    "η", "SWC1", "η", "*", "η", "SDC1", "η", "SD",
];

/// A.5, SPECIAL's function field.
const A5_SPECIAL: [&str; 64] = [
    "SLL", "*", "SRL", "SRA", "SLLV", "*", "SRLV", "SRAV", //
    "JR", "JALR", "MOVZ", "MOVN", "SYSCALL", "BREAK", "*", "SYNC", //
    "MFHI", "MTHI", "MFLO", "MTLO", "DSLLV", "*", "DSRLV", "DSRAV", //
    "MULT", "MULTU", "DIV", "DIVU", "η", "η", "η", "η", //
    "ADD", "ADDU", "SUB", "SUBU", "AND", "OR", "XOR", "NOR", //
    "MFSA", "MTSA", "SLT", "SLTU", "DADD", "DADDU", "DSUB", "DSUBU", //
    "TGE", "TGEU", "TLT", "TLTU", "TEQ", "*", "TNE", "*", //
    "DSLL", "*", "DSRL", "DSRA", "DSLL32", "*", "DSRL32", "DSRA32",
];

/// A.5, REGIMM's rt field.
const A5_REGIMM: [&str; 32] = [
    "BLTZ", "BGEZ", "BLTZL", "BGEZL", "*", "*", "*", "*", //
    "TGEI", "TGEIU", "TLTI", "TLTIU", "TEQI", "*", "TNEI", "*", //
    "BLTZAL", "BGEZAL", "BLTZALL", "BGEZALL", "*", "*", "*", "*", //
    "MTSAB", "MTSAH", "*", "*", "*", "*", "*", "*",
];

/// B.5, MMI's function field.
const B5_MMI: [&str; 64] = [
    "MADD", "MADDU", "*", "*", "PLZCW", "*", "*", "*", //
    "MMI0", "MMI2", "*", "*", "*", "*", "*", "*", //
    "MFHI1", "MTHI1", "MFLO1", "MTLO1", "*", "*", "*", "*", //
    "MULT1", "MULTU1", "DIV1", "DIVU1", "*", "*", "*", "*", //
    "MADD1", "MADDU1", "*", "*", "*", "*", "*", "*", //
    "MMI1", "MMI3", "*", "*", "*", "*", "*", "*", //
    "PMFHL", "PMTHL", "*", "*", "PSLLH", "*", "PSRLH", "PSRAH", //
    "*", "*", "*", "*", "PSLLW", "*", "PSRLW", "PSRAW",
];

/// B.5, MMI0 to MMI3 by bits 10-6, rows by bits 10-8.
const B5_MMI0: [&str; 32] = [
    "PADDW", "PSUBW", "PCGTW", "PMAXW", "PADDH", "PSUBH", "PCGTH", "PMAXH", "PADDB", "PSUBB", "PCGTB", "*", "*", "*", "*", "*", //
    "PADDSW", "PSUBSW", "PEXTLW", "PPACW", "PADDSH", "PSUBSH", "PEXTLH", "PPACH", "PADDSB", "PSUBSB", "PEXTLB", "PPACB", "*", "*", "PEXT5", "PPAC5",
];
const B5_MMI1: [&str; 32] = [
    "*", "PABSW", "PCEQW", "PMINW", "PADSBH", "PABSH", "PCEQH", "PMINH", "*", "*", "PCEQB", "*", "*", "*", "*", "*", //
    "PADDUW", "PSUBUW", "PEXTUW", "*", "PADDUH", "PSUBUH", "PEXTUH", "*", "PADDUB", "PSUBUB", "PEXTUB", "QFSRV", "*", "*", "*", "*",
];
const B5_MMI2: [&str; 32] = [
    "PMADDW", "*", "PSLLVW", "PSRLVW", "PMSUBW", "*", "*", "*", "PMFHI", "PMFLO", "PINTH", "*", "PMULTW", "PDIVW", "PCPYLD", "*", //
    "PMADDH", "PHMADH", "PAND", "PXOR", "PMSUBH", "PHMSBH", "*", "*", "*", "*", "PEXEH", "PREVH", "PMULTH", "PDIVBW", "PEXEW", "PROT3W",
];
const B5_MMI3: [&str; 32] = [
    "PMADDUW", "*", "*", "PSRAVW", "*", "*", "*", "*", "PMTHI", "PMTLO", "PINTEH", "*", "PMULTUW", "PDIVUW", "PCPYUD", "*", //
    "*", "*", "POR", "PNOR", "*", "*", "*", "*", "*", "*", "PEXCH", "PCPYH", "*", "*", "PEXCW", "*",
];

/// C.2, COP0's rs field.
const C2_COP0: [&str; 32] = [
    "MF0", "*", "*", "*", "MT0", "*", "*", "*", "BC0", "*", "*", "*", "*", "*", "*", "*", //
    "C0", "*", "*", "*", "*", "*", "*", "*", "*", "*", "*", "*", "*", "*", "*", "*",
];
/// C.2, MF0 and MT0 with rd = Debug (24), by bits 2-0; with rd = Perf (25), by bit 0.
const C2_DEBUG: [[&str; 8]; 2] =
    [["MFBPC", "ϕ", "MFIAB", "MFIABM", "MFDAB", "MFDABM", "MFDVB", "MFDVBM"], ["MTBPC", "ϕ", "MTIAB", "MTIABM", "MTDAB", "MTDABM", "MTDVB", "MTDVBM"]];
const C2_PERF: [[&str; 2]; 2] = [["MFPS", "MFPC"], ["MTPS", "MTPC"]];
const C2_BC0: [&str; 8] = ["BC0F", "BC0T", "BC0FL", "BC0TL", "*", "*", "*", "*"];
/// C.2, C0's function field.
const C2_C0: [&str; 64] = {
    let mut t = ["ϕ"; 64];
    t[0x01] = "TLBR";
    t[0x02] = "TLBWI";
    t[0x06] = "TLBWR";
    t[0x08] = "TLBP";
    t[0x18] = "ERET";
    t[0x38] = "EI";
    t[0x39] = "DI";
    t
};

/// D.4, COP1's rs field.
const D4_COP1: [&str; 32] = [
    "MFC1", "DMFC1", "CFC1", "*", "MTC1", "DMTC1", "CTC1", "*", "BC1", "*", "*", "*", "*", "*", "*", "*", //
    "S", "D", "ϕ", "ϕ", "W", "L", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ",
];
const D4_BC1: [&str; 8] = ["BC1F", "BC1T", "*", "*", "*", "*", "*", "*"];
/// D.4, the function field with rs = S or D, one table for both formats.
const D4_SD: [&str; 64] = [
    "ADD", "SUB", "MUL", "DIV", "SQRT", "ABS", "MOV", "NEG", //
    "ROUND.L", "TRUNC.L", "CEIL.L", "FLOOR.L", "ROUND.W", "TRUNC.W", "CEIL.W", "FLOOR.W", //
    "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", //
    "CVT.S", "CVT.D", "ϕ", "ϕ", "CVT.W", "CVT.L", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", "ϕ", //
    "C.F", "C.UN", "C.EQ", "C.UEQ", "C.OLT", "C.ULT", "C.OLE", "C.ULE", //
    "C.SF", "C.NGLE", "C.SEQ", "C.NGL", "C.LT", "C.NGE", "C.LE", "C.NGT",
];
/// D.4, the function field with rs = W or L.
const D4_WL: [&str; 64] = {
    let mut t = ["ϕ"; 64];
    t[0x20] = "CVT.S";
    t[0x21] = "CVT.D";
    t
};

/// ps2tek's tables where they differ from the TX79's: the OpCode field, COP1, BC1, FPU.S, FPU.W, COP2 and BC2.
const PS2TEK_PRIMARY_ROWS_6_7: [&str; 16] =
    ["---", "LWC1", "---", "PREF", "---", "---", "LQC2", "LD", "---", "SWC1", "---", "---", "---", "---", "SQC2", "SD"];
const PS2TEK_COP1: [&str; 32] = {
    let mut t = ["---"; 32];
    t[0] = "MFC1";
    t[2] = "CFC1";
    t[4] = "MTC1";
    t[6] = "CTC1";
    t
};
const PS2TEK_BC: [&str; 4] = ["F", "T", "FL", "TL"];
const PS2TEK_FPU_S: [&str; 64] = [
    "ADD.S", "SUB.S", "MUL.S", "DIV.S", "SQRT.S", "ABS.S", "MOV.S", "NEG.S", //
    "---", "---", "---", "---", "---", "---", "---", "---", //
    "---", "---", "---", "---", "---", "---", "RSQRT.S", "---", //
    "ADDA.S", "SUBA.S", "MULA.S", "---", "MADD.S", "MSUB.S", "MADDA.S", "MSUBA.S", //
    "---", "---", "---", "---", "CVT.W", "---", "---", "---", //
    "MAX.S", "MIN.S", "---", "---", "---", "---", "---", "---", //
    "C.F", "---", "C.EQ", "---", "C.LT", "---", "C.LE", "---", //
    "---", "---", "---", "---", "---", "---", "---", "---",
];
const PS2TEK_COP2: [&str; 16] =
    ["---", "QMFC2", "CFC2", "---", "---", "QMTC2", "CTC2", "---", "BC2", "---", "---", "---", "---", "---", "---", "---"];

/// The TX79's cells the EE reads otherwise, with what the EE's table reads (ps2tek) and the dispute that records it.
const DIFFERENCES: &[(&str, usize, Option<&str>, &str)] = &[
    ("primary", 0x12, Some("cop2 class"), "D-3"),
    ("primary", 0x35, Option::None, "D-1"),
    ("primary", 0x36, Some("lqc2"), "D-3"),
    ("primary", 0x3D, Option::None, "D-1"),
    ("primary", 0x3E, Some("sqc2"), "D-3"),
    ("cop1", 0x01, Option::None, "D-1"),
    ("cop1", 0x05, Option::None, "D-1"),
    ("cop1", 0x11, Option::None, "D-1"),
    ("cop1", 0x15, Option::None, "D-1"),
    ("bc1", 0x02, Some("bc1fl"), "D-2"),
    ("bc1", 0x03, Some("bc1tl"), "D-2"),
    ("s", 0x08, Option::None, "D-1"),
    ("s", 0x09, Option::None, "D-1"),
    ("s", 0x0A, Option::None, "D-1"),
    ("s", 0x0B, Option::None, "D-1"),
    ("s", 0x0C, Option::None, "D-1"),
    ("s", 0x0D, Option::None, "D-1"),
    ("s", 0x0E, Option::None, "D-1"),
    ("s", 0x0F, Option::None, "D-1"),
    ("s", 0x16, Some("rsqrt.s"), "D-1"),
    ("s", 0x18, Some("adda.s"), "D-1"),
    ("s", 0x19, Some("suba.s"), "D-1"),
    ("s", 0x1A, Some("mula.s"), "D-1"),
    ("s", 0x1C, Some("madd.s"), "D-1"),
    ("s", 0x1D, Some("msub.s"), "D-1"),
    ("s", 0x1E, Some("madda.s"), "D-1"),
    ("s", 0x1F, Some("msuba.s"), "D-1"),
    ("s", 0x21, Option::None, "D-1"),
    ("s", 0x25, Option::None, "D-1"),
    ("s", 0x28, Some("max.s"), "D-1"),
    ("s", 0x29, Some("min.s"), "D-1"),
    ("s", 0x31, Option::None, "D-1"),
    ("s", 0x33, Option::None, "D-1"),
    ("s", 0x34, Some("c.lt.s"), "D-1"),
    ("s", 0x35, Option::None, "D-1"),
    ("s", 0x36, Some("c.le.s"), "D-1"),
    ("s", 0x37, Option::None, "D-1"),
    ("s", 0x38, Option::None, "D-1"),
    ("s", 0x39, Option::None, "D-1"),
    ("s", 0x3A, Option::None, "D-1"),
    ("s", 0x3B, Option::None, "D-1"),
    ("s", 0x3C, Option::None, "D-1"),
    ("s", 0x3D, Option::None, "D-1"),
    ("s", 0x3E, Option::None, "D-1"),
    ("s", 0x3F, Option::None, "D-1"),
    ("w", 0x21, Option::None, "D-1"),
];

fn difference(table: &str, i: usize) -> Option<Option<&'static str>> {
    DIFFERENCES.iter().find(|d| d.0 == table && d.1 == i).map(|d| d.2)
}

fn reserved(cell: &str) -> bool {
    matches!(cell, "*" | "η" | "ϕ" | "---")
}

/// The bits a form's operands occupy; a COP0 move's rd keeps clear of Debug and Perf, which are other instructions.
pub(crate) fn operand_bits(op: Op, seed: &mut u64) -> u32 {
    let (rs, rt, rd, sa, imm) = (31 << 21, 31 << 16, 31 << 11, 31 << 6, 0xFFFF);
    let r = mix(seed);
    match op {
        Op::Vu0(o) => {
            let dest = 0xF << 21;
            r & match o.form {
                vu::Form::Raw => 0,
                vu::Form::TIs => dest | rt | 0xF << 11,
                vu::Form::FmacQ | vu::Form::FmacI => dest | rd | sa,
                vu::Form::Acc => dest | rt | rd,
                vu::Form::AccQ | vu::Form::AccI => dest | rd,
                vu::Form::Fmac => dest | rt | rd | sa,
            }
        }
        Op::Cpu(_, form) => {
            let bits = r & match form {
                None => 0,
                RdRsRt | RdRtRs => rd | rs | rt,
                RdRtSa => rd | rt | sa,
                RdRs | Jalr => rd | rs,
                RdRt => rd | rt,
                RsRt => rs | rt,
                Rd => rd,
                Rs => rs,
                Rt => rt,
                RtRsSimm | RtRsUimm | Branch2 | Mem(..) | Cache | Pref => rt | rs | imm,
                RtUimm => rt | imm,
                RsSimm | RsUimm | Branch1 => rs | imm,
                Branch0 => imm,
                Jump => 0x03FF_FFFF,
                Code => 0xF_FFFF << 6,
                Trap => rs | rt | 0x3FF << 6,
                Cop0 | RtFs | RtFcr | RtVf | RtVc => rt | rd,
                RtPerf => rt | 31 << 1,
                FdFsFt => sa | rd | rt,
                FdFs => sa | rd,
                FsFt => rd | rt,
            };
            if form == Cop0 && (bits >> 11 & 31) >> 1 == 12 { bits & !(1 << 14) } else { bits }
        }
    }
}

/// The word decodes to `op`, and with random operands its text assembles back to the same word and the same text.
fn reads_back(word: u32, op: Op, seed: &mut u64) {
    for _ in 0..64 {
        let pc = mix(seed) & !3;
        let w = word | operand_bits(op, seed);
        assert_eq!(operation(w), Some(op), "{w:08x}");
        let i = decode(w, pc);
        let text = if i.operands.is_empty() { i.mnemonic.clone() } else { format!("{} {}", i.mnemonic, i.operands) };
        let back = encode(&text, pc, &|_| Option::None).unwrap_or_else(|e| panic!("{w:08x} \"{text}\": {e}"));
        assert_eq!(back, w, "\"{text}\"");
    }
}

struct Tally {
    named: usize,
    reserved: usize,
    differences: usize,
}

/// One cell of the TX79's tables: checked as the manual reads it unless the EE's table differs there.
fn cell(table: &str, i: usize, cell: &str, expect: Option<String>, word: u32, seed: &mut u64, tally: &mut Tally) {
    if let Some(ee) = difference(table, i) {
        tally.differences += 1;
        match (ee, operation(word)) {
            (Option::None, got) => assert_eq!(got, Option::None, "{table}[{i:#x}] {cell}: the EE reserves it"),
            (Some(class), _) if class.ends_with(" class") => {}
            (Some(name), got) => {
                let got = got.unwrap_or_else(|| panic!("{table}[{i:#x}]: the EE has {name}"));
                assert_eq!(got.mnemonic(), name);
                reads_back(word, got, seed);
            }
        }
        return;
    }
    match expect {
        Option::None => {
            tally.reserved += 1;
            assert_eq!(operation(word), Option::None, "{table}[{i:#x}] {cell} at {word:08x}");
        }
        Some(want) => {
            tally.named += 1;
            let got = operation(word).unwrap_or_else(|| panic!("{table}[{i:#x}] {cell} at {word:08x} decodes to nothing"));
            let m = got.mnemonic();
            assert!(m == want || m.starts_with(&format!("{want}.")), "{table}[{i:#x}]: {m}, the manual {want}");
            reads_back(word, got, seed);
        }
    }
}

fn name(cell: &str) -> Option<String> {
    if reserved(cell) { Option::None } else { Some(cell.to_ascii_lowercase()) }
}

const CLASSES: [&str; 13] = ["SPECIAL", "REGIMM", "COP0", "COP1", "MMI", "MMI0", "MMI1", "MMI2", "MMI3", "BC0", "C0", "BC1", "S"];

/// The gate of `Sapphire_Plan.md` §7, S5: every encoding of the TX79 manual's opcode tables decodes to the manual's
/// instruction, and its disassembly, with operands at random, reassembles to the same word; a reserved cell decodes to
/// nothing; and each cell where the EE's table differs reads as ps2tek says, its dispute named.
#[test]
fn every_encoding_of_the_tx79_tables_decodes_disassembles_and_reassembles() {
    let mut seed = 0x7979_0001_2001_0400;
    let s = &mut seed;
    let mut t = Tally { named: 0, reserved: 0, differences: 0 };
    let is_class = |c: &str| CLASSES.contains(&c) || matches!(c, "D" | "W" | "L");
    for (i, c) in A5_PRIMARY.iter().enumerate() {
        if !is_class(c) {
            cell("primary", i, c, name(c), (i as u32) << 26, s, &mut t);
        }
    }
    for (i, c) in A5_SPECIAL.iter().enumerate() {
        cell("special", i, c, name(c), i as u32, s, &mut t);
    }
    for (i, c) in A5_REGIMM.iter().enumerate() {
        cell("regimm", i, c, name(c), 1 << 26 | (i as u32) << 16, s, &mut t);
    }
    for (i, c) in B5_MMI.iter().enumerate() {
        if !is_class(c) {
            cell("mmi", i, c, name(c), 0x1C << 26 | i as u32, s, &mut t);
        }
    }
    for (table, funct) in [(&B5_MMI0, 0x08), (&B5_MMI1, 0x28), (&B5_MMI2, 0x09), (&B5_MMI3, 0x29)] {
        for (i, c) in table.iter().enumerate() {
            cell("mmi0-3", i, c, name(c), 0x1C << 26 | (i as u32) << 6 | funct, s, &mut t);
        }
    }
    for (i, c) in C2_COP0.iter().enumerate() {
        let expect = match *c {
            "MF0" => Some("mfc0".into()),
            "MT0" => Some("mtc0".into()),
            _ => name(c),
        };
        if !is_class(c) {
            cell("cop0", i, c, expect, 0x10 << 26 | (i as u32) << 21, s, &mut t);
        }
    }
    for (m, rs) in [(0, 0u32), (1, 4)] {
        for (i, c) in C2_DEBUG[m].iter().enumerate() {
            cell("debug", i, c, name(c), 0x10 << 26 | rs << 21 | 24 << 11 | i as u32, s, &mut t);
        }
        for (i, c) in C2_PERF[m].iter().enumerate() {
            cell("perf", i, c, name(c), 0x10 << 26 | rs << 21 | 25 << 11 | i as u32, s, &mut t);
        }
    }
    for i in 0..32 {
        let c = C2_BC0.get(i).copied().unwrap_or("*");
        cell("bc0", i, c, name(c), 0x10 << 26 | 8 << 21 | (i as u32) << 16, s, &mut t);
    }
    for (i, c) in C2_C0.iter().enumerate() {
        cell("c0", i, c, name(c), 0x10 << 26 | 0x10 << 21 | i as u32, s, &mut t);
    }
    for (i, c) in D4_COP1.iter().enumerate() {
        if !is_class(c) || difference("cop1", i).is_some() {
            cell("cop1", i, c, name(c), 0x11 << 26 | (i as u32) << 21, s, &mut t);
        }
    }
    for i in 0..32 {
        let c = D4_BC1.get(i).copied().unwrap_or("*");
        cell("bc1", i, c, name(c), 0x11 << 26 | 8 << 21 | (i as u32) << 16, s, &mut t);
    }
    for (fmt, rs, table) in [("s", 0x10u32, &D4_SD), ("d", 0x11, &D4_SD), ("w", 0x14, &D4_WL), ("l", 0x15, &D4_WL)] {
        for (i, c) in table.iter().enumerate() {
            let itself = (fmt == "s" && *c == "CVT.S") || (fmt == "d" && *c == "CVT.D");
            let expect = if itself { Option::None } else { name(c).map(|n| format!("{n}.{fmt}")) };
            let word = 0x11 << 26 | rs << 21 | i as u32;
            if matches!(fmt, "d" | "l") {
                t.differences += 1;
                assert_eq!(operation(word), Option::None, "{fmt}[{i:#x}] {c}: the EE has no {fmt} format (D-1)");
            } else {
                cell(fmt, i, c, expect, word, s, &mut t);
            }
        }
    }
    assert_eq!((t.named, t.reserved, t.differences), (261, 371, 174), "named, reserved and different cells");
}

/// ps2tek's own tables where the EE differs from the TX79: every named cell decodes to ps2tek's name and reads back,
/// every empty one decodes to nothing, and COP2's operations are VU0's.
#[test]
fn the_ee_s_cells_by_ps2tek() {
    let mut seed = 0x5A99_0002;
    let s = &mut seed;
    let check = |word: u32, c: &str, want: Option<String>, s: &mut u64| match want {
        Option::None => assert_eq!(operation(word), Option::None, "{c} {word:08x}"),
        Some(want) => {
            let op = operation(word).unwrap_or_else(|| panic!("{c} {word:08x}"));
            assert_eq!(op.mnemonic(), want);
            reads_back(word, op, s);
        }
    };
    for (i, c) in PS2TEK_PRIMARY_ROWS_6_7.iter().enumerate() {
        check(((0x30 + i) as u32) << 26, c, name(c), s);
    }
    for (i, c) in PS2TEK_COP1.iter().enumerate() {
        if !matches!(i, 8 | 16 | 20) {
            check(0x11 << 26 | (i as u32) << 21, c, name(c), s);
        }
    }
    for (z, base) in [(1, 0x11u32), (2, 0x12)] {
        for i in 0..32u32 {
            let want = PS2TEK_BC.get(i as usize).map(|b| format!("bc{z}{}", b.to_ascii_lowercase()));
            check(base << 26 | 8 << 21 | i << 16, "BC", want, s);
        }
    }
    for (i, c) in PS2TEK_FPU_S.iter().enumerate() {
        let want = name(c).map(|n| match n.as_str() {
            "cvt.w" => "cvt.w.s".into(),
            n if n.starts_with("c.") => format!("{n}.s"),
            _ => n,
        });
        check(0x11 << 26 | 0x10 << 21 | i as u32, c, want, s);
    }
    for i in 0..64u32 {
        check(0x11 << 26 | 0x14 << 21 | i, "FPU.W", (i == 0x20).then(|| "cvt.s.w".into()), s);
    }
    for (i, c) in PS2TEK_COP2.iter().enumerate() {
        if *c != "BC2" {
            check(0x12 << 26 | (i as u32) << 21, c, name(c), s);
        }
    }
    let mut ops = 0;
    for rs in 0x10..0x20u32 {
        for bits in (0..64u32).chain((0..128).map(|i| (i / 4) << 6 | 0x3C | (i % 4))) {
            let w = 0x12 << 26 | rs << 21 | bits;
            match (operation(w), vu::operation(bits)) {
                (Some(Op::Vu0(a)), Some(b)) => {
                    assert_eq!(a, b);
                    reads_back(w, Op::Vu0(a), s);
                    ops += 1;
                }
                (Option::None, Option::None) => {}
                other => panic!("{w:08x}: {other:?}"),
            }
        }
    }
    assert_eq!(ops, 16 * (55 + 4 + 65), "every VU0 operation under every rs from 10h");
}

/// The spelling: Appendix A's, the R3000's, with the C790's three-operand multiplies and its own moves.
#[test]
fn the_spelling_is_appendix_a_s() {
    let t = |w: u32, pc: u32| {
        let i = decode(w, pc);
        if i.operands.is_empty() { i.mnemonic } else { format!("{} {}", i.mnemonic, i.operands) }
    };
    assert_eq!(t(0, 0), "nop");
    assert_eq!(t(0x0085_1018, 0), "mult v0, a0, a1");
    assert_eq!(encode("mult a0, a1", 0, &|_| Option::None), Ok(0x0085_0018));
    assert_eq!(t(0x7085_0009 | 0x0D << 6, 0), "pdivw a0, a1");
    assert_eq!(t(0x7C85_0010, 0), "sq a1, 0x10(a0)");
    assert_eq!(decode(0x7805_0010, 0).reference, Some((Reference::Read, 0x10)));
    assert_eq!(t(0x4002_C000, 0), "mfbpc v0");
    assert_eq!(t(0x4002_6000, 0), "mfc0 v0, Status");
    assert_eq!(t(0x4082_C803, 0), "mtpc v0, 1");
    assert_eq!(t(0x0000_040F, 0), "sync.p");
    assert_eq!(t(0x7000_1030 | 2 << 6, 0), "pmfhl.slw v0");
    assert_eq!(t(0x4600_1018, 0), "adda.s f2, f0");
    assert_eq!(t(0x4602_0034, 0), "c.lt.s f0, f2");
    assert_eq!(t(0x4822_0800, 0), "qmfc2 v0, vf1");
    assert_eq!(t(0xD841_0010, 0), "lqc2 vf1, 0x10(v0)");
    assert_eq!(t(0x4BE3_1088, 0), "vmaddx.xyzw vf2, vf2, vf3");
    assert_eq!(t(0x1000_FFFF, 0x8001_0000), "beq zero, zero, 0x80010000");
    assert_eq!(t(0xD400_0000, 0), ".word 0xd4000000", "LDC1 is the TX79's alone (D-1)");
    assert_eq!(t(0x4620_0000, 0), ".word 0x46200000", "the D format is the TX79's alone (D-1)");
    assert!(encode("mfc0 v0, Debug", 0, &|_| Option::None).is_err(), "rd 24 is MFBPC");
    assert!(encode("add.d f0, f0, f0", 0, &|_| Option::None).is_err());
}
