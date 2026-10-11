# What MFC0 of the registers psx-spx calls garbage (16-31) returns (BlackMoon_R3000.md §6.1, its open item): each read
# in runs of eight straight after MFC0 of PRId. Results from 80100000h: words 0-15 for n = 16..31, word 16 a read of
# 16 long after PRId, word 17 a read of 16 straight after EPC. Words 18-27, each after a register of a known value
# (BPC 11111111h, BDA 22222222h, BPCM 33333333h, BDAM 44444444h, SR 0, DCIC 0): 18 16 after BPC, 19 16 after BDA, 20 20
# after BPCM, 21 31 after BDAM, 22 16 after SR, 23 16 a NOP after BDA, 24 16 eight NOPs after BDA, 25 16 after a loop
# of 1,000, 26 16 straight after MTC0 of 55555555h to BDA, 27 16 after MFC0 of DCIC. Word 28 counts the exceptions
# taken, word 29 holds the last one's CAUSE (its own vector, the I-cache off). 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
vector: j handler
        nop
start:  mtc0 zero, SR
        lui t0, 0xFFFE
        lui t1, 0x0001
        ori t1, t1, 0xE188
        sw t1, 0x130(t0)          # BCC: the I-cache off, so the vector is fetched as written
        lui t0, 0x8001
        lw t1, 8(t0)
        lw t2, 12(t0)
        lui t3, 0x8000
        sw t1, 0x80(t3)
        sw t2, 0x84(t3)
        lui a0, 0x8010
        mfc0 t0, PRID
        mfc0 t1, $16
        mfc0 t2, $17
        mfc0 t3, $18
        mfc0 t4, $19
        mfc0 t5, $20
        mfc0 t6, $21
        mfc0 t7, $22
        mfc0 t8, $23
        nop
        sw t1, 0(a0)
        sw t2, 4(a0)
        sw t3, 8(a0)
        sw t4, 12(a0)
        sw t5, 16(a0)
        sw t6, 20(a0)
        sw t7, 24(a0)
        sw t8, 28(a0)
        mfc0 t0, PRID
        mfc0 t1, $24
        mfc0 t2, $25
        mfc0 t3, $26
        mfc0 t4, $27
        mfc0 t5, $28
        mfc0 t6, $29
        mfc0 t7, $30
        mfc0 t8, $31
        nop
        sw t1, 32(a0)
        sw t2, 36(a0)
        sw t3, 40(a0)
        sw t4, 44(a0)
        sw t5, 48(a0)
        sw t6, 52(a0)
        sw t7, 56(a0)
        sw t8, 60(a0)
        mfc0 t0, PRID
        nop
        nop
        nop
        mfc0 t1, $16              # result 16: the read long after PRId
        nop
        sw t1, 64(a0)
        mfc0 t0, EPC
        mfc0 t1, $16              # result 17: straight after EPC
        nop
        sw t1, 68(a0)
        mtc0 zero, DCIC
        lui t0, 0x1111
        ori t0, t0, 0x1111
        mtc0 t0, BPC
        lui t0, 0x2222
        ori t0, t0, 0x2222
        mtc0 t0, BDA
        lui t0, 0x3333
        ori t0, t0, 0x3333
        mtc0 t0, BPCM
        lui t0, 0x4444
        ori t0, t0, 0x4444
        mtc0 t0, BDAM
        nop
        mfc0 t0, BPC
        mfc0 t1, $16              # 18
        nop
        sw t1, 72(a0)
        mfc0 t0, BDA
        mfc0 t1, $16              # 19
        nop
        sw t1, 76(a0)
        mfc0 t0, BPCM
        mfc0 t1, $20              # 20
        nop
        sw t1, 80(a0)
        mfc0 t0, BDAM
        mfc0 t1, $31              # 21
        nop
        sw t1, 84(a0)
        mfc0 t0, SR
        mfc0 t1, $16              # 22
        nop
        sw t1, 88(a0)
        mfc0 t0, BDA
        nop
        mfc0 t1, $16              # 23
        nop
        sw t1, 92(a0)
        mfc0 t0, BDA
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mfc0 t1, $16              # 24
        nop
        sw t1, 96(a0)
        addiu t2, zero, 1000
wait:   addiu t2, t2, -1
        bne t2, zero, wait
        nop
        mfc0 t1, $16              # 25
        nop
        sw t1, 100(a0)
        lui t0, 0x5555
        ori t0, t0, 0x5555
        mtc0 t0, BDA
        mfc0 t1, $16              # 26
        nop
        sw t1, 104(a0)
        mfc0 t0, DCIC
        mfc0 t1, $16              # 27
        nop
        sw t1, 108(a0)
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(a0)
done:   j done
        nop
handler: lui k1, 0x8010
        lw k0, 112(k1)
        nop
        addiu k0, k0, 1
        sw k0, 112(k1)
        mfc0 k0, CAUSE
        nop
        sw k0, 116(k1)
        mfc0 k0, EPC
        nop
        addiu k0, k0, 4
        jr k0
        rfe
