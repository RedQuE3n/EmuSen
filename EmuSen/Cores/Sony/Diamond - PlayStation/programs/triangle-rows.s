# D-41: the order a triangle's rows are drawn in, read through the texture cache (Diamond_Disputes.md D-41,
# Diamond_GPU.md §6). A 4-bit page at 256,0, palette at 0,500 reading 8000h + i, is filled with texels 1 and the cache
# cleared; a raw textured triangle or quad, U = 40h + x and V = 40h + y, is drawn at 0,0, so texture lines y and
# y + 64 share cache entries; the page is filled with texels 2, and a 64x64 rectangle from texel 40h,40h drawn at
# 100,0 reads 8001h where its entry was kept from the shape's last fetch of that line and 8002h where not. Seven
# shapes; each one's columns 0, 16, 32 and 48 (one pixel pair an entry) read back to 80100000h on, 64 lines a
# column, 256 words a shape. 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x1F80
        sw zero, 0x1814(a0)           # GP1(00h)
# the palette: GP0(A0h) to 0,500, 16x1, entry i 8000h + i, once the FIFO is empty
        lui t0, 0xA000
        sw t0, 0x1810(a0)
        lui t0, 500
        sw t0, 0x1810(a0)
        lui t0, 1
        ori t0, t0, 16
        sw t0, 0x1810(a0)
        lui t3, 0x8001
        ori t3, t3, 0x8000            # entries 0 and 1
        lui t6, 2
        addiu t6, t6, 2
        addiu t5, zero, 8
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
# records of a count and that many GP0 words, -1 for the read-back, 0 to end
list:
        .word 4                      # a 4-bit page at 256,0; the area 0,0 to 1023,511; no offset
        .word 0xE1000404
        .word 0xE3000000
        .word 0xE407FFFF
        .word 0xE5000000
# 1. flat at the bottom: (64,0) (0,72) (64,72)
        .word 4
        .word 0x02204088
        .word 0x00000100
        .word 0x01000100
        .word 0x01000000
        .word 7
        .word 0x25808080
        .word 0x00000040
        .word 0x7D004080
        .word 0x00480000
        .word 0x00048840
        .word 0x00480040
        .word 0x00008880
        .word 3
        .word 0x02408810
        .word 0x00000100
        .word 0x01000100
        .word 4
        .word 0x65808080
        .word 0x00000064
        .word 0x7D004040
        .word 0x00400040
        .word -1
# 2. flat at the top: (0,0) (64,0) (0,72)
        .word 4
        .word 0x02204088
        .word 0x00000100
        .word 0x01000100
        .word 0x01000000
        .word 7
        .word 0x25808080
        .word 0x00000000
        .word 0x7D004040
        .word 0x00000040
        .word 0x00044080
        .word 0x00480000
        .word 0x00008840
        .word 3
        .word 0x02408810
        .word 0x00000100
        .word 0x01000100
        .word 4
        .word 0x65808080
        .word 0x00000064
        .word 0x7D004040
        .word 0x00400040
        .word -1
# 3. the middle vertex low on the left: (32,0) (0,72) (64,80)
        .word 4
        .word 0x02204088
        .word 0x00000100
        .word 0x01000100
        .word 0x01000000
        .word 7
        .word 0x25808080
        .word 0x00000020
        .word 0x7D004060
        .word 0x00480000
        .word 0x00048840
        .word 0x00500040
        .word 0x00009080
        .word 3
        .word 0x02408810
        .word 0x00000100
        .word 0x01000100
        .word 4
        .word 0x65808080
        .word 0x00000064
        .word 0x7D004040
        .word 0x00400040
        .word -1
# 4. the middle vertex low on the right: (32,0) (64,72) (0,80)
        .word 4
        .word 0x02204088
        .word 0x00000100
        .word 0x01000100
        .word 0x01000000
        .word 7
        .word 0x25808080
        .word 0x00000020
        .word 0x7D004060
        .word 0x00480040
        .word 0x00048880
        .word 0x00500000
        .word 0x00009040
        .word 3
        .word 0x02408810
        .word 0x00000100
        .word 0x01000100
        .word 4
        .word 0x65808080
        .word 0x00000064
        .word 0x7D004040
        .word 0x00400040
        .word -1
# 5. the middle vertex high on the left: (32,0) (0,8) (64,80)
        .word 4
        .word 0x02204088
        .word 0x00000100
        .word 0x01000100
        .word 0x01000000
        .word 7
        .word 0x25808080
        .word 0x00000020
        .word 0x7D004060
        .word 0x00080000
        .word 0x00044840
        .word 0x00500040
        .word 0x00009080
        .word 3
        .word 0x02408810
        .word 0x00000100
        .word 0x01000100
        .word 4
        .word 0x65808080
        .word 0x00000064
        .word 0x7D004040
        .word 0x00400040
        .word -1
# 6. shape 3 with its vertices in another order: (0,72) (64,80) (32,0)
        .word 4
        .word 0x02204088
        .word 0x00000100
        .word 0x01000100
        .word 0x01000000
        .word 7
        .word 0x25808080
        .word 0x00480000
        .word 0x7D008840
        .word 0x00500040
        .word 0x00049080
        .word 0x00000020
        .word 0x00004060
        .word 3
        .word 0x02408810
        .word 0x00000100
        .word 0x01000100
        .word 4
        .word 0x65808080
        .word 0x00000064
        .word 0x7D004040
        .word 0x00400040
        .word -1
# 7. a quad: (0,0) (64,0) (0,72) (64,72)
        .word 4
        .word 0x02204088
        .word 0x00000100
        .word 0x01000100
        .word 0x01000000
        .word 9
        .word 0x2D808080
        .word 0x00000000
        .word 0x7D004040
        .word 0x00000040
        .word 0x00044080
        .word 0x00480000
        .word 0x00008840
        .word 0x00480040
        .word 0x00008880
        .word 3
        .word 0x02408810
        .word 0x00000100
        .word 0x01000100
        .word 4
        .word 0x65808080
        .word 0x00000064
        .word 0x7D004040
        .word 0x00400040
        .word -1
        .word 0
go:     or s0, ra, zero
        lui t9, 0x8010
next:   lw t1, 0(s0)
        addiu s0, s0, 4
        beq t1, zero, finish
        nop
        addiu t2, zero, -1
        beq t1, t2, cols
        nop
idle:   lw t0, 0x1814(a0)
        lui t2, 0x1400
        and t0, t0, t2
        bne t0, t2, idle
        nop
word:   lw t0, 0(s0)
        addiu s0, s0, 4
        addiu t1, t1, -1
        bne t1, zero, word
        sw t0, 0x1810(a0)
        j next
        nop
# columns 0, 16, 32 and 48 of the rectangle at 100,0: a pixel pair a line, 64 lines each
cols:   addiu s1, zero, 100
col:    or a1, s1, zero
        lui a2, 64
        ori a2, a2, 2
        jal readback
        ori a3, zero, 64
        addiu s1, s1, 16
        addiu t2, zero, 164
        bne s1, t2, col
        nop
        j next
        nop
finish: lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        lui t0, 0x8010
        sw t1, -16(t0)
done:   j done
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
