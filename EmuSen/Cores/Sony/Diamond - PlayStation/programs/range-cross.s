# GP1(07h) moved across or onto the beam's line (Diamond_Native.md §29). Interrupts off, NTSC at 320, the range 16-256
# set and two vblank edges let pass before each case; the next vblank edge, at line 256, is the reference. Nine cases,
# each a line L and a new range written at L's middle: 0, line 100 with the range unchanged; 1, line 100 with Y2 50
# (behind); 2, line 100 with Y2 100 (onto); 3, line 100 with Y1 150; 4, line 258 with Y2 262; 5, line 5 with Y1 2;
# 6, line 5 with Y1 5; 7, line 100 with Y2 300, past the field (psx-spx: no vblank interrupts); 8, line 100 with Y1 50
# (behind, the line still in the picture: Diamond_Native.md §28).
# Part 1, from 80100000h, 20h a case: timer 1 counts hblanks from the reference edge; the count at the write, GPUSTAT
# read just after it, eight samples of GPUSTAT bit 31 taken as the count changes (the first sample in the high bit),
# the count at the first and second vblank edges (I_STAT bit 0) after the write, and the edges seen, within 600 lines.
# Part 2, from 80100400h, 20h a case for timer 1 sync mode 0, then from 80100600h for mode 3, on the system clock:
# timer 2 counts the system clock over 8 from the reference edge; the mode is written at L's middle, the range just
# after, and the count read at once, 1.5, 3 and 20 lines later (404, 807 and 5,381 ticks), then timer 2 at the first read.
# Part 3, from 80100800h, as part 2's mode 0 with the range written first and the mode just after.
# 800FFFF0h holds D1A0D00Eh when done.
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
        or gp, zero, zero             # the mode before the range
        or fp, zero, zero             # the case
        or s5, s0, zero
p1:     jal cases
        nop
        jal part1
        nop
        addiu s5, s5, 0x20
        addiu fp, fp, 1
        addiu t0, zero, 9
        bne fp, t0, p1
        nop
        addiu s5, s0, 0x400
        addiu k1, zero, 0x0001        # timer 1: sync mode 0
        or fp, zero, zero
p2:     jal cases
        nop
        jal part2
        nop
        addiu s5, s5, 0x20
        addiu fp, fp, 1
        addiu t0, zero, 9
        bne fp, t0, p2
        nop
        addiu s5, s0, 0x600
        addiu k1, zero, 0x0007        # timer 1: sync mode 3
        or fp, zero, zero
p3:     jal cases
        nop
        jal part2
        nop
        addiu s5, s5, 0x20
        addiu fp, fp, 1
        addiu t0, zero, 9
        bne fp, t0, p3
        nop
        addiu s5, s0, 0x800
        addiu k1, zero, 0x0001        # timer 1: sync mode 0, written after the range
        addiu gp, zero, 1
        or fp, zero, zero
p4:     jal cases
        nop
        jal part2
        nop
        addiu s5, s5, 0x20
        addiu fp, fp, 1
        addiu t0, zero, 9
        bne fp, t0, p4
        nop
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(s0)
done:   j done
        nop
# case fp: a1 the hblank count, a3 timer 2's ticks over 8 to L's middle, a2 the new GP1(07h) word, from the table
# that follows the jump, whose address the jump leaves in ra
cases:  or t9, ra, zero
        jal tget
        nop
        .word 107
        .word 28925
        .word 0x07040010
        .word 0
        .word 107
        .word 28925
        .word 0x0700C810
        .word 0
        .word 107
        .word 28925
        .word 0x07019010
        .word 0
        .word 107
        .word 28925
        .word 0x07040096
        .word 0
        .word 2
        .word 672
        .word 0x07041810
        .word 0
        .word 12
        .word 3363
        .word 0x07040002
        .word 0
        .word 12
        .word 3363
        .word 0x07040005
        .word 0
        .word 107
        .word 28925
        .word 0x0704B010
        .word 0
        .word 107
        .word 28925
        .word 0x07040032
        .word 0
tget:   sll t0, fp, 4
        addu t0, t0, ra
        lw a1, 0(t0)
        lw a3, 4(t0)
        lw a2, 8(t0)
        jr t9
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
part1:  or s7, ra, zero
        jal settle
        nop
        addiu t0, zero, 0x0100
        sh t0, 0x1114(a0)             # timer 1: hblanks, from zero
w1:     lhu t0, 0x1110(a0)
        nop
        bne t0, a1, w1
        nop
        lhu t1, 0x1120(a0)
d1:     lhu t2, 0x1120(a0)
        nop
        subu t2, t2, t1
        andi t2, t2, 0xFFFF
        sltiu t2, t2, 1000
        bne t2, zero, d1
        nop
        sw a2, 0x1814(a0)             # the new range
        lw t3, 0x1814(a0)
        lhu t4, 0x1110(a0)
        nop
        sw t4, 0(s5)
        sw t3, 4(s5)
        or s1, t4, zero
        or s2, zero, zero
        or s3, zero, zero
        or s4, zero, zero
        addiu s6, a1, 600
l1:     lhu t0, 0x1110(a0)
        nop
        beq t0, s1, noline
        nop
        or s1, t0, zero
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
        sll t3, s4, 2
        addu t3, t3, s5
        sw t0, 12(t3)
        addiu s4, s4, 1
        addiu t3, zero, 2
        beq s4, t3, out1
        nop
noedge: sltu t1, t0, s6
        bne t1, zero, l1
        nop
out1:   sw s3, 8(s5)
        sw s4, 20(s5)
        jr s7
        nop
part2:  or s7, ra, zero
        jal settle
        nop
        addiu t0, zero, 0x0200
        sh t0, 0x1124(a0)             # timer 2: the system clock over 8, from zero
w2:     lhu t0, 0x1120(a0)
        nop
        sltu t1, t0, a3
        bne t1, zero, w2
        nop
        bne gp, zero, later
        nop
        sh k1, 0x1114(a0)             # timer 1's mode, its sync on
        j read
        sw a2, 0x1814(a0)             # the new range
later:  sw a2, 0x1814(a0)
        sh k1, 0x1114(a0)
read:   lhu t2, 0x1110(a0)
        lhu t0, 0x1120(a0)
        sw t2, 0(s5)
        sw t0, 16(s5)
        addiu t1, t0, 404
        jal at
        addiu s6, s5, 4
        addiu t1, t0, 807
        jal at
        addiu s6, s5, 8
        addiu t1, t0, 5381
        jal at
        addiu s6, s5, 12
        sw zero, 0x1124(a0)           # timer 2: the system clock again
        jr s7
        nop
# timer 1's count at s6 once timer 2 reaches t1
at:     lhu t2, 0x1120(a0)
        nop
        sltu t3, t2, t1
        bne t3, zero, at
        nop
        lhu t2, 0x1110(a0)
        jr ra
        sw t2, 0(s6)
