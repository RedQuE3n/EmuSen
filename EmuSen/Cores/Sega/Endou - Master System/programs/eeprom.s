; The Game Gear's EEPROM board (Jadeite_Native.md §9.3): the 93C46B driven through $8000 while $FFFC's bit 4 is set,
; DI on bit 0, CLK on bit 1, CS on bit 2, DO read on bit 0 (Endou_Disputes.md D-26). In turn: a WRITE before EWEN, which
; does nothing; EWEN; WRITE of $1234 to word 3 and the status polled, its 35-cycle loops at $C000; READ of word 3 at
; $C002; ERASE of word 3, polled at $C004, read at $C006; WRAL of $5AA5, polled at $C008, words 0 and 1 read in one
; READ at $C00A and $C00C; EWDS and a WRITE of 0 to word 1, which does nothing, word 1 read at $C00E; then done at $C010.
CS      equ 4
CLK     equ 2
EEPROM  equ $8000
        di
        im 1
        ld sp,$DFF0
        ld a,$10
        ld ($FFFC),a
        call start
        ld hl,$4100             ; WRITE word 1, still disabled
        ld b,8
        call send
        ld hl,$9999
        ld b,16
        call send
        call start
        ld hl,$3000             ; EWEN
        ld b,8
        call send
        call start
        ld hl,$4300             ; WRITE word 3
        ld b,8
        call send
        ld hl,$1234
        ld b,16
        call send
        call wait
        ld ($C000),de
        call start
        ld hl,$8300             ; READ word 3
        ld b,8
        call send
        call receive
        ld ($C002),hl
        call start
        ld hl,$C300             ; ERASE word 3
        ld b,8
        call send
        call wait
        ld ($C004),de
        call start
        ld hl,$8300
        ld b,8
        call send
        call receive
        ld ($C006),hl
        call start
        ld hl,$1000             ; WRAL
        ld b,8
        call send
        ld hl,$5AA5
        ld b,16
        call send
        call wait
        ld ($C008),de
        call start
        ld hl,$8000             ; READ word 0, then word 1 with CS held high
        ld b,8
        call send
        call receive
        ld ($C00A),hl
        call receive
        ld ($C00C),hl
        call start
        ld hl,$0000             ; EWDS
        ld b,8
        call send
        call start
        ld hl,$4100             ; WRITE word 1, disabled again
        ld b,8
        call send
        ld hl,$0000
        ld b,16
        call send
        call start
        ld hl,$8100
        ld b,8
        call send
        call receive
        ld ($C00E),hl
        xor a
        ld (EEPROM),a
        ld a,1
        ld ($C010),a
done:   jr done

; CS low, then high, then the START bit: DI high at a rising CLK.
start:  xor a
        ld (EEPROM),a
        ld a,CS
        ld (EEPROM),a
        ld a,CS|1
        ld (EEPROM),a
        ld a,CS|1|CLK
        ld (EEPROM),a
        ret

; B bits of HL from bit 15, each on DI at a rising CLK.
send:   add hl,hl
        ld a,CS
        adc a,0
        ld (EEPROM),a
        or CLK
        ld (EEPROM),a
        djnz send
        ret

; Sixteen bits from DO into HL, each after a rising CLK.
receive:
        ld b,16
        ld hl,0
.bit:   ld a,CS
        ld (EEPROM),a
        ld a,CS|CLK
        ld (EEPROM),a
        ld a,(EEPROM)
        rra
        adc hl,hl
        djnz .bit
        ret

; CS low, then high, and DO polled until it reads ready; the loops in DE.
wait:   xor a
        ld (EEPROM),a
        ld a,CS
        ld (EEPROM),a
        ld de,0
.poll:  inc de
        ld a,(EEPROM)
        rra
        jr nc,.poll
        ret
