# ps1-tests' dma/chopping, rewritten to record in RAM (Diamond_Records.md §1, §4). This file is the program's own
# part only: the entry, the stubs and the start-up. The test's compiled code (80010008h-80010BDBh: main.cpp with the
# common/ files it links) is not kept here: tests/support/records.rs carries it from the local release build 158 at
# the @release line, word for word at the release's addresses, its words pinned by SHA-256, and points its calls into
# PSn00bSDK at the stubs below, which make the bus accesses the release's library was seen to make on Diamond. Bare
# metal, interrupts off. Results from 80100000h, six words a row, 130 rows in the log's order: the sync mode, the
# bytes, the block size or the DMA window in words (0 unchopped), the CPU window in cycles (0 but chopped), the CPU
# cycles, and 1 for "(GPU busy)". 800FFFF0h holds D1A0D00Eh when done.
        j setup
        nop
# @release 80010008 80010BDC: the release build 158's own code, carried here by tests/support/records.rs
# PSn00bSDK, as the release's calls were seen to reach the bus
none:   jr ra                         # SetDefDispEnv, SetDefDrawEnv, EnterCriticalSection, ExitCriticalSection
        nop
# DrawPrim(a0), timed as the release's was seen to time its accesses on Diamond: GPUSTAT; when bit 26 is set, the
# channel and GPUSTAT again, otherwise GPUSTAT until bit 28; GP1(04h) 0; the packet's words to GP0 nine cycles
# apart; then GPUSTAT until bit 28. Each GPUSTAT read is a call of ReadGPUstat there, twelve cycles a poll.
drawprim: lui t0, 0x1F80
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        lw t1, 0x1814(t0)
        lui t2, 0x0400
        and t2, t1, t2
        beq t2, zero, dpb
        lui t2, 0x1000
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        lw t1, 0x10A8(t0)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        lw t1, 0x1814(t0)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        j dps
        nop
dpb:
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
dpl:    lw t1, 0x1814(t0)
        nop
        nop
        nop
        nop
        nop
        nop
        and t1, t1, t2
        beq t1, zero, dpl
        nop
        nop
        nop
        nop
        nop
        nop
        nop
dps:    lui t1, 0x0400
        sw t1, 0x1814(t0)
        lbu t1, 3(a0)
        addiu a0, a0, 4
        nop
dpw:    lw t3, 0(a0)
        addiu t1, t1, -1
        sw t3, 0x1810(t0)
        bne t1, zero, dpw
        addiu a0, a0, 4
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        lw t1, 0x1814(t0)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
dpq:    lw t1, 0x1814(t0)
        nop
        nop
        nop
        nop
        nop
        nop
        and t1, t1, t2
        beq t1, zero, dpq
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        jr ra
        nop
# DrawSync(0): GPUSTAT, then GPUSTAT until bit 28, as the release's
drawsync: lui t0, 0x1F80
        lui t2, 0x1000
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        lw t1, 0x1814(t0)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
dsl:    lw t1, 0x1814(t0)
        nop
        nop
        nop
        nop
        nop
        nop
        and t1, t1, t2
        beq t1, zero, dsl
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        jr ra
        or v0, zero, zero
# ReadGPUstat
gpustat: lui t0, 0x1F80
        lw v0, 0x1814(t0)
        jr ra
        nop
# ResetGraph(0): DPCR, DICR, timer 1 on hblanks, GP1(00h)
resetgraph: lui t0, 0x1F80
        lui t1, 0x3B33
        ori t1, t1, 0x3B33
        sw t1, 0x10F0(t0)
        sw zero, 0x10F4(t0)
        addiu t1, zero, 0x1D00
        sh t1, 0x1114(t0)
        sw zero, 0x1814(t0)
        jr ra
        or v0, zero, zero
# PutDispEnv: 640x480 interlaced, as the release wrote it
putdisp: lui t0, 0x1F80
        lui t1, 0x06C6
        ori t1, t1, 0xC26C
        sw t1, 0x1814(t0)
        lui t1, 0x0704
        ori t1, t1, 0x0010
        sw t1, 0x1814(t0)
        lui t1, 0x0800
        ori t1, t1, 0x00A7
        sw t1, 0x1814(t0)
        lui t1, 0x0500
        jr ra
        sw t1, 0x1814(t0)
# PutDrawEnv: GP1(04h) 2 and a one-node list of E3h, E4h, E5h and E1h by DMA, as the release sent it
putdraw: lui t0, 0x1F80
        lui t1, 0x0400
        ori t1, t1, 2
        sw t1, 0x1814(t0)
        lui t3, 0x800F
        ori t3, t3, 0xFF00
        lui t1, 0x04FF
        ori t1, t1, 0xFFFF
        sw t1, 0(t3)
        lui t1, 0xE300
        sw t1, 4(t3)
        lui t1, 0xE407
        ori t1, t1, 0xFFFF
        sw t1, 8(t3)
        lui t1, 0xE500
        sw t1, 12(t3)
        lui t1, 0xE100
        ori t1, t1, 0x060A
        sw t1, 16(t3)
        sw t3, 0x10A0(t0)
        sw zero, 0x10A4(t0)
        lui t1, 0x0100
        ori t1, t1, 0x0401
        jr ra
        sw t1, 0x10A8(t0)
# SetDispMask(1)
dispmask: lui t0, 0x1F80
        lui t1, 0x0300
        jr ra
        sw t1, 0x1814(t0)
# SetVideoMode(MODE_NTSC)
videomode: lui t0, 0x1F80
        lui t1, 0x0800
        jr ra
        sw t1, 0x1814(t0)
# malloc(8192): the release's buffer
malloc: lui v0, 0x8001
        jr ra
        ori v0, v0, 0x3C94
# printf: a row's heading (blockSize; unchopped; windows) and its result, by the format's address; cursor at 800FFFE0h
record: lui t0, 0x8010
        lw t1, -0x20(t0)
        lui t2, 0x8001
        ori t3, t2, 0x1DD0
        beq a0, t3, rh
        ori t3, t2, 0x1E20
        beq a0, t3, rn
        ori t3, t2, 0x1E70
        beq a0, t3, rw
        ori t3, t2, 0x1DB4
        bne a0, t3, rx
        ori t3, t2, 0x1DA8
        sw a1, 16(t1)                 # the cycles
        xor t3, a2, t3
        sltiu t3, t3, 1
        sw t3, 20(t1)                 # "(GPU busy)"
        addiu t1, t1, 24
        sw t1, -0x20(t0)
rx:     jr ra
        nop
rw:     lw t3, 0x10(sp)               # the CPU window, the fifth argument
        sw a1, 0(t1)
        sw a2, 4(t1)
        sw a3, 8(t1)
        jr ra
        sw t3, 12(t1)
rn:     or a3, zero, zero
rh:     sw a1, 0(t1)
        sw a2, 4(t1)
        sw a3, 8(t1)
        jr ra
        sw zero, 12(t1)
# VSync, called once the table is done
finish: lui t0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(t0)
done:   j done
        nop
# the stack and gp as the release's start-up leaves them, its data zeroed, then main
setup:  mtc0 zero, SR
        lui sp, 0x801F
        ori sp, sp, 0xFEE0
        lui gp, 0x8001
        ori gp, gp, 0xA150
        lui t0, 0x8001
        ori t0, t0, 0x2000
        lui t1, 0x8001
        ori t1, t1, 0x3C94
z1:     sw zero, 0(t0)
        addiu t0, t0, 4
        bne t0, t1, z1
        nop
        lui t0, 0x8010
        sw t0, -0x20(t0)
        jal 0x80010020
        nop
