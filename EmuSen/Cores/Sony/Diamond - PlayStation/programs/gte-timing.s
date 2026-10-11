# The GTE's timing and its hazards (BlackMoon_GTE.md §6, §8). Table A from 80100000h: each of the 22 commands followed
# at once by MFC2, the cycles over a NOP and MFC2, by timer 2 on the system clock from the I-cache with interrupts
# off, each timing run twice and its second pass kept, the GTE drained before each. Table B from 80100100h: RTPS, N
# NOPs and MFC2 against the NOPs and MFC2, N = 0..20. Table C from 80100200h: RTPS followed at once by CFC2, SWC2,
# MTC2, CTC2, LWC2, RTPS and NCLIP, each against a NOP in RTPS's place. Table E from 80100280h: six numbers psx-spx
# lists as N/A, as table A. Table D from 80100300h, values not cycles:
# IR3 then IR1 read N = 0..3 NOPs after IRGB is written, LZCR read N = 0..3 NOPs after LZCS is written, and SZ3 after
# RTPS issued N = 0..2 NOPs after MTC2 or LWC2 writes VZ0. 800FFFF0h holds D1A0D00Eh when done. Written in the shape
# every case shares, one block a case.
        j start
        nop
start:  lui t0, 0x4000
        mtc0 t0, SR
        lui s0, 0x1F80
        sw zero, 0x1124(s0)
        lui s5, 0x8010
        lui s6, 0x1F80
        sw zero, 0x0(s6)
        # A0: rtps, then MFC2
        addiu t6, zero, 2
t1:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t1
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t2:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t2
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x0(s5)
        # A1: nclip, then MFC2
        addiu t6, zero, 2
t3:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nclip
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t3
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t4:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t4
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x4(s5)
        # A2: op, then MFC2
        addiu t6, zero, 2
t5:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        op
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t5
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t6:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t6
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x8(s5)
        # A3: dpcs, then MFC2
        addiu t6, zero, 2
t7:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        dpcs
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t7
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t8:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t8
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0xC(s5)
        # A4: intpl, then MFC2
        addiu t6, zero, 2
t9:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        intpl
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t9
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t10:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t10
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x10(s5)
        # A5: mvmva, then MFC2
        addiu t6, zero, 2
t11:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        mvmva
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t11
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t12:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t12
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x14(s5)
        # A6: ncds, then MFC2
        addiu t6, zero, 2
t13:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        ncds
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t13
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t14:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t14
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x18(s5)
        # A7: cdp, then MFC2
        addiu t6, zero, 2
t15:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        cdp
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t15
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t16:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t16
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x1C(s5)
        # A8: ncdt, then MFC2
        addiu t6, zero, 2
t17:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        ncdt
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t17
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t18:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t18
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x20(s5)
        # A9: nccs, then MFC2
        addiu t6, zero, 2
t19:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nccs
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t19
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t20:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t20
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x24(s5)
        # A10: cc, then MFC2
        addiu t6, zero, 2
t21:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        cc
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t21
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t22:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t22
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x28(s5)
        # A11: ncs, then MFC2
        addiu t6, zero, 2
t23:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        ncs
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t23
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t24:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t24
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x2C(s5)
        # A12: nct, then MFC2
        addiu t6, zero, 2
t25:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nct
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t25
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t26:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t26
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x30(s5)
        # A13: sqr, then MFC2
        addiu t6, zero, 2
t27:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        sqr
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t27
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t28:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t28
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x34(s5)
        # A14: dcpl, then MFC2
        addiu t6, zero, 2
t29:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        dcpl
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t29
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t30:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t30
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x38(s5)
        # A15: dpct, then MFC2
        addiu t6, zero, 2
t31:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        dpct
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t31
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t32:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t32
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x3C(s5)
        # A16: avsz3, then MFC2
        addiu t6, zero, 2
t33:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        avsz3
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t33
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t34:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t34
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x40(s5)
        # A17: avsz4, then MFC2
        addiu t6, zero, 2
t35:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        avsz4
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t35
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t36:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t36
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x44(s5)
        # A18: rtpt, then MFC2
        addiu t6, zero, 2
t37:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtpt
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t37
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t38:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t38
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x48(s5)
        # A19: gpf, then MFC2
        addiu t6, zero, 2
t39:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        gpf
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t39
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t40:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t40
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x4C(s5)
        # A20: gpl, then MFC2
        addiu t6, zero, 2
t41:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        gpl
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t41
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t42:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t42
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x50(s5)
        # A21: ncct, then MFC2
        addiu t6, zero, 2
t43:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        ncct
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t43
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t44:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t44
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x54(s5)
        # B0: rtps, 0 NOPs, MFC2
        addiu t6, zero, 2
t45:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t45
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t46:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t46
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x100(s5)
        # B1: rtps, 1 NOPs, MFC2
        addiu t6, zero, 2
t47:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t47
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t48:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t48
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x104(s5)
        # B2: rtps, 2 NOPs, MFC2
        addiu t6, zero, 2
t49:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t49
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t50:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t50
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x108(s5)
        # B3: rtps, 3 NOPs, MFC2
        addiu t6, zero, 2
t51:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        nop
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t51
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t52:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        nop
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t52
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x10C(s5)
        # B4: rtps, 4 NOPs, MFC2
        addiu t6, zero, 2
t53:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        nop
        nop
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t53
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t54:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        nop
        nop
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t54
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x110(s5)
        # B5: rtps, 5 NOPs, MFC2
        addiu t6, zero, 2
t55:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        nop
        nop
        nop
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t55
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t56:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        nop
        nop
        nop
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t56
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x114(s5)
        # B6: rtps, 6 NOPs, MFC2
        addiu t6, zero, 2
t57:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        nop
        nop
        nop
        nop
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t57
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t58:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t58
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x118(s5)
        # B7: rtps, 7 NOPs, MFC2
        addiu t6, zero, 2
t59:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t59
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t60:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t60
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x11C(s5)
        # B8: rtps, 8 NOPs, MFC2
        addiu t6, zero, 2
t61:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t61
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t62:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t62
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x120(s5)
        # B9: rtps, 9 NOPs, MFC2
        addiu t6, zero, 2
t63:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t63
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t64:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t64
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x124(s5)
        # B10: rtps, 10 NOPs, MFC2
        addiu t6, zero, 2
t65:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t65
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t66:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t66
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x128(s5)
        # B11: rtps, 11 NOPs, MFC2
        addiu t6, zero, 2
t67:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t67
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t68:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t68
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x12C(s5)
        # B12: rtps, 12 NOPs, MFC2
        addiu t6, zero, 2
t69:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t69
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t70:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t70
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x130(s5)
        # B13: rtps, 13 NOPs, MFC2
        addiu t6, zero, 2
t71:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t71
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t72:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t72
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x134(s5)
        # B14: rtps, 14 NOPs, MFC2
        addiu t6, zero, 2
t73:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t73
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t74:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t74
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x138(s5)
        # B15: rtps, 15 NOPs, MFC2
        addiu t6, zero, 2
t75:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t75
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t76:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t76
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x13C(s5)
        # B16: rtps, 16 NOPs, MFC2
        addiu t6, zero, 2
t77:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t77
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t78:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t78
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x140(s5)
        # B17: rtps, 17 NOPs, MFC2
        addiu t6, zero, 2
t79:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t79
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t80:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t80
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x144(s5)
        # B18: rtps, 18 NOPs, MFC2
        addiu t6, zero, 2
t81:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t81
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t82:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t82
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x148(s5)
        # B19: rtps, 19 NOPs, MFC2
        addiu t6, zero, 2
t83:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t83
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t84:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t84
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x14C(s5)
        # B20: rtps, 20 NOPs, MFC2
        addiu t6, zero, 2
t85:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t85
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t86:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
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
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t86
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x150(s5)
        # C0: rtps, then cfc2 t9, FLAG
        addiu t6, zero, 2
t87:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        cfc2 t9, FLAG
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t87
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t88:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        cfc2 t9, FLAG
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t88
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x200(s5)
        # C1: rtps, then swc2 VXY0, 0x0(s6)
        addiu t6, zero, 2
t89:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        swc2 VXY0, 0x0(s6)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t89
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t90:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        swc2 VXY0, 0x0(s6)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t90
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x204(s5)
        # C2: rtps, then mtc2 zero, VXY0
        addiu t6, zero, 2
t91:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        mtc2 zero, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t91
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t92:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mtc2 zero, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t92
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x208(s5)
        # C3: rtps, then ctc2 zero, TRX
        addiu t6, zero, 2
t93:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        ctc2 zero, TRX
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t93
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t94:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        ctc2 zero, TRX
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t94
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x20C(s5)
        # C4: rtps, then lwc2 VXY0, 0x0(s6)
        addiu t6, zero, 2
t95:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        lwc2 VXY0, 0x0(s6)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t95
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t96:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        lwc2 VXY0, 0x0(s6)
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t96
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x210(s5)
        # C5: rtps, then rtps
        addiu t6, zero, 2
t97:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        rtps
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t97
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t98:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        rtps
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t98
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x214(s5)
        # C6: rtps, then nclip
        addiu t6, zero, 2
t99:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        rtps
        nclip
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t99
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t100:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        nclip
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t100
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x218(s5)
        # E0: number 00h, then MFC2
        addiu t6, zero, 2
t101:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        .word 0x4A000000
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t101
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t102:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t102
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x280(s5)
        # E1: number 02h, then MFC2
        addiu t6, zero, 2
t103:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        .word 0x4A000002
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t103
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t104:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t104
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x284(s5)
        # E2: number 15h, then MFC2
        addiu t6, zero, 2
t105:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        .word 0x4A000015
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t105
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t106:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t106
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x288(s5)
        # E3: number 1Fh, then MFC2
        addiu t6, zero, 2
t107:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        .word 0x4A00001F
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t107
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t108:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t108
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x28C(s5)
        # E4: number 2Fh, then MFC2
        addiu t6, zero, 2
t109:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        .word 0x4A00002F
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t109
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t110:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t110
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x290(s5)
        # E5: number 3Ch, then MFC2
        addiu t6, zero, 2
t111:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        .word 0x4A00003C
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t111
        subu t7, t7, t8
        or s1, t7, zero
        addiu t6, zero, 2
t112:      mfc2 t9, VXY0
        nop
        lw t8, 0x1120(s0)
        nop
        mfc2 t9, VXY0
        lw t7, 0x1120(s0)
        addiu t6, t6, -1
        bne t6, zero, t112
        subu t7, t7, t8
        subu s1, s1, t7
        sw s1, 0x294(s5)
        lui a0, 0x0000
        ori a0, a0, 0x7FFF
        # D: IR3 0 NOPs after IRGB
        mtc2 zero, IR1
        mtc2 zero, IR2
        mtc2 zero, IR3
        nop
        nop
        nop
        nop
        mtc2 a0, IRGB
        mfc2 t9, IR3
        nop
        sw t9, 0x300(s5)
        # D: IR1 0 NOPs after IRGB
        mtc2 zero, IR1
        mtc2 zero, IR2
        mtc2 zero, IR3
        nop
        nop
        nop
        nop
        mtc2 a0, IRGB
        mfc2 t9, IR1
        nop
        sw t9, 0x304(s5)
        # D: IR3 1 NOPs after IRGB
        mtc2 zero, IR1
        mtc2 zero, IR2
        mtc2 zero, IR3
        nop
        nop
        nop
        nop
        mtc2 a0, IRGB
        nop
        mfc2 t9, IR3
        nop
        sw t9, 0x308(s5)
        # D: IR1 1 NOPs after IRGB
        mtc2 zero, IR1
        mtc2 zero, IR2
        mtc2 zero, IR3
        nop
        nop
        nop
        nop
        mtc2 a0, IRGB
        nop
        mfc2 t9, IR1
        nop
        sw t9, 0x30C(s5)
        # D: IR3 2 NOPs after IRGB
        mtc2 zero, IR1
        mtc2 zero, IR2
        mtc2 zero, IR3
        nop
        nop
        nop
        nop
        mtc2 a0, IRGB
        nop
        nop
        mfc2 t9, IR3
        nop
        sw t9, 0x310(s5)
        # D: IR1 2 NOPs after IRGB
        mtc2 zero, IR1
        mtc2 zero, IR2
        mtc2 zero, IR3
        nop
        nop
        nop
        nop
        mtc2 a0, IRGB
        nop
        nop
        mfc2 t9, IR1
        nop
        sw t9, 0x314(s5)
        # D: IR3 3 NOPs after IRGB
        mtc2 zero, IR1
        mtc2 zero, IR2
        mtc2 zero, IR3
        nop
        nop
        nop
        nop
        mtc2 a0, IRGB
        nop
        nop
        nop
        mfc2 t9, IR3
        nop
        sw t9, 0x318(s5)
        # D: IR1 3 NOPs after IRGB
        mtc2 zero, IR1
        mtc2 zero, IR2
        mtc2 zero, IR3
        nop
        nop
        nop
        nop
        mtc2 a0, IRGB
        nop
        nop
        nop
        mfc2 t9, IR1
        nop
        sw t9, 0x31C(s5)
        lui a1, 0x0001
        # D: LZCR 0 NOPs after LZCS
        mtc2 zero, LZCS
        nop
        nop
        nop
        nop
        mtc2 a1, LZCS
        mfc2 t9, LZCR
        nop
        sw t9, 0x320(s5)
        # D: LZCR 1 NOPs after LZCS
        mtc2 zero, LZCS
        nop
        nop
        nop
        nop
        mtc2 a1, LZCS
        nop
        mfc2 t9, LZCR
        nop
        sw t9, 0x324(s5)
        # D: LZCR 2 NOPs after LZCS
        mtc2 zero, LZCS
        nop
        nop
        nop
        nop
        mtc2 a1, LZCS
        nop
        nop
        mfc2 t9, LZCR
        nop
        sw t9, 0x328(s5)
        # D: LZCR 3 NOPs after LZCS
        mtc2 zero, LZCS
        nop
        nop
        nop
        nop
        mtc2 a1, LZCS
        nop
        nop
        nop
        mfc2 t9, LZCR
        nop
        sw t9, 0x32C(s5)
        lui t0, 0x0000
        ori t0, t0, 0x1000
        ctc2 t0, RT11RT12
        ctc2 zero, RT13RT21
        ctc2 t0, RT22RT23
        ctc2 zero, RT31RT32
        ctc2 t0, RT33
        ctc2 zero, TRX
        ctc2 zero, TRY
        ctc2 zero, TRZ
        mtc2 zero, VXY0
        addiu a2, zero, 0x100
        sw a2, 0x4(s6)
        # D: SZ3 after RTPS 0 NOPs after mtc2 a2, VZ0
        mtc2 zero, VZ0
        nop
        nop
        nop
        nop
        mtc2 a2, VZ0
        rtps
        mfc2 t9, SZ3
        nop
        sw t9, 0x330(s5)
        # D: SZ3 after RTPS 1 NOPs after mtc2 a2, VZ0
        mtc2 zero, VZ0
        nop
        nop
        nop
        nop
        mtc2 a2, VZ0
        nop
        rtps
        mfc2 t9, SZ3
        nop
        sw t9, 0x334(s5)
        # D: SZ3 after RTPS 2 NOPs after mtc2 a2, VZ0
        mtc2 zero, VZ0
        nop
        nop
        nop
        nop
        mtc2 a2, VZ0
        nop
        nop
        rtps
        mfc2 t9, SZ3
        nop
        sw t9, 0x338(s5)
        # D: SZ3 after RTPS 0 NOPs after lwc2 VZ0, 0x4(s6)
        mtc2 zero, VZ0
        nop
        nop
        nop
        nop
        lwc2 VZ0, 0x4(s6)
        rtps
        mfc2 t9, SZ3
        nop
        sw t9, 0x33C(s5)
        # D: SZ3 after RTPS 1 NOPs after lwc2 VZ0, 0x4(s6)
        mtc2 zero, VZ0
        nop
        nop
        nop
        nop
        lwc2 VZ0, 0x4(s6)
        nop
        rtps
        mfc2 t9, SZ3
        nop
        sw t9, 0x340(s5)
        # D: SZ3 after RTPS 2 NOPs after lwc2 VZ0, 0x4(s6)
        mtc2 zero, VZ0
        nop
        nop
        nop
        nop
        lwc2 VZ0, 0x4(s6)
        nop
        nop
        rtps
        mfc2 t9, SZ3
        nop
        sw t9, 0x344(s5)
        lui t0, 0xD1A0
        ori t0, t0, 0xD00E
        lui t1, 0x8010
        sw t0, -0x10(t1)
done:   j done
        nop
