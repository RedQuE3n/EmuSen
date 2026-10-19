; Sega's mapper: banks in each slot, the fixed first kilobyte, a bank past the ROM, RAM's copy of $FFFF, the
; cartridge RAM in both banks; results from $C000 (Endou_Native.md §9.8).
        di
        ld sp,$DFF0
        ld a,5
        ld ($FFFF),a
        ld a,($BF00)
        ld ($C000),a
        ld a,6
        ld ($FFFE),a
        ld a,($7F00)
        ld ($C001),a
        ld a,7
        ld ($FFFD),a
        ld a,($3F00)
        ld ($C002),a
        ld a,($0000)
        ld ($C003),a
        ld a,33
        ld ($FFFF),a
        ld a,($BF00)
        ld ($C004),a
        ld a,($FFFF)
        ld ($C005),a
        ld a,$08
        ld ($FFFC),a
        ld a,$77
        ld ($8000),a
        ld a,($8000)
        ld ($C006),a
        ld a,$0C
        ld ($FFFC),a
        ld a,($8000)
        ld ($C007),a
        xor a
        ld ($FFFC),a
        ld a,($BF00)
        ld ($C008),a
done:   jr done
