# Booted by the open kernel from a synthetic disc's SYSTEM.CNF (BOOT = cdrom:\DIR\PROG.EXE;1 hello, TCB = 6,
# EVENT = 14, STACK = 801FF800): its entry state, then the kernel's CD and file calls on the disc, each made with
# S0-S7, GP and FP set and checked after it (Diamond_OpenKernel_Phase2.md §17). Results from 80100000h, a word each:
# a0, a1, sp, gp, [180h], GetConf's three, the calls that changed a saved register, then each call's v0 in order;
# DATA.BIN's bytes land at 80110000h and 80112000h, sector 16 at 80118000h, direntries at 80100200h; 800FFFF0h holds
# D1A0D00Eh when done.
        lui t7, 0x8010
        sw a0, 0(t7)
        sw a1, 4(t7)
        sw sp, 8(t7)
        sw gp, 12(t7)
        lui t0, 0x8000
        lw t1, 0x180(t0)
        nop
        sw t1, 16(t7)
        sw zero, 32(t7)
        jal strings_end
        nop
        .word 0x6F726463   # "cdrom:\DATA.BIN;1"
        .word 0x445C3A6D
        .word 0x2E415441
        .word 0x3B4E4942
        .word 0x00000031
        .word 0x5249445C   # "\DIR\PROG.EXE;1"
        .word 0x4F52505C
        .word 0x58452E47
        .word 0x00313B45
        .word 0x6F726463   # "cdrom:\*"
        .word 0x2A5C3A6D
        .word 0x00000000
        .word 0x6F726463   # "cdrom:\DIR"
        .word 0x445C3A6D
        .word 0x00005249
        .word 0x6F726463   # "cdrom:PROG.EXE;1"
        .word 0x52503A6D
        .word 0x452E474F
        .word 0x313B4558
        .word 0x00000000
        .word 0x6F726463   # "cdrom:\NONE.BIN;1"
        .word 0x4E5C3A6D
        .word 0x2E454E4F
        .word 0x3B4E4942
        .word 0x00000031
strings_end:
        lui t7, 0x8010
        sw ra, 0x300(t7)
        lui a0, 0x8010
        ori a0, a0, 20
        lui a1, 0x8010
        ori a1, a1, 24
        lui a2, 0x8010
        ori a2, a2, 28
        ori t1, zero, 0x9D
        jal kcall
        ori t2, zero, 0xA0           # GetConf
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 0
        lui a1, 0x0000
        ori a1, a1, 0x0001
        ori t1, zero, 0x32
        jal kcall
        ori t2, zero, 0xB0           # open, result 36
        lui t7, 0x8010
        sw v0, 36(t7)
        lui t7, 0x8010
        lw s0, 36(t7)
        nop
        sw s0, 0x304(t7)
        lui t7, 0x8010
        lw a0, 0x304(t7)
        nop
        lui a1, 0x0000
        ori a1, a1, 0x0064
        lui a2, 0x0000
        ori a2, a2, 0x0000
        ori t1, zero, 0x33
        jal kcall
        ori t2, zero, 0xB0           # lseek(fd, 100, 0), result 40
        lui t7, 0x8010
        sw v0, 40(t7)
        lui t7, 0x8010
        lw a0, 0x304(t7)
        nop
        lui a1, 0x8011
        ori a1, a1, 0x0000
        lui a2, 0x0000
        ori a2, a2, 0x0FA0
        ori t1, zero, 0x34
        jal kcall
        ori t2, zero, 0xB0           # read 4000, result 44
        lui t7, 0x8010
        sw v0, 44(t7)
        lui t7, 0x8010
        lw a0, 0x304(t7)
        nop
        lui a1, 0x8011
        ori a1, a1, 0x2000
        lui a2, 0x0000
        ori a2, a2, 0x07D0
        ori t1, zero, 0x34
        jal kcall
        ori t2, zero, 0xB0           # read 2000: the 900 left, result 48
        lui t7, 0x8010
        sw v0, 48(t7)
        lui t7, 0x8010
        lw a0, 0x304(t7)
        nop
        lui a1, 0x8011
        ori a1, a1, 0x4000
        lui a2, 0x0000
        ori a2, a2, 0x07D0
        ori t1, zero, 0x34
        jal kcall
        ori t2, zero, 0xB0           # read past the end: -1, result 52
        lui t7, 0x8010
        sw v0, 52(t7)
        lui t7, 0x8010
        lw a0, 0x304(t7)
        nop
        ori t1, zero, 0x36
        jal kcall
        ori t2, zero, 0xB0           # close, result 56
        lui t7, 0x8010
        sw v0, 56(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 20
        ori t1, zero, 0xA4
        jal kcall
        ori t2, zero, 0xA0           # CdGetLbn, result 60
        lui t7, 0x8010
        sw v0, 60(t7)
        lui a0, 0x0000
        ori a0, a0, 0x0001
        lui a1, 0x0000
        ori a1, a1, 0x0010
        lui a2, 0x8011
        ori a2, a2, 0x8000
        ori t1, zero, 0xA5
        jal kcall
        ori t2, zero, 0xA0           # CdReadSector(1, 16, ...), result 64
        lui t7, 0x8010
        sw v0, 64(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 36
        lui a1, 0x8010
        ori a1, a1, 0x0200
        ori t1, zero, 0x42
        jal kcall
        ori t2, zero, 0xB0           # firstfile2, result 68
        lui t7, 0x8010
        sw v0, 68(t7)
        lui a0, 0x8010
        ori a0, a0, 0x0228
        ori t1, zero, 0x43
        jal kcall
        ori t2, zero, 0xB0           # nextfile, result 72
        lui t7, 0x8010
        sw v0, 72(t7)
        lui a0, 0x8010
        ori a0, a0, 0x0250
        ori t1, zero, 0x43
        jal kcall
        ori t2, zero, 0xB0           # nextfile, result 76
        lui t7, 0x8010
        sw v0, 76(t7)
        lui a0, 0x8010
        ori a0, a0, 0x0278
        ori t1, zero, 0x43
        jal kcall
        ori t2, zero, 0xB0           # nextfile: none left, result 80
        lui t7, 0x8010
        sw v0, 80(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 48
        ori t1, zero, 0x40
        jal kcall
        ori t2, zero, 0xB0           # cd, result 84
        lui t7, 0x8010
        sw v0, 84(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 60
        lui a1, 0x0000
        ori a1, a1, 0x0001
        ori t1, zero, 0x32
        jal kcall
        ori t2, zero, 0xB0           # open relative to the directory, result 88
        lui t7, 0x8010
        sw v0, 88(t7)
        lui t7, 0x8010
        lw s0, 88(t7)
        nop
        sw s0, 0x304(t7)
        lui t7, 0x8010
        lw a0, 0x304(t7)
        nop
        ori t1, zero, 0x36
        jal kcall
        ori t2, zero, 0xB0           # close, result 92
        lui t7, 0x8010
        sw v0, 92(t7)
        lui t7, 0x8010
        lw a0, 0x300(t7)
        nop
        addiu a0, a0, 80
        lui a1, 0x0000
        ori a1, a1, 0x0001
        ori t1, zero, 0x32
        jal kcall
        ori t2, zero, 0xB0           # open a file not there, result 96
        lui t7, 0x8010
        sw v0, 96(t7)
        ori t1, zero, 0x54
        jal kcall
        ori t2, zero, 0xB0           # _get_errno, result 100
        lui t7, 0x8010
        sw v0, 100(t7)
        ori t1, zero, 0xA6
        jal kcall
        ori t2, zero, 0xA0           # CdGetStatus, result 104
        lui t7, 0x8010
        sw v0, 104(t7)
        lui t0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(t0)
done:   j done
        nop
# The kernel's function t1 in the table t2 with a0-a3, S0-S7, GP and FP set first and their XOR and SP checked after;
# a mismatch counted at 80100020h.
kcall:  lui t7, 0x8010
        sw ra, 0x3F0(t7)
        sw sp, 0x3F4(t7)
        lui s0, 0x5100
        ori s1, s0, 1
        ori s2, s0, 2
        ori s3, s0, 3
        ori s4, s0, 4
        ori s5, s0, 5
        ori s6, s0, 6
        ori s7, s0, 7
        lui gp, 0x8012
        ori gp, gp, 0x3450
        lui fp, 0x3030
        addiu sp, sp, -32
        jalr t2
        nop
        addiu sp, sp, 32
        lui t7, 0x8010
        sw v0, 0x3F8(t7)
        xor t3, s0, s1
        xor t3, t3, s2
        xor t3, t3, s3
        xor t3, t3, s4
        xor t3, t3, s5
        xor t3, t3, s6
        xor t3, t3, s7
        xor t3, t3, gp
        xor t3, t3, fp
        lw t4, 0x3F4(t7)
        lui t5, 0xB022
        ori t5, t5, 0x3450
        bne sp, t4, bad
        nop
        beq t3, t5, good
        nop
bad:    lw t6, 32(t7)
        nop
        addiu t6, t6, 1
        sw t6, 32(t7)
good:   lw ra, 0x3F0(t7)
        lw v0, 0x3F8(t7)
        jr ra
        nop
