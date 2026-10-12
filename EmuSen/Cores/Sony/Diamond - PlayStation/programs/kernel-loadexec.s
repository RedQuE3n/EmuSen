# Booted by the open kernel from a synthetic disc (BOOT = cdrom:\FIRST.EXE;1), as Mega Man Legends' first program
# runs before it hands over (Diamond_OpenKernel_Phase2.md §26): an exit structure at 80180000h whose RA is this
# program's hook at 80010008h, vblank enabled, then _96_init and CdReadSector with the hook recording I_MASK, then
# LoadExec of SECOND.EXE over this program. The hook counts itself at 80180100h, ORs the I_MASK it finds into
# 80180104h while 80180110h is set, acknowledges what is pending and returns from the exception. Results: 80180108h
# the count at LoadExec, 80180130h CdReadSector's v0, 80180134h _96_init's, sector 16 at 80190000h.
        j main
        nop
hook:   lui t0, 0x8018
        lw t1, 0x100(t0)
        nop
        addiu t1, t1, 1
        sw t1, 0x100(t0)
        lui t2, 0x1F80
        lw t3, 0x1074(t2)
        lw t4, 0x110(t0)
        nop
        beq t4, zero, ack
        nop
        lw t5, 0x104(t0)
        nop
        or t5, t5, t3
        sw t5, 0x104(t0)
ack:    lw t3, 0x1070(t2)
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
        lui t1, 0x8001
        ori t1, t1, 0x0008
        sw t1, 0(t0)
        lui t1, 0x801F
        ori t1, t1, 0xE000
        sw t1, 4(t0)
        ori t2, zero, 8
clear:  addu t3, t0, t2
        sw zero, 0(t3)
        addiu t2, t2, 4
        slti t4, t2, 0x140
        bne t4, zero, clear
        nop
        lui a0, 0x8018
        ori t1, zero, 0x19            # B(19h) HookEntryInt
        ori t2, zero, 0xB0
        jalr t2
        nop
        lui t0, 0x8018
        lui t2, 0x1F80
        ori t3, zero, 1
        sw t3, 0x1074(t2)             # vblank enabled
wait:   lw t1, 0x100(t0)
        nop
        slti t2, t1, 4
        bne t2, zero, wait
        nop
        ori t1, zero, 1
        sw t1, 0x110(t0)              # the hook records I_MASK from here
        ori t1, zero, 0x71            # A(71h) _96_init
        ori t2, zero, 0xA0
        jalr t2
        nop
        lui t0, 0x8018
        sw v0, 0x134(t0)
        ori a0, zero, 4
        ori a1, zero, 16
        lui a2, 0x8019
        ori t1, zero, 0xA5            # A(A5h) CdReadSector
        ori t2, zero, 0xA0
        jalr t2
        nop
        lui t0, 0x8018
        sw v0, 0x130(t0)
        sw zero, 0x110(t0)
        lw t1, 0x100(t0)
        nop
        sw t1, 0x108(t0)
        jal name_end
        nop
        .word 0x6F726463              # "cdrom:\SECOND.EXE;1"
        .word 0x535C3A6D
        .word 0x4E4F4345
        .word 0x58452E44
        .word 0x00313B45
name_end:
        addu a0, ra, zero
        lui a1, 0x801F
        ori a1, a1, 0xF000
        addu a2, zero, zero
        ori t1, zero, 0x51            # A(51h) LoadExec
        ori t2, zero, 0xA0
        jalr t2
        nop
lost:   j lost
        nop
