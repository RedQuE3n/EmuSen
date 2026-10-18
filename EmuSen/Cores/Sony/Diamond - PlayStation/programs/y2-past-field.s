# Y2 at or past the field's last line (Diamond_Native.md §29.10). psx-spx, GP1(07h): "If Y2 is set to a much too large
# value, then the hardware stops to generate vblank interrupts (IRQ0)." Interrupts off, NTSC at 320, 263 lines a field.
# Each case sets the range 16-256, lets two vblank edges pass, takes the next one (line 256) as its reference with timer 1
# counting hblanks from it, and at count 107 (line 100) writes Y1 16 and the case's Y2: 0, 256; 1, 262; 2, 263, the
# field's line count; 3, 300. To count 538 (line 5, two fields on) it counts the vblank interrupts (I_STAT bit 0) and
# keeps the count at the first, and samples GPUSTAT bit 31 as the count changes from 275 (line 5 of the next field) for
# eight lines, the first sample in the high bit. At count 538 it writes timer 1's mode, sync mode 0 on the system clock,
# and timer 2's, the system clock from zero, and reads timer 1 at once and when timer 2 reaches 3,228, 6,457 and 43,051
# (1.5, 3 and 20 lines). Results from 80100000h, 20h a case: the interrupts, the first one's count, the samples, the
# four reads. 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x1F80
        sw zero, 0x1074(a0)           # I_MASK: nothing reaches the CPU
        sw zero, 0x1814(a0)           # GP1(00h)
        lui t0, 0x0800
        ori t0, t0, 0x0001
        sw t0, 0x1814(a0)             # GP1(08h): NTSC, 320
        sw zero, 0x1124(a0)           # timer 2: the system clock, free-running
        lui s0, 0x8010
        or s5, s0, zero
        lui a2, 0x0704                # Y1 16, Y2 256
        jal case
        ori a2, a2, 0x0010
        addiu s5, s0, 0x20
        lui a2, 0x0704                # Y2 262
        jal case
        ori a2, a2, 0x1810
        addiu s5, s0, 0x40
        lui a2, 0x0704                # Y2 263
        jal case
        ori a2, a2, 0x1C10
        addiu s5, s0, 0x60
        lui a2, 0x0704                # Y2 300
        jal case
        ori a2, a2, 0xB010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(s0)
done:   j done
        nop
# the range 16-256, two edges let pass, and back at the third, at line 256
settle: lui t0, 0x0704
        ori t0, t0, 0x0010
        sw t0, 0x1814(a0)
        addiu t3, zero, 3
sedge:  addiu t0, zero, -2
        sw t0, 0x1070(a0)
spoll:  lw t0, 0x1070(a0)
        nop
        andi t0, t0, 1
        beq t0, zero, spoll
        nop
        addiu t3, t3, -1
        bne t3, zero, sedge
        nop
        addiu t0, zero, -2
        jr ra
        sw t0, 0x1070(a0)
# one case: the GP1(07h) word a2, its results at s5
case:   or s7, ra, zero
        jal settle
        nop
        addiu t0, zero, 0x0100
        sh t0, 0x1114(a0)             # timer 1: hblanks, from zero
        addiu a1, zero, 107
w1:     lhu t0, 0x1110(a0)
        nop
        bne t0, a1, w1
        nop
        sw a2, 0x1814(a0)             # the case's range
        or s1, t0, zero               # the last count seen
        or s2, zero, zero             # samples
        or s3, zero, zero             # their bits
        or s4, zero, zero             # interrupts
        or s6, zero, zero             # the first one's count
        addiu a3, zero, 538
l1:     lhu t0, 0x1110(a0)
        nop
        beq t0, s1, noline
        nop
        or s1, t0, zero
        sltiu t1, t0, 275
        bne t1, zero, noline
        nop
        sltiu t1, s2, 8
        beq t1, zero, noline
        nop
        lw t2, 0x1814(a0)
        sll s3, s3, 1
        srl t2, t2, 31
        or s3, s3, t2
        addiu s2, s2, 1
noline: lw t1, 0x1070(a0)
        nop
        andi t1, t1, 1
        beq t1, zero, noedge
        nop
        addiu t2, zero, -2
        sw t2, 0x1070(a0)
        bne s4, zero, notfirst
        nop
        or s6, t0, zero
notfirst: addiu s4, s4, 1
noedge: bne t0, a3, l1
        nop
        addiu t0, zero, 0x0001
        sh t0, 0x1114(a0)             # timer 1: sync mode 0, the system clock
        sh zero, 0x1124(a0)           # timer 2: the system clock, from zero
        lhu t2, 0x1110(a0)
        nop
        sw t2, 12(s5)
        addiu t1, zero, 3228
        jal at
        nop
        sw t2, 16(s5)
        addiu t1, zero, 6457
        jal at
        nop
        sw t2, 20(s5)
        lui t1, 0
        ori t1, t1, 43051
        jal at
        nop
        sw t2, 24(s5)
        sw s4, 0(s5)
        sw s6, 4(s5)
        sw s3, 8(s5)
        jr s7
        nop
# timer 1's count in t2 once timer 2 reaches t1
at:     lhu t3, 0x1120(a0)
        nop
        sltu t3, t3, t1
        bne t3, zero, at
        nop
        lhu t2, 0x1110(a0)
        jr ra
        nop
