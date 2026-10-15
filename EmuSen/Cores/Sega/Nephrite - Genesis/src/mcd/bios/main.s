; Nephrite's open Sega CD BIOS: the main CPU's program, the boot ROM at $000000 in mode 2 (Nephrite_OpenBios.md).
; It starts the Genesis and the sub CPU, shows its own screen while the sub CPU reads the disc, and hands the disc's
; initial program the main CPU's work RAM and its jump table (BM §4-2; the Technical Bulletins' "The status after
; booting is completed", p. 15-16). build.rs places the font at FONT_AT and the sub CPU's program at SUB_AT, and
; gives SUB_SIZE.

SUB_AT          equ $10000
FONT_AT         equ $E000

VDP_DATA        equ $C00000
VDP_CTRL        equ $C00004
PSG             equ $C00011
VERSION         equ $A10001
PAD1_DATA       equ $A10003
PAD1_CTRL       equ $A10009
Z80_BUSREQ      equ $A11100
Z80_RESET       equ $A11200
TMSS            equ $A14000

; The gate array, the main CPU's side (HM §4)
GA_INT          equ $A12000         ; IFL2 (byte)
GA_RESET        equ $A12001         ; SBRQ, SRES (byte)
GA_WP           equ $A12002         ; write protection (byte)
GA_MEM          equ $A12003         ; bank, DMNA, RET (byte)
GA_HINT         equ $A12006
GA_CFM          equ $A1200E
GA_CMD0         equ $A12010
GA_STA0         equ $A12020
GA_STA1         equ $A12022
GA_STA2         equ $A12024
PRG_WINDOW      equ $020000
WORD_RAM        equ $200000

; The work RAM the program finds (Technical Bulletins p. 15): the initial program from $FF0000, the stack below
; $FFFD00, the jump table from there; the VDP's registers 0-18 from $FFFDB4.
IP_START        equ $FF0000
STACK_TOP       equ $FFFD00
JUMPS           equ $FFFD00
_MLEVEL6        equ $FFFD06
_MLEVEL4        equ $FFFD0C
VDP_REGS        equ $FFFDB4
FRAMES          equ $FFFDE0         ; the boot's own: V-INTs counted, the pad, the state shown
PAD_LAST        equ $FFFDE2
SHOWN           equ $FFFDE4
CMD_SEQ         equ $FFFDE6
RAM_MARK        equ $FFFDF0

; ======================================================================================================================

        org     $0000
        dc.l    STACK_TOP, main_start
        dc.l    $FFFD7E, $FFFD7E, $FFFD84, $FFFD8A, $FFFD78, $FFFD90, $FFFDA2, $FFFDA8, $FFFD96, $FFFD9C
        dc.l    $FFFD84, $FFFD84, $FFFD84, $FFFD84, $FFFD84, $FFFD84, $FFFD84, $FFFD84, $FFFD84, $FFFD84
        dc.l    $FFFD84, $FFFD84
        dc.l    rte_stub
        dc.l    rte_stub, $FFFD12, rte_stub, $FFFD0C, rte_stub, $FFFD06, rte_stub
        dc.l    $FFFD18, $FFFD1E, $FFFD24, $FFFD2A, $FFFD30, $FFFD36, $FFFD3C, $FFFD42
        dc.l    $FFFD48, $FFFD4E, $FFFD54, $FFFD5A, $FFFD60, $FFFD66, $FFFD6C, $FFFD72
        dc.l    $FFFD84, $FFFD84, $FFFD84, $FFFD84, $FFFD84, $FFFD84, $FFFD84, $FFFD84
        dc.l    $FFFD84, $FFFD84, $FFFD84, $FFFD84, $FFFD84, $FFFD84, $FFFD84, $FFFD84

        ; The header: "SEGA" at $100 is how a cartridge finds a Sega CD beneath it (plutiedev, "Sub CPU in mode 1").
        org     $0100
        dc.b    "SEGA NEPHRITE   "
        dc.b    "OPEN BIOS 1.00  "
        dc.b    "NEPHRITE OPEN BIOS FOR THE SEGA CD              "
        dc.b    "NEPHRITE OPEN BIOS FOR THE SEGA CD              "
        dc.b    "BR 00000000-00  "
        even

        ; $0200-$1FFF: an RTS in every word. A disc's initial program may call routines of a player's BIOS that no
        ; manual names; here each returns at once, and each one a game is seen to call is recorded
        ; (Nephrite_OpenBios.md §5 Q4).
        org     $0200
        dcb.w   ($0368-$0200)/2,$4E75
        bra.w   set_vint            ; $0368 (D-41)
        dcb.w   ($2000-$036C)/2,$4E75

        org     $2000

main_start:
        move    #$2700,sr
        ; TMSS: a console whose version register's low nibble is not 0 waits for "SEGA" at $A14000 (plutiedev).
        move.b  VERSION,d0
        and.b   #$0F,d0
        beq     .tmss
        move.l  #"SEGA",TMSS
.tmss:  ; A reset with the system set up goes to the program's own reset entry, `_reset`.
        cmp.l   #"OPEN",RAM_MARK
        bne     cold
        lea     STACK_TOP,sp
        jmp     JUMPS

cold:   lea     STACK_TOP,sp
        lea     IP_START,a0
        move.w  #($10000/4)-1,d0
.clear: clr.l   (a0)+
        dbra    d0,.clear
        lea     jump_template(pc),a0
        lea     JUMPS,a1
        moveq   #(jump_template_end-jump_template)/2-1,d0
.jumps: move.w  (a0)+,(a1)+
        dbra    d0,.jumps
        ; The Z80 given a DI and a HALT and let run, its bus released: a program may ask for the bus and wait for the
        ; grant, which a Z80 held in reset never gives (plutiedev, "Using the Z80"; D-46). The PSG silent.
        move.w  #0,Z80_RESET
        move.w  #$100,Z80_BUSREQ
        move.w  #$100,Z80_RESET
        move.b  #$F3,$A00000
        move.b  #$76,$A00001
        move.w  #0,Z80_RESET
        moveq   #20,d0
.zw:    dbra    d0,.zw
        move.w  #$100,Z80_RESET
        move.w  #0,Z80_BUSREQ
        move.b  #$9F,PSG
        move.b  #$BF,PSG
        move.b  #$DF,PSG
        move.b  #$FF,PSG
        move.b  #$40,PAD1_CTRL
        bsr     vdp_init
        bsr     load_font
        bsr     draw_screen
        bsr     start_sub
        move.l  #"OPEN",RAM_MARK
        move.w  #$8174,d0           ; the display on
        bsr     vdp_reg
        move    #$2000,sr

; The screen until the sub CPU has the initial program in Word RAM.
show:   bsr     wait_frame
        move.w  GA_STA0,d0
        cmp.w   #6,d0
        beq     start_ip
        cmp.w   SHOWN,d0
        beq     .pad
        move.w  d0,SHOWN
        bsr     draw_state
.pad:   bsr     read_start
        bra     show

; The initial program from Word RAM into work RAM, and the hand-over.
start_ip:
.ret:   btst    #0,GA_MEM           ; RET: Word RAM the main CPU's
        beq     .ret
        moveq   #0,d0
        move.w  GA_STA1,d0
        addq.l  #3,d0
        lsr.l   #2,d0
        subq.w  #1,d0
        lea     WORD_RAM,a0
        lea     IP_START,a1
.copy:  move.l  (a0)+,(a1)+
        dbra    d0,.copy
        move    #$2700,sr
        move.w  #$8134,d0           ; the display off, V-INT on
        bsr     vdp_reg
        move.w  #-1,GA_CMD0         ; the hand-over (sub.s, load_ip)
.sta:   tst.w   GA_STA0
        bne     .sta
        lea     GA_CMD0,a0
        moveq   #7,d0
.cmd:   clr.w   (a0)+
        dbra    d0,.cmd
        clr.b   GA_CFM
        moveq   #0,d0
        move.w  GA_STA2,d0
        lea     IP_START,a0
        add.l   d0,a0
        lea     STACK_TOP,sp
        moveq   #0,d0
        moveq   #0,d1
        ; Interrupts open, V-INT on: a program may wait on its own V-INT without setting the mask (D-45).
        move    #$2000,sr
        jmp     (a0)

; The sub CPU: the gate array reset (HM §4-1; plutiedev), the sub CPU held and its bus taken, its program into PRG-RAM
; through bank 0, its system area write-protected (BM §1-4), Word RAM given to it, and the reset released.
start_sub:
        move.w  #$FF00,GA_WP
        move.b  #3,GA_RESET
        move.b  #2,GA_RESET
        move.b  #0,GA_RESET
        move.w  #$FD0C,GA_HINT      ; H-INT through `_mlevel4`
        move.b  #2,GA_RESET
.bus:   btst    #1,GA_RESET
        beq     .bus
        move.b  #0,GA_WP
        move.b  #0,GA_MEM
        lea     SUB_AT,a0
        lea     PRG_WINDOW,a1
        move.w  #SUB_SIZE/4-1,d0
.copy:  move.l  (a0)+,(a1)+
        dbra    d0,.copy
        move.b  #$2A,GA_WP          ; $0000-$53FF protected from the sub CPU
        bset    #1,GA_MEM           ; DMNA: Word RAM the sub CPU's
        move.b  #1,GA_RESET         ; SRES released, the bus given back
        rts

; ---- the screen: the boot's own font and words, no logo

vdp_init:
        lea     vdp_values(pc),a0
        lea     VDP_REGS,a1
        moveq   #18,d1
        move.w  #$8000,d0
.reg:   move.b  (a0)+,d0
        move.w  d0,VDP_CTRL
        move.w  d0,(a1)+
        add.w   #$100,d0
        dbra    d1,.reg
        move.l  #$40000000,VDP_CTRL ; VRAM from 0
        move.w  #$7FFF,d0
.vram:  move.w  #0,VDP_DATA
        dbra    d0,.vram
        move.l  #$40000010,VDP_CTRL ; VSRAM
        moveq   #39,d0
.vs:    move.w  #0,VDP_DATA
        dbra    d0,.vs
        move.l  #$C0000000,VDP_CTRL ; CRAM: a dark blue, white, a pale green
        move.w  #$0400,VDP_DATA
        move.w  #$0EEE,VDP_DATA
        move.w  #$0AEA,VDP_DATA
        moveq   #60,d0
.cram:  move.w  #0,VDP_DATA
        dbra    d0,.cram
        rts

vdp_reg:
        move.w  d0,VDP_CTRL
        move.w  d0,d1
        lsr.w   #8,d1
        and.w   #$1F,d1
        add.w   d1,d1
        lea     VDP_REGS,a0
        move.w  d0,0(a0,d1.w)
        rts

; Mode 5, 40 cells wide; plane A at $C000, the window at $F000, plane B at $E000, sprites at $D800, scrolls at $FC00.
vdp_values:
        dc.b    $04,$34,$30,$3C,$07,$6C,$00,$00,$00,$00,$FF,$00,$81,$3F,$00,$02,$01,$00,$00
        even

load_font:
        move.l  #$40000000,VDP_CTRL
        lea     FONT_AT,a0
        move.w  #64*8-1,d0
.l:     move.l  (a0)+,VDP_DATA
        dbra    d0,.l
        rts

; a0 a string ending in 0, d0 its column, d1 its row, d2 the palette's bits (0 or $2000).
print:
        mulu    #64,d1
        add.w   d0,d1
        add.w   d1,d1
        add.w   #$C000,d1
        moveq   #0,d0
        move.w  d1,d0
        and.w   #$3FFF,d0
        swap    d0
        rol.w   #2,d1
        and.w   #3,d1
        move.w  d1,d0
        or.l    #$40000000,d0
        move.l  d0,VDP_CTRL
.ch:    moveq   #0,d0
        move.b  (a0)+,d0
        beq     .end
        sub.w   #32,d0
        bcs     .sp
        cmp.w   #64,d0
        bcs     .put
.sp:    moveq   #0,d0
.put:   or.w    d2,d0
        move.w  d0,VDP_DATA
        bra     .ch
.end:   rts

draw_screen:
        lea     title1(pc),a0
        moveq   #16,d0
        moveq   #7,d1
        moveq   #0,d2
        bsr     print
        lea     title2(pc),a0
        moveq   #7,d0
        moveq   #9,d1
        moveq   #0,d2
        bsr     print
        lea     help(pc),a0
        moveq   #5,d0
        moveq   #22,d1
        moveq   #0,d2
        bsr     print
        move.w  #-1,SHOWN
        rts

; The sub CPU's state (sub.s, set_state) in words on row 14.
draw_state:
        move.w  d0,-(sp)
        lea     blank(pc),a0
        moveq   #0,d0
        moveq   #14,d1
        moveq   #0,d2
        bsr     print
        move.w  (sp)+,d0
        cmp.w   #7,d0
        bcs     .k
        moveq   #0,d0
.k:     add.w   d0,d0
        lea     states(pc),a1
        move.w  0(a1,d0.w),d0
        lea     0(a1,d0.w),a0
        move.l  a0,a1
        moveq   #0,d0
.len:   tst.b   (a1)+
        beq     .c
        addq.w  #1,d0
        bra     .len
.c:     neg.w   d0
        add.w   #40,d0
        lsr.w   #1,d0
        moveq   #14,d1
        move.w  #$0000,d2
        bra     print

states:
        dc.w    s_start-states, s_open-states, s_nodisc-states, s_read-states, s_audio-states, s_other-states
        dc.w    s_go-states
s_start:    dc.b    "STARTING",0
s_open:     dc.b    "THE TRAY IS OPEN",0
s_nodisc:   dc.b    "NO DISC",0
s_read:     dc.b    "READING THE DISC",0
s_audio:    dc.b    "AN AUDIO CD: NO GAME TO START",0
s_other:    dc.b    "NOT A SEGA CD GAME DISC",0
s_go:       dc.b    "STARTING THE GAME",0
title1:     dc.b    "NEPHRITE",0
title2:     dc.b    "OPEN BIOS FOR THE SEGA CD",0
help:       dc.b    "START: OPEN OR CLOSE THE TRAY",0
blank:      dc.b    "                                        ",0
        even

; START on pad 1, pressed since the last frame, asks the sub CPU to open or close the tray.
read_start:
        move.b  #$00,PAD1_DATA
        nop
        nop
        move.b  PAD1_DATA,d0
        move.b  #$40,PAD1_DATA
        not.b   d0
        and.b   #$20,d0
        move.b  PAD_LAST,d1
        move.b  d0,PAD_LAST
        not.b   d1
        and.b   d1,d0
        beq     .r
        addq.w  #1,CMD_SEQ
        move.w  CMD_SEQ,d0
        lsl.w   #8,d0
        move.b  #1,d0
        move.w  d0,GA_CMD0
.r:     rts

wait_frame:
        move.w  FRAMES,d0
.w:     cmp.w   FRAMES,d0
        beq     .w
        rts

; ---- the handlers the jump table starts with

; V-INT: level 2 to the sub CPU each frame, which its system needs while it loads (Technical Bulletins p. 18).
vint_default:
        move.b  #1,GA_INT
        addq.w  #1,FRAMES
        rte

; $0368: a1 the V-INT handler, set in `_mlevel6` and V-INT enabled, every register kept (the 240p Test Suite's
; start-up, "Enable VBlank"; Nephrite_Disputes.md D-41).
set_vint:
        movem.l d0-d1/a0,-(sp)
        move.l  a1,_MLEVEL6+2
        move.w  VDP_REGS+2,d0
        or.w    #$8120,d0
        bsr     vdp_reg
        movem.l (sp)+,d0-d1/a0
        rts

rte_stub:
        rte

rts_stub:
        rts

; A fault: the screen red, the CPU stopped.
main_fatal:
        move    #$2700,sr
        move.l  #$C0000000,VDP_CTRL
        move.w  #$000E,VDP_DATA
.h:     stop    #$2700
        bra     .h

; The jump table as it starts (Technical Bulletins p. 15-16): `_reset`, the three levels, the sixteen traps, the
; errors and `_vint_ex`, each a JMP whose address a program sets at its label plus 2.
jump_template:
        jmp     cold.l              ; _reset
        jmp     vint_default.l      ; _mlevel6
        jmp     rte_stub.l          ; _mlevel4
        jmp     rte_stub.l          ; _mlevel2
        jmp     rte_stub.l          ; _mtrap00
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
        jmp     rte_stub.l          ; _mtrap15
        jmp     main_fatal.l        ; _mchkerr
        jmp     main_fatal.l        ; _madrerr
        jmp     main_fatal.l        ; _mcoderr
        jmp     main_fatal.l        ; _mdiverr
        jmp     main_fatal.l        ; _mtrperr
        jmp     main_fatal.l        ; _mnocod0
        jmp     main_fatal.l        ; _mnocod1
        jmp     main_fatal.l        ; _mspverr
        jmp     main_fatal.l        ; _mtrace
        jmp     rts_stub.l          ; _vint_ex
jump_template_end:

main_end:
