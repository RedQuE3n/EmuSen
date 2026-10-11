# amidog's psxtest_gte 1.4 timing loops (BlackMoon_GTE.md §6, D-13), in the shape its OFFICIAL and UNOFFICIAL TIMING
# tests run them, as disassembled from its RAM. For each of the 64 command numbers, from the table at 80010008h, the
# routine at 80010108h is copied to 80040000h + 400h x n with the number's command word in place of each FFFFFFFFh,
# and called twice from the I-cache with interrupts off: once with a0 = 0 (one pass of each loop, filling the cache),
# then with a0 = 500 (501 passes). Each of its ten loops resets timer 2 on the system clock by writing its mode and
# reads the counter after the last pass: the command then CFC2 FLAG after 0, 1, 4 and 7 NOPs; MFC2 MAC1 after 0, 1, 4
# and 7 NOPs, three NOPs after it; the command twice 4 NOPs apart and CFC2; twice 7 apart and MFC2. The ten counts of
# number n are at 80100000h + 40h x n; 800FFFF0h holds D1A0D00Eh when done. The 22 commands are written in
# psx-spx's encodings, every other number as 4A000000h plus the number.
        j start
        nop
cmds:
        .word 0x4A000000
        .word 0x4A180001
        .word 0x4A000002
        .word 0x4A000003
        .word 0x4A000004
        .word 0x4A000005
        .word 0x4B400006
        .word 0x4A000007
        .word 0x4A000008
        .word 0x4A000009
        .word 0x4A00000A
        .word 0x4A00000B
        .word 0x4B70000C
        .word 0x4A00000D
        .word 0x4A00000E
        .word 0x4A00000F
        .word 0x4A780010
        .word 0x4A980011
        .word 0x4A400012
        .word 0x4AE80413
        .word 0x4B280414
        .word 0x4A000015
        .word 0x4AF80416
        .word 0x4A000017
        .word 0x4A000018
        .word 0x4A000019
        .word 0x4A00001A
        .word 0x4B08041B
        .word 0x4B38041C
        .word 0x4A00001D
        .word 0x4AC8041E
        .word 0x4A00001F
        .word 0x4AD80420
        .word 0x4A000021
        .word 0x4A000022
        .word 0x4A000023
        .word 0x4A000024
        .word 0x4A000025
        .word 0x4A000026
        .word 0x4A000027
        .word 0x4AA00428
        .word 0x4A680029
        .word 0x4AF8002A
        .word 0x4A00002B
        .word 0x4A00002C
        .word 0x4B58002D
        .word 0x4B68002E
        .word 0x4A00002F
        .word 0x4A280030
        .word 0x4A000031
        .word 0x4A000032
        .word 0x4A000033
        .word 0x4A000034
        .word 0x4A000035
        .word 0x4A000036
        .word 0x4A000037
        .word 0x4A000038
        .word 0x4A000039
        .word 0x4A00003A
        .word 0x4A00003B
        .word 0x4A00003C
        .word 0x4B90003D
        .word 0x4BA0003E
        .word 0x4B18043F
tmpl:   addu t7, zero, a0
        lui t5, 0x1F80
        addu a0, zero, t7
        addiu t1, zero, 0x0
        sh t1, 0x1124(t5)
l0:     .word 0xFFFFFFFF
        cfc2 t1, FLAG
        nop
        addu t0, t0, t1
        bgtz a0, l0
        addiu a0, a0, -0x1
        lhu t0, 0x1120(t5)
        nop
        addu v0, zero, t0
        sw v0, 0x0(a1)
        addu a0, zero, t7
        addiu t1, zero, 0x0
        sh t1, 0x1124(t5)
l1:     .word 0xFFFFFFFF
        nop
        cfc2 t1, FLAG
        nop
        addu t0, t0, t1
        bgtz a0, l1
        addiu a0, a0, -0x1
        lhu t0, 0x1120(t5)
        nop
        addu v0, zero, t0
        sw v0, 0x4(a1)
        addu a0, zero, t7
        addiu t1, zero, 0x0
        sh t1, 0x1124(t5)
l2:     .word 0xFFFFFFFF
        nop
        nop
        nop
        nop
        cfc2 t1, FLAG
        nop
        addu t0, t0, t1
        bgtz a0, l2
        addiu a0, a0, -0x1
        lhu t0, 0x1120(t5)
        nop
        addu v0, zero, t0
        sw v0, 0x8(a1)
        addu a0, zero, t7
        addiu t1, zero, 0x0
        sh t1, 0x1124(t5)
l3:     .word 0xFFFFFFFF
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        cfc2 t1, FLAG
        nop
        addu t0, t0, t1
        bgtz a0, l3
        addiu a0, a0, -0x1
        lhu t0, 0x1120(t5)
        nop
        addu v0, zero, t0
        sw v0, 0xc(a1)
        addu a0, zero, t7
        addiu t1, zero, 0x0
        sh t1, 0x1124(t5)
l4:     .word 0xFFFFFFFF
        mfc2 t1, MAC1
        nop
        nop
        nop
        addu t0, t0, t1
        bgtz a0, l4
        addiu a0, a0, -0x1
        lhu t0, 0x1120(t5)
        nop
        addu v0, zero, t0
        sw v0, 0x10(a1)
        addu a0, zero, t7
        addiu t1, zero, 0x0
        sh t1, 0x1124(t5)
l5:     .word 0xFFFFFFFF
        nop
        mfc2 t1, MAC1
        nop
        nop
        nop
        addu t0, t0, t1
        bgtz a0, l5
        addiu a0, a0, -0x1
        lhu t0, 0x1120(t5)
        nop
        addu v0, zero, t0
        sw v0, 0x14(a1)
        addu a0, zero, t7
        addiu t1, zero, 0x0
        sh t1, 0x1124(t5)
l6:     .word 0xFFFFFFFF
        nop
        nop
        nop
        nop
        mfc2 t1, MAC1
        nop
        nop
        nop
        addu t0, t0, t1
        bgtz a0, l6
        addiu a0, a0, -0x1
        lhu t0, 0x1120(t5)
        nop
        addu v0, zero, t0
        sw v0, 0x18(a1)
        addu a0, zero, t7
        addiu t1, zero, 0x0
        sh t1, 0x1124(t5)
l7:     .word 0xFFFFFFFF
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mfc2 t1, MAC1
        nop
        nop
        nop
        addu t0, t0, t1
        bgtz a0, l7
        addiu a0, a0, -0x1
        lhu t0, 0x1120(t5)
        nop
        addu v0, zero, t0
        sw v0, 0x1c(a1)
        addu a0, zero, t7
        addiu t1, zero, 0x0
        sh t1, 0x1124(t5)
l8:     .word 0xFFFFFFFF
        nop
        nop
        nop
        nop
        .word 0xFFFFFFFF
        nop
        nop
        nop
        nop
        cfc2 t1, FLAG
        nop
        addu t0, t0, t1
        bgtz a0, l8
        addiu a0, a0, -0x1
        lhu t0, 0x1120(t5)
        nop
        addu v0, zero, t0
        sw v0, 0x20(a1)
        addu a0, zero, t7
        addiu t1, zero, 0x0
        sh t1, 0x1124(t5)
l9:     .word 0xFFFFFFFF
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        .word 0xFFFFFFFF
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        mfc2 t1, MAC1
        nop
        nop
        nop
        addu t0, t0, t1
        bgtz a0, l9
        addiu a0, a0, -0x1
        lhu t0, 0x1120(t5)
        nop
        addu v0, zero, t0
        sw v0, 0x24(a1)
        jr ra
        nop
        .word 0xFFFFFFFE
start:  lui t0, 0x4000
        mtc0 t0, SR
        lui t5, 0x1F80
        sh zero, 0x1104(t5)
        sh zero, 0x1114(t5)
        sh zero, 0x1124(t5)
        sw zero, 0x1074(t5)
        sw zero, 0x1070(t5)
        lui s3, 0x8001
        ori s3, s3, 0x8
        lui s1, 0x8004
        lui s2, 0x8010
        addiu s7, zero, 64
next:   lw s4, 0x0(s3)
        lui t2, 0x8001
        ori t2, t2, 0x108
        or t3, s1, zero
copy:   lw t4, 0x0(t2)
        addiu t6, zero, -1
        bne t4, t6, keep
        addiu t2, t2, 4
        or t4, s4, zero
keep:   sw t4, 0x0(t3)
        addiu t6, zero, -2
        bne t4, t6, copy
        addiu t3, t3, 4
        addu a0, zero, zero
        jalr s1
        or a1, s2, zero
        addiu a0, zero, 500
        jalr s1
        or a1, s2, zero
        addiu s3, s3, 4
        addiu s1, s1, 0x400
        addiu s7, s7, -1
        bne s7, zero, next
        addiu s2, s2, 0x40
        lui t0, 0xD1A0
        ori t0, t0, 0xD00E
        lui t1, 0x8010
        sw t0, -0x10(t1)
done:   j done
        nop
