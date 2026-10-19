; The H counter latched as TH rises through port $3F on line $40, then TH made to fall, which latches nothing; the two
; reads from $C000 (Endou_Native.md §10.5).
        di
        ld sp,$DFF0
wait:   in a,($7E)
        cp $40
        jr nz,wait
        ld a,$55
        out ($3F),a
        ld a,$FF
        out ($3F),a
        in a,($7F)
        ld ($C000),a
        ld a,$55
        out ($3F),a
        in a,($7F)
        ld ($C001),a
done:   jr done
