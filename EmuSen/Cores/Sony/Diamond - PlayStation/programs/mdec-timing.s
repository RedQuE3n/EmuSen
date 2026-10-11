# How long the MDEC takes to decode by DMA (Diamond_MDEC.md §4): five streams, each decoded with DMA0 in and DMA1 out
# in blocks of 32 words after a reset, the tables and the command written by the CPU. Timer 2 on the system clock / 8
# from DMA0's start: the tick each channel is first seen idle. The streams: A, 16 colour macroblocks of DC-only
# blocks at 15 bits; B, 16 at 15 bits whose blocks carry 63 AC values each; C, 96 DC-only blocks at 8 bits; D, A's
# stream at 24 bits; E, 96 blocks of 63 AC values at 4 bits. Results from 80100000h, two words a case (DMA0's tick,
# DMA1's tick, FFFFh if it never ended); 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a1, 0x1F80
        lui s0, 0x8010
        lui t0, 0x0765
        ori t0, t0, 0x43A9
        sw t0, 0x10F0(a1)             # DPCR: channels 0 and 1 enabled
        ori t0, zero, 0x0200
        sw t0, 0x1124(a1)             # timer 2: the system clock / 8
# stream A at 80120000h: 96 words of FE000400h, a block of DC 0 then the end code
        lui a2, 0x8012
        lui t1, 0xFE00
        ori t1, t1, 0x0400
        addiu t2, zero, 96
genA:   sw t1, 0(a2)
        addiu t2, t2, -1
        bne t2, zero, genA
        addiu a2, a2, 4
# stream B at 80121000h: 96 blocks of 32 words, DC 0 then 63 AC values of 1 with no run
        lui a2, 0x8012
        ori a2, a2, 0x1000
        addiu t3, zero, 96
genB:   lui t1, 0x0001
        ori t1, t1, 0x0400
        sw t1, 0(a2)
        addiu a2, a2, 4
        lui t1, 0x0001
        ori t1, t1, 0x0001
        addiu t2, zero, 31
genB1:  sw t1, 0(a2)
        addiu t2, t2, -1
        bne t2, zero, genB1
        addiu a2, a2, 4
        addiu t3, t3, -1
        bne t3, zero, genB
        nop
# the five cases: command, stream, input blocks of 32 words, output blocks
        lui a0, 0x3800
        ori a0, a0, 96
        lui a2, 0x8012
        addiu a3, zero, 3
        jal case
        addiu t9, zero, 64
        lui a0, 0x3800
        ori a0, a0, 3072
        lui a2, 0x8012
        ori a2, a2, 0x1000
        addiu a3, zero, 96
        jal case
        addiu t9, zero, 64
        lui a0, 0x2800
        ori a0, a0, 96
        lui a2, 0x8012
        addiu a3, zero, 3
        jal case
        addiu t9, zero, 48
        lui a0, 0x3000
        ori a0, a0, 96
        lui a2, 0x8012
        addiu a3, zero, 3
        jal case
        addiu t9, zero, 96
        lui a0, 0x2000
        ori a0, a0, 3072
        lui a2, 0x8012
        ori a2, a2, 0x1000
        addiu a3, zero, 96
        jal case
        addiu t9, zero, 24
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        lui t2, 0x8010
        sw t1, -16(t2)
done:   j done
        nop
# one case: a0 the command, a2 the stream, a3 its blocks, t9 the output's blocks; two ticks to s0, which advances
case:   or s1, ra, zero
        lui t0, 0x8000
        sw t0, 0x1824(a1)             # MDEC1: reset
        lui t0, 0x6000
        sw t0, 0x1824(a1)             # both requests enabled
        lui t0, 0x4000
        ori t0, t0, 1
        sw t0, 0x1820(a1)             # MDEC(2), both tables
        lui t1, 0x0202
        ori t1, t1, 0x0202
        addiu t2, zero, 32
quant:  sw t1, 0x1820(a1)
        addiu t2, t2, -1
        bne t2, zero, quant
        nop
        jal ready
        nop
        lui t0, 0x6000
        sw t0, 0x1820(a1)             # MDEC(3)
        lui t1, 0x5A82
        ori t1, t1, 0x5A82
        addiu t2, zero, 32
scale:  sw t1, 0x1820(a1)
        addiu t2, t2, -1
        bne t2, zero, scale
        nop
        jal ready
        nop
        sw a0, 0x1820(a1)             # the decode command
        lui t0, 0x8014
        sw t0, 0x1090(a1)             # DMA1: 80140000h
        sll t0, t9, 16
        ori t0, t0, 32
        sw t0, 0x1094(a1)
        lui t0, 0x0100
        ori t0, t0, 0x0200
        sw t0, 0x1098(a1)             # mode 1, to RAM, start
        sw a2, 0x1080(a1)             # DMA0: the stream
        sll t0, a3, 16
        ori t0, t0, 32
        sw t0, 0x1084(a1)
        lui t3, 0x0100
        ori t3, t3, 0x0201
        sw zero, 0x1120(a1)           # timer 2 from 0
        sw t3, 0x1088(a1)             # mode 1, from RAM, start
        ori t4, zero, 0xFFFF          # DMA0's tick until seen
        lui t5, 0x0004                # a bound on the polls
        lui t6, 0x0100
poll:   lw t0, 0x1088(a1)
        lw t1, 0x1098(a1)
        lhu t2, 0x1120(a1)
        and t0, t0, t6
        bne t0, zero, out1
        nop
        ori t7, zero, 0xFFFF
        beq t4, t7, set0
        nop
        j out1
        nop
set0:   or t4, t2, zero
out1:   and t1, t1, t6
        beq t1, zero, end
        addiu t5, t5, -1
        bne t5, zero, poll
        nop
        ori t2, zero, 0xFFFF
end:    sw t4, 0(s0)
        sw t2, 4(s0)
        jr s1
        addiu s0, s0, 8
# waits for MDEC1's busy bit to clear
ready:  lw t0, 0x1824(a1)
        lui t1, 0x2000
        and t0, t0, t1
        bne t0, zero, ready
        nop
        jr ra
        nop
