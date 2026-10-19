; Endou's open boot program, the part that runs from RAM at $C700 (Endou_OpenBoot.md §3.2): with the BIOS off it
; enables the card, the cartridge and the expansion slot in turn and takes the first that holds a game, checks an
; export cartridge's header (TMR SEGA, the region code $4, the checksum over the size code's range less the header,
; banks past 32 KiB paged into slot 2 by $FFFF), and hands the game over on the VDP's last line with the slot's port
; $3E value at $C000. A refusal switches the ROM back on and returns to it. build.rs gives ROM_NO_GAME and ROM_REFUSE.

BIOS_ON equ $E3
NO_HEADER equ 1
JAPANESE equ 2
REGION  equ 3
SIZE    equ 4
CHECKSUM equ 5

        org $C700
start:  ld hl,slots
        ld b,3
.slot:  ld a,(hl)
        out ($3E),a
        ld (chosen),a
        push hl
        push bc
        call present
        pop bc
        pop hl
        jr nc,found
        inc hl
        djnz .slot
        ld a,BIOS_ON
        out ($3E),a
        jp ROM_NO_GAME

; The enabled slot: carry clear when it holds a game, with (header) its header's address or 0 for none. A game is a
; header with TMR SEGA at $7FF0, $3FF0 or $1FF0, or else 64 bytes across $0000-$BFFF that are not all one value; an
; empty slot reads one value throughout, the floating bus's or $FF (Endou_OpenBoot.md §2, U4).
present:
        ld hl,places
        ld b,3
.place: ld e,(hl)
        inc hl
        ld d,(hl)
        inc hl
        push hl
        push bc
        ex de,hl
        ld (header),hl
        ld de,signature
        ld b,8
.cmp:   ld a,(de)
        cp (hl)
        jr nz,.no
        inc hl
        inc de
        djnz .cmp
        pop bc
        pop hl
        or a
        ret
.no:    pop bc
        pop hl
        djnz .place
        ld hl,0
        ld (header),hl
        ld a,(hl)
        ld c,a
        ld b,64
        ld de,$0300
.sample:
        ld a,(hl)
        cp c
        jr nz,.data
        add hl,de
        djnz .sample
        scf
        ret
.data:  or a
        ret

found:  ld hl,(header)
        ld a,h
        or l
        ld a,NO_HEADER
        jp z,refuse
        ld de,15
        add hl,de
        ld a,(hl)
        ld (code),a
        and $F0
        cp $40
        jr z,export
        cp $30
        ld a,JAPANESE
        jp z,refuse
        ld a,REGION
        jp refuse

; The size code's range in KiB, the stated sum, then the sum: the part below $8000 read directly, the header's 16
; bytes left out, then each bank from 2 paged into slot 2.
export: ld hl,(header)
        ld de,10
        add hl,de
        ld e,(hl)
        inc hl
        ld d,(hl)
        ld (stated),de
        ld a,(code)
        and $0F
        add a,a
        ld e,a
        ld d,0
        ld hl,sizes
        add hl,de
        ld e,(hl)
        inc hl
        ld d,(hl)
        ld a,d
        or e
        ld a,SIZE
        jp z,refuse
        ld hl,0
        ld (sum),hl
        ld hl,32
        or a
        sbc hl,de
        jr nc,.small
        ex de,hl
        srl h
        rr l
        srl h
        rr l
        srl h
        rr l
        srl h
        rr l
        ld a,l
        sub 2
        ld (banks),a
        ld de,32
        jr .direct
.small: xor a
        ld (banks),a
.direct:
        ex de,hl
        add hl,hl
        add hl,hl
        add hl,hl
        add hl,hl
        add hl,hl
        add hl,hl
        ld (end),hl
        ld hl,(header)
        srl h
        rr l
        srl h
        rr l
        srl h
        rr l
        srl h
        rr l
        ld (hgroup),hl
        ld de,(end)
        or a
        sbc hl,de
        jr c,.split
        ld bc,(end)
        ld hl,0
        call sum_groups
        jr .banks
.split: ld bc,(hgroup)
        ld hl,0
        call sum_groups
        ld hl,(end)
        ld de,(hgroup)
        or a
        sbc hl,de
        dec hl
        ld b,h
        ld c,l
        ld hl,(header)
        ld de,16
        add hl,de
        call sum_groups
.banks: ld a,(banks)
        or a
        jr z,.compare
        ld (count),a
        ld a,1
        ld (paged),a
        ld a,2
.bank:  ld (bank),a
        ld ($FFFF),a
        ld hl,$8000
        ld bc,$0400
        call sum_groups
        ld a,(bank)
        inc a
        ld hl,count
        dec (hl)
        jr nz,.bank
.compare:
        ld hl,(sum)
        ld de,(stated)
        or a
        sbc hl,de
        ld a,CHECKSUM
        jp nz,refuse

; The hand-over: the VDP's registers to Sega's power-on values (the display off with R1 first), VRAM and CRAM cleared,
; the RAM cleared but $C000 and this program, slot 2 back to bank 2 where it was paged, then on the VDP's last line the
; status read, the slot's value at $C000 and in port $3E, and the game from $0000 (Endou_OpenBoot.md §3.2).
handover:
        ld hl,power_on
        ld b,power_on_end-power_on
.reg:   ld a,(hl)
        out ($BF),a
        inc hl
        ex (sp),hl
        ex (sp),hl
        djnz .reg
        xor a
        out ($BF),a
        ld a,$40
        out ($BF),a
        ld c,$BE
        xor a
        ld e,16
.vram:  out (c),a
        nop
        out (c),a
        nop
        out (c),a
        nop
        out (c),a
        djnz .vram
        dec e
        jr nz,.vram
        out ($BF),a
        ld a,$C0
        out ($BF),a
        xor a
        ld b,32
.cram:  out (c),a
        nop
        djnz .cram
        ld hl,$C000
        ld de,$C001
        ld bc,start-$C001
        ld (hl),0
        ldir
        ld hl,ram_end
        ld de,ram_end+1
        ld bc,$DFFF-ram_end
        ld (hl),0
        ldir
        ld a,(paged)
        or a
        jr z,.line
        ld a,2
        ld ($FFFF),a
.line:  in a,($7E)
        cp $FF
        jr z,.line
.last:  in a,($7E)
        cp $FF
        jr nz,.last
        in a,($BF)
        ld a,(chosen)
        ld ($C000),a
        out ($3E),a
        jp $0000

refuse: ld b,a
        ld a,BIOS_ON
        out ($3E),a
        ld a,b
        jp ROM_REFUSE

; BC 16-byte groups from HL, aligned to 16, added to (sum); none when BC is 0. C counts the inner loop's groups and B
; its rounds, so that BC = 256 rounds of 256 is the most.
sum_groups:
        ld a,b
        or c
        ret z
        ld a,c
        or a
        jr z,.whole
        inc b
.whole: ld de,(sum)
        ld a,e
.group: add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc l
        add a,(hl)
        jr nc,$+3
        inc d
        inc hl
        dec c
        jr nz,.group
        dec b
        jr nz,.group
        ld e,a
        ld (sum),de
        ret

; The card's, the cartridge's and the expansion slot's port $3E values, each with the BIOS off.
slots:  db $CB,$AB,$6B
places: dw $7FF0,$3FF0,$1FF0
signature:
        db "TMR SEGA"
; The size codes' ranges in KiB, 0 for a code SMS Power!'s "ROM Header" does not list.
sizes:  dw 256,512,1024,0,0,0,0,0,0,0,8,16,32,48,64,128
; Sega's power-on values (MK3, "VDP register description"), R1 first so that the display goes off before the rest.
power_on:
        db $A0,$81, $36,$80, $FF,$82, $FF,$83, $FF,$84, $FF,$85, $FB,$86, $00,$87, $00,$88, $00,$89, $FF,$8A
power_on_end:
chosen: db 0
paged:  db 0
code:   db 0
banks:  db 0
count:  db 0
bank:   db 0
header: dw 0
hgroup: dw 0
end:    dw 0
stated: dw 0
sum:    dw 0
ram_end:
