# GPUSTAT's ready bits before, during and after drawing (Diamond_GPU.md §2.3), in the four ways ps1-tests' gpu/gpustat
# draws: by the CPU with the DMA direction off, by a linked-list DMA, and by block DMA with directions 1 and 2. Each
# draws two 128x128 rectangles but the first, which draws one. Results from 80100000h: twelve GPUSTAT values, before,
# during (300 cycles into a DMA) and after each way; 800FFFF0h holds D1A0D00Eh when done. The lists are at 80100100h,
# 80100180h and 80100200h.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x1F80
        lui s0, 0x8010
        ori t0, zero, 0x0800          # DPCR: the GPU's channel enabled
        sw t0, 0x10F0(a0)
        sw zero, 0x1814(a0)           # GP1(00h)
        lui t0, 0x0300
        sw t0, 0x1814(a0)             # GP1(03h): display on
        lui t0, 0xE400
        lui t1, 0x0007
        ori t1, t1, 0xFFFF
        or t0, t0, t1
        sw t0, 0x1810(a0)             # GP0(E4h): area to 1023,511, as the C program's PutDrawEnv sets it
# by the CPU, direction 0
        lui t0, 0x0400
        sw t0, 0x1814(a0)
        jal status
        nop
        sw v0, 0(s0)
        lui t0, 0x6000
        ori t0, t0, 0x0080            # a 128x128 rectangle at (0, 0)
        sw t0, 0x1810(a0)
        sw zero, 0x1810(a0)
        lui t0, 0x0080
        ori t0, t0, 0x0080
        sw t0, 0x1810(a0)
        jal status
        nop
        sw v0, 4(s0)
        jal wait
        nop
        jal status
        nop
        sw v0, 8(s0)
# a linked list, direction 2
        lui t0, 0x0400
        ori t0, t0, 2
        sw t0, 0x1814(a0)
        addiu t1, s0, 0x100           # two nodes of three words
        lui t0, 0x0310
        ori t0, t0, 0x0110
        sw t0, 0(t1)
        lui t0, 0x6000
        ori t0, t0, 0x8000
        sw t0, 4(t1)
        addiu t0, zero, 0x80
        sw t0, 8(t1)
        lui t0, 0x0080
        ori t0, t0, 0x0080
        sw t0, 12(t1)
        lui t0, 0x03FF
        ori t0, t0, 0xFFFF
        sw t0, 16(t1)
        lui t0, 0x6080
        sw t0, 20(t1)
        addiu t0, zero, 0x100
        sw t0, 24(t1)
        lui t0, 0x0080
        ori t0, t0, 0x0080
        sw t0, 28(t1)
        jal status
        nop
        sw v0, 12(s0)
        lui a1, 0x0100
        ori a1, a1, 0x0401
        jal setup
        or a2, zero, zero
        jal short
        nop
        jal status
        nop
        sw v0, 16(s0)
        jal wait
        nop
        jal status
        nop
        sw v0, 20(s0)
# blocks of one word, direction 1, then direction 2
        addiu s1, zero, 1
again:  lui t0, 0x0400
        addiu t2, s1, 0
        or t0, t0, t2
        sw t0, 0x1814(a0)
        sll t3, s1, 7
        addiu t1, s0, 0x100
        addu t1, t1, t3               # 80100180h, then 80100200h
        sw zero, 0(t1)
        lui t0, 0x6000
        ori t0, t0, 0x8080
        sw t0, 4(t1)
        lui t0, 0x0080
        sw t0, 8(t1)
        lui t0, 0x0080
        ori t0, t0, 0x0080
        sw t0, 12(t1)
        sw zero, 16(t1)
        lui t0, 0x6080
        ori t0, t0, 0x8000
        sw t0, 20(t1)
        lui t0, 0x0080
        ori t0, t0, 0x0080
        sw t0, 24(t1)
        sw t0, 28(t1)
        addiu t4, s1, -1
        addiu t5, zero, 12
        mult t4, t5
        mflo t4
        addu t6, s0, t4               # results from 80100018h, then 80100024h
        jal status
        nop
        sw v0, 24(t6)
        lui a1, 0x0100
        ori a1, a1, 0x0201
        lui a2, 0x0008
        jal setup
        ori a2, a2, 1
        jal short
        nop
        jal status
        nop
        sw v0, 28(t6)
        jal wait
        nop
        jal status
        nop
        sw v0, 32(t6)
        addiu t0, zero, 2
        bne s1, t0, again
        addiu s1, s1, 1
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(s0)
done:   j done
        nop
# GPUSTAT in v0
status: lw v0, 0x1814(a0)
        jr ra
        nop
# DMA 2 from t1 with BCR a2 and CHCR a1, once the channel is idle
setup:  lw t0, 0x10A8(a0)
        lui t7, 0x0100
        and t0, t0, t7
        bne t0, zero, setup
        nop
        sw t1, 0x10A0(a0)
        sw a2, 0x10A4(a0)
        sw a1, 0x10A8(a0)
        jr ra
        nop
# about 300 cycles: into a DMA but well before its first rectangle is drawn
short:  addiu t0, zero, 100
s1:     addiu t0, t0, -1
        bne t0, zero, s1
        nop
        jr ra
        nop
# about 400,000 cycles
wait:   lui t0, 0x0002
w1:     addiu t0, t0, -1
        bne t0, zero, w1
        nop
        jr ra
        nop
