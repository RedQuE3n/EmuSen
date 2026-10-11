# ps1-tests' dma/dpcr, rewritten to record in RAM (Diamond_Records.md §11, §13). This file is the program's own part
# only: the entry, the stubs and the start-up. The test's compiled code (80010008h-800109AFh: main.cpp with the common/
# files it links) is not kept here: tests/support/records.rs carries it from the local release build 158 at the line
# marked @release, word for word at the release's addresses, its words pinned by SHA-256, and points its calls into
# PSn00bSDK at the stubs below, which make the bus accesses the release's library was seen to make on Diamond. Bare
# metal, interrupts off. Results from 80100000h: every printf call in order, six words a call (the format's address
# and its first five arguments), zero after the last; psxrecords.py renders them with the release's formats.
# 800FFFF0h holds D1A0D00Eh when done.
        j setup
        nop
# @release 80010008 800109B0: the release build 158's own code, carried here by tests/support/records.rs
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
# PutDispEnv: 320x240, as the release wrote it
putdisp: lui t0, 0x1F80
        lui t1, 0x06C5
        ori t1, t1, 0x8258
        sw t1, 0x1814(t0)
        lui t1, 0x0704
        ori t1, t1, 0x0010
        sw t1, 0x1814(t0)
        lui t1, 0x0800
        ori t1, t1, 0x0001
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
        ori t1, t1, 0x020A
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
# VSync, called once the tests are done
finish: lui t0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(t0)
done:   j done
        nop
# SpuInit, as the release's call was seen to reach the bus: SPUCNT 0 and SPUSTAT until its mode bits follow;
# 1D80h-1D86h, 1DB0h-1DB6h and 1D90h-1D9Ah 0, 1DA2h FFFEh; the 24 voices' seven registers 0 from voice 23 down; 1D8Ch
# and 1D8Eh FFFFh; 1DACh 4; DPCR read and written with channel 4 enabled; SPUCNT C001h and SPUSTAT until its mode bits
# follow; 1D80h, 1D82h, 1DB0h and 1DB2h 3FFFh, the last two twice. Each SPUSTAT wait was met at its first read on
# Diamond; here it gives up after 10,000.
spuinit: addiu sp, sp, -8
        sw ra, 0(sp)
        lui t0, 0x1F80
        sh zero, 0x1DAA(t0)
        jal spuwait
        or t4, zero, zero
        sh zero, 0x1D80(t0)
        sh zero, 0x1D82(t0)
        sh zero, 0x1D84(t0)
        sh zero, 0x1D86(t0)
        sh zero, 0x1DB0(t0)
        sh zero, 0x1DB2(t0)
        sh zero, 0x1DB4(t0)
        sh zero, 0x1DB6(t0)
        sh zero, 0x1D90(t0)
        sh zero, 0x1D92(t0)
        sh zero, 0x1D94(t0)
        sh zero, 0x1D96(t0)
        sh zero, 0x1D98(t0)
        sh zero, 0x1D9A(t0)
        addiu t1, zero, -2
        sh t1, 0x1DA2(t0)
        lui t2, 0x1F80
        ori t2, t2, 0x1D70
        lui t3, 0x1F80
        ori t3, t3, 0x1BF0
sv:     sh zero, 0(t2)
        sh zero, 2(t2)
        sh zero, 4(t2)
        sh zero, 6(t2)
        sh zero, 14(t2)
        sh zero, 8(t2)
        sh zero, 10(t2)
        addiu t2, t2, -16
        bne t2, t3, sv
        nop
        addiu t1, zero, -1
        sh t1, 0x1D8C(t0)
        sh t1, 0x1D8E(t0)
        addiu t1, zero, 4
        sh t1, 0x1DAC(t0)
        lw t1, 0x10F0(t0)
        lui t2, 0x0008
        or t1, t1, t2
        sw t1, 0x10F0(t0)
        ori t1, zero, 0xC001
        sh t1, 0x1DAA(t0)
        jal spuwait
        addiu t4, zero, 1
        ori t1, zero, 0x3FFF
        sh t1, 0x1D80(t0)
        sh t1, 0x1D82(t0)
        sh t1, 0x1DB0(t0)
        sh t1, 0x1DB2(t0)
        sh t1, 0x1DB0(t0)
        sh t1, 0x1DB2(t0)
        lw ra, 0(sp)
        addiu sp, sp, 8
        jr ra
        nop
# SPUSTAT until its low six bits read t4, at most 10,000 times
spuwait: ori t5, zero, 10000
sw1:    lhu t6, 0x1DAE(t0)
        addiu t5, t5, -1
        andi t6, t6, 0x3F
        beq t6, t4, sw2
        nop
        bne t5, zero, sw1
        nop
sw2:    jr ra
        nop
# strlen(a0)
strlen: or v0, a0, zero
sl:     lbu t0, 0(v0)
        nop
        bne t0, zero, sl
        addiu v0, v0, 1
        addiu v0, v0, -1
        jr ra
        subu v0, v0, a0
# printf(a0, ...): the call as it was made, six words at the cursor (800FFFE0h): the format's address, a1 to a3, and
# the fifth and sixth arguments from the caller's frame
record: lui t0, 0x8010
        lw t1, -0x20(t0)
        lw t2, 16(sp)
        lw t3, 20(sp)
        sw a0, 0(t1)
        sw a1, 4(t1)
        sw a2, 8(t1)
        sw a3, 12(t1)
        sw t2, 16(t1)
        sw t3, 20(t1)
        addiu t1, t1, 24
        jr ra
        sw t1, -0x20(t0)
# the stack and gp as the release's start-up leaves them, its data and the record zeroed, then main
setup:  mtc0 zero, SR
        lui sp, 0x801F
        ori sp, sp, 0xFEE0
        lui gp, 0x8001
        ori gp, gp, 0xA150
        lui t0, 0x8001
        ori t0, t0, 0x1F00
        lui t1, 0x8001
        ori t1, t1, 0x3000
z1:     sw zero, 0(t0)
        addiu t0, t0, 4
        bne t0, t1, z1
        nop
        lui t0, 0x8010
        ori t0, t0, 0x0000
        lui t1, 0x8010
        ori t1, t1, 0x3000
z2:     sw zero, 0(t0)
        addiu t0, t0, 4
        bne t0, t1, z2
        nop
        lui t0, 0x8001
        ori t0, t0, 0x1F18
        lui t1, 0x6C6C                # TEST_STRING, "Hello there   :)", at the release's address
        ori t1, t1, 0x6548
        sw t1, 0(t0)
        lui t1, 0x6874
        ori t1, t1, 0x206F
        sw t1, 4(t0)
        lui t1, 0x2065
        ori t1, t1, 0x7265
        sw t1, 8(t0)
        lui t1, 0x293A
        ori t1, t1, 0x2020
        sw t1, 12(t0)
        sw zero, 16(t0)
        lui t1, 0x8001
        sw t0, 0x2160(t1)             # and its pointer, at gp - 7FF0h
        lui t0, 0x8010
        sw t0, -0x20(t0)
        jal 0x80010020
        nop
