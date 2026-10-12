# When DICR's master flag sets I_STAT's bit 3 (Diamond_Disputes.md D-29, Diamond_Records.md §16.3): psx-spx has bit 31
# recalculated on every write and I_STAT's bit set on its 0-to-1 transition, so a write that leaves the flag up sets
# nothing. Channel 6 fills four words for each completion. Interrupts off. Results from 80100000h, two words a case,
# I_STAT's bit 3 and DICR, in the order below; 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x8010
        lui a1, 0x1F80
        sw zero, 0x1074(a1)       # I_MASK: nothing
        lui t0, 0x0800
        sw t0, 0x10F0(a1)         # DPCR: channel 6 enabled
        lui t0, 0x7F00
        sw t0, 0x10F4(a1)         # DICR: every flag acknowledged, everything off
        lui s0, 0x00C0            # the master enable and channel 6's
        lui s1, 0x0040            # channel 6's alone
        lui s2, 0x40C0            # both, and channel 6's flag acknowledged
        ori s3, s0, 0x8000        # both, and the bus error flag
        sw s0, 0x10F4(a1)
        jal fill                  # 0: a completion with the flag down: I_STAT's bit set
        nop
        jal note
        nop
        sw s0, 0x10F4(a1)         # 1: a write that keeps the flag: nothing set
        jal note
        nop
        sw s1, 0x10F4(a1)         # 2: the master enable cleared: bit 31 down
        jal note
        nop
        sw s0, 0x10F4(a1)         # 3: the master enable set again with the flag kept: bit 31 up by a write
        jal note
        nop
        jal fill                  # 4: a second completion with bit 31 already up: nothing set
        nop
        jal note
        nop
        sw s2, 0x10F4(a1)         # 5: the flag acknowledged
        jal note
        nop
        sw s3, 0x10F4(a1)         # 6: the bus error flag written: bit 31 up by a write
        jal note
        nop
        sw s3, 0x10F4(a1)         # 7: written again
        jal note
        nop
        sw s0, 0x10F4(a1)         # 8: and cleared
        jal note
        nop
        jal fill                  # 9: a completion with the flag down again
        nop
        jal note
        nop
        sw s2, 0x10F4(a1)
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        lui t0, 0x8010
        sw t1, -16(t0)
done:   j done
        nop

# I_STAT's bit 3 and DICR recorded, then I_STAT's bit 3 acknowledged.
note:   lw t0, 0x1070(a1)
        lw t1, 0x10F4(a1)
        andi t0, t0, 8
        sw t0, 0(a0)
        sw t1, 4(a0)
        ori t0, zero, 0xFFF7
        sw t0, 0x1070(a1)
        jr ra
        addiu a0, a0, 8

# Channel 6 over four words at 80100400h, waited for.
fill:   lui t0, 0x0010
        ori t0, t0, 0x040C
        sw t0, 0x10E0(a1)         # MADR
        addiu t0, zero, 4
        sw t0, 0x10E4(a1)         # BCR
        lui t0, 0x1100
        ori t0, t0, 2
        sw t0, 0x10E8(a1)         # CHCR: start and trigger
        lui t2, 0x0100
wait:   lw t0, 0x10E8(a1)
        nop
        and t0, t0, t2
        bne t0, zero, wait
        nop
        addiu t0, zero, 64
rest:   bne t0, zero, rest
        addiu t0, t0, -1
        jr ra
        nop
