# A DMA interrupt handled without acknowledging its channel's flag (Diamond_Disputes.md D-29, Diamond_Records.md
# §16.3): channel 6 completes with its interrupt enabled, and the handler acknowledges I_STAT's bit 3 and writes DICR
# with channel 3's flag acknowledged, leaving channel 6's up. The write leaves bit 31 up, so nothing asks again and the
# handler runs once; were the write to set I_STAT's bit 3, it would run without end, which the handler stops at 1,000
# by clearing I_MASK. Installs its own vector, the I-cache off. Results from 80100000h: the handler's runs, DICR and
# I_STAT's bit 3 after 4,096 turns of a loop; 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
vector: j handler
        nop
start:  mtc0 zero, SR
        lui t0, 0xFFFE
        lui t1, 0x0001
        ori t1, t1, 0xE188
        sw t1, 0x130(t0)          # BCC: the I-cache off, so the vector is fetched as written
        lui t0, 0x8001
        lw t1, 8(t0)
        lw t2, 12(t0)
        lui t3, 0x8000
        sw t1, 0x80(t3)
        sw t2, 0x84(t3)
        lui a0, 0x8010
        lui a1, 0x1F80
        sw zero, 0(a0)
        sw zero, 0x1074(a1)       # I_MASK: nothing yet
        lui t0, 0x0800
        sw t0, 0x10F0(a1)         # DPCR: channel 6 enabled
        lui t0, 0x7F00
        sw t0, 0x10F4(a1)         # DICR: every flag acknowledged
        lui t0, 0x00C8
        sw t0, 0x10F4(a1)         # the master enable, channel 6's and channel 3's
        sw zero, 0x1070(a1)       # I_STAT: everything acknowledged
        addiu t0, zero, 8
        sw t0, 0x1074(a1)         # I_MASK: the DMA's
        addiu t0, zero, 0x0401
        mtc0 t0, SR               # IM2 and IEc
        lui t0, 0x0010
        ori t0, t0, 0x040C
        sw t0, 0x10E0(a1)         # MADR: four words at 80100400h
        addiu t0, zero, 4
        sw t0, 0x10E4(a1)         # BCR
        lui t0, 0x1100
        ori t0, t0, 2
        sw t0, 0x10E8(a1)         # CHCR: start and trigger
        addiu t0, zero, 4096
turn:   bne t0, zero, turn
        addiu t0, t0, -1
        mtc0 zero, SR
        lw t0, 0x10F4(a1)
        lw t1, 0x1070(a1)
        sw t0, 4(a0)
        andi t1, t1, 8
        sw t1, 8(a0)
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(a0)
done:   j done
        nop

handler: lui k0, 0x8010
        lw k1, 0(k0)
        nop
        addiu k1, k1, 1
        sw k1, 0(k0)
        lui k0, 0x1F80
        slti k1, k1, 1000
        bne k1, zero, ack
        nop
        sw zero, 0x1074(k0)       # the thousandth run: I_MASK cleared, so the program ends
ack:    ori k1, zero, 0xFFF7
        sw k1, 0x1070(k0)         # I_STAT's bit 3 acknowledged
        lui k1, 0x08C8
        sw k1, 0x10F4(k0)         # DICR: channel 3's flag acknowledged, channel 6's left up
        mfc0 k0, EPC
        nop
        jr k0
        rfe
