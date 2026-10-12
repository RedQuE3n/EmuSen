# One _card_write of sector 3Fh on the port 80100100h names, after InitCARD2, StartCARD2 and _new_card, with
# interrupts on, so that the test can raise vblank at any instruction of the call (Diamond_OpenKernel_Phase2.md §23):
# the 128 bytes at 80120000h are 3n+1; _card_wait's status lands at 80100004h; 800FFFF0h holds D1A0D00Eh when done.
        lui t0, 0x8012
        addu t1, zero, zero
fill:   addu t2, t1, t1
        addu t2, t2, t1
        addiu t2, t2, 1
        addu t3, t0, t1
        sb t2, 0(t3)
        addiu t1, t1, 1
        ori t4, zero, 128
        bne t1, t4, fill
        nop
        addu a0, zero, zero
        jal kcall
        ori t1, zero, 0x4A           # InitCARD2(0)
        jal kcall
        ori t1, zero, 0x4B           # StartCARD2
        jal kcall
        ori t1, zero, 0x50           # _new_card
        lui t7, 0x8010
        lw a0, 0x100(t7)
        ori a1, zero, 0x3F
        lui a2, 0x8012
        jal kcall
        ori t1, zero, 0x4E           # _card_write(port, 3Fh, 80120000h)
        lui t7, 0x8010
        sw v0, 0(t7)
        lw a0, 0x100(t7)
        nop
        srl a0, a0, 4
        jal kcall
        ori t1, zero, 0x5D           # _card_wait(slot)
        lui t7, 0x8010
        sw v0, 4(t7)
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(t7)
done:   j done
        nop
# B0h function t1 with a0-a2.
kcall:  addiu sp, sp, -32
        sw ra, 28(sp)
        ori t2, zero, 0xB0
        jalr t2
        nop
        lw ra, 28(sp)
        nop
        jr ra
        addiu sp, sp, 32
