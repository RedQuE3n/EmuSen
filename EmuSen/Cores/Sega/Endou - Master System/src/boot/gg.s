; Endou's open boot program for the Game Gear (Jadeite_OpenBoot.md §3): 1 KiB at $0000-$03FF, the cartridge answering
; above it. It waits for the VDP as Sega's Game Gear notes ask, silences the PSG, shows its own screen in beryl-font's
; letters with no logo inside the LCD's window, looks for TMR SEGA at $7FF0, $3FF0 or $1FF0 and refuses a cartridge
; without it in words, and hands over from RAM on the VDP's last line with $A8 in port $3E and at $C000. build.rs gives
; FONT_AT, where it places the glyphs A to Z and the full stop as one bit a dot, seven rows each, and GLYPHS, their
; count. Every write to the VDP is at least 16 cycles after the one before with the display off and 29 with it on
; (Jadeite_OpenBoot.md §2, U10).

STACK   equ $DFF0
NAMES   equ $3800
STUB    equ $C700
HANDED  equ $A8

        org $0000
        di
        im 1
        ld sp,STACK
        jp start

        org $0038
        reti

        org $0066
        retn

; The V counter at $B0 before the VDP is touched (GG, "VDP initialization"); the PSG silenced (GG, the PSG manual).
start:  in a,($7E)
        cp $B0
        jr nz,start
        ld hl,silence
        ld b,4
.psg:   ld a,(hl)
        out ($7F),a
        inc hl
        djnz .psg
        ld hl,screen
        ld b,screen_end-screen
        call registers
        ld hl,NAMES             ; the name table and the sprite table, $3800-$3FFF, cleared
        call vram_address
        ld bc,$0800
.clear: xor a
        out ($BE),a
        dec bc
        ld a,b
        or c
        jr nz,.clear
        ld hl,$3F00             ; the sprite list ends at its first entry
        call vram_address
        ld a,$D0
        out ($BE),a
        ld hl,32                ; the glyphs from tile 1, tile 0 blank: each row in bitplane 0, the other three clear
        call vram_address
        ld hl,FONT_AT
        ld d,GLYPHS
.glyph: ld b,7
.row:   ld a,(hl)
        out ($BE),a
        inc hl
        xor a
        out ($BE),a
        nop
        nop
        out ($BE),a
        nop
        nop
        out ($BE),a
        djnz .row
        ld b,4
.blank: out ($BE),a
        djnz .blank
        dec d
        jr nz,.glyph
        ld hl,0                 ; CRAM cleared, 64 bytes in GG mode, 32 twice over in the other
        call cram_address
        xor a
        ld b,64
.cram:  out ($BE),a
        nop
        djnz .cram
        ld hl,0                 ; the letters white: a word of 12 bits in GG mode, a byte of six in the other (U3)
        call cram_address
        in a,($00)
        and $1F
        ld hl,gg_colours
        ld b,4
        jr z,.colours
        ld hl,sms_colours
        ld b,2
.colours:
        ld a,(hl)
        out ($BE),a
        inc hl
        djnz .colours
        ld hl,title
        call text
        ld a,$C0                ; the display on
        out ($BF),a
        ld a,$81
        out ($BF),a

; Sega's header: TMR SEGA at $7FF0, $3FF0 or $1FF0, the cartridge's while the boot ROM is on (Jadeite_OpenBoot.md §1.3).
        ld hl,places
        ld c,3
.place: ld e,(hl)
        inc hl
        ld d,(hl)
        inc hl
        push hl
        ld hl,signature
        ld b,8
.cmp:   ld a,(de)
        cp (hl)
        jr nz,.next
        inc de
        inc hl
        djnz .cmp
        pop hl
        jr handover
.next:  pop hl
        dec c
        jr nz,.place

; No header: the refusal's words, and a halt with interrupts off; Start's NMI in a Master System cartridge's mode
; returns to it (U13).
        ld hl,refusal
        call text
.halt:  halt
        jr .halt

; The hand-over: the registers to Sega's power-on values (the display off with R1 first), VRAM and CRAM cleared, the
; RAM cleared, $A8 at $C000 and the hand-over at $C700; then on the VDP's last line the status read, and from RAM port
; $3E written, which takes the boot ROM away, and the game from $0000 (Jadeite_OpenBoot.md §3, U6-U9).
handover:
        ld hl,power_on
        ld b,power_on_end-power_on
        call registers
        xor a
        out ($BF),a
        ld a,$40
        out ($BF),a
        ld c,$BE
        xor a
        ld e,64
.vram:  out (c),a
        nop
        djnz .vram
        dec e
        jr nz,.vram
        out ($BF),a
        ld a,$C0
        out ($BF),a
        xor a
        ld b,64
.cram:  out (c),a
        nop
        djnz .cram
        ld hl,$C000
        ld de,$C001
        ld bc,$1FFF
        ld (hl),a
        ldir
        ld hl,stub
        ld de,STUB
        ld bc,stub_end-stub
        ldir
        ld a,HANDED
        ld ($C000),a
.line:  in a,($7E)
        cp $FF
        jr z,.line
.last:  in a,($7E)
        cp $FF
        jr nz,.last
        in a,($BF)
        ld a,HANDED
        jp STUB

stub:   out ($3E),a
        jp $0000
stub_end:

; B bytes from HL to the control port: each register's value, then $80 and its number.
registers:
        ld a,(hl)
        out ($BF),a
        inc hl
        ex (sp),hl
        ex (sp),hl
        djnz registers
        ret

; The VDP's address for writing VRAM at HL.
vram_address:
        ld a,l
        out ($BF),a
        ex (sp),hl
        ex (sp),hl
        ld a,h
        or $40
        out ($BF),a
        ret

; The VDP's address for writing CRAM at L.
cram_address:
        ld a,l
        out ($BF),a
        ex (sp),hl
        ex (sp),hl
        ld a,$C0
        out ($BF),a
        ret

; Lines of text from HL: each its name table address, then its characters ended by 0; the list ended by an address 0.
; A letter is tile 1 to 26, the full stop tile GLYPHS, anything else the blank tile 0.
text:   ld e,(hl)
        inc hl
        ld d,(hl)
        inc hl
        ld a,d
        or e
        ret z
        ex de,hl
        call vram_address
        ex de,hl
.char:  ld a,(hl)
        inc hl
        or a
        jr z,text
        cp $2E                 ; the full stop
        jr nz,.letter
        ld a,$40+GLYPHS
.letter:
        sub $40                ; A is $41, tile 1
        jr nc,.put
        xor a
.put:   out ($BE),a
        ex (sp),hl
        ex (sp),hl
        xor a
        out ($BE),a
        ex (sp),hl
        ex (sp),hl
        jr .char

silence:
        db $9F,$BF,$DF,$FF

; The screen's registers: mode 4, the left column masked, the display off; the tables where Sega's power-on values
; put them.
screen: db $26,$80, $80,$81, $FF,$82, $FF,$83, $FF,$84, $FF,$85, $FB,$86, $00,$87, $00,$88, $00,$89, $FF,$8A
screen_end:

; Sega's power-on values (MK3, "VDP register description"), R1 first so that the display goes off before the rest.
power_on:
        db $A0,$81, $36,$80, $FF,$82, $FF,$83, $FF,$84, $FF,$85, $FB,$86, $00,$87, $00,$88, $00,$89, $FF,$8A
power_on_end:

; Colour 0 black, colour 1 white: two words in GG mode, two bytes in a Master System cartridge's.
gg_colours:
        db $00,$00,$FF,$0F
sms_colours:
        db $00,$3F

places: dw $7FF0,$3FF0,$1FF0
signature:
        db "TMR SEGA"

; A line at row R and column C is at NAMES + R*64 + C*2; the LCD's window is columns 6 to 25 and rows 3 to 20.
title:  dw NAMES+9*64+8*2
        db "ENDOU OPEN BOOT",0
        dw 0

refusal:
        dw NAMES+12*64+8*2
        db "THE GAME CANNOT",0
        dw NAMES+13*64+8*2
        db "START. IT HAS NO",0
        dw NAMES+14*64+8*2
        db "SEGA HEADER.",0
        dw 0
