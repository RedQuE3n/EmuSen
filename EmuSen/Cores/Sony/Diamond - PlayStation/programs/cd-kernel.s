# Calls the kernel's CD functions as a game started by the BIOS may (psx-spx "BIOS CDROM Functions"), the player's
# BIOS taken as a black box: A(71h) _96_init, A(A6h) CdGetStatus, A(A5h) CdReadSector(1, 16, buffer), A(A4h)
# CdGetLbn("\SYSTEM.CNF;1") (Diamond_CDROM.md §9). Before each call its number is written at 80100000h, after it its
# result at 80100004h + 4n; the sector read lands at 80110000h; 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  lui s1, 0x8010
        lui a3, 0x8010
        ori t0, zero, 92
        sb t0, 256(a3)
        ori t0, zero, 83
        sb t0, 257(a3)
        ori t0, zero, 89
        sb t0, 258(a3)
        ori t0, zero, 83
        sb t0, 259(a3)
        ori t0, zero, 84
        sb t0, 260(a3)
        ori t0, zero, 69
        sb t0, 261(a3)
        ori t0, zero, 77
        sb t0, 262(a3)
        ori t0, zero, 46
        sb t0, 263(a3)
        ori t0, zero, 67
        sb t0, 264(a3)
        ori t0, zero, 78
        sb t0, 265(a3)
        ori t0, zero, 70
        sb t0, 266(a3)
        ori t0, zero, 59
        sb t0, 267(a3)
        ori t0, zero, 49
        sb t0, 268(a3)
        ori t0, zero, 0
        sb t0, 269(a3)
        ori t0, zero, 1
        sw t0, 0(s1)
        jal a0call
        ori t1, zero, 0x71            # _96_init
        sw v0, 4(s1)
        ori t0, zero, 2
        sw t0, 0(s1)
        jal a0call
        ori t1, zero, 0xA6            # CdGetStatus
        sw v0, 8(s1)
        ori t0, zero, 3
        sw t0, 0(s1)
        ori a0, zero, 1
        ori a1, zero, 16
        lui a2, 0x8011
        jal a0call
        ori t1, zero, 0xA5            # CdReadSector(1, 16, 80110000h)
        sw v0, 12(s1)
        ori t0, zero, 4
        sw t0, 0(s1)
        lui a0, 0x8010
        ori a0, a0, 0x0100
        jal a0call
        ori t1, zero, 0xA4            # CdGetLbn
        sw v0, 16(s1)
        ori t0, zero, 5
        sw t0, 0(s1)
        lui a0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(a0)
done:   j done
        nop
# Calls A0h function t1 with the arguments in a0-a2.
a0call: addiu sp, sp, -8
        sw ra, 4(sp)
        ori t2, zero, 0xA0
        jalr t2
        nop
        lw ra, 4(sp)
        nop
        jr ra
        addiu sp, sp, 8
