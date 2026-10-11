# A mode write that turns a timer's sync on (Diamond_Records.md §8.1, Diamond_Disputes.md D-6). Interrupts off; for
# timers 0 and 1 on the system clock, each sync mode 0 to 3 and k = 0 to 3: I_STAT's vblank polled, k x 8,192 loops
# of four cycles, the mode written with its sync on, the count read twice, the mode written 0 again. psx-spx's
# "Timers": the write resets the count, so a read is the few cycles since the write, or zero where the mode pauses or
# waits for its blank. Results from 80100000h, two words a case at (16 timer + 4 mode + k) x 8. 800FFFF0h holds
# D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x1F80
        sw zero, 0x1074(a0)           # I_MASK: nothing reaches the CPU
        sw zero, 0x1814(a0)           # GP1(00h)
        lui t0, 0x0800
        ori t0, t0, 0x0001
        sw t0, 0x1814(a0)             # GP1(08h): NTSC, 320
        lui s0, 0x8010
        or s1, zero, zero             # the timer's offset, 0 then 10h
ntimer: or s2, zero, zero             # the sync mode
nmode:  or s3, zero, zero             # k
nk:     jal case
        nop
        addiu s0, s0, 8
        addiu s3, s3, 1
        addiu t0, zero, 4
        bne s3, t0, nk
        nop
        addiu s2, s2, 1
        bne s2, t0, nmode
        nop
        addiu s1, s1, 0x10
        addiu t0, zero, 0x20
        bne s1, t0, ntimer
        nop
        lui t0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(t0)
done:   j done
        nop
# one case: the timer at 1F801100h + s1, the mode s2, k s3, its two reads at s0
case:   addu a1, a0, s1
        addiu t0, zero, -2
        sw t0, 0x1070(a0)             # I_STAT's vblank acknowledged
vb:     lw t0, 0x1070(a0)
        nop
        andi t0, t0, 1
        beq t0, zero, vb
        nop
        sll t0, s3, 13
        beq t0, zero, write
        nop
dl:     nop
        addiu t0, t0, -1
        bne t0, zero, dl
        nop
write:  sll t1, s2, 1
        ori t1, t1, 1
        sh t1, 0x1104(a1)             # the mode, its sync on
        lhu t2, 0x1100(a1)
        lhu t3, 0x1100(a1)
        sh zero, 0x1104(a1)           # free-running again
        sw t2, 0(s0)
        jr ra
        sw t3, 4(s0)
