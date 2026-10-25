# The memory controller's serial interface to the RDRAMs (Sapphire_Native.md §35.3, D-322): a table of commands at
# `cmds`, each a pair of words, the value for MCH_DRD (1000F440h) and then for MCH_RICM (1000F430h, bit 31 the start);
# after each, MCH_RICM read until bit 31 clears (at most 1000 reads), then both registers read. Four words a command
# from 00200000h: the RICM written, the polls left, RICM read, DRD read. A pair of zero words ends the table; then the
# done word. The table follows the branch that skips it, so RA holds its address.
        lui a0, 0x0020
        lui s0, 0xB000
        ori s0, s0, 0xF430
        bgezal zero, past
        nop
cmds:   .word 0x00000000
        .word 0x80210000
        .word 0x00000000
        .word 0x80210000
        .word 0x00000000
        .word 0x80210000
        .word 0x00000000
        .word 0x80210040
        .word 0x00000000
        .word 0x80210000
        .word 0x00000000
        .word 0x80210000
        .word 0x00000000
        .word 0x80210000
        .word 0x00000080
        .word 0x80210040
        .word 0x00000000
        .word 0x80210000
        .word 0x00000000
        .word 0x80210000
        .word 0x00000000
        .word 0x80230000
        .word 0x00000000
        .word 0x80240000
        .word 0x00000000
        .word 0x80400000
        .word 0x00000000
        .word 0x80400015
        .word 0x00000000
        .word 0x8040001F
        .word 0x00000000
        .word 0x80400080
        .word 0x00000000
        .word 0x80220000
        .word 0x00000000
        .word 0x80230040
        .word 0x00000000
        .word 0x80230080
        .word 0x0000ABCD
        .word 0x80000000
        .word 0x00000000
        .word 0x80210000
        .word 0x00000000
        .word 0x00210000
        .word 0
        .word 0
past:   addu s1, ra, zero
next:   lw t0, 0(s1)
        lw t1, 4(s1)
        nop
        or t2, t0, t1
        beq t2, zero, done
        nop
        sw t0, 16(s0)
        sw t1, 0(s0)
        addiu t3, zero, 1000
poll:   lw t4, 0(s0)
        nop
        bgez t4, ready
        nop
        addiu t3, t3, -1
        bne t3, zero, poll
        nop
ready:  lw t4, 0(s0)
        lw t5, 16(s0)
        sw t1, 0(a0)
        sw t3, 4(a0)
        sw t4, 8(a0)
        sw t5, 12(a0)
        addiu a0, a0, 16
        addiu s1, s1, 8
        beq zero, zero, next
        nop
done:   lui a0, 0x0020
        lui t0, 0x5A99
        ori t0, t0, 0xD00E
        sw t0, -16(a0)
end:    beq zero, zero, end
        nop
