# A memory card in port 1's slot through its protocol (Diamond_Pad.md §8), each byte sent as the kernel's driver does
# (programs/pad.s): Write sector 0100h with the bytes 3n, Read it back, Get ID twice, and Write sector 0101h with a
# bad checksum. Replies from 80100000h (the write), 80100100h (the read), 80100200h (the first ID), 80100280h (the
# second) and 80100300h (the bad write), and for each byte 1 at the same offset plus 400h when IRQ7 rose before the
# timeout; 800FFFF0h holds D1A0D00Eh when done. The packets are built at 80110000h.
        j start
        nop
start:  mtc0 zero, SR
        lui s0, 0x1F80
        ori t0, zero, 0x40
        sh t0, 0x104A(s0)             # CR: reset
        ori t0, zero, 0x88
        sh t0, 0x104E(s0)             # BR: 136 cycles a bit
        ori t0, zero, 0x0D
        sh t0, 0x1048(s0)             # MR: factor 1, 8 bits
# the write of sector 0100h: 81h 57h 00h 00h 01h 00h, 128 bytes of 3n, the checksum, three zeros
        lui a2, 0x8011
        ori t0, zero, 0x81
        sb t0, 0(a2)
        ori t0, zero, 0x57
        sb t0, 1(a2)
        sb zero, 2(a2)
        sb zero, 3(a2)
        ori t0, zero, 0x01
        sb t0, 4(a2)
        sb zero, 5(a2)
        ori t1, zero, 0x01            # the checksum, from MSB xor LSB
        or t2, zero, zero             # n
        or t3, zero, zero             # 3n
data:   addu t4, a2, t2
        sb t3, 6(t4)
        xor t1, t1, t3
        addiu t2, t2, 1
        addiu t3, t3, 3
        slti t5, t2, 128
        bne t5, zero, data
        nop
        sb t1, 134(a2)
        sb zero, 135(a2)
        sb zero, 136(a2)
        sb zero, 137(a2)
        lui s1, 0x8010
        jal xfer
        ori a3, zero, 138
# the read of sector 0100h: 81h 52h 00h 00h 01h 00h and 134 zeros
        ori t0, zero, 0x52
        sb t0, 1(a2)
        or t2, zero, zero
clear:  addu t4, a2, t2
        sb zero, 6(t4)
        addiu t2, t2, 1
        slti t5, t2, 134
        bne t5, zero, clear
        nop
        lui s1, 0x8010
        ori s1, s1, 0x100
        jal xfer
        ori a3, zero, 140
# Get ID twice: 81h 53h and eight zeros; the second's FLAG shows the write cleared bit 3
        ori t0, zero, 0x53
        sb t0, 1(a2)
        sb zero, 4(a2)
        lui s1, 0x8010
        ori s1, s1, 0x200
        jal xfer
        ori a3, zero, 10
        lui s1, 0x8010
        ori s1, s1, 0x280
        jal xfer
        ori a3, zero, 10
# a write of sector 0101h with a checksum off by one, which the card refuses with 4Eh
        ori t0, zero, 0x57
        sb t0, 1(a2)
        ori t0, zero, 0x01
        sb t0, 4(a2)
        ori t0, zero, 0x01
        sb t0, 5(a2)
        ori t0, zero, 0x01            # the data are now zero, the right checksum 0
        sb t0, 134(a2)
        lui s1, 0x8010
        ori s1, s1, 0x300
        jal xfer
        ori a3, zero, 138
        lui a0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(a0)
done:   j done
        nop
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
