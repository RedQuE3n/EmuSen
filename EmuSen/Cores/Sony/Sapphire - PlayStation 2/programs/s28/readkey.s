# N command 0Ch as the player's BIOS sends it before a disc boot (Sapphire_Native.md §39, D-461): its seven parameter
# bytes, twice, each time the N status, error and I_STAT, then every byte of the drive's registers 00h-3Fh recorded;
# before them the disk type, drive status, N status and 38h; after them a Seek, a NOP and a Stop, each followed by
# the N status, error and 38h.
        lui a0, 0x0020
        lui s0, 0xBF40
        ori s0, s0, 0x2000
        lui t2, 0x0100
r0:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0xC0
        addiu t1, zero, 0x40
        beq t0, t1, r1
        nop
        addiu t2, t2, -1
        bne t2, zero, r0
        nop
r1:     nop
        lbu t0, 15(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 10(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 5(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 12(a0)
        addiu a0, a0, 16
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
w0:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d0
        nop
        addiu t2, t2, -1
        bne t2, zero, w0
        nop
d0:     nop
        lbu t0, 5(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 6(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 8(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 0(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 1(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 2(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 3(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 4(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 5(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 6(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 7(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 8(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 9(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 10(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 11(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 12(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 13(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 14(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 15(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 16(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 17(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 18(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 19(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 20(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 21(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 22(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 23(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 24(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 25(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 26(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 27(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 28(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 29(s0)
        nop
        sw t0, 128(a0)
        lbu t0, 30(s0)
        nop
        sw t0, 132(a0)
        lbu t0, 31(s0)
        nop
        sw t0, 136(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 140(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 144(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 148(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 152(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 156(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 160(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 164(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 168(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 172(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 176(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 180(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 184(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 188(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 192(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 196(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 200(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 204(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 208(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 212(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 216(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 220(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 224(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 228(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 232(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 236(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 240(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 244(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 248(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 252(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 256(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 260(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 264(a0)
        addiu a0, a0, 268
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
w1:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, d1
        nop
        addiu t2, t2, -1
        bne t2, zero, w1
        nop
d1:     nop
        lbu t0, 5(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 6(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 8(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 0(s0)
        nop
        sw t0, 12(a0)
        lbu t0, 1(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 2(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 3(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 4(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 5(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 6(s0)
        nop
        sw t0, 36(a0)
        lbu t0, 7(s0)
        nop
        sw t0, 40(a0)
        lbu t0, 8(s0)
        nop
        sw t0, 44(a0)
        lbu t0, 9(s0)
        nop
        sw t0, 48(a0)
        lbu t0, 10(s0)
        nop
        sw t0, 52(a0)
        lbu t0, 11(s0)
        nop
        sw t0, 56(a0)
        lbu t0, 12(s0)
        nop
        sw t0, 60(a0)
        lbu t0, 13(s0)
        nop
        sw t0, 64(a0)
        lbu t0, 14(s0)
        nop
        sw t0, 68(a0)
        lbu t0, 15(s0)
        nop
        sw t0, 72(a0)
        lbu t0, 16(s0)
        nop
        sw t0, 76(a0)
        lbu t0, 17(s0)
        nop
        sw t0, 80(a0)
        lbu t0, 18(s0)
        nop
        sw t0, 84(a0)
        lbu t0, 19(s0)
        nop
        sw t0, 88(a0)
        lbu t0, 20(s0)
        nop
        sw t0, 92(a0)
        lbu t0, 21(s0)
        nop
        sw t0, 96(a0)
        lbu t0, 22(s0)
        nop
        sw t0, 100(a0)
        lbu t0, 23(s0)
        nop
        sw t0, 104(a0)
        lbu t0, 24(s0)
        nop
        sw t0, 108(a0)
        lbu t0, 25(s0)
        nop
        sw t0, 112(a0)
        lbu t0, 26(s0)
        nop
        sw t0, 116(a0)
        lbu t0, 27(s0)
        nop
        sw t0, 120(a0)
        lbu t0, 28(s0)
        nop
        sw t0, 124(a0)
        lbu t0, 29(s0)
        nop
        sw t0, 128(a0)
        lbu t0, 30(s0)
        nop
        sw t0, 132(a0)
        lbu t0, 31(s0)
        nop
        sw t0, 136(a0)
        lbu t0, 32(s0)
        nop
        sw t0, 140(a0)
        lbu t0, 33(s0)
        nop
        sw t0, 144(a0)
        lbu t0, 34(s0)
        nop
        sw t0, 148(a0)
        lbu t0, 35(s0)
        nop
        sw t0, 152(a0)
        lbu t0, 36(s0)
        nop
        sw t0, 156(a0)
        lbu t0, 37(s0)
        nop
        sw t0, 160(a0)
        lbu t0, 38(s0)
        nop
        sw t0, 164(a0)
        lbu t0, 39(s0)
        nop
        sw t0, 168(a0)
        lbu t0, 40(s0)
        nop
        sw t0, 172(a0)
        lbu t0, 41(s0)
        nop
        sw t0, 176(a0)
        lbu t0, 42(s0)
        nop
        sw t0, 180(a0)
        lbu t0, 43(s0)
        nop
        sw t0, 184(a0)
        lbu t0, 44(s0)
        nop
        sw t0, 188(a0)
        lbu t0, 45(s0)
        nop
        sw t0, 192(a0)
        lbu t0, 46(s0)
        nop
        sw t0, 196(a0)
        lbu t0, 47(s0)
        nop
        sw t0, 200(a0)
        lbu t0, 48(s0)
        nop
        sw t0, 204(a0)
        lbu t0, 49(s0)
        nop
        sw t0, 208(a0)
        lbu t0, 50(s0)
        nop
        sw t0, 212(a0)
        lbu t0, 51(s0)
        nop
        sw t0, 216(a0)
        lbu t0, 52(s0)
        nop
        sw t0, 220(a0)
        lbu t0, 53(s0)
        nop
        sw t0, 224(a0)
        lbu t0, 54(s0)
        nop
        sw t0, 228(a0)
        lbu t0, 55(s0)
        nop
        sw t0, 232(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 236(a0)
        lbu t0, 57(s0)
        nop
        sw t0, 240(a0)
        lbu t0, 58(s0)
        nop
        sw t0, 244(a0)
        lbu t0, 59(s0)
        nop
        sw t0, 248(a0)
        lbu t0, 60(s0)
        nop
        sw t0, 252(a0)
        lbu t0, 61(s0)
        nop
        sw t0, 256(a0)
        lbu t0, 62(s0)
        nop
        sw t0, 260(a0)
        lbu t0, 63(s0)
        nop
        sw t0, 264(a0)
        addiu a0, a0, 268
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t0, zero, 16
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t0, zero, 0
        sb t0, 5(s0)
        addiu t1, zero, 5
        sb t1, 4(s0)
        lui t2, 0x0400
x0:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, y0
        nop
        addiu t2, t2, -1
        bne t2, zero, x0
        nop
y0:     nop
        lbu t0, 5(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 6(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 8(a0)
        addiu a0, a0, 12
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t1, zero, 0
        sb t1, 4(s0)
        lui t2, 0x0400
x1:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, y1
        nop
        addiu t2, t2, -1
        bne t2, zero, x1
        nop
y1:     nop
        lbu t0, 5(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 6(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 8(a0)
        addiu a0, a0, 12
        addiu t0, zero, 255
        sb t0, 8(s0)
        addiu t1, zero, 3
        sb t1, 4(s0)
        lui t2, 0x0400
x2:     lbu t0, 5(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, y2
        nop
        addiu t2, t2, -1
        bne t2, zero, x2
        nop
y2:     nop
        lbu t0, 5(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 6(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 56(s0)
        nop
        sw t0, 8(a0)
        addiu a0, a0, 12
        lui a0, 0x0020
        lui t0, 0x5A99
        ori t0, t0, 0xD00E
        sw t0, -16(a0)
end:    beq zero, zero, end
        nop
