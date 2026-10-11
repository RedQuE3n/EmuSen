# D-23: the texture cache's shape at 8 bits and what clears it (Diamond_Disputes.md D-23, Diamond_GPU.md §4). An 8-bit
# page at 64,0 whose palette at 0,500 reads 8000h + i for entry i, its texels all one value set by a fill, is drawn raw
# as rectangles 64 by 1 from texel 0 of line 0 or 32 onto lines 10 to 16, the texels changed by fills between, and the
# page set elsewhere and back before the last three. Lines 10 to 16, 64 pixels each, are read back to 80100000h, two
# pixels a word. 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x1F80
        sw zero, 0x1814(a0)           # GP1(00h)
# the palette: GP0(A0h) to 0,500, 256x1, entry i 8000h + i, once the FIFO is empty
        lui t0, 0xA000
        sw t0, 0x1810(a0)
        lui t0, 500
        sw t0, 0x1810(a0)
        lui t0, 1
        ori t0, t0, 256
        sw t0, 0x1810(a0)
        lui t3, 0x8001
        ori t3, t3, 0x8000            # entries 0 and 1
        lui t6, 2
        addiu t6, t6, 2
        addiu t5, zero, 128
pal:    lw t0, 0x1814(a0)
        lui t2, 0x1000
        and t0, t0, t2
        beq t0, zero, pal
        nop
        sw t3, 0x1810(a0)
        addu t3, t3, t6
        addiu t5, t5, -1
        bne t5, zero, pal
        nop
        bgezal zero, go
        nop
list:
        .word 1                      # GP0(E1h): an 8-bit page at 64,0
        .word 0xE1000081
        .word 1                      # GP0(E3h): area from 0,0
        .word 0xE3000000
        .word 1                      # GP0(E4h): area to 1023,511
        .word 0xE407FFFF
        .word 1                      # GP0(E5h): no offset
        .word 0xE5000000
        .word 4                      # the page's texels all 1, the cache cleared
        .word 0x02004008
        .word 0x00000040
        .word 0x00400040
        .word 0x01000000
        .word 4                      # line 10: texels 0-63 of line 0, fetched
        .word 0x65808080
        .word 0x000A0000
        .word 0x7D000000
        .word 0x00010040
        .word 3                      # the page's texels all 2
        .word 0x02008010
        .word 0x00000040
        .word 0x00400040
        .word 4                      # line 11: line 0 again
        .word 0x65808080
        .word 0x000B0000
        .word 0x7D000000
        .word 0x00010040
        .word 4                      # line 12: line 32
        .word 0x65808080
        .word 0x000C0000
        .word 0x7D002000
        .word 0x00010040
        .word 4                      # line 13: line 0 again
        .word 0x65808080
        .word 0x000D0000
        .word 0x7D000000
        .word 0x00010040
        .word 5                      # texels 3; a 15-bit page and back
        .word 0x0200C018
        .word 0x00000040
        .word 0x00400040
        .word 0xE1000101
        .word 0xE1000081
        .word 4                      # line 14: line 0 again
        .word 0x65808080
        .word 0x000E0000
        .word 0x7D000000
        .word 0x00010040
        .word 5                      # texels 4; a 4-bit page and back
        .word 0x02080020
        .word 0x00000040
        .word 0x00400040
        .word 0xE1000001
        .word 0xE1000081
        .word 4                      # line 15: line 0 again
        .word 0x65808080
        .word 0x000F0000
        .word 0x7D000000
        .word 0x00010040
        .word 5                      # texels 5; the page at 128,0 and back
        .word 0x02084028
        .word 0x00000040
        .word 0x00400040
        .word 0xE1000082
        .word 0xE1000081
        .word 4                      # line 16: line 0 again
        .word 0x65808080
        .word 0x00100000
        .word 0x7D000000
        .word 0x00010040
        .word 0
go:     or a1, ra, zero
        jal send
        nop
        lui t9, 0x8010
        lui a1, 10
        lui a2, 7
        ori a2, a2, 64
        jal readback
        ori a3, zero, 224
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
