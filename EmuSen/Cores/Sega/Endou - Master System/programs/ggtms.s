; The TMS9918's Graphics I in the Game Gear's own mode (Jadeite_Native.md §4): pattern 0's rows four pixels of the
; colour table's foreground, 6, then four of its background, 12, everywhere; the backdrop 4; codes shown as CRAM's
; entries 16 plus the code, words written through the latch [R MacDonald's `msvdp` §14].
        di
        im 1
        ld sp,$DFF0
        ld hl,regs
        ld bc,(regs_end-regs)<<8|$BF
        otir
        ld hl,$4000
        call address
        ld b,8
pattern:
        ld a,$F0
        out ($BE),a
        djnz pattern
        ld hl,$6000
        call address
        ld a,$6C
        out ($BE),a
        ld hl,$7B00
        call address
        ld a,$D0
        out ($BE),a
        ld hl,$C000+2*20
        call address
        ld hl,colours
        ld bc,(colours_end-colours)<<8|$BE
        otir
        ld a,$C0
        out ($BF),a
        ld a,$81
        out ($BF),a
done:   jr done
address:
        ld a,l
        out ($BF),a
        ld a,h
        out ($BF),a
        ret
regs:   db $00,$80,$80,$81,$0E,$82,$80,$83,$00,$84,$76,$85,$00,$86,$F4,$87
regs_end:
colours:
        dw $0789,$0000,$0123,$0000,$0000,$0000,$0000,$0000,$0456
colours_end:
