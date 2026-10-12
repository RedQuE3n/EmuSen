# The open kernel's interrupt path under a game's load (Diamond_OpenKernel_Phase2.md §25.3): the pads started with a
# pad on port 1 and port 2 empty, timer 2 at its target every 4,097 ticks of the system clock / 8 (32,776 cycles, as
# Crash Bandicoot runs it) with its IRQ, vblank and timer 2 enabled in I_MASK, ExitCriticalSection, then a loop in user
# RAM. 800FFFF0h holds D1A0D00Eh once the loop runs.
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
        ori t2, zero, 0x1001
        sh t2, 0x1128(t0)             # timer 2's target
        ori t2, zero, 0x0258
        sh t2, 0x1124(t0)             # its mode: reset and IRQ at the target, repeated, system clock / 8
        ori t2, zero, 0x41
        sw t2, 0x1074(t0)             # I_MASK: vblank and timer 2
        ori a0, zero, 2
        syscall                       # ExitCriticalSection
        lui a0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(a0)
loop:   addiu s0, s0, 1
        j loop
        nop
