; Nephrite's open Sega CD BIOS: the sub CPU's program, copied by the main CPU into PRG-RAM from $000000 and run
; there (Nephrite_OpenBios.md). Written from the call interface the Mega-CD BIOS Manual describes (BM, cited by
; section), with the entry addresses and function numbers of the 240p Test Suite's cdbios.asm, the registers of the
; Mega-CD Hardware Manual (HM) and the drive's and the CDC's protocol as Nephrite_Native.md §51-§52 record them.
; No other BIOS's code was read; Nephrite_OpenBios.md §9 is the reading log.

; ---- the gate array, the sub CPU's side (HM §3)

GA_LED          equ $FF8000         ; LEDs: bit 1 green, bit 0 red (byte)
GA_MEM          equ $FF8003         ; RET, DMNA, MODE (byte)
GA_CDCMODE      equ $FF8004         ; EDT, DSR, DD2-DD0 (byte)
GA_CDCRS0       equ $FF8005         ; the CDC's register address
GA_CDCDATA      equ $FF8007         ; the CDC's register data
GA_HOST         equ $FF8008         ; host data (word)
GA_SUBFLAG      equ $FF800F         ; the sub CPU's communication flag
GA_CMD0         equ $FF8010         ; the main CPU's first command word
GA_STA0         equ $FF8020         ; the sub CPU's status words
GA_MASK         equ $FF8033         ; IEN1-IEN6
GA_FADER        equ $FF8034
GA_HOCK         equ $FF8037
GA_CDDSTAT      equ $FF8038         ; ten status nibbles
GA_CDDCMD       equ $FF8042         ; ten command nibbles; the last sends them

; The CDC's registers (Sanyo's LC8951 manual §10)
CDC_IFSTAT      equ 1
CDC_IFCTRL      equ 1
CDC_DBCL        equ 2
CDC_HEAD0       equ 4
CDC_PTL         equ 8
CDC_CTRL0       equ 10
CDC_STAT3       equ 15
CDC_RESET       equ 15

; ---- the memory map (BM §1-4) and the entries (cdbios.asm)

_CDSTAT         equ $5E80           ; the status table CDBSTAT returns (BM §3-B), 32 bytes
_BOOTSTAT       equ $5EA0
_USERMODE       equ $5EA6
_SETJMPTBL      equ $5F0A
_WAITVSYNC      equ $5F10
_BURAM          equ $5F16
_CDBOOT         equ $5F1C
_CDBIOS         equ $5F22
_USERCALL0      equ $5F28
_USERCALL1      equ $5F2E
_USERCALL2      equ $5F34
_USERCALL3      equ $5F3A
_ADRERR         equ $5F40
_CODERR         equ $5F46
_DEVERR         equ $5F4C
_CHKERR         equ $5F52
_TRPERR         equ $5F58
_SPVERR         equ $5F5E
_TRACE          equ $5F64
_NOCOD0         equ $5F6A
_NOCOD1         equ $5F70
_LEVEL1         equ $5F76
_LEVEL2         equ $5F7C
_LEVEL3         equ $5F82
_LEVEL4         equ $5F88
_LEVEL5         equ $5F8E
_LEVEL6         equ $5F94
_LEVEL7         equ $5F9A
_TRAP00         equ $5FA0

USER_HEADER     equ $6000           ; the system program's header (BM §4-3)
BOOT_BUFFER     equ $40000          ; the system area while the disc boots
STACK_TOP       equ $5E80           ; the heap and stack, $5C00-$5E7F

; ---- the work area, $5400-$5BFF

TOC             equ $5400           ; tracks 1-99: mm ss ff in BCD and a byte of flags, bit 7 for data
TOCFIRST        equ $5590
TOCLAST         equ $5591
TOCNEXT         equ $5592
TOCVALID        equ $5593
LEADOUT         equ $5594           ; long: mm ss ff 00 in BCD
REQ             equ $5598           ; the drive request posted, 0 for none
REQP0           equ $559C
REQP1           equ $55A0
PHASE           equ $55A4           ; 0 idle, 1 reading the TOC
STEP            equ $55A6
PLAYMODE        equ $55A8           ; 0 on, 1 once, 2 repeat, 3 once from a seek
PLAYSTART       equ $55AA
PLAYEND         equ $55AE
STAT            equ $55B2           ; the drive's last ten status nibbles
CMDBUF          equ $55BC           ; the next ten command nibbles
HEADLBA         equ $55C6           ; the head's place, from the last absolute time reported
PAUSELIMIT      equ $55CA           ; CDBPAUSE's, in 1/75 s
PAUSECOUNT      equ $55CC
FADERVOL        equ $55CE
FADERTARGET     equ $55D0
FADERSTEP       equ $55D2
MASTERVOL       equ $55D4
ROMMODE         equ $55D6           ; 0 no read, 1 a read from a sector, 2 every sector (CDCSTART)
ROMWANT         equ $55D8
ROMLEFT         equ $55DC           ; sectors left, -1 for no end
QCOUNT          equ $55E0
QHEAD           equ $55E2           ; byte offsets into QUEUE
QTAIL           equ $55E4
QUEUE           equ $55E6           ; 5 sectors: the block pointer (word), a word spare, the header (long)
PREPARED        equ $560E           ; 0, 1 after CDCREAD, 2 after CDCTRN
CDCSETMODEV     equ $5610
WRRQOFF         equ $5612
RESYNC          equ $5614
VSYNC           equ $5616
LEDMODE         equ $5618
LEDTICK         equ $561A
ROMDONE         equ $561C
SCANPREV        equ $561E
PAUSELBA        equ $5620
STANDBY         equ $5624
DECWAIT         equ $5626
AUTOPLAY        equ $5628
LAPS            equ $562A           ; exchanges with the drive, for time-outs
LASTCMD0        equ $562E
READPKT         equ $5630           ; the boot's own ROMREADN table
HDRTMP          equ $5638
BOOTKIND        equ $563C           ; -1 not known, 0 no disc, 1 audio, 7 a game's
LASTHDR         equ $5640           ; the header of the last block the decoder reported
CALLSEEN        equ $5644           ; a bit for each function _CDBIOS has been called with: $00-$27, then $80-$9F
WORK_END        equ $5650

; ======================================================================================================================

        org     $0000
        dc.l    STACK_TOP, sub_start
        dc.l    _ADRERR, _ADRERR, _CODERR, _DEVERR, _CHKERR, _TRPERR, _SPVERR, _TRACE, _NOCOD0, _NOCOD1
        dc.l    _CODERR, _CODERR, _CODERR, _CODERR, _CODERR, _CODERR, _CODERR, _CODERR, _CODERR, _CODERR
        dc.l    _CODERR, _CODERR
        dc.l    rte_stub
        dc.l    _LEVEL1, _LEVEL2, _LEVEL3, _LEVEL4, _LEVEL5, _LEVEL6, _LEVEL7
        dc.l    _TRAP00, _TRAP00+6, _TRAP00+12, _TRAP00+18, _TRAP00+24, _TRAP00+30, _TRAP00+36, _TRAP00+42
        dc.l    _TRAP00+48, _TRAP00+54, _TRAP00+60, _TRAP00+66, _TRAP00+72, _TRAP00+78, _TRAP00+84, _TRAP00+90
        dc.l    _CODERR, _CODERR, _CODERR, _CODERR, _CODERR, _CODERR, _CODERR, _CODERR
        dc.l    _CODERR, _CODERR, _CODERR, _CODERR, _CODERR, _CODERR, _CODERR, _CODERR

        org     $0100
        dc.b    "NEPHRITE OPEN SUB-CPU BIOS 1.00"
        even

        org     $0200

; ---- reset

sub_start:
        move    #$2700,sr
        lea     STACK_TOP,sp
        lea     $5400,a0
        move.w  #(WORK_END-$5400)/2-1,d0
.clear: clr.w   (a0)+
        dbra    d0,.clear
        lea     _CDSTAT,a0
        moveq   #31,d0
.clst:  clr.b   (a0)+
        dbra    d0,.clst
        move.w  #4500,PAUSELIMIT    ; a minute, BM §3-B's lowest value for a released program
        move.w  #$400,d0
        move.w  d0,MASTERVOL
        move.w  d0,FADERVOL
        move.w  d0,FADERTARGET
        move.w  d0,FADERSTEP
        lsl.w   #4,d0
        move.w  d0,GA_FADER
        move.w  #-1,LEDMODE
        move.w  #-1,BOOTKIND
        move.l  #-1,ROMLEFT
        ; The CDC reset, its decoder and transfer interrupts enabled, its output enabled.
        move.b  #CDC_RESET,GA_CDCRS0
        move.b  #0,GA_CDCDATA
        move.b  #CDC_IFCTRL,GA_CDCRS0
        move.b  #$22,GA_CDCDATA
        bsr     decoder_off
        ; Levels 2, 4 and 5 (BM §2-1), and the drive clocked.
        move.b  #$34,GA_MASK
        move.b  #4,GA_HOCK
        move.w  #1,PHASE            ; the TOC is read at power-on
        clr.w   STEP
        move    #$2000,sr
        bra     boot

rte_stub:
        rte

rts_stub:
        rts

; A fault: both LEDs blink (BM §3-L's error) and the program stops.
sub_fatal:
        move    #$2700,sr
        move.b  #3,GA_LED
.halt:  bra     .halt

; ======================================================================================================================
; The boot (BM §4-2, §4-3; the Disc Format Specifications, appendix 2)

boot:
        moveq   #0,d0
        bsr     set_state
.wait:  bsr     poll_main
        tst.b   TOCVALID
        bne     .toc
        move.w  _CDSTAT,d1
        moveq   #1,d0
        btst    #14,d1
        bne     .show
        moveq   #2,d0
        btst    #12,d1
        bne     .show
        moveq   #3,d0
.show:  bsr     set_state
        bra     .wait

.toc:   moveq   #3,d0
        bsr     set_state
        moveq   #0,d0
        move.b  TOCFIRST,d0
        lsl.w   #2,d0
        lea     TOC,a0
        tst.b   3(a0,d0.w)          ; track 1's flags: a data track?
        bmi     .data
        move.w  #1,BOOTKIND
        moveq   #4,d0
        bra     .refused

.data:  moveq   #0,d0
        moveq   #1,d1
        lea     BOOT_BUFFER,a0
        bsr     read_sectors
        bcs     .wait
        lea     BOOT_BUFFER,a0
        lea     disc_mark(pc),a1
        moveq   #13,d0
.mark:  cmpm.b  (a0)+,(a1)+
        bne     .other
        dbra    d0,.mark
        move.w  #7,BOOTKIND
        bsr     load_ip
        bcs     .wait
        bsr     load_sp
        bcs     .wait
        bra     run_user

.other: moveq   #5,d0
        clr.w   BOOTKIND
.refused:
        bsr     set_state
        ; Nothing to boot: wait for the tray to open and the disc to change.
.held:  bsr     poll_main
        tst.b   TOCVALID
        bne     .held
        bra     .wait

disc_mark:
        dc.b    "SEGADISCSYSTEM"
        even

; The state the main CPU shows: 0 starting, 1 tray open, 2 no disc, 3 reading, 4 an audio disc, 5 not a Sega CD
; disc, 6 the initial program in Word RAM.
set_state:
        move.w  d0,GA_STA0
        move.w  d0,_BOOTSTAT
        rts

; The main CPU's requests while the boot waits: a new number in the high byte of its first command word, and 1 in
; its low byte to open the tray or close it.
poll_main:
        move.w  GA_CMD0,d0
        cmp.w   LASTCMD0,d0
        beq     .none
        move.w  d0,LASTCMD0
        cmp.b   #1,d0
        bne     .none
        move.w  #$0A,d0             ; DRVOPEN
        btst    #6,_CDSTAT
        beq     .post
        lea     drvinit_all(pc),a0
        move.w  #$10,d0             ; DRVINIT
.post:  jsr     _CDBIOS
.none:  rts

drvinit_all:
        dc.b    $01,$FF

; The initial program: the system area's bytes $200-$7FF, then the rest from its disc address, into Word RAM for the
; main CPU (the Technical Bulletins' "The U.S. BIOS" and boot diagram).
load_ip:
        lea     BOOT_BUFFER+$200,a0
        lea     $80000,a1
        move.w  #$600/4-1,d0
.first: move.l  (a0)+,(a1)+
        dbra    d0,.first
        move.l  #$600,d2            ; bytes of the program
        move.l  BOOT_BUFFER+$30,d0  ; its disc address
        move.l  BOOT_BUFFER+$34,d3  ; its size
        cmp.l   #$800,d0
        bcs     .given
        tst.l   d3
        beq     .given
        move.l  d0,d4
        and.l   #$7FF,d4
        lsr.l   #8,d0
        lsr.l   #3,d0
        move.l  d4,d1
        add.l   d3,d1
        add.l   #$7FF,d1
        lsr.l   #8,d1
        lsr.l   #3,d1
        lea     BOOT_BUFFER+$1000,a0
        bsr     read_sectors
        bcs     .fail
        lea     BOOT_BUFFER+$1000,a0
        add.l   d4,a0
        lea     $80600,a1
        move.l  d3,d0
.rest:  move.b  (a0)+,(a1)+
        subq.l  #1,d0
        bne     .rest
        add.l   d3,d2
.given: move.b  #1,GA_MEM           ; Word RAM back to the main CPU (HM §3-1, RET)
        move.w  d2,GA_STA0+2
        move.w  BOOT_BUFFER+$3A,GA_STA0+4
        moveq   #6,d0
        bsr     set_state
        ; The hand-over: the main CPU acknowledges with $FFFF in its first command word, the statuses are cleared, and
        ; the main CPU clears its commands, so that the program starts with every communication word 0.
.ack:   cmp.w   #-1,GA_CMD0
        bne     .ack
        lea     GA_STA0,a0
        moveq   #7,d0
.sta:   clr.w   (a0)+
        dbra    d0,.sta
.clr:   tst.w   GA_CMD0
        bne     .clr
        andi    #$FE,ccr
        rts
.fail:  ori     #1,ccr
        rts

; The system program into PRG-RAM at $6000, and its user calls into the jump table (BM §4-3, §5-3).
load_sp:
        move.l  BOOT_BUFFER+$40,d0
        move.l  BOOT_BUFFER+$44,d1
        tst.l   d1
        beq     .none
        lsr.l   #8,d0
        lsr.l   #3,d0
        add.l   #$7FF,d1
        lsr.l   #8,d1
        lsr.l   #3,d1
        lea     USER_HEADER,a0
        bsr     read_sectors
        bcs     .fail
        move.l  USER_HEADER+$18,d0  ; the user call table's offset
        lea     USER_HEADER,a0
        add.l   d0,a0
        move.l  a0,a3
        lea     _USERCALL0,a1
        moveq   #3,d1
.call:  move.w  (a0)+,d0
        beq     .none
        move.l  a3,a2
        add.w   d0,a2
        move.w  #$4EF9,(a1)+
        move.l  a2,(a1)+
        dbra    d1,.call
.none:  andi    #$FE,ccr
        rts
.fail:  ori     #1,ccr
        rts

; The user program's turn (BM §5-3): its initialisation before level 2 is enabled, then its main routine, called with
; _USERMODE and again after each level 2.
run_user:
        lea     CALLSEEN,a0         ; from here the calls are the program's
        moveq   #9,d0
.seen:  clr.b   (a0)+
        dbra    d0,.seen
        clr.b   GA_SUBFLAG
        bclr    #2,GA_MASK
        jsr     _USERCALL0
        bset    #2,GA_MASK
.main:  move.w  _USERMODE,d0
        jsr     _USERCALL1
        bsr     wait_vsync
        bra     .main

; Waits for the next level 2.
wait_vsync:
        move.w  d0,-(sp)
        move.w  VSYNC,d0
.w:     cmp.w   VSYNC,d0
        beq     .w
        move.w  (sp)+,d0
        rts

; d1 sectors from logical sector d0 to a0, through the calls a program makes (BM p. 44-45); carry set when the drive
; gives nothing for ten seconds.
read_sectors:
        movem.l d2-d4/a2,-(sp)
        move.l  a0,a2
        move.l  d1,d4
        lea     READPKT,a0
        move.l  d0,(a0)
        move.l  d1,4(a0)
        move.w  #$20,d0             ; ROMREADN
        jsr     _CDBIOS
        move.b  #3,GA_CDCMODE       ; the sub CPU reads the host data
.next:  move.l  LAPS,d3
        add.l   #750,d3
.stat:  cmp.l   LAPS,d3
        bcs     .late
        move.w  #$8A,d0             ; CDCSTAT
        jsr     _CDBIOS
        bcs     .stat
        move.w  #$8B,d0             ; CDCREAD
        jsr     _CDBIOS
        bcs     .stat
        move.l  a2,a0
        lea     HDRTMP,a1
        move.w  #$8C,d0             ; CDCTRN
        jsr     _CDBIOS
        bcs     .stat
        move.l  a0,a2
        move.w  #$8D,d0             ; CDCACK
        jsr     _CDBIOS
        subq.l  #1,d4
        bne     .next
        movem.l (sp)+,d2-d4/a2
        andi    #$FE,ccr
        rts
.late:  move.w  #$89,d0             ; CDCSTOP
        jsr     _CDBIOS
        movem.l (sp)+,d2-d4/a2
        ori     #1,ccr
        rts

; ======================================================================================================================
; _CDBIOS (BM §2-1 "Call Method", §3): the function in d0.w; d0, d1, a0 and a1 not kept; drive, CD-DA and CD-ROM
; calls posted for the level 4 exchange, the others done at once.

cdbios:
        movem.l d2-d7/a2-a6,-(sp)
        cmp.w   #$22,d0
        bcs     .low
        sub.w   #$80,d0
        bcs     .none
        cmp.w   #$19,d0
        bcc     .none
        move.w  d0,d2
        lsr.w   #3,d2
        lea     CALLSEEN+5,a2
        bset    d0,0(a2,d2.w)
        add.w   d0,d0
        lea     high_calls(pc),a2
        move.w  0(a2,d0.w),d2
        jsr     0(a2,d2.w)
        movem.l (sp)+,d2-d7/a2-a6
        rts
.low:   move.w  d0,d2
        lsr.w   #3,d2
        lea     CALLSEEN,a2
        bset    d0,0(a2,d2.w)
        add.w   d0,d0
        lea     low_calls(pc),a2
        move.w  0(a2,d0.w),d2
        jsr     0(a2,d2.w)
        movem.l (sp)+,d2-d7/a2-a6
        rts
.none:  andi    #$FE,ccr
        movem.l (sp)+,d2-d7/a2-a6
        rts

low_calls:
        dc.w    call_none-low_calls     ; $00
        dc.w    call_none-low_calls     ; $01
        dc.w    post_plain-low_calls    ; $02 MSCSTOP
        dc.w    post_plain-low_calls    ; $03 MSCPAUSEON
        dc.w    post_plain-low_calls    ; $04 MSCPAUSEOFF
        dc.w    post_plain-low_calls    ; $05 MSCSCANFF
        dc.w    post_plain-low_calls    ; $06 MSCSCANFR
        dc.w    post_plain-low_calls    ; $07 MSCSCANOFF
        dc.w    post_plain-low_calls    ; $08 ROMPAUSEON
        dc.w    post_plain-low_calls    ; $09 ROMPAUSEOFF
        dc.w    post_plain-low_calls    ; $0A DRVOPEN
        dc.w    call_none-low_calls     ; $0B
        dc.w    call_none-low_calls     ; $0C
        dc.w    call_none-low_calls     ; $0D
        dc.w    call_none-low_calls     ; $0E
        dc.w    call_none-low_calls     ; $0F
        dc.w    post_drvinit-low_calls  ; $10 DRVINIT
        dc.w    post_word-low_calls     ; $11 MSCPLAY
        dc.w    post_word-low_calls     ; $12 MSCPLAY1
        dc.w    post_word-low_calls     ; $13 MSCPLAYR
        dc.w    post_long-low_calls     ; $14 MSCPLAYT
        dc.w    post_word-low_calls     ; $15 MSCSEEK
        dc.w    post_long-low_calls     ; $16 MSCSEEKT
        dc.w    post_read-low_calls     ; $17 ROMREAD
        dc.w    post_long-low_calls     ; $18 ROMSEEK
        dc.w    post_word-low_calls     ; $19 MSCSEEK1
        dc.w    call_none-low_calls     ; $1A
        dc.w    call_none-low_calls     ; $1B
        dc.w    call_none-low_calls     ; $1C
        dc.w    call_none-low_calls     ; $1D
        dc.w    call_none-low_calls     ; $1E TESTENTRY
        dc.w    call_none-low_calls     ; $1F TESTENTRYLOOP
        dc.w    post_read-low_calls     ; $20 ROMREADN
        dc.w    post_read-low_calls     ; $21 ROMREADE

high_calls:
        dc.w    call_cdbchk-high_calls  ; $80 CDBCHK
        dc.w    call_cdbstat-high_calls ; $81 CDBSTAT
        dc.w    call_tocwrite-high_calls ; $82 CDBTOCWRITE
        dc.w    call_tocread-high_calls ; $83 CDBTOCREAD
        dc.w    call_cdbpause-high_calls ; $84 CDBPAUSE
        dc.w    call_fdrset-high_calls  ; $85 FDRSET
        dc.w    call_fdrchg-high_calls  ; $86 FDRCHG
        dc.w    call_cdcstart-high_calls ; $87 CDCSTART
        dc.w    call_cdcstart-high_calls ; $88 CDCSTARTP
        dc.w    call_cdcstop-high_calls ; $89 CDCSTOP
        dc.w    call_cdcstat-high_calls ; $8A CDCSTAT
        dc.w    call_cdcread-high_calls ; $8B CDCREAD
        dc.w    call_cdctrn-high_calls  ; $8C CDCTRN
        dc.w    call_cdcack-high_calls  ; $8D CDCACK
        dc.w    call_none-high_calls    ; $8E SCDINIT
        dc.w    call_none-high_calls    ; $8F SCDSTART
        dc.w    call_none-high_calls    ; $90 SCDSTOP
        dc.w    call_scdstat-high_calls ; $91 SCDSTAT
        dc.w    call_null-high_calls    ; $92 SCDREAD
        dc.w    call_null-high_calls    ; $93 SCDPQ
        dc.w    call_null-high_calls    ; $94 SCDPQL
        dc.w    call_ledset-high_calls  ; $95 LEDSET
        dc.w    call_setmode-high_calls ; $96 CDCSETMODE
        dc.w    call_none-high_calls    ; $97 WONDERREQ
        dc.w    call_none-high_calls    ; $98 WONDERCHK

call_none:
        andi    #$FE,ccr
        rts

call_null:
        ori     #1,ccr
        rts

; A call for the drive: its number last, so that level 4 never takes one half written. d0 holds the number doubled.
post_plain:
        lsr.w   #1,d0
        move.w  d0,REQ
        andi    #$FE,ccr
        rts

post_drvinit:
        lsr.w   #1,d0
        moveq   #0,d1
        move.b  (a0),d1
        move.l  d1,REQP0
        move.w  d0,REQ
        andi    #$FE,ccr
        rts

post_word:
        lsr.w   #1,d0
        moveq   #0,d1
        move.w  (a0),d1
        move.l  d1,REQP0
        move.w  d0,REQ
        andi    #$FE,ccr
        rts

post_long:
        lsr.w   #1,d0
        move.l  (a0),REQP0
        move.w  d0,REQ
        andi    #$FE,ccr
        rts

; A read: the CDC's queue emptied now, so that no sector of an earlier read is taken for this one (BM §3-R).
post_read:
        lsr.w   #1,d0
        move.l  (a0),REQP0
        move.l  4(a0),REQP1
        move    sr,-(sp)
        move    #$2700,sr
        bsr     queue_flush
        bsr     decoder_off
        move.w  #1,ROMMODE
        move.l  REQP0,ROMWANT
        move.l  #-1,ROMLEFT
        cmp.w   #$20,d0
        bne     .e
        move.l  REQP1,ROMLEFT
.e:     cmp.w   #$21,d0
        bne     .post
        move.l  REQP1,d1
        sub.l   REQP0,d1
        addq.l  #1,d1
        move.l  d1,ROMLEFT
.post:  move.w  d0,REQ
        move    (sp)+,sr
        andi    #$FE,ccr
        rts

call_cdbchk:
        tst.w   REQ
        bne     call_null
        andi    #$FE,ccr
        rts

call_cdbstat:
        bsr     compose_status
        lea     _CDSTAT,a0
        andi    #$FE,ccr
        rts

; A TOC table of mm ss ff tno in BCD, a frame's bit 7 for data, written over the TOC read from the disc (BM §3-B).
; The table ends at a long of $FFFFFFFF, an end the manual names ENDOFTOCTBL without its value (Nephrite_OpenBios.md).
call_tocwrite:
.next:  move.l  (a0)+,d1
        cmp.l   #-1,d1
        beq     .end
        moveq   #0,d0
        move.b  d1,d0
        bsr     bcd_to_bin
        tst.w   d0
        beq     .next
        cmp.w   #99,d0
        bhi     .next
        cmp.b   TOCFIRST,d0
        bcc     .f
        move.b  d0,TOCFIRST
.f:     cmp.b   TOCLAST,d0
        bls     .l
        move.b  d0,TOCLAST
.l:     lea     TOC,a1
        lsl.w   #2,d0
        add.w   d0,a1
        swap    d1
        move.w  d1,(a1)+            ; mm ss
        swap    d1
        lsr.w   #8,d1
        move.b  d1,d2
        and.b   #$7F,d1
        move.b  d1,(a1)+
        and.b   #$80,d2
        move.b  d2,(a1)
        bra     .next
.end:   move.b  #1,TOCVALID
        andi    #$FE,ccr
        rts

; d1.w a track: d0.l its start as mm ss ff tt in BCD and d1.b $FF for data, $00 for music; tt 00 for a track not in
; the TOC, the time then the disc's start (BM §3-B).
call_tocread:
        moveq   #0,d0
        tst.b   TOCVALID
        beq     .bad
        cmp.b   TOCFIRST,d1
        bcs     .bad
        cmp.b   TOCLAST,d1
        bhi     .bad
        lea     TOC,a0
        move.w  d1,d2
        lsl.w   #2,d2
        move.l  0(a0,d2.w),d0
        move.b  d0,d3               ; the flags
        move.w  d1,d0
        and.w   #$FF,d0
        bsr     bin_to_bcd
        move.l  0(a0,d2.w),d2
        clr.b   d2
        or.b    d0,d2
        move.l  d2,d0
        moveq   #0,d1
        tst.b   d3
        bpl     .ok
        moveq   #-1,d1
.ok:    andi    #$FE,ccr
        rts
.bad:   move.l  #$00020000,d0
        moveq   #0,d1
        andi    #$FE,ccr
        rts

call_cdbpause:
        move.w  d1,PAUSELIMIT
        andi    #$FE,ccr
        rts

; The fader (BM §3-F): bit 15 the master volume, else the volume, which the master's caps; taken at the next exchange.
call_fdrset:
        move.w  d1,d0
        and.w   #$7FFF,d0
        cmp.w   #$400,d0
        bls     .r
        move.w  #$400,d0
.r:     tst.w   d1
        bpl     .vol
        move.w  d0,MASTERVOL
        cmp.w   FADERTARGET,d0
        bcc     .ok
        move.w  d0,FADERTARGET
        bra     .ok
.vol:   cmp.w   MASTERVOL,d0
        bls     .set
        move.w  MASTERVOL,d0
.set:   move.w  d0,FADERTARGET
        move.w  #$400,FADERSTEP
.ok:    andi    #$FE,ccr
        rts

; d1.l the volume to reach (high word) and the step an exchange (low word).
call_fdrchg:
        move.w  d1,FADERSTEP
        swap    d1
        and.w   #$7FF,d1
        cmp.w   MASTERVOL,d1
        bls     .s
        move.w  MASTERVOL,d1
.s:     move.w  d1,FADERTARGET
        andi    #$FE,ccr
        rts

; The CDC's calls (BM §3-C).
call_cdcstart:
        move    sr,-(sp)
        move    #$2700,sr
        bsr     queue_flush
        move.w  #2,ROMMODE
        bsr     decoder_on
        move    (sp)+,sr
        andi    #$FE,ccr
        rts

call_cdcstop:
        move    sr,-(sp)
        move    #$2700,sr
        bsr     decoder_off
        clr.w   ROMMODE
        clr.w   DECWAIT
        clr.w   RESYNC
        move    (sp)+,sr
        andi    #$FE,ccr
        rts

call_cdcstat:
        tst.w   QCOUNT
        beq     call_null
        andi    #$FE,ccr
        rts

; The sector at the queue's head handed to the destination the caller set: its data from the header on, the header in
; d0. Once handed and not yet acknowledged, carry set.
call_cdcread:
        tst.w   QCOUNT
        beq     call_null
        tst.w   PREPARED
        bne     call_null
        lea     QUEUE,a0
        move.w  QHEAD,d2
        move.w  0(a0,d2.w),d1       ; the block pointer: the header
        move.l  4(a0,d2.w),d0
        addq.w  #4,d1
        and.w   #$3FFF,d1
        bsr     sector_bytes
        subq.w  #1,d3
        move.b  #CDC_DBCL,GA_CDCRS0
        move.b  d3,GA_CDCDATA
        lsr.w   #8,d3
        move.b  d3,GA_CDCDATA
        move.b  d1,GA_CDCDATA
        lsr.w   #8,d1
        move.b  d1,GA_CDCDATA
        move.b  #0,GA_CDCDATA        ; DTTRG (a CLR would read first, and the read moves the address on)
        move.w  #1,PREPARED
        move.l  d0,_CDSTAT+28
        andi    #$FE,ccr
        rts

; The bytes of a sector's transfer: 2,048 in mode 1, 2,336 in mode 2 (CDCSETMODE's bit 0).
sector_bytes:
        move.w  #2048,d3
        btst    #0,CDCSETMODEV+1
        beq     .r
        move.w  #2336,d3
.r:     rts

; The prepared sector read by the sub CPU through the host data register to a0, its header to a1.
call_cdctrn:
        cmp.w   #1,PREPARED
        bne     call_null
        move.b  GA_CDCMODE,d0
        and.b   #7,d0
        cmp.b   #3,d0
        bne     call_null
        bsr     sector_bytes
        lsr.w   #1,d3
        subq.w  #1,d3
.word:  btst    #6,GA_CDCMODE       ; DSR
        beq     .word
        move.w  GA_HOST,(a0)+
        dbra    d3,.word
        lea     QUEUE,a2
        move.w  QHEAD,d2
        move.l  4(a2,d2.w),(a1)+
        move.w  #2,PREPARED
        andi    #$FE,ccr
        rts

call_cdcack:
        move    sr,-(sp)
        move    #$2700,sr
        tst.w   QCOUNT
        beq     .x
        move.w  QHEAD,d0
        addq.w  #8,d0
        cmp.w   #40,d0
        bcs     .h
        moveq   #0,d0
.h:     move.w  d0,QHEAD
        subq.w  #1,QCOUNT
.x:     clr.w   PREPARED
        tst.w   WRRQOFF
        beq     .r
        clr.w   WRRQOFF
        tst.w   ROMMODE
        beq     .r
        bsr     decoder_on
.r:     move    (sp)+,sr
        andi    #$FE,ccr
        rts

call_scdstat:
        moveq   #0,d0
        moveq   #0,d1
        andi    #$FE,ccr
        rts

; d1.w a mode of BM §3-L, 0-7; any other gives the LEDs back to the system.
call_ledset:
        cmp.w   #8,d1
        bcs     .m
        moveq   #-1,d1
.m:     move.w  d1,LEDMODE
        andi    #$FE,ccr
        rts

call_setmode:
        move.w  d1,CDCSETMODEV
        andi    #$FE,ccr
        rts

; ---- the queue of sectors decoded and not yet acknowledged, and the decoder

queue_flush:
        clr.w   QCOUNT
        clr.w   QHEAD
        clr.w   QTAIL
        clr.w   PREPARED
        clr.w   WRRQOFF
        clr.w   RESYNC
        clr.w   ROMDONE
        rts

; CTRL0: DECEN and WRRQ, decoding into the buffer; DECEN alone, decoding and writing nothing; or off.
decoder_on:
        move.b  #CDC_CTRL0,GA_CDCRS0
        move.b  #$84,GA_CDCDATA
        move.b  #0,GA_CDCDATA
        rts

decoder_hold:
        move.b  #CDC_CTRL0,GA_CDCRS0
        move.b  #$80,GA_CDCDATA
        move.b  #0,GA_CDCDATA
        rts

decoder_off:
        move.b  #CDC_CTRL0,GA_CDCRS0
        move.b  #0,GA_CDCDATA
        move.b  #0,GA_CDCDATA
        rts

; ======================================================================================================================
; Level 5, the CDC: each sector decoded, its header and block pointer; a read's sector queued in order, one missed
; asked for again (BM §3-R, §3-C).

level5:
        movem.l d0-d3/a0,-(sp)
        move.b  #CDC_IFSTAT,GA_CDCRS0
        move.b  GA_CDCDATA,d0
        btst    #5,d0               ; DECI, set at 0
        bne     .done
        move.b  #CDC_HEAD0,GA_CDCRS0
        moveq   #3,d2
.head:  lsl.l   #8,d1
        move.b  GA_CDCDATA,d1
        dbra    d2,.head
        moveq   #0,d3
        move.b  GA_CDCDATA,d3       ; PTL
        move.b  GA_CDCDATA,d2       ; PTH
        lsl.w   #8,d2
        or.w    d2,d3
        move.b  #CDC_STAT3,GA_CDCRS0
        move.b  GA_CDCDATA,d0       ; clears DECI
        ; A decoder interrupt with no new block (the head travelling in a seek) repeats the last header: not a sector.
        cmp.l   LASTHDR,d1
        beq     .done
        move.l  d1,LASTHDR
        move.l  d1,_CDSTAT+28
        move.w  ROMMODE,d0
        beq     .done
        cmp.w   #2,d0
        beq     .take
        tst.w   DECWAIT
        bne     .done
        tst.l   ROMLEFT
        beq     .done
        move.l  d1,d0
        bsr     msf_to_lba
        cmp.l   ROMWANT,d0
        blt     .done
        bgt     .missed
.take:  cmp.w   #5,QCOUNT
        bcc     .full
        lea     QUEUE,a0
        move.w  QTAIL,d2
        move.w  d3,0(a0,d2.w)
        move.l  d1,4(a0,d2.w)
        addq.w  #8,d2
        cmp.w   #40,d2
        bcs     .t
        moveq   #0,d2
.t:     move.w  d2,QTAIL
        addq.w  #1,QCOUNT
        cmp.w   #2,ROMMODE
        beq     .done
        addq.l  #1,ROMWANT
        tst.l   ROMLEFT
        bmi     .done
        subq.l  #1,ROMLEFT
        bne     .done
        move.w  #1,ROMDONE
        bra     .done
.full:  bsr     decoder_hold
        move.w  #1,WRRQOFF
        cmp.w   #1,ROMMODE
        bne     .done
.missed:
        move.w  #1,RESYNC
.done:  movem.l (sp)+,d0-d3/a0
        rte

; ======================================================================================================================
; Level 4, the drive's exchange every 1/75 s (BM §2-1, §2-4): the status read, the posted call or the drive's own
; course taken, the next command sent; the fader and the LEDs moved on.

level4:
        movem.l d0-d7/a0-a6,-(sp)
        addq.l  #1,LAPS
        lea     CMDBUF,a0
        moveq   #9,d0
.clr:   clr.b   (a0)+
        dbra    d0,.clr
        bsr     cdd_read
        bne     .send
        bsr     cdd_update
        cmp.w   #1,PHASE
        bne     .idle
        bsr     toc_step
        bra     .send
.idle:  bsr     idle_step
.send:  bsr     cdd_send
        bsr     fader_step
        bsr     compose_status
        bsr     led_step
        movem.l (sp)+,d0-d7/a0-a6
        rte

; The ten status nibbles into STAT; Z set when their checksum holds.
cdd_read:
        lea     GA_CDDSTAT,a0
        lea     STAT,a1
        moveq   #0,d1
        moveq   #8,d2
.n:     move.b  (a0)+,d0
        and.b   #15,d0
        move.b  d0,(a1)+
        add.b   d0,d1
        dbra    d2,.n
        move.b  (a0),d0
        and.b   #15,d0
        move.b  d0,(a1)
        not.b   d1
        and.b   #15,d1
        cmp.b   d0,d1
        rts

; CMDBUF to the drive with its checksum; the tenth nibble sends it.
cdd_send:
        lea     CMDBUF,a0
        lea     GA_CDDCMD,a1
        moveq   #0,d1
        moveq   #8,d2
.n:     move.b  (a0)+,d0
        add.b   d0,d1
        move.b  d0,(a1)+
        dbra    d2,.n
        not.b   d1
        and.b   #15,d1
        move.b  d1,(a1)
        rts

; What the status reports, into the status table (BM §3-B): BP0-BP3, the times and the flags; the head's place from
; the absolute time.
cdd_update:
        lea     _CDSTAT+4,a1
        move.b  STAT,(a1)
        move.b  STAT+1,1(a1)
        move.b  STAT+8,15(a1)
        move.b  STAT+1,d0
        bne     .rel
        cmp.b   #1,STAT             ; a place only while the head reads
        beq     .abs
        cmp.b   #4,STAT
        bne     .r
.abs:   lea     STAT+2,a0
        bsr     nibbles_bcd
        move.l  d0,4(a1)
        bsr     msf_to_lba
        move.l  d0,HEADLBA
        rts
.rel:   cmp.b   #1,d0
        bne     .trk
        lea     STAT+2,a0
        bsr     nibbles_bcd
        move.l  d0,8(a1)
        rts
.trk:   cmp.b   #2,d0
        bne     .r
        move.b  STAT+2,d0
        lsl.b   #4,d0
        or.b    STAT+3,d0
        bsr     bcd_to_bin
        move.b  d0,3(a1)
.r:     rts

; Six nibbles at a0 as mm ss ff 00 in BCD, in d0.
nibbles_bcd:
        moveq   #0,d0
        moveq   #2,d2
.p:     move.b  (a0)+,d1
        lsl.b   #4,d1
        or.b    (a0)+,d1
        lsl.l   #8,d0
        move.b  d1,d0
        dbra    d2,.p
        lsl.l   #8,d0
        rts

; Reading the TOC (BM §3-D DRVINIT): the drive spun up, the disc's length, its first and last tracks, then each
; track's start, each a report the drive gives (Nephrite_Native.md §51).
toc_step:
        move.b  STAT,d0
        cmp.b   #5,d0               ; the tray open: nothing to read
        beq     .none
        cmp.b   #$B,d0
        beq     .none
        move.w  STEP,d1
        bne     .s1
        cmp.b   #$F,STAT+1          ; still busy
        beq     .r
        moveq   #3,d0
        bsr     cmd_report
        move.w  #1,STEP
.r:     rts
.none:  clr.b   TOCVALID
        clr.w   PHASE
        rts
.s1:    cmp.w   #1,d1
        bne     .s2
        cmp.b   #3,STAT+1
        bne     .r
        lea     STAT+2,a0
        bsr     nibbles_bcd
        move.l  d0,LEADOUT
        move.l  d0,_CDSTAT+20
        moveq   #4,d0
        bsr     cmd_report
        move.w  #2,STEP
        rts
.s2:    cmp.w   #2,d1
        bne     .s3
        cmp.b   #4,STAT+1
        bne     .r
        move.b  STAT+2,d0
        lsl.b   #4,d0
        or.b    STAT+3,d0
        bsr     bcd_to_bin
        move.b  d0,TOCFIRST
        move.b  d0,TOCNEXT
        move.b  d0,_CDSTAT+16
        move.b  STAT+4,d0
        lsl.b   #4,d0
        or.b    STAT+5,d0
        bsr     bcd_to_bin
        move.b  d0,TOCLAST
        move.b  d0,_CDSTAT+17
        move.w  #3,STEP
        bra     ask_track
.s3:    cmp.b   #5,STAT+1
        bne     .r
        moveq   #0,d0
        move.b  TOCNEXT,d0
        bsr     bin_to_bcd
        and.b   #15,d0
        cmp.b   STAT+8,d0
        bne     .r
        moveq   #0,d0
        move.b  TOCNEXT,d0
        lsl.w   #2,d0
        lea     TOC,a1
        add.w   d0,a1
        move.b  STAT+2,d0
        lsl.b   #4,d0
        or.b    STAT+3,d0
        move.b  d0,(a1)+
        move.b  STAT+4,d0
        lsl.b   #4,d0
        or.b    STAT+5,d0
        move.b  d0,(a1)+
        move.b  STAT+6,d0
        move.b  d0,d1
        and.b   #7,d0
        lsl.b   #4,d0
        or.b    STAT+7,d0
        move.b  d0,(a1)+
        and.b   #8,d1               ; the data flag in the frame's tens
        lsl.b   #4,d1
        move.b  d1,(a1)
        move.b  TOCNEXT,d0
        addq.b  #1,d0
        move.b  d0,TOCNEXT
        cmp.b   TOCLAST,d0
        bls     ask_track
        move.b  #1,TOCVALID
        clr.w   PHASE
        tst.w   AUTOPLAY
        beq     .r
        clr.w   AUTOPLAY
        moveq   #0,d0
        move.b  TOCFIRST,d0
        move.l  d0,REQP0
        move.w  #$11,REQ            ; MSCPLAY from the first track (DRVINIT's bit 7)
        rts

ask_track:
        moveq   #0,d0
        move.b  TOCNEXT,d0
        bsr     bin_to_bcd
        lea     CMDBUF,a0
        move.b  #2,(a0)
        move.b  #5,3(a0)
        move.b  d0,d1
        lsr.b   #4,d1
        move.b  d1,4(a0)
        and.b   #15,d0
        move.b  d0,5(a0)
        rts

; Command 2, a report of kind d0.
cmd_report:
        move.b  #2,CMDBUF
        move.b  d0,CMDBUF+3
        rts

; Command d0 (3 play, 4 seek) at logical sector d1, its absolute time in BCD nibbles.
cmd_at:
        lea     CMDBUF,a0
        move.b  d0,(a0)
        add.l   #150,d1
        bpl     .p
        moveq   #0,d1
.p:     divu    #4500,d1
        move.w  d1,d0               ; minutes
        clr.w   d1
        swap    d1
        divu    #75,d1
        move.l  d1,d2
        swap    d2                  ; frames
        bsr     bin_to_bcd
        bsr     .put
        move.w  d1,d0               ; seconds
        bsr     bin_to_bcd
        bsr     .put
        move.w  d2,d0
        bsr     bin_to_bcd
.put:   addq.l  #2,a0
        move.b  d0,d3
        lsr.b   #4,d3
        move.b  d3,(a0)
        and.b   #15,d0
        move.b  d0,1(a0)
        rts

; The drive's course between calls, then a posted call taken.
idle_step:
        move.b  STAT,d0
        cmp.b   #5,d0
        beq     .gone
        cmp.b   #$B,d0
        bne     .there
.gone:  clr.b   TOCVALID
        bra     .req
.there: tst.b   TOCVALID
        bne     .req
        move.w  #1,PHASE            ; a disc in a closed tray: read its TOC
        clr.w   STEP
        rts
.req:   move.w  REQ,d0
        beq     .course
        clr.w   REQ
        bra     take_call
.course:
        ; A read: the decoder on once the drive seeks; a sector asked for again; the end of a counted read.
        tst.w   DECWAIT
        beq     .rs
        cmp.b   #2,STAT
        bne     .rs
        clr.w   DECWAIT
        bsr     decoder_on
.rs:    tst.w   RESYNC
        beq     .done
        cmp.w   #5,QCOUNT
        bcc     .done
        clr.w   RESYNC
        cmp.w   #1,ROMMODE
        bne     .done
        bsr     read_from_want
        rts
.done:  tst.w   ROMDONE
        beq     .music
        clr.w   ROMDONE
        bsr     decoder_off
        move.b  #6,CMDBUF           ; a counted read ends paused (BM §3-R ROMREADN)
        rts
.music: move.b  STAT,d0
        cmp.b   #1,d0
        bne     .seek1
        move.w  PLAYMODE,d1
        beq     .lead
        cmp.w   #3,d1
        beq     .lead
        move.l  HEADLBA,d2
        cmp.l   PLAYEND,d2
        blt     .lead
        cmp.w   #1,d1
        bne     .again
        clr.w   PLAYMODE
        move.b  #6,CMDBUF           ; MSCPLAY1: a track played once, then the pause
        rts
.again: moveq   #3,d0
        move.l  PLAYSTART,d1
        bra     cmd_at              ; MSCPLAYR: the track again
.seek1: cmp.b   #4,d0
        bne     .lead
        cmp.w   #3,PLAYMODE
        bne     .pause
        move.w  #1,PLAYMODE
        move.b  #7,CMDBUF           ; MSCSEEK1: paused at the track, then played once
        rts
.pause: addq.w  #1,PAUSECOUNT       ; CDBPAUSE: a pause held too long stops the drive
        move.w  PAUSELIMIT,d1
        cmp.w   #-1,d1
        beq     .r
        cmp.w   PAUSECOUNT,d1
        bcc     .r
        clr.w   PAUSECOUNT
        move.w  #1,STANDBY
        move.l  HEADLBA,PAUSELBA
        move.b  #1,CMDBUF
        rts
.lead:  clr.w   PAUSECOUNT
        cmp.b   #$C,STAT            ; the lead-out: the drive stops
        bne     .r
        move.b  #1,CMDBUF
.r:     rts

read_from_want:
        move.w  #1,DECWAIT
        bsr     decoder_off
        moveq   #3,d0
        move.l  ROMWANT,d1
        subq.l  #2,d1
        bra     cmd_at

; A posted call, d0 its number.
take_call:
        clr.w   STANDBY
        clr.w   PAUSECOUNT
        cmp.w   #$10,d0
        beq     .init
        cmp.w   #$0A,d0
        bne     .music
        clr.b   TOCVALID
        move.b  #$D,CMDBUF          ; DRVOPEN
        rts
.init:  move.l  REQP0,d1
        btst    #7,d1
        sne     d1
        and.w   #1,d1
        move.w  d1,AUTOPLAY
        move.w  #1,PHASE
        clr.w   STEP
        cmp.b   #5,STAT
        bne     .r
        move.b  #$C,CMDBUF          ; the tray closed first
.r:     rts
.music: cmp.w   #$08,d0
        beq     .rpause
        cmp.w   #$09,d0
        beq     .rresume
        cmp.w   #$17,d0
        beq     .read
        cmp.w   #$20,d0
        beq     .read
        cmp.w   #$21,d0
        beq     .read
        ; every other call stops a read
        move.w  d0,d7
        bsr     decoder_off
        clr.w   ROMMODE
        clr.w   DECWAIT
        move.w  d7,d0
        cmp.w   #$02,d0
        bne     .p1
        clr.w   PLAYMODE
        move.b  #1,CMDBUF
        rts
.p1:    cmp.w   #$03,d0
        bne     .p2
        move.b  #6,CMDBUF
        rts
.p2:    cmp.w   #$04,d0
        bne     .p3
        tst.w   STANDBY
        bne     .wake
        cmp.b   #0,STAT
        beq     .wake
        cmp.b   #9,STAT
        beq     .wake
        move.b  #7,CMDBUF
        rts
.wake:  moveq   #3,d0
        move.l  PAUSELBA,d1
        bra     cmd_at
.p3:    cmp.w   #$05,d0
        beq     .scan
        cmp.w   #$06,d0
        bne     .p4
.scan:  cmp.b   #4,STAT
        seq     d1
        and.w   #1,d1
        move.w  d1,SCANPREV
        addq.b  #3,d0               ; 8 forward, 9 back
        move.b  d0,CMDBUF
        rts
.p4:    cmp.w   #$07,d0
        bne     .p5
        moveq   #7,d1
        tst.w   SCANPREV
        beq     .so
        moveq   #6,d1
.so:    move.b  d1,CMDBUF
        rts
.p5:    cmp.w   #$14,d0             ; MSCPLAYT
        beq     .time
        cmp.w   #$16,d0             ; MSCSEEKT
        bne     .p6
.time:  move.w  d0,d7
        move.l  REQP0,d0
        bsr     msf_to_lba
        move.l  d0,d1
        clr.w   PLAYMODE
        moveq   #3,d0
        cmp.w   #$14,d7
        beq     cmd_at
        moveq   #4,d0
        bra     cmd_at
.p6:    cmp.w   #$18,d0             ; ROMSEEK
        bne     .track
        moveq   #4,d0
        move.l  REQP0,d1
        subq.l  #2,d1
        bra     cmd_at
.track: move.w  d0,d7               ; MSCPLAY, MSCPLAY1, MSCPLAYR, MSCSEEK, MSCSEEK1
        move.l  REQP0,d0
        bsr     track_bounds
        bcs     .r
        move.l  d1,PLAYSTART
        move.l  d2,PLAYEND
        moveq   #0,d3
        cmp.w   #$12,d7
        bne     .t1
        moveq   #1,d3
.t1:    cmp.w   #$13,d7
        bne     .t2
        moveq   #2,d3
.t2:    cmp.w   #$19,d7
        bne     .t3
        moveq   #3,d3
.t3:    move.w  d3,PLAYMODE
        moveq   #3,d0
        cmp.w   #$15,d7
        beq     .seek
        cmp.w   #$19,d7
        bne     cmd_at
.seek:  moveq   #4,d0
        bra     cmd_at
.rpause:
        move.w  #0,DECWAIT
        bsr     decoder_off
        move.b  #6,CMDBUF
        rts
.rresume:
        tst.w   ROMMODE
        beq     .r
        bra     read_from_want
.read:  clr.w   PLAYMODE
        bra     read_from_want

; d0 a track: d1 its first sector, d2 the next track's (or the lead-out's); carry set for a track not in the TOC.
track_bounds:
        tst.b   TOCVALID
        beq     .bad
        cmp.b   TOCFIRST,d0
        bcs     .bad
        cmp.b   TOCLAST,d0
        bhi     .bad
        move.w  d0,d3
        lsl.w   #2,d0
        lea     TOC,a0
        move.l  0(a0,d0.w),d0
        bsr     msf_to_lba
        move.l  d0,d1
        move.l  LEADOUT,d0
        cmp.b   TOCLAST,d3
        beq     .end
        addq.w  #1,d3
        lsl.w   #2,d3
        move.l  0(a0,d3.w),d0
.end:   bsr     msf_to_lba
        move.l  d0,d2
        andi    #$FE,ccr
        rts
.bad:   ori     #1,ccr
        rts

; The fader moved a step toward its target each exchange (BM §3-F: FDRCHG's step an exchange).
fader_step:
        move.w  FADERVOL,d0
        move.w  FADERTARGET,d1
        cmp.w   d1,d0
        beq     .w
        bcs     .up
        sub.w   FADERSTEP,d0
        bcs     .to
        cmp.w   d1,d0
        bcc     .w
.to:    move.w  d1,d0
        bra     .w
.up:    add.w   FADERSTEP,d0
        cmp.w   d1,d0
        bls     .w
        move.w  d1,d0
.w:     move.w  d0,FADERVOL
        lsl.w   #4,d0
        move.w  d0,GA_FADER
        rts

; bios_status, the LEDs' word and the volume (BM §3-B).
compose_status:
        move.b  STAT,d0
        move.w  #$4000,d1
        cmp.b   #5,d0
        beq     .put
        move.w  #$1000,d1
        cmp.b   #$B,d0
        beq     .put
        move.w  #$2000,d1
        tst.w   PHASE
        beq     .run
        tst.w   STEP
        bne     .put
        cmp.b   #$F,STAT+1
        bne     .put
        move.w  #$8000,d1           ; not ready: the drive still spinning up
        bra     .put
.run:   moveq   #0,d1
        cmp.b   #2,d0
        bne     .c1
        move.w  #$0800,d1
        tst.w   ROMMODE
        beq     .put
        move.w  #$0808,d1
        bra     .put
.c1:    cmp.b   #1,d0
        bne     .c3
        move.w  #$0100,d1
        tst.w   ROMMODE
        beq     .put
        move.w  #$0101,d1
        bra     .put
.c3:    cmp.b   #3,d0
        bne     .c4
        move.w  #$0300,d1
        bra     .put
.c4:    cmp.b   #4,d0
        bne     .put
        move.w  #$0500,d1
        tst.w   ROMMODE
        beq     .put
        move.w  #$0505,d1
.put:   move.w  d1,_CDSTAT
        move.w  MASTERVOL,_CDSTAT+24
        move.w  FADERVOL,d0
        lsl.w   #4,d0
        move.w  d0,_CDSTAT+26
        rts

; The LEDs (BM §3-L): the system's mode from the drive's state, or the one LEDSET chose; a blink half a second.
led_step:
        addq.w  #1,LEDTICK
        move.w  LEDMODE,d0
        bpl     .mode
        moveq   #1,d0               ; ready, a disc in
        move.w  _CDSTAT,d1
        move.w  d1,d2
        and.w   #$5000,d2           ; the tray open, or no disc
        beq     .disc
        moveq   #0,d0
        bra     .mode
.disc:  tst.w   STANDBY
        beq     .acc
        moveq   #3,d0
        bra     .mode
.acc:   and.w   #$0909,d1           ; playing, seeking or reading
        beq     .mode
        moveq   #2,d0
.mode:  add.w   d0,d0
        lea     led_modes(pc),a0
        move.b  0(a0,d0.w),d1       ; green
        move.b  1(a0,d0.w),d2       ; red
        btst    #5,LEDTICK+1
        sne     d3
        moveq   #0,d0
        bsr     .one
        add.b   d0,d0
        move.b  d2,d1
        bsr     .one
        move.b  d0,GA_LED
        rts
.one:   cmp.b   #1,d1               ; 0 off, 1 on, 2 blinking
        beq     .on
        cmp.b   #2,d1
        bne     .off
        tst.b   d3
        beq     .off
.on:    or.b    #1,d0
.off:   rts

led_modes:
        dc.b    1,2, 1,0, 1,1, 2,0, 2,2, 2,1, 0,2, 0,1

; ---- numbers

; d0 mm ss ff xx in BCD to a logical sector: 00:02:00 is 0.
msf_to_lba:
        movem.l d1-d2,-(sp)
        move.l  d0,d2
        swap    d0
        lsr.w   #8,d0
        bsr     bcd_to_bin
        mulu    #4500,d0
        move.l  d0,d1
        move.l  d2,d0
        swap    d0
        and.w   #$FF,d0
        bsr     bcd_to_bin
        mulu    #75,d0
        add.l   d0,d1
        move.l  d2,d0
        lsr.w   #8,d0
        and.w   #$FF,d0
        bsr     bcd_to_bin
        add.l   d0,d1
        sub.l   #150,d1
        move.l  d1,d0
        movem.l (sp)+,d1-d2
        rts

; d0.b in BCD to binary, the whole of d0.
bcd_to_bin:
        move.l  d1,-(sp)
        and.l   #$FF,d0
        move.w  d0,d1
        lsr.w   #4,d1
        mulu    #10,d1
        and.w   #15,d0
        add.w   d1,d0
        move.l  (sp)+,d1
        rts

; d0.w, 0-99, to BCD in d0.b.
bin_to_bcd:
        move.l  d1,-(sp)
        and.l   #$FFFF,d0
        divu    #10,d0
        move.l  d0,d1
        swap    d1
        lsl.w   #4,d0
        or.w    d1,d0
        move.l  (sp)+,d1
        rts

; ======================================================================================================================
; Level 2, from the main CPU (BM §5-3): the count _WAITVSYNC waits on, then the user's level 2.

level2:
        movem.l d0-d7/a0-a6,-(sp)
        addq.w  #1,VSYNC
        jsr     _USERCALL2
        movem.l (sp)+,d0-d7/a0-a6
        rte

; _BURAM (BM §7-1) on the internal backup RAM, 8 KiB on the odd bytes from $FE0001. Its footer, the last $40 bytes,
; is the one `docs/bram/emutos-mcd_bram-filesystem.md` gives (Nephrite_OpenBios.md §10): BRMINIT knows a formatted RAM
; by its last $20 bytes, BRMSTAT reads the free blocks and the files from it, BRMFORMAT writes a blank. The directory's
; calls wait for the format's specification (§5 Q2) and fail, leaving the memory as it stands.
BRAM            equ $FE0001
BRAM_BYTES      equ $2000

buram:
        movem.l d2-d3/a2-a3,-(sp)
        tst.w   d0
        beq     .init
        cmp.w   #1,d0
        beq     .stat
        cmp.w   #6,d0
        beq     .format
.fail:  movem.l (sp)+,d2-d3/a2-a3
        ori     #1,ccr
        rts
.ok:    movem.l (sp)+,d2-d3/a2-a3
        andi    #$FE,ccr
        rts

; BRMINIT: carry clear and d0.w the size in $1000 bytes for a Sega volume; else carry set, d1.w 1 for no volume, 2 for
; a RAM that carries the footer's last line with another identifier.
.init:  moveq   #BRAM_BYTES/$1000,d0
        bsr     bram_is_sega
        beq     .ok
        moveq   #1,d1
        lea     bram_footer+$30(pc),a2
        lea     BRAM+2*(BRAM_BYTES-$10),a3
        moveq   #15,d2
.other: cmpm.b  (a2)+,(a3)+
        bne     .fail
        addq.l  #1,a3
        dbra    d2,.other
        moveq   #2,d1
        bra     .fail

; BRMSTAT: d0.w the free blocks, d1.w the files, from the footer.
.stat:  bsr     bram_is_sega
        bne     .fail
        lea     BRAM+2*(BRAM_BYTES-$30),a2
        bsr     bram_word
        move.w  d2,d0
        lea     BRAM+2*(BRAM_BYTES-$28),a2
        bsr     bram_word
        move.w  d2,d1
        bra     .ok

; BRMFORMAT: every byte 0, then the footer, with no file and every block free but the footer's three.
.format:
        lea     BRAM,a2
        move.w  #BRAM_BYTES-$40-1,d2
.zero:  clr.b   (a2)
        addq.l  #2,a2
        dbra    d2,.zero
        lea     bram_footer(pc),a3
        moveq   #$3F,d2
.foot:  move.b  (a3)+,(a2)
        addq.l  #2,a2
        dbra    d2,.foot
        lea     BRAM+2*(BRAM_BYTES-$30),a2
        move.w  #BRAM_BYTES/$40-3,d3
        moveq   #3,d2
.free:  move.w  d3,d0
        lsr.w   #8,d0
        move.b  d0,(a2)
        move.b  d3,2(a2)
        addq.l  #4,a2
        dbra    d2,.free
        bra     .ok

; Z set when the RAM's last $20 bytes are a Sega volume's.
bram_is_sega:
        lea     bram_footer+$20(pc),a2
        lea     BRAM+2*(BRAM_BYTES-$20),a3
        moveq   #$1F,d2
.c:     cmpm.b  (a2)+,(a3)+
        bne     .r
        addq.l  #1,a3
        dbra    d2,.c
        moveq   #0,d2
.r:     rts

; The big-endian word at a2, the RAM's odd bytes, into d2.
bram_word:
        move.b  (a2),d2
        lsl.w   #8,d2
        move.b  2(a2),d2
        rts

bram_footer:
        dc.b    $5F,$5F,$5F,$5F,$5F,$5F,$5F,$5F,$5F,$5F,$5F,$00,$00,$00,$00,$40
        dc.b    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0
        dc.b    "SEGA_CD_ROM",0,1,0,0,0
        dc.b    "RAM_CARTRIDGE___"
        even

; _CDBOOT (BM §6-1): the tray's calls, and the disc's kind as the boot found it.
cdboot:
        cmp.w   #2,d0
        bne     .stat
        move.w  #$0A,d0             ; CBTOPENDISC: DRVOPEN
        jmp     _CDBIOS
.stat:  cmp.w   #3,d0
        bne     .chk
        btst    #6,_CDSTAT          ; CBTOPENSTAT: done once the tray is open
        beq     .busy
        bra     .ok
.chk:   cmp.w   #5,d0
        bne     .ok
        move.w  BOOTKIND,d0         ; CBTCHKSTAT
        bmi     .busy
.ok:    andi    #$FE,ccr
        rts
.busy:  ori     #1,ccr
        rts

code_end:

; ======================================================================================================================
; The common work and the jump table (BM §1-4, §5): every entry a JMP the program may rewrite but for the levels the
; system keeps (BM §5-4).

        org     $5EE0
        jmp     rts_stub.l
        jmp     rts_stub.l
        jmp     rts_stub.l
        jmp     rts_stub.l
        jmp     rts_stub.l
        jmp     rts_stub.l
        jmp     rts_stub.l
        org     $5F0A
        jmp     rts_stub.l            ; _SETJMPTBL
        jmp     wait_vsync.l          ; _WAITVSYNC
        jmp     buram.l               ; _BURAM
        jmp     cdboot.l              ; _CDBOOT
        jmp     cdbios.l              ; _CDBIOS
        jmp     rts_stub.l            ; _USERCALL0
        jmp     rts_stub.l            ; _USERCALL1
        jmp     rts_stub.l            ; _USERCALL2
        jmp     rts_stub.l            ; _USERCALL3
        jmp     sub_fatal.l           ; _ADRERR
        jmp     sub_fatal.l           ; _CODERR
        jmp     sub_fatal.l           ; _DEVERR
        jmp     sub_fatal.l           ; _CHKERR
        jmp     sub_fatal.l           ; _TRPERR
        jmp     sub_fatal.l           ; _SPVERR
        jmp     sub_fatal.l           ; _TRACE
        jmp     sub_fatal.l           ; _NOCOD0
        jmp     sub_fatal.l           ; _NOCOD1
        jmp     rte_stub.l            ; _LEVEL1
        jmp     level2.l              ; _LEVEL2
        jmp     rte_stub.l            ; _LEVEL3
        jmp     level4.l              ; _LEVEL4
        jmp     level5.l              ; _LEVEL5
        jmp     rte_stub.l            ; _LEVEL6
        jmp     rte_stub.l            ; _LEVEL7
        jmp     rte_stub.l            ; _TRAP00
        jmp     rte_stub.l
        jmp     rte_stub.l
        jmp     rte_stub.l
        jmp     rte_stub.l
        jmp     rte_stub.l
        jmp     rte_stub.l
        jmp     rte_stub.l
        jmp     rte_stub.l
        jmp     rte_stub.l
        jmp     rte_stub.l
        jmp     rte_stub.l
        jmp     rte_stub.l
        jmp     rte_stub.l
        jmp     rte_stub.l
        jmp     rte_stub.l            ; _TRAP15
image_end:
