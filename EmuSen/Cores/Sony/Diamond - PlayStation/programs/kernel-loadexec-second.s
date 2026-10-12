# The program `kernel-loadexec.s` starts by LoadExec, loaded over it (Diamond_OpenKernel_Phase2.md §26). At 80010008h,
# where the first program's hook was, it marks 8018010Ch, acknowledges what is pending and returns from the exception.
# At its entry: I_MASK at 80180120h, I_STAT at 80180124h and the first hook's count at 80180128h; then SetIrqAutoAck
# for vblank, vblank enabled for about three frames and 8018010Ch copied to 8018012Ch. 800FFFF0h holds D1A0D00Eh when
# done.
        j main
        nop
marker: lui t0, 0x8018
        ori t1, zero, 1
        sw t1, 0x10C(t0)
        lui t2, 0x1F80
        lw t3, 0x1070(t2)
        lw t4, 0x1074(t2)
        nop
        and t3, t3, t4
        nor t3, t3, zero
        sw t3, 0x1070(t2)
        ori t1, zero, 0x17
        ori t2, zero, 0xB0
        jr t2
        nop
main:   lui t0, 0x8018
        lui t2, 0x1F80
        lw t3, 0x1074(t2)
        nop
        sw t3, 0x120(t0)
        lw t3, 0x1070(t2)
        nop
        sw t3, 0x124(t0)
        lw t3, 0x100(t0)
        nop
        sw t3, 0x128(t0)
        addu a0, zero, zero
        ori a1, zero, 1
        ori t1, zero, 0x0D            # C(0Dh) SetIrqAutoAck(0, 1)
        ori t2, zero, 0xC0
        jalr t2
        nop
        lui t0, 0x8018
        lui t2, 0x1F80
        ori t3, zero, 1
        sw t3, 0x1074(t2)
        lui t4, 0x0008
spin:   addiu t4, t4, -1
        bne t4, zero, spin
        nop
        sw zero, 0x1074(t2)
        lw t3, 0x10C(t0)
        nop
        sw t3, 0x12C(t0)
        lui t5, 0x800F
        ori t5, t5, 0xFFF0
        lui t6, 0xD1A0
        ori t6, t6, 0xD00E
        sw t6, 0(t5)
done:   j done
        nop
