# A fuzz of the GTE in the manner of ps1-tests' gte-fuzz, for the witnesses (BlackMoon_GTE.md §7.3): every command
# 40 times and six numbers psx-spx lists as N/A 4 times each, every register written in order with a value
# drawn as gte-fuzz draws its own (one of ten fixed values, a random word, or its low or high half) from xorshift32,
# seeded 5EED0001h, the command's sf, lm and MVMVA bits drawn too; the command run from an uncached slot it is
# written into; every register read back. From 80100000h, 129 words a case: the command, the 64 values written, the
# 64 read. 800FFFF0h holds D1A0D00Eh when done, 800FFFF4h the count of cases. No kernel is needed.
        j start
        nop
# 80010008h: the fixed values.
        .word 0x00000000
        .word 0xFFFFFFFF
        .word 0x7FFFFFFF
        .word 0x80000000
        .word 0x7FFF7FFF
        .word 0x80008000
        .word 0x55555555
        .word 0xAAAAAAAA
        .word 0x00001000
        .word 0x0000FFFF
# 80010030h: the slot the command is written into, run through KSEG1.
        .word 0x4A000000
        jr ra
        nop
# 8001003Ch: the commands, each a number and a count, ended by a count of zero.
        .word 0x01
        .word 40
        .word 0x06
        .word 40
        .word 0x0C
        .word 40
        .word 0x10
        .word 40
        .word 0x11
        .word 40
        .word 0x12
        .word 40
        .word 0x13
        .word 40
        .word 0x14
        .word 40
        .word 0x16
        .word 40
        .word 0x1B
        .word 40
        .word 0x1C
        .word 40
        .word 0x1E
        .word 40
        .word 0x20
        .word 40
        .word 0x28
        .word 40
        .word 0x29
        .word 40
        .word 0x2A
        .word 40
        .word 0x2D
        .word 40
        .word 0x2E
        .word 40
        .word 0x30
        .word 40
        .word 0x3D
        .word 40
        .word 0x3E
        .word 40
        .word 0x3F
        .word 40
        .word 0x00
        .word 4
        .word 0x02
        .word 4
        .word 0x15
        .word 4
        .word 0x1F
        .word 4
        .word 0x2F
        .word 4
        .word 0x3C
        .word 4
        .word 0
        .word 0
# A value in v0, from s7 (xorshift32), drawn as gte-fuzz's getRandomValue draws one.
value:
        sll t0, s7, 13
        xor s7, s7, t0
        srl t0, s7, 17
        xor s7, s7, t0
        sll t0, s7, 5
        xor s7, s7, t0
        andi t1, s7, 31
        sll t0, s7, 13
        xor s7, s7, t0
        srl t0, s7, 17
        xor s7, s7, t0
        sll t0, s7, 5
        xor s7, s7, t0
        or v0, s7, zero
        sltiu t2, t1, 10
        beq t2, zero, wide
        sll t2, t1, 2
        addu t2, t2, s4
        lw v0, 0x0(t2)
        jr ra
        nop
wide:   sltiu t2, t1, 20
        bne t2, zero, drawn
        sltiu t2, t1, 25
        beq t2, zero, high
        nop
        jr ra
        andi v0, v0, 0xFFFF
high:   sltiu t2, t1, 30
        beq t2, zero, drawn
        lui t2, 0xFFFF
        and v0, v0, t2
drawn:  jr ra
        nop
start:  lui t0, 0x4000
        mtc0 t0, SR
        lui s4, 0x8001
        ori s4, s4, 0x0008
        lui s2, 0x8001
        ori s2, s2, 0x003C
        lui s7, 0x5EED
        ori s7, s7, 0x0001
        lui a1, 0x8010
        or s6, zero, zero
outer:  lw s3, 0x0(s2)
        lw s1, 0x4(s2)
        nop
        beq s1, zero, finish
        nop
inner:
        sll t0, s7, 13
        xor s7, s7, t0
        srl t0, s7, 17
        xor s7, s7, t0
        sll t0, s7, 5
        xor s7, s7, t0
        andi t1, s7, 0x01
        sll t1, t1, 19
        andi t2, s7, 0x02
        sll t2, t2, 9
        andi t3, s7, 0x0C
        sll t3, t3, 11
        andi t4, s7, 0x30
        sll t4, t4, 11
        andi t5, s7, 0xC0
        sll t5, t5, 11
        or s0, s3, t1
        or s0, s0, t2
        or s0, s0, t3
        or s0, s0, t4
        or s0, s0, t5
        sw s0, 0x0(a1)
        jal value
        nop
        sw v0, 0x4(a1)
        mtc2 v0, $0
        jal value
        nop
        sw v0, 0x8(a1)
        mtc2 v0, $1
        jal value
        nop
        sw v0, 0xC(a1)
        mtc2 v0, $2
        jal value
        nop
        sw v0, 0x10(a1)
        mtc2 v0, $3
        jal value
        nop
        sw v0, 0x14(a1)
        mtc2 v0, $4
        jal value
        nop
        sw v0, 0x18(a1)
        mtc2 v0, $5
        jal value
        nop
        sw v0, 0x1C(a1)
        mtc2 v0, $6
        jal value
        nop
        sw v0, 0x20(a1)
        mtc2 v0, $7
        jal value
        nop
        sw v0, 0x24(a1)
        mtc2 v0, $8
        jal value
        nop
        sw v0, 0x28(a1)
        mtc2 v0, $9
        jal value
        nop
        sw v0, 0x2C(a1)
        mtc2 v0, $10
        jal value
        nop
        sw v0, 0x30(a1)
        mtc2 v0, $11
        jal value
        nop
        sw v0, 0x34(a1)
        mtc2 v0, $12
        jal value
        nop
        sw v0, 0x38(a1)
        mtc2 v0, $13
        jal value
        nop
        sw v0, 0x3C(a1)
        mtc2 v0, $14
        jal value
        nop
        sw v0, 0x40(a1)
        mtc2 v0, $15
        jal value
        nop
        sw v0, 0x44(a1)
        mtc2 v0, $16
        jal value
        nop
        sw v0, 0x48(a1)
        mtc2 v0, $17
        jal value
        nop
        sw v0, 0x4C(a1)
        mtc2 v0, $18
        jal value
        nop
        sw v0, 0x50(a1)
        mtc2 v0, $19
        jal value
        nop
        sw v0, 0x54(a1)
        mtc2 v0, $20
        jal value
        nop
        sw v0, 0x58(a1)
        mtc2 v0, $21
        jal value
        nop
        sw v0, 0x5C(a1)
        mtc2 v0, $22
        jal value
        nop
        sw v0, 0x60(a1)
        mtc2 v0, $23
        jal value
        nop
        sw v0, 0x64(a1)
        mtc2 v0, $24
        jal value
        nop
        sw v0, 0x68(a1)
        mtc2 v0, $25
        jal value
        nop
        sw v0, 0x6C(a1)
        mtc2 v0, $26
        jal value
        nop
        sw v0, 0x70(a1)
        mtc2 v0, $27
        jal value
        nop
        sw v0, 0x74(a1)
        mtc2 v0, $28
        jal value
        nop
        sw v0, 0x78(a1)
        mtc2 v0, $29
        jal value
        nop
        sw v0, 0x7C(a1)
        mtc2 v0, $30
        jal value
        nop
        sw v0, 0x80(a1)
        mtc2 v0, $31
        jal value
        nop
        sw v0, 0x84(a1)
        ctc2 v0, $0
        jal value
        nop
        sw v0, 0x88(a1)
        ctc2 v0, $1
        jal value
        nop
        sw v0, 0x8C(a1)
        ctc2 v0, $2
        jal value
        nop
        sw v0, 0x90(a1)
        ctc2 v0, $3
        jal value
        nop
        sw v0, 0x94(a1)
        ctc2 v0, $4
        jal value
        nop
        sw v0, 0x98(a1)
        ctc2 v0, $5
        jal value
        nop
        sw v0, 0x9C(a1)
        ctc2 v0, $6
        jal value
        nop
        sw v0, 0xA0(a1)
        ctc2 v0, $7
        jal value
        nop
        sw v0, 0xA4(a1)
        ctc2 v0, $8
        jal value
        nop
        sw v0, 0xA8(a1)
        ctc2 v0, $9
        jal value
        nop
        sw v0, 0xAC(a1)
        ctc2 v0, $10
        jal value
        nop
        sw v0, 0xB0(a1)
        ctc2 v0, $11
        jal value
        nop
        sw v0, 0xB4(a1)
        ctc2 v0, $12
        jal value
        nop
        sw v0, 0xB8(a1)
        ctc2 v0, $13
        jal value
        nop
        sw v0, 0xBC(a1)
        ctc2 v0, $14
        jal value
        nop
        sw v0, 0xC0(a1)
        ctc2 v0, $15
        jal value
        nop
        sw v0, 0xC4(a1)
        ctc2 v0, $16
        jal value
        nop
        sw v0, 0xC8(a1)
        ctc2 v0, $17
        jal value
        nop
        sw v0, 0xCC(a1)
        ctc2 v0, $18
        jal value
        nop
        sw v0, 0xD0(a1)
        ctc2 v0, $19
        jal value
        nop
        sw v0, 0xD4(a1)
        ctc2 v0, $20
        jal value
        nop
        sw v0, 0xD8(a1)
        ctc2 v0, $21
        jal value
        nop
        sw v0, 0xDC(a1)
        ctc2 v0, $22
        jal value
        nop
        sw v0, 0xE0(a1)
        ctc2 v0, $23
        jal value
        nop
        sw v0, 0xE4(a1)
        ctc2 v0, $24
        jal value
        nop
        sw v0, 0xE8(a1)
        ctc2 v0, $25
        jal value
        nop
        sw v0, 0xEC(a1)
        ctc2 v0, $26
        jal value
        nop
        sw v0, 0xF0(a1)
        ctc2 v0, $27
        jal value
        nop
        sw v0, 0xF4(a1)
        ctc2 v0, $28
        jal value
        nop
        sw v0, 0xF8(a1)
        ctc2 v0, $29
        jal value
        nop
        sw v0, 0xFC(a1)
        ctc2 v0, $30
        jal value
        nop
        sw v0, 0x100(a1)
        ctc2 v0, $31
        lui t4, 0x4A00
        or t4, t4, s0
        lui t3, 0xA001
        ori t3, t3, 0x0030
        sw t4, 0x0(t3)
        jalr t3
        nop
        mfc2 t0, $0
        nop
        sw t0, 0x104(a1)
        mfc2 t0, $1
        nop
        sw t0, 0x108(a1)
        mfc2 t0, $2
        nop
        sw t0, 0x10C(a1)
        mfc2 t0, $3
        nop
        sw t0, 0x110(a1)
        mfc2 t0, $4
        nop
        sw t0, 0x114(a1)
        mfc2 t0, $5
        nop
        sw t0, 0x118(a1)
        mfc2 t0, $6
        nop
        sw t0, 0x11C(a1)
        mfc2 t0, $7
        nop
        sw t0, 0x120(a1)
        mfc2 t0, $8
        nop
        sw t0, 0x124(a1)
        mfc2 t0, $9
        nop
        sw t0, 0x128(a1)
        mfc2 t0, $10
        nop
        sw t0, 0x12C(a1)
        mfc2 t0, $11
        nop
        sw t0, 0x130(a1)
        mfc2 t0, $12
        nop
        sw t0, 0x134(a1)
        mfc2 t0, $13
        nop
        sw t0, 0x138(a1)
        mfc2 t0, $14
        nop
        sw t0, 0x13C(a1)
        mfc2 t0, $15
        nop
        sw t0, 0x140(a1)
        mfc2 t0, $16
        nop
        sw t0, 0x144(a1)
        mfc2 t0, $17
        nop
        sw t0, 0x148(a1)
        mfc2 t0, $18
        nop
        sw t0, 0x14C(a1)
        mfc2 t0, $19
        nop
        sw t0, 0x150(a1)
        mfc2 t0, $20
        nop
        sw t0, 0x154(a1)
        mfc2 t0, $21
        nop
        sw t0, 0x158(a1)
        mfc2 t0, $22
        nop
        sw t0, 0x15C(a1)
        mfc2 t0, $23
        nop
        sw t0, 0x160(a1)
        mfc2 t0, $24
        nop
        sw t0, 0x164(a1)
        mfc2 t0, $25
        nop
        sw t0, 0x168(a1)
        mfc2 t0, $26
        nop
        sw t0, 0x16C(a1)
        mfc2 t0, $27
        nop
        sw t0, 0x170(a1)
        mfc2 t0, $28
        nop
        sw t0, 0x174(a1)
        mfc2 t0, $29
        nop
        sw t0, 0x178(a1)
        mfc2 t0, $30
        nop
        sw t0, 0x17C(a1)
        mfc2 t0, $31
        nop
        sw t0, 0x180(a1)
        cfc2 t0, $0
        nop
        sw t0, 0x184(a1)
        cfc2 t0, $1
        nop
        sw t0, 0x188(a1)
        cfc2 t0, $2
        nop
        sw t0, 0x18C(a1)
        cfc2 t0, $3
        nop
        sw t0, 0x190(a1)
        cfc2 t0, $4
        nop
        sw t0, 0x194(a1)
        cfc2 t0, $5
        nop
        sw t0, 0x198(a1)
        cfc2 t0, $6
        nop
        sw t0, 0x19C(a1)
        cfc2 t0, $7
        nop
        sw t0, 0x1A0(a1)
        cfc2 t0, $8
        nop
        sw t0, 0x1A4(a1)
        cfc2 t0, $9
        nop
        sw t0, 0x1A8(a1)
        cfc2 t0, $10
        nop
        sw t0, 0x1AC(a1)
        cfc2 t0, $11
        nop
        sw t0, 0x1B0(a1)
        cfc2 t0, $12
        nop
        sw t0, 0x1B4(a1)
        cfc2 t0, $13
        nop
        sw t0, 0x1B8(a1)
        cfc2 t0, $14
        nop
        sw t0, 0x1BC(a1)
        cfc2 t0, $15
        nop
        sw t0, 0x1C0(a1)
        cfc2 t0, $16
        nop
        sw t0, 0x1C4(a1)
        cfc2 t0, $17
        nop
        sw t0, 0x1C8(a1)
        cfc2 t0, $18
        nop
        sw t0, 0x1CC(a1)
        cfc2 t0, $19
        nop
        sw t0, 0x1D0(a1)
        cfc2 t0, $20
        nop
        sw t0, 0x1D4(a1)
        cfc2 t0, $21
        nop
        sw t0, 0x1D8(a1)
        cfc2 t0, $22
        nop
        sw t0, 0x1DC(a1)
        cfc2 t0, $23
        nop
        sw t0, 0x1E0(a1)
        cfc2 t0, $24
        nop
        sw t0, 0x1E4(a1)
        cfc2 t0, $25
        nop
        sw t0, 0x1E8(a1)
        cfc2 t0, $26
        nop
        sw t0, 0x1EC(a1)
        cfc2 t0, $27
        nop
        sw t0, 0x1F0(a1)
        cfc2 t0, $28
        nop
        sw t0, 0x1F4(a1)
        cfc2 t0, $29
        nop
        sw t0, 0x1F8(a1)
        cfc2 t0, $30
        nop
        sw t0, 0x1FC(a1)
        cfc2 t0, $31
        nop
        sw t0, 0x200(a1)
        addiu a1, a1, 516
        addiu s6, s6, 1
        addiu s1, s1, -1
        bne s1, zero, inner
        nop
        addiu s2, s2, 8
        j outer
        nop
finish: lui t1, 0x8010
        sw s6, -0xC(t1)
        lui t0, 0xD1A0
        ori t0, t0, 0xD00E
        sw t0, -0x10(t1)
done:   j done
        nop
