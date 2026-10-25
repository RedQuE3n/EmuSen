//! The S27 programs (Sapphire_Native.md §34.10): a bare EE program that replays a command log through the IPU's
//! registers with the DMAC moving the data, as an FMV player would: the stream image in memory behind the code, toIPU
//! (channel 4) carrying it in normal mode from the first BCLR on, and for each command of the table IPU_CTRL written
//! first where the log wrote it, fromIPU (channel 3) started for the command's output straight into the record, IPU_CMD
//! written, IPU_CTRL.BUSY and channel 3's STR awaited in bounded loops, then IPU_CMD, IPU_CTRL (its FIFO counts
//! cleared, which depend on the DMA's pace) and IPU_BP's pointer recorded with the EE cycles the command took by Count.
//! Its record is one qword of registers, one of timing, then the output, for each command; the timing is not graded.
//! In S7's form: entered at 00100000h, run from kseg0 once its first jump is taken, everything else through kseg1.

#![allow(dead_code)]

use super::driver::Issued;
use sapphire::ipu::code;

pub const ORIGIN: u32 = 0x0010_0000;
pub const RECORD: u32 = 0x0020_0000;
pub const DONE_AT: u32 = 0x001F_FFF0;
pub const DONE: u32 = 0x5A99_D00E;
/// The bounded waits' passes, far beyond any command's time on the slowest referee.
const LIMIT: u32 = 4_000_000;

/// A program: its name, its image from `ORIGIN`, and its record (a word each, `None` where any value is taken).
pub struct Program {
    pub name: String,
    pub image: Vec<u32>,
    pub expected: Vec<Option<u32>>,
    pub commands: usize,
    /// Each command's word and output qwords, for reading a referee's record by command.
    pub words: Vec<u32>,
    pub out: Vec<usize>,
}

fn li(lines: &mut Vec<String>, reg: &str, v: u32) {
    lines.push(format!("lui {reg}, 0x{:x}", v >> 16));
    lines.push(format!("ori {reg}, {reg}, 0x{:x}", v & 0xFFFF));
}

/// The code, with the table and the stream image at the addresses given.
fn code(table: u32, entries: usize, stream: u32, qwords: usize) -> Vec<String> {
    let mut l = Vec::new();
    let k1 = 0xA000_0000u32;
    l.push("lui s1, 0xa020".to_string());
    li(&mut l, "t9", (ORIGIN + 20) | 0x8000_0000);
    l.push("jr t9".into());
    l.push("nop".into());
    for (reg, v) in [(0x1000_2010u32, 0x4000_0000u32), (0x1000_B000, 0), (0x1000_B400, 0), (0x1000_E020, 0), (0x1000_E010, 0x3FF), (0x1000_F590, 0), (0x1000_E000, 1)] {
        li(&mut l, "t6", k1 | reg);
        li(&mut l, "t7", v);
        l.push("sw t7, 0(t6)".into());
    }
    li(&mut l, "s0", k1 | table);
    li(&mut l, "s2", k1 | (table + 16 * entries as u32));
    li(&mut l, "s3", 0x1FFF_FFFF);
    li(&mut l, "s4", 0xFFFF_FF00);
    li(&mut l, "s5", k1 | 0x1000_2000);
    li(&mut l, "s6", k1 | 0x1000_B000);
    li(&mut l, "s7", k1 | 0x1000_B400);
    l.push("loop:".into());
    l.push("beq s0, s2, done".into());
    l.push("nop".into());
    l.push("lw t0, 0(s0)".into());
    l.push("lw t1, 4(s0)".into());
    l.push("lw t2, 8(s0)".into());
    l.push("lw a0, 12(s0)".into());
    l.push("addiu t3, zero, -1".into());
    l.push("beq t2, t3, noctrl".into());
    l.push("nop".into());
    l.push("sw t2, 0x10(s5)".into());
    l.push("noctrl:".into());
    l.push("beq t1, zero, nodma".into());
    l.push("nop".into());
    l.push("and t7, s1, s3".into());
    l.push("daddiu t7, t7, 32".into());
    l.push("sw t7, 0x10(s6)".into());
    l.push("sw t1, 0x20(s6)".into());
    l.push("addiu t7, zero, 0x100".into());
    l.push("sw t7, 0(s6)".into());
    l.push("nodma:".into());
    l.push("mfc0 t8, Count".into());
    l.push("sw t0, 0(s5)".into());
    li(&mut l, "t5", LIMIT);
    l.push("w1:".into());
    l.push("lw t2, 0x10(s5)".into());
    l.push("bgez t2, w1done".into());
    l.push("nop".into());
    l.push("addiu t5, t5, -1".into());
    l.push("bne t5, zero, w1".into());
    l.push("nop".into());
    l.push("w1done:".into());
    l.push("mfc0 t9, Count".into());
    l.push("beq t1, zero, w2done".into());
    l.push("nop".into());
    li(&mut l, "t4", LIMIT);
    l.push("w2:".into());
    l.push("lw t2, 0(s6)".into());
    l.push("andi t2, t2, 0x100".into());
    l.push("beq t2, zero, w2done".into());
    l.push("nop".into());
    l.push("addiu t4, t4, -1".into());
    l.push("bne t4, zero, w2".into());
    l.push("nop".into());
    l.push("w2done:".into());
    l.push("beq a0, zero, nostart".into());
    l.push("nop".into());
    li(&mut l, "t7", stream);
    l.push("sw t7, 0x10(s7)".into());
    li(&mut l, "t7", qwords as u32);
    l.push("sw t7, 0x20(s7)".into());
    l.push("addiu t7, zero, 0x101".into());
    l.push("sw t7, 0(s7)".into());
    l.push("nostart:".into());
    l.push("ld t2, 0(s5)".into());
    l.push("sd t2, 0(s1)".into());
    l.push("lw t2, 0x10(s5)".into());
    l.push("and t2, t2, s4".into());
    l.push("sw t2, 8(s1)".into());
    l.push("lw t2, 0x20(s5)".into());
    l.push("andi t2, t2, 0x7f".into());
    l.push("sw t2, 12(s1)".into());
    l.push("subu t2, t9, t8".into());
    l.push("sw t2, 16(s1)".into());
    l.push("sw t5, 20(s1)".into());
    l.push("sw t4, 24(s1)".into());
    l.push("sw zero, 28(s1)".into());
    l.push("sll t3, t1, 4".into());
    l.push("daddu s1, s1, t3".into());
    l.push("daddiu s1, s1, 32".into());
    l.push("daddiu s0, s0, 16".into());
    l.push("beq zero, zero, loop".into());
    l.push("nop".into());
    l.push("done:".into());
    li(&mut l, "t0", DONE);
    li(&mut l, "t1", k1 | DONE_AT);
    l.push("sw t0, 0(t1)".into());
    l.push("end:".into());
    l.push("beq zero, zero, end".into());
    l.push("nop".into());
    l
}

fn assemble(lines: &[String]) -> Vec<u32> {
    emusen_mips::assemble(&lines.join("\n"), ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}"))
}

/// The program replaying `log` over `image` (the driver's stream image, `driver::image`).
pub fn build(name: &str, image: &[u128], log: &[Issued]) -> Program {
    let probe = assemble(&code(0, log.len(), 0, image.len()));
    let table = (ORIGIN + 4 * probe.len() as u32).next_multiple_of(16);
    let stream = (table + 16 * log.len() as u32).next_multiple_of(16);
    let mut words = assemble(&code(table, log.len(), stream, image.len()));
    assert_eq!(words.len(), probe.len());
    words.resize(((table - ORIGIN) / 4) as usize, 0);
    let mut expected = Vec::new();
    for (k, i) in log.iter().enumerate() {
        let out = i.out.len() as u32;
        let start = u32::from(k == 0 && i.word >> 28 == code::BCLR);
        words.extend([i.word, out, i.ctrl_write.unwrap_or(u32::MAX), start]);
        let decoded = matches!(i.word >> 28, code::VDEC | code::FDEC);
        expected.extend([decoded.then_some(i.cmd as u32), decoded.then_some((i.cmd >> 32) as u32), Some(i.ctrl & 0xFFFF_FF00), Some(i.bp & 0x7F)]);
        expected.extend([None; 4]);
        for q in &i.out {
            expected.extend((0..4).map(|w| Some((q >> (32 * w)) as u32)));
        }
    }
    words.extend(image.iter().flat_map(|q| (0..4).map(move |w| (q >> (32 * w)) as u32)));
    assert!(ORIGIN + 4 * words.len() as u32 <= DONE_AT, "{name}: the image reaches the done word");
    let (cmds, out) = (log.iter().map(|i| i.word).collect(), log.iter().map(|i| i.out.len()).collect());
    Program { name: name.to_string(), image: words, expected, commands: log.len(), words: cmds, out }
}
