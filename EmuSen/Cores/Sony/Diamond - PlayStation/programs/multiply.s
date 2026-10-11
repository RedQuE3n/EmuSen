# D-2: the multiplier's latency (BlackMoon_R3000.md §8). Each case times MULT, MULTU, DIV or DIVU of its rs and rt
# followed at once by MFLO, against MFLO alone, by timer 2 on the system clock, with interrupts off; each timing
# loops twice and keeps the second pass, run from the I-cache. A second table times MULTU of a small rs followed
# by N NOPs and MFLO, N = 0..15, against the N NOPs and MFLO alone. Results from 80100000h, one word a case, the
# cycles over the baseline; the NOP table from 80100100h. 800FFFF0h holds D1A0D00Eh when done. Written in the
# shape every case shares, one block a case.
        j start
        nop
start:  mtc0 zero, SR
        lui s0, 0x1F80
        sw zero, 0x1124(s0)
        lui s5, 0x8010
        lui a0, 0x0000          # case 0: multu 00000000h, 00000003h
        ori a0, a0, 0x0000
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t1:      lw t8, 0x1120(s0)
        multu a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t1
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t2:      lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t2
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x0(s5)
        lui a0, 0x0000          # case 1: multu 000007FFh, 00000003h
        ori a0, a0, 0x07FF
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t3:      lw t8, 0x1120(s0)
        multu a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t3
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t4:      lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t4
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x4(s5)
        lui a0, 0x0000          # case 2: multu 00000800h, 00000003h
        ori a0, a0, 0x0800
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t5:      lw t8, 0x1120(s0)
        multu a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t5
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t6:      lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t6
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x8(s5)
        lui a0, 0x000F          # case 3: multu 000FFFFFh, 00000003h
        ori a0, a0, 0xFFFF
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t7:      lw t8, 0x1120(s0)
        multu a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t7
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t8:      lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t8
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0xC(s5)
        lui a0, 0x0010          # case 4: multu 00100000h, 00000003h
        ori a0, a0, 0x0000
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t9:      lw t8, 0x1120(s0)
        multu a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t9
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t10:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t10
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x10(s5)
        lui a0, 0xFFFF          # case 5: multu FFFFFFFFh, 00000003h
        ori a0, a0, 0xFFFF
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t11:     lw t8, 0x1120(s0)
        multu a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t11
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t12:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t12
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x14(s5)
        lui a0, 0x0000          # case 6: mult 000007FFh, 00000003h
        ori a0, a0, 0x07FF
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t13:     lw t8, 0x1120(s0)
        mult a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t13
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t14:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t14
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x18(s5)
        lui a0, 0x0000          # case 7: mult 00000800h, 00000003h
        ori a0, a0, 0x0800
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t15:     lw t8, 0x1120(s0)
        mult a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t15
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t16:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t16
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x1C(s5)
        lui a0, 0xFFFF          # case 8: mult FFFFF800h, 00000003h
        ori a0, a0, 0xF800
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t17:     lw t8, 0x1120(s0)
        mult a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t17
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t18:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t18
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x20(s5)
        lui a0, 0xFFFF          # case 9: mult FFFFF7FFh, 00000003h
        ori a0, a0, 0xF7FF
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t19:     lw t8, 0x1120(s0)
        mult a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t19
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t20:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t20
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x24(s5)
        lui a0, 0x000F          # case 10: mult 000FFFFFh, 00000003h
        ori a0, a0, 0xFFFF
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t21:     lw t8, 0x1120(s0)
        mult a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t21
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t22:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t22
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x28(s5)
        lui a0, 0x0010          # case 11: mult 00100000h, 00000003h
        ori a0, a0, 0x0000
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t23:     lw t8, 0x1120(s0)
        mult a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t23
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t24:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t24
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x2C(s5)
        lui a0, 0xFFF0          # case 12: mult FFF00000h, 00000003h
        ori a0, a0, 0x0000
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t25:     lw t8, 0x1120(s0)
        mult a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t25
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t26:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t26
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x30(s5)
        lui a0, 0xFFEF          # case 13: mult FFEFFFFFh, 00000003h
        ori a0, a0, 0xFFFF
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t27:     lw t8, 0x1120(s0)
        mult a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t27
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t28:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t28
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x34(s5)
        lui a0, 0x8000          # case 14: mult 80000000h, 00000003h
        ori a0, a0, 0x0000
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t29:     lw t8, 0x1120(s0)
        mult a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t29
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t30:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t30
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x38(s5)
        lui a0, 0x0000          # case 15: multu 00000001h, FFFFFFFFh
        ori a0, a0, 0x0001
        lui a1, 0xFFFF
        ori a1, a1, 0xFFFF
        addiu t6, zero, 2
t31:     lw t8, 0x1120(s0)
        multu a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t31
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t32:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t32
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x3C(s5)
        lui a0, 0x0000          # case 16: mult 00000003h, 80000000h
        ori a0, a0, 0x0003
        lui a1, 0x8000
        ori a1, a1, 0x0000
        addiu t6, zero, 2
t33:     lw t8, 0x1120(s0)
        mult a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t33
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t34:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t34
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x40(s5)
        lui a0, 0xFFFF          # case 17: divu FFFFFFFFh, 00000003h
        ori a0, a0, 0xFFFF
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t35:     lw t8, 0x1120(s0)
        divu a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t35
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t36:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t36
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x44(s5)
        lui a0, 0x0000          # case 18: divu 00000001h, 00000003h
        ori a0, a0, 0x0001
        lui a1, 0x0000
        ori a1, a1, 0x0003
        addiu t6, zero, 2
t37:     lw t8, 0x1120(s0)
        divu a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t37
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t38:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t38
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x48(s5)
        lui a0, 0x8000          # case 19: div 80000000h, FFFFFFFFh
        ori a0, a0, 0x0000
        lui a1, 0xFFFF
        ori a1, a1, 0xFFFF
        addiu t6, zero, 2
t39:     lw t8, 0x1120(s0)
        div a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t39
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t40:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t40
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x4C(s5)
        lui a0, 0x0000          # case 20: div 00000005h, 00000000h
        ori a0, a0, 0x0005
        lui a1, 0x0000
        ori a1, a1, 0x0000
        addiu t6, zero, 2
t41:     lw t8, 0x1120(s0)
        div a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t41
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t42:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t42
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x50(s5)
        lui a0, 0x0000          # case 21: divu 00000007h, 80000000h
        ori a0, a0, 0x0007
        lui a1, 0x8000
        ori a1, a1, 0x0000
        addiu t6, zero, 2
t43:     lw t8, 0x1120(s0)
        divu a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t43
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t44:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t44
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x54(s5)
        addiu a0, zero, 3
        addiu a1, zero, 3
        nop                       # NOP case 0
        addiu t6, zero, 2
t45:     lw t8, 0x1120(s0)
        multu a0, a1
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t45
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t46:     lw t8, 0x1120(s0)
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t46
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x100(s5)
        nop                       # NOP case 1
        addiu t6, zero, 2
t47:     lw t8, 0x1120(s0)
        multu a0, a1
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t47
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t48:     lw t8, 0x1120(s0)
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t48
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x104(s5)
        nop                       # NOP case 2
        addiu t6, zero, 2
t49:     lw t8, 0x1120(s0)
        multu a0, a1
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t49
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t50:     lw t8, 0x1120(s0)
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t50
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x108(s5)
        nop                       # NOP case 3
        addiu t6, zero, 2
t51:     lw t8, 0x1120(s0)
        multu a0, a1
        nop
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t51
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t52:     lw t8, 0x1120(s0)
        nop
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t52
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x10C(s5)
        nop                       # NOP case 4
        addiu t6, zero, 2
t53:     lw t8, 0x1120(s0)
        multu a0, a1
        nop
        nop
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t53
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t54:     lw t8, 0x1120(s0)
        nop
        nop
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t54
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x110(s5)
        nop                       # NOP case 5
        addiu t6, zero, 2
t55:     lw t8, 0x1120(s0)
        multu a0, a1
        nop
        nop
        nop
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t55
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t56:     lw t8, 0x1120(s0)
        nop
        nop
        nop
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t56
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x114(s5)
        nop                       # NOP case 6
        addiu t6, zero, 2
t57:     lw t8, 0x1120(s0)
        multu a0, a1
        nop
        nop
        nop
        nop
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t57
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t58:     lw t8, 0x1120(s0)
        nop
        nop
        nop
        nop
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t58
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x118(s5)
        nop                       # NOP case 7
        addiu t6, zero, 2
t59:     lw t8, 0x1120(s0)
        multu a0, a1
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t59
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t60:     lw t8, 0x1120(s0)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t60
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x11C(s5)
        nop                       # NOP case 8
        addiu t6, zero, 2
t61:     lw t8, 0x1120(s0)
        multu a0, a1
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t61
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t62:     lw t8, 0x1120(s0)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t62
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x120(s5)
        nop                       # NOP case 9
        addiu t6, zero, 2
t63:     lw t8, 0x1120(s0)
        multu a0, a1
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t63
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t64:     lw t8, 0x1120(s0)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t64
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x124(s5)
        nop                       # NOP case 10
        addiu t6, zero, 2
t65:     lw t8, 0x1120(s0)
        multu a0, a1
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
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t65
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t66:     lw t8, 0x1120(s0)
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
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t66
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x128(s5)
        nop                       # NOP case 11
        addiu t6, zero, 2
t67:     lw t8, 0x1120(s0)
        multu a0, a1
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
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t67
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t68:     lw t8, 0x1120(s0)
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
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t68
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x12C(s5)
        nop                       # NOP case 12
        addiu t6, zero, 2
t69:     lw t8, 0x1120(s0)
        multu a0, a1
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
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t69
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t70:     lw t8, 0x1120(s0)
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
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t70
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x130(s5)
        nop                       # NOP case 13
        addiu t6, zero, 2
t71:     lw t8, 0x1120(s0)
        multu a0, a1
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
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t71
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t72:     lw t8, 0x1120(s0)
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
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t72
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x134(s5)
        nop                       # NOP case 14
        addiu t6, zero, 2
t73:     lw t8, 0x1120(s0)
        multu a0, a1
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
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t73
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t74:     lw t8, 0x1120(s0)
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
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t74
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x138(s5)
        nop                       # NOP case 15
        addiu t6, zero, 2
t75:     lw t8, 0x1120(s0)
        multu a0, a1
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
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t75
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t76:     lw t8, 0x1120(s0)
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
        mflo t9
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t76
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x13C(s5)
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(s5)
done:   j done
        nop
