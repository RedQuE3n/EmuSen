# What MFC0 of the registers psx-spx calls garbage (16-31) returns (BlackMoon_R3000.md §6.1, its open item): each read
# in runs of eight straight after MFC0 of PRId. Results from 80100000h: words 0-15 for n = 16..31, word 16 a read of
# 16 long after PRId, word 17 a read of 16 straight after EPC. 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
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
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(a0)
done:   j done
        nop
