; The Game Gear's system ports in GG mode (Jadeite_Native.md §9.2): ports $00-$06 read at power-on into $C000-$C006;
; the parallel pins' levels and directions, the byte to send and the serial status at $C008-$C00C; the clocks a byte
; takes at 300 bits a second as a count of 33-cycle loops at $C00D; the receive register at $C00F; writes of $FF to
; the even ports $00-$04 leaving the work RAM on ($C011), where port $3E's bit 4 would switch it off; port $06 putting
; every channel on the right, tone 0 sounding; then port $00, Start and the market, at $C012 on every pass.
        di
        im 1
        ld sp,$DFF0
        ld hl,$C000
        ld c,0
read:   in a,(c)
        ld (hl),a
        inc hl
        inc c
        ld a,c
        cp 7
        jr nz,read
        ld a,$55
        out ($01),a
        ld a,$0F
        out ($02),a
        in a,($01)
        ld ($C008),a
        in a,($02)
        ld ($C009),a
        ld a,$A5
        out ($03),a
        in a,($03)
        ld ($C00A),a
        in a,($05)
        ld ($C00B),a
        ld a,$D0
        out ($05),a
        ld a,$3C
        out ($03),a
        in a,($05)
        ld ($C00C),a
        ld de,0
tx:     inc de
        in a,($05)
        rra
        jr c,tx
        ld ($C00D),de
        in a,($04)
        ld ($C00F),a
        ld a,$FF
        out ($00),a
        out ($02),a
        out ($04),a
        ld a,$5A
        ld ($C010),a
        ld a,($C010)
        ld ($C011),a
        ld a,$0F
        out ($06),a
        ld a,$8E
        out ($7F),a
        ld a,$0F
        out ($7F),a
        ld a,$90
        out ($7F),a
loop:   in a,($00)
        ld ($C012),a
        jr loop
