# The referees' gate (Sapphire_Referees.md §4): a bare EE program that calls no kernel, computes twelve words of MIPS I
# arithmetic whose results the architecture fixes, writes them from 00200000h, and then writes 5A99D00Eh to 001FFFF0h.
# Assembled at 00100000h by examples/elf.rs. Loaded values are used only after a nop, so no load delay is relied on.
        lui a0, 0x0020
        lui t0, 0x7FFF
        ori t0, t0, 0xFFFF
        addiu t1, zero, 1
        addu t2, t0, t1           # 0: 7FFFFFFFh + 1, which addu wraps without a trap: 80000000h
        sw t2, 0(a0)
        sra t3, t2, 4             # 1: F8000000h
        sw t3, 4(a0)
        srl t3, t2, 4             # 2: 08000000h
        sw t3, 8(a0)
        sltu t4, t1, t2           # 3: 1 below 80000000h unsigned: 1
        sw t4, 12(a0)
        slt t4, t1, t2            # 4: 1 below -2^31 signed: 0
        sw t4, 16(a0)
        mult t0, t0               # 5, 6: 7FFFFFFFh squared, LO 00000001h and HI 3FFFFFFFh
        mflo t5
        mfhi t6
        sw t5, 20(a0)
        sw t6, 24(a0)
        addiu t5, zero, -7
        addiu t6, zero, 2
        div t5, t6                # 7, 8: -7 / 2, the quotient FFFFFFFDh and the remainder FFFFFFFFh
        mflo t7
        mfhi t8
        sw t7, 28(a0)
        sw t8, 32(a0)
        addiu t0, zero, 0
        addiu t1, zero, 100
sum:    addu t0, t0, t1
        addiu t1, t1, -1
        bne t1, zero, sum
        nop
        sw t0, 36(a0)             # 9: 100 + 99 + ... + 1 = 5050, 000013BAh
        addiu t2, zero, 0
        beq zero, zero, over
        addiu t2, t2, 1           # the delay slot, which runs
        addiu t2, t2, 16          # skipped
over:   sw t2, 40(a0)             # 10: 00000001h
        lui t3, 0x1122
        ori t3, t3, 0x3344
        sw t3, 0x100(a0)
        addiu t4, zero, 0x80
        sb t4, 0x101(a0)          # the word is now 11228044h
        lb t5, 0x101(a0)
        lbu t6, 0x100(a0)
        lhu t7, 0x102(a0)
        nop
        sll t6, t6, 8
        xor t6, t6, t5
        sll t7, t7, 16
        xor t7, t7, t6            # 11: lb's FFFFFF80h, lbu's 44h shifted to 4400h, lhu's 1122h shifted: EEDDBB80h
        sw t7, 44(a0)
        lui t0, 0x5A99
        ori t0, t0, 0xD00E
        sw t0, -16(a0)            # done: 001FFFF0h
end:    beq zero, zero, end
        nop
