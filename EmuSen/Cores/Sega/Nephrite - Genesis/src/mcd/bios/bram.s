; The backup RAM's file system for _BURAM's calls (BM §7-1), on any volume of the format Nephrite_Bram.md states: the
; footer (§3), the directory growing down from it in coded blocks of two entries (§5), the block code (§6), and the
; data area from block 1 (§7). A volume is d7.l bytes whose first byte is at a5, one byte at every second address;
; a6 points past a work area of BW_SIZE bytes. Where the page leaves a behaviour open, the choice made is recorded in
; Nephrite_OpenBios.md §14 and as a dispute (Nephrite_Disputes.md D-49 on). bram_call's caller keeps d2-d7/a2-a6.

F_U             equ -32             ; a block's 32 data bytes
F_SYM           equ -96             ; its 64 symbols, then its 64 stored bytes
F_M             equ -136            ; the 36-byte message: the CRC, the data, the CRC's complement
F_ENT           equ -152            ; a directory entry: the name, the flag, the first block, the blocks
F_NEW           equ -184            ; a directory block's new data
F_NEED          equ -186
F_JLAST         equ -188
F_START         equ -190
BW_SIZE         equ 190

GF_INV3         equ $3E             ; 1/(alpha+1) in GF(64) by x^6+x+1

; d0.w the function: carry clear on success.
bram_call:
        cmp.w   #8,d0
        bhi     bc_fail
        add.w   d0,d0
        move.w  .tab(pc,d0.w),d0
        jmp     .tab(pc,d0.w)
.tab:   dc.w    b_init-.tab, b_stat-.tab, b_serch-.tab, b_read-.tab, b_write-.tab
        dc.w    b_del-.tab, b_format-.tab, b_dir-.tab, b_verify-.tab

bc_ok:  andi    #$FE,ccr
        rts
bc_fail:
        ori     #1,ccr
        rts

; BRMINIT: a Sega volume gives carry clear and d0.w its size in $1000 bytes; else carry set, d1.w 1 unformatted or 2
; another format. The identifier's 12 bytes go to a1's buffer for a volume of either format.
b_init:
        bsr     vol_kind
        move.w  d0,d1
        beq     .sega
        cmp.w   #2,d1
        bne     bc_fail
        bsr     vol_name
        moveq   #2,d1
        bra     bc_fail
.sega:  bsr     vol_name
        move.l  d7,d0
        moveq   #12,d1
        lsr.l   d1,d0
        bra     bc_ok

; BRMSTAT: d0.w the free blocks, d1.w the files, as the footer holds them; both -1 when the volume is not Sega's.
b_stat:
        bsr     vol_kind
        tst.w   d0
        bne     .no
        bsr     vol_name
        bsr     vol_counts
        move.w  d4,d0
        move.w  d5,d1
        bra     bc_ok
.no:    moveq   #-1,d0
        moveq   #-1,d1
        bra     bc_fail

; BRMSERCH: a0 the name; found, d0.w its blocks, d1.b 0 normal or -1 protected, a0 the address of its first block.
b_serch:
        move.l  a0,a2
        bsr     vol_kind
        tst.w   d0
        bne     bc_fail
        bsr     vol_counts
        move.l  a2,a0
        bsr     dir_find
        tst.w   d6
        beq     bc_fail
        moveq   #0,d2
        move.w  F_ENT+12(a6),d2
        lsl.l   #7,d2
        add.l   a5,d2
        subq.l  #1,d2
        move.l  d2,a0
        bsr     ent_out
        bra     bc_ok

; BRMREAD: a0 the name, a1 the buffer; d0.w and d1.b as BRMSERCH's. A protected block past correcting fails the call.
b_read:
        move.l  a0,a2
        move.l  a1,a3
        bsr     vol_kind
        tst.w   d0
        bne     bc_fail
        bsr     vol_counts
        move.l  a2,a0
        bsr     dir_find
        tst.w   d6
        beq     bc_fail
        bsr     data_read
        bsr     ent_out
        tst.w   d3
        bmi     bc_fail
        bra     bc_ok

; BRMWRITE: a0 the name, the flag and the blocks; a1 the data. A name already there with the same blocks and flag is
; written over in place; with others it is deleted and written anew.
b_write:
        move.l  a0,a2
        move.l  a1,a3
        bsr     vol_kind
        tst.w   d0
        bne     bc_fail
        bsr     vol_counts
        move.w  12(a2),d3
        beq     bc_fail
        move.b  11(a2),d2
        sne     d2
        move.l  a2,a0
        bsr     dir_find
        tst.w   d6
        beq     .new
        cmp.w   F_ENT+14(a6),d3
        bne     .move
        cmp.b   F_ENT+11(a6),d2
        bne     .move
        bsr     data_write
        bra     bc_ok
.move:  bsr     dir_delete
.new:   move.w  d5,d0               ; the blocks, and the directory block a new entry may add (Nephrite_Bram.md §5.4)
        addq.w  #3,d0
        lsr.w   #1,d0
        move.w  d5,d1
        addq.w  #2,d1
        lsr.w   #1,d1
        sub.w   d1,d0
        add.w   d3,d0
        cmp.w   d4,d0
        bhi     bc_fail
        move.w  d0,F_NEED(a6)
        bsr     files_end
        move.w  d0,F_START(a6)
        move.l  d7,d1               ; and the data stays below the directory
        lsr.l   #6,d1
        subq.w  #1,d1
        move.w  d5,d6
        addq.w  #3,d6
        lsr.w   #1,d6
        sub.w   d6,d1
        add.w   d3,d0
        cmp.w   d1,d0
        bhi     bc_fail
        move.w  F_START(a6),F_ENT+12(a6)
        move.w  d3,F_ENT+14(a6)
        move.b  d2,F_ENT+11(a6)
        move.l  a2,a0
        lea     F_ENT(a6),a1
        moveq   #10,d1
.name:  move.b  (a0)+,(a1)+
        dbra    d1,.name
        bsr     data_write
        move.w  d5,d0               ; entry n+1's block: a new one, or the last with the entry in its second slot
        lsr.w   #1,d0
        bsr     dir_at
        move.l  d0,a4
        btst    #0,d5
        bne     .join
        lea     F_U(a6),a1
        moveq   #15,d1
.zero:  clr.b   (a1)+
        dbra    d1,.zero
        bra     .put
.join:  move.l  a4,d0
        bsr     bv_load
        bsr     bc_decode
        lea     F_U(a6),a1
.put:   lea     F_ENT(a6),a0
        moveq   #15,d1
.ent:   move.b  (a0)+,(a1)+
        dbra    d1,.ent
        bsr     bc_encode
        move.l  a4,d0
        bsr     bv_store
        sub.w   F_NEED(a6),d4
        addq.w  #1,d5
        bsr     vol_setcounts
        bra     bc_ok

; BRMDEL: a0 the name. The data after it and the entries after it move down (Nephrite_Bram.md §7.3).
b_del:
        move.l  a0,a2
        bsr     vol_kind
        tst.w   d0
        bne     bc_fail
        bsr     vol_counts
        move.l  a2,a0
        bsr     dir_find
        tst.w   d6
        beq     bc_fail
        bsr     dir_delete
        bra     bc_ok

; BRMFORMAT: the footer, with no file and every block free but three; nothing below it changes (Nephrite_Bram.md §4).
b_format:
        move.l  d7,d0
        sub.l   #$40,d0
        bsr     bv_at
        lea     bram_footer(pc),a1
        moveq   #63,d1
.f:     move.b  (a1)+,(a0)
        addq.l  #2,a0
        dbra    d1,.f
        move.l  d7,d0
        lsr.l   #6,d0
        subq.w  #3,d0
        move.w  d0,d4
        moveq   #0,d5
        bsr     vol_setcounts
        bra     bc_ok

; BRMDIR: d1.l files to skip (high) and the buffer's room in entries (low), a0 a template where `*` matches the rest,
; a1 the buffer, which takes each matching entry's 16 bytes; carry set when a match finds the buffer full.
b_dir:
        move.l  a0,a2
        move.l  a1,a3
        move.w  d1,d3
        swap    d1
        move.w  d1,d2
        bsr     vol_kind
        tst.w   d0
        bne     bc_fail
        bsr     vol_counts
        moveq   #1,d6
.e:     cmp.w   d5,d6
        bhi     bc_ok
        move.w  d6,d0
        bsr     dir_entry
        move.l  a2,a0
        lea     F_ENT(a6),a1
        moveq   #10,d1
.c:     move.b  (a0)+,d0
        cmp.b   #$2A,d0
        beq     .hit
        cmp.b   (a1)+,d0
        bne     .nx
        dbra    d1,.c
.hit:   tst.w   d2
        beq     .take
        subq.w  #1,d2
        bra     .nx
.take:  tst.w   d3
        beq     bc_fail
        subq.w  #1,d3
        lea     F_ENT(a6),a0
        moveq   #15,d1
.t:     move.b  (a0)+,(a3)+
        dbra    d1,.t
.nx:    addq.w  #1,d6
        bra     .e

; BRMVERIFY: as BRMWRITE's; carry clear when the file holds the data, else d0.w 0 not found or -1 different.
b_verify:
        move.l  a0,a2
        move.l  a1,a3
        bsr     vol_kind
        tst.w   d0
        bne     .none
        bsr     vol_counts
        move.l  a2,a0
        bsr     dir_find
        tst.w   d6
        beq     .none
        move.w  12(a2),d0
        cmp.w   F_ENT+14(a6),d0
        bne     .diff
        move.b  11(a2),d0
        sne     d0
        cmp.b   F_ENT+11(a6),d0
        bne     .diff
        bsr     data_verify
        bne     .diff
        bra     bc_ok
.none:  moveq   #0,d0
        bra     bc_fail
.diff:  moveq   #-1,d0
        bra     bc_fail

; ---- the volume

; a0 the address of volume byte d0.l.
bv_at:  move.l  d0,a0
        add.l   a0,a0
        add.l   a5,a0
        rts

; The 64 bytes at volume byte d0.l into F_SYM, or back.
bv_load:
        bsr     bv_at
        lea     F_SYM(a6),a1
        moveq   #63,d1
.l:     move.b  (a0),(a1)+
        addq.l  #2,a0
        dbra    d1,.l
        rts

bv_store:
        bsr     bv_at
        lea     F_SYM(a6),a1
        moveq   #63,d1
.l:     move.b  (a1)+,(a0)
        addq.l  #2,a0
        dbra    d1,.l
        rts

; Z set when the d1.w+1 volume bytes at a0 equal the bytes at a2; both pass them.
bv_cmp: movem.l d3,-(sp)
        moveq   #0,d3
.l:     move.b  (a0),d0
        addq.l  #2,a0
        cmp.b   (a2)+,d0
        beq     .e
        moveq   #1,d3
.e:     dbra    d1,.l
        tst.w   d3
        movem.l (sp)+,d3
        rts

; d0.w 0 for a Sega volume, 1 unformatted, 2 another format (Nephrite_Bram.md §3): RAM_CARTRIDGE___ needs one of its
; first three longwords whole; then SEGA_CD_ROM and its zero exactly, or another name of 0-9, A-Z and _.
vol_kind:
        movem.l d2/a2,-(sp)
        move.l  d7,d0
        sub.l   #$10,d0
        bsr     bv_at
        lea     bram_footer+$30(pc),a2
        moveq   #2,d2
.lw:    moveq   #3,d1
        bsr     bv_cmp
        beq     .id
        dbra    d2,.lw
        moveq   #1,d0
        bra     .out
.id:    move.l  d7,d0
        sub.l   #$20,d0
        bsr     bv_at
        lea     bram_footer+$20(pc),a2
        moveq   #11,d1
        bsr     bv_cmp
        bne     .other
        moveq   #0,d0
        bra     .out
.other: move.l  d7,d0
        sub.l   #$20,d0
        bsr     bv_at
        moveq   #10,d2
.ch:    move.b  (a0),d0
        addq.l  #2,a0
        cmp.b   #$5F,d0
        beq     .nx
        cmp.b   #$30,d0
        blo     .unf
        cmp.b   #$39,d0
        bls     .nx
        cmp.b   #$41,d0
        blo     .unf
        cmp.b   #$5A,d0
        bhi     .unf
.nx:    dbra    d2,.ch
        moveq   #2,d0
        bra     .out
.unf:   moveq   #1,d0
.out:   movem.l (sp)+,d2/a2
        rts

; The footer's identifier, 12 bytes, into a1's buffer when a1 is not 0.
vol_name:
        move.l  a1,d0
        beq     .r
        move.l  d7,d0
        sub.l   #$20,d0
        bsr     bv_at
        moveq   #11,d1
.c:     move.b  (a0),(a1)+
        addq.l  #2,a0
        dbra    d1,.c
.r:     rts

; d4.w the free blocks and d5.w the files, each the value three or four of its copies hold, else 0 (Nephrite_Bram.md
; §3); a count past the volume reads as none.
vol_counts:
        move.l  d7,d0
        sub.l   #$30,d0
        bsr     vol_count
        move.w  d0,d4
        move.l  d7,d0
        sub.l   #$28,d0
        bsr     vol_count
        move.w  d0,d5
        move.l  d7,d0
        lsr.l   #5,d0
        cmp.w   d0,d5
        bls     .r
        moveq   #0,d4
        moveq   #0,d5
.r:     rts

vol_count:
        movem.l d2-d6,-(sp)
        bsr     bv_at
        bsr     .word
        move.w  d1,d2
        bsr     .word
        move.w  d1,d3
        bsr     .word
        move.w  d1,d4
        bsr     .word
        move.w  d1,d5
        move.w  d2,d0
        bsr     .held
        bcc     .out
        move.w  d3,d0
        bsr     .held
        bcc     .out
        moveq   #0,d0
.out:   movem.l (sp)+,d2-d6
        rts
.word:  move.b  (a0),d1
        lsl.w   #8,d1
        move.b  2(a0),d1
        addq.l  #4,a0
        rts
; carry clear when three or more of d2-d5 equal d0.w
.held:  moveq   #0,d6
        cmp.w   d2,d0
        bne     .h3
        addq.w  #1,d6
.h3:    cmp.w   d3,d0
        bne     .h4
        addq.w  #1,d6
.h4:    cmp.w   d4,d0
        bne     .h5
        addq.w  #1,d6
.h5:    cmp.w   d5,d0
        bne     .h6
        addq.w  #1,d6
.h6:    cmp.w   #3,d6
        rts

; Each count's four copies from d4.w and d5.w.
vol_setcounts:
        move.l  d7,d0
        sub.l   #$30,d0
        bsr     bv_at
        moveq   #3,d1
.f:     move.w  d4,d0
        lsr.w   #8,d0
        move.b  d0,(a0)
        move.b  d4,2(a0)
        addq.l  #4,a0
        dbra    d1,.f
        moveq   #3,d1
.n:     move.w  d5,d0
        lsr.w   #8,d0
        move.b  d0,(a0)
        move.b  d5,2(a0)
        addq.l  #4,a0
        dbra    d1,.n
        rts

; ---- the directory

; d0.l the volume byte of directory block d0.w, counted from the footer down.
dir_at: and.l   #$FFFF,d0
        lsl.l   #6,d0
        neg.l   d0
        add.l   d7,d0
        sub.l   #$80,d0
        rts

; Entry d0.w, from 1, into F_ENT: an odd entry is its block's second half, an even one its first.
dir_entry:
        movem.l d2,-(sp)
        move.w  d0,d2
        subq.w  #1,d0
        lsr.w   #1,d0
        bsr     dir_at
        bsr     bv_load
        bsr     bc_decode
        lea     F_U+16(a6),a0
        btst    #0,d2
        bne     .odd
        lea     F_U(a6),a0
.odd:   lea     F_ENT(a6),a1
        moveq   #15,d1
.c:     move.b  (a0)+,(a1)+
        dbra    d1,.c
        movem.l (sp)+,d2
        rts

; d6.w the entry named by a0's 11 bytes among d5.w's, with F_ENT filled, or 0.
dir_find:
        movem.l d2/a2,-(sp)
        move.l  a0,a2
        moveq   #0,d6
        moveq   #1,d2
.e:     cmp.w   d5,d2
        bhi     .out
        move.w  d2,d0
        bsr     dir_entry
        move.l  a2,a0
        lea     F_ENT(a6),a1
        moveq   #10,d1
.c:     cmpm.b  (a0)+,(a1)+
        bne     .nx
        dbra    d1,.c
        move.w  d2,d6
        bra     .out
.nx:    addq.w  #1,d2
        bra     .e
.out:   movem.l (sp)+,d2/a2
        rts

; d0.w the first block after the last file: where the next file goes (Nephrite_Bram.md §7.2).
files_end:
        tst.w   d5
        bne     .some
        moveq   #1,d0
        rts
.some:  move.w  d5,d0
        bsr     dir_entry
        move.w  F_ENT+12(a6),d0
        add.w   F_ENT+14(a6),d0
        rts

; d0.w the blocks of F_ENT's file, d1.b 0 normal or -1 protected.
ent_out:
        move.w  F_ENT+14(a6),d0
        moveq   #0,d1
        tst.b   F_ENT+11(a6)
        beq     .r
        moveq   #-1,d1
.r:     rts

; Entry d6.w of d5.w deleted, with F_ENT its entry: the data after it moved down over it, the entries after it each one
; earlier with its first block lowered, then the counts in d4.w and d5.w (Nephrite_Bram.md §7.3).
dir_delete:
        movem.l d2-d3/d6/a2-a4,-(sp)
        moveq   #0,d2
        move.w  F_ENT+12(a6),d2
        moveq   #0,d3
        move.w  F_ENT+14(a6),d3
        bsr     files_end
        and.l   #$FFFF,d0
        sub.l   d2,d0
        sub.l   d3,d0
        beq     .dir
        lsl.l   #6,d0
        move.l  d0,a4
        move.l  d2,d0
        add.l   d3,d0
        lsl.l   #6,d0
        bsr     bv_at
        move.l  a0,a2
        move.l  d2,d0
        lsl.l   #6,d0
        bsr     bv_at
        move.l  a4,d0
.mv:    move.b  (a2),(a0)
        addq.l  #2,a2
        addq.l  #2,a0
        subq.l  #1,d0
        bne     .mv
.dir:   move.w  d5,d0
        subq.w  #1,d0
        beq     .counts
        subq.w  #1,d0
        lsr.w   #1,d0
        move.w  d0,F_JLAST(a6)
        move.w  d6,d2
        subq.w  #1,d2
        lsr.w   #1,d2
.blk:   move.w  d2,d0
        add.w   d0,d0
        addq.w  #1,d0
        lea     F_NEW+16(a6),a1
        bsr     slot_fill
        move.w  d2,d0
        add.w   d0,d0
        addq.w  #2,d0
        lea     F_NEW(a6),a1
        bsr     slot_fill
        lea     F_NEW(a6),a0
        lea     F_U(a6),a1
        moveq   #31,d1
.cp:    move.b  (a0)+,(a1)+
        dbra    d1,.cp
        bsr     bc_encode
        move.w  d2,d0
        bsr     dir_at
        bsr     bv_store
        addq.w  #1,d2
        cmp.w   F_JLAST(a6),d2
        bls     .blk
.counts:
        add.w   d3,d4
        move.w  d5,d0
        addq.w  #2,d0
        lsr.w   #1,d0
        add.w   d0,d4
        move.w  d5,d0
        addq.w  #1,d0
        lsr.w   #1,d0
        sub.w   d0,d4
        subq.w  #1,d5
        bsr     vol_setcounts
        movem.l (sp)+,d2-d3/d6/a2-a4
        rts

; New entry d0.w's 16 bytes at a1, while entry d6.w of d5.w is deleted and d3.w blocks freed: an earlier entry as it
; was, a later one the entry after it with its first block lowered, past the last zeros.
slot_fill:
        movem.l d2/a2,-(sp)
        move.l  a1,a2
        move.w  d5,d1
        subq.w  #1,d1
        cmp.w   d1,d0
        bhi     .zero
        cmp.w   d6,d0
        blo     .same
        addq.w  #1,d0
        bsr     dir_entry
        sub.w   d3,F_ENT+12(a6)
        bra     .copy
.same:  bsr     dir_entry
.copy:  lea     F_ENT(a6),a0
        moveq   #15,d1
.c:     move.b  (a0)+,(a2)+
        dbra    d1,.c
        bra     .out
.zero:  moveq   #15,d1
.z:     clr.b   (a2)+
        dbra    d1,.z
.out:   movem.l (sp)+,d2/a2
        rts

; ---- the data

; F_ENT's file into a3's buffer, 64 bytes a block, or 32 decoded from each protected one; d3.w -1 when a protected
; block could not be corrected, else 0.
data_read:
        movem.l d4-d5,-(sp)
        moveq   #0,d3
        moveq   #0,d4
        move.w  F_ENT+12(a6),d4
        move.w  F_ENT+14(a6),d5
        beq     .out
        subq.w  #1,d5
        tst.b   F_ENT+11(a6)
        bne     .prot
        move.l  d4,d0
        lsl.l   #6,d0
        bsr     bv_at
.n:     moveq   #63,d1
.nb:    move.b  (a0),(a3)+
        addq.l  #2,a0
        dbra    d1,.nb
        dbra    d5,.n
        bra     .out
.prot:  move.l  d4,d0
        lsl.l   #6,d0
        bsr     bv_load
        bsr     bc_decode
        tst.w   d0
        bpl     .ok
        moveq   #-1,d3
.ok:    lea     F_U(a6),a0
        moveq   #31,d1
.pc:    move.b  (a0)+,(a3)+
        dbra    d1,.pc
        addq.l  #1,d4
        dbra    d5,.prot
.out:   movem.l (sp)+,d4-d5
        rts

; a3's data into F_ENT's file, as data_read reads it.
data_write:
        movem.l d4-d5/a3,-(sp)
        moveq   #0,d4
        move.w  F_ENT+12(a6),d4
        move.w  F_ENT+14(a6),d5
        beq     .out
        subq.w  #1,d5
        tst.b   F_ENT+11(a6)
        bne     .prot
        move.l  d4,d0
        lsl.l   #6,d0
        bsr     bv_at
.n:     moveq   #63,d1
.nb:    move.b  (a3)+,(a0)
        addq.l  #2,a0
        dbra    d1,.nb
        dbra    d5,.n
        bra     .out
.prot:  lea     F_U(a6),a1
        moveq   #31,d1
.pc:    move.b  (a3)+,(a1)+
        dbra    d1,.pc
        bsr     bc_encode
        move.l  d4,d0
        lsl.l   #6,d0
        bsr     bv_store
        addq.l  #1,d4
        dbra    d5,.prot
.out:   movem.l (sp)+,d4-d5/a3
        rts

; Z set when F_ENT's file holds a3's data, as data_read would read it.
data_verify:
        movem.l d2-d5,-(sp)
        moveq   #0,d2
        moveq   #0,d4
        move.w  F_ENT+12(a6),d4
        move.w  F_ENT+14(a6),d5
        beq     .out
        subq.w  #1,d5
        tst.b   F_ENT+11(a6)
        bne     .prot
        move.l  d4,d0
        lsl.l   #6,d0
        bsr     bv_at
.n:     moveq   #63,d1
.nb:    move.b  (a0),d0
        addq.l  #2,a0
        cmp.b   (a3)+,d0
        beq     .ns
        moveq   #1,d2
.ns:    dbra    d1,.nb
        dbra    d5,.n
        bra     .out
.prot:  move.l  d4,d0
        lsl.l   #6,d0
        bsr     bv_load
        bsr     bc_decode
        lea     F_U(a6),a0
        moveq   #31,d1
.pc:    move.b  (a0)+,d0
        cmp.b   (a3)+,d0
        beq     .ps
        moveq   #1,d2
.ps:    dbra    d1,.pc
        addq.l  #1,d4
        dbra    d5,.prot
.out:   tst.w   d2
        movem.l (sp)+,d2-d5
        rts

; ---- the block code (Nephrite_Bram.md §6)

; F_U's 32 bytes coded into F_SYM's 64 stored bytes.
bc_encode:
        movem.l d2-d6/a2-a3,-(sp)
        lea     F_U(a6),a0          ; §6.1: CRC-16 with 0x1021 from 0, most significant bit first
        moveq   #0,d0
        moveq   #31,d2
.byte:  moveq   #0,d1
        move.b  (a0)+,d1
        lsl.w   #8,d1
        eor.w   d1,d0
        moveq   #7,d3
.bit:   add.w   d0,d0
        bcc     .nb
        eor.w   #$1021,d0
.nb:    dbra    d3,.bit
        dbra    d2,.byte
        lea     F_M(a6),a1
        move.w  d0,d1
        lsr.w   #8,d1
        move.b  d1,(a1)+
        move.b  d0,(a1)+
        lea     F_U(a6),a0
        moveq   #31,d2
.cp:    move.b  (a0)+,(a1)+
        dbra    d2,.cp
        not.w   d0
        move.w  d0,d1
        lsr.w   #8,d1
        move.b  d1,(a1)+
        move.b  d0,(a1)+
        bsr     bc_split            ; §6.2
        lea     bc_order(pc),a2     ; §6.3: each codeword's two parity symbols
        lea     F_SYM(a6),a3
        moveq   #7,d6
.cw:    moveq   #0,d4
        moveq   #0,d5
        moveq   #5,d2
.d:     moveq   #0,d3
        move.b  (a2)+,d3
        move.b  (a3,d3.w),d1
        eor.b   d1,d4
        move.b  d5,d0
        bsr     gf_alpha
        eor.b   d1,d0
        move.b  d0,d5
        dbra    d2,.d
        move.b  d5,d0
        bsr     gf_alpha
        bsr     gf_alpha
        eor.b   d4,d0
        moveq   #GF_INV3,d1
        bsr     gf_mul
        moveq   #0,d3
        move.b  (a2)+,d3
        move.b  d0,(a3,d3.w)
        eor.b   d0,d4
        move.b  (a2)+,d3
        move.b  d4,(a3,d3.w)
        dbra    d6,.cw
        bsr     bc_columns          ; §6.4
        movem.l (sp)+,d2-d6/a2-a3
        rts

; F_SYM's 64 stored bytes decoded into F_U, one symbol error a codeword corrected, the low bits and the CRC not used
; (Nephrite_Bram.md §6.7); d0.w 0 whole, 1 corrected, -1 a codeword past correcting.
bc_decode:
        movem.l d2-d6/a2-a3,-(sp)
        lea     F_SYM(a6),a0
        moveq   #63,d1
.sh:    move.b  (a0),d0
        lsr.b   #2,d0
        move.b  d0,(a0)+
        dbra    d1,.sh
        moveq   #0,d6
        lea     bc_order(pc),a2
        lea     F_SYM(a6),a3
        moveq   #7,d5
.cw:    moveq   #0,d4
        moveq   #0,d0
        moveq   #7,d2
.h:     moveq   #0,d3
        move.b  (a2)+,d3
        move.b  (a3,d3.w),d1
        eor.b   d1,d4
        bsr     gf_alpha
        eor.b   d1,d0
        dbra    d2,.h
        move.b  d4,d1
        or.b    d0,d1
        beq     .next
        tst.b   d4
        beq     .bad
        tst.b   d0
        beq     .bad
        move.b  d0,d1
        move.b  d4,d0
        moveq   #7,d2
.f:     cmp.b   d1,d0
        beq     .fix
        bsr     gf_alpha
        dbra    d2,.f
        bra     .bad
.fix:   moveq   #0,d3
        move.b  -8(a2,d2.w),d3
        eor.b   d4,(a3,d3.w)
        tst.w   d6
        bmi     .next
        moveq   #1,d6
        bra     .next
.bad:   moveq   #-1,d6
.next:  dbra    d5,.cw
        bsr     bc_join
        lea     F_M+2(a6),a0
        lea     F_U(a6),a1
        moveq   #31,d1
.u:     move.b  (a0)+,(a1)+
        dbra    d1,.u
        move.w  d6,d0
        movem.l (sp)+,d2-d6/a2-a3
        rts

; F_M's 36 bytes as 48 six-bit symbols into F_SYM, most significant bit first.
bc_split:
        movem.l d2-d3,-(sp)
        lea     F_M(a6),a0
        lea     F_SYM(a6),a1
        moveq   #11,d2
.g:     moveq   #0,d0
        move.b  (a0)+,d0
        lsl.l   #8,d0
        move.b  (a0)+,d0
        lsl.l   #8,d0
        move.b  (a0)+,d0
        lsl.l   #8,d0
        moveq   #3,d3
.s:     rol.l   #6,d0
        move.b  d0,d1
        and.b   #63,d1
        move.b  d1,(a1)+
        dbra    d3,.s
        dbra    d2,.g
        movem.l (sp)+,d2-d3
        rts

; F_SYM's first 48 symbols back into F_M's 36 bytes.
bc_join:
        movem.l d2-d3,-(sp)
        lea     F_SYM(a6),a0
        lea     F_M(a6),a1
        moveq   #11,d2
.g:     moveq   #0,d0
        moveq   #3,d3
.s:     lsl.l   #6,d0
        move.b  (a0)+,d1
        and.b   #63,d1
        or.b    d1,d0
        dbra    d3,.s
        move.l  d0,d1
        swap    d1
        move.b  d1,(a1)+
        move.w  d0,d1
        lsr.w   #8,d1
        move.b  d1,(a1)+
        move.b  d0,(a1)+
        dbra    d2,.g
        movem.l (sp)+,d2-d3
        rts

; Each grid column's 16 check bits from its eight symbols by §6.4's register, each symbol then put in bits 7-2.
bc_columns:
        movem.l d2-d6/a2,-(sp)
        lea     F_SYM(a6),a0
        moveq   #7,d6
.col:   moveq   #0,d4
        moveq   #0,d5
        moveq   #7,d3
.row:   move.w  d4,d0
        rol.w   #2,d0
        and.w   #3,d0
        lsl.w   #2,d4
        move.w  d0,d1
        eor.w   d1,d4
        lsl.w   #4,d1
        eor.w   d1,d4
        lsl.w   #2,d1
        eor.w   d1,d4
        lsl.w   #2,d1
        eor.w   d1,d4
        move.b  (a0,d5.w),d0
        lea     bc_tbits(pc),a2
        moveq   #5,d2
.b:     move.w  (a2)+,d1
        lsr.b   #1,d0
        bcc     .n
        eor.w   d1,d4
.n:     dbra    d2,.b
        addq.w  #8,d5
        dbra    d3,.row
        moveq   #56,d5
        moveq   #7,d3
.put:   move.b  (a0,d5.w),d0
        lsl.b   #2,d0
        move.w  d4,d1
        and.b   #3,d1
        or.b    d1,d0
        move.b  d0,(a0,d5.w)
        lsr.w   #2,d4
        subq.w  #8,d5
        dbra    d3,.put
        addq.l  #1,a0
        dbra    d6,.col
        movem.l (sp)+,d2-d6/a2
        rts

; d0.b times alpha in GF(64) by x^6+x+1.
gf_alpha:
        add.b   d0,d0
        btst    #6,d0
        beq     .r
        eor.b   #$43,d0
.r:     rts

; d0.b times d1.b.
gf_mul: movem.l d2-d3,-(sp)
        moveq   #0,d2
        moveq   #5,d3
.l:     add.b   d2,d2
        btst    #6,d2
        beq     .a
        eor.b   #$43,d2
.a:     btst    d3,d1
        beq     .b
        eor.b   d0,d2
.b:     dbra    d3,.l
        move.b  d2,d0
        movem.l (sp)+,d2-d3
        rts

; §6.3's table: each codeword's six data symbols, then its two parity symbols, by index.
bc_order:
        dc.b    0, 9, 18, 27, 36, 45, 48, 56
        dc.b    1, 10, 19, 28, 37, 46, 49, 57
        dc.b    2, 11, 20, 29, 38, 47, 50, 58
        dc.b    3, 12, 21, 30, 39, 8, 51, 59
        dc.b    4, 13, 22, 31, 40, 17, 52, 60
        dc.b    5, 14, 23, 32, 41, 26, 53, 61
        dc.b    6, 15, 24, 33, 42, 35, 54, 62
        dc.b    7, 16, 25, 34, 43, 44, 55, 63

; §6.4's T[b], a symbol bit's contribution to the register.
bc_tbits:
        dc.w    $000E, $003E, $00FE, $03FE, $0FFE, $3FFE

; The footer a format writes (Nephrite_Bram.md §4), the counts filled in.
bram_footer:
        dc.b    $5F,$5F,$5F,$5F,$5F,$5F,$5F,$5F,$5F,$5F,$5F,$00,$00,$00,$00,$40
        dc.b    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0
        dc.b    "SEGA_CD_ROM",0,1,0,0,0
        dc.b    "RAM_CARTRIDGE___"
        even
