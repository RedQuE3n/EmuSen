; The Game Gear's own mode (Jadeite_Native.md §4): CRAM written as words through the latch at even addresses, one
; latch written out to two entries by odd writes; tile 1 (colour 1) inside the LCD's window, tile 2 (colour 2) on its
; edge cells and tile 3 (colour 3, which must never show) outside it; a sprite across the window's left edge on lines 100-107; entry 1
; changed by the line interrupt at line 99 and set back by the frame interrupt, which keeps the V counter at $C000.
; From frame 30 M1 asks for 224 lines, whose name table is at $3700 (Endou_Disputes.md D-21); $3700-$37FF holds tile 3.
        di
        jp main
        org $38
        push af
        push hl
        in a,($BF)
        rlca
        jr nc,line
        in a,($7E)
        ld ($C000),a
        ld hl,frames
        inc (hl)
        ld a,(hl)
        cp 30
        jr nz,blue
        ld a,$F0
        out ($BF),a
        ld a,$81
        out ($BF),a
blue:   ld hl,$0F00
        jr colour
line:   ld hl,$0F0F
colour: ld a,$02
        out ($BF),a
        ld a,$C0
        out ($BF),a
        ld a,l
        out ($BE),a
        ld a,h
        out ($BE),a
        pop hl
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
        ; Tiles 1-4 at $0020: tile n's every row of colour n; tile 4 the sprite's, colour 4 of the sprite palette.
        ld hl,$4020
        call address
        ld de,$0108
tiles:  ld b,8
rows:   ld a,d
        ld c,4
planes: rrca
        push af
        sbc a,a
        out ($BE),a
        pop af
        dec c
        jr nz,planes
        djnz rows
        inc d
        ld a,d
        cp 5
        jr nz,tiles
        ; The name tables, $3700-$3EFF: tile 3 everywhere, then tile 2 over the window's cells and tile 1 inside them.
        ld hl,$7700
        call address
        ld bc,$0400
names:  ld a,3
        out ($BE),a
        xor a
        out ($BE),a
        dec bc
        ld a,b
        or c
        jr nz,names
        ld hl,$7800+3*64+6*2
        ld d,18
window: call address
        ld b,20
        ld a,d
        cp 18
        jr z,edge
        cp 1
        jr z,edge
cells:  ld a,b
        cp 20
        jr z,edgecell
        cp 1
        jr z,edgecell
        ld a,1
        jr cell
edgecell:
        ld a,2
cell:   out ($BE),a
        xor a
        out ($BE),a
        djnz cells
        jr nextrow
edge:   ld a,2
        out ($BE),a
        xor a
        out ($BE),a
        djnz edge
nextrow:
        ld bc,64
        add hl,bc
        dec d
        jr nz,window
        ; One sprite, tile 4 (sprite tiles from $0000 by register 6), at X 44, Y 99: lines 100-107, columns 44-51.
        ld hl,$7F00
        call address
        ld a,99
        out ($BE),a
        ld a,$D0
        out ($BE),a
        ld hl,$7F80
        call address
        ld a,44
        out ($BE),a
        ld a,4
        out ($BE),a
        ; CRAM: each entry's low byte at its even address, latched, then its high byte at the odd address.
        ld hl,$C000
        call address
        ld hl,colours
        ld bc,$40BE
        otir
        ; A latch of $FF written out by odd writes to $07, entry 3 ($0FFF, white, never shown), and to $05, entry 2 ($00FF).
        ld hl,$C006
        call address
        ld a,$FF
        out ($BE),a
        ld hl,$C007
        call address
        ld a,$0F
        out ($BE),a
        ld hl,$C005
        call address
        xor a
        out ($BE),a
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
regs:   db $16,$80,$00,$81,$FF,$82,$FF,$83,$FF,$84,$FF,$85,$FB,$86,$00,$87,$00,$88,$00,$89,99,$8A
regs_end:
colours:
        dw $0000,$0F00,$0000,$0333,$0000,$0000,$0000,$0000,$0000,$0000,$0000,$0000,$0000,$0000,$0000,$0000
        dw $0050,$0000,$0000,$0000,$00F0,$0000,$0000,$0000,$0000,$0000,$0000,$0000,$0000,$0000,$0000,$0000
frames: equ $C100
