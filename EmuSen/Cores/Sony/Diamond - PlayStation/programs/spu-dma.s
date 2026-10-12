# How long the SPU's transfers take (Diamond_SPU.md §6): timer 2 on the system clock, interrupts off, the code run from
# the I-cache's second pass. From 80100000h, a word each: DMA4 writing 1 KiB in blocks of 16 words (CHCR's start to
# its clear, polled), the same in blocks of 1 word, in one burst of 256 words (sync mode 0), DMA4 reading 1 KiB in
# blocks of 16; then the cycles from ATTR's write of manual write mode to STATX showing it, and from there to STATX's
# busy bit clearing after 32 halfwords; then the polls of CHCR while the first case ran. Each case twice, the second
# kept. 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui s0, 0x1F80
        lui s1, 0x8010
        lui t0, 0x2209
        ori t0, t0, 0x31E1
        sw t0, 0x1014(s0)
        lw t0, 0x10F0(s0)
        lui t1, 0x0008
        or t0, t0, t1
        sw t0, 0x10F0(s0)
        ori t0, zero, 4
        sh t0, 0x1DAC(s0)
        ori s7, zero, 0xC000
        sh zero, 0x1D80(s0)
        sh zero, 0x1D82(s0)
        sh zero, 0x1124(s0)               # timer 2: the system clock, free running
        ori s5, zero, 2
twice:  lui t0, 0x0010
        ori a3, t0, 0x0010                # 16 blocks of 16 words
        lui a1, 0x0100
        ori a1, a1, 0x0201
        jal timed
        ori a2, zero, 0x20
        sw v0, 0(s1)
        sw v1, 24(s1)
        lui t0, 0x0100
        ori a3, t0, 0x0001                # 256 blocks of 1 word
        lui a1, 0x0100
        ori a1, a1, 0x0201
        jal timed
        ori a2, zero, 0x20
        sw v0, 4(s1)
        ori a3, zero, 0x0100              # 256 words at once
        lui a1, 0x1100
        ori a1, a1, 0x0001
        jal timed
        ori a2, zero, 0x20
        sw v0, 8(s1)
        lui t0, 0x0010
        ori a3, t0, 0x0010
        lui a1, 0x0100
        ori a1, a1, 0x0200
        jal timed
        ori a2, zero, 0x30
        sw v0, 12(s1)
# manual write: 32 halfwords into the FIFO in stop mode, then the mode written and timed until shown, then until idle
        jal mode
        or a0, zero, zero
        ori t0, zero, 0x0200
        sh t0, 0x1DA6(s0)
        ori t1, zero, 32
mfill:  sh t1, 0x1DA8(s0)
        addiu t1, t1, -1
        bne t1, zero, mfill
        nop
        ori t0, zero, 0xC010
        lhu t2, 0x1120(s0)
        sh t0, 0x1DAA(s0)
mapply: lhu t1, 0x1DAE(s0)
        nop
        andi t1, t1, 0x3F
        ori t3, zero, 0x10
        bne t1, t3, mapply
        nop
        lhu t3, 0x1120(s0)
mbusy:  lhu t1, 0x1DAE(s0)
        nop
        andi t1, t1, 0x400
        bne t1, zero, mbusy
        nop
        lhu t4, 0x1120(s0)
        subu t0, t3, t2
        andi t0, t0, 0xFFFF
        sw t0, 16(s1)
        subu t0, t4, t3
        andi t0, t0, 0xFFFF
        sw t0, 20(s1)
        addiu s5, s5, -1
        bne s5, zero, twice
        nop
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(s1)
done:   j done
        nop

# timed: DMA4 from 80120000h with BCR a3 and CHCR a1 in the SPU's mode a2 (20h write, 30h read) at sound RAM 1000h;
# v0 the cycles from CHCR's write to the poll that sees its start bit clear, v1 the polls
timed:  or s6, ra, zero
        or a0, zero, zero
        jal mode
        nop
        ori t0, zero, 0x0200
        sh t0, 0x1DA6(s0)
        jal mode
        or a0, a2, zero
        lui t0, 0x8012
        sw t0, 0x10C0(s0)
        sw a3, 0x10C4(s0)
        lui t5, 0x0100
        or v1, zero, zero
        lhu t2, 0x1120(s0)
        sw a1, 0x10C8(s0)
tpoll:  lw t1, 0x10C8(s0)
        addiu v1, v1, 1
        and t1, t1, t5
        bne t1, zero, tpoll
        nop
        lhu t3, 0x1120(s0)
        nop
        subu v0, t3, t2
        andi v0, v0, 0xFFFF
        jal mode
        or a0, zero, zero
        jr s6
        nop

# mode: ATTR = s7 | a0, then STATX polled until its bits 0-5 show it
mode:   or t0, s7, a0
        sh t0, 0x1DAA(s0)
        andi t0, t0, 0x3F
mwait:  lhu t1, 0x1DAE(s0)
        nop
        andi t1, t1, 0x3F
        bne t1, t0, mwait
        nop
        jr ra
        nop
