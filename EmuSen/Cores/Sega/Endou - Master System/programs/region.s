; SMS Power!'s "Region Detection" sample, as games use it: 1 in $C000 for a Japanese console.
        di
        ld sp,$DFF0
        call is_japanese
        ld ($C000),a
done:   jr done
is_japanese:
        ld a,%11110101
        out ($3F),a
        in a,($DD)
        and %11000000
        cp %11000000
        jp nz,japanese
        ld a,%01010101
        out ($3F),a
        in a,($DD)
        and %11000000
        jp nz,japanese
        ld a,%11111111
        out ($3F),a
        ld a,0
        ret
japanese:
        ld a,1
        ret
