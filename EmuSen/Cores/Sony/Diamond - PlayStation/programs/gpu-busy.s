# D-11: how long the GPU stays busy after a DMA of 2,048 words to VRAM (Diamond_Disputes.md D-11), the shape of
# ps1-tests' dma/chopping: GP0(A0h) by the CPU for 128x32 pixels at 0,256, then DMA channel 2 in blocks of 1 to 128
# words (sync mode 1), and in one burst (mode 0). Timer 2 on the system clock from the channel's start: the cycle the
# channel is seen idle, the first poll that finds GPUSTAT bit 28 set, and the first that finds bit 26 set. Results
# from 80100000h, four words a case (those three and GPUSTAT as the channel ends), blocks of 1, 2, 4 ... 128, then
# the burst. 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x1F80
        lui s0, 0x8010
        sw zero, 0x1814(a0)           # GP1(00h)
        ori t0, zero, 0x0800
        sw t0, 0x10F0(a0)             # DPCR: the GPU's channel enabled
        lui t0, 0x0400
        ori t0, t0, 2
        sw t0, 0x1814(a0)             # GP1(04h): DMA direction 2, CPU to GPU
        addiu s1, zero, 1             # the block size
        or s2, s0, zero
case:   jal idle
        nop
        lui t0, 0xA000
        sw t0, 0x1810(a0)
        lui t0, 0x0100
        sw t0, 0x1810(a0)             # 0,256
        lui t0, 0x0020
        ori t0, t0, 0x0080
        sw t0, 0x1810(a0)             # 128x32
        lui t0, 0x8002
        sw t0, 0x10A0(a0)             # MADR: any RAM
        ori t0, zero, 0x0800
        divu t0, s1
        mflo t1
        sll t1, t1, 16
        or t1, t1, s1
        sw t1, 0x10A4(a0)             # BCR: 2048 / s1 blocks of s1
        lui t3, 0x0100
        ori t3, t3, 0x0201            # CHCR: mode 1, from RAM, start
        sw zero, 0x1124(a0)           # timer 2 from 0, the system clock
        jal measure
        sw t3, 0x10A8(a0)
        addiu s2, s2, 16
        addiu t0, zero, 128
        bne s1, t0, case
        sll s1, s1, 1
# the burst
        jal idle
        nop
        lui t0, 0xA000
        sw t0, 0x1810(a0)
        lui t0, 0x0100
        sw t0, 0x1810(a0)
        lui t0, 0x0020
        ori t0, t0, 0x0080
        sw t0, 0x1810(a0)
        lui t0, 0x8002
        sw t0, 0x10A0(a0)
        ori t0, zero, 0x0800
        sw t0, 0x10A4(a0)             # BCR: 2048 words
        lui t3, 0x1100
        ori t3, t3, 0x0001            # CHCR: mode 0, from RAM, start and trigger
        sw zero, 0x1124(a0)
        jal measure
        sw t3, 0x10A8(a0)
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(s0)
done:   j done
        nop
# once the channel is idle: its end, then bits 28 and 26, to s2
measure: lw t0, 0x10A8(a0)
        lui t2, 0x0100
        and t0, t0, t2
        bne t0, zero, measure
        nop
        lhu t4, 0x1120(a0)
        lw t5, 0x1814(a0)
        sw t4, 0(s2)
        sw t5, 12(s2)
        or t6, zero, zero             # bit 28 seen
        or t7, zero, zero             # bit 26 seen
m1:     lw t0, 0x1814(a0)
        lhu t4, 0x1120(a0)
        lui t2, 0x1000
        and t1, t0, t2
        beq t1, zero, m2
        nop
        bne t6, zero, m2
        nop
        sw t4, 4(s2)
        addiu t6, zero, 1
m2:     lui t2, 0x0400
        and t1, t0, t2
        beq t1, zero, m3
        nop
        bne t7, zero, m3
        nop
        sw t4, 8(s2)
        addiu t7, zero, 1
m3:     and t1, t6, t7
        beq t1, zero, m1
        nop
        jr ra
        nop
# until GPUSTAT bits 26 and 28 are set and the channel is idle, and some time more
idle:   lw t0, 0x1814(a0)
        lui t2, 0x1400
        and t0, t0, t2
        bne t0, t2, idle
        nop
        lui t0, 0x0002
w1:     addiu t0, t0, -1
        bne t0, zero, w1
        nop
        jr ra
        nop
