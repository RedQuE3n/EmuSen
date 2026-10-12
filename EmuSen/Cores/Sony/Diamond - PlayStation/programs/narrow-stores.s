# A byte or halfword store to a 32-bit register at an address not word-aligned (psx-spx "Unpredictable Things", "(w32)"):
# DICR, DPCR, timer 0's target, I_MASK and channel 2's MADR, each read back. Interrupts off. Results from 80100000h in the
# order below; 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x8010
        lui a1, 0x1F80
        sw zero, 0x10F4(a1)
        addiu t0, zero, 0x88
        sb t0, 0x10F6(a1)         # 0: DICR's third byte, 88h
        lw t1, 0x10F4(a1)
        nop
        sw t1, 0(a0)
        addiu t0, zero, 0x3F
        sw t0, 0x10F4(a1)
        lui t0, 0x1234
        ori t0, t0, 0x0044
        sh t0, 0x10F6(a1)         # 1: DICR's upper half from 12340044h, bits 0-5 set before
        lw t1, 0x10F4(a1)
        nop
        sw t1, 4(a0)
        sw zero, 0x10F4(a1)
        lw t2, 0x10F0(a1)         # DPCR kept to restore
        addiu t0, zero, 0x0B
        sb t0, 0x10F1(a1)         # 2: DPCR's second byte, 0Bh
        lw t1, 0x10F0(a1)
        nop
        sw t1, 8(a0)
        sw t2, 0x10F0(a1)
        sw zero, 0x1108(a1)
        addiu t0, zero, 0x1234
        sh t0, 0x110A(a1)         # 3: timer 0's target at its upper half
        lw t1, 0x1108(a1)
        nop
        andi t1, t1, 0xFFFF
        sw t1, 12(a0)
        sh t0, 0x1108(a1)         # 4: the same at its own address
        lw t1, 0x1108(a1)
        nop
        andi t1, t1, 0xFFFF
        sw t1, 16(a0)
        sw zero, 0x1074(a1)
        addiu t0, zero, 0x05
        sb t0, 0x1075(a1)         # 5: I_MASK's second byte, 05h
        lhu t1, 0x1074(a1)
        nop
        sw t1, 20(a0)
        sw zero, 0x1074(a1)
        sw zero, 0x10A0(a1)
        addiu t0, zero, 0x12
        sb t0, 0x10A2(a1)         # 6: channel 2's MADR, third byte 12h
        lw t1, 0x10A0(a1)
        nop
        sw t1, 24(a0)
        sw zero, 0x10A0(a1)
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(a0)
done:   j done
        nop
