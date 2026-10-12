# D-47: primitives whose first vertex and the drawing offset pass 1,023 together (Diamond_Disputes.md D-47). The area
# is VRAM's right half, x 512 to 1023 and lines 0 to 255, and the offset 768, the right buffer's centre, as a game
# drawing a 512-wide screen there sets them. A flat triangle whose first vertex is at 286, past the screen's right
# edge, and its others at 224 on it; the same triangle from a first vertex on the screen; a shaded triangle from 300;
# a flat line from 300 leftward to 200. Dithering off. VRAM's x 896 to 1023, lines 0 to 191, is then read back two
# pixels a word to 80100000h. 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x1F80
        sw zero, 0x1814(a0)           # GP1(00h)
        bgezal zero, go               # ra: the table's address
        nop
list:
        .word 1                      # GP0(E1h): dithering off, drawing to the display allowed
        .word 0xE1000400
        .word 1                      # GP0(E3h): area from 512,0
        .word 0xE3000200
        .word 1                      # GP0(E4h): area to 1023,255
        .word 0xE403FFFF
        .word 1                      # GP0(E5h): offset 768,0
        .word 0xE5000300
        .word 3                      # GP0(02h): VRAM 512..1023, 0..255 black
        .word 0x02000000
        .word 0x00000200
        .word 0x01000200
        .word 4                      # a flat triangle from (286,16), then (224,16) and (224,48)
        .word 0x200000FF
        .word 0x0010011E
        .word 0x001000E0
        .word 0x003000E0
        .word 4                      # the same from (224,64), then (286,64) and (224,96)
        .word 0x200000FF
        .word 0x004000E0
        .word 0x0040011E
        .word 0x006000E0
        .word 6                      # a shaded triangle from (300,112), then (200,112) and (200,144)
        .word 0x300000FF
        .word 0x0070012C
        .word 0x0000FF00
        .word 0x007000C8
        .word 0x00FF0000
        .word 0x009000C8
        .word 3                      # a flat line from (300,176) to (200,176)
        .word 0x4000FF00
        .word 0x00B0012C
        .word 0x00B000C8
        .word 0
go:     or a1, ra, zero
        jal send
        nop
        lui t9, 0x8010
        ori a1, zero, 896
        lui a2, 192
        ori a2, a2, 128
        jal readback
        ori a3, zero, 12288
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        lui t0, 0x8010
        sw t1, -16(t0)
done:   j done
        nop
# send the table at a1: records of a count and that many GP0 words, each record once the GPU is idle, to a count of 0
send:   lw t1, 0(a1)
        addiu a1, a1, 4
        beq t1, zero, sent
        nop
idle:   lw t0, 0x1814(a0)
        lui t2, 0x1400
        and t0, t0, t2
        bne t0, t2, idle
        nop
word:   lw t0, 0(a1)
        addiu a1, a1, 4
        addiu t1, t1, -1
        bne t1, zero, word
        sw t0, 0x1810(a0)
        j send
        nop
sent:   jr ra
        nop
# read VRAM from yx a1, size a2 (h << 16 | w): a3 words of it to t9 on, once the GPU is idle
readback: lw t0, 0x1814(a0)
        lui t2, 0x1400
        and t0, t0, t2
        bne t0, t2, readback
        nop
        lui t0, 0xC000
        sw t0, 0x1810(a0)
        sw a1, 0x1810(a0)
        sw a2, 0x1810(a0)
rb1:    lw t0, 0x1814(a0)
        lui t2, 0x0800
        and t0, t0, t2
        beq t0, zero, rb1
        nop
        lw t0, 0x1810(a0)
        addiu a3, a3, -1
        sw t0, 0(t9)
        bne a3, zero, rb1
        addiu t9, t9, 4
        jr ra
        nop
