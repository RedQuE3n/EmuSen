# Whether the EE reaches the CDVD's registers on a referee: reads the N status, drive status, disk type and S status,
# sends S command 03h with parameter 00h and records the status and four result bytes.
        lui a0, 0x0020
        lui s0, 0xBF40
        ori s0, s0, 0x2000
        lbu t0, 5(s0)
        nop
        sw t0, 0(a0)
        lbu t0, 10(s0)
        nop
        sw t0, 4(a0)
        lbu t0, 15(s0)
        nop
        sw t0, 8(a0)
        lbu t0, 23(s0)
        nop
        sw t0, 12(a0)
        sb zero, 23(s0)
        addiu t1, zero, 3
        sb t1, 22(s0)
        addiu t2, zero, 20000
wait:   lbu t0, 23(s0)
        nop
        andi t0, t0, 0x80
        beq t0, zero, ok
        nop
        addiu t2, t2, -1
        bne t2, zero, wait
        nop
ok:     lbu t0, 23(s0)
        nop
        sw t0, 16(a0)
        lbu t0, 24(s0)
        nop
        sw t0, 20(a0)
        lbu t0, 24(s0)
        nop
        sw t0, 24(a0)
        lbu t0, 24(s0)
        nop
        sw t0, 28(a0)
        lbu t0, 24(s0)
        nop
        sw t0, 32(a0)
        lbu t0, 23(s0)
        nop
        sw t0, 36(a0)
        lui t0, 0x5A99
        ori t0, t0, 0xD00E
        sw t0, -16(a0)
end:    beq zero, zero, end
        nop
