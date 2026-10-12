# The open kernel's memory card calls on a blank card in slot 1 and an empty slot 2 (Diamond_OpenKernel_Phase2.md
# §17): InitCARD2, StartCARD2 and _bu_init, then format, two files made, written, read back, listed, erased,
# undeleted, erased again and renamed, an asynchronous read and a low-level read each ended by its event, and the
# empty slot's timeout. Each call's v0 from 80100000h, a word each; the 16 KiB written comes from 80120000h, is read
# back to 80130000h, its first frame asynchronously to 80140000h, sector 0 by _card_read to 80150000h; direntries at
# 80100200h. 800FFFF0h holds D1A0D00Eh when done.
        jal strings_end
        nop
        .word 0x30307562   # "bu00:"
        .word 0x0000003A
        .word 0x30307562   # "bu00:BASLUS-00000DIAMOND"
        .word 0x5341423A
        .word 0x2D53554C
        .word 0x30303030
        .word 0x41494430
        .word 0x444E4F4D
        .word 0x00000000
        .word 0x30307562   # "bu00:BASLUS-00000SECOND"
        .word 0x5341423A
        .word 0x2D53554C
        .word 0x30303030
        .word 0x43455330
        .word 0x00444E4F
        .word 0x30307562   # "bu00:BASLUS*"
        .word 0x5341423A
        .word 0x2A53554C
        .word 0x00000000
        .word 0x30307562   # "bu00:*"
        .word 0x00002A3A
        .word 0x30307562   # "bu00:BASLUS-00000RENAMED"
        .word 0x5341423A
        .word 0x2D53554C
        .word 0x30303030
        .word 0x4E455230
        .word 0x44454D41
        .word 0x00000000
        .word 0x30317562   # "bu10:X"
        .word 0x0000583A
strings_end:
        lui t7, 0x8010
        sw ra, 0x300(t7)
# The pattern: word k of 16 KiB is k XOR A5000000h.
        lui t0, 0x8012
        lui t2, 0xA500
        addu t1, zero, zero
fill:   xor t3, t1, t2
        sw t3, 0(t0)
        addiu t0, t0, 4
        addiu t1, t1, 1
        ori t4, zero, 0x1000
        bne t1, t4, fill
        nop
        lui a0, 0x0000
        ori a0, a0, 0x0000
        ori t1, zero, 0x4A
        jal kcall
        ori t2, zero, 0xB0           # InitCARD2(0), result 0
        lui t7, 0x8010
        sw v0, 0(t7)
        ori t1, zero, 0x4B
        jal kcall
        ori t2, zero, 0xB0           # StartCARD2, result 4
        lui t7, 0x8010
        sw v0, 4(t7)
        ori t1, zero, 0x55
        jal kcall
        ori t2, zero, 0xA0           # _bu_init, result 8
        lui t7, 0x8010
        sw v0, 8(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 0
        ori t1, zero, 0x41
        jal kcall
        ori t2, zero, 0xB0           # format, result 12
        lui t7, 0x8010
        sw v0, 12(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 8
        lui a1, 0x0002
        ori a1, a1, 0x0203
        ori t1, zero, 0x32
        jal kcall
        ori t2, zero, 0xB0           # open, a new file of 2 blocks, result 16
        lui t7, 0x8010
        sw v0, 16(t7)
        lui t7, 0x8010
        lw a0, 16(t7)
        nop
        lui a1, 0x8012
        ori a1, a1, 0x0000
        lui a2, 0x0000
        ori a2, a2, 0x4000
        ori t1, zero, 0x35
        jal kcall
        ori t2, zero, 0xB0           # write 16 KiB, result 20
        lui t7, 0x8010
        sw v0, 20(t7)
        lui t7, 0x8010
        lw a0, 16(t7)
        nop
        ori t1, zero, 0x36
        jal kcall
        ori t2, zero, 0xB0           # close, result 24
        lui t7, 0x8010
        sw v0, 24(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 8
        lui a1, 0x0000
        ori a1, a1, 0x0001
        ori t1, zero, 0x32
        jal kcall
        ori t2, zero, 0xB0           # open it again, result 28
        lui t7, 0x8010
        sw v0, 28(t7)
        lui t7, 0x8010
        lw a0, 28(t7)
        nop
        lui a1, 0x8013
        ori a1, a1, 0x0000
        lui a2, 0x0000
        ori a2, a2, 0x4000
        ori t1, zero, 0x34
        jal kcall
        ori t2, zero, 0xB0           # read 16 KiB, result 32
        lui t7, 0x8010
        sw v0, 32(t7)
        lui t7, 0x8010
        lw a0, 28(t7)
        nop
        ori t1, zero, 0x36
        jal kcall
        ori t2, zero, 0xB0           # close, result 36
        lui t7, 0x8010
        sw v0, 36(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 36
        lui a1, 0x0001
        ori a1, a1, 0x0203
        ori t1, zero, 0x32
        jal kcall
        ori t2, zero, 0xB0           # open, a new file of 1 block, result 40
        lui t7, 0x8010
        sw v0, 40(t7)
        lui t7, 0x8010
        lw a0, 40(t7)
        nop
        ori t1, zero, 0x36
        jal kcall
        ori t2, zero, 0xB0           # close, result 44
        lui t7, 0x8010
        sw v0, 44(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 60
        lui a1, 0x8010
        ori a1, a1, 0x0200
        ori t1, zero, 0x42
        jal kcall
        ori t2, zero, 0xB0           # firstfile2, result 48
        lui t7, 0x8010
        sw v0, 48(t7)
        lui a0, 0x8010
        ori a0, a0, 0x0228
        ori t1, zero, 0x43
        jal kcall
        ori t2, zero, 0xB0           # nextfile, result 52
        lui t7, 0x8010
        sw v0, 52(t7)
        lui a0, 0x8010
        ori a0, a0, 0x0250
        ori t1, zero, 0x43
        jal kcall
        ori t2, zero, 0xB0           # nextfile: none left, result 56
        lui t7, 0x8010
        sw v0, 56(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 36
        ori t1, zero, 0x45
        jal kcall
        ori t2, zero, 0xB0           # erase, result 60
        lui t7, 0x8010
        sw v0, 60(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 36
        lui a1, 0x0000
        ori a1, a1, 0x0001
        ori t1, zero, 0x32
        jal kcall
        ori t2, zero, 0xB0           # open the erased file: -1, result 64
        lui t7, 0x8010
        sw v0, 64(t7)
        ori t1, zero, 0x54
        jal kcall
        ori t2, zero, 0xB0           # _get_errno, result 68
        lui t7, 0x8010
        sw v0, 68(t7)
        lui a0, 0x0000
        ori a0, a0, 0x0001
        ori t1, zero, 0x1A
        jal kcall
        ori t2, zero, 0xC0           # set_card_find_mode(1), result 72
        lui t7, 0x8010
        sw v0, 72(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 76
        lui a1, 0x8010
        ori a1, a1, 0x0278
        ori t1, zero, 0x42
        jal kcall
        ori t2, zero, 0xB0           # firstfile2 of the deleted, result 76
        lui t7, 0x8010
        sw v0, 76(t7)
        lui a0, 0x0000
        ori a0, a0, 0x0000
        ori t1, zero, 0x1A
        jal kcall
        ori t2, zero, 0xC0           # set_card_find_mode(0), result 80
        lui t7, 0x8010
        sw v0, 80(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 36
        ori t1, zero, 0x46
        jal kcall
        ori t2, zero, 0xB0           # undelete, result 84
        lui t7, 0x8010
        sw v0, 84(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 36
        ori t1, zero, 0x45
        jal kcall
        ori t2, zero, 0xB0           # erase again, result 88
        lui t7, 0x8010
        sw v0, 88(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 8
        lui t7, 0x8010
        lw a1, 0x300(t7)
        nop
        addiu a1, a1, 84
        ori t1, zero, 0x44
        jal kcall
        ori t2, zero, 0xB0           # rename, result 92
        lui t7, 0x8010
        sw v0, 92(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 8
        lui a1, 0x0000
        ori a1, a1, 0x0001
        ori t1, zero, 0x32
        jal kcall
        ori t2, zero, 0xB0           # open the old name: -1, result 96
        lui t7, 0x8010
        sw v0, 96(t7)
        lui a0, 0xF400
        ori a0, a0, 0x0001
        lui a1, 0x0000
        ori a1, a1, 0x0004
        lui a2, 0x0000
        ori a2, a2, 0x2000
        lui a3, 0x0000
        ori a3, a3, 0x0000
        ori t1, zero, 0x08
        jal kcall
        ori t2, zero, 0xB0           # OpenEvent(F4000001h, 4), result 100
        lui t7, 0x8010
        sw v0, 100(t7)
        lui t7, 0x8010
        lw a0, 100(t7)
        nop
        ori t1, zero, 0x0C
        jal kcall
        ori t2, zero, 0xB0           # EnableEvent, result 104
        lui t7, 0x8010
        sw v0, 104(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 84
        lui a1, 0x0000
        ori a1, a1, 0x8001
        ori t1, zero, 0x32
        jal kcall
        ori t2, zero, 0xB0           # open, asynchronous, result 108
        lui t7, 0x8010
        sw v0, 108(t7)
        lui t7, 0x8010
        lw a0, 108(t7)
        nop
        lui a1, 0x8014
        ori a1, a1, 0x0000
        lui a2, 0x0000
        ori a2, a2, 0x0080
        ori t1, zero, 0x34
        jal kcall
        ori t2, zero, 0xB0           # read a frame, asynchronous, result 112
        lui t7, 0x8010
        sw v0, 112(t7)
        lui t7, 0x8010
        lui t8, 0x0006
        sw t8, 116(t7)
w116:    lui t7, 0x8010
        lw a0, 100(t7)
        ori t1, zero, 0x0B
        jal kcall
        ori t2, zero, 0xB0           # TestEvent until the read's event, result 116: 1, or 0 on the timeout
        lui t7, 0x8010
        bne v0, zero, x116
        nop
        lw t8, 116(t7)
        nop
        addiu t8, t8, -1
        sw t8, 116(t7)
        bne t8, zero, w116
        nop
x116:   sw v0, 116(t7)
        lui t7, 0x8010
        lw a0, 108(t7)
        nop
        ori t1, zero, 0x36
        jal kcall
        ori t2, zero, 0xB0           # close, result 120
        lui t7, 0x8010
        sw v0, 120(t7)
        lui a0, 0xF000
        ori a0, a0, 0x0011
        lui a1, 0x0000
        ori a1, a1, 0x0004
        lui a2, 0x0000
        ori a2, a2, 0x2000
        lui a3, 0x0000
        ori a3, a3, 0x0000
        ori t1, zero, 0x08
        jal kcall
        ori t2, zero, 0xB0           # OpenEvent(F0000011h, 4), result 124
        lui t7, 0x8010
        sw v0, 124(t7)
        lui t7, 0x8010
        lw a0, 124(t7)
        nop
        ori t1, zero, 0x0C
        jal kcall
        ori t2, zero, 0xB0           # EnableEvent, result 128
        lui t7, 0x8010
        sw v0, 128(t7)
        lui a0, 0x0000
        ori a0, a0, 0x0000
        lui a1, 0x0000
        ori a1, a1, 0x0000
        lui a2, 0x8015
        ori a2, a2, 0x0000
        ori t1, zero, 0x4F
        jal kcall
        ori t2, zero, 0xB0           # _card_read(0, 0), result 132
        lui t7, 0x8010
        sw v0, 132(t7)
        lui t7, 0x8010
        lui t8, 0x0006
        sw t8, 136(t7)
w136:    lui t7, 0x8010
        lw a0, 124(t7)
        ori t1, zero, 0x0B
        jal kcall
        ori t2, zero, 0xB0           # TestEvent until its event, result 136: 1, or 0 on the timeout
        lui t7, 0x8010
        bne v0, zero, x136
        nop
        lw t8, 136(t7)
        nop
        addiu t8, t8, -1
        sw t8, 136(t7)
        bne t8, zero, w136
        nop
x136:   sw v0, 136(t7)
        lui a0, 0x0000
        ori a0, a0, 0x0000
        ori t1, zero, 0x5C
        jal kcall
        ori t2, zero, 0xB0           # _card_status(0), result 140
        lui t7, 0x8010
        sw v0, 140(t7)
        lui a0, 0xF000
        ori a0, a0, 0x0011
        lui a1, 0x0000
        ori a1, a1, 0x0100
        lui a2, 0x0000
        ori a2, a2, 0x2000
        lui a3, 0x0000
        ori a3, a3, 0x0000
        ori t1, zero, 0x08
        jal kcall
        ori t2, zero, 0xB0           # OpenEvent(F0000011h, 100h), result 144
        lui t7, 0x8010
        sw v0, 144(t7)
        lui t7, 0x8010
        lw a0, 144(t7)
        nop
        ori t1, zero, 0x0C
        jal kcall
        ori t2, zero, 0xB0           # EnableEvent, result 148
        lui t7, 0x8010
        sw v0, 148(t7)
        lui a0, 0x0000
        ori a0, a0, 0x0010
        ori t1, zero, 0xAB
        jal kcall
        ori t2, zero, 0xA0           # _card_info(10h), slot 2 empty, result 152
        lui t7, 0x8010
        sw v0, 152(t7)
        lui t7, 0x8010
        lui t8, 0x0006
        sw t8, 156(t7)
w156:    lui t7, 0x8010
        lw a0, 144(t7)
        ori t1, zero, 0x0B
        jal kcall
        ori t2, zero, 0xB0           # TestEvent until its timeout event, result 156: 1, or 0 on the timeout
        lui t7, 0x8010
        bne v0, zero, x156
        nop
        lw t8, 156(t7)
        nop
        addiu t8, t8, -1
        sw t8, 156(t7)
        bne t8, zero, w156
        nop
x156:   sw v0, 156(t7)
        lui a0, 0x0000
        ori a0, a0, 0x0001
        ori t1, zero, 0x5C
        jal kcall
        ori t2, zero, 0xB0           # _card_status(1), result 160
        lui t7, 0x8010
        sw v0, 160(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 112
        lui a1, 0x0000
        ori a1, a1, 0x0001
        ori t1, zero, 0x32
        jal kcall
        ori t2, zero, 0xB0           # open on the empty slot: -1, result 164
        lui t7, 0x8010
        sw v0, 164(t7)
        lui t0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(t0)
done:   j done
        nop
# The kernel's function t1 in the table t2 with a0-a3.
kcall:  addiu sp, sp, -32
        sw ra, 28(sp)
        jalr t2
        nop
        lw ra, 28(sp)
        nop
        jr ra
        addiu sp, sp, 32
