; A mode 4 picture: tile 1 (colour 1 left, colour 2 right) over the name table at $3800, a sprite of tile 2 (colour 3)
; at X 100, Y 80, the X scroll 3 and the left column masked; the backdrop's CRAM entry set by the line interrupt at
; line 99, mid-line, and set back by the frame interrupt (Endou_Native.md §10.8).
        di
        jp main
        org $38
        push af
        in a,($BF)
        rlca
        ld a,$11
        out ($BF),a
        ld a,$C0
        out ($BF),a
        ld a,$30
        jr nc,line
        ld a,$15
line:   out ($BE),a
        pop af
        ei
        ret
main:   im 1
        ld sp,$DFF0
        ld hl,regs
        ld bc,(regs_end-regs)<<8|$BF
        otir
        ld hl,$4020
        call address
        ld b,8
tile1:  ld a,$F0
        out ($BE),a
        ld a,$0F
        out ($BE),a
        xor a
        out ($BE),a
        out ($BE),a
        djnz tile1
        ld b,8
tile2:  ld a,$FF
        out ($BE),a
        out ($BE),a
        xor a
        out ($BE),a
        out ($BE),a
        djnz tile2
        ld hl,$7800
        call address
        ld bc,$0300
names:  ld a,1
        out ($BE),a
        xor a
        out ($BE),a
        dec bc
        ld a,b
        or c
        jr nz,names
        ld hl,$7F00
        call address
        ld a,79
        out ($BE),a
        ld a,$D0
        out ($BE),a
        ld hl,$7F80
        call address
        ld a,100
        out ($BE),a
        ld a,2
        out ($BE),a
        ld hl,$C000
        call address
        ld hl,colours
        ld bc,$20BE
        otir
        ld a,$E0
        out ($BF),a
        ld a,$81
        out ($BF),a
        ei
done:   jr done
address:
        ld a,l
        out ($BF),a
        ld a,h
        out ($BF),a
        ret
regs:   db $36,$80,$00,$81,$FF,$82,$FF,$83,$FF,$84,$FF,$85,$FB,$86,$01,$87,$03,$88,$00,$89,99,$8A
regs_end:
colours:
        db $00,$03,$0C,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00
        db $00,$15,$2A,$3F,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00
