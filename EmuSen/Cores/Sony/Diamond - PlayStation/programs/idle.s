# The idle skipper's program (Diamond_Speed.md §4.5): the loop forms games wait in, each recording what it leaves, so
# that a run with the skipper and one without can be compared word for word and state for state. Bare metal, timer 2
# counting the system clock. Results from 80100000h: 000h eight vblank waits with a timeout on the stack, each the
# timeout left and timer 2; 040h register timeouts that expire, after 9 to 13, 500 and 1001 turns; 080h the same on the
# stack; 0C0h a DMA6 and a DMA2 transfer waited for on their CHCR; 0D0h a wait on timer 2 itself; 0E0h
# timeouts counting up, after 9 to 13 and 777 turns, each turn ending on a store; 100h three waits on I_STAT for the
# vblank whose turns grow the turn after timer 2's interrupt is flagged, the flag carried in the scratchpad, so that
# nothing in the registers or the bus's timing shows the event. 800FFFF0h holds D1A0D00Eh when
# done, and the program then idles on a branch to itself.
        j start
        nop
vector: j handler                     # copied to 80000080h
        nop
handler: lui k0, 0x1F80
        addiu k1, zero, -2
        sw k1, 0x1070(k0)             # the vblank acknowledged
        lui k1, 0x8002
        lw k0, 0(k1)
        nop
        addiu k0, k0, 1
        sw k0, 0(k1)                  # the vblank count at 80020000h
        mfc0 k0, EPC
        nop
        jr k0
        rfe
# vsync(a0 the count to wait for): the timeout loop of a library's VSync, its counter on the stack; v0 the timeout left
vsync:  lui t3, 0x0007
        sw t3, 0x10(sp)
        addiu v1, zero, -1
        lui t1, 0x8002
vw:     lw v0, 0x10(sp)
        nop
        addiu v0, v0, -1
        sw v0, 0x10(sp)
        lw v0, 0x10(sp)
        nop
        bne v0, v1, vp
        nop
        jr ra
        nop
vp:     lw v0, 0(t1)
        nop
        slt v0, v0, a0
        bne v0, zero, vw
        nop
        lw v0, 0x10(sp)
        jr ra
        nop
# regwait(a0 turns): a poll of a word nothing sets, its timeout in s0; v0 what is left
regwait: or s0, a0, zero
        lui t1, 0x8002
rw:     lw v0, 4(t1)
        nop
        bne v0, zero, rd
        addiu s0, s0, -1
        bne s0, zero, rw
        nop
rd:     jr ra
        or v0, s0, zero
# memwait(a0 turns): the same with its timeout on the stack
memwait: sw a0, 0x14(sp)
        lui t1, 0x8002
mw:     lw v0, 4(t1)
        nop
        bne v0, zero, md
        nop
        lw v0, 0x14(sp)
        nop
        addiu v0, v0, -1
        sw v0, 0x14(sp)
        bne v0, zero, mw
        nop
md:     jr ra
        lw v0, 0x14(sp)
# upwait(a0 turns): a poll counting up to its limit in s1, the word it polls copied to the scratchpad after each read
upwait: addu s1, zero, zero
        lui t1, 0x8002
        lui t2, 0x1F80
uw:     lw v0, 4(t1)
        addiu s1, s1, 1
        sw v0, 0x10(t2)
        beq s1, a0, ud
        nop
        lw v0, 4(t1)
        nop
        beq v0, zero, uw
        sw v0, 8(t1)
ud:     jr ra
        or v0, s1, zero
# stamp(a1 where): timer 2's count to a1
stamp:  lui t0, 0x1F80
        lhu t0, 0x1120(t0)
        jr ra
        sw t0, 0(a1)
start:  mtc0 zero, SR
        lui sp, 0x801F
        ori sp, sp, 0xFF00
        lui t0, 0x8001
        lw t1, 8(t0)
        lw t2, 12(t0)
        lui t3, 0x8000
        sw t1, 0x80(t3)
        sw t2, 0x84(t3)
        lui s7, 0x1F80
        sh zero, 0x1124(s7)           # timer 2: the system clock, free running
        lui s6, 0x8010                # results
        lui t0, 0x8002
        sw zero, 0(t0)
        sw zero, 4(t0)
        sw zero, 0x1070(s7)
        addiu t0, zero, 1
        sw t0, 0x1074(s7)             # I_MASK: the vblank
        addiu t0, zero, 0x0401
        mtc0 t0, SR                   # IEc and IM2
        addiu s5, zero, 8
v8:     lui t0, 0x8002
        lw a0, 0(t0)
        nop
        jal vsync
        addiu a0, a0, 1
        sw v0, 0(s6)
        jal stamp
        addiu a1, s6, 4
        addiu s5, s5, -1
        bne s5, zero, v8
        addiu s6, s6, 8
        mtc0 zero, SR
        lui s6, 0x8010
        addiu s6, s6, 0x40
        addiu s4, zero, 9
r5:     jal regwait
        or a0, s4, zero
        jal stamp
        or a1, s6, zero
        addiu s4, s4, 1
        sltiu t0, s4, 14
        bne t0, zero, r5
        addiu s6, s6, 4
        jal regwait
        addiu a0, zero, 500
        jal stamp
        or a1, s6, zero
        jal regwait
        addiu a0, zero, 1001
        jal stamp
        addiu a1, s6, 4
        lui s6, 0x8010
        addiu s6, s6, 0x80
        addiu s4, zero, 9
m5:     jal memwait
        or a0, s4, zero
        jal stamp
        or a1, s6, zero
        addiu s4, s4, 1
        sltiu t0, s4, 14
        bne t0, zero, m5
        addiu s6, s6, 4
        jal memwait
        addiu a0, zero, 500
        jal stamp
        or a1, s6, zero
        jal memwait
        addiu a0, zero, 1001
        jal stamp
        addiu a1, s6, 4
        lui s6, 0x8010
        addiu s6, s6, 0xE0
        addiu s4, zero, 9
u5:     jal upwait
        or a0, s4, zero
        jal stamp
        or a1, s6, zero
        addiu s4, s4, 1
        sltiu t0, s4, 14
        bne t0, zero, u5
        addiu s6, s6, 4
        jal upwait
        addiu a0, zero, 777
        jal stamp
        or a1, s6, zero
        lui s6, 0x8010
        addiu s6, s6, 0xC0
        lui t0, 0x0800
        ori t0, t0, 0x0B00
        sw t0, 0x10F0(s7)             # DPCR: DMA2 and DMA6 on
        lui t0, 0x8004
        ori t0, t0, 0x0FFC
        sw t0, 0x10E0(s7)             # DMA6 from 80040FFCh down
        addiu t0, zero, 0x0400
        sw t0, 0x10E4(s7)
        lui t0, 0x1100
        ori t0, t0, 0x0002
        sw t0, 0x10E8(s7)             # start and trigger, backward
        lui t1, 0x0100
dw:     lw t0, 0x10E8(s7)
        nop
        and t0, t0, t1
        bne t0, zero, dw
        nop
        jal stamp
        or a1, s6, zero
        lui t0, 0x0400
        ori t0, t0, 2
        sw t0, 0x1814(s7)             # GP1(04h): DMA to GP0
        lui t0, 0xA000
        sw t0, 0x1810(s7)             # GP0(A0h): 64 by 64 to VRAM at 0,0
        sw zero, 0x1810(s7)
        lui t0, 0x0040
        ori t0, t0, 0x0040
        sw t0, 0x1810(s7)
        lui t0, 0x8004
        sw t0, 0x10A0(s7)             # DMA2 from 80040000h, 128 blocks of 16 words
        lui t0, 0x0080
        ori t0, t0, 0x0010
        sw t0, 0x10A4(s7)
        lui t0, 0x0100
        ori t0, t0, 0x0201
        sw t0, 0x10A8(s7)             # from RAM, in blocks, started
        lui t1, 0x0100
gw:     lw t0, 0x10A8(s7)
        nop
        and t0, t0, t1
        bne t0, zero, gw
        nop
        jal stamp
        addiu a1, s6, 4
        lui s6, 0x8010
        addiu s6, s6, 0xD0
        lhu t1, 0x1120(s7)
        nop
        addiu t1, t1, 0x0800
        andi t1, t1, 0xFFFF
tw:     lhu t0, 0x1120(s7)
        nop
        sltu t0, t0, t1
        bne t0, zero, tw
        nop
        jal stamp
        or a1, s6, zero
        lui s6, 0x8010
        addiu s6, s6, 0x100
        addiu t0, zero, 0x3000
        sh t0, 0x1128(s7)             # timer 2's target
        addiu t0, zero, 0x0058
        sh t0, 0x1124(s7)             # timer 2: reset and IRQ at the target, repeated
        sw zero, 0x1070(s7)           # I_STAT clear; I_MASK keeps the vblank, SR its IEc clear
        addiu t2, zero, 3
        lui t4, 0x1F80
        sw zero, 0x10(t4)
iw:     lw v0, 0x10(t4)               # the last turn's I_STAT, kept in the scratchpad
        nop
        andi v1, v0, 0x40
        beq v1, zero, is
        nop
        nop
        nop
        nop
is:     lw v0, 0x1070(s7)
        nop
        sw v0, 0x10(t4)
        andi v0, v0, 1
        or v1, zero, zero
        beq v0, zero, iw
        or v0, zero, zero
        jal stamp
        or a1, s6, zero
        sw zero, 0x1070(s7)
        addiu t2, t2, -1
        bne t2, zero, iw
        addiu s6, s6, 4
        lui t0, 0xD1A0
        ori t0, t0, 0xD00E
        lui t1, 0x8010
        sw t0, -0x10(t1)
idle:   beq zero, zero, idle
        nop
