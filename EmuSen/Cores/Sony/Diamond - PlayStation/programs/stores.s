# D-5: what RAM stores cost (Diamond_Disputes.md D-5). Timed by timer 2 on the system clock from the I-cache with
# interrupts off, each sequence looped twice and its second pass kept, less the two timer reads alone: N stores to
# RAM (A), N stores and a load of another RAM word (B), and N stores and eight NOPs (C), for N = 1 to 8. The
# closing timer read is itself a load. Results from 80100000h: A's eight words, B's from 80100020h, C's from
# 80100040h. 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui s0, 0x1F80
        sw zero, 0x1124(s0)
        lui s5, 0x8010
        lui s6, 0x8018                # the stored words
        lui s7, 0x8019                # the loaded word
        addiu t6, zero, 2
b1:     lw t8, 0x1120(s0)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, b1
        subu s4, t7, t8               # the baseline
        addiu t6, zero, 2               # A, N = 1
l1:      lw t8, 0x1120(s0)
        sw t6, 0(s6)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l1
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 0(s5)
        addiu t6, zero, 2               # A, N = 2
l2:      lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l2
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 4(s5)
        addiu t6, zero, 2               # A, N = 3
l3:      lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l3
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 8(s5)
        addiu t6, zero, 2               # A, N = 4
l4:      lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l4
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 12(s5)
        addiu t6, zero, 2               # A, N = 5
l5:      lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        sw t6, 16(s6)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l5
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 16(s5)
        addiu t6, zero, 2               # A, N = 6
l6:      lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        sw t6, 16(s6)
        sw t6, 20(s6)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l6
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 20(s5)
        addiu t6, zero, 2               # A, N = 7
l7:      lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        sw t6, 16(s6)
        sw t6, 20(s6)
        sw t6, 24(s6)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l7
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 24(s5)
        addiu t6, zero, 2               # A, N = 8
l8:      lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        sw t6, 16(s6)
        sw t6, 20(s6)
        sw t6, 24(s6)
        sw t6, 28(s6)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l8
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 28(s5)
        addiu t6, zero, 2               # B, N = 1
l9:      lw t8, 0x1120(s0)
        sw t6, 0(s6)
        lw t9, 0(s7)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l9
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 32(s5)
        addiu t6, zero, 2               # B, N = 2
l10:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        lw t9, 0(s7)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l10
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 36(s5)
        addiu t6, zero, 2               # B, N = 3
l11:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        lw t9, 0(s7)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l11
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 40(s5)
        addiu t6, zero, 2               # B, N = 4
l12:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        lw t9, 0(s7)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l12
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 44(s5)
        addiu t6, zero, 2               # B, N = 5
l13:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        sw t6, 16(s6)
        lw t9, 0(s7)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l13
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 48(s5)
        addiu t6, zero, 2               # B, N = 6
l14:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        sw t6, 16(s6)
        sw t6, 20(s6)
        lw t9, 0(s7)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l14
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 52(s5)
        addiu t6, zero, 2               # B, N = 7
l15:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        sw t6, 16(s6)
        sw t6, 20(s6)
        sw t6, 24(s6)
        lw t9, 0(s7)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l15
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 56(s5)
        addiu t6, zero, 2               # B, N = 8
l16:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        sw t6, 16(s6)
        sw t6, 20(s6)
        sw t6, 24(s6)
        sw t6, 28(s6)
        lw t9, 0(s7)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l16
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 60(s5)
        addiu t6, zero, 2               # C, N = 1
l17:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l17
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 64(s5)
        addiu t6, zero, 2               # C, N = 2
l18:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l18
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 68(s5)
        addiu t6, zero, 2               # C, N = 3
l19:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l19
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 72(s5)
        addiu t6, zero, 2               # C, N = 4
l20:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l20
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 76(s5)
        addiu t6, zero, 2               # C, N = 5
l21:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        sw t6, 16(s6)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l21
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 80(s5)
        addiu t6, zero, 2               # C, N = 6
l22:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        sw t6, 16(s6)
        sw t6, 20(s6)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l22
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 84(s5)
        addiu t6, zero, 2               # C, N = 7
l23:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        sw t6, 16(s6)
        sw t6, 20(s6)
        sw t6, 24(s6)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l23
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 88(s5)
        addiu t6, zero, 2               # C, N = 8
l24:     lw t8, 0x1120(s0)
        sw t6, 0(s6)
        sw t6, 4(s6)
        sw t6, 8(s6)
        sw t6, 12(s6)
        sw t6, 16(s6)
        sw t6, 20(s6)
        sw t6, 24(s6)
        sw t6, 28(s6)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, l24
        subu t7, t7, t8
        subu t7, t7, s4
        sw t7, 92(s5)
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(s5)
done:   j done
        nop
