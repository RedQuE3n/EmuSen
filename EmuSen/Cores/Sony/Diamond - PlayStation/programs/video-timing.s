# D-0: the line length (Diamond_Disputes.md D-0). With interrupts off, I_STAT's vblank bit is polled and each edge
# stamped with timer 2 on the system clock, unwrapped by the poll, and with timer 1 counting hblanks; 4 settling edges,
# then 41 stamped, in each of six GP1(08h) modes: NTSC and PAL at 320 pixels, both interlaced at 240 lines (bit 5),
# and both at 480 (bits 5 and 2). Results from 80100000h, 200h a mode: 41 cycle stamps, then from +100h the 41
# timer 1 counts. 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x1F80
        sw zero, 0x1074(a0)           # I_MASK: nothing reaches the CPU
        sw zero, 0x1814(a0)           # GP1(00h)
        sw zero, 0x1124(a0)           # timer 2: the system clock, free-running
        addiu t0, zero, 0x0100
        sw t0, 0x1114(a0)             # timer 1: hblanks
        lui s0, 0x8010
        lui s4, 0x0800
        ori a1, s4, 0x0001            # GP1(08h): NTSC, 320
        jal measure
        or s5, s0, zero
        ori a1, s4, 0x0009            # PAL, 320
        jal measure
        addiu s5, s0, 0x200
        ori a1, s4, 0x0021            # NTSC, 320, interlaced
        jal measure
        addiu s5, s0, 0x400
        ori a1, s4, 0x0029            # PAL, 320, interlaced
        jal measure
        addiu s5, s0, 0x600
        ori a1, s4, 0x0025            # NTSC, 320, 480 lines
        jal measure
        addiu s5, s0, 0x800
        ori a1, s4, 0x002D            # PAL, 320, 480 lines
        jal measure
        addiu s5, s0, 0xA00
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(s0)
done:   j done
        nop
# the mode a1, results from s5: s3 the stamp's high part, s2 timer 2's last value, s6 the edge from -4
measure: sw a1, 0x1814(a0)
        lhu s2, 0x1120(a0)
        or s3, zero, zero
        addiu s6, zero, -4
poll:   lw t0, 0x1070(a0)
        lhu t1, 0x1120(a0)
        lhu t5, 0x1110(a0)
        sltu t2, t1, s2
        beq t2, zero, nowrap
        or s2, t1, zero
        lui t3, 1
        addu s3, s3, t3
nowrap: andi t0, t0, 1
        beq t0, zero, poll
        nop
        addiu t4, zero, -2
        sw t4, 0x1070(a0)             # the vblank acknowledged
        bltz s6, skip
        addu t6, s3, t1
        sll t7, s6, 2
        addu t7, t7, s5
        sw t6, 0(t7)
        sw t5, 0x100(t7)
skip:   addiu t8, zero, 40
        bne s6, t8, poll
        addiu s6, s6, 1
        jr ra
        nop
