# Reads a disc's sector as psx-spx's "ReadN" chapter describes the kernel's sequence after INT1 (Diamond_CDROM.md §9),
# polling with interrupts off: Nop until the motor turns, GetID's two responses, Setloc 00:02:16 and Setmode 80h, ReadN, the first INT1,
# HCHPCTL cleared then BFRD, DMA channel 3 of 512 words to 80110000h, the INT1 acknowledged, then Pause's two. From
# 80100000h: each response's type and first byte, GetID's eight bytes, I_STAT's IRQ2 as seen after the first response,
# the top byte of DMA's CHCR after the transfer; 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui s0, 0x1F80
        lui s1, 0x8010
        ori t0, zero, 1
        sb t0, 0x1800(s0)             # bank 1
        ori t0, zero, 0x1F
        sb t0, 0x1802(s0)             # HINTMSK: every flag
        sb t0, 0x1803(s0)             # HCLRCTL: acknowledge all
        addiu t0, zero, -1
        sw t0, 0x1070(s0)             # I_STAT left as it is
spin:   ori a0, zero, 0x01            # Nop, until the motor is on
        jal command
        nop
        jal response
        nop
        jal ack
        nop
        lbu t0, -1(s1)
        addiu s1, s1, -2
        andi t0, t0, 2
        beq t0, zero, spin
        nop
        ori a0, zero, 0x01            # Nop once more, its IRQ2 seen
        jal command
        nop
        jal response
        nop
        lw t0, 0x1070(s0)
        nop
        srl t0, t0, 2
        andi t0, t0, 1
        sb t0, 0(s1)                  # I_STAT's IRQ2 rose
        addiu s1, s1, 1
        jal ack
        nop
        ori a0, zero, 0x1A            # GetID
        jal command
        nop
        jal response
        nop
        jal ack
        nop
        jal response                  # its second, eight bytes
        nop
        ori t3, zero, 7
getid:  lbu t1, 0x1801(s0)
        nop
        sb t1, 0(s1)
        addiu t3, t3, -1
        bne t3, zero, getid
        addiu s1, s1, 1
        jal ack
        nop
        sb zero, 0x1800(s0)           # bank 0
        sb zero, 0x1802(s0)           # parameters: 00:02:16
        ori t0, zero, 0x02
        sb t0, 0x1802(s0)
        ori t0, zero, 0x16
        sb t0, 0x1802(s0)
        ori a0, zero, 0x02            # Setloc
        jal command
        nop
        jal response
        nop
        jal ack
        nop
        sb zero, 0x1800(s0)
        ori t0, zero, 0x80
        sb t0, 0x1802(s0)
        ori a0, zero, 0x0E            # Setmode: double speed, 800h
        jal command
        nop
        jal response
        nop
        jal ack
        nop
        ori a0, zero, 0x06            # ReadN
        jal command
        nop
        jal response
        nop
        jal ack
        nop
        jal response                  # INT1
        nop
        sb zero, 0x1800(s0)
        sb zero, 0x1803(s0)           # HCHPCTL cleared
        ori t0, zero, 0x80
        sb t0, 0x1803(s0)             # BFRD
        lw t0, 0x10F0(s0)
        nop
        ori t0, t0, 0x8000
        sw t0, 0x10F0(s0)             # DPCR: channel 3 on
        lui t0, 0x8011
        sw t0, 0x10B0(s0)
        lui t0, 0x0001
        ori t0, t0, 0x0200
        sw t0, 0x10B4(s0)             # 512 words
        lui t0, 0x1100
        sw t0, 0x10B8(s0)
dmaw:   lw t0, 0x10B8(s0)
        nop
        srl t1, t0, 24
        andi t1, t1, 1
        bne t1, zero, dmaw
        nop
        srl t0, t0, 24
        sb t0, 0(s1)                  # CHCR's top byte
        addiu s1, s1, 1
        jal ack
        nop
        ori a0, zero, 0x09            # Pause
        jal command
        nop
        jal response
        nop
        jal ack
        nop
        jal response
        nop
        jal ack
        nop
        lui a0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(a0)
done:   j done
        nop
# Writes command a0 in bank 0.
command:
        sb zero, 0x1800(s0)
        jr ra
        sb a0, 0x1801(s0)
# Waits for HINTSTS's type, then records it and the result's first byte at s1.
response:
        ori t0, zero, 1
        sb t0, 0x1800(s0)
        lui t2, 0x0400
wint:   lbu t1, 0x1803(s0)
        nop
        andi t1, t1, 7
        bne t1, zero, gotint
        addiu t2, t2, -1
        bne t2, zero, wint
        nop
gotint: sb t1, 0(s1)
        lbu t1, 0x1801(s0)
        nop
        sb t1, 1(s1)
        jr ra
        addiu s1, s1, 2
# Acknowledges the HC05's flags.
ack:    ori t0, zero, 1
        sb t0, 0x1800(s0)
        ori t0, zero, 0x1F
        jr ra
        sb t0, 0x1803(s0)
