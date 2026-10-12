# The SPU's voices as the mixed output hears them (Diamond_SPU.md §10): scenes on voice 1, each keyed on at a vblank and
# separated by silence, for the references' audio. Sound RAM from 1000h: a DC block of nibble 7 that loops on itself,
# a silent loop, a one-shot pair (DC, then a block with end and mute), and from 1040h a 16-block wave of a 16-sample
# pattern, each block with its own filter and shift (0 to 4, 0 to 15), looping from its first. The scenes: A linear
# ADSR; B exponential ADSR; C the volume sweeps; D the wave at eight pitches; E pitch modulation by voice 0, then with
# the pitch's sign glitch; F noise at three rates; G the one-shot, then a start inside the wave with no loop start
# flag; H a pulse into psx-spx's "Room" reverb. Between scenes three frames at main volume zero with every voice off
# and at level zero, so that each scene starts from exact silence. The scenes begin 400 frames after the program does, so that a
# referee's recording is cut at frame 400 unheard before (Diamond_SPU.md §10.1); 801000FCh holds 5350553Fh from the
# start. Interrupts off; 800FFFF0h holds D1A0D00Eh when done, 80100000h the frame count at each scene's key on.
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
start:  mtc0 zero, SR
        lui s0, 0x1F80
        lui s1, 0x8010
        lui t0, 0x5350
        ori t0, t0, 0x553F
        sw t0, 0xFC(s1)                   # the start marker
        or s5, zero, zero                 # frames counted
        lui t0, 0x2209
        ori t0, t0, 0x31E1
        sw t0, 0x1014(s0)                 # DEV4: 220931E1h, psx-spx's value for reads
        lw t0, 0x10F0(s0)
        lui t1, 0x0008
        or t0, t0, t1
        sw t0, 0x10F0(s0)                 # DPCR: DMA4 on
        ori t0, zero, 4
        sh t0, 0x1DAC(s0)                 # RAM_CTRL
        ori s7, zero, 0xC000              # ATTR: on, unmuted
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
        sh zero, 0x1DB4(s0)
        sh zero, 0x1DB6(s0)
        addiu t0, zero, -1
        sh t0, 0x1D8C(s0)                 # key off every voice
        ori t0, zero, 0xFF
        sh t0, 0x1D8E(s0)
# every voice silent: volumes zero, the silent loop at 1010h, pitch 1000h
        or t1, zero, zero
vinit:  addu t2, s0, t1
        sh zero, 0x1C00(t2)
        sh zero, 0x1C02(t2)
        ori t0, zero, 0x1000
        sh t0, 0x1C04(t2)
        ori t0, zero, 0x0202
        sh t0, 0x1C06(t2)
        sh zero, 0x1C08(t2)
        sh zero, 0x1C0A(t2)
        addiu t1, t1, 16
        slti t0, t1, 0x180
        bne t0, zero, vinit
        nop
# sound RAM's image at 80120000h: DC loop, silent loop, the one-shot pair, padding, the wave
        lui a2, 0x8012
        or t1, zero, zero
zero1:  addu t2, a2, t1
        sw zero, 0(t2)
        addiu t1, t1, 4
        slti t0, t1, 0x140
        bne t0, zero, zero1
        nop
        ori t0, zero, 0x0700
        sh t0, 0(a2)                      # 1000h: shift 0, filter 0, start + end + repeat
        sh t0, 0x10(a2)                   # 1010h: the same, silent
        sh zero, 0x20(a2)                 # 1020h: no flags
        ori t0, zero, 0x0100
        sh t0, 0x30(a2)                   # 1030h: end and mute
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
        ori t8, t9, 0x18                  # pattern, at 80010018h
        ori a3, a2, 0x40                  # the wave's first block
        or t2, zero, zero                 # block
        or t3, zero, zero                 # the pattern's byte
wblk:   addu t4, t9, t2
        lbu t4, 0(t4)
        nop
        sb t4, 0(a3)
        or t5, zero, zero
        bne t2, zero, wf1
        nop
        ori t5, zero, 4                   # block 0: loop start
wf1:    ori t6, zero, 15
        bne t2, t6, wf2
        nop
        ori t5, zero, 3                   # block 15: end + repeat
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
        slti t7, t2, 16
        bne t7, zero, wblk
        nop
        or a0, a2, zero
        ori a2, zero, 0x1000
        jal upload
        ori a3, zero, 5
# the reverb's work area for "Room", 26C0h bytes from FB28h × 8, zeroed from 80130000h
        lui a0, 0x8013
        or t1, zero, zero
zero2:  addu t2, a0, t1
        sw zero, 0(t2)
        addiu t1, t1, 4
        slti t0, t1, 0x2700
        bne t0, zero, zero2
        nop
        lui a2, 0x0007
        ori a2, a2, 0xD940
        jal upload
        ori a3, zero, 156
        jal frames
        ori a0, zero, 400

# A: linear ADSR: attack shift 10 +7, decay 6, sustain level 8; sustain linear down shift 16 -7; release exponential 12
        ori t0, zero, 0x3FFF
        sh t0, 0x1C10(s0)
        sh t0, 0x1C12(s0)
        ori t0, zero, 0x1000
        sh t0, 0x1C14(s0)
        ori t0, zero, 0x0200
        sh t0, 0x1C16(s0)
        ori t0, zero, 0x2868
        sh t0, 0x1C18(s0)
        ori t0, zero, 0x506C
        sh t0, 0x1C1A(s0)
        jal keyon
        nop
        jal frames
        ori a0, zero, 15
        jal keyoff
        nop
        jal frames
        ori a0, zero, 25
        jal gap
        nop
# B: exponential ADSR: attack shift 13 +6, decay 2, sustain level 12; sustain exponential up shift 20 +5; release linear 10
        ori t0, zero, 0xB52C
        sh t0, 0x1C18(s0)
        ori t0, zero, 0x948A
        sh t0, 0x1C1A(s0)
        jal keyon
        nop
        jal frames
        ori a0, zero, 15
        jal keyoff
        nop
        jal frames
        ori a0, zero, 15
        jal gap
        nop
# C: the sweeps, left linear up shift 8 from zero, right exponential down shift 10 from 7FFEh
        ori t0, zero, 0x000F
        sh t0, 0x1C18(s0)
        ori t0, zero, 0x1FC5
        sh t0, 0x1C1A(s0)
        sh zero, 0x1C10(s0)
        jal frames
        ori a0, zero, 1
        ori t0, zero, 0x8020
        sh t0, 0x1C10(s0)
        ori t0, zero, 0xE028
        sh t0, 0x1C12(s0)
        jal keyon
        nop
        jal frames
        ori a0, zero, 20
        jal keyoff
        nop
        jal frames
        ori a0, zero, 10
        jal gap
        nop
# D: the wave at eight pitches, six frames each
        ori t0, zero, 0x3FFF
        sh t0, 0x1C10(s0)
        sh t0, 0x1C12(s0)
        ori t0, zero, 0x0208
        sh t0, 0x1C16(s0)
        jal keyon
        nop
        lui t9, 0x8001
        ori t9, t9, 0x0FF0
        ori t0, zero, 0x1000
        sh t0, 0(t9)
        ori t0, zero, 0x0E00
        sh t0, 2(t9)
        ori t0, zero, 0x1800
        sh t0, 4(t9)
        ori t0, zero, 0x2C00
        sh t0, 6(t9)
        ori t0, zero, 0x3FFF
        sh t0, 8(t9)
        ori t0, zero, 0x0400
        sh t0, 10(t9)
        ori t0, zero, 0x7000
        sh t0, 12(t9)
        ori t0, zero, 0x0123
        sh t0, 14(t9)
        or s4, zero, zero
dpitch: addu t1, t9, s4
        lhu t0, 0(t1)
        nop
        sh t0, 0x1C14(s0)
        jal frames
        ori a0, zero, 6
        addiu s4, s4, 2
        slti t0, s4, 16
        bne t0, zero, dpitch
        nop
        jal keyoff
        nop
        jal frames
        ori a0, zero, 10
        jal gap
        nop
# E: pitch modulation: voice 0 the wave at 80h, silent, modulating voice 1's wave at 1000h; then voice 1 at 9000h
        ori t0, zero, 0x1000
        sh t0, 0x1C14(s0)
        ori t0, zero, 0x0080
        sh t0, 0x1C04(s0)
        ori t0, zero, 0x0208
        sh t0, 0x1C06(s0)
        ori t0, zero, 0x000F
        sh t0, 0x1C08(s0)
        ori t0, zero, 0x1FC5
        sh t0, 0x1C0A(s0)
        ori t0, zero, 2
        sh t0, 0x1D90(s0)
        ori t0, zero, 3
        jal keyonv
        nop
        jal frames
        ori a0, zero, 20
        ori t0, zero, 0x9000
        sh t0, 0x1C14(s0)
        jal frames
        ori a0, zero, 10
        ori t0, zero, 3
        sh t0, 0x1D8C(s0)
        jal frames
        ori a0, zero, 10
        sh zero, 0x1D90(s0)
        jal gap
        nop
# F: noise on voice 1, shift 15 step 7, shift 10 step 4, shift 4 step 6
        ori t0, zero, 0x1000
        sh t0, 0x1C14(s0)
        ori t0, zero, 2
        sh t0, 0x1D94(s0)
        ori s7, zero, 0xFF00
        jal mode
        or a0, zero, zero
        jal keyon
        nop
        jal frames
        ori a0, zero, 6
        ori s7, zero, 0xE800
        jal mode
        or a0, zero, zero
        jal frames
        ori a0, zero, 6
        ori s7, zero, 0xD200
        jal mode
        or a0, zero, zero
        jal frames
        ori a0, zero, 6
        jal keyoff
        nop
        sh zero, 0x1D94(s0)
        ori s7, zero, 0xC000
        jal mode
        or a0, zero, zero
        jal frames
        ori a0, zero, 10
        jal gap
        nop
# G: the one-shot pair, attack shift 4; then a start at the wave's block 14, which has no loop start flag
        ori t0, zero, 0x0204
        sh t0, 0x1C16(s0)
        ori t0, zero, 0x100F
        sh t0, 0x1C18(s0)
        jal keyon
        nop
        jal frames
        ori a0, zero, 8
        jal keyoff
        nop
        jal frames
        ori a0, zero, 4
        jal gap
        nop
        ori t0, zero, 0x0224
        sh t0, 0x1C16(s0)
        jal keyon
        nop
        jal frames
        ori a0, zero, 10
        jal keyoff
        nop
        jal frames
        ori a0, zero, 10
        jal gap
        nop
# H: the one-shot pulse into "Room": its 32 registers, ESA FB28h, EVOL 4000h, EON voice 1, the master enable
        lui t9, 0x8001
        ori t9, t9, 0x20                  # room
        or t1, zero, zero
hreg:   addu t2, t9, t1
        lhu t0, 0(t2)
        addu t2, s0, t1
        sh t0, 0x1DC0(t2)
        addiu t1, t1, 2
        slti t0, t1, 64
        bne t0, zero, hreg
        nop
        ori t0, zero, 0xFB28
        sh t0, 0x1DA2(s0)
        ori t0, zero, 0x4000
        sh t0, 0x1D84(s0)
        sh t0, 0x1D86(s0)
        ori t0, zero, 2
        sh t0, 0x1D98(s0)
        ori s7, zero, 0xC080
        jal mode
        or a0, zero, zero
        ori t0, zero, 0x0204
        sh t0, 0x1C16(s0)
        ori t0, zero, 0x000F
        sh t0, 0x1C18(s0)
        jal keyon
        nop
        jal frames
        ori a0, zero, 60
        jal keyoff
        nop
        jal frames
        ori a0, zero, 4
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        lui t2, 0x8010
        sw t1, -16(t2)
done:   j done
        nop

# keyon: the main volume up and voice 1 on, the frame count recorded; keyonv: the voices in t0
keyon:  ori t0, zero, 2
keyonv: ori t1, zero, 0x3FFF
        sh t1, 0x1D80(s0)
        sh t1, 0x1D82(s0)
        sh t0, 0x1D88(s0)
        sll t1, s5, 0
        sw t1, 0(s1)
        jr ra
        addiu s1, s1, 4
keyoff: ori t0, zero, 2
        jr ra
        sh t0, 0x1D8C(s0)

# gap: every voice off, voices 0 and 1 at zero, the main volume at zero, three frames, so that each scene starts from
# silence exactly
gap:    or s6, ra, zero
        addiu t0, zero, -1
        sh t0, 0x1D8C(s0)
        sh t0, 0x1D8E(s0)
        sh zero, 0x1D80(s0)
        sh zero, 0x1D82(s0)
        sh zero, 0x1C0C(s0)
        sh zero, 0x1C1C(s0)
        jal frames
        ori a0, zero, 3
        sh zero, 0x1C0C(s0)
        sh zero, 0x1C1C(s0)
        jr s6
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
        addiu s5, s5, 1
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
