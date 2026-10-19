; Codemasters' mapper: slots 2 and 1 banked by their own addresses, then the RAM bit; results from $C000.
        di
        ld sp,$DFF0
        ld a,5
        ld ($8000),a
        ld a,($BF00)
        ld ($C000),a
        ld a,3
        ld ($4000),a
        ld a,($7F00)
        ld ($C001),a
        ld a,$84
        ld ($4000),a
        ld a,$5A
        ld ($A000),a
        ld a,($A000)
        ld ($C002),a
        ld a,($BF00)
        ld ($C003),a
        ld a,($8000)
        ld ($C004),a
        ld a,($7F00)
        ld ($C005),a
done:   jr done
