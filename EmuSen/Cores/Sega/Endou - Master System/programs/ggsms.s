; A Master System cartridge's picture on the Game Gear (Jadeite_Native.md §3, Endou_Disputes.md D-22): tile 1
; everywhere, its top row colour 1 (white) and its left column colour 2 (red) on colour 0 (black), so that every eighth
; line and column is lit, in each phase of three; the backdrop blue. From frame 30 M1 asks for 224 lines, whose name
; table at $3700 holds tile 1 as well.
        di
        jp main
        org $38
        push af
        in a,($BF)
        pop af
        ei
        ret
main:   im 1
        ld sp,$DFF0
        xor a
        ld (frames),a
        ld hl,regs
        ld bc,(regs_end-regs)<<8|$BF
        otir
        ld hl,$4020
        call address
        ld a,$FF
        out ($BE),a
        xor a
        out ($BE),a
        out ($BE),a
        out ($BE),a
        ld b,7
column: xor a
        out ($BE),a
        ld a,$80
        out ($BE),a
        xor a
        out ($BE),a
        out ($BE),a
        djnz column
        ld hl,$7700
        call address
        ld bc,$0400
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
        ld a,$D0
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
wait:   halt
        ld hl,frames
        inc (hl)
        ld a,(hl)
        cp 30
        jr nz,wait
        ld a,$F0
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
regs:   db $06,$80,$00,$81,$FF,$82,$FF,$83,$FF,$84,$FF,$85,$FB,$86,$00,$87,$00,$88,$00,$89,$FF,$8A
regs_end:
colours:
        db $00,$3F,$03,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00
        db $30,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00
frames: equ $C100
