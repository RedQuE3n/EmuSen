# D-42: GP1(01h) written while a fill draws (Diamond_Disputes.md D-42, Diamond_GPU.md §2.5). VRAM cleared black; then
# four white fills of 240x511 at x 0, 256, 512 and 768, each written by the CPU with no wait, followed by a delay loop
# of 0, 100, 400 and 1,600 turns and GP1(01h), the GPU then left to go idle. Column 0 of each band read back, a pixel
# pair a line, 511 words a band, to 80100000h on. 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x1F80
        sw zero, 0x1814(a0)           # GP1(00h)
        jal idle
        nop
        lui t0, 0x0200
        sw t0, 0x1810(a0)             # GP0(02h) black
        sw zero, 0x1810(a0)           # at 0,0
        lui t0, 0x01FF
        ori t0, t0, 0x03F0
        sw t0, 0x1810(a0)             # 1008 x 511
        jal idle
        nop
        or s0, zero, zero             # the band's x
        bgezal zero, go
        nop
delays: .word 0
        .word 100
        .word 400
        .word 1600
go:     or s1, ra, zero
band:   lui t0, 0x02FF
        ori t0, t0, 0xFFFF
        lw t3, 0(s1)
        addiu s1, s1, 4
        sw t0, 0x1810(a0)             # GP0(02h) white
        sw s0, 0x1810(a0)             # at x,0
        lui t0, 0x01FF
        ori t0, t0, 0x00F0
        sw t0, 0x1810(a0)             # 240 x 511
wait:   bne t3, zero, wait
        addiu t3, t3, -1
        lui t0, 0x0100
        sw t0, 0x1814(a0)             # GP1(01h)
        jal idle
        nop
        addiu s0, s0, 256
        addiu t2, zero, 1024
        bne s0, t2, band
        nop
        lui t9, 0x8010
        or s0, zero, zero
col:    or a1, s0, zero
        lui a2, 511
        ori a2, a2, 2
        jal readback
        ori a3, zero, 511
        addiu s0, s0, 256
        addiu t2, zero, 1024
        bne s0, t2, col
        nop
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        lui t0, 0x8010
        sw t1, -16(t0)
done:   j done
        nop
# until GPUSTAT bits 26 and 28 are set
idle:   lw t0, 0x1814(a0)
        lui t2, 0x1400
        and t0, t0, t2
        bne t0, t2, idle
        nop
        jr ra
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
