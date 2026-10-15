| cddprobe, the sub CPU's half: records every drive exchange's status, sending NOP, through four phases: from
| power-on (marker $AA), after a reset of the peripherals ($BB), after one TOC request ($CC), after HOCK dropped and
| raised with no reset ($DD). A record: the stopwatch (word), $FF8037 (byte), the marker (byte), $FF8038-$FF8041 (ten
| bytes, a nibble each), $FF8036 (word). $EEEEEEEE ends the records.
    .text
    .org 0
    dc.l 0x0007FF00, start
    .rept 23
    dc.l bad
    .endr
    | 0x70: level 4 autovector (vector 28) - vectors 25..31 at 0x64..0x7C
    .org 0x64
    dc.l nothing, nothing, nothing, int4, nothing, nothing, nothing
    .rept 32
    dc.l nothing
    .endr

    .org 0x200
| a5: the record pointer; d7: records left in this phase; d6: the phase's marker
start:
    move.w #0x2700,sr
    lea 0x40000,a5
    move.w #0x00AA,d6
    bsr waitres0
    move.w #300,d7
    bsr hookon
    move.w #0x2000,sr
1:  tst.w d7
    bgt 1b
| phase B: what a reset of the peripherals does to the drive
    move.w #0x2700,sr
    move.b #0,0xFF8033
    move.b #0,0xFF8037
    move.b #0,0xFF8001
    bsr waitres0
    move.w #0x00BB,d6
    move.w #300,d7
    bsr hookon
    move.w #0x2000,sr
1:  tst.w d7
    bgt 1b
| phase C: a TOC request (report 4) then NOPs
    move.w #0x2700,sr
    move.w #0x00CC,d6
    move.w #200,d7
    move.w #0x0200,0xFF8042
    move.w #0x0004,0xFF8044
    move.w #0x0000,0xFF8046
    move.w #0x0000,0xFF8048
    move.w #0x0009,0xFF804A
    move.w #0x2000,sr
1:  tst.w d7
    bgt 1b
| phase D: HOCK dropped and raised again with no reset
    move.w #0x2700,sr
    move.b #0,0xFF8037
    move.w #0x00DD,d6
    move.w #150,d7
    move.w #2000,d0
2:  dbra d0,2b
    bsr hookon
    move.w #0x2000,sr
1:  tst.w d7
    bgt 1b
    move.w #0x2700,sr
    move.l #0xEEEEEEEE,(a5)+
    move.l #0xEEEEEEEE,(a5)+
9:  bra 9b

waitres0:
    btst #0,0xFF8001
    beq waitres0
    rts

hookon:
    move.w #0,0xFF800C
    move.w #0x0010,0xFF8032
    move.b #4,0xFF8037
    rts

int4:
    move.w 0xFF800C,(a5)+
    move.b 0xFF8037,(a5)+
    move.b d6,(a5)+
    move.l 0xFF8038,(a5)+
    move.l 0xFF803C,(a5)+
    move.w 0xFF8040,(a5)+
    move.w 0xFF8036,(a5)+
    cmp.w #0x00CC,d6
    bne 1f
    cmp.w #200,d7
    beq 2f
1:  move.w #0,0xFF8042
    move.w #0,0xFF8044
    move.w #0,0xFF8046
    move.w #0,0xFF8048
    move.w #0x000F,0xFF804A
2:  subq.w #1,d7
    rte

nothing:
    rte
bad:
    move.l #0xBADBAD00,(a5)+
    move.l #0xBADBAD00,(a5)+
8:  bra 8b
