# GP1(00h) during a fill (Diamond_GPU.md §2.5, Diamond_Records.md §11-§17): a fill of 1008x511 started, GPUSTAT read
# while it draws (A), GP1(00h) written, GPUSTAT read at once (B) and 100 cycles on (C), the polls until bit 26 sets
# counted (D), and GPUSTAT then (E); results from 80100000h, five words. psx-spx "GP1(00h)" gives GPUSTAT 14802000h
# after a reset; DuckStation, SwanStation, Beetle PSX and PCSX-Redux read it ready at the first poll.
        lui t0, 0x1F80
        mtc0 zero, SR
        sw zero, 0x1814(t0)           # GP1(00h)
        lui t1, 0x0400
        sw t1, 0x1814(t0)             # GP1(04h) 0
        lui t1, 0x0200
        sw t1, 0x1810(t0)             # GP0(02h) black
        sw zero, 0x1810(t0)           # at 0,0
        lui t1, 0x01FF
        ori t1, t1, 0x03F0
        sw t1, 0x1810(t0)             # 1008 x 511
        lui t9, 0x8010
        nop
        nop
        lw t2, 0x1814(t0)
        nop
        sw t2, 0(t9)                  # A: drawing
        sw zero, 0x1814(t0)           # GP1(00h) during the fill
        lw t2, 0x1814(t0)
        nop
        sw t2, 4(t9)                  # B: at once
        ori t3, zero, 50
d1:     bne t3, zero, d1
        addiu t3, t3, -1
        lw t2, 0x1814(t0)
        nop
        sw t2, 8(t9)                  # C: 100 cycles on
        or t4, zero, zero
        lui t5, 0x0400
p1:     lw t2, 0x1814(t0)
        addiu t4, t4, 1
        and t2, t2, t5
        beq t2, zero, p1
        nop
        sw t4, 12(t9)                 # D: polls until bit 26
        lw t2, 0x1814(t0)
        nop
        sw t2, 16(t9)                 # E: then
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(t9)
done:   j done
        nop
