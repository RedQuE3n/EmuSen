# D-23: what 480-line interlace leaves undrawn (Diamond_Disputes.md D-23, Diamond_GPU.md §5). In 640x480 interlaced
# NTSC, each case waits for vblank's interrupt and then draws, with GP0(E1h) bit 10 clear, a white fill of 16 by 4 and a
# white rectangle of 4 by 4 beside it: at 0,0 with the display starting at line 0, at 0,8 with it starting at line 1;
# then at 64,0 with bit 10 set. Lines 0 to 11, 80 pixels each, are read back to 80100000h, two pixels a word.
# 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x1F80
        sw zero, 0x1814(a0)           # GP1(00h)
        lui t0, 0x0300
        sw t0, 0x1814(a0)             # GP1(03h): display on
        lui t0, 0x0800
        ori t0, t0, 0x27
        sw t0, 0x1814(a0)             # GP1(08h): 640, 480 lines, NTSC, interlaced
        bgezal zero, go
        nop
setup:
        .word 1                      # GP0(E3h): area from 0,0
        .word 0xE3000000
        .word 1                      # GP0(E4h): area to 1023,511
        .word 0xE407FFFF
        .word 1                      # GP0(E5h): no offset
        .word 0xE5000000
        .word 3                      # GP0(02h): 0..95, 0..15 black
        .word 0x02000000
        .word 0x00000000
        .word 0x00100060
        .word 0
go:     or a1, ra, zero
        jal send
        nop
        jal vblank
        nop
        bgezal zero, go1
        nop
case1:
        .word 7                      # bit 10 clear, the display from line 0: a fill at 0,0 and a rectangle at 32,0
        .word 0xE1000000
        .word 0x02FFFFFF
        .word 0x00000000
        .word 0x00040010
        .word 0x60FFFFFF
        .word 0x00000020
        .word 0x00040004
        .word 0
go1:    or a1, ra, zero
        jal send
        nop
        lui t0, 0x0500
        ori t0, t0, 0x0400
        sw t0, 0x1814(a0)             # GP1(05h): the display from line 1
        jal vblank
        nop
        bgezal zero, go2
        nop
case2:
        .word 7                      # bit 10 clear, the display from line 1: a fill at 0,8 and a rectangle at 32,8
        .word 0xE1000000
        .word 0x02FFFFFF
        .word 0x00080000
        .word 0x00040010
        .word 0x60FFFFFF
        .word 0x00080020
        .word 0x00040004
        .word 0
go2:    or a1, ra, zero
        jal send
        nop
        jal vblank
        nop
        bgezal zero, go3
        nop
case3:
        .word 4                      # bit 10 set: a fill at 64,0
        .word 0xE1000400
        .word 0x02FFFFFF
        .word 0x00000040
        .word 0x00040010
        .word 0
go3:    or a1, ra, zero
        jal send
        nop
        lui t9, 0x8010
        or a1, zero, zero
        lui a2, 12
        ori a2, a2, 80
        jal readback
        ori a3, zero, 480
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        lui t0, 0x8010
        sw t1, -16(t0)
done:   j done
        nop
# wait for vblank's interrupt: I_STAT bit 0 acknowledged, then polled
vblank: addiu t0, zero, -2
        sw t0, 0x1070(a0)
vb1:    lw t0, 0x1070(a0)
        nop
        andi t0, t0, 1
        beq t0, zero, vb1
        nop
        jr ra
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
