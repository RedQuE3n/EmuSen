# A program that reaches past B(5Bh)'s table entry as Mega Man Legends' pad library does (Diamond_OpenKernel_Phase2.md
# §27), run on the open kernel: the entry E taken from GetB0Table's table, words stored at the game's offsets past it
# (9 from E+62Ch, 11 from E+594h, 4 from E+3D8h and from E+4E0h), FlushCache, InitPAD2 with buffers at 80100000h and
# 80100040h, StartPAD2, ChangeClearPAD(0) and ChangeClearPAD(1); then, inside a critical section and each after a
# FlushCache, calls at E+884h (twice, the second with the cache warm), E+7A0h, E+894h and E+FCh (twice), a return's
# delay slot, from where the slide is longest. Results from 80100100h: E, v0 after each of the six calls (set to
# 12345678h before it), S2-S7 after them all, the two ChangeClearPAD results. 800FFFF0h holds D1A0D00Eh once port 1's
# pad has been read.
        j start
        nop
flush:  ori t1, zero, 0x44            # A(44h) FlushCache
        ori t2, zero, 0xA0
        jr t2
        nop
call:   lui v0, 0x1234                # a call at t9 with v0 marked; v0 afterwards to 0(s3), s3 moved on
        ori v0, v0, 0x5678
        addu s2, ra, zero
        jalr t9
        nop
        sw v0, 0(s3)
        addu ra, s2, zero
        jr ra
        addiu s3, s3, 4
start:  ori a0, zero, 1
        syscall                       # EnterCriticalSection
        ori t1, zero, 0x57            # B(57h) GetB0Table
        jal 0x800000b0
        nop
        lw s0, 0x16c(v0)              # E
        lui s1, 0x8010
        sw s0, 0x100(s1)
        addiu t3, zero, -1
        addiu t0, s0, 0x62c
        ori t1, zero, 9
st1:    sw t3, 0(t0)
        addiu t1, t1, -1
        bne t1, zero, st1
        addiu t0, t0, 4
        addiu t0, s0, 0x594
        ori t1, zero, 11
st2:    sw t3, 0(t0)
        addiu t1, t1, -1
        bne t1, zero, st2
        addiu t0, t0, 4
        addu t0, s0, zero
        ori t1, zero, 4
st3:    sw t3, 0x3d8(t0)
        sw t3, 0x4e0(t0)
        addiu t1, t1, -1
        bne t1, zero, st3
        addiu t0, t0, 4
        jal flush
        nop
        lui a0, 0x8010
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
        ori a0, zero, 0
        ori t1, zero, 0x5b            # B(5Bh) ChangeClearPAD(0): 1, the default
        jal 0x800000b0
        nop
        sw v0, 0x134(s1)
        ori a0, zero, 1
        ori t1, zero, 0x5b            # ChangeClearPAD(1): 0, and the flag as it was
        jal 0x800000b0
        nop
        sw v0, 0x138(s1)
        addiu s3, s1, 0x104
        lui s4, 0x5454
        lui s5, 0x5555
        lui s6, 0x5656
        lui s7, 0x5757
        jal flush
        nop
        jal call
        addiu t9, s0, 0x884
        jal call
        addiu t9, s0, 0x884
        jal flush
        nop
        jal call
        addiu t9, s0, 0x7a0
        jal flush
        nop
        jal call
        addiu t9, s0, 0x894
        jal flush
        nop
        jal call
        addiu t9, s0, 0xfc
        jal call
        addiu t9, s0, 0xfc
        sw s2, 0x11c(s1)
        sw s3, 0x120(s1)
        sw s4, 0x124(s1)
        sw s5, 0x128(s1)
        sw s6, 0x12c(s1)
        sw s7, 0x130(s1)
        ori a0, zero, 2
        syscall                       # ExitCriticalSection
        ori t2, zero, 0x41
wait:   lbu t0, 1(s1)                 # port 1's ID, once a vblank has read the pad
        nop
        bne t0, t2, wait
        nop
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(s1)
done:   j done
        nop
