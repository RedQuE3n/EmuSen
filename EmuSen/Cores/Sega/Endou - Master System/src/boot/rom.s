; Endou's open boot program, the part in ROM (Endou_OpenBoot.md §3.1): the console set up as Sega's Mark III manual
; says the BIOS sets it, the program's own screen in beryl-font's letters with no logo, and the RAM program of ram.s
; copied to $C700 and entered there. The RAM program comes back to $0080 with no game found, or to $0083 with the reason
; a game is refused in A. build.rs gives FONT_AT, RAM_AT and RAM_LEN, and places the font's tiles and the RAM program.
; Every write to the VDP is at least 16 cycles after the one before with the display off and 29 with it on, the
; manual's rates (Endou_OpenBoot.md §2, U12).

RAM_PROGRAM equ $C700
NAMES   equ $3800
STACK   equ $DFF0

        org $0000
        di
        im 1
        ld sp,STACK
        jp start

        org $0038
        reti

        org $0066
        retn

        org $0080
        jp no_game
        jp refuse

start:  ld a,$E3                ; the BIOS, the work RAM and the I/O chip on, every slot off (U1)
        out ($3E),a
        ld a,$A5                ; the TH pins outputs at a high level, the TR pins inputs (MK3, the trackball)
        out ($3F),a
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
        ld hl,$0000             ; the font's 64 tiles from tile 0, the character 32 + n in tile n
        call vram_address
        ld hl,FONT_AT
        ld bc,64*32
.font:  ld a,(hl)
        out ($BE),a
        inc hl
        dec bc
        ld a,b
        or c
        jr nz,.font
        ld hl,$0000             ; the colours
        call cram_address
        ld hl,colours
        ld b,32
.cram:  ld a,(hl)
        out ($BE),a
        inc hl
        djnz .cram
        ld hl,title
        call text
        ld a,$C0                ; the display on
        out ($BF),a
        ld a,$81
        out ($BF),a
        ld hl,RAM_AT
        ld de,RAM_PROGRAM
        ld bc,RAM_LEN
        ldir
        jp RAM_PROGRAM

; No game in any slot: the reason 0. A refusal: the reason in A. The ROM is on again, the display still on.
no_game:
        xor a
refuse: ld sp,STACK
        add a,a
        ld e,a
        ld d,0
        ld hl,reasons
        add hl,de
        ld a,(hl)
        inc hl
        ld h,(hl)
        ld l,a
        call text
wait:   halt
        jr wait

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
        sub 32
        out ($BE),a
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

; CRAM: the background and the backdrop dark blue, the letters white.
colours:
        db $10,$3F,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00
        db $10,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00,$00

; A line at row R and column C is at NAMES + R*64 + C*2; the visible columns are 1 to 31.
title:  dw NAMES+9*64+9*2
        db "ENDOU OPEN BOOT",0
        dw 0

reasons:
        dw no_game_text, no_header_text, japanese_text, region_text, size_text, checksum_text

no_game_text:
        dw NAMES+13*64+3*2
        db "NO GAME IS IN THE CONSOLE.",0
        dw 0

no_header_text:
        dw NAMES+13*64+5*2
        db "THE GAME CANNOT START.",0
        dw NAMES+15*64+5*2
        db "IT HAS NO SEGA HEADER.",0
        dw NAMES+16*64+3*2
        db "IT MAY BE A JAPANESE GAME.",0
        dw 0

japanese_text:
        dw NAMES+13*64+5*2
        db "THE GAME CANNOT START.",0
        dw NAMES+15*64+2*2
        db "IT IS FOR A JAPANESE CONSOLE.",0
        dw 0

region_text:
        dw NAMES+13*64+5*2
        db "THE GAME CANNOT START.",0
        dw NAMES+15*64+3*2
        db "IT IS NOT FOR THIS CONSOLE.",0
        dw 0

size_text:
        dw NAMES+13*64+5*2
        db "THE GAME CANNOT START.",0
        dw NAMES+15*64+4*2
        db "ITS HEADER'S SIZE CODE IS",0
        dw NAMES+16*64+7*2
        db "NOT ONE SEGA LISTS.",0
        dw 0

checksum_text:
        dw NAMES+13*64+5*2
        db "THE GAME CANNOT START.",0
        dw NAMES+15*64+3*2
        db "ITS CHECKSUM DOES NOT MATCH",0
        dw NAMES+16*64+12*2
        db "ITS DATA.",0
        dw 0
