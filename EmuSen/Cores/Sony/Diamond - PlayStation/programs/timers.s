# ps1-tests' timers (MIT), rewritten to record in RAM: the same measurements in the same order, each function's
# timed instructions as the release build 158 has them, printf replaced by stores (Diamond_Records.md §2). Bare
# metal: the release's GP1 and timer 1 set-up written directly, interrupts off but for the frame rows, where a
# vblank handler of its own toggles currVsync as the test's callback does. Results from 80100000h, ten words a row
# (results[1..10]): 000h the 18 delay and frame rows; 300h timer 0's sync rows (clock, resolution, sync); A00h timer
# 1's and 2's (timer, clock, sync); D00h the target check's ticks 0-11, D30h its mode read, D34h-D40h its four
# booleans; D80h the six mode reads of the flag check. 800FFFF0h holds D1A0D00Eh when done.
# The timed sequences are from ps1-tests, Copyright (c) 2020 Jakub Czekański, under the MIT License.
        j start
        nop
vector: j handler                     # copied to 80000080h
        nop
        .word 0x06C4E24E              # 80010010h: GP1(06h) and GP1(08h) as PutDispEnv writes them, 256 to 640
        .word 0x08000000
        .word 0x06C58258
        .word 0x08000001
        .word 0x06C9B21B
        .word 0x08000064
        .word 0x06C67267
        .word 0x08000002
        .word 0x06C6C26C
        .word 0x08000003
# io.c's read16 and write16, and main.c's delay, as compiled
read16: lhu v0, 0(a0)
        jr ra
        andi v0, v0, 0xFFFF
write16: sh a1, 0(a0)
        jr ra
        nop
delay:  srl a0, a0, 2
dl:     nop
        addiu a0, a0, -1
        bne a0, zero, dl
        nop
        jr ra
        nop
# testTimerWithCyclesDelay(a0 timer, a1 mode, a2 cycles), its row at a3
cycles: addiu sp, sp, -0x70
        sw s1, 0x4C(sp)
        lui s1, 0x01F8
        addiu s1, s1, 0x0110
        addu s1, a0, s1
        sll s1, s1, 4                 # the counter, 1F801100h + 10h a timer
        sw s4, 0x58(sp)
        addiu s4, s1, 4               # its mode
        sw s6, 0x60(sp)
        sw s0, 0x48(sp)
        or s6, a0, zero
        or s0, a1, zero
        or a0, s4, zero
        sw ra, 0x6C(sp)
        sw fp, 0x68(sp)
        sw s5, 0x5C(sp)
        sw s3, 0x54(sp)
        sw s2, 0x50(sp)
        or s3, a2, zero
        sw a3, 0x44(sp)
        jal read16
        sw s7, 0x64(sp)
        sll a1, s0, 8
        andi a1, a1, 0xFF00
        or a0, s4, zero
        jal write16                   # initTimer
        sw v0, 0x40(sp)
        jal read16
        or a0, s1, zero
        addiu fp, sp, 0x10            # results[11]
        addiu s2, sp, 0x3C
        ori s5, zero, 0xFFFF
c1:     jal read16
        or a0, s1, zero               # start
        or a0, s3, zero
        jal delay
        or s7, v0, zero
        jal read16
        or a0, s1, zero               # end
        or a0, s4, zero
        jal read16
        or s0, v0, zero               # the flags
        andi v0, v0, 0x1000
        beq v0, zero, c2
        nop
        addu s0, s0, s5               # end += 0xffff
c2:     subu s0, s0, s7
        sw s0, 0(fp)
        addiu fp, fp, 4
        bne fp, s2, c1
        nop
        lw a1, 0x40(sp)
        jal write16                   # restoreTimer
        or a0, s4, zero
        lw a0, 0x44(sp)
        jal keep
        addiu a1, sp, 0x14
        j leave
        nop
# testTimerWithFrameDelay(a0 timer, a1 mode), its row at a3
frame:  addiu sp, sp, -0x70
        sw s2, 0x50(sp)
        lui s2, 0x01F8
        addiu s2, s2, 0x0110
        addu s2, a0, s2
        sw s1, 0x4C(sp)
        sll s2, s2, 4
        or s1, a1, zero
        sw s3, 0x54(sp)
        sw s0, 0x48(sp)
        sll s3, s1, 8
        addiu s0, s2, 4
        sw s5, 0x5C(sp)
        andi s3, s3, 0xFFFF
        or s5, a0, zero
        or a0, s0, zero
        sw ra, 0x6C(sp)
        sw fp, 0x68(sp)
        sw s7, 0x64(sp)
        sw s4, 0x58(sp)
        sw a3, 0x44(sp)
        jal read16
        sw s6, 0x60(sp)
        or a1, s3, zero
        or a0, s0, zero
        sw v0, 0x40(sp)
        jal write16                   # initTimer
        nop
        jal irqon                     # in place of InterruptCallback(0, vblank_irq)
        nop
        addiu s7, sp, 0x10
        addiu s1, sp, 0x3C
        ori s4, zero, 0xFFFF
        or a1, s3, zero
f1:     jal write16                   # the counter reset by its mode
        or a0, s0, zero
f2:     lw v1, -0x7FF0(gp)            # waitVsync: currVsync against prevVsync
        lw v0, -0x7FF4(gp)
        nop
        beq v1, v0, f2
        nop
        lw v0, -0x7FF0(gp)
        or a0, s2, zero
        sw v0, -0x7FF4(gp)
        jal read16
        nop
        or a0, s0, zero
        jal read16
        or s6, v0, zero
        andi v0, v0, 0x1000
        beq v0, zero, f3
        nop
        addu s6, s6, s4
f3:     sw s6, 0(s7)
        addiu s7, s7, 4
        bne s7, s1, f1
        or a1, s3, zero
        lw a1, 0x40(sp)
        jal write16                   # restoreTimer
        or a0, s0, zero
        jal irqoff
        nop
        lw a0, 0x44(sp)
        jal keep
        addiu a1, sp, 0x14
        j leave
        nop
# testTimerWithCyclesDelaySyncEnabled(a0 timer, a1 mode, a2 sync, a3 cycles), its row at t9
sync:   addiu sp, sp, -0x70
        sw s2, 0x50(sp)
        lui s2, 0x01F8
        addiu s2, s2, 0x0110
        addu s2, a0, s2
        sll s2, s2, 4
        sw s3, 0x54(sp)
        addiu s3, s2, 4
        sw s7, 0x64(sp)
        sw s5, 0x5C(sp)
        or s7, a2, zero
        or s5, a0, zero
        or a0, s3, zero
        sw ra, 0x6C(sp)
        sw fp, 0x68(sp)
        sw s6, 0x60(sp)
        sw s4, 0x58(sp)
        sw s1, 0x4C(sp)
        sw s0, 0x48(sp)
        or s4, a3, zero
        sw t9, 0x44(sp)
        jal read16
        or s0, a1, zero
        sll v1, s7, 1
        andi v1, v1, 6
        sll a1, s0, 8
        or a1, v1, a1
        ori a1, a1, 1
        andi a1, a1, 0xFF07
        or a0, s3, zero
        sw v0, 0x40(sp)
        jal write16                   # initTimerWithSync
        addiu fp, sp, 0x10
        addiu s1, sp, 0x3C
        ori s6, zero, 0xFFFF
y1:     jal delay
        or a0, s4, zero
        jal read16
        or a0, s2, zero               # end
        or a0, s3, zero
        jal read16
        or s0, v0, zero
        andi v0, v0, 0x1000
        beq v0, zero, y2
        nop
        addu s0, s0, s6
y2:     sw s0, 0(fp)
        addiu fp, fp, 4
        bne fp, s1, y1
        or a0, s3, zero
        lw a1, 0x40(sp)
        jal write16                   # restoreTimer
        nop
        lw a0, 0x44(sp)
        jal keep
        addiu a1, sp, 0x14
        j leave
        nop
# the shared epilogue of the three above
leave:  lw ra, 0x6C(sp)
        lw fp, 0x68(sp)
        lw s7, 0x64(sp)
        lw s6, 0x60(sp)
        lw s5, 0x5C(sp)
        lw s4, 0x58(sp)
        lw s3, 0x54(sp)
        lw s2, 0x50(sp)
        lw s1, 0x4C(sp)
        lw s0, 0x48(sp)
        jr ra
        addiu sp, sp, 0x70
# readTimerValues: ticks[*value]++ ten thousand times, ticks at t7
ticks:  addiu v1, zero, 10000
        lui a1, 0x1F80
tk:     lhu v0, 0x1120(a1)
        addiu v1, v1, -1
        andi v0, v0, 0xFFFF
        sll v0, v0, 1
        addu v0, t7, v0
        lhu a0, 0(v0)
        andi v1, v1, 0xFFFF
        addiu a0, a0, 1
        bne v1, zero, tk
        sh a0, 0(v0)
        jr ra
        nop
# testTimerTargetBehaviour: timer 2 to a target of 10 with the reset at the target; its record at 80100D00h
target: addiu sp, sp, -0x298
        sw ra, 0x294(sp)
        sw s7, 0x290(sp)
        sw s6, 0x28C(sp)
        sw s5, 0x288(sp)
        sw s1, 0x278(sp)
        sw s0, 0x274(sp)
        lui a0, 0x1F80
        jal read16
        addiu a0, a0, 0x1124
        addiu s1, sp, 0x10            # ticks[300]
        or s5, v0, zero
        addiu a0, sp, 0x268
        or v1, s1, zero
tz:     sw zero, 0(v1)
        addiu v1, v1, 4
        bne v1, a0, tz
        nop
        lui s0, 0x1F80
        addiu a0, s0, 0x1128
        jal write16                   # the target
        addiu a1, zero, 10
        addiu a0, s0, 0x1124
        jal write16                   # the reset at the target, the system clock
        addiu a1, zero, 8
        or a1, zero, zero
        jal write16                   # the counter
        addiu a0, s0, 0x1120
        jal read16                    # the reached bits cleared
        ori a0, s0, 0x1124
        jal ticks
        addiu t7, sp, 0x10
        jal read16
        ori a0, s0, 0x1124
        or s6, v0, zero               # the status
        lui s7, 0x8010
        ori s7, s7, 0x0D00
        addiu t0, zero, 12
        or t1, s1, zero
        or t2, s7, zero
tc:     lhu t3, 0(t1)
        addiu t1, t1, 2
        sw t3, 0(t2)
        addiu t0, t0, -1
        bne t0, zero, tc
        addiu t2, t2, 4
        sw s6, 0x30(s7)
        andi v0, s6, 0x0800
        sltu v0, zero, v0
        sw v0, 0x34(s7)               # reached the target
        andi v0, s6, 0x1000
        sltu v0, zero, v0
        sw v0, 0x38(s7)               # reached FFFFh
        lhu v0, 0x24(sp)
        nop
        sltu v0, zero, v0
        sw v0, 0x3C(s7)               # reset after the target
        lhu v0, 0x26(sp)
        nop
        sltiu v0, v0, 1
        sw v0, 0x40(s7)               # target + 1 not reached
        lui a0, 0x1F80
        or a1, s5, zero
        jal write16                   # restoreTimer
        addiu a0, a0, 0x1124
        lw ra, 0x294(sp)
        lw s7, 0x290(sp)
        lw s6, 0x28C(sp)
        lw s5, 0x288(sp)
        lw s1, 0x278(sp)
        lw s0, 0x274(sp)
        jr ra
        addiu sp, sp, 0x298
# testTimerResetFlagWritable: timer 0's mode read after writes and after reaching FFFFh; six words at 80100D80h
flags:  addiu sp, sp, -0x38
        sw s0, 0x1C(sp)
        lui s0, 0x1F80
        sw ra, 0x34(sp)
        sw s7, 0x30(sp)
        sw s4, 0x2C(sp)
        sw s1, 0x20(sp)
        lui s7, 0x8010
        ori s7, s7, 0x0D80
        jal read16
        addiu a0, s0, 0x1104
        or s1, v0, zero
        addiu a0, s0, 0x1108
        jal write16
        ori a1, zero, 0xFFFF
        or a1, zero, zero
        jal write16
        ori a0, s0, 0x1104
        jal read16
        ori a0, s0, 0x1104
        sw v0, 0(s7)                  # after writing 0000h
        ori a1, zero, 0xFFFF
        jal write16
        ori a0, s0, 0x1104
        jal read16
        ori a0, s0, 0x1104
        sw v0, 4(s7)                  # after writing FFFFh
        ori a0, s0, 0x1104
        jal write16
        addiu a1, zero, 8
        or a1, zero, zero
        jal write16
        addiu a0, s0, 0x1100
        jal read16
        ori a0, s0, 0x1104
        lui s4, 0x0001
        jal delay
        ori a0, s4, 0x86A0            # 100,000
        jal read16
        ori a0, s0, 0x1104
        ori a0, s0, 0x1104
        jal read16
        sw v0, 8(s7)                  # after the wait
        sw v0, 12(s7)                 # and again
        jal delay
        ori a0, s4, 0x86A0
        addiu a1, zero, 8
        jal write16
        ori a0, s0, 0x1104
        jal read16
        ori a0, s0, 0x1104
        ori a0, s0, 0x1104
        jal read16
        sw v0, 16(s7)                 # after the wait and a write
        sw v0, 20(s7)                 # and again
        or a1, s1, zero
        jal write16
        ori a0, s0, 0x1104
        lw ra, 0x34(sp)
        lw s7, 0x30(sp)
        lw s4, 0x2C(sp)
        lw s1, 0x20(sp)
        lw s0, 0x1C(sp)
        jr ra
        addiu sp, sp, 0x38
# ten words from a1 to a0
keep:   addiu t0, zero, 10
k1:     lw t1, 0(a1)
        addiu a1, a1, 4
        sw t1, 0(a0)
        addiu t0, t0, -1
        bne t0, zero, k1
        addiu a0, a0, 4
        jr ra
        nop
# the vblank interrupt alone, acknowledged first, and off again
irqon:  lui t0, 0x1F80
        addiu t1, zero, -2
        sw t1, 0x1070(t0)
        addiu t1, zero, 1
        sw t1, 0x1074(t0)
        addiu t1, zero, 0x0401
        mtc0 t1, SR
        jr ra
        nop
irqoff: mtc0 zero, SR
        lui t0, 0x1F80
        jr ra
        sw zero, 0x1074(t0)
handler: lui k0, 0x1F80
        addiu k1, zero, -2
        sw k1, 0x1070(k0)             # the vblank acknowledged
        lw k1, -0x7FF0(gp)
        nop
        sltiu k1, k1, 1
        sw k1, -0x7FF0(gp)            # vblank_irq: currVsync = !currVsync
        mfc0 k0, EPC
        nop
        jr k0
        rfe
# setDisplayResolution: GP1(06h) a0, GP1(07h), GP1(08h) a1, GP1(05h)
setres: lui t0, 0x1F80
        sw a0, 0x1814(t0)
        lui t1, 0x0704
        ori t1, t1, 0x0010
        sw t1, 0x1814(t0)
        sw a1, 0x1814(t0)
        lui t1, 0x0500
        jr ra
        sw t1, 0x1814(t0)
# the resolution's pair from the table at 80010010h, a0 its index
pair:   lui t0, 0x8001
        sll t1, a0, 3
        addu t0, t0, t1
        lw a0, 0x10(t0)
        lw a1, 0x14(t0)
        j setres
        nop
start:  mtc0 zero, SR
        lui sp, 0x801F
        ori sp, sp, 0xFF00
        lui gp, 0x8010
        addiu gp, gp, 0x7FD0          # currVsync at 800FFFE0h, prevVsync at 800FFFDCh
        sw zero, -0x7FF0(gp)
        sw zero, -0x7FF4(gp)
        lui t0, 0x8001
        lw t1, 8(t0)
        lw t2, 12(t0)
        lui t3, 0x8000
        sw t1, 0x80(t3)
        sw t2, 0x84(t3)
        lui a0, 0x1F80
        sw zero, 0x1074(a0)           # I_MASK
        addiu t0, zero, 0x1D00
        sh t0, 0x1114(a0)             # timer 1 on hblanks, as ResetGraph leaves it
        sw zero, 0x1814(a0)           # GP1(00h)
        jal pair
        addiu a0, zero, 1             # 320x240
        lui a0, 0x1F80
        lui t0, 0x0400
        ori t0, t0, 2
        sw t0, 0x1814(a0)             # GP1(04h)
        lui t0, 0x0300
        sw t0, 0x1814(a0)             # GP1(03h): the display on
        lui t0, 0x0800
        ori t0, t0, 1
        sw t0, 0x1814(a0)             # SetVideoMode(MODE_NTSC)
# the delay and frame rows: timers 0 to 2, clocks 0 and 1 (timer 2's 0 and 2)
        lui s7, 0x8010
        or s1, zero, zero
m1:     or s2, zero, zero
m2:     or s3, s2, zero
        addiu t0, zero, 2
        bne s1, t0, m3
        nop
        sll s3, s2, 1
m3:     or a0, s1, zero
        or a1, s3, zero
        addiu a2, zero, 1000
        jal cycles
        or a3, s7, zero
        or a0, s1, zero
        or a1, s3, zero
        addiu a2, zero, 5000
        jal cycles
        addiu a3, s7, 40
        or a0, s1, zero
        or a1, s3, zero
        jal frame
        addiu a3, s7, 80
        addiu s7, s7, 120
        addiu s2, s2, 1
        addiu t0, zero, 2
        bne s2, t0, m2
        nop
        addiu s1, s1, 1
        addiu t0, zero, 3
        bne s1, t0, m1
        nop
# timer 0's sync rows: clocks 0 and 1, five resolutions, syncs 0 to 3
        lui s7, 0x8010
        ori s7, s7, 0x0300
        or s2, zero, zero
n1:     or s3, zero, zero
n2:     jal pair
        or a0, s3, zero
        or s4, zero, zero
n3:     or a0, zero, zero
        or a1, s2, zero
        or a2, s4, zero
        addiu a3, zero, 100
        jal sync
        or t9, s7, zero
        addiu s7, s7, 40
        addiu s4, s4, 1
        addiu t0, zero, 4
        bne s4, t0, n3
        nop
        addiu s3, s3, 1
        addiu t0, zero, 5
        bne s3, t0, n2
        nop
        addiu s2, s2, 1
        addiu t0, zero, 2
        bne s2, t0, n1
        nop
        jal pair
        addiu a0, zero, 1             # 320x240 again
# timers 1 and 2's sync rows: clocks 0 and 1, syncs 0 to 3
        lui s7, 0x8010
        ori s7, s7, 0x0A00
        addiu s1, zero, 1
p1:     or s2, zero, zero
p2:     or s4, zero, zero
p3:     or a0, s1, zero
        or a1, s2, zero
        or a2, s4, zero
        addiu a3, zero, 100
        jal sync
        or t9, s7, zero
        addiu s7, s7, 40
        addiu s4, s4, 1
        addiu t0, zero, 4
        bne s4, t0, p3
        nop
        addiu s2, s2, 1
        addiu t0, zero, 2
        bne s2, t0, p2
        nop
        addiu s1, s1, 1
        addiu t0, zero, 3
        bne s1, t0, p1
        nop
        jal target
        nop
        jal flags
        nop
        lui t0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(t0)
done:   j done
        nop
