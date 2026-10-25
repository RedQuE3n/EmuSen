# A frame counter for the referees' cost (Sapphire_Referees.md §2.1): writes 5A99D00Eh to 001FFFF0h at once, then counts
# the GS's vertical syncs at 00200000h, polling CSR's VSINT bit (bit 3 of 12001000h) and writing it back to clear it.
        lui a0, 0x0020
        lui a1, 0x1200
        sw zero, 0(a0)
        lui t0, 0x5A99
        ori t0, t0, 0xD00E
        sw t0, -16(a0)
        addiu t1, zero, 0
        addiu t2, zero, 8
wait:   lw t3, 0x1000(a1)
        nop
        andi t3, t3, 8
        beq t3, zero, wait
        nop
        sw t2, 0x1000(a1)
        addiu t1, t1, 1
        beq zero, zero, wait
        sw t1, 0(a0)
