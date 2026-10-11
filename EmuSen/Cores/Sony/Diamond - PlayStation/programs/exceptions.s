# What CAUSE holds after each kind of exception, CE (bits 28-29) above all, which the reading of BlackMoon_R3000.md
# §5.4 takes from the opcode's low two bits; that section's open item.
# Runs with no kernel and with one: installs its own vector, the I-cache off. Results from 80100000h, one CAUSE
# word per exception in the order below; 800FFFF0h holds D1A0D00Eh when done, 800FFFF4h the count.
        j start
        nop
vector: j handler
        nop
start:  mtc0 zero, SR
        lui t0, 0xFFFE
        lui t1, 0x0001
        ori t1, t1, 0xE188
        sw t1, 0x130(t0)          # BCC: the I-cache off, so the vector is fetched as written
        lui t0, 0x8001
        lw t1, 8(t0)
        lw t2, 12(t0)
        lui t3, 0x8000
        sw t1, 0x80(t3)
        sw t2, 0x84(t3)
        lui s7, 0x8010            # the results' cursor
        lui s6, 0x8010
        syscall                   # Sys
        break                     # Bp
        lui t0, 0x7FFF
        ori t0, t0, 0xFFFF
        add t1, t0, t0            # Ov
        lui a0, 0x8010
        ori a0, a0, 0x0801
        lw t1, 0(a0)              # AdEL, opcode 23h
        lh t1, 0(a0)              # AdEL, opcode 21h
        sh t1, 0(a0)              # AdES, opcode 29h
        sw t1, 0(a0)              # AdES, opcode 2Bh
        .word 0xFC000000          # reserved, opcode 3Fh
        .word 0xEC000000          # SWC3 with CU3 clear: CpU, CE 3
        mfc2 t1, $0               # CpU, CE 2
        lwc2 $0, 0(s6)            # CpU, CE 2
        lui t0, 0x7FFF
        ori t0, t0, 0xFFFF
        addi t1, t0, 1            # Ov from ADDI, opcode 08h
        subu t9, s7, s6
        srl t9, t9, 2
        lui t0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(t0)
        sw t9, -12(t0)
done:   j done
        nop
handler: mfc0 k0, CAUSE
        nop
        sw k0, 0(s7)
        addiu s7, s7, 4
        mfc0 k0, EPC
        nop
        addiu k0, k0, 4
        jr k0
        rfe
