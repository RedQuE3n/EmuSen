//! Stage S33's paired program for the debugger's gate (Sapphire_Native.md §47.8, §41.10 Q3): the IOP polls the SIF while
//! the EE's DMA sends it a packet at a known instant, counting its polls of its SIF1 channel until the packet has landed
//! and of MSFLG until the EE sets it, and keeps both counts in its RAM. Each count is a measure of the IOP's time against
//! the EE's, so a run that let the IOP lag behind the board (the debugger's probe stopping it where the board does not
//! bring it to time) leaves different counts in the state. Every register and field is ps2tek's "SIF Registers", "SIF DMA"
//! and "IOP DMA" and the EE User's Manual's chapter 5, as the S22 programs use them.

#![allow(dead_code)]

use sapphire::board::System;

pub const EE_ORIGIN: u32 = 0x8010_0000;
pub const IOP_ORIGIN: u32 = 0xA010_0000;
/// Where the IOP keeps its two counts, and where the packet lands in IOP RAM.
pub const COUNTS: u32 = 0x001D_0000;
pub const LANDS: u32 = 0x001C_0000;

/// The EE: the packet (the IOP's tag, then 64 words) written at 00300000h, a delay, SIF1 started in normal mode toward
/// the IOP, a second delay and MSFLG's bit 0 set; then an idle loop.
const EE: &str = "
    lui t0, 0xa030
    lui t1, 0x801c
    sw t1, 0(t0)
    ori t1, zero, 64
    sw t1, 4(t0)
    sw zero, 8(t0)
    sw zero, 12(t0)
    ori t2, zero, 64
    lui t1, 0x5300
    fill: sw t1, 16(t0)
    addiu t1, t1, 1
    addiu t0, t0, 4
    addiu t2, t2, -1
    bne t2, zero, fill
    nop
    lui t0, 0xb001
    ori t1, zero, 1
    sw t1, -0x2000(t0)
    ori t2, zero, 3000
    delay: addiu t2, t2, -1
    bne t2, zero, delay
    nop
    lui t1, 0x30
    sw t1, -0x3bf0(t0)
    ori t1, zero, 17
    sw t1, -0x3be0(t0)
    ori t1, zero, 0x101
    sw t1, -0x3c00(t0)
    ori t2, zero, 2000
    again: addiu t2, t2, -1
    bne t2, zero, again
    nop
    ori t1, zero, 1
    sw t1, -0xde0(t0)
    end: beq zero, zero, end
    nop
";

/// The IOP: its SIF channels enabled, SIF1 started to receive in chain mode, then SIF1's CHCR polled and counted until
/// its STR clears, then MSFLG polled and counted until set; both counts kept at `COUNTS`.
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
    lui t1, 0x0100
    ori t1, t1, 0x0600
    sw t1, 0x1538(t0)
    lui t4, 0xbd00
    lui t6, 0xa01d
    or t3, zero, zero
    landed: lw t5, 0x1538(t0)
    addiu t3, t3, 1
    srl t5, t5, 24
    andi t5, t5, 1
    bne t5, zero, landed
    nop
    sw t3, 0(t6)
    or t3, zero, zero
    flag: lw t5, 0x20(t4)
    addiu t3, t3, 1
    beq t5, zero, flag
    nop
    sw t3, 4(t6)
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

/// The IOP's two counts, and the packet's first and last words where it landed.
pub fn record(sys: &System) -> [u32; 4] {
    let w = |a: u32| sys.board.iop.bus.word(a);
    [w(COUNTS), w(COUNTS + 4), w(LANDS), w(LANDS + 252)]
}
