; FM voices for the references' audio and the tests (Endou_Native.md §16.9): the FM unit found as SMS Power!'s "FM Chip
; Detection" finds it, its result at $C001, then switched on by port $F2 and written from a table of (frames, count,
; address and data pairs), each voice held for its frames, counted by the VDP's frame interrupt. Each write waits out
; the YM2413's 84 clocks [D the application manual, Table II-2].
        di
        jp main
        org $38
        push af
        in a,($BF)
        pop af
        ei
        reti
main:   im 1
        ld sp,$DFF0
        ld a,$AB|$04
        out ($3E),a
        ld bc,$0700
detect: ld a,b
        and 1
        out ($F2),a
        ld e,a
        in a,($F2)
        and 7
        cp e
        jr nz,miss
        inc c
miss:   djnz detect
        ld a,c
        cp 7
        ld a,0
        jr nz,none
        inc a
none:   ld ($C001),a
        out ($F2),a
        ld a,$AB
        out ($3E),a
        ld hl,regs
        ld bc,(regs_end-regs)<<8|$BF
        otir
        ei
        ld hl,voices
next:   ld d,(hl)
        inc hl
        ld a,d
        or a
        jr z,done
        ld b,(hl)
        inc hl
write:  ld a,(hl)
        out ($F0),a
        inc hl
        ld a,(hl)
        nop
        nop
        out ($F1),a
        inc hl
        ld c,6
pause:  dec c
        jr nz,pause
        djnz write
wait:   halt
        dec d
        jr nz,wait
        jr next
done:   halt
        jr done

; The display off and the frame interrupt on.
regs:   db $00,$80,$20,$81
regs_end:

; Each voice: frames, pairs, then the pairs.
voices:
        ; the user instrument as a carrier's sine at 437 Hz, then keyed off
        db 60,12, $00,$21, $01,$21, $02,$3F, $03,$00, $04,$F0, $05,$F0, $06,$0F, $07,$0F, $30,$00, $10,$20, $20,$19, $0E,$00
        db 10,1, $20,$09
        ; the modulator at TL 16 with feedback 3, both slots' envelopes moving
        db 60,9, $02,$10, $03,$03, $04,$C3, $05,$D2, $06,$24, $07,$33, $20,$09, $10,$40, $20,$19
        db 20,1, $20,$09
        ; instruments 1, 3, 5, 8, 11 and 14 on channels 2 to 7, a chord
        db 60,18, $31,$10, $32,$30, $33,$50, $34,$80, $35,$B0, $36,$E0, $11,$AC, $12,$20, $13,$81, $14,$AC, $15,$20, $16,$81, $21,$18, $22,$19, $23,$1B, $24,$1A, $25,$1B, $26,$1D
        db 20,6, $21,$08, $22,$09, $23,$0B, $24,$0A, $25,$0B, $26,$0D
        ; the rhythm: its frequencies as the manual advises, then each drum
        db 4,10, $36,$00, $37,$00, $38,$00, $16,$20, $17,$50, $18,$C0, $26,$05, $27,$05, $28,$01, $0E,$20
        db 20,1, $0E,$30
        db 20,1, $0E,$28
        db 20,1, $0E,$24
        db 20,1, $0E,$22
        db 20,1, $0E,$21
        db 30,1, $0E,$3F
        db 30,1, $0E,$20
        db 0
