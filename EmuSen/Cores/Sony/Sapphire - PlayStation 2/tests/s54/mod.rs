//! The programs of the scheduler's step (Sapphire_Scheduler.md §54.7): each makes one of the board's calls visible
//! that the step answers without making it, so that a call left out or made late changes what the program records.
//! Registers and fields are ps2tek's "SIF Registers", "SIF DMA" and "IOP DMA", the EE User's Manual's chapter 5 and
//! the VU User's Manual's §5, as the S22, S13 and S33 programs use them.

#![allow(dead_code)]

use sapphire::board::System;

pub const EE_ORIGIN: u32 = 0x8010_0000;
pub const IOP_ORIGIN: u32 = 0xA010_0000;
/// Where the EE keeps its records, in RDRAM; where the IOP keeps its own, in its RAM.
pub const RECORDS: u32 = 0x0020_0000;
pub const IOP_RECORDS: u32 = 0x001D_0000;

fn ee(src: &str) -> Vec<u32> {
    emusen_mips::assemble(src, EE_ORIGIN, &emusen_mips::Dialect::R5900).expect("the EE's side assembles")
}

fn paired(ee_src: &str, iop_src: &str) -> System {
    paired_at(ee_src, iop_src, IOP_ORIGIN)
}

/// The two programs, the IOP's entered at `origin` (a cached one with its cache on, as the BIOS leaves it).
fn paired_at(ee_src: &str, iop_src: &str, origin: u32) -> System {
    let mut sys = System::new(EE_ORIGIN);
    sys.board.map.load(EE_ORIGIN & 0x1FFF_FFFF, &ee(ee_src));
    let iop = emusen_mips::assemble(iop_src, origin, &blackmoon_r3000::dialect(None)).expect("the IOP's side assembles");
    sys.board.iop.bus.load(origin & 0x1F_FFFF, &iop);
    if origin < 0xA000_0000 {
        sys.board.iop.bus.icache.bcc = sapphire::iop::icache::bcc::KERNEL;
    }
    sys.board.iop.enter(origin);
    sys
}

/// The IOP brought to each EE instruction's time: the IOP writes a count into SMCOM as fast as it can, and the EE reads
/// SMCOM 512 times, after delays of 0 to 7 turns, keeping each value with Count. The EE reads the register with the IOP
/// where the board left it after the instruction before, so a time at which the IOP was not brought along shows as a
/// value read one IOP store early.
pub fn lockstep() -> System {
    paired(
        "
        lui t0, 0xb001
        lui t9, 0xa020
        ori s0, zero, 512
        read: lw t1, -0xdf0(t0)
        mfc0 t3, Count
        sw t1, 0(t9)
        sw t3, 4(t9)
        addiu t9, t9, 8
        andi t2, s0, 7
        delay: beq t2, zero, done
        addiu t2, t2, -1
        beq zero, zero, delay
        nop
        done: addiu s0, s0, -1
        bne s0, zero, read
        nop
        end: beq zero, zero, end
        nop
        ",
        "
        lui t0, 0xbd00
        or t1, zero, zero
        count: sw t1, 0x10(t0)
        beq zero, zero, count
        addiu t1, t1, 1
        ",
    )
}

/// SIF1 kicked by the EE: eight packets, each sent once the one before has gone and a delay of a varying length has
/// passed; the IOP re-arms its SIF1 channel for each and counts its polls of the channel's CHCR until the packet has
/// landed (S33's paired program, `tests/s33/mod.rs`, with the delays varied). The EE's DMAC moves the FIFO at its
/// events and the IOP's side takes the kick at its next service, so a kick the IOP is not told of shows in the counts.
pub fn sif_kick() -> System {
    paired(
        "
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
        sll t2, s2, 9
        addiu t2, t2, 777
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
        ",
        "
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
        ",
    )
}

/// The VUs reached through COP2 between the board's calls: 48 rounds of a macro DIV whose quotient lands during a delay
/// of 3 to 10 turns, read back by CFC2 with Count; VU0's microprogram by VCALLMS and VU1's by CMSAR1, each after a delay,
/// and VPU-STAT read while VU1 may still run. Each COP2 instruction reaches the units' state itself, so a rest the board
/// keeps across one leaves the units where the plain run would not have them.
pub fn vpu() -> System {
    let mut sys = System::new(EE_ORIGIN);
    sys.board.map.load(
        EE_ORIGIN & 0x1FFF_FFFF,
        &ee("
        lui t9, 0xa020
        ori s0, zero, 48
        lui t0, 0x4040
        qmtc2 t0, vf1
        lui t0, 0x3f80
        qmtc2 t0, vf2
        round: vdiv q, vf1x, vf2x
        andi t1, s0, 7
        addiu t1, t1, 3
        d1: addiu t1, t1, -1
        bne t1, zero, d1
        nop
        cfc2 t2, vi22
        mfc0 t3, Count
        sw t2, 0(t9)
        sw t3, 4(t9)
        vcallms 0x0
        andi t1, s0, 3
        addiu t1, t1, 2
        d2: addiu t1, t1, -1
        bne t1, zero, d2
        nop
        ctc2 zero, vi31
        andi t1, s0, 15
        addiu t1, t1, 1
        d3: addiu t1, t1, -1
        bne t1, zero, d3
        nop
        cfc2 t4, vi29
        mfc0 t3, Count
        sw t4, 8(t9)
        sw t3, 12(t9)
        addiu t9, t9, 16
        addiu s0, s0, -1
        bne s0, zero, round
        nop
        end: beq zero, zero, end
        nop
        "),
    );
    let micro = emusen_mips::vu::assemble("nop | iaddiu vi5, vi5, 0x1\nnop | move vf3, vf1\n[e] nop | move vf4, vf2\nnop | iaddiu vi6, vi6, 0x1").expect("the microprogram assembles");
    for (i, p) in micro.iter().enumerate() {
        sys.board.vpu.units.vu0.micro[i * 8..i * 8 + 8].copy_from_slice(&p.to_le_bytes());
        sys.board.vpu.units.vu1.micro[i * 8..i * 8 + 8].copy_from_slice(&p.to_le_bytes());
    }
    sys.ee.cop0[sapphire::ee::cop0::STATUS] |= sapphire::ee::status::CU2;
    sys
}

/// Where the console hook's export table and the bytes the IOP writes are.
pub const TABLE: u32 = 0x001E_0000;
pub const TEXT: &[u8] = b"Hi!\n";

/// The IOP's console (S10, `iop::tty`) on a program of the project's own: the IOP registers an export table named
/// `ioman` whose entry 7 is a `write` of its own (its first instruction at 00100008h), from its cache, that returns its size, then writes "Hi!" and a line feed to standard
/// output by that entry after delays of 300 to 1,000 turns, for ever; the EE divides and waits for each quotient, so
/// that the board brings the IOP through several of its instructions at a time. The hook is found at the first frame's
/// end, and from then on every call is met at the hook's boundary, where the batched run must stop.
pub fn console() -> System {
    paired_at(
        "
        ori t0, zero, 7
        ori t1, zero, 3
        end: div t0, t1
        mflo t2
        beq zero, zero, end
        nop
        ",
        "
        beq zero, zero, main
        nop
        write: jr ra
        addu v0, a2, zero
        main: lui t0, 0xa01e
        lui t1, 0x616d
        ori t1, t1, 0x6f69
        sw t1, 12(t0)
        ori t1, zero, 0x6e
        sw t1, 16(t0)
        lui t1, 0x8010
        ori t1, t1, 0x0008
        sw t1, 48(t0)
        lui t1, 0x41c0
        sw t1, 0(t0)
        lui t1, 0xa01f
        lui t2, 0x0a21
        ori t2, t2, 0x6948
        sw t2, 0(t1)
        or s0, zero, zero
        call: ori a0, zero, 1
        lui a1, 0x801f
        jal write
        ori a2, zero, 4
        addiu s0, s0, 1
        andi t2, s0, 7
        sll t2, t2, 7
        addiu t2, t2, 300
        delay: addiu t2, t2, -1
        bne t2, zero, delay
        nop
        beq zero, zero, call
        nop
        ",
        0x8010_0000,
    )
}

/// The EE's records: `n` words from `RECORDS`.
pub fn ee_records(sys: &System, n: usize) -> Vec<u32> {
    let ram = &sys.board.map.rdram;
    (0..n).map(|i| RECORDS as usize + 4 * i).map(|a| u32::from_le_bytes([ram[a], ram[a + 1], ram[a + 2], ram[a + 3]])).collect()
}

/// The IOP's records: `n` words from `IOP_RECORDS`.
pub fn iop_records(sys: &System, n: usize) -> Vec<u32> {
    (0..n).map(|i| sys.board.iop.bus.word(IOP_RECORDS + 4 * i as u32)).collect()
}
