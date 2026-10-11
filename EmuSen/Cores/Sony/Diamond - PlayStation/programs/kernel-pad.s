# The open kernel's pad functions reading SIO0 (Diamond_OpenKernel.md §4.13), run on the kernel: InitPAD2 with buffers
# at 80100000h and 80100040h of 22h bytes, StartPAD2, vblank enabled in I_MASK, ExitCriticalSection; each vblank then
# fills both buffers. 800FFFF0h holds D1A0D00Eh once the pads are started.
        j start
        nop
start:  lui a0, 0x8010
        ori a1, zero, 0x22
        lui a2, 0x8010
        ori a2, a2, 0x40
        ori a3, zero, 0x22
        ori t1, zero, 0x12            # B(12h) InitPAD2
        jal 0x800000b0
        nop
        ori t1, zero, 0x13            # B(13h) StartPAD2
        jal 0x800000b0
        nop
        lui t0, 0x1F80
        ori t2, zero, 1
        sw t2, 0x1074(t0)             # I_MASK: vblank
        ori a0, zero, 2
        syscall                       # ExitCriticalSection
        lui a0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(a0)
done:   j done
        nop
