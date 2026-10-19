; The TMS9918's modes on an SG-1000 (Endou_Native.md §11): a Graphics II picture of all three pattern and colour
; tables with six sprites on lines 100-107 (two overlapping, one moved by its early clock bit, a fifth on the line and a
; sixth), then from frame 30 Graphics I, from frame 60 Text and from frame 90 Multicolor. The frame interrupt keeps
; the status of each of the first 16 frames from $C000, and the main program reads a byte written in the 16K mode back
; in the 4K mode at $C010 and $C011.
        di
        jp main
        org $38
        push af
        push hl
        in a,($BF)
        ld hl,(count)
        ld h,$C0
        bit 4,l
        jr nz,counted
        ld (hl),a
counted:
        ld hl,frames
        inc (hl)
        ld a,(hl)
        cp 30
        call z,graphic1
        cp 60
        call z,text
        cp 90
        call z,multicolor
        ld hl,count
        inc (hl)
        pop hl
        pop af
        ei
        ret
graphic1:
        ld hl,mode_g1
        jr setmode
text:
        ld hl,mode_text
        jr setmode
multicolor:
        ld hl,mode_mc
setmode:
        push bc
        push af
        ld bc,8<<8|$BF
        otir
        pop af
        pop bc
        ret
main:   im 1
        ld sp,$C3F0
        xor a
        ld (count),a
        ld (frames),a
        ld hl,regs
        ld bc,(regs_end-regs)<<8|$BF
        otir
        ; Patterns $0000-$17FF: each byte its address's low byte XOR its high byte.
        ld hl,$4000
        call address
        ld de,$0000
pattern:
        ld a,e
        xor d
        out ($BE),a
        inc de
        ld a,d
        cp $18
        jr nz,pattern
        ; Colours $2000-$37FF: each byte its address's low byte plus its high byte, never 0 in either nibble's place.
        ld hl,$6000
        call address
        ld de,$2000
colour:
        ld a,e
        add a,d
        or $11
        out ($BE),a
        inc de
        ld a,d
        cp $38
        jr nz,colour
        ; Names $3800-$3AFF: each its position's low byte.
        ld hl,$7800
        call address
        ld de,$0000
names:  ld a,e
        out ($BE),a
        inc de
        ld a,d
        cp 3
        jr nz,names
        ; Sprite patterns at $1000 are the pattern table's third; the sprite table at $3B00.
        ld hl,$7B00
        call address
        ld hl,sprites
        ld bc,(sprites_end-sprites)<<8|$BE
        otir
        ; A byte at $3C40 in the 16K mode, read back at $3C40 and $3E00 in the 4K mode.
        ld hl,$7C40
        call address
        ld a,$A5
        out ($BE),a
        ld a,$00
        out ($BF),a
        ld a,$81
        out ($BF),a
        ld hl,$3C40
        call address
        in a,($BE)
        ld ($C010),a
        ld hl,$3E00
        call address
        in a,($BE)
        ld ($C011),a
        ; The display and the frame interrupt on, in the 16K mode.
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
count:  equ $C100
frames: equ $C101
regs:   db $02,$80,$80,$81,$0E,$82,$FF,$83,$03,$84,$76,$85,$02,$86,$F4,$87
regs_end:
sprites:
        db 99,10,0,$0F
        db 99,14,1,$06
        db 99,40,2,$83
        db 99,70,3,$0A
        db 99,100,4,$0C
        db 99,130,5,$0D
        db $D0
sprites_end:
mode_g1:
        db $00,$80,$E0,$81,$80,$83,$00,$84
mode_text:
        db $00,$80,$F0,$81,$80,$83,$00,$84
mode_mc:
        db $00,$80,$E8,$81,$80,$83,$00,$84
