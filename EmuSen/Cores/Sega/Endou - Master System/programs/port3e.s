; Port $3E: the cartridge off from a routine in RAM, the work RAM off, and port $3E read; results from $C000.
        di
        ld sp,$DFF0
        ld hl,routine
        ld de,$C100
        ld bc,routine_end-routine
        ldir
        call $C100
        ld a,$AB|$10
        out ($3E),a
        ld a,$99
        ld ($C010),a
        ld a,$AB
        out ($3E),a
        ld a,($C010)
        ld ($C002),a
        in a,($3E)
        ld ($C003),a
done:   jr done
routine:
        ld a,$AB|$40
        out ($3E),a
        ld a,($0100)
        ld b,a
        ld a,$AB
        out ($3E),a
        ld a,b
        ld ($C000),a
        ld a,($0100)
        ld ($C001),a
        ret
routine_end:
