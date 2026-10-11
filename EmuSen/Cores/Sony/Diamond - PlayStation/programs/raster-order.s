# D-7: where the rasteriser's interpolation starts (Diamond_Disputes.md D-7). One shaded triangle drawn in each of its
# six vertex orders, side by side 64 pixels apart, and one shaded quad in the four orders that keep the diagonal from
# its second vertex to its third, below; then a triangle with two vertices in its leftmost column, in its six orders;
# dithering off. VRAM is then read back two pixels a word: the triangles' 384x32 from 0,0 to 80100000h, the quads'
# 256x48 from 0,64 to 80106000h, the second triangles' 384x32 from 0,128 to 8010C000h. 800FFFF0h holds D1A0D00Eh
# when done.
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
        .word 6                      # a shaded triangle, vertices ABC, at x 0
        .word 0x30000AFA
        .word 0x00010002
        .word 0x005AC803
        .word 0x00090028
        .word 0x00FF854D
        .word 0x001E000D
        .word 6                      # a shaded triangle, vertices ACB, at x 64
        .word 0x30000AFA
        .word 0x00010042
        .word 0x00FF854D
        .word 0x001E004D
        .word 0x005AC803
        .word 0x00090068
        .word 6                      # a shaded triangle, vertices BAC, at x 128
        .word 0x305AC803
        .word 0x000900A8
        .word 0x00000AFA
        .word 0x00010082
        .word 0x00FF854D
        .word 0x001E008D
        .word 6                      # a shaded triangle, vertices BCA, at x 192
        .word 0x305AC803
        .word 0x000900E8
        .word 0x00FF854D
        .word 0x001E00CD
        .word 0x00000AFA
        .word 0x000100C2
        .word 6                      # a shaded triangle, vertices CAB, at x 256
        .word 0x30FF854D
        .word 0x001E010D
        .word 0x00000AFA
        .word 0x00010102
        .word 0x005AC803
        .word 0x00090128
        .word 6                      # a shaded triangle, vertices CBA, at x 320
        .word 0x30FF854D
        .word 0x001E014D
        .word 0x005AC803
        .word 0x00090168
        .word 0x00000AFA
        .word 0x00010142
        .word 6                      # the same with C at A's column, vertices ABC, at x 0
        .word 0x30000AFA
        .word 0x00810002
        .word 0x005AC803
        .word 0x00890028
        .word 0x00FF854D
        .word 0x009E0002
        .word 6                      # the same with C at A's column, vertices ACB, at x 64
        .word 0x30000AFA
        .word 0x00810042
        .word 0x00FF854D
        .word 0x009E0042
        .word 0x005AC803
        .word 0x00890068
        .word 6                      # the same with C at A's column, vertices BAC, at x 128
        .word 0x305AC803
        .word 0x008900A8
        .word 0x00000AFA
        .word 0x00810082
        .word 0x00FF854D
        .word 0x009E0082
        .word 6                      # the same with C at A's column, vertices BCA, at x 192
        .word 0x305AC803
        .word 0x008900E8
        .word 0x00FF854D
        .word 0x009E00C2
        .word 0x00000AFA
        .word 0x008100C2
        .word 6                      # the same with C at A's column, vertices CAB, at x 256
        .word 0x30FF854D
        .word 0x009E0102
        .word 0x00000AFA
        .word 0x00810102
        .word 0x005AC803
        .word 0x00890128
        .word 6                      # the same with C at A's column, vertices CBA, at x 320
        .word 0x30FF854D
        .word 0x009E0142
        .word 0x005AC803
        .word 0x00890168
        .word 0x00000AFA
        .word 0x00810142
        .word 8                      # a shaded quad, vertices 0123, its diagonal 1-2 throughout, at x 0
        .word 0x380514F0
        .word 0x00420003
        .word 0x0046B40A
        .word 0x0046002D
        .word 0x00FA1E5A
        .word 0x00680008
        .word 0x0082FAA0
        .word 0x006C0032
        .word 8                      # a shaded quad, vertices 0213, its diagonal 1-2 throughout, at x 64
        .word 0x380514F0
        .word 0x00420043
        .word 0x00FA1E5A
        .word 0x00680048
        .word 0x0046B40A
        .word 0x0046006D
        .word 0x0082FAA0
        .word 0x006C0072
        .word 8                      # a shaded quad, vertices 3120, its diagonal 1-2 throughout, at x 128
        .word 0x3882FAA0
        .word 0x006C00B2
        .word 0x0046B40A
        .word 0x004600AD
        .word 0x00FA1E5A
        .word 0x00680088
        .word 0x000514F0
        .word 0x00420083
        .word 8                      # a shaded quad, vertices 3210, its diagonal 1-2 throughout, at x 192
        .word 0x3882FAA0
        .word 0x006C00F2
        .word 0x00FA1E5A
        .word 0x006800C8
        .word 0x0046B40A
        .word 0x004600ED
        .word 0x000514F0
        .word 0x004200C3
        .word 0
go:     or a1, ra, zero
        jal send
        nop
        lui t9, 0x8010
        or a1, zero, zero
        lui a2, 32
        ori a2, a2, 384
        jal readback
        ori a3, zero, 6144
        lui t9, 0x8010
        ori t9, t9, 0x6000
        lui a1, 64
        lui a2, 48
        ori a2, a2, 256
        jal readback
        ori a3, zero, 6144
        lui t9, 0x8010
        ori t9, t9, 0xC000
        lui a1, 128
        lui a2, 32
        ori a2, a2, 384
        jal readback
        ori a3, zero, 6144
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
