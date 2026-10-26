//! Stage S33's paired program for the debugger's gate (Sapphire_Native.md §47.8, §41.10 Q3): the IOP polls the SIF while
//! the EE's DMA sends it eight packets, each at a known instant, counting its polls of its SIF1 channel until each packet
//! has landed, and keeps the counts in its RAM. Each count is a measure of the IOP's time against
//! the EE's, so a run that let the IOP lag behind the board (the debugger's probe stopping it where the board does not
//! bring it to time) leaves different counts in the state. Every register and field is ps2tek's "SIF Registers", "SIF DMA"
//! and "IOP DMA" and the EE User's Manual's chapter 5, as the S22 programs use them.

#![allow(dead_code)]

use sapphire::board::System;

pub const EE_ORIGIN: u32 = 0x8010_0000;
pub const IOP_ORIGIN: u32 = 0xA010_0000;
/// Where the IOP keeps its counts, and where the packets land in IOP RAM.
pub const COUNTS: u32 = 0x001D_0000;
pub const LANDS: u32 = 0x001C_0000;

/// The EE: eight packets (each the IOP's tag, then four words) written from 00300000h; then each sent in turn by SIF1
/// in normal mode once the one before has gone and a short delay has passed, and an idle loop.
const EE: &str = "
    lui t0, 0xa030
    ori t2, zero, 8
    lui t1, 0x801c
    lui t3, 0x5300
    fill: sw t1, 0(t0)
    ori t4, zero, 4
    sw t4, 4(t0)
    sw zero, 8(t0)
    sw zero, 12(t0)
    sw t3, 16(t0)
    sw t3, 20(t0)
    sw t3, 24(t0)
    sw t3, 28(t0)
    addiu t3, t3, 1
    addiu t1, t1, 0x10
    addiu t0, t0, 32
    addiu t2, t2, -1
    bne t2, zero, fill
    nop
    lui t0, 0xb001
    ori t1, zero, 1
    sw t1, -0x2000(t0)
    lui s1, 0x30
    ori s2, zero, 8
    send: lw t1, -0x3c00(t0)
    andi t1, t1, 0x100
    bne t1, zero, send
    nop
    ori t2, zero, 4000
    delay: addiu t2, t2, -1
    bne t2, zero, delay
    nop
    sw s1, -0x3bf0(t0)
    ori t1, zero, 2
    sw t1, -0x3be0(t0)
    ori t1, zero, 0x101
    sw t1, -0x3c00(t0)
    addiu s1, s1, 32
    addiu s2, s2, -1
    bne s2, zero, send
    nop
    end: beq zero, zero, end
    nop
";

/// The IOP: its SIF channels enabled; for each of the eight packets, SIF1 started to receive in chain mode and its CHCR
/// polled and counted until its STR clears, each count kept at `COUNTS` in turn.
const IOP: &str = "
    lui t0, 0xbf80
    ori t1, zero, 0x800
    sw t1, 0x10f0(t0)
    ori t1, zero, 0x8800
    sw t1, 0x1570(t0)
    lui t1, 0x80
    sw t1, 0x10f4(t0)
    lui t1, 0x3f00
    sw t1, 0x1574(t0)
    ori t1, zero, 1
    sw t1, 0x1578(t0)
    lui t6, 0xa01d
    ori s0, zero, 8
    next: lui t1, 0x0100
    ori t1, t1, 0x0600
    sw t1, 0x1538(t0)
    or t3, zero, zero
    landed: lw t5, 0x1538(t0)
    addiu t3, t3, 1
    srl t5, t5, 24
    andi t5, t5, 1
    bne t5, zero, landed
    nop
    sw t3, 0(t6)
    addiu t6, t6, 4
    addiu s0, s0, -1
    bne s0, zero, next
    nop
    end: j end
    nop
";

/// The machine: the EE entered at its program, the IOP at its own, the IOP's counts cleared.
pub fn system() -> System {
    let ee = emusen_mips::assemble(EE, EE_ORIGIN, &emusen_mips::Dialect::R5900).expect("the EE's side assembles");
    let iop = emusen_mips::assemble(IOP, IOP_ORIGIN, &blackmoon_r3000::dialect(None)).expect("the IOP's side assembles");
    let mut sys = System::new(EE_ORIGIN);
    sys.board.map.load(EE_ORIGIN & 0x1FFF_FFFF, &ee);
    sys.board.iop.bus.load(IOP_ORIGIN & 0x1F_FFFF, &iop);
    sys.board.iop.enter(IOP_ORIGIN);
    sys
}

/// The IOP's eight counts, and each packet's first word where it landed.
pub fn record(sys: &System) -> ([u32; 8], [u32; 8]) {
    let w = |a: u32| sys.board.iop.bus.word(a);
    (std::array::from_fn(|i| w(COUNTS + 4 * i as u32)), std::array::from_fn(|i| w(LANDS + 16 * i as u32)))
}
