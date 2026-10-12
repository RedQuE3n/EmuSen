# Whether channel 6 runs beside channel 2's list (Diamond_Disputes.md D-41). Channel 2 is given a list and channel
# 6 is then started over sixteen words filled with 5A5A5A5Ah. Cases 0 and 1: four rectangles of 1023x511, which keep
# the GPU drawing and channel 2 waiting for its request. Cases 2 and 3: 4,096 nodes of no words, which channel 2
# walks with nothing to wait for. Cases 0 and 2 rank channel 2 above channel 6 in DPCR (3 against 7, as a program
# that draws with lists leaves them), cases 1 and 3 below it (7 against 0). Interrupts off. Results from 80100000h,
# eight words a case: channel 2's CHCR before channel 6 starts; GPUSTAT then; channel 6's CHCR, the table's lowest
# word and channel 2's CHCR straight after the start; and the same three once channel 2 has ended. 800FFFF0h holds
# D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x8010
        lui a1, 0x1F80
        sw zero, 0x1074(a1)       # I_MASK: nothing
        lui s0, 0x0F00
        ori s0, s0, 0x0B00        # DPCR: channel 2 at 3, channel 6 at 7
        addiu s3, zero, 0         # rectangles
        jal case
        nop
        lui s0, 0x0800
        ori s0, s0, 0x0F00        # DPCR: channel 2 at 7, channel 6 at 0
        jal case
        nop
        lui s0, 0x0F00
        ori s0, s0, 0x0B00
        addiu s3, zero, 1         # nodes of no words
        jal case
        nop
        lui s0, 0x0800
        ori s0, s0, 0x0F00
        jal case
        nop
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        lui t0, 0x8010
        sw t1, -16(t0)
done:   j done
        nop

case:   or s7, ra, zero
        sw zero, 0x1814(a1)       # GP1(00h)
        lui t0, 0x0400
        ori t0, t0, 2
        sw t0, 0x1814(a1)         # GP1(04h): DMA direction 2, CPU to GP0
        lui t0, 0xE300
        sw t0, 0x1810(a1)         # the drawing area from 0,0
        lui t0, 0xE407
        ori t0, t0, 0xFFFF
        sw t0, 0x1810(a1)         # to 1023,511
        lui t0, 0xE500
        sw t0, 0x1810(a1)         # no offset
        sw s0, 0x10F0(a1)
        lui s1, 0x8010
        ori s1, s1, 0x0400        # the table, sixteen words
        lui t0, 0x5A5A
        ori t0, t0, 0x5A5A
        addiu t1, zero, 16
        or t2, s1, zero
fill:   sw t0, 0(t2)
        addiu t1, t1, -1
        bne t1, zero, fill
        addiu t2, t2, 4
        lui s2, 0x8010
        ori s2, s2, 0x0800        # the list
        bne s3, zero, empty
        nop
        addiu t1, zero, 4         # four nodes of sixteen bytes
        or t2, s2, zero
node:   addiu t3, t2, 16
        lui t0, 0x00FF
        ori t0, t0, 0xFFFF
        and t3, t3, t0            # the next node's address
        addiu t4, zero, 1
        bne t1, t4, link
        nop
        or t3, t0, zero           # the last node ends the list
link:   lui t0, 0x0300
        or t0, t0, t3
        sw t0, 0(t2)              # three words, and the next
        lui t0, 0x6020
        ori t0, t0, 0x4060
        sw t0, 4(t2)              # GP0(60h): a rectangle of one colour
        sw zero, 8(t2)            # at 0,0
        lui t0, 0x01FF
        ori t0, t0, 0x03FF
        sw t0, 12(t2)             # 1023 by 511
        addiu t1, t1, -1
        bne t1, zero, node
        addiu t2, t2, 16
        j go
        addiu s4, zero, 512       # the first nodes sent and the GPU drawing before channel 6 starts
empty:  addiu t1, zero, 4096      # 4,096 nodes of a header alone, each naming the next
        or t2, s2, zero
        lui t0, 0x00FF
        ori t0, t0, 0xFFFF
none:   addiu t3, t2, 4
        and t3, t3, t0
        sw t3, 0(t2)
        addiu t1, t1, -1
        bne t1, zero, none
        addiu t2, t2, 4
        sw t0, -4(t2)             # the last node ends the list
        addiu s4, zero, 16        # a few nodes walked before channel 6 starts
go:     lui t0, 0x00FF
        ori t0, t0, 0xFFFF
        and t1, s2, t0
        sw t1, 0x10A0(a1)         # channel 2's MADR
        sw zero, 0x10A4(a1)
        lui t0, 0x0100
        ori t0, t0, 0x0401
        sw t0, 0x10A8(a1)         # CHCR: a list from RAM, started
        or t0, s4, zero
settle: bne t0, zero, settle
        addiu t0, t0, -1
        lw t0, 0x10A8(a1)
        lw t1, 0x1814(a1)
        sw t0, 0(a0)              # channel 2's CHCR before
        sw t1, 4(a0)              # GPUSTAT before
        lui t0, 0x00FF
        ori t0, t0, 0xFFFF
        addiu t1, s1, 60
        and t1, t1, t0
        sw t1, 0x10E0(a1)         # channel 6's MADR: the table's last word
        addiu t0, zero, 16
        sw t0, 0x10E4(a1)
        lui t0, 0x1100
        ori t0, t0, 2
        sw t0, 0x10E8(a1)         # CHCR: start and trigger
        lw t0, 0x10E8(a1)
        lw t1, 0(s1)
        lw t2, 0x10A8(a1)
        sw t0, 8(a0)              # channel 6's CHCR straight after
        sw t1, 12(a0)             # the table's lowest word
        sw t2, 16(a0)             # channel 2's CHCR
        lui t3, 0x0100
        lui t4, 0x0080            # at most 8,388,608 polls
wait:   lw t0, 0x10A8(a1)
        addiu t4, t4, -1
        and t0, t0, t3
        beq t0, zero, ended
        nop
        bne t4, zero, wait
        nop
ended:  addiu t0, zero, 256
rest:   bne t0, zero, rest
        addiu t0, t0, -1
        lw t0, 0x10E8(a1)
        lw t1, 0(s1)
        lw t2, 0x10A8(a1)
        sw t0, 20(a0)             # the same three once channel 2 has ended
        sw t1, 24(a0)
        sw t2, 28(a0)
        addiu a0, a0, 32
        jr s7
        nop
