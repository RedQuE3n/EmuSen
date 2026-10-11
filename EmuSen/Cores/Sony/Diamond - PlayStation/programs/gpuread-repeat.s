# D-23: what GPUREAD gives once a VRAM-to-CPU transfer is read (Diamond_Disputes.md D-23, Diamond_GPU.md §3). Six
# pixels written at 0,0 as 0101h to 0606h; a transfer of those six read as its three words and two more, then one of
# the pixel at 1,0 read as its word and four more. The ten words are stored at 80100000h. 800FFFF0h holds D1A0D00Eh
# when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x1F80
        sw zero, 0x1814(a0)           # GP1(00h)
        bgezal zero, go
        nop
list:
        .word 6                      # GP0(A0h): six pixels at 0,0
        .word 0xA0000000
        .word 0x00000000
        .word 0x00010006
        .word 0x02020101
        .word 0x04040303
        .word 0x06060505
        .word 0
go:     or a1, ra, zero
        jal send
        nop
        lui t9, 0x8010
        or a1, zero, zero
        lui a2, 1
        ori a2, a2, 6
        jal readmore
        ori a3, zero, 5
        ori a1, zero, 1
        lui a2, 1
        ori a2, a2, 1
        jal readmore
        ori a3, zero, 5
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        lui t0, 0x8010
        sw t1, -16(t0)
done:   j done
        nop
# a transfer from yx a1, size a2, once the GPU is idle: its ready bit polled, then a3 words read with no poll
readmore: lw t0, 0x1814(a0)
        lui t2, 0x1400
        and t0, t0, t2
        bne t0, t2, readmore
        nop
        lui t0, 0xC000
        sw t0, 0x1810(a0)
        sw a1, 0x1810(a0)
        sw a2, 0x1810(a0)
rm1:    lw t0, 0x1814(a0)
        lui t2, 0x0800
        and t0, t0, t2
        beq t0, zero, rm1
        nop
rm2:    lw t0, 0x1810(a0)
        addiu a3, a3, -1
        sw t0, 0(t9)
        bne a3, zero, rm2
        addiu t9, t9, 4
        jr ra
        nop
# send the table at a1: records of a count and that many GP0 words, each record once the GPU is idle, to a count of 0
send:   lw t1, 0(a1)
        addiu a1, a1, 4
        beq t1, zero, sent
        nop
idle:   lw t0, 0x1814(a0)
        lui t2, 0x1400
        and t0, t0, t2
        bne t0, t2, idle
        nop
word:   lw t0, 0(a1)
        addiu a1, a1, 4
        addiu t1, t1, -1
        bne t1, zero, word
        sw t0, 0x1810(a0)
        j send
        nop
sent:   jr ra
        nop
