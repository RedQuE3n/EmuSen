# The load delay's corners (BlackMoon_R3000.md §5.3): a second load of the register a load is pending for, an
# instruction writing it, LWL/LWR after a load, MFC0 after a load. Results from 80100000h, eight words; 800FFFF0h holds
# D1A0D00Eh when done. Data: 80100F00h = 11111111h, 80100F04h = 22222222h.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x8010
        lui t0, 0x1111
        ori t0, t0, 0x1111
        sw t0, 0xF00(a0)
        lui t0, 0x2222
        ori t0, t0, 0x2222
        sw t0, 0xF04(a0)
        addiu t0, zero, 0x55
        lw t0, 0xF00(a0)          # a load pending for t0
        lw t0, 0xF04(a0)          # a second load of t0 while the first is pending
        or t1, t0, zero           # result 0: 55h if the first is cancelled, 11111111h if it lands
        or t2, t0, zero           # result 1: 22222222h
        sw t1, 0(a0)
        sw t2, 4(a0)
        addiu t3, zero, 0x66
        lw t3, 0xF00(a0)
        addiu t3, zero, 0x77      # the instruction's own write over the pending load
        or t4, t3, zero           # result 2: 77h
        sw t4, 8(a0)
        addiu t5, zero, 0
        lw t5, 0xF00(a0)
        lwl t5, 0xF07(a0)         # LWL merging with the pending load's value
        lwr t5, 0xF04(a0)
        nop
        sw t5, 12(a0)             # result 3: 22222222h
        lui t6, 0x3333
        lw t6, 0xF00(a0)
        mfc0 t6, SR               # MFC0 through the same delay, after a pending load
        or t7, t6, zero           # result 4: as the first case
        nop
        or t8, t6, zero           # result 5: SR (0)
        sw t7, 16(a0)
        sw t8, 20(a0)
        lw s1, 0xF00(a0)
        or s2, s1, zero           # result 6: the old s1 (0), the load not yet landed
        or s3, s1, zero           # result 7: 11111111h
        sw s2, 24(a0)
        sw s3, 28(a0)
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        sw t1, -16(a0)
done:   j done
        nop
