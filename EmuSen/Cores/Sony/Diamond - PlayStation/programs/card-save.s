# A game's save through the card in port 1's slot (Diamond_Cards.md §10): Get ID, then Write sector SECTOR with 128
# bytes of FILL, the two words after the jump, which a test sets in the image; then Get ID again for as long as the
# program runs, each pass keeping the flag the card answered. Replies from 80100000h (the first ID), 80100100h (the
# write) and 80100200h (the latest ID), the passes counted at 80100280h; 800FFFF0h holds D1A0D00Eh once the write is
# done. Each byte is sent as programs/card.s sends it, its xfer the same; the packets are built at 80110000h.
        j start
        nop
        .word 0x100                   # SECTOR
        .word 0xA5                    # FILL
start:  mtc0 zero, SR
        lui s0, 0x1F80
        ori t0, zero, 0x40
        sh t0, 0x104A(s0)             # CR: reset
        ori t0, zero, 0x88
        sh t0, 0x104E(s0)             # BR: 136 cycles a bit
        ori t0, zero, 0x0D
        sh t0, 0x1048(s0)             # MR: factor 1, 8 bits
        lui t0, 0x8001
        lw s2, 8(t0)
        lw s3, 12(t0)
        lui a2, 0x8011
        lui s1, 0x8010
        jal id
        nop
# the write: 81h 57h 00h 00h, the sector's MSB and LSB, 128 bytes of the fill, the checksum, three zeros
        ori t0, zero, 0x81
        sb t0, 0(a2)
        ori t0, zero, 0x57
        sb t0, 1(a2)
        sb zero, 2(a2)
        sb zero, 3(a2)
        srl t1, s2, 8
        sb t1, 4(a2)
        sb s2, 5(a2)
        xor t1, t1, s2
        andi t1, t1, 0xFF             # the checksum, from MSB xor LSB
        or t2, zero, zero
data:   addu t4, a2, t2
        sb s3, 6(t4)
        xor t1, t1, s3
        addiu t2, t2, 1
        slti t5, t2, 128
        bne t5, zero, data
        nop
        sb t1, 134(a2)
        sb zero, 135(a2)
        sb zero, 136(a2)
        sb zero, 137(a2)
        lui s1, 0x8010
        ori s1, s1, 0x100
        jal xfer
        ori a3, zero, 138
        lui a0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(a0)
# then the ID, again and again, its flag the card's as the game would next see it
        or s4, zero, zero
again:  lui s1, 0x8010
        ori s1, s1, 0x200
        jal id
        nop
        addiu s4, s4, 1
        lui t0, 0x8010
        sw s4, 0x280(t0)
        lui t4, 0x0004
pause:  addiu t4, t4, -1
        bne t4, zero, pause
        nop
        j again
        nop
# Get ID: 81h 53h and eight zeros from a2, the replies to s1.
id:     ori t0, zero, 0x81
        sb t0, 0(a2)
        ori t0, zero, 0x53
        sb t0, 1(a2)
        sb zero, 2(a2)
        sb zero, 3(a2)
        sb zero, 4(a2)
        sb zero, 5(a2)
        sb zero, 6(a2)
        sb zero, 7(a2)
        sb zero, 8(a2)
        sb zero, 9(a2)
        j xfer
        ori a3, zero, 10
# One transfer on port 1: a3 bytes from a2, their replies to s1 and the IRQ7 flags to s1 + 400h.
xfer:   ori t0, zero, 0x1003
        sh t0, 0x104A(s0)             # CR: TXEN, DTR and DSRIEN, port 1
        or t8, a2, zero
        or t9, a3, zero
next:   lbu t2, 0(t8)
        nop
        sb t2, 0x1040(s0)
        ori t4, zero, 50
w100:   addiu t4, t4, -1
        bne t4, zero, w100
        nop
        addiu t5, zero, -0x81
        sw t5, 0x1070(s0)             # I_STAT: IRQ7 acknowledged first
        ori t5, zero, 0x1013
        sh t5, 0x104A(s0)             # then CR.4
        lui t4, 0x0001                # a timeout of 65,536 polls, past a read's late /ACK
wirq:   lw t5, 0x1070(s0)
        nop
        andi t5, t5, 0x80
        bne t5, zero, got
        addiu t4, t4, -1
        bne t4, zero, wirq
        nop
got:    srl t5, t5, 7
wrx:    lw t6, 0x1044(s0)
        nop
        andi t6, t6, 2
        beq t6, zero, wrx
        nop
        lbu t6, 0x1040(s0)
        nop
        sb t6, 0(s1)
        sb t5, 0x400(s1)
        addiu s1, s1, 1
        addiu t8, t8, 1
        addiu t9, t9, -1
        bne t9, zero, next
        nop
        jr ra
        sh zero, 0x104A(s0)           # deselected
