; Voices for the references' audio (Endou_Native.md §12.6): the PSG written through $7F from a table of (frames, count,
; bytes), each voice held for its frames, counted by the VDP's frame interrupt; runs on the Master System and the SG-1000.
        di
        jp main
        org $38
        push af
        in a,($BF)
        pop af
        ei
        reti
main:   im 1
        ld sp,$C3F0
        ld hl,regs
        ld bc,(regs_end-regs)<<8|$BF
        otir
        ld hl,quiet
        ld bc,4<<8|$7F
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
        ld c,$7F
        otir
wait:   halt
        dec d
        jr nz,wait
        jr next
done:   halt
        jr done
regs:   db $00,$80,$A0,$81
regs_end:
quiet:  db $9F,$BF,$DF,$FF
; 1 s each: tone 0 at $0FE full, then 8 dB down; tone 1 at $020; periodic noise at rates 0, 2 and tone 2's $040;
; white noise at rates 0 and 1.
voices: db 60,3,$8E,$0F,$90
        db 60,1,$94
        db 60,4,$9F,$A0,$02,$B0
        db 60,3,$BF,$E0,$F0
        db 60,1,$E2
        db 60,4,$C0,$04,$DF,$E3
        db 60,1,$E4
        db 60,1,$E5
; A tone held high by a half-period of 1, stepped through the 16 attenuations, 4 frames each.
        db 4,4,$FF,$81,$00,$90
        db 4,1,$91
        db 4,1,$92
        db 4,1,$93
        db 4,1,$94
        db 4,1,$95
        db 4,1,$96
        db 4,1,$97
        db 4,1,$98
        db 4,1,$99
        db 4,1,$9A
        db 4,1,$9B
        db 4,1,$9C
        db 4,1,$9D
        db 4,1,$9E
        db 4,1,$9F
; All four at full volume: tones $0FE, $17F, $11D and white noise at rate 2; then silence.
        db 60,10,$8E,$0F,$90,$AF,$17,$B0,$CD,$11,$D0,$E6
        db 1,1,$F0
        db 30,4,$9F,$BF,$DF,$FF
        db 0
