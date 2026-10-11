# What SWC0, SWC1 and SWC3 store with their CU bit set (BlackMoon_R3000.md D-21): each case stores to its own word,
# filled first with 5A5A5A5Ah, its GPR of the same number holding a known value and the data bus last carrying C0DE00nnh
# (a load just before) or 5EED5EEDh (a store just before). SWC2 of the GTE's VXY0 is the control. Cases 11-13 store
# to C0000000h, where no device answers, and record the CAUSE their exception wrote (5A5A5A5Ah if none): SWC3, SW (the
# control) and SWC0. Interrupts off; installs its own vector, the I-cache off. Results from 80100000h, one word per case;
# 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
vector: j handler
        nop
start:  lui t0, 0xF000
        mtc0 t0, SR               # CU0-CU3, kernel mode
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
        lui t0, 0x5A5A
        ori t0, t0, 0x5A5A
        addiu t1, zero, 0
fill:   addu t2, a0, t1
        sw t0, 0(t2)
        addiu t1, t1, 4
        slti t2, t1, 56
        bne t2, zero, fill
        nop
        addiu t1, zero, 0
mark:   addu t2, a0, t1
        lui t3, 0xC0DE
        srl t4, t1, 2
        or t3, t3, t4
        sw t3, 0x200(t2)
        addiu t1, t1, 4
        slti t2, t1, 56
        bne t2, zero, mark
        nop
        lui t0, 0x1234
        ori t0, t0, 0x5678
        mtc2 t0, $0
        lui s0, 0x5EED
        ori s0, s0, 0x5EED
        lui at, 0x0101
        ori at, at, 0x0101
        lui v0, 0x0202
        ori v0, v0, 0x0202
        lui t4, 0x0C0C
        ori t4, t4, 0x0C0C
        lui t7, 0x0F0F
        ori t7, t7, 0x0F0F
        lw t9, 0x200(a0)
        swc0 PRID, 0(a0)          # 0: SWC0 of PRId (amidog's case)
        lw t9, 0x204(a0)
        swc0 SR, 4(a0)            # 1: SWC0 of SR
        lw t9, 0x208(a0)
        swc0 $1, 8(a0)            # 2: SWC0 of an absent register
        lw t9, 0x20C(a0)
        .word 0xE481000C          # 3: swc1 $1, 12(a0)
        lw t9, 0x210(a0)
        .word 0xE4820010          # 4: swc1 $2, 16(a0)
        lw t9, 0x214(a0)
        .word 0xEC810014          # 5: swc3 $1, 20(a0)
        lw t9, 0x218(a0)
        .word 0xEC8F0018          # 6: swc3 $15, 24(a0)
        lw t9, 0x21C(a0)
        swc2 $0, 28(a0)           # 7: SWC2 of VXY0, 12345678h
        sw s0, 0x300(a0)
        .word 0xEC810020          # 8: swc3 $1, 32(a0) after a store
        sw s0, 0x300(a0)
        .word 0xE0810024          # 9: swc0 $1, 36(a0) after a store
        nop
        nop
        .word 0xEC810028          # 10: swc3 $1, 40(a0) after two NOPs
        nop
        lui a1, 0xC000
        addiu s7, a0, 44          # where the handler writes CAUSE
        .word 0xECA10000          # 11: swc3 $1, 0(a1)
        nop
        addiu s7, a0, 48
        sw at, 0(a1)              # 12: sw
        nop
        addiu s7, a0, 52
        .word 0xE0A10000          # 13: swc0 $1, 0(a1)
        nop
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(a0)
done:   j done
        nop
handler: mfc0 k0, CAUSE
        nop
        sw k0, 0(s7)
        mfc0 k0, EPC
        nop
        addiu k0, k0, 4
        jr k0
        rfe
