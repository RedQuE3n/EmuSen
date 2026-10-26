# N command 0Ch's parameters swept (Sapphire_Native.md §39, D-461): for each set of seven bytes, the error and the drive's
# registers 20h-3Fh recorded, 33 words a set.
        lui a0, 0x0020
        lui s0, 0xBF40
        ori s0, s0, 0x2000
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w0:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d0
        nop
        addiu t2, t2, -1
        bne t2, zero, w0
        nop
d0:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 1
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w1:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d1
        nop
        addiu t2, t2, -1
        bne t2, zero, w1
        nop
d1:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 2
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w2:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d2
        nop
        addiu t2, t2, -1
        bne t2, zero, w2
        nop
d2:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 3
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w3:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d3
        nop
        addiu t2, t2, -1
        bne t2, zero, w3
        nop
d3:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 4
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w4:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d4
        nop
        addiu t2, t2, -1
        bne t2, zero, w4
        nop
d4:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 8
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w5:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d5
        nop
        addiu t2, t2, -1
        bne t2, zero, w5
        nop
d5:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 9
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w6:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d6
        nop
        addiu t2, t2, -1
        bne t2, zero, w6
        nop
d6:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 11
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w7:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d7
        nop
        addiu t2, t2, -1
        bne t2, zero, w7
        nop
d7:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 64
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w8:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d8
        nop
        addiu t2, t2, -1
        bne t2, zero, w8
        nop
d8:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 65
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w9:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d9
        nop
        addiu t2, t2, -1
        bne t2, zero, w9
        nop
d9:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 67
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w10:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d10
        nop
        addiu t2, t2, -1
        bne t2, zero, w10
        nop
d10:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 72
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w11:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d11
        nop
        addiu t2, t2, -1
        bne t2, zero, w11
        nop
d11:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 74
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w12:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d12
        nop
        addiu t2, t2, -1
        bne t2, zero, w12
        nop
d12:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 75
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w13:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d13
        nop
        addiu t2, t2, -1
        bne t2, zero, w13
        nop
d13:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 3
        sb t0, 5(s0)
        addiu t0, zero, 1
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w14:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d14
        nop
        addiu t2, t2, -1
        bne t2, zero, w14
        nop
d14:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 3
        sb t0, 5(s0)
        addiu t0, zero, 4
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w15:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d15
        nop
        addiu t2, t2, -1
        bne t2, zero, w15
        nop
d15:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 3
        sb t0, 5(s0)
        addiu t0, zero, 12
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w16:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d16
        nop
        addiu t2, t2, -1
        bne t2, zero, w16
        nop
d16:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 3
        sb t0, 5(s0)
        addiu t0, zero, 13
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w17:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d17
        nop
        addiu t2, t2, -1
        bne t2, zero, w17
        nop
d17:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 1
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 75
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w18:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d18
        nop
        addiu t2, t2, -1
        bne t2, zero, w18
        nop
d18:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 1
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 75
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w19:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d19
        nop
        addiu t2, t2, -1
        bne t2, zero, w19
        nop
d19:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 1
        sb t0, 5(s0)
        addiu t0, zero, 75
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 12
        sb t1, 4(s0)
        lui t2, 0x0400
w20:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d20
        nop
        addiu t2, t2, -1
        bne t2, zero, w20
        nop
d20:     lbu t0, 6(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 128(a0)
        addiu a0, a0, 132
        lui a0, 0x0020
        lui t0, 0x5A99
        ori t0, t0, 0xD00E
        sw t0, -16(a0)
end:    beq zero, zero, end
        nop
