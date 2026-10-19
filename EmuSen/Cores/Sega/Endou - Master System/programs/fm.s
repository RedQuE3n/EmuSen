; The FM unit's detection as SMS Power!'s "FM Chip Detection" describes it, on a console with no FM unit: the I/O chip
; switched off by port $3E, bit 0 written to $F2 and read back seven times; the count of matches from $C000, the result
; from $C001, each read from $C010. Then a tone, and $F2, $F0 and $F1 written as an FM game would (Endou_Native.md §12.4).
        di
        im 1
        ld sp,$DFF0
        ld a,$AB|$04
        out ($3E),a
        ld hl,$C010
        ld bc,$0700
loop:   ld a,b
        and 1
        out ($F2),a
        ld e,a
        in a,($F2)
        ld (hl),a
        inc hl
        and 7
        cp e
        jr nz,miss
        inc c
miss:   djnz loop
        ld a,c
        ld ($C000),a
        cp 7
        ld a,0
        jr nz,none
        inc a
none:   ld ($C001),a
        out ($F2),a
        ld a,$AB
        out ($3E),a
        ld a,$8E
        out ($7F),a
        ld a,$0F
        out ($7F),a
        ld a,$90
        out ($7F),a
        ld a,$03
        out ($F2),a
        ld a,$30
        out ($F0),a
        ld a,$1F
        out ($F1),a
done:   jr done
