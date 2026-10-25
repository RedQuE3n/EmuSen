# Each S command 00h..FFh but 0Fh (which ends PCSX2's run) sent with no parameters: the S status after it, the result bytes read until the status says
# no data (at most 16), and the drive status; 20 words a command from 00200000h, then the done word.
        lui a0, 0x0020
        lui s0, 0xBF40
        ori s0, s0, 0x2000
        addiu s1, zero, 0
next:   addiu t6, zero, 64
drain:  lbu t0, 23(s0)
        nop
        andi t0, t0, 0x40
        bne t0, zero, clean
        nop
        lbu t0, 24(s0)
        addiu t6, t6, -1
        bne t6, zero, drain
        nop
clean:  addiu t0, zero, 15
        beq s1, t0, out
        addiu t3, zero, 255
        sb s1, 22(s0)
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
        lbu t0, 10(s0)
        nop
        sw t0, 8(a0)
        addiu t3, zero, 0
        addiu t4, a0, 12
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
        addiu s1, s1, 1
        slti t5, s1, 256
        bne t5, zero, next
        nop
        lui a0, 0x0020
        lui t0, 0x5A99
        ori t0, t0, 0xD00E
        sw t0, -16(a0)
end:    beq zero, zero, end
        nop
