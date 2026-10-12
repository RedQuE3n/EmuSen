# The SPU's voices and reverb as sound RAM records them (Diamond_SPU.md §10.2), so that every referee is graded: the
# capture buffers hold voice 1's and voice 3's output after the envelope, before the volumes, one sample a sample in a
# ring of 512, read back by DMA4. Sound RAM from 1000h as programs/spu-voices.s lays it, and at 1140h four blocks of
# filters 5, 6, 7 and 0 (shifts 0 to 3) looping. A sync is the capture buffer's IRQ at its first halfword, IRQA 100h.
# Results from 80100000h:
#   T1 80100000h: voice 1 on the wave at E00h (a period of 512 samples) and voice 3 at 1C00h (256), 20 frames on, the
#      two buffers; 80100800h: their ENVX, LSAX and ENDX (a word each)
#   T2 80100900h: voice 1 on the DC block keyed on at a sync, the next 512 samples: attack shift 3 +7, decay 3 to
#      level 4, sustain exponential down shift 8
#   T3 80100D00h: voice 1 held at its sustain, keyed off at a sync: release exponential shift 6
#   T5 80101100h: voice 1 on the wave at 1000h, pitch modulated by voice 0 on the wave at 100h, keyed on at a sync
#   T6 80101500h: voice 1 on noise of shift 15 step 7, keyed on at a sync; 80101900h: the 512 samples after the noise
#      is set to shift 10 step 4 at the next sync
#   T7 80101D00h: voice 3 keyed on at 1020h with LSAX 1234h: LSAX and ENDX a frame later, then ENVX, ENDX and LSAX after
#      its end and mute; then keyed on at the wave, LSAX a frame later
#   T8 80101E00h: voice 1 on the filter 5-7 blocks at E00h (a period of 128), 20 frames on
#   T9 80102200h: psx-spx's "Room" work area, 26C0h bytes, 3 frames after a pulse of 56 samples went in, its writes
#      stopped by ATTR's master enable a frame before the read
#   T10 801048C0h: the same area for a set of registers that writes each stage apart (`simple`): the same side's
#      reflections at 80h and 100h, the other side's at 180h and 200h, the all-pass filters' at 280h to 400h, one comb
#      reading 20h behind each side's first reflection, vIIR 4000h, vLIN 7FFFh and vRIN 4000h, the pulse with T2's attack,
#      a frame on, before any stage's writes reach another's
#   T11 80106F80h: voice 1 on the DC block keyed on at a sync with ADSR1 000Fh, the next 512 samples
# Interrupts off; 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
heads:  .word 0x23120100
        .word 0x17064534
        .word 0x0B4A3928
        .word 0x4F3E2D1C
pattern: .word 0x24676420
        .word 0xECA9ACE0
room:   .word 0x005B007D
        .word 0x54B86D80
        .word 0x0000BED0
        .word 0xBA800000
        .word 0x53005800
        .word 0x033304D6
        .word 0x022703F0
        .word 0x01EF0374
        .word 0x01B50334
        .word 0x00000000
        .word 0x00000000
        .word 0x00000000
        .word 0x00000000
        .word 0x013601B4
        .word 0x005C00B8
        .word 0x80008000
heads2: .word 0x03726150
simple: .word 0x00100010
        .word 0x40004000
        .word 0x00000000
        .word 0x00000000
        .word 0x10002000
        .word 0x01000080
        .word 0x00E00060
        .word 0x00010001
        .word 0x00200020
        .word 0x02000180
        .word 0x00010001
        .word 0x00010001
        .word 0x00200020
        .word 0x03000280
        .word 0x04000380
        .word 0x40007FFF
start:  mtc0 zero, SR
        lui s0, 0x1F80
        lui s1, 0x8010
        lui t0, 0x2209
        ori t0, t0, 0x31E1
        sw t0, 0x1014(s0)                 # DEV4: 220931E1h, psx-spx's value for reads
        lw t0, 0x10F0(s0)
        lui t1, 0x0008
        or t0, t0, t1
        sw t0, 0x10F0(s0)                 # DPCR: DMA4 on
        ori t0, zero, 4
        sh t0, 0x1DAC(s0)                 # RAM_CTRL: one 512 KiB bank, which the capture IRQ needs
        ori s7, zero, 0xC000
        jal mode
        or a0, zero, zero
        sh zero, 0x1D80(s0)
        sh zero, 0x1D82(s0)
        sh zero, 0x1D84(s0)
        sh zero, 0x1D86(s0)
        sh zero, 0x1D90(s0)
        sh zero, 0x1D92(s0)
        sh zero, 0x1D94(s0)
        sh zero, 0x1D96(s0)
        sh zero, 0x1D98(s0)
        sh zero, 0x1D9A(s0)
        sh zero, 0x1DB0(s0)
        sh zero, 0x1DB2(s0)
        addiu t0, zero, -1
        sh t0, 0x1D8C(s0)
        ori t0, zero, 0xFF
        sh t0, 0x1D8E(s0)
        or t1, zero, zero
vinit:  addu t2, s0, t1
        sh zero, 0x1C00(t2)
        sh zero, 0x1C02(t2)
        ori t0, zero, 0x1000
        sh t0, 0x1C04(t2)
        ori t0, zero, 0x0202
        sh t0, 0x1C06(t2)
        ori t0, zero, 0x000F
        sh t0, 0x1C08(t2)
        ori t0, zero, 0x1FC0
        sh t0, 0x1C0A(t2)
        addiu t1, t1, 16
        slti t0, t1, 0x180
        bne t0, zero, vinit
        nop
        addiu t0, zero, -1
        sh t0, 0x1D88(s0)                 # every voice keyed on at the silent loop, so that each starts there
        ori t0, zero, 0xFF
        sh t0, 0x1D8A(s0)
        jal frames
        ori a0, zero, 1
        ori t0, zero, 0x0100
        sh t0, 0x1DA4(s0)                 # IRQA: the first halfword of voice 1's capture buffer
# sound RAM's image at 80120000h
        lui a2, 0x8012
        or t1, zero, zero
zero1:  addu t2, a2, t1
        sw zero, 0(t2)
        addiu t1, t1, 4
        slti t0, t1, 0x180
        bne t0, zero, zero1
        nop
        ori t0, zero, 0x0700
        sh t0, 0(a2)
        sh t0, 0x10(a2)
        sh zero, 0x20(a2)
        ori t0, zero, 0x0100
        sh t0, 0x30(a2)
        lui t3, 0x7777
        ori t3, t3, 0x7777
        sw t3, 4(a2)
        sw t3, 8(a2)
        sw t3, 12(a2)
        sh t3, 2(a2)
        sw t3, 0x24(a2)
        sw t3, 0x28(a2)
        sw t3, 0x2C(a2)
        sh t3, 0x22(a2)
        sw t3, 0x34(a2)
        sw t3, 0x38(a2)
        sw t3, 0x3C(a2)
        sh t3, 0x32(a2)
        lui t9, 0x8001
        ori t9, t9, 8                     # heads
        ori a3, a2, 0x40
        jal wave
        ori s4, zero, 16
        lui t9, 0x8001
        ori t9, t9, 0x60                  # heads2
        ori a3, a2, 0x140
        jal wave
        ori s4, zero, 4
        or a0, a2, zero
        ori a2, zero, 0x1000
        jal upload
        ori a3, zero, 6
# the reverb's work area, zeroed
        lui a0, 0x8013
        or t1, zero, zero
zero2:  addu t2, a0, t1
        sw zero, 0(t2)
        addiu t1, t1, 4
        slti t0, t1, 0x26C0
        bne t0, zero, zero2
        nop
        lui a2, 0x0007
        ori a2, a2, 0xD940
        jal upload
        ori a3, zero, 155
        jal frames
        ori a0, zero, 2

# T1: the wave on voices 1 and 3, the buffers once steady
        ori t0, zero, 0x0208
        sh t0, 0x1C16(s0)
        sh t0, 0x1C36(s0)
        ori t0, zero, 0x0E00
        sh t0, 0x1C14(s0)
        ori t0, zero, 0x1C00
        sh t0, 0x1C34(s0)
        ori t0, zero, 0x000A
        sh t0, 0x1D88(s0)
        jal frames
        ori a0, zero, 20
        lui a0, 0x8010
        ori a2, zero, 0x800
        jal download
        ori a3, zero, 32
        lhu t0, 0x1C1C(s0)
        nop
        sw t0, 0x800(s1)
        lhu t0, 0x1C3C(s0)
        nop
        sw t0, 0x804(s1)
        lhu t0, 0x1C1E(s0)
        nop
        sw t0, 0x808(s1)
        lhu t0, 0x1C3E(s0)
        nop
        sw t0, 0x80C(s1)
        lhu t0, 0x1D9C(s0)
        nop
        sw t0, 0x810(s1)
        jal silence
        nop

# T2: the DC block keyed on at a sync
        ori t0, zero, 0x0200
        sh t0, 0x1C16(s0)
        ori t0, zero, 0x1000
        sh t0, 0x1C14(s0)
        ori t0, zero, 0x0C34
        sh t0, 0x1C18(s0)
        ori t0, zero, 0xC800
        sh t0, 0x1C1A(s0)
        jal sync
        nop
        ori t0, zero, 2
        sh t0, 0x1D88(s0)
        jal sync
        nop
        lui a0, 0x8010
        ori a0, a0, 0x900
        ori a2, zero, 0x800
        jal download
        ori a3, zero, 16
        jal silence
        nop

# T3: the release at a sync
        ori t0, zero, 0x0200
        sh t0, 0x1C16(s0)
        ori t0, zero, 0x000F
        sh t0, 0x1C18(s0)
        ori t0, zero, 0x1FE6
        sh t0, 0x1C1A(s0)
        ori t0, zero, 2
        sh t0, 0x1D88(s0)
        jal frames
        ori a0, zero, 2
        jal sync
        nop
        ori t0, zero, 2
        sh t0, 0x1D8C(s0)
        jal sync
        nop
        lui a0, 0x8010
        ori a0, a0, 0xD00
        ori a2, zero, 0x800
        jal download
        ori a3, zero, 16
        jal silence
        nop

# T5: pitch modulation
        ori t0, zero, 0x1FC0
        sh t0, 0x1C1A(s0)
        ori t0, zero, 0x0208
        sh t0, 0x1C06(s0)
        sh t0, 0x1C16(s0)
        ori t0, zero, 0x0100
        sh t0, 0x1C04(s0)
        ori t0, zero, 0x1000
        sh t0, 0x1C14(s0)
        ori t0, zero, 2
        sh t0, 0x1D90(s0)
        jal sync
        nop
        ori t0, zero, 3
        sh t0, 0x1D88(s0)
        jal sync
        nop
        lui a0, 0x8010
        ori a0, a0, 0x1100
        ori a2, zero, 0x800
        jal download
        ori a3, zero, 16
        sh zero, 0x1D90(s0)
        jal silence
        nop

# T6: noise, then its rate changed
        ori t0, zero, 2
        sh t0, 0x1D94(s0)
        ori s7, zero, 0xFF00
        jal mode
        or a0, zero, zero
        jal sync
        nop
        ori t0, zero, 2
        sh t0, 0x1D88(s0)
        jal sync
        nop
        ori s7, zero, 0xE800
        ori t0, zero, 0xE840
        sh t0, 0x1DAA(s0)
        lui a0, 0x8010
        ori a0, a0, 0x1500
        ori a2, zero, 0x800
        jal download
        ori a3, zero, 16
        jal sync
        nop
        lui a0, 0x8010
        ori a0, a0, 0x1900
        ori a2, zero, 0x800
        jal download
        ori a3, zero, 16
        sh zero, 0x1D94(s0)
        ori s7, zero, 0xC000
        jal mode
        or a0, zero, zero
        jal silence
        nop

# T7: LSAX and ENDX around a key on with no loop start flag, and after an end and mute
        ori t0, zero, 0x1234
        sh t0, 0x1C3E(s0)
        ori t0, zero, 0x0204
        sh t0, 0x1C36(s0)
        ori t0, zero, 0x1000
        sh t0, 0x1C34(s0)
        ori t0, zero, 8
        sh t0, 0x1D88(s0)
        jal frames
        ori a0, zero, 1
        lhu t0, 0x1C3E(s0)
        nop
        sw t0, 0x1D00(s1)
        lhu t0, 0x1D9C(s0)
        nop
        sw t0, 0x1D04(s1)
        jal frames
        ori a0, zero, 3
        lhu t0, 0x1C3C(s0)
        nop
        sw t0, 0x1D08(s1)
        lhu t0, 0x1D9C(s0)
        nop
        sw t0, 0x1D0C(s1)
        lhu t0, 0x1C3E(s0)
        nop
        sw t0, 0x1D10(s1)
        ori t0, zero, 0x0208
        sh t0, 0x1C36(s0)
        ori t0, zero, 8
        sh t0, 0x1D88(s0)
        jal frames
        ori a0, zero, 1
        lhu t0, 0x1C3E(s0)
        nop
        sw t0, 0x1D14(s1)
        jal silence
        nop

# T8: the filter 5-7 blocks
        ori t0, zero, 0x0228
        sh t0, 0x1C16(s0)
        ori t0, zero, 0x0E00
        sh t0, 0x1C14(s0)
        ori t0, zero, 2
        sh t0, 0x1D88(s0)
        jal frames
        ori a0, zero, 20
        lui a0, 0x8010
        ori a0, a0, 0x1E00
        ori a2, zero, 0x800
        jal download
        ori a3, zero, 16
        jal silence
        nop

# T9: a pulse into "Room", the work area read with its writes stopped
        lui t9, 0x8001
        ori t9, t9, 0x20
        or t1, zero, zero
treg:   addu t2, t9, t1
        lhu t0, 0(t2)
        addu t2, s0, t1
        sh t0, 0x1DC0(t2)
        addiu t1, t1, 2
        slti t0, t1, 64
        bne t0, zero, treg
        nop
        ori t0, zero, 0xFB28
        sh t0, 0x1DA2(s0)
        ori t0, zero, 0x3FFF
        sh t0, 0x1C10(s0)
        sh t0, 0x1C12(s0)
        ori t0, zero, 2
        sh t0, 0x1D98(s0)
        ori s7, zero, 0xC080
        jal mode
        or a0, zero, zero
        ori t0, zero, 0x0204
        sh t0, 0x1C16(s0)
        ori t0, zero, 0x1000
        sh t0, 0x1C14(s0)
        ori t0, zero, 2
        sh t0, 0x1D88(s0)
        jal frames
        ori a0, zero, 3
        ori s7, zero, 0xC000
        jal mode
        or a0, zero, zero
        jal frames
        ori a0, zero, 1
        lui a0, 0x8010
        ori a0, a0, 0x2200
        lui a2, 0x0007
        ori a2, a2, 0xD940
        jal download
        ori a3, zero, 155
        jal silence
        nop
# T10: the same side's reflection alone, the work area zeroed first
        lui a0, 0x8013
        lui a2, 0x0007
        ori a2, a2, 0xD940
        jal upload
        ori a3, zero, 155
        lui t9, 0x8001
        ori t9, t9, 0x64                  # simple
        or t1, zero, zero
t10r:   addu t2, t9, t1
        lhu t0, 0(t2)
        addu t2, s0, t1
        sh t0, 0x1DC0(t2)
        addiu t1, t1, 2
        slti t0, t1, 64
        bne t0, zero, t10r
        nop
        ori t0, zero, 0xFB28
        sh t0, 0x1DA2(s0)
        ori t0, zero, 0x3FFF
        sh t0, 0x1C10(s0)
        sh t0, 0x1C12(s0)
        ori t0, zero, 0x0C0F
        sh t0, 0x1C18(s0)
        ori t0, zero, 0x0204
        sh t0, 0x1C16(s0)
        ori s7, zero, 0xC080
        jal mode
        or a0, zero, zero
        ori t0, zero, 2
        sh t0, 0x1D88(s0)
        jal frames
        ori a0, zero, 1
        ori s7, zero, 0xC000
        jal mode
        or a0, zero, zero
        jal frames
        ori a0, zero, 1
        lui a0, 0x8010
        ori a0, a0, 0x48C0
        lui a2, 0x0007
        ori a2, a2, 0xD940
        jal download
        ori a3, zero, 155
        jal silence
        nop
# T11: the fastest attack and decay
        ori t0, zero, 0x0200
        sh t0, 0x1C16(s0)
        ori t0, zero, 0x000F
        sh t0, 0x1C18(s0)
        sh zero, 0x1D98(s0)
        jal sync
        nop
        ori t0, zero, 2
        sh t0, 0x1D88(s0)
        jal sync
        nop
        lui a0, 0x8010
        ori a0, a0, 0x6F80
        ori a2, zero, 0x800
        jal download
        ori a3, zero, 16
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(s1)
done:   j done
        nop

# wave: s4 blocks at a3 from the headers at t9, the 16-sample pattern at 80010018h, the first a loop start and the
# last an end and repeat
wave:   lui t8, 0x8001
        ori t8, t8, 0x18
        or t2, zero, zero
        or t3, zero, zero
wblk:   addu t4, t9, t2
        lbu t4, 0(t4)
        nop
        sb t4, 0(a3)
        or t5, zero, zero
        bne t2, zero, wf1
        nop
        ori t5, zero, 4
wf1:    addiu t6, s4, -1
        bne t2, t6, wf2
        nop
        ori t5, t5, 3
wf2:    sb t5, 1(a3)
        or t6, zero, zero
wdat:   andi t7, t3, 7
        addu t7, t8, t7
        lbu t7, 0(t7)
        addu t4, a3, t6
        sb t7, 2(t4)
        addiu t3, t3, 1
        addiu t6, t6, 1
        slti t7, t6, 14
        bne t7, zero, wdat
        nop
        addiu a3, a3, 16
        addiu t2, t2, 1
        slt t7, t2, s4
        bne t7, zero, wblk
        nop
        jr ra
        nop

# silence: every voice off and at level zero, the silent loop, two frames
silence: or s6, ra, zero
        addiu t0, zero, -1
        sh t0, 0x1D8C(s0)
        ori t0, zero, 0xFF
        sh t0, 0x1D8E(s0)
        jal frames
        ori a0, zero, 2
        or t1, zero, zero
sinit:  addu t2, s0, t1
        sh zero, 0x1C0C(t2)
        ori t0, zero, 0x0202
        sh t0, 0x1C06(t2)
        addiu t1, t1, 16
        slti t0, t1, 0x180
        bne t0, zero, sinit
        nop
        ori t0, zero, 0x000F
        sh t0, 0x1C18(s0)
        ori t0, zero, 0x1FC0
        sh t0, 0x1C1A(s0)
        jr s6
        nop

# sync: IRQ9's flag acknowledged, some 6,000 cycles let pass so that the capture position has left IRQA's eight bytes,
# then the flag enabled again and polled: the capture buffers' write of their first halfword
sync:   sh s7, 0x1DAA(s0)
        ori t0, zero, 3000
sdelay: addiu t0, t0, -1
        bne t0, zero, sdelay
        nop
        ori t0, s7, 0x40
        sh t0, 0x1DAA(s0)
swait:  lhu t1, 0x1DAE(s0)
        nop
        andi t1, t1, 0x40
        beq t1, zero, swait
        nop
        jr ra
        nop

# frames: a0 vblanks, polled on I_STAT
frames: ori t0, zero, 1
fwait:  lw t1, 0x1070(s0)
        nop
        and t1, t1, t0
        beq t1, zero, fwait
        nop
        addiu t1, zero, -2
        sw t1, 0x1070(s0)
        addiu a0, a0, -1
        bne a0, zero, fwait
        nop
        jr ra
        nop

# mode: ATTR = s7 | a0, then STATX polled until its bits 0-5 show it
mode:   or t0, s7, a0
        sh t0, 0x1DAA(s0)
        andi t0, t0, 0x3F
mwait:  lhu t1, 0x1DAE(s0)
        nop
        andi t1, t1, 0x3F
        bne t1, t0, mwait
        nop
        jr ra
        nop

# upload: a3 blocks of 16 words from a0 to sound RAM at a2 by DMA4, then the FIFO emptied
upload: or s6, ra, zero
        or s3, a0, zero
        jal mode
        or a0, zero, zero
        srl t0, a2, 3
        sh t0, 0x1DA6(s0)
        jal mode
        ori a0, zero, 0x20
        sw s3, 0x10C0(s0)
        sll t0, a3, 16
        ori t0, t0, 0x10
        sw t0, 0x10C4(s0)
        lui t0, 0x0100
        ori t0, t0, 0x0201
        sw t0, 0x10C8(s0)
uwait:  lw t1, 0x10C8(s0)
        lui t0, 0x0100
        and t1, t1, t0
        bne t1, zero, uwait
        nop
ubusy:  lhu t1, 0x1DAE(s0)
        nop
        andi t1, t1, 0x400
        bne t1, zero, ubusy
        nop
        jal mode
        or a0, zero, zero
        jr s6
        nop

# download: a3 blocks of 16 words from sound RAM at a2 to a0 by DMA4
download: or s6, ra, zero
        or s3, a0, zero
        jal mode
        or a0, zero, zero
        srl t0, a2, 3
        sh t0, 0x1DA6(s0)
        jal mode
        ori a0, zero, 0x30
        sw s3, 0x10C0(s0)
        sll t0, a3, 16
        ori t0, t0, 0x10
        sw t0, 0x10C4(s0)
        lui t0, 0x0100
        ori t0, t0, 0x0200
        sw t0, 0x10C8(s0)
dwait:  lw t1, 0x10C8(s0)
        lui t0, 0x0100
        and t1, t1, t0
        bne t1, zero, dwait
        nop
        jal mode
        or a0, zero, zero
        jr s6
        nop
