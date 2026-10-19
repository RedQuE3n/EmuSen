; The PSG written through $7F and $41, then the V counter and the VDP status read; results from $C000.
        di
        ld sp,$DFF0
        ld a,$9F
        out ($7F),a
        ld a,$C4
        out ($41),a
        in a,($7E)
        ld ($C000),a
        in a,($BF)
        ld ($C001),a
done:   jr done
