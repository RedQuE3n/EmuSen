# Reads port 1's controller over SIO0 as psx-spx's "Emulation Note" describes the kernel's driver (Diamond_Pad.md §4):
# each byte sent, about 100 cycles waited, IRQ7 acknowledged in I_STAT and then in CR, the next IRQ7 awaited with a
# timeout, the reply read once RXRDY is set. Three transfers of five bytes, 01h 42h 00h 00h 00h to port 1, the same to
# port 2, and 81h 42h 00h 00h 00h to port 1's card slot. Results from 80100000h: for each byte its reply and a 1 when
# IRQ7 rose before the timeout; 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui s0, 0x1F80
        lui s1, 0x8010
        ori t0, zero, 0x40
        sh t0, 0x104A(s0)             # CR: reset
        ori t0, zero, 0x88
        sh t0, 0x104E(s0)             # BR: the kernel's rate, 136 cycles a bit
        ori t0, zero, 0x0D
        sh t0, 0x1048(s0)             # MR: factor 1, 8 bits
        ori a1, zero, 0x1003          # CR: TXEN, DTR and DSRIEN, port 1
        jal xfer
        ori a2, zero, 0x01
        ori a1, zero, 0x3003          # port 2
        jal xfer
        ori a2, zero, 0x01
        ori a1, zero, 0x1003          # port 1, the card's address
        jal xfer
        ori a2, zero, 0x81
        lui a0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(a0)
done:   j done
        nop
# One transfer: selects with a1, sends a2 and then 42h, 00h, 00h, 00h from t7, low byte first.
xfer:   sh a1, 0x104A(s0)
        or t2, a2, zero
        ori t7, zero, 0x42
        ori t3, zero, 5
next:   sb t2, 0x1040(s0)
        ori t4, zero, 50
w100:   addiu t4, t4, -1
        bne t4, zero, w100
        nop
        addiu t5, zero, -0x81
        sw t5, 0x1070(s0)             # I_STAT: IRQ7 acknowledged first
        ori t5, a1, 0x10
        sh t5, 0x104A(s0)             # then CR.4
        ori t4, zero, 2000
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
        sb t5, 1(s1)
        addiu s1, s1, 2
        andi t2, t7, 0xFF
        srl t7, t7, 8
        addiu t3, t3, -1
        bne t3, zero, next
        nop
        jr ra
        sh zero, 0x104A(s0)           # deselected
