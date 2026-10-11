# D-8: where a flipped rectangle's texture starts (Diamond_Disputes.md D-8). A 15-bit texture at 512,0 whose texel
# at U, V holds 8000h + V*100h + U is drawn raw as rectangles two lines high: unflipped, x-flipped at even and odd
# columns with even and odd U, y-flipped at V 0 and 1, both, and one pixel wide. The 192x2 drawn from 0,0 is read
# back to 80100000h, two pixels a word. 800FFFF0h holds D1A0D00Eh when done.
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
        .word 1                      # GP0(E3h): area from 0,0
        .word 0xE3000000
        .word 1                      # GP0(E4h): area to 1023,511
        .word 0xE407FFFF
        .word 1                      # GP0(E5h): no offset
        .word 0xE5000000
        .word 3                      # GP0(02h): VRAM 0..511, 0..255 black
        .word 0x02000000
        .word 0x00000000
        .word 0x01000200
        .word 0
go:     or a1, ra, zero
        jal send
        nop
# the texture: GP0(A0h) to 512,0, 256x2, a word of two texels at a time once the FIFO is empty
        lui t0, 0xA000
        sw t0, 0x1810(a0)
        ori t0, zero, 512
        sw t0, 0x1810(a0)
        lui t0, 2
        ori t0, t0, 256
        sw t0, 0x1810(a0)
        lui t3, 0x8000
        ori t3, t3, 0x8000            # texels 0 and 1 of row 0, less the second's 1
        lui t4, 1                     # the second texel's 1
        addiu t5, zero, 256
tex:    lw t0, 0x1814(a0)
        lui t2, 0x1000
        and t0, t0, t2
        beq t0, zero, tex
        nop
        addu t0, t3, t4
        sw t0, 0x1810(a0)
        lui t6, 2
        addiu t6, t6, 2
        addu t3, t3, t6               # the next pair
        addiu t5, t5, -1
        addiu t7, zero, 128
        bne t5, t7, nextrow
        nop
        lui t3, 0x8100
        ori t3, t3, 0x8100            # row 1 starts at U 0
nextrow: bne t5, zero, tex
        nop
        bgezal zero, go2
        nop
rects:
        .word 5                      # flip none at x 0, U 0, V 0, 8 wide
        .word 0xE1000508
        .word 0x65808080
        .word 0x00000000
        .word 0x00000000
        .word 0x00020008
        .word 5                      # flip x at x 16, U 0, V 0, 8 wide
        .word 0xE1001508
        .word 0x65808080
        .word 0x00000010
        .word 0x00000000
        .word 0x00020008
        .word 5                      # flip x at x 33, U 0, V 0, 8 wide
        .word 0xE1001508
        .word 0x65808080
        .word 0x00000021
        .word 0x00000000
        .word 0x00020008
        .word 5                      # flip x at x 48, U 1, V 0, 8 wide
        .word 0xE1001508
        .word 0x65808080
        .word 0x00000030
        .word 0x00000001
        .word 0x00020008
        .word 5                      # flip x at x 65, U 1, V 0, 8 wide
        .word 0xE1001508
        .word 0x65808080
        .word 0x00000041
        .word 0x00000001
        .word 0x00020008
        .word 5                      # flip x at x 80, U 10, V 0, 8 wide
        .word 0xE1001508
        .word 0x65808080
        .word 0x00000050
        .word 0x0000000A
        .word 0x00020008
        .word 5                      # flip x at x 97, U 11, V 0, 8 wide
        .word 0xE1001508
        .word 0x65808080
        .word 0x00000061
        .word 0x0000000B
        .word 0x00020008
        .word 5                      # flip y at x 112, U 0, V 0, 8 wide
        .word 0xE1002508
        .word 0x65808080
        .word 0x00000070
        .word 0x00000000
        .word 0x00020008
        .word 5                      # flip y at x 128, U 0, V 1, 8 wide
        .word 0xE1002508
        .word 0x65808080
        .word 0x00000080
        .word 0x00000100
        .word 0x00020008
        .word 5                      # flip xy at x 144, U 0, V 0, 8 wide
        .word 0xE1003508
        .word 0x65808080
        .word 0x00000090
        .word 0x00000000
        .word 0x00020008
        .word 5                      # flip xy at x 161, U 5, V 1, 8 wide
        .word 0xE1003508
        .word 0x65808080
        .word 0x000000A1
        .word 0x00000105
        .word 0x00020008
        .word 5                      # flip x at x 176, U 0, V 0, 1 wide
        .word 0xE1001508
        .word 0x65808080
        .word 0x000000B0
        .word 0x00000000
        .word 0x00020001
        .word 0
go2:    or a1, ra, zero
        jal send
        nop
        lui t9, 0x8010
        or a1, zero, zero
        lui a2, 2
        ori a2, a2, 192
        jal readback
        ori a3, zero, 192
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
