# Nephrite_Disputes — the disputes log

*Created 2026-10-05, at stage 3, with its first entry.* This is the log `Nephrite_Plan.md` §1.4 defines, in the shape
`VenusRT_Disputes.md` gives. An entry records a rule the documents and the test programs leave open or in
disagreement, the evidence taken, and the rule as implemented. It is the record of which sources a dispute step read,
so that the clean-room protocol of `Nephrite_Plan.md` §1.3 can be audited from outside. The Beryl crates keep their
CPUs' disputes in their own record pages.

## The rules this log keeps

- **A dispute step is separate.** It ends with a rule stated in hardware terms, and a writer's step implements it and
  cites the entry.
- **The order of recourse is fixed:** the documents, then a test program written for the purpose, then the MiSTer RTL,
  run as a black box where it can be. No emulator source is ever read.
- **Every RTL file opened is listed** with the lines read and what for. Nothing from it is transcribed, here or in
  the code.
- **An entry is never deleted.** A conclusion later overturned is struck through and followed by the new one.

## Entry shape

```
### D-<n>. <component>: <the rule, one sentence, in hardware terms>
- Opened: <date>, by <what left it open or showed the disagreement>
- Documents read: <title, section> for each
- Test program: <the program written or run, and what it showed>
- Referee: <RTL files and lines read, and how it was run>
- Conclusion: <the rule as implemented, in prose>; argued | measured | open
- Pinned by: <the test that fails if the rule is changed>
- Implemented in: <the commit that cites this entry>
```

## Entries

### D-1. The Z80's window onto the 68000's bus: the waits each access costs the two processors, and BUSREQ's latency
- Opened: 2026-10-05, at stage 3 step 2. Stage 3 step 1 (`Nephrite_Native.md` §9.2) took three T-states for the Z80
  and three 68000 clocks for the 68000 per window access, and BUSREQ granted at the 68000's next instruction, as
  provisional values; the plan's §2.3 lists the Z80's waits as thin.
- Documents read: Charles MacDonald, "Sega Genesis hardware notes" §2.2-§2.3 (BUSREQ, RESET and the bank; no timing);
  Sega, Genesis Technical Overview 1.00, §4 "Z80 Control" and §5 "Z80 Area" (pp. 77-78) and "68K control of Z80"
  (p. 91: the procedure, and the Z80's interrupt "generated 16ms period and 64ms length", read as 64 µs); Sega, Genesis
  Technical Bulletins, "Precaution when accessing the Z80 bus from a 68000 main" and "Reading the controller pads" (the
  wait of a Z80 access to `$A100xx`, "250ns to 110ns", without the window's own). None gives the window's waits or
  BUSREQ's latency.
- Test program: none in the corpus measures them (searched 2026-10-05: the console test programs of the plan's §3.1,
  the r57shell and SpritesMind folders).
- Referee: Nuked-MD's whole board, `md_board`, compiled with Verilator 5.046 and run as a black box by a bench of its
  pins (`EmuSen.WiseMan/Reference/rtl68k/tb_md.cpp`; programs and readings by `mdboard.py`; the FPGA memory block its
  VRAM names replaced by a behavioural one written from the instance's port list, `vram_ip.v`). The programs, written
  for the purpose and hand-assembled: the 68000 loads a Z80 loop that reads banked cartridge ROM (or, as the control,
  its own RAM) and then counts in a loop in its own RAM, so that the cartridge's pins show only the Z80's window reads
  and the RAM's pins the 68000's rounds; a second program requests the bus, polls until it is granted and releases
  it, marking each in RAM. **Progress:** the board builds (17 s) and boots the program after a power-on reset of its
  own of about 1,400,000 MCLK2 cycles; the 68000 runs the loader and copies the Z80's program. **Not yet measured:**
  every word the 68000 copies to its RAM arrives as `$FFFF` though the cartridge's pins drove `$41F9`, so the RAM loop
  never runs. A defect of the bench's model of the cartridge or the RAM is suspected (a ROM's hold time after OE was
  added, without effect) and is the next step's first task.
  Files read in this step: `rtl/nuked-md/md_board.v`, the port list (lines 28-137), the instance names (`fc1004 ym`,
  `m68kcpu m68k`, `z80cpu z80`, `vram vram1`, `vram vram2`), the pin assignments of the RAMs (lines 678-685) and the
  data and address buses' multiplexing (lines 725-800, 820-835), to know how the pins are driven; `rtl/nuked-md/vram.v`,
  the `vram_ip` instance (lines 46-57); `rtl/ram_md.v`, the `vram_ip` module's port list (lines 1-20). No other RTL
  file was opened; the 68000's and the Z80's own were run, not read, as in `Beryl_M68k.md` §6.
  **Stage 3 step 3 (2026-10-05): the `$FFFF`s were the bench's.** A trace of the pins (`TB_TRACE`) showed `cart_cs`
  and `cart_oe` low through the 68000's write of its own RAM at `$FF0000`: the board leaves both low on cycles that
  are not the cartridge's, and the bench's cartridge drove the data bus whenever they were low, so the RAM took the
  ROM's `$FF`s. The bench's cartridge now answers only its own range, the first 4 MiB, as a cartridge decodes it
  from the address lines; with that the copy arrives and the RAM loop runs. Two more facts of the pins, read from the
  port list and confirmed on the trace: the RAM's address lines are ordered `{ VA14, IA14, VA12-VA0 }`, so `$FF1000`
  shows as `$5000` on them; and a write to the cartridge's range shows as `cart_lwr` or `cart_uwr` high (positive
  logic) with `cart_cs` high. **The measurements**, the 68000 counting in its RAM while the Z80 reads (or writes) the
  cartridge through the window, over 1,275 to 1,875 of the Z80's rounds (`mdboard.py window`, `window-write`):
  the 68000's round, `move.l d0,(a0)`, `addq.l`, `bra.s`, 30 clocks by the manual, takes 30.75 without the window;
  the Z80's, `ld a,(nn)` (or `ld (nn),a`) and `jr`, 25 T-states, takes 27.75 with the window, so **an access waits
  2.75 T-states** on average, reads and writes alike; the Z80's rounds, seen on the cartridge's pins, fall on the
  68000's clock (multiples of 14 MCLK2: 798 to 882), the wait depending on where the access meets the 68000's bus
  cycle. **The 68000 loses 9.5 of its clocks an access** (133 MCLK2), from its rounds beside the accesses against the
  control's. **BUSREQ** (`busreq`, `busreq-tight`): a request written and the grant polled at once found the bus
  granted at the first poll in all 667 requests with the Z80 reading the window and all 739 with it in its own RAM,
  the grant arriving within about 16 of the 68000's clocks of the request's write; that is Nephrite's rule, the
  grant at the next instruction boundary. No RTL file was opened in this step; the board was run as before.
- Conclusion: ~~open. Nephrite keeps the provisional values of `Nephrite_Native.md` §9.2.~~ **Measured** (stage 3
  step 3): a window access costs the Z80 41 master clocks (2.75 T-states, its mean) and the 68000 66 (9.5 of its
  clocks, 66.5, rounded down), paid at the 68000's next access; BUSREQ is granted at the 68000's next instruction
  boundary. The alignment the wait depends on is not modelled; the means are.
- Pinned by: `genesis.rs`'s `WINDOW_Z80_WAIT` and `WINDOW_68K_STALL`, and the corpus's anchors
  (`Nephrite_Native.md` §11.1); no test program reaches the window's timing.
- Implemented in: the commit "Nephrite, stage 3 step 3".

### D-2. OPEN. The 68000 loses clocks to the main RAM's refresh, on the cartridge's bus as well as its own RAM
- Opened: 2026-10-05, at stage 3 step 3, by Sonic the Hedgehog 2 running about 12 frames ahead of Genesis Plus GX
  (`Nephrite_Native.md` §11.3): the lead starts in its boot program's checksum of the ROM, a loop of the 68000 alone
  over the cartridge, which Nephrite times by the 68000's manual.
- Documents read: the MegaDrive Wiki's "Mega Drive" article (`railgun/Mega_Drive.wiki`, "Hardware"): the main RAM
  is "refreshed every 128 68k clocks - if accessing it during refresh, it will be delayed one 68k clock". It says
  nothing of the cartridge.
- Test program: none in the corpus measures it.
- Referee: Nuked-MD's board, run as a black box (`mdboard.py rom-loop`): `add.w (a0)+,d1`, `cmp.l a0,d0`, `bcc.s`,
  24 clocks by the manual, with the Z80 held in reset. Summing the cartridge, 2,341 rounds take 24.38 clocks on
  average: 1,895 take 24 and 446 take 26, one loss of two clocks every 128.0 clocks. Summing the 68000's RAM, 2,288
  rounds take 24.94: 1,532 take 24, 298 take 26, 370 take 27 and 88 take 29. No RTL file was opened.
- Conclusion: open. The cartridge's figure agrees with the wiki's period and not with its one clock; the RAM's needs
  a model of where in the 68000's bus cycle the refresh falls, which these loops do not separate. Nephrite takes no
  refresh yet.
- Pinned by: nothing yet.
- Implemented in: not yet.

### D-3. Shadow/highlight: palette 3's colour 14 brightens and colour 15 darkens, and an operator pixel gives no priority
- Opened: 2026-10-05, at stage 4 step 2, by two documents in disagreement. MacDonald's "Sega Genesis VDP
  documentation" §16 has colour `$3E` drawing the pixel under it at half intensity and `$3F` at double; plutiedev's
  "Shadow/highlight" has palette 3's colour 14 brighter and colour 15 darker, layers' priority deciding the shadow
  with "transparent pixels in sprites" excluded, colour 14 of any palette never darker, and brighter and darker
  together giving a normal pixel.
- Documents read: those two; TmEE's measured levels (SpritesMind topic 2188: shadow and highlight add no levels).
- Test program: the shadow/highlight programs of Genesis Plus GX's published `md_test` collection (`SHLTEST.BIN`,
  `SHLTEST2.BIN`, `stetest.bin`, `STETEST2.bin`), test programs whose pictures were compared, not read as code.
- Referee: Genesis Plus GX run as a black box through the probe, the picture at frame 300 compared up to a one-to-one
  colour map. With plutiedev's rule `SHLTEST`, `SHLTEST2` and `stetest` match exactly. `STETEST2` first differed in its
  third row, which Nephrite drew highlighted and the reference at normal intensity, until an operator pixel stopped
  counting as a high-priority sprite pixel (it is transparent); then its picture matches but for pixels whose
  intensities the measured ladder maps to one level (shadow's step 4 and normal's step 2, for instance), which a
  one-to-one map cannot follow. MacDonald's assignment would swap the operators' rows; neither reference nor test
  agrees with it.
- Conclusion: **argued from plutiedev and the references' pictures**: as plutiedev states, with an operator pixel
  transparent for priority. No console photograph of these programs is in the corpus.
- Pinned by: nothing yet in the crate (the comparison needs the reference); `Nephrite_Native.md` §14.3 records it.
- Implemented in: the commit "Nephrite, stage 4 step 2".

### D-4. The V counter in PAL and V30, the interlaced fields, and NTSC's V30: measured on the board
- Opened: 2026-10-05, at stage 4 step 3, by step 1's argued PAL jumps (`Nephrite_Native.md` §13.1: MacDonald gives
  NTSC's jump from `$EA` to `$1E5`; no document in the corpus gives PAL's, nor the counter in V30 or interlace).
- Documents read: MacDonald, "Sega Genesis VDP documentation" §5 (the HV counter: NTSC V28's jump; "VC0 is replaced
  with VC8 when in interlace mode 2", from the manual); plutiedev, "Screen resolution" (V30 "only works properly on
  PAL systems ... the console will misbehave" on NTSC, without saying how; double resolution's fields and the status
  register's bit 4, 0 for even lines and 1 for odd).
- Test program: the bench's own, `mdboard.py vcounter` and `interlace`: the 68000, from its RAM, sets register 1
  (V28 or V30) and register 12 (H40 or H32, and the interlace mode), then reads the HV counter in a loop and stores
  it to RAM, the bench printing each store; the V counter's changes and the H counter beside each give the jumps,
  where the counter steps and how many lines a field has. `TB_PAL` in the bench's environment makes the board PAL
  (an empty value counts as set). And a picture program, `mdboard.py picture ntsc-v30` (rows of colour in V30 on an
  NTSC board), whose display enables the bench captures.
- Referee: Nuked-MD's board run as a black box, as in D-1. **NTSC V28** jumps from `$EA` to `$1E5`, as MacDonald
  says; **NTSC V30** does not jump: the counter runs on to `$1FF`, and the board's vertical display enable rises
  every 512 lines, 240 shown, with one vertical sync in the capture (the "misbehaving" of plutiedev's page: a frame
  of 512 lines). **PAL V28** jumps from `$102` to `$1CA` and **PAL V30** from `$10A` to `$1D2`, the values step 1
  argued. The counter steps where Nemesis's tables put it, at the 8-bit H `$A5` in H40 and `$85` in H32. In
  **interlace mode 1** the port gives the counter's low byte with bit 0 replaced by bit 8; in **mode 2** the counter
  doubled, bit 8 below it. **The fields**: NTSC alternates 262 and 263 lines, the longer jumping one line lower
  (`$1E4`); PAL alternates 313 and 312, jumping one line earlier, after `$101`, to `$1C9` or `$1CA`. PAL V30's
  picture (`picture pal-v30`, its header Europe's) shows 240 lines and matches Nephrite's exactly.
  Files read in this step: the port list of the bench's generated model header (`obj/Vmd_board.h`, lines 31-109),
  to find the board's video pins (`V_R`, `V_G`, `V_B`, `vdp_de_h`, `vdp_de_v`, `V_HS`, `V_VS`, `vdp_hclk1`,
  `vdp_intfield`, `vdp_m5`); no RTL source file was opened.
- Conclusion: **measured**: the jumps and formats above; NTSC V30 a frame of 512 lines with 240 shown; V30 shows
  240 lines on either board. Which of the two fields is the longer one is argued (the odd one), since the counter
  program did not read the status register beside it.
- Pinned by: `vdp.rs`'s `the_v_counter_jumps_where_the_board_does`, and `pictures.rs`'s
  `ntsc_v30_runs_the_counter_to_512_lines`.
- Implemented in: the commit "Nephrite, stage 4 step 3".

### D-5. MacDonald's window bug takes only the fine scroll's partial column, and the window's registers take effect at the next line
- Opened: 2026-10-05, at stage 4 step 3, by MacDonald's own test program and Genesis Plus GX disagreeing: the
  program's comment (`window_distortion_src/MAIN.ASM`) says the two plane A columns after a left window "have their
  name table data fetched from the next two columns over" when plane A's scroll has its low four bits set; the
  reference, at frame 300 of the same program, shows no shift; Nephrite then shifted 16 pixels.
- Documents read: the program's source comment; MacDonald §17 (registers 17 and 18).
- Test program: MacDonald's "Window distortion bug" (its source in the corpus), with its `lsr.w #4,d0` made a `nop`
  so that the scroll steps every frame, run on the board through 16 frames (every fine scroll); its RAM clear
  shortened for the bench's sake (the board's RAM starts zero).
- Referee: the board as a black box, its pictures taken from the video pins (`tb_md.cpp`'s `TB_PICTURE`: the pins
  each MCLK2 cycle, cut into frames by the display enables and sampled mid-pixel on `vdp_hclk1`). With fine scroll
  `s`, the pixels from the window's edge to the edge plus `s` show plane A as it lies 16 pixels on, and the rest of
  the line its own: the partial 2-cell column the scroll exposes takes the next column's data; at `s` 0 nothing is
  shifted. The line on which the program turns the window off, after reading the V counter at 111, still shows the
  window: a write to register 17 during a line moves the next line's window. Nephrite with both rules matches the
  board's 16 frames exactly up to the colour map, and Genesis Plus GX's frame 300 too.
- Conclusion: **measured**: as stated, registers 17 and 18 taken as the line begins, beside the scroll values.
- Pinned by: `programs.rs`'s `the_window_bug_takes_the_partial_column_from_the_next` (fails with a 16-pixel band and
  with the registers read at the line's end; both mutations tried).
- Implemented in: the commit "Nephrite, stage 4 step 3".

### D-6. A CRAM dot is the pixel at H minus $18, and the first blank line keeps the active slots until H $14E or $10E
- Opened: 2026-10-05, at stage 4 step 3: no document places the dots (MacDonald calls them "CRAM write dots", and
  the reference draws none), and the step's first placement put them two pixels left of where the board shows.
- Documents read: MacDonald §16; the Exodus collection's "CRAM flicker" program (no source).
- Test program: `mdboard.py picture cram-dots` and `cram-dots-h32`: the display all backdrop and the 68000 writing
  CRAM entry 2, which nothing shows, as fast as the FIFO takes it; and the Exodus "CRAM flicker" program to frame 32.
- Referee: the board as in D-5. The dots fall on the pixel at H minus `$18` in both widths, one pixel each; those the
  FIFO lands after the V counter's step fall on the last pixels of the line before (to x 315); on the last shown
  line the FIFO's queued writes drain into the first blank line's free slots from H `$14F` (H40) and `$10F` (H32),
  not from the line's start. With these, every line of the picture programs matches but seven of H40's 224, where
  the 68000's loop meets the slots a write apart (the refresh's loss of D-2, which Nephrite does not take, is the
  likely cause), and on the last line of each width one dot more in Nephrite, the 68000's write after the drain; the CRAM flicker program's dots match on every line of two
  frames and all but one of the third.
- Conclusion: **measured**, as stated.
- Pinned by: `pictures.rs`'s `cram_dots_fall_where_the_board_shows_them`.
- Implemented in: the commit "Nephrite, stage 4 step 3".

### D-7. Mode 4 on the Genesis: its map of VRAM, its step of one, its CRAM, and its colours
- Opened: 2026-10-05, at stage 4 step 3: the documents describe mode 4 as the Master System's (MacDonald's "SMS VDP
  documentation" §7-§10 and §13, his Genesis document's register 1, bit 2) and say nothing of where its 16 KiB lie
  in the Genesis's 64 or of its colours; the step's first renderer, reading VRAM as the Master System's, drew the
  board's picture as noise.
- Documents read: those; MacDonald's `newreg.txt` (the left column's garbage under a fine scroll).
- Test program: `mdboard.py mode4-ports`: in mode 4, bytes written after one-word commands, a marker at each address
  bit, words at an even and an odd address, four bytes with register 15 at 2, and CRAM's 32 bytes; then mode 5
  reads all of VRAM and CRAM back to RAM. And four pictures: patterns, names and sprites written in mode 5 and shown
  in mode 4; the same written in mode 4 a byte at a time; the 64 Master System colours in mode 4 (`mode4-colours-0`
  and `-32`).
- Referee: the board as in D-5. **VRAM**: mode 4's address bits 1-8 lie one place up in VRAM's, its bit 9 at bit 1,
  bits 10-13 in place; a write writes the word there (a 68000 byte write, which puts the byte on both halves, fills
  both bytes), an odd address swapping the word's bytes as in mode 5; the renderer reads an even address's byte from
  the word's low byte. **The step** is one whatever register 15 holds (which mode 4 does not take). **CRAM**: an entry
  a byte address, keeping the word's bits 0-2, 3-5 and 9-11 as the entry's red, green and blue. **Colours**: the
  entry's red and green fields hold the Master System's six bits, `--BBGGRR`, each two bits shown at the board's
  levels 0, 95, 161, 255 (blue 0, 109, 161, 255). With these, the two colour pictures match exactly and the other
  two but for the left five pixels, where under a scroll of 5 the board shows the garbage `newreg.txt` reports.
- Conclusion: **measured**, as stated; the left column's garbage under a fine scroll is not modelled (the wrapped
  tile is drawn). The levels are the board's DAC model's; no measurement of a console's mode 4 output is in the corpus.
- Pinned by: `pictures.rs`'s `mode_4_writes_land_where_the_board_put_them` and `mode_4_shows_the_master_system_colours`.
- Implemented in: the commit "Nephrite, stage 4 step 3".

### D-8. The eight-colour mode, double resolution's patterns, and which rows a field shows
- Opened: 2026-10-05, at stage 4 step 3. MacDonald §17 says register 0's bit 2 clear keeps "only the LSB of each
  colour component" without saying at what level it is shown; he says double resolution ignores a name's bit 0,
  where plutiedev calls an 8x16 tile "two consecutive 8x8 tiles", which puts name n at n x 64 instead; Genesis Plus
  GX's `wtest_i2` drew its window from tile 2 as n x 64 has it.
- Documents read: MacDonald §6, §9, §17; plutiedev, "Screen resolution"; the MegaDrive Wiki's VDP article.
- Test program: `mdboard.py picture palette64` and `palette8` (64 colours whose components step differently, with
  the bit set and clear); `interlace1` and `interlace2` (16-row cells, each row its own colour); `field-colour`
  (every pixel colour 1, which the 68000 sets in vertical blanking from the status register's odd-field bit).
- Referee: the board as in D-5. The eight-colour mode shows each component's lowest bit at the level of step 1, and
  matches Nephrite exactly with that rule. In double resolution each field is 224 lines: the board's fields, its
  field pin alternating, are the odd and even rows of Nephrite's 448 exactly, with name n at n x 64; the field whose
  pin is set, and in which the status register's bit 4 reads 1, shows the odd rows.
- Conclusion: **measured**, as stated. **Argued**: Nephrite draws both rows of each line every field, from the
  state of the field being drawn, where the board shows only the field's own rows. The other field's rows are not
  the VDP's state, and keeping them would put a 448-line picture into every state (the kit's C7 and C8 require the
  picture to follow from the state); a program that changes the picture between fields (as `field-colour` does)
  shows both rows in the colour of the field drawn.
- Pinned by: `pictures.rs`'s `the_eight_colour_mode_keeps_each_components_low_bit`,
  `double_resolution_patterns_are_sixty_four_bytes` and `each_field_draws_in_the_colour_its_status_bit_chose`.
- Implemented in: the commit "Nephrite, stage 4 step 3".

### D-9. Mid-line changes: where each kind of write takes effect along the line
- Opened: 2026-10-05, at stage 4 step 4, by the slot-stamped renderer of `Nephrite_Plan.md` §5.2: no document says
  how far along a line a write to CRAM, VSRAM, VRAM or a register shows, and Genesis Plus GX applies them a line at a
  time.
- Documents read: MacDonald §16 (the CRAM dots); the MegaDrive Wiki's VDP article (access slots, the line's fetches).
- Test program: the bench's, each written for the purpose (`mdboard.py picture <name>`). Transfers from the 68000's
  RAM started by the line interrupt, whose writes land slot by slot whatever the 68000's timing: `cram-dma` (40 CRAM
  entries shown as 40 columns), `cram-dma-one` (one entry under every pixel), `vsram-dma` (bands scrolled per 2-cell
  column), `pattern-dma` (tiles' rows rewritten, inverted every frame). The 68000 reading the HV counter, writing a
  register and storing the reading in a loop: `reg-backdrop` (register 7, H40), `reg-backdrop-h32` and `reg-display`
  (register 1's display bit), the RAM log beside each picture.
- Referee: the board as in D-5. **CRAM**: a write shows from its own pixel, H minus `$18`, the dot of D-6 being that
  pixel in the written colour; with the transfer's start of D-11, `cram-dma` and `cram-dma-one` match the board on
  every row of two frames. **VSRAM**: a column's vertical scroll is read before the column is shown; a write
  reaches the columns whose read comes after it, which with D-11's start matches the board exactly for any lead from
  20 to 34 pixels (outside it, 256 or 272 pixels differ). **VRAM**: the pixels the VDP has fetched already show the
  old patterns; a lead of 4 to 16 pixels leaves 9 pixels on 4 rows of `pattern-dma` (2 pixels each, single pattern
  bytes), 0 leaves 10 and 24 or more leave 15 or more, so the lead is weakly bracketed. **Registers**: the backdrop's
  change falls 9 to 11 pixels after the HV read before the write (the board's 410, 828 and 432 of 1,725 in H40; 835
  and 805 at 8 and 9 in H32), which Nephrite gives with the write shown 20 master clocks (H40) or 25 (H32) after the
  68000 makes it, two and a half pixels. **The display bit**: blanking begins 12 pixels later than that (the board's
  21 to 23 after the read, overlap 0.95 with Nephrite's); the planes come back mostly on a 16-pixel boundary (669 of
  845 changes, 114 on an 8-pixel one, 57 one pixel after), modelled as the first 16-pixel boundary from 24 pixels on,
  which gives 0.56 of the board's distribution. No RTL file was opened.
- Conclusion: **measured**: CRAM at the beam's pixel; VSRAM by column, read 27 pixels before the column is shown;
  registers 2.5 pixels after the write; the display bit's blanking 12 pixels later still. **Measured in part**: VRAM's
  lead of 8 pixels, and the display bit's return.
- Pinned by: `pictures.rs`'s `mid_line_transfers_draw_as_the_board_does` (the board's rows, hashed in
  `board_rows.rs`; it fails with CRAM writes applied at the line's end, 142 rows of 224, and with VSRAM applied at
  once, 192) and `register_writes_show_where_the_board_shows_them` (fails without the write's delay).
- Implemented in: the commit "Nephrite, stage 4 step 4".

### D-10. Register 1's bit 7, the 128 KiB VRAM mode, on a board with 64 KiB: where a write lands
- Opened: 2026-10-05, at stage 4 step 4, by `512PAL`, whose register table sets register 1 to `$FC`: the board's
  picture of it looked like neither Nephrite's nor Genesis Plus GX's, which both ignore the bit.
- Documents read: the MegaDrive Wiki's VDP article (bit 7 "allows use of an additional 64 KB of external VRAM";
  "the additional VRAM is interleaved with the regular 64KB", so that the mode on a console without it garbles the
  graphics; the interleave's reference, Nemesis's SpritesMind post, is not in the corpus).
- Test program: `mdboard.py vram128`: words written with the mode set to an address at each address bit, an odd
  address and a run of eight, then VRAM read back with it clear, and read with it set.
- Referee: the board as in D-1. A word written in the mode stores one byte, the word's low one, at the byte address
  with address bit 1 inverted at bit 0, bits 2-9 in place, bit 10 at bit 1 and bits 11-15 one place down; a read in
  the mode returns that byte as the word's low half (its high half the last written word's high byte). With the
  writes modelled, `512PAL`'s picture differs from the board's by 10,544 pixels where it differed by 65,348; the
  rest is how the display reads VRAM in the mode, which a fetch of the low byte at the same address with 0 or `$FF`
  for the missing one made worse (38,052 and 43,796), and the 68000's timing. The same program with the bit cleared
  differs from the board by 2,028 to 2,208 pixels, on 64 rows: its line interrupt's CRAM writes begin one to four
  slots earlier in Nephrite, the 68000 running its two-line handler a little faster than the board's (D-2's
  class).
- Conclusion: **measured** for writes, which Nephrite models; reads through the port and the display's fetches in
  the mode are open (Nephrite reads VRAM as in 64 KiB mode).
- Pinned by: nothing yet in the crate; `Nephrite_Native.md` §16.3 records the comparison.
- Implemented in: the commit "Nephrite, stage 4 step 4".

### D-11. The FIFO's write path: a word reaching an empty FIFO is written 176 master clocks on, a transfer reads 88 after its command
- Opened: 2026-10-05, at stage 4 step 4, by the transfer pictures of D-9 landing every write one slot earlier in
  Nephrite than on the board.
- Documents read: none gives the start latency.
- Test program: `mdboard.py picture dma-start-0` to `dma-start-10`: a transfer of 64 words from RAM into one CRAM
  entry from the line interrupt after line 150, its command delayed by 0 to 10 NOPs (28 master clocks each); the
  first colour change along line 150 is the first write. And the last part of VDPFIFOTesting's FIFO Wait States (its
  source in the corpus), repeated 64 times at stepped phases with its samples stored to RAM.
- Referee: the board as in D-1, Nephrite's own command and slot times logged beside it. The 68000's handler runs as
  the board's within a few master clocks (its first fetch 585 to 615 clocks before line 99's pixel 0 against
  Nephrite's 540 to 586). The board's first write lands at the first external slot at least 256 to 284 master clocks
  after the command's second word, Nephrite's at the first slot after the command: with 270 added before the first
  read, all six sweep pictures, `cram-dma`, `cram-dma-one` and `vsram-dma` match the board exactly. But the delay
  breaks VDPFIFOTesting's FIFO Wait States, whose tenth part, on the console, sees the FIFO full after a three-word
  transfer begun with three writes queued; and the board, running that part at 64 phases, shows the busy flag
  (status bit 1) in 53 samples, which Nephrite, holding the 68000 until the transfer's last read, never shows. The
  bench's cartridge does not answer the VDP's reads (the board drives `cart_cs` and `cart_oe` high for them, which
  the bench does not decode), so transfers from the cartridge, VDPFIFOTesting's own, are not measured on the board.
  `PAL512` shows the board's picture as one colour for the same reason. The HV logic-analyser ROM (Nemesis, "hv
  logic analyser noint - H40V28", run with port 1's C held for its start and its 30,722 samples read from the RAM
  log): the V counter steps between the 8-bit H `$9C`-`$9F` and `$A5`-`$A8` on both, but the board's pairs of reads
  8 clocks apart differ by one or two H values where Nephrite's always differ by the same, the VDP's answer to a
  port read not taking a fixed time on the board.
- ~~Conclusion: open. Nephrite keeps no start delay (`Hw::transfer_start_delay` 0), VDPFIFOTesting's 122 passing; the
  tests set 270 to hold the renderer to the board. The next dispute step: the bench's cartridge for the VDP's reads,
  then the latency with a cartridge source against VDPFIFOTesting, the busy flag at the 68000's release, and the
  port's read time.~~ Superseded by the second step, below.
- Second step (2026-10-05, the stage's dispute step). **The bench's cartridge**: during a transfer from the cartridge
  the board raises `cart_dma` and `vdp_dma` and leaves `cart_cs` high, as the bench's trace of the pins shows;
  `tb_md.cpp` now answers the cartridge's range then as well. A transfer from `$4000`
  (`mdboard.py picture dma-start-rom-0` to `-10`) lands where the same transfer from RAM does, and `PAL512`, one
  colour on the board before, now matches Nephrite at frames 3 to 5 on every pixel, up to a one-to-one map of
  colours.
- Second step's test programs: `mdboard.py picture write-landing-10` to `-24` (and `-off-`, `-h32-`, `-off-h32-`):
  the line interrupt after line 150 sets CRAM's address, waits 10 to 24 NOPs and writes one word through the data
  port, so the first changed pixel on line 150 is the write; in H40 and H32, the display on and off. The sweep
  pictures above, again. And `mdboard.py picture fifo-wait-states`, the first step's replica of VDPFIFOTesting's
  tenth part, now a named program.
- Second step's referee: the board as in D-1; no RTL file was opened. The 68000's single writes land on the board
  176 master clocks after the word reaches an empty FIFO, at the first slot from then: with that start Nephrite
  lands all sixteen points with the display on exactly (in both widths), and with the display off five of eight
  (H40) and four of eight (H32) on the board's pixel, the rest 2 to 4 pixels away (the board itself alternates
  between two pixels 2 to 6 apart from frame to frame). A transfer's first read follows its command by 88 master
  clocks, and its first word, reaching an empty FIFO, waits the same start: 264 in all, inside the first step's
  bracket of 256 to 284, and the transfer sweeps (`dma-start-*` from RAM and from the cartridge) match the board at
  all twelve points. The first step put the whole delay before the transfer's first read, which is why it broke
  VDPFIFOTesting: its tenth part queues three writes before the transfer, so the transfer's words reach a FIFO that
  is not empty and follow without the start, filling it as the console sees; the start belongs to the write path,
  not to the transfer. A start of 264 before the first read, with the transfer's words exempt from the write path's
  start, fails that part (the FIFO never full). **The busy flag**: with the cartridge answering, the board running
  `fifo-wait-states` at 64 phases shows no busy sample (753 samples empty, 335 neither), and neither does
  Nephrite; on both the FIFO's empty flag returns 7 to 10 samples after the command. The 53 busy samples of the
  first step were the bench's: its cartridge did not answer the transfer's reads. The board shows the FIFO full at
  none of the 64 phases and Nephrite at one; the console sees it within 2,048 iterations, which 64 phases sample too
  coarsely to test. Holding the busy flag while the FIFO drains fails VDPFIFOTesting's FIFO Wait States, as holding
  later control-port writes while it drains fails many parts; the flag is set only while a transfer reads.
  **With the display off, a transfer's words** do not wait the start: MD1536 (the Genesis Plus GX suite's), whose
  colours are made by CRAM transfers with the display off, run on the board to its frame 300 (about forty minutes of
  the bench), is Nephrite's frame 300 on every pixel up to a one-to-one map of colours, and 259 pixels of CRAM dots
  differ when the start is applied to those words. Its early frames (3 to 6, on the board) are the board's either way.
  Why the display's state matters here is not known; the rule is the measurement.
- Conclusion: **measured**, and one rule satisfies the console and the board: a word that reaches an empty FIFO,
  with nothing waiting to be written, is written at the first slot at least 176 master clocks on; a word behind
  others follows them; a transfer from the 68000's bus reads first 88 master clocks after its command and then
  every 20, the 68000 released at its last read; the words of a transfer made with the display off do not wait the
  start; the busy flag is set only while the transfer reads.
  The two measurements did not conflict: the first step's busy samples and its cartridge transfers were the bench's
  unanswered reads, and its delay was placed on the transfer where it belongs to the write path. **Open**: the
  display-off residual of 2 to 4 pixels for single writes, and why a transfer with the display off is exempt.
- Pinned by: `pictures.rs`'s `writes_land_where_the_board_lands_them` (the board's sweeps; it fails with the write
  path's start at 0 and with the transfer's first read at 0) and `mid_line_transfers_draw_as_the_board_does` (fails
  with the start at 0); `programs.rs`'s `vdpfifotesting_passes_every_test` (121 of 122 with the busy flag held while
  the FIFO drains) and `md1536_shows_the_boards_picture` (fails with the display-off transfer's words waiting).
- Implemented in: the commit "Nephrite, stage 4: D-11 settled".

### D-12. The status flags and the port's read: vertical blanking at H `$14C` or `$10C`, the odd flag at the frame interrupt
- Opened: 2026-10-05, at the stage 4 dispute step, by the plan's oracle for the stage (the logic-analyser ROMs
  through the pad-port receiver) and by the HV ROM's reads of D-11's first step, whose durations varied on the board.
- Documents read: MacDonald's "Sega Genesis VDP documentation" (the status register's bits) and his `m5hvc.txt`
  (vertical blanking from H `$A8` of the 8-bit counter in H40 and `$87` in H32).
- Test program: Nemesis's "sr logic analyser noint - H40V28", "- H32V28" and "intnorm - H40V28" and "hv logic
  analyser noint - H32V28" (and "- H40V28", D-11), each run on the board with port 1's C held for its start and its
  samples (status, then the HV counter, eight clocks apart) read from the RAM log in place of the receiver; Nephrite
  runs the same images to frame 30 and its samples are read from its RAM. And programs written for the purpose:
  `mdboard.py picture status-hv-h40` and `-h32` (the status register and the HV counter read as one long in a loop,
  a NOP every other round so that the reads drift across the line) and `status-boot` (the status register at
  power-on and after mode 4 and mode 5 are set, about four frames apart), their samples in the pictures' RAM logs.
- Referee: the board as in D-1; no RTL file was opened. **The V counter** steps at the same H on both. **Horizontal
  blanking** sets and clears on both within one sample of each other (from H `$E4` to `$06`-`$07` in H40, from
  `$E9`-`$EA` to `$05`-`$06` in H32). **Vertical blanking** sets on the board two pixels after the V counter's step to
  line 224, at H `$14C` of the 9-bit counter in H40 and `$10C` in H32, where Nephrite, after `m5hvc.txt`, had `$150`
  and `$10E`; it clears with the step to line 255 on both. **The odd flag**, in "intnorm", changes on the board
  between H `$01` and `$09` of line 224, the frame interrupt's H, where Nephrite had changed it as the line began; it
  now changes with the frame interrupt, between `$FE` and `$0A`. **F**, the frame interrupt pending, sets on line 224
  between H `$FA` and `$10` on both, and stays set in these programs, which do not take the interrupt. `status-boot`'s
  five reads agree but for the hblank bit of two, which depends on where in the line each read falls. **The port's
  read time**: every status and HV read on the board takes the same 56 of the bench's clocks (28 master clocks, four
  68000 clocks); the varying intervals of D-11's first step are the 68000's RAM writes, 24 of 278 taking 98 of the
  bench's clocks where the rest take 56, the refresh of D-2.
- Conclusion: **measured**: vertical blanking at H `$14C` (H40) and `$10C` (H32), not `m5hvc.txt`'s; the odd flag with
  the frame interrupt; horizontal blanking and F as Nephrite had them; the port's read has no wait of its own. The RAM
  write's loss is D-2's, still open.
- Pinned by: `vdp.rs`'s `the_blanking_flag_changes_two_pixels_after_the_v_counter`; the odd flag's time by nothing
  yet.
- Implemented in: the commit "Nephrite, stage 4: D-11 settled".

### D-13. OPEN. The YM2612's status at ports 1-3: the discrete chip of model 1 gives its timer flags without the busy flag there, the model 2 ASIC gives the busy flag at every port
- Opened: 2026-10-05, at stage 5 step 1, by the documents' disagreement over what a read of the YM2612's ports 1-3
  returns.
- Documents read: SpritesMind topic 386 ("New Documentation: An authoritative reference on the YM2612"), as prose:
  Nemesis's status-register section (the normal status register "returned from the YM2612 under all circumstances,
  regardless of the address the YM2612 is being read from"); Stef's later report that the busy bit "can be read only
  on port 0"; Eke's tests on a VA4 model 1 with a discrete YM2612 and a VA0 model 2 with the 315-5660 ASIC ("BUSY flag
  can only be read from port 0 (A0=A1=0) on discrete YM2612 while it can be read from any port on ASIC-integrated
  version"; set only by data writes; 32 internal clocks, 192 of the 68000's, alike on both), with Hellfire's slower
  music on model 2 as its consequence. Sauraen's reading of the die (the busy flag a timer started by a data write) in
  the same thread.
- Test program: none yet. `sounds.rs`'s busy-flag program reads ports 0 and 2 on Nephrite.
- Referee: none yet. The board's OPN2 has an input named `ym2612_status_enable`, which suggests the bench can be run
  as either chip; a dispute step would run a busy-flag program reading every port, the board as a black box.
- Conclusion: open; argued from Eke's tests, the most specific: on the discrete chip ports 1-3 give the timer flags
  with the busy bit clear, on the ASIC every port gives the whole status. Nemesis's statement predates them and is
  read as being about the timer flags.
- Pinned by: `ym2612.rs`'s `the_busy_flag_follows_a_data_write_for_192_of_the_68000s_clocks_and_reads_at_port_0_on_the_discrete_chip`
  and `sounds.rs`'s `a_program_sees_the_busy_flag_for_192_clocks_at_port_0_only`.
- Implemented in: the commit "Nephrite, stage 5 step 1".

### D-14. OPEN. The 68000's accesses to the YM2612 through the Z80's area: how long each takes
- Opened: 2026-10-05, at stage 5 step 1, by `mdboard.py sound dac-square`: a 68000 loop writing the DAC through
  `$A04000` runs at 543.6 Hz on Nephrite and ClownMDEmu and at 535 Hz on BlastEm and PicoDrive, about 110 more of the
  68000's clocks for each half period's two writes.
- Documents read: none gives a wait for the 68000's accesses to `$A00000`-`$A0FFFF` beyond the bus request itself.
- Test program: `dac-square` (above).
- Referee: none yet; the board's pins time a 68000 access to that area directly, as D-1 timed the Z80's window.
- Conclusion: open. Nephrite gives the access the 68000's four clocks, as for any other address.
- Pinned by: nothing yet.
- Implemented in: not yet.
