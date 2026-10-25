# The output channel's control (Sapphire_Native.md §14): a bare EE program that writes the line below a byte at a time to
# KPUTCHAR, 1000F180h (ps2tek's "Memory Map", "Console output"), through KSEG1, and then 5A99D00Eh to 001FFFF0h.
#     EMUSEN KPUTCHAR CONTROL
        lui a1, 0xB001            # KSEG1, uncached: B0010000h - 0E80h is KPUTCHAR, B000F180h
        addiu t0, zero, 0x45     # 'E'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x4D     # 'M'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x55     # 'U'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x53     # 'S'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x45     # 'E'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x4E     # 'N'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x20     # space
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x4B     # 'K'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x50     # 'P'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x55     # 'U'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x54     # 'T'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x43     # 'C'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x48     # 'H'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x41     # 'A'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x52     # 'R'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x20     # space
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x43     # 'C'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x4F     # 'O'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x4E     # 'N'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x54     # 'T'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x52     # 'R'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x4F     # 'O'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x4C     # 'L'
        sb t0, -0x0E80(a1)        # B000F180h
        addiu t0, zero, 0x0A     # line feed
        sb t0, -0x0E80(a1)        # B000F180h
        lui a0, 0x0020
        lui t0, 0x5A99
        ori t0, t0, 0xD00E
        sw t0, -16(a0)            # done: 001FFFF0h
end:    beq zero, zero, end
        nop
