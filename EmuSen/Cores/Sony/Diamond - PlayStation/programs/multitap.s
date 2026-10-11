# Port 1's multitap (Diamond_Pad.md §9), each byte sent as programs/pad.s does: slot A read with the request bit set
# (01h 42h 01h 00h 00h), then the long read that request asks for (01h 42h 00h and 32 zeros), then slot C read by its
# own address (03h 42h 00h 00h 00h), and the card in slot A's Get ID (81h 53h and eight zeros). Replies from 80100000h,
# 80100040h, 80100080h and 801000C0h, IRQ7 flags at the same offsets plus 400h; 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui s0, 0x1F80
        ori t0, zero, 0x40
        sh t0, 0x104A(s0)
        ori t0, zero, 0x88
        sh t0, 0x104E(s0)
        ori t0, zero, 0x0D
        sh t0, 0x1048(s0)
        lui a2, 0x8011
        or t2, zero, zero
clear:  addu t4, a2, t2
        sb zero, 0(t4)
        addiu t2, t2, 1
        slti t5, t2, 64
        bne t5, zero, clear
        nop
        ori t0, zero, 0x01
        sb t0, 0(a2)
        ori t0, zero, 0x42
        sb t0, 1(a2)
        ori t0, zero, 0x01
        sb t0, 2(a2)                  # the request
        lui s1, 0x8010
        jal xfer
        ori a3, zero, 5
        sb zero, 2(a2)
        lui s1, 0x8010
        ori s1, s1, 0x40
        jal xfer
        ori a3, zero, 35
        ori t0, zero, 0x03
        sb t0, 0(a2)
        lui s1, 0x8010
        ori s1, s1, 0x80
        jal xfer
        ori a3, zero, 5
        ori t0, zero, 0x81
        sb t0, 0(a2)
        ori t0, zero, 0x53
        sb t0, 1(a2)
        lui s1, 0x8010
        ori s1, s1, 0xC0
        jal xfer
        ori a3, zero, 10
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
