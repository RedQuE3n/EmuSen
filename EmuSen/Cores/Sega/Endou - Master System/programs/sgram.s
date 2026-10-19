; An SG-1000 RAM board: 2 KiB at $8000 mirrored, and the 1 KiB of work RAM mirrored; results from $C000.
        di
        ld sp,$C3F0
        ld a,$5A
        ld ($8000),a
        ld a,($8800)
        ld ($C000),a
        ld a,($C400)
        ld ($C001),a
done:   jr done
