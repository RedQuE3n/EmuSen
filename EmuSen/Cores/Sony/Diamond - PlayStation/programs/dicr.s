# What DICR reads after each write (Diamond_Records.md §16.2): bit 6 above all, which psx-spx gives as channel 6's
# control bit and ps1-tests' io-access-bitwidth log reads zero. Interrupts off. Results from 80100000h, one DICR read
# per case in the order below; 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x8010
        lui a1, 0x1F80
        lui t0, 0x1234
        ori t0, t0, 0x5678
        sw t0, 0x10F4(a1)         # 0: a word of 12345678h
        lw t1, 0x10F4(a1)
        nop
        sw t1, 0(a0)
        sh t0, 0x10F4(a1)         # 1: a halfword store of the same register
        lw t1, 0x10F4(a1)
        nop
        sw t1, 4(a0)
        sb t0, 0x10F4(a1)         # 2: a byte store of it
        lw t1, 0x10F4(a1)
        nop
        sw t1, 8(a0)
        addiu t0, zero, 0x7F
        sw t0, 0x10F4(a1)         # 3: bits 0-6
        lw t1, 0x10F4(a1)
        nop
        sw t1, 12(a0)
        addiu t0, zero, 0x40
        sw t0, 0x10F4(a1)         # 4: bit 6 alone
        lw t1, 0x10F4(a1)
        nop
        sw t1, 16(a0)
        ori t0, zero, 0x7FFF
        sw t0, 0x10F4(a1)         # 5: bits 0-14
        lw t1, 0x10F4(a1)
        nop
        sw t1, 20(a0)
        lui t0, 0x00FF
        ori t0, t0, 0x007F
        sw t0, 0x10F4(a1)         # 6: bits 0-6, the masks and the master enable
        lw t1, 0x10F4(a1)
        nop
        sw t1, 24(a0)
        addiu t0, zero, 0x7F
        sw t0, 0x10F4(a1)
        lhu t1, 0x10F4(a1)        # 7: a halfword read after 7Fh
        nop
        sw t1, 28(a0)
        lbu t1, 0x10F4(a1)        # 8: a byte read
        nop
        sw t1, 32(a0)
        sw zero, 0x10F4(a1)       # 9: zero
        lw t1, 0x10F4(a1)
        nop
        sw t1, 36(a0)
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(a0)
done:   j done
        nop
