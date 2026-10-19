; The data port written at three rates during the active display (OTIR, 21 cycles apart; OUTI and JR, 28; OUTI, NOP
; and JR, 32) and at OUTI's 16 during vertical blanking, 64 bytes each to $0000, $0040, $0080 and $00C0, then read
; back slowly in blanking to $C000-$C0FF; $C100 is $5A when done (Endou_Native.md §10.5).
        di
        ld sp,$DFF0
        ld hl,regs
        ld bc,(regs_end-regs)<<8|$BF
        otir
        ld d,$20
        call wait
        ld hl,$4000
        call address
        ld hl,data
        ld bc,$40BE
        otir
        ld d,$40
        call wait
        ld hl,$4040
        call address
        ld hl,data
        ld bc,$40BE
rate28: outi
        jr nz,rate28
        ld d,$60
        call wait
        ld hl,$4080
        call address
        ld hl,data
        ld bc,$40BE
rate32: outi
        nop
        jr nz,rate32
        ld d,$C8
        call wait
        ld hl,$40C0
        call address
        ld hl,data
        ld bc,$40BE
rate16: outi
        outi
        outi
        outi
        jr nz,rate16
        ld hl,$0000
        call address
        ld hl,$C000
        ld b,0
back:   in a,($BE)
        ld (hl),a
        inc hl
        djnz back
        ld a,$5A
        ld ($C100),a
done:   jr done
; The V counter waited for until it reads D.
wait:   in a,($7E)
        cp d
        jr nz,wait
        ret
; HL to the control port, low byte first.
address:
        ld a,l
        out ($BF),a
        ld a,h
        out ($BF),a
        ret
regs:   db $06,$80,$C0,$81,$FF,$82,$FF,$85,$FB,$86,$00,$87
regs_end:
data:   db 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32
        db 33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64
