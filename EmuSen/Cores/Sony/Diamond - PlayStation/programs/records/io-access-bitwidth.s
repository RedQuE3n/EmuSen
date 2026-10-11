# ps1-tests' cpu/io-access-bitwidth, rewritten to record in RAM (Diamond_Records.md §11, §14). This file is the
# program's own part only: the entry, the stubs, the exception path and the start-up. The test's compiled code
# (80010010h-80014517h: main.cpp and asm.s with the common/ files it links) is not kept here: tests/support/records.rs
# carries it from the local release build 158 at the @release line, word for word at the release's addresses, its
# words pinned by SHA-256, and points its calls into PSn00bSDK at the stubs below. Bare metal, interrupts off; an
# exception goes through the program's own stand-in for the kernel's path to the handler the test hooks. Results from
# 80100000h: every printf call in order, six words a call (the format's address and its first five arguments), zero
# after the last; in the BIOS row a value is recorded only as whether it is the ROM's own word (format 1) or not (2).
# psxrecords.py renders them with the release's formats. 800FFFF0h holds D1A0D00Eh when done.
        j setup
        nop
vector: j handler                     # copied to 80000080h
        nop
# @release 80010010 80014518: the release build 158's own code, carried here by tests/support/records.rs
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
# printf(a0, ...): the call as it was made, six words at the cursor (800FFFE0h): the format's address, a1 to a3, and
# the fifth and sixth arguments from the caller's frame. The section's width (800FFFE4h) and the row's address
# (800FFFE8h) are kept from their headings, and for the BIOS row the ROM's word before the row's writes (800FFFECh), so
# that a value read there is compared with it at its width and never stored.
record: lui t0, 0x8010
        lui t2, 0x8001
        ori t3, t2, 0x5668            # "\nReading as %dbit, writing as:\n"
        bne a0, t3, m1
        ori t3, t2, 0x56B8            # "%-10s (0x%08x)   "
        sw a1, -0x1C(t0)
m1:     bne a0, t3, m2
        ori t3, t2, 0x56E4            # "%#10x    "
        sw a2, -0x18(t0)
        lui t4, 0xBFC0
        bne a2, t4, m2
        nop
        lw t4, 0(a2)
        nop
        sw t4, -0x14(t0)
m2:     bne a0, t3, m3
        lui t4, 0xBFC0
        lw t5, -0x18(t0)
        nop
        bne t5, t4, m3
        nop
        lw t4, -0x14(t0)
        lw t5, -0x1C(t0)
        addiu t6, zero, 32
        subu t6, t6, t5
        sllv t4, t4, t6
        srlv t4, t4, t6
        xor t4, t4, a1
        sltu t4, zero, t4
        addiu a0, t4, 1
        or a1, zero, zero
m3:     lw t1, -0x20(t0)
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
# The kernel's exception path, as the test relies on it: the registers saved in the current thread (800FFE10h, which
# 108h reaches through 800FFE00h), with EPC as its return address and SR and CAUSE beside them; the handler hooked at
# 300h called; the registers restored and a return to the thread's return address, which the handler moves past the
# fault.
handler: lui k0, 0x800F
        ori k0, k0, 0xFE10
        sw at, 0xC(k0)
        sw v0, 0x10(k0)
        sw v1, 0x14(k0)
        sw a0, 0x18(k0)
        sw a1, 0x1C(k0)
        sw a2, 0x20(k0)
        sw a3, 0x24(k0)
        sw t0, 0x28(k0)
        sw t1, 0x2C(k0)
        sw t2, 0x30(k0)
        sw t3, 0x34(k0)
        sw t4, 0x38(k0)
        sw t5, 0x3C(k0)
        sw t6, 0x40(k0)
        sw t7, 0x44(k0)
        sw s0, 0x48(k0)
        sw s1, 0x4C(k0)
        sw s2, 0x50(k0)
        sw s3, 0x54(k0)
        sw s4, 0x58(k0)
        sw s5, 0x5C(k0)
        sw s6, 0x60(k0)
        sw s7, 0x64(k0)
        sw t8, 0x68(k0)
        sw t9, 0x6C(k0)
        sw gp, 0x78(k0)
        sw sp, 0x7C(k0)
        sw fp, 0x80(k0)
        sw ra, 0x84(k0)
        mfhi k1
        sw k1, 0x8C(k0)
        mflo k1
        sw k1, 0x90(k0)
        mfc0 k1, EPC
        nop
        sw k1, 0x88(k0)
        mfc0 k1, SR
        nop
        sw k1, 0x94(k0)
        mfc0 k1, CAUSE
        nop
        sw k1, 0x98(k0)
        lui sp, 0x800F
        ori sp, sp, 0xFD00
        lw t0, 0x300(zero)
        nop
        jalr t0
        nop
        lui k0, 0x800F
        ori k0, k0, 0xFE10
        lw k1, 0x8C(k0)
        nop
        mthi k1
        lw k1, 0x90(k0)
        nop
        mtlo k1
        lw at, 0xC(k0)
        lw v0, 0x10(k0)
        lw v1, 0x14(k0)
        lw a0, 0x18(k0)
        lw a1, 0x1C(k0)
        lw a2, 0x20(k0)
        lw a3, 0x24(k0)
        lw t0, 0x28(k0)
        lw t1, 0x2C(k0)
        lw t2, 0x30(k0)
        lw t3, 0x34(k0)
        lw t4, 0x38(k0)
        lw t5, 0x3C(k0)
        lw t6, 0x40(k0)
        lw t7, 0x44(k0)
        lw s0, 0x48(k0)
        lw s1, 0x4C(k0)
        lw s2, 0x50(k0)
        lw s3, 0x54(k0)
        lw s4, 0x58(k0)
        lw s5, 0x5C(k0)
        lw s6, 0x60(k0)
        lw s7, 0x64(k0)
        lw t8, 0x68(k0)
        lw t9, 0x6C(k0)
        lw gp, 0x78(k0)
        lw sp, 0x7C(k0)
        lw fp, 0x80(k0)
        lw ra, 0x84(k0)
        lw k1, 0x88(k0)
        nop
        jr k1
        rfe
# the stack and gp as the release's start-up leaves them, its data and the record zeroed, then main
setup:  mtc0 zero, SR
        lui sp, 0x801F
        ori sp, sp, 0xFEE0
        lui gp, 0x8001
        ori gp, gp, 0xE9D0
        lui t0, 0x8001
        ori t0, t0, 0x5000
        lui t1, 0x8001
        ori t1, t1, 0x8000
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
        lui t0, 0x800F
        ori t0, t0, 0xFE00
        lui t1, 0x800F
        ori t1, t1, 0xFF00
z3:     sw zero, 0(t0)
        addiu t0, t0, 4
        bne t0, t1, z3
        nop
        lui t0, 0x8001
        lw t1, 8(t0)
        lw t2, 12(t0)
        lui t3, 0x8000
        sw t1, 0x80(t3)
        sw t2, 0x84(t3)
        lui t1, 0x800F
        ori t1, t1, 0xFE00
        sw t1, 0x108(t3)              # the process table, its first entry the current thread
        ori t2, t1, 0x0010
        sw t2, 0(t1)
        lui t0, 0x8010
        sw t0, -0x20(t0)
        jal 0x80010020
        nop
