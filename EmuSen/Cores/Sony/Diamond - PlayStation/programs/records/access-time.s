# ps1-tests' cpu/access-time, rewritten to record in RAM (Diamond_Records.md §1, §3). This file is the program's own
# part only: the entry, the stubs and the start-up. The test's compiled code, readAccessTime<> for each of the 17
# regions, runTests, and io.c's read16 and write16 (80010078h-80014577h), is not kept here: tests/support/records.rs
# carries it from the local release build 158 at the @release line, word for word at the release's addresses, its
# words pinned by SHA-256, and points its calls to printf, EnterCriticalSection and ExitCriticalSection at the stubs
# below. Bare metal, interrupts off; the misaligned SPUCNT word's address error returns through the program's own
# handler. Results from 80100000h: per region in the table's order, per width (8, 16, 32 bits), two words, the whole
# cycles a load and the hundredths, as the release prints them (24 bytes a region). 800FFFF0h holds D1A0D00Eh when
# done.
        j setup
        nop
vector: j handler                     # copied to 80000080h
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
        nop
# @release 80010078 80014578: the release build 158's own code, carried here by tests/support/records.rs
# printf: the cells' "%2d.%2-d" carries the whole cycles in a1 and the hundredths in a2; the cursor at 800FFFE0h
record: lui t0, 0x8001
        ori t0, t0, 0x5834
        bne a0, t0, r1
        lui t1, 0x8010
        lw t2, -0x20(t1)
        nop
        sw a1, 0(t2)
        sw a2, 4(t2)
        addiu t2, t2, 8
        sw t2, -0x20(t1)
r1:     jr ra
        nop
# EnterCriticalSection and ExitCriticalSection: interrupts are off throughout
critical: jr ra
        nop
# the address error of SPUCNT's 32-bit load: past the load
handler: mfc0 k0, EPC
        nop
        addiu k0, k0, 4
        jr k0
        rfe
setup:  mtc0 zero, SR
        lui sp, 0x801F
        ori sp, sp, 0xFF00
        lui t0, 0x8001
        lw t1, 8(t0)
        lw t2, 12(t0)
        lui t3, 0x8000
        sw t1, 0x80(t3)
        sw t2, 0x84(t3)
        lui t1, 0x8010
        sw t1, -0x20(t1)
        jal 0x8001446c
        nop
        lui t0, 0x8010
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(t0)
done:   j done
        nop
