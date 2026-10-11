# D-11: the drawing time of small primitives (Diamond_Disputes.md D-11, Diamond_GPU.md §8). Each record of the
# table is written by the CPU once the GPU is idle (GPUSTAT bits 26 and 28 set) and a while more, with timer 2 on the
# system clock started from 0 at the first word: the timer read after the last word is written, then at the first
# poll that finds bit 26 set. Results from 80100000h, two halfwords a record in one word (after the words, ready),
# in the table's order, 38 records: the four settings, flat rectangles of 1x1 to 128x64 and GP0(68h)'s 1x1, fills,
# flat triangles, flat lines, copies and a one-word CPU-to-VRAM transfer. 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x1F80
        sw zero, 0x1814(a0)           # GP1(00h)
        lui s0, 0x8010
        bgezal zero, go               # ra: the table's address
        nop
list:
        .word 1                     # GP0(E1h): dithering off, drawing to the display allowed
        .word 0xE1000400
        .word 1                     # GP0(E3h): area from 0,0
        .word 0xE3000000
        .word 1                     # GP0(E4h): area to 1023,511
        .word 0xE407FFFF
        .word 1                     # GP0(E5h): no offset
        .word 0xE5000000
        .word 3                     # a flat rectangle 1x1 at 0,256
        .word 0x60204080
        .word 0x01000000
        .word 0x00010001
        .word 3                     # a flat rectangle 2x1 at 0,256
        .word 0x60204080
        .word 0x01000000
        .word 0x00010002
        .word 3                     # a flat rectangle 8x1 at 0,256
        .word 0x60204080
        .word 0x01000000
        .word 0x00010008
        .word 3                     # a flat rectangle 32x1 at 0,256
        .word 0x60204080
        .word 0x01000000
        .word 0x00010020
        .word 3                     # a flat rectangle 128x1 at 0,256
        .word 0x60204080
        .word 0x01000000
        .word 0x00010080
        .word 3                     # a flat rectangle 1x2 at 0,256
        .word 0x60204080
        .word 0x01000000
        .word 0x00020001
        .word 3                     # a flat rectangle 1x8 at 0,256
        .word 0x60204080
        .word 0x01000000
        .word 0x00080001
        .word 3                     # a flat rectangle 1x32 at 0,256
        .word 0x60204080
        .word 0x01000000
        .word 0x00200001
        .word 3                     # a flat rectangle 1x128 at 0,256
        .word 0x60204080
        .word 0x01000000
        .word 0x00800001
        .word 3                     # a flat rectangle 8x8 at 0,256
        .word 0x60204080
        .word 0x01000000
        .word 0x00080008
        .word 3                     # a flat rectangle 32x32 at 0,256
        .word 0x60204080
        .word 0x01000000
        .word 0x00200020
        .word 3                     # a flat rectangle 64x64 at 0,256
        .word 0x60204080
        .word 0x01000000
        .word 0x00400040
        .word 3                     # a flat rectangle 128x64 at 0,256
        .word 0x60204080
        .word 0x01000000
        .word 0x00400080
        .word 2                     # GP0(68h): a flat 1x1 rectangle at 0,256
        .word 0x68204080
        .word 0x01000000
        .word 3                     # a fill 16x1 at 0,256
        .word 0x02204080
        .word 0x01000000
        .word 0x00010010
        .word 3                     # a fill 16x2 at 0,256
        .word 0x02204080
        .word 0x01000000
        .word 0x00020010
        .word 3                     # a fill 16x8 at 0,256
        .word 0x02204080
        .word 0x01000000
        .word 0x00080010
        .word 3                     # a fill 16x32 at 0,256
        .word 0x02204080
        .word 0x01000000
        .word 0x00200010
        .word 3                     # a fill 16x128 at 0,256
        .word 0x02204080
        .word 0x01000000
        .word 0x00800010
        .word 3                     # a fill 64x1 at 0,256
        .word 0x02204080
        .word 0x01000000
        .word 0x00010040
        .word 3                     # a fill 256x1 at 0,256
        .word 0x02204080
        .word 0x01000000
        .word 0x00010100
        .word 3                     # a fill 64x64 at 0,256
        .word 0x02204080
        .word 0x01000000
        .word 0x00400040
        .word 4                     # a flat triangle of one pixel
        .word 0x20204080
        .word 0x01000000
        .word 0x01000001
        .word 0x01010000
        .word 4                     # a flat triangle, legs of 8
        .word 0x20204080
        .word 0x01000000
        .word 0x01000008
        .word 0x01080000
        .word 4                     # a flat triangle, legs of 32
        .word 0x20204080
        .word 0x01000000
        .word 0x01000020
        .word 0x01200000
        .word 4                     # a flat triangle, legs of 128
        .word 0x20204080
        .word 0x01000000
        .word 0x01000080
        .word 0x01800000
        .word 3                     # a flat line of one pixel
        .word 0x40204080
        .word 0x01000000
        .word 0x01000000
        .word 3                     # a flat line 8 across
        .word 0x40204080
        .word 0x01000000
        .word 0x01000007
        .word 3                     # a flat line 64 across
        .word 0x40204080
        .word 0x01000000
        .word 0x0100003F
        .word 3                     # a flat line 64 down
        .word 0x40204080
        .word 0x01000000
        .word 0x013F0000
        .word 4                     # a copy 1x1 from 0,256 to 512,256
        .word 0x80000000
        .word 0x01000000
        .word 0x01000200
        .word 0x00010001
        .word 4                     # a copy 16x1
        .word 0x80000000
        .word 0x01000000
        .word 0x01000200
        .word 0x00010010
        .word 4                     # a copy 16x16
        .word 0x80000000
        .word 0x01000000
        .word 0x01000200
        .word 0x00100010
        .word 4                     # a CPU-to-VRAM transfer of 2x1, one word
        .word 0xA0000000
        .word 0x01000000
        .word 0x00010002
        .word 0x7FFF7FFF
        .word 0
go:     or a1, ra, zero
next:   lw t1, 0(a1)
        addiu a1, a1, 4
        beq t1, zero, finish
        nop
idle:   lw t0, 0x1814(a0)
        lui t2, 0x1400
        and t0, t0, t2
        bne t0, t2, idle
        nop
        ori t0, zero, 0x1000
w1:     addiu t0, t0, -1
        bne t0, zero, w1
        nop
        sw zero, 0x1124(a0)           # timer 2 from 0, the system clock
word:   lw t0, 0(a1)
        addiu a1, a1, 4
        addiu t1, t1, -1
        bne t1, zero, word
        sw t0, 0x1810(a0)
        lhu t4, 0x1120(a0)
        lui t2, 0x0400
poll:   lw t0, 0x1814(a0)
        lhu t5, 0x1120(a0)
        and t0, t0, t2
        beq t0, zero, poll
        nop
        sll t5, t5, 16
        or t5, t5, t4
        sw t5, 0(s0)
        j next
        addiu s0, s0, 4
finish: lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        lui t0, 0x8010
        sw t1, -16(t0)
done:   j done
        nop
