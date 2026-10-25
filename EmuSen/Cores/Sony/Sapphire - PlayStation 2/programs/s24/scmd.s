# The S commands the player's BIOS was seen to send, with its parameters (Sapphire_Native.md §35.3, D-328): a table at
# `cmds` of entries of five words, the command, the parameter count and up to three words holding the parameter bytes
# low first; each sent after the result FIFO is drained, then the S status, the count of result bytes (at most 16)
# and the bytes read until the status says none is left. 20 words an entry from 00200000h, then the done word.
        lui a0, 0x0020
        lui s0, 0xBF40
        ori s0, s0, 0x2000
        bgezal zero, past
        nop
cmds:   .word 0x1a
        .word 4
        .word 0x43411e02
        .word 0
        .word 0
        .word 0x22
        .word 0
        .word 0x0
        .word 0
        .word 0
        .word 0x24
        .word 1
        .word 0x1
        .word 0
        .word 0
        .word 0x17
        .word 1
        .word 0x8
        .word 0
        .word 0
        .word 0x1e
        .word 0
        .word 0x0
        .word 0
        .word 0
        .word 0x36
        .word 0
        .word 0x0
        .word 0
        .word 0
        .word 0xff
        .word 0
        .word 0x0
        .word 0
        .word 0
past:   addu s1, ra, zero
next:   lw s2, 0(s1)
        lw s3, 4(s1)
        addiu t0, zero, 0xFF
        beq s2, t0, done
        nop
        addiu t6, zero, 64
drain:  lbu t0, 23(s0)
        nop
        andi t0, t0, 0x40
        bne t0, zero, send
        nop
        lbu t0, 24(s0)
        addiu t6, t6, -1
        bne t6, zero, drain
        nop
send:   addiu t1, s1, 8
        addiu t2, zero, 0
param:  beq t2, s3, cmd
        nop
        addu t3, t1, t2
        lbu t4, 0(t3)
        nop
        sb t4, 23(s0)
        addiu t2, t2, 1
        beq zero, zero, param
        nop
cmd:    sb s2, 22(s0)
        addiu t2, zero, 20000
wait:   lbu t0, 23(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, idle
        nop
        addiu t2, t2, -1
        bne t2, zero, wait
        nop
idle:   lbu t0, 23(s0)
        nop
        sw t0, 0(a0)
        addiu t3, zero, 0
        addiu t4, a0, 8
more:   lbu t0, 23(s0)
        nop
        andi t0, t0, 0x40
        bne t0, zero, out
        nop
        lbu t0, 24(s0)
        nop
        sw t0, 0(t4)
        addiu t4, t4, 4
        addiu t3, t3, 1
        slti t5, t3, 16
        bne t5, zero, more
        nop
out:    sw t3, 4(a0)
        addiu a0, a0, 80
        addiu s1, s1, 20
        beq zero, zero, next
        nop
done:   lui a0, 0x0020
        lui t0, 0x5A99
        ori t0, t0, 0xD00E
        sw t0, -16(a0)
end:    beq zero, zero, end
        nop
