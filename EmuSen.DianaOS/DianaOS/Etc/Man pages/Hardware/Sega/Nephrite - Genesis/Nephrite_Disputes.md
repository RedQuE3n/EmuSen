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

### D-2. The 68000 loses clocks to two refreshes: the bus's, every 128 clocks, on the cartridge, and the main RAM's, on the RAM
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
- ~~Conclusion: open.~~ The cartridge's figure agrees with the wiki's period and not with its one clock; the RAM's needs
  a model of where in the 68000's bus cycle the refresh falls, which these loops do not separate. ~~Nephrite takes no
  refresh yet.~~
- Measured 2026-10-05, at stage 5's 68000 step, on the board's cartridge reads (the bench's `c` lines) in programs
  written for it: a voice (`fm-dr39`), the busy-flag sweep of D-13, and `mdboard.py`'s `RAM_LOOPS`, unrolled RAM
  reads with 0 to 9 NOPs between and long reads. The board's 68000 was followed read by read against Nephrite's, and
  the board's main-RAM and video strobes logged (`TB_MEM`); no RTL file was opened. There are two refreshes:
  - **The bus's**, every 896 master clocks (128 of the 68000's clocks), its phase in the idle loop's 95 periods
    fixed to within the 68000's clock. The first access at or after one waits two clocks if it is to the
    cartridge's area. If that access is to the Z80's area, the I/O area or the main RAM, it passes without a wait
    and the refresh is spent: seen at a write to `$A11200`, at a read of the YM2612 that began on the refresh, and
    at the first RAM read of the RAM loops. Accesses to the VDP pass too: making them wait breaks
    `register_writes_show_where_the_board_shows_them` and VDPFIFOTesting. This is the 2 clocks in 128 of the
    cartridge loop above.
  - **The main RAM's**, which is not periodic in time. A RAM access within 133 master clocks of a request waits
    three clocks, and the next request comes 806 after that access began; a request no RAM access meets in its 133
    is served without a wait, and the next comes 806 after the 133 run out. The intervals between waits are then
    as the board's: 819 and 861 for reads 56 apart, 931 for reads 224 apart, 875 for reads 140 apart. The rule was
    fitted to the board's reads in ten programs: seven agree read for read (1,000 to 2,900 reads each); three
    keep the board's time until one wait, where the board waits four or five clocks, or none, and the rule three.
    `RAS1` and `CAS1`, the only such strobes among the board's nets, cycle every 20 master clocks with a CAS-before-RAS
    cycle 640 apart, video RAM's by their pace; nothing on the bench shows the main RAM's own refresh, and the rule
    is the timing's, not a mechanism read.
  - **Checked against this entry's first measurement**, which the rules were not fitted to: Nephrite running the
    cartridge loop takes 26 clocks in 19.1% of its rounds (the board 19.0%), the RAM loop 26, 27 and 29 in 13.2%,
    16.2% and 3.7% (the board 13.0%, 16.2% and 3.8%).
  - **Where they fall.** The phases were measured from the board's 68000 start, which comes 0.61 of a frame
    (544,592 master clocks, modulo a frame) later against the picture than Nephrite's: the frame offset of stage 4's
    picture tests, Nephrite's frame 3 the board's 2. Placed by the 68000's start, the bus refresh makes a voice's
    reads the board's to the master clock and leaves one point of D-11's write sweep (H32, the display on, 24 NOPs)
    a slot late; placed against the picture, as the board has it there (762 master clocks after power-on in
    Nephrite), the whole sweep is the board's and a voice's samples still are, its reads 113 master clocks from the
    board's around each refresh. Nephrite places it against the picture. The RAM's, whose next request depends on
    the accesses made, keeps the place fitted from the 68000's start (its first request at 1,414).
- Conclusion: **measured on the board**, the RAM's rule as a fit with the residuals above. Nephrite implements both.
  Open: the RAM's few four- and five-clock waits; why the 68000 starts at another point of the frame than on the
  board, which is stage 4's.
- Pinned by: `genesis.rs`'s `ram_loops_keep_the_boards_time_through_both_refreshes` (three loops read for read
  against `board_bus.rs`, with the bus refresh at the board run's phase; it fails with either refresh taken out) and
  `the_refreshes_cost_the_boards_share_of_rounds` (this entry's loops); `pictures.rs`'s
  `register_writes_show_where_the_board_shows_them` and `writes_land_where_the_board_lands_them` fail with either
  refresh taken out, and the second with the bus refresh at the 68000-start phase.
- Implemented in: the commit "Nephrite, stage 5: the 68000's side".
- 2026-10-06, at stage 5's placement step: **the bus refresh and the Z80's area.** Loops of `$A04000` reads and
  writes with 0 to 7 NOPs, and key sweeps on both parts, with the 68000's strobes logged by `TB_MEM`, show the rule
  this record left out. The refresh is requested every 896 master clocks, its phase fitted from the board's own slow
  cycles: in three loops, 670 of 670 cartridge cycles that are the first after a request wait the two clocks (DTACK
  41 cycles of the bench's clock after AS, not 13). Where the first cycle after a request is to the Z80's area, one
  that starts within a clock of it (6 master clocks) has the ordinary five clocks (27) and the cartridge cycle after
  it no wait, so it took the refresh free; one that starts two or four clocks after it (14, 28) is a clock longer
  (41). Nephrite holds a cycle to the Z80's area that starts one to six of the 68000's clocks after the latest
  request (`REFRESH_Z80_LATE`). The held cycle's strobe on the Z80's bus is held with it, and the YM2612 takes the
  write where the strobe falls and 48 cycles of the bench's clock, where it would rise without the wait: of 36,180
  data writes in the sweeps and status loops, the busy flag of every one ends where D-13's rule has it counted from
  that place. With both, all 83,451 of the 68000's cycles in 16 traced runs (12 loops, four key sweeps on both
  parts) start at the board's master clock when Nephrite is placed as the board is; it was the drift on channels 4
  to 6 (`Nephrite_Native.md` §24.3, §25.2), and is the six-clock read of the YM2612 left open in §22.5 there.
- 2026-10-06, at stage 5's picture step (D-26): **the refresh runs from power-on**, and its request before the
  68000's start is pending at the 68000's first access, which waits two clocks on the board. Counted with that wait,
  the placements fitted against the 68000's start move 14 master clocks later, the same against the board's power-on:
  the bus refresh's first request after the start at 889 (it was 875), the main RAM's at 1,428 (1,414). **A refresh
  requested while a write waits for a full FIFO** is done in the hold, the bus the refresh's for some 126 master
  clocks from its request: a cartridge read starting before then waits until it ends, two clocks at most.

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
- 2026-10-05, at stage 5's 68000 step: with D-2's two refreshes in, the write path's start and the transfer's first
  read keep their values, 176 and 88; every point of both sweeps and VDPFIFOTesting's 122 still pass, the bus
  refresh placed against the picture as on the board (D-2). Making the VDP's accesses wait for the bus refresh breaks
  `register_writes_show_where_the_board_shows_them` at every pair of the two values tried, and the transfer start
  the write sweep then wants (46) fails VDPFIFOTesting's FIFO Wait States.
- 2026-10-06, at stage 5's picture step (D-26): with the picture placed against the 68000's start as the board's
  is, a write held for a full FIFO ends 24 master clocks after its slot, on the 68000's own clock, its DTACK 4 after
  the slot in 1,243 holds of 1,243; and the control port is taken 14 before the 68000's cycle ends. The write path's
  start and the transfer's first read keep 176 and 88, and every point of both sweeps and VDPFIFOTesting's 122 pass.

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
- Measured 2026-10-05, at stage 5's writer's step (`mdboard.py`'s `status_ports`, the board's
  `ym2612_status_enable` input set by `TB_YMSTATUS`): both timers overflowed, a data write, and the four ports read
  8 clocks on and again long after. Every port reads `$83` while the busy flag stands and `$03` after it, with the
  input at 1 and at 0 alike. The board's FM chip is the YM3438 inside its FC1004 (D-16), so this is the ASIC's half
  of Eke's finding from a second witness. The bench has no discrete YM2612 to show the other half, and what the
  input changes was not found by these reads.
- The flag's length on the board, from a voice's bus log (D-14): it reads set at every poll up to 2,534 cycles after
  the data strobe rises and clear at every poll from 3,066, and either way at 2,800: about 1,400 master clocks, 200 of
  the 68000's, where Nephrite counts 1,344 (Eke's 192) from the write.
- ~~Conclusion, 2026-10-05: **the ASIC's half measured on the board; the discrete chip's still argued** from Eke's
  tests. Nephrite's two models are unchanged.~~
- The flag's length, measured 2026-10-05 at stage 5's 68000 step (a program writing `$2A` and reading the status
  once, 44 to 51 NOPs later, at 14 places in the sample: 112 trials, the strobes from the bus log): the flag reads
  set until the 33rd slot edge after the first edge at or after the data strobe, and clear from it, in all 112. A
  slot is 42 master clocks, six of the chip's input clocks; the edges fall at the slots' starts as channel 1's turn
  on the pins gives them, 22 master clocks into a slot after a sample's deadline in Nephrite's terms. So the flag
  stands 1,386 to 1,428 master clocks, and a flat 1,344 from the write is short by one to two slots. A fit of a
  plain length, or of edges every 6 or 252 master clocks, fails trials; the 42 fits with the edge placed within 7
  master clocks.
- **Which chip that speaks for**: the board's YM3438 of the model 2 ASIC (D-16). Eke found the flag the same length
  on both chips, so Nephrite gives the discrete YM2612 the same rule; that half is argued.
- Conclusion: **the flag at ports 1-3 and its length, measured for the ASIC**; for the discrete chip, the port
  measured by Eke and the length argued from his finding that the two agree.
- Pinned by: `ym2612.rs`'s `the_busy_flag_runs_33_slots_from_the_next_slot_edge_and_reads_at_port_0_on_the_discrete_chip`,
  `sounds.rs`'s `a_program_sees_the_busy_flag_for_192_clocks_at_port_0_only`, and `voices.rs`'s
  `the_whole_machine_plays_every_voice_as_the_board_does` (it fails with 32 slots).
- Implemented in: the commit "Nephrite, stage 5 step 1"; the length in "Nephrite, stage 5: the 68000's side".
- 2026-10-06, at stage 5's SSG-EG and CSM step: the edges, measured again with the chip given the board's writes and
  answering at the board's read strobes (D-22's 103 status runs), fall 32 master clocks into a slot after a sample's
  deadline in D-24's frame; at 29 to 34 one read of the 103 runs differs, at 22 dozens do. The 22 above was fitted on
  the whole machine, whose 68000 timing had placed the reads (D-24). The length is unchanged.

### D-14. The 68000's accesses to the Z80's area take five clocks, and their strobe on the Z80's bus ends four clocks before the 68000's cycle
- Opened: 2026-10-05, at stage 5 step 1, by `mdboard.py sound dac-square`: a 68000 loop writing the DAC through
  `$A04000` runs at 543.6 Hz on Nephrite and ClownMDEmu and at 535 Hz on BlastEm and PicoDrive, about 110 more of the
  68000's clocks for each half period's two writes.
- Documents read: none gives a wait for the 68000's accesses to `$A00000`-`$A0FFFF` beyond the bus request itself.
- Test program: `dac-square` (above).
- Referee: none yet; the board's pins time a 68000 access to that area directly, as D-1 timed the Z80's window.
- ~~Conclusion: open. Nephrite gives the access the 68000's four clocks, as for any other address.~~
- Measured 2026-10-05, at stage 5's bench step (`mdboard.py pins dac-square`, the log read by `pin_records`; no RTL
  file opened): the board's square is 535.04 Hz, a half period of 7,168.2 of the 68000's clocks where the loop's
  instructions add up to the 7,055 Nephrite takes, 112.9 more. D-2's refresh, two clocks lost in every 128 on the
  cartridge's bus, is 110.2 of them; the 2.7 left are 1.3 for each of the half period's two writes. The Z80 bus's
  strobes (`TB_ZBUS`) say the same of single accesses: an address write's and its data write's begin 17 clocks apart,
  and a data write's and a status read's 9, where the instructions give 16 and 8.
- ~~Conclusion: **measured in part.** The slower loop of BlastEm and PicoDrive is D-2's refresh, which this entry had
  taken for the access; an access to the YM2612 costs the 68000 about one clock more than four on the board. Open:
  whether it is always one, or one or two by where the access falls, which a sweep of the bus log would say. Nephrite
  gives the access four clocks and takes no refresh.~~
- Measured 2026-10-05, at stage 5's writer's step, from the bus log of a voice's 37 writes (`mdboard.py board-fm`'s
  `voice.bus`): the busy poll, `tst.b (a2)` and a taken `bmi.s`, 18 clocks by the manual, has its read strobes 266
  cycles apart, 19 clocks, in 301 intervals of 357, and 294, two more, in the other 56, which is D-2's refresh, two
  clocks lost in every 128. So an access costs one clock more, every time.
- What it adds up to: the board's 68000 makes the voice's key-on write 65,702 master clocks after it starts, and
  Nephrite's 62,132, 3.54 samples sooner: the poll's extra clock, the refresh, and a busy flag some 56 master clocks
  longer (D-13), which costs a round of the poll in most writes. Until these are in, a 68000's key-on lands in
  another sample than the board's, and an envelope on the whole machine starts a few samples out
  (`Nephrite_Native.md` §21.4).
- ~~Conclusion: **measured.** An access to the YM2612 through the Z80's area takes the 68000 five clocks; the slower
  loop of BlastEm and PicoDrive is D-2's refresh. Nephrite gives the access four clocks and takes no refresh.~~
- Measured further 2026-10-05, at stage 5's 68000 step: reads and writes of the Z80's RAM through `$A00000` take
  five clocks as the YM2612's do (the next fetch 63 master clocks after the one before, against 56); writes to
  `$A11100` and `$A11200` take four. On the Z80's bus the access's strobe ends 28 master clocks, four clocks, before
  the 68000's cycle does, for a read (its data taken then) and a write alike, and the Z80's reset line moves in a
  write to `$A11200` as early, at the cycle's start. Nephrite hands the YM2612 that time. With these, D-2's refreshes
  and D-13's length, the 68000's reads in a voice's program are the board's to the master clock (3,930 reads, the
  bus refresh at the board run's phase), and every voice's samples are the board's on the whole machine.
- Conclusion: **measured.** Every access to `$A00000`-`$A0FFFF` takes the 68000 five clocks, and reaches the Z80's
  bus four clocks before the cycle ends; the slower loop of BlastEm and PicoDrive is D-2's refresh.
- Pinned by: `voices.rs`'s `the_whole_machine_plays_every_voice_as_the_board_does` (fails with four clocks). The
  strobe's lead is pinned by no test: with the bus refresh against the picture the voices' samples do not show it;
  the bus log does.
- Implemented in: the commit "Nephrite, stage 5: the 68000's side".

### D-15. OPEN. A status read straight after a data write: the board leaves the register at 0, and whether a console does is not known
- *Split 2026-10-06, at stage 5's write pipeline step: this record keeps the writes a read leaves at 0; when each
  register's write is taken, measured here at first, is D-25. Until then it was "When an operator's or a channel's
  register write takes effect, and which writes are lost".*
- Opened: 2026-10-05, at stage 5 step 2, by the board's FM output (`mdboard.py sound fm-sine` and its variants):
  writes made through `$A04000` at the pace of a busy-flag loop were partly lost, the multiple of operator S4 among
  them, which halved its frequency.
- Documents read: SpritesMind topic 386, as prose: Sauraen's reading of the die (2015) that the chip-level registers
  `$21`-`$2F` take a write at once, while a write to an operator's or a channel's register is held in one address
  register and one data register until that operator's turn in the chip's circular register file comes round, a later
  write before then replacing the held one; that the busy flag is a timer started by the data write and may read clear
  for up to 18 of the 68000's clocks after it.
- Test program: `mdboard.py sound fm-sine` and the other FM voices, with and without waits between writes.
- Referee: the board as in D-1, its OPN2 output pins logged by `tb_md.cpp`'s `TB_AUDIO`; no RTL file was opened.
  Writes after the key-on took effect (multiple 2 halved the period as the phase generator's rule says). ~~The board's
  results with waits between all writes were not consistent enough to settle the rule (D-16).~~
- ~~Conclusion: open. Nephrite applies every write at once, which is what a program that waits between writes sees.~~
- Measured 2026-10-05, at stage 5's bench step (`mdboard.py read-after-write`; the Z80 bus as the board carries it,
  its strobes, address and data, printed by `tb_md.cpp`'s `TB_ZBUS`; no RTL file opened): **the lost writes were a
  read's doing, not the register file's.** The first voices polled the busy flag in the instruction after each data
  write. With nothing between the two, the read's strobe falls 39 master clocks after the write's strobe rises (53
  where the board stretches the access), and the outcome follows from one thing, where in a slot of six of the chip's
  clocks the read's strobe ends: at two of the six places the register is left holding 0 in place of the byte
  written, in 18 trials of 18; at the other four the byte is kept, in 31 of 31.
  - It is the same for a chip-level register: the DAC's data was left at 0 at those two places in 16 trials of 16 and
    kept at the others in 33 of 33. The slot-held write of Sauraen's reading is therefore not what is lost.
  - The 0 is not the status byte: with both timers overflowed, the status reading `$03`, TL was left at 0 and not
    at 3, in 14 trials of 14 at the two places, and kept at the others in 35 of 35 (`read-after-write 0 timers`).
  - One NOP between the write and the read (the read's strobe 67 master clocks after the write's) keeps every write,
    49 of 49, and so do two.
  - The busy bit is not yet steady at the early read: of the first fourteen trials it read clear in eight, set in
    four and rose while the strobe was low in two; one NOP later it read set in thirteen and rose during the read in
    one. That is Sauraen's "may read clear" for a time after the write, in kind.
  - So the voices' lost multiple and levels were writes left at 0, a third of them, by where each fell: a multiple of 0
    is a half, which halved S4's frequency, and a TL of 0 is full level. `ym_writes` now leaves four NOPs after each
    data write, and every write of every voice since has taken effect (D-16).
  - ~~A write followed by a taken branch is lost~~ (`Nephrite_Native.md` §19.4, as first written): the branch was
    the busy poll's, and it was the poll's read, coming at once, that mattered.
- ~~Not measured: when a held write lands.~~ Moved to D-25 with the measurements that followed.
- ~~Conclusion: **open, on two counts.**~~ Sauraen's held write is D-25's, and measured there.
- Conclusion: **open.** Whether a console leaves 0 in a register that is read at once after its write is not known:
  no document read here says so, a driver that polled the busy flag in the instruction after a data write would lose
  a third of its writes, and the board is a model of the chip, not the chip. A program for a console can be made from
  `read_after_write`, whose outcome a recording of the DAC's level would show. Nephrite loses no write to a read, and
  will not until a console has been recorded.
- Pinned by: nothing yet.
- Implemented in: not, deliberately (above).

### D-16. The operator's log-sine and exponent tables (settled), and the bench's FM output (measured: it plays every write and referees the FM unit)
- Opened: 2026-10-05, at stage 5 step 2, by the operator unit's two tables, whose form the documents give (a
  quarter-wave of 256 log-sine entries in 4.8 fixed point; 256 entries of 2^−x as eleven bits whose top one is set;
  Nemesis's and Sauraen's prose, topic 386) but not the two offsets that fix every entry's last bit: where in each
  entry the sine is taken, and whether the exponent's index starts at 0 or 1.
- Documents read: topic 386's prose on the operator unit (Nemesis, 2008; Sauraen, 2015-16). The offsets Nephrite
  uses (each sine at its entry's middle; the exponent at index plus one) were in code blocks of Nemesis's post that
  were displayed before they were identified as his emulator's code (blocks 24-27 of page 11, one of them a method of
  his emulator's YM2612 class); they are the one item of the step not drawn from a clean source, and are to be
  replaced by whatever a measurement shows.
- Test program: `mdboard.py sound op-alone-40` and `op-alone-c0` (the test registers `$21` and `$2C` with the status
  read as the 68000's samples): the reads give a channel's nine-bit output over four slots, not an operator's
  fourteen-bit one, so the tables were not read. The FM voices on the board: see below.
- Referee: the board as in D-1. A single operator plays a clean sine on the board when its setup writes come
  without waits, some of them lost; ~~with every write applied, through the 68000 with any wait or through a Z80
  program, the board's channel stays at rest, though a Z80 program's DAC writes show. The bench's YM2612 is not yet
  understood well enough to referee the operator unit;~~ the references' audio graded this step instead
  (`Nephrite_Native.md` §19.3). At the nine-bit output the two tables' offsets change nothing measurable in that audio.
- ~~Conclusion: open.~~ Superseded for the tables by the next entry, and for the bench by the one after it.
- The tables, 2026-10-05: written again, independently, by someone who had seen none of the excluded material,
  from "OPLx decapsulated" (Matthew Gambrell and Olli Niemitalo, 2008), whose Tables I and II print every entry of the
  OPL2's log-sine and exponent ROMs (`fm_tables.rs`, its provenance in `Nephrite_OperatorTables.md`). Their log-sine
  table is Nephrite's to the bit; their exponent table, the ROM's ascending order with its leading one implied, is
  Nephrite's read backwards with 1,024 added. `fm.rs` now reads both from `fm_tables.rs` (2^−x for a fraction f being
  `EXP[255 − f] + 1024`) and its own copies are gone; every crate test and every FM trace is unchanged. The excluded
  blocks are the source of nothing; the record that they were displayed stands.
- Open, for the bench if it can answer: no clean source yet says that the OPN2's ROMs are the OPL2's.
- ~~Conclusion: **the tables: documented** (the OPL2's ROMs, printed); **the bench: open**.~~
- The bench, 2026-10-05, at stage 5's bench step. No RTL file was opened. Two files Verilator generates from the
  board were searched for names and nothing else: `Vmd_board.h`, for the top level's ports, and
  `Vmd_board___024root.h`, for the board's own nets, of which `ZA`, `ZD`, `ZRD` and `ZWR` (the Z80 bus between the
  chips) are now printed by `TB_ZBUS`. The names of the generated files were listed with them; one is named for a
  YM3438's channel register, and the top level offers the nine-bit `MOL`/`MOR` beside the ten-bit
  `MOL_2612`/`MOR_2612` this step logs, so the board's FM unit may be the YM3438 with the YM2612's output as a
  choice. Which it is matters to the ladder effect and to D-13, and is for those steps to find out.
  - **Why it stayed at rest.** It did not. The earlier runs were read at the wrong place in the sample: the turn of
    channel 1 on the pins moves with the Z80's reset line (D-19), 1,223 cycles into the sample in a program that only
    releases the line and 915 in one that pulses it to load a Z80 program, and the first comparisons looked at 1,223
    in both. The voices that did go wrong had writes left at 0 by a status read (D-15).
  - **Reading the pins.** Times are MCLK2 cycles, two to a master clock: a sample is 2,016 of them and a slot 84.
    ~~A channel's value is on `MOL_2612` for four slots of every sample, its level plus one when that is not negative
    and its ten-bit two's complement when it is; at rest the pin reads 1.~~ *Corrected at the writer's step, with
    `MOL` logged beside it (`TB_MOL`):* a channel's turn is four slots; `MOL_2612` carries its level for the first of
    them, one higher when it is not negative and as ten-bit two's complement when it is, and for the other three
    stands at 1 if the level was not negative and at 1,023 (−1) if it was; `MOL` carries the level plus 256 for
    those three and 256 outside a turn. The two give the same nine-bit level in every sample of `fm-sine` (1,223
    compared) and of the DAC's ramp (1,235), and every comparison reads the first slot. Channel 1's turn begins 1,226 cycles after the
    reset line's assertion, in every sample; channel 6's, where the DAC's register shows as twice its signed value,
    1,344 after channel 1's, the fifth turn of six, as the order 1, 5, 3, 2, 6, 4 of the thread's page 54 has it.
    `mdboard.py pins` logs a program's pins and `mdboard.py fm` compares the busiest channel with `fmtrace`'s output,
    each from its first sample that is neither 0 nor −1, with no other alignment.
  - **What was compared**: 62 voices, 200,220 samples. The thirteen of the algorithms, levels, multiples, detune and
    feedback (`fm-sine`, `fm-tl16`, `fm-mul3`, `fm-dt7`, `fm-feedback5`, `fm-chain`, `fm-alg1` to `fm-alg7`), 5,191
    samples each; three envelopes (`fm-attack`, `fm-envelope` to its key-off, `fm-envelope-ks`); twenty decays and
    sixteen attacks by rate (`fm-dr38` to `fm-dr61`, `fm-ar44` to `fm-ar59`); and ten voices of a decaying S3 into S4
    (`fm-rom0` to `fm-rom9`).

    | The FM unit | Samples equal to the board's, of 200,220 |
    |---|---|
    | as committed | 129,855 |
    | with operator 1's output used a sample late (D-17) | 154,137 |
    | and the envelope's step taken after the sample's output (D-17) | 190,269 |
    | and rates 48 to 59 doubling on the board's cycles (D-18) | 200,220 |

    The last three rows are builds made outside the tree; the committed unit is the first row and is not changed by
    this step. As committed, the five voices with one sounding operator and no moving envelope are already exact
    (`fm-sine`, `fm-tl16`, `fm-mul3`, `fm-dt7`, `fm-feedback5`: 25,954 samples of 25,954).
  - **The two ROMs.** With all three rules in, each of the 512 entries was moved up one and down one, one at a time,
    in a build made outside the tree for the purpose (the tables in the tree were not touched, and no entry was
    changed to fit anything). Nine of the 1,024 changes would take a zero entry below zero, which a ROM cannot hold;
    every one of the other 1,015 alters at least one of the 200,220 samples. The first 52
    voices alone leave 5 log-sine entries unpinned in each direction and 135 and 106 of the exponent's; the ten
    `fm-rom` voices, in which every bit of S3's output moves S4's phase, pin the rest. So the board's FM unit
    computes with a log-sine and an exponent ROM equal, entry for entry, to the OPL2's as "OPLx decapsulated" prints
    them and `fm_tables.rs` holds them. This is the measurement the tables' entry left open, and it changes no entry.
  - **What that is worth.** It is agreement between the printed OPL2 tables and a model of the Genesis's chip, taken
    through the model's pins. It is evidence about the console only so far as the model's ROM contents were read from
    the OPN2's die, which its pins cannot say and which nothing read here says. A change of one in a single entry is
    excluded; a set of changes that cancel in every one of the compared samples is not.
  - **Not done.** The test register's fourteen-bit read of one operator: the reads still give a channel's nine-bit
    output. The pins give nothing finer than nine bits a channel, so the operator's own output is seen only through
    what it does to another's phase.
- Which chip, 2026-10-05, at stage 5's writer's step. By its pins: the busy flag reads at every port (D-13), which
  Eke found of the ASIC and not of the discrete YM2612, and the board gives its FM output in two forms (above). By
  its files, opened for this and nothing else, as a search for seven names (`ym2612_status_enable`, `MOL_2612`,
  `MOR_2612`, `ym3438`, `ym2612`, and the instance lines): `md_board.v` lines 101-102 (the two outputs' port
  declarations, commented "ym3438 linear, unsigned" and "ym2612 dac emulation, signed"), 131, 309 (`fc1004 ym`),
  351-352, 495, 532, 591, 637, 660 (the other instances' names) and 889-890 (the board's mix of the ten-bit output
  with the PSG, printed by the search, noted as seen and used for nothing); `fc1004.v` lines 68, 211, 387
  (`ym3438 fm`), 405 and 409. **The bench's FM chip is the YM3438 inside the FC1004, the model 2 ASIC**; its
  `MOL_2612` is that model's rendering of the YM2612's DAC, not a second chip.
  - What each measurement therefore speaks for. The logic (the operators and their order, the envelope, the two
    ROMs, the reset line, the busy flag and the status ports, the read after a write): the YM3438 as the ASIC has
    it. For model 1's discrete YM2612 these stand as far as the two chips share their logic, which is argued, not
    measured here; D-13 is a known difference. The output: `MOL` is the YM3438's; `MOL_2612` is an emulation of
    the YM2612's DAC inside the model, so what it shows of the ladder effect is a model's account of the discrete
    chip, to be weighed beside the documents and not as a transcription of its die.
- Conclusion: **the tables: documented** (the OPL2's ROMs, printed) **and, on the board, measured**; **the bench:
  plays every write and referees the FM unit to the sample, and is the YM3438 of the model 2 ASIC.** What it found
  against the unit as step 2 left it is D-17 and D-18.
- Pinned by: `fm_tables.rs`'s tests (the formulas, spot checks against the printed tables, the exponent's identity);
  from the writer's step, `voices.rs`'s six tests against `board_fm.rs`, which hold the chip to the board's 200,220
  samples given the board's own write times.
- Implemented in: the commit "Merge nephrite-fmtables"; the bench's tools in the commit "Nephrite, stage 5: the bench
  step".

### D-17. The FM unit's order within a sample: operator 1's output reaches the others and the sum a sample late, and an envelope step shows from the next sample
- Opened: 2026-10-05, at stage 5's bench step, by the board's pins once every write was applied (D-16): the voices
  with one sounding operator agreed with Nephrite's to the sample, those in which operator 1 modulates another or
  sounds beside the others did not (`fm-chain` 3,328 samples of 5,191, `fm-alg7` 557), and no voice with a moving
  envelope did (`fm-envelope` 297 of 2,900).
- Documents read: Sauraen's account of the die (topic 386, 2015-16) as step 2 took it (`Nephrite_Native.md` §19.1):
  operators evaluated in the order 1, 3, 2, 4, operator 3 taking the stored outputs of 1 and 2, operators 2 and 4
  operator 1's newest. In this step, the thread's page 54, a table of the chip's 24 slots (the channels in the order
  1, 5, 3, 2, 6, 4, the operators by sixes; "channel output always store value from 24 cycles back"), read as a
  table; three lines of that post, which carry code, were not displayed.
- Test program: the voices of D-16, by name in `mdboard.py`'s `SOUNDS`.
- Referee: the board as in D-16; no RTL file opened. Each rule below was tried as a build of `fm.rs` outside the tree,
  traced by `examples/fmtrace.rs` and compared with the pins.
  - Operator 1 a sample late: the thirteen algorithm and level voices go from 43,200 samples of 67,482 to 67,482 of
    67,482, every algorithm among them. Using the stored output in operator 2 alone, or the newest in operator 3,
    fits some algorithms and not others.
  - The envelope's step after the output: `fm-attack` goes from 5,759 of 5,800 to 5,800, `fm-envelope` from 297 of
    2,900 to 2,900, and the decays D-18 leaves alone and the ten `fm-rom` voices to every sample. Keeping the step
    before the output and moving the envelope's divider by one sample instead is not the same thing, and does not
    fit: it agrees on the slow voices and fails the fast attacks (`fm-ar47` 35 samples of 3,000). The two
    arrangements differ only in what becomes of a key taken on the sample the step would move to.
- Conclusion: **measured on the board.**
  1. Operator 1's output is used a sample after it is made. Operator 2, operator 4 and, in algorithm 7, the
     channel's sum take the output operator 1 made in the previous sample; operator 3 takes the one before that.
     Operator 1's own feedback, from its last two outputs, is as argued, and so are operator 2's stored output into
     operators 3 and 4 and operator 3's newest into operator 4.
  2. On a sample in which the envelope's cycle comes, every operator sounds at the attenuation it had, and the step
     shows from the next sample. A key taken at that sample's start is stepped at its end.
  ~~The committed `fm.rs` has step 2's argued order; the writer's step that follows implements these and cites this
  entry.~~
- Pinned by: `voices.rs`'s `the_algorithms_and_levels_are_the_boards_sample_for_sample` (the first rule: with
  operator 2 or the sum taking operator 1's newest, `fm-chain` or `fm-alg7` has no block of the board's) and
  `the_envelopes_are_the_boards_sample_for_sample`, `the_decays_by_rate…`, `the_attacks_by_rate…` and
  `a_modulator_into_a_carrier…` (the second: with the step before the output all four fail).
- Implemented in: the commit "Nephrite, stage 5: the board's FM rules".
- 2026-10-06, at stage 5's SSG-EG and CSM step: the first rule is restated by D-24, S1 keyed a sample after S2-S4
  and its output used at once; the voices above are the board's under either.

### D-18. The envelope's rates 48 to 59: the double step falls on cycle 0 of each four, on 0 and 2, or on 0, 1 and 2
- Opened: 2026-10-05, at stage 5's bench step, by the decays by rate: with D-17's order in, every rate below 48 and
  rates 48, 52, 56, 60 and 61 agreed with the board to the sample, and rates 49-51, 53-55 and 57-59 did not
  (`fm-dr49` 1,829 samples of 1,919; `fm-ar51` 23 of 3,000).
- Documents read: Nemesis's table of increments (topic 386, 2008) as step 2 read it into `fm.rs`: at rates 48 to 59
  a step of 1, 2 or 4, doubled on some of the counter's eight cycles, which for a rate's low two bits of 1, 2 and 3
  it put on cycle 3 of each four, on 1 and 3, and on 1, 2 and 3. A search of the saved thread for the table's rows,
  by their digits alone, found none, and the post was not otherwise read again in this step.
- Test program: `fm-dr38` to `fm-dr61` and `fm-ar44` to `fm-ar59` (`rate_voice`: S4 alone with key scaling 3, where
  the rate is twice the register plus a key code of 18 or 19), and `fm-envelope-ks`, whose attack is rate 54.
- Referee: the board as in D-16; no RTL file opened. Turning the eight cycles round by one amount does not mend it:
  the rates of low bits 1 want a turn of 3 or 7, those of 2 any odd turn, those of 3 a turn of 1 or 5, while rate 47,
  which misses one step in eight, fixes the turn at 0. With the double steps on cycle 0 of each four, on 0 and 2,
  and on 0, 1 and 2, all twenty decays (31,038 samples) and all sixteen attacks (48,000) agree to the sample, and
  `fm-envelope-ks` goes from 2,748 of 3,000 to 3,000.
- Conclusion: **measured on the board.** Numbering the envelope counter's cycles so that rate 47 takes no step on
  cycle 0 of eight, as rates 44 to 47 are tabled and as the board has them, a rate of 48 to 59 whose low two bits are
  1 doubles its step on cycle 0 of each four, one whose low bits are 2 on cycles 0 and 2, one whose low bits are 3 on
  cycles 0, 1 and 2. These are step 2's three rows read backwards. ~~Whether the table or step 2's reading of its
  columns is at fault is not established; the board's cycles stand either way.~~ Rates below 48 are as step 2 has them
  (38, 39 and 42 to 47 compared).
- The table and the reading, 2026-10-05, at the writer's step: the table was found in the saved thread (page 8,
  "Table 2: Attenuation increment values") and its rows read with every identifier in the block masked. Its row for
  rates 48 to 51 is `1,1,1,1,1,1,1,1  1,1,1,2,1,1,1,2  1,2,1,2,1,2,1,2  1,2,2,2,1,2,2,2`, and those for 52 to 59
  the same doubled and doubled again: step 2 read it rightly, and it is the table that differs from the board. Its
  rows for rates below 48 run in the same column order and are the board's, so the order of its columns is not in
  doubt. The table and the board give every rate the same steps in each four cycles; they differ in which cycles
  take them, which a measurement of an envelope's length does not see and the board's samples do. The board's are
  implemented.
- Pinned by: `voices.rs`'s `the_decays_by_rate…`, `the_attacks_by_rate…` and `the_envelopes…` (with step 2's rows
  `fm-dr49` has 895 samples of 1,919), and `fm.rs`'s increments by cycle for rates 47, 49, 50 and 51.
- Implemented in: the commit "Nephrite, stage 5: the board's FM rules".

### D-19. The Z80's reset line resets the YM2612: its voices stop, its sample cycle restarts from the line's assertion, and its envelope counter from the release
- Opened: 2026-10-05, at stage 5's bench step, by channel 1's turn on the pins, which came 1,223 cycles into the
  sample in a program that only releases the Z80's reset and 915 in one that pulses it (D-16).
- Documents read: Charles MacDonald, `gen-hw.txt`, on `$A11200`: "The /RESET line is shared with the YM2612. For as
  long as the Z80 is reset, the YM2612 cannot be used."
- Test program: `mdboard.py reset-replay`: a Z80 program plays S4's attack at AR 12, and the 68000 pulses the reset
  twice, so that the Z80 writes the same voice again, at the same cycles after each release to within 2.
- Referee: the board as in D-16, its reset line and pins logged (`TB_PINS`, `TB_AUDIO`); no RTL file opened.
  - A sounding voice stops at the pulse and the pins rest until the Z80 has written it again.
  - Channel 1's turn comes 1,226 cycles after the line's assertion, to the cycle, after every pulse whose voice
    sounded: eight in three runs, held 0.28, 1.28, 14.4 and 14.6 samples. After the release it comes wherever that
    leaves it (666, 414, 1,982 and 2,010 cycles on). From power-on, where the line starts asserted, it is 1,223.
  - After a pulse of 1.28 or 14.4 samples the replayed attack leaves rest 868 samples after the release, each time:
    the same count after pulses of different lengths, and after two releases 1,464.8 samples apart, where a counter
    left running would put the envelope's steps 25 samples elsewhere. So the envelope's counter runs from the release.
  - After the 0.28-sample pulse a Z80 loader makes, the attack leaves rest at 855, not 868: that pulse restarted the
    cycle and seemingly not the counter. One case, the same in every run, and not resolved.
  - After one pulse of 1.26 samples the voice did not sound again, though the Z80 made the same 72 writes; after one
    of 1.28 it did. A reset shorter than about a sample and a third leaves the board's chip in no dependable state.
- ~~Conclusion: **measured on the board**, and documented in its first part. Asserting the Z80's reset resets the
  YM2612: its voices stop at once, its 24-slot cycle restarts from the assertion, and its envelope counter is held
  until the release. Whether every register is cleared, or the keys alone, was not separated: the replay writes them
  all. Nephrite does not reset the YM2612 with the line at all; its envelope voices agree with the board's because a
  program that only releases the line does so 0.79 of a sample after the 68000 starts (1,596 cycles on the board),
  inside Nephrite's first. The writer's step that follows implements the reset and cites this entry; how short a
  pulse may be is left at the board's 1.28 samples.~~
- The writer's step, 2026-10-05 (`mdboard.py`'s `reset_sweep`: a decay at rate 39 from an instant attack, replayed
  after 36 pulses held 1.83 to 2.82 samples in steps of 28 master clocks, with the four pulses of 0.28 a Z80 loader
  makes; the Z80's writes taken from the bus log, 72 after each release, the key-on's strobe 295,683 master clocks
  after it to within 9). The chip was given each replay's writes at the board's times and its samples compared with
  the board's 512, from the first off rest. Counting master clocks from the line's assertion, a sample every 1,008:
  - **A key-on is in time for a sample if its strobe rises no later than 87 into it** (every replay agrees for 76 to
    100 and not for 72 or 104). Seen directly on the pins: a key written 73 into the sample is first heard in
    channel 1's turn three samples on, one written 103 into it in the turn after that.
  - **The envelope's counter starts with the sample the release comes before, and a release up to 67 after that
    moment still counts as before it** (56 to 80; so the release has until 154 into the sample where a key has until
    87).
  - **The counter's first cycle is the fourth sample from its start**: neither the third nor the fifth agrees with
    any replay, nor with any of the 49 voices in which an envelope moves.
  - The pins show a sample 2,542 master clocks after the moment its key had to be in by: two samples and half
    another.
  - With these three, all 40 replays are the board's, 20,480 samples, the four of the loader's 0.28-sample pulse
    among them: ~~the short pulse's one odd case~~ was the counter's start, which the attack of the first test
    could not place.
  - Short pulses, noted and not pursued: of 30 replays in three further sweeps, after pulses of 0.19 to 1.86
    samples, 22 are the chip's as modelled and 8 are not, keeping something of the voice that was sounding; with the one that left the
    chip mute (above), a reset held less than about 1.8 samples does not always take on the board. Every pulse of
    1.83 samples or more did, 36 of 36.
  - The power-on voices under the same rule. A program that only releases the line does so 798 master clocks after
    the 68000 starts; the board's cycle, begun when the line was asserted at power-on, puts that release 523 into a
    sample, the counter's start at the next and its first cycle three on. Step 2's unit, whose counter cycled on
    its first sample, agreed with the board's envelopes only because Nephrite's 68000 makes the key-on 3.54 samples
    sooner than the board's (D-14): one cycle of three samples out on each side. With the chip given the board's
    write times all 62 voices are the board's under the rule and 13 without it.
- Conclusion: **measured on the board**, and documented in its first part. Asserting the Z80's reset resets the
  YM2612: its registers, keys, timers and operators go to their power-on state and its sample cycle restarts from
  the assertion; the release starts the envelope's counter, as above. Nephrite resets the chip whatever the pulse's
  length, and at power-on, where the line starts asserted, keeps its sample cycle as it starts. Whether every
  register is cleared or the keys alone was not separated on the board: the replays write them all.
- Pinned by: `voices.rs`'s `a_voice_replayed_after_a_reset_is_the_boards_wherever_the_release_falls` (the 40
  replays; it fails for a restart outside 76 to 100, a leeway outside 56 to 80, or a first cycle other than the
  fourth sample) and `the_z80s_reset_line_resets_the_ym2612` (the line's wiring through the 68000's `$A11200`).
- Implemented in: the commit "Nephrite, stage 5: the board's FM rules".
- 2026-10-05, at stage 5's 68000 step: the power-on cycle, where the line starts asserted, now has its first
  deadline 793 master clocks after power-on, where the board's falls against the picture (D-2's placement); it was
  458, the board's against the 68000's start. Both give every voice the board's samples on the whole machine.
- 2026-10-06, at stage 5's SSG-EG and CSM step: in D-24's frame the 87 and the 793 above are key moments, and a
  sample's deadline comes 472 after its key moment: `RESTART` is 559 and `POWER_ON` 1,265, the same placements. The
  board's first deadline after power-on, in its own clock, is 537 (D-24).
- 2026-10-06, at stage 5's placement step: `POWER_ON` is 910, the board's first deadline against its 68000's start
  (the key and CSM sweeps of all six channels each in the board's sample on the whole machine there, 1,000 runs),
  as decided in `Nephrite_Native.md` §22.3.
- 2026-10-06, at stage 5's picture step: `POWER_ON` is 924 from the 68000's start, the same deadline against the
  board's power-on, now that the 68000's first access waits the pending bus refresh (D-26).

### D-20. The LFO: a 128-count cycle whose divider never stops, a triangle of tremolo, and each operator taking the count at its own place in the sample
- Opened: 2026-10-06, at stage 5's LFO step, by the documents giving the LFO's speeds in Hz and its depths in cents
  and dB and nothing of its counting or of when an operator reads it.
- Documents read: the YM2608 manual §2-6 (`$22`: bit 3 the switch, bits 2-0 the speed, 3.98 to 72.2 Hz; PMS 0-7 as 0
  to 80 cents; AMS 0-3 as 0, 1.4, 5.9 and 11.8 dB; AMON, bit 7 of `$60`-`$6E`, per operator). `Nephrite_LfoTables.md`
  for the vibrato's offsets and its §4 for the counter's steps as measured there (108, 77, 71, 67, 62, 44, 8 and 5
  samples a count; the first step short after the LFO is turned on).
- Test program: `mdboard.py`'s LFO voices (`fm-lfo0` to `fm-lfo7`, `fm-ams1`, `fm-ams2`, `fm-am-off`, `fm-lfo-off`,
  `fm-lfo-restart`, `fm-am-mod`, `fm-pms1` to `fm-pms7`, `fm-pm-kc`, `fm-pm-c1s1` to `fm-pm-c6s4`, `fm-am-s1` to
  `fm-am-s3`, `fm-am-c2s1` to `fm-am-c6s4`, `fm-pm-1500`, `fm-pm-ch3-normal`, `fm-pm-ch3-p0`, `fm-pm-ch3`,
  `fm-pm-ch3-swap`): 61 voices, 8,600 samples each, 17,000 for the slow speeds.
- Referee: the board as in D-16 (the model 2 ASIC's YM3438), each voice's pins against the whole machine's trace
  from the first sample off rest. No RTL file was opened. Each rule was tried against its alternatives:
  - **The counter.** Seven bits, stepping every 108, 77, 71, 67, 62, 44, 8 or 5 samples by `$22`'s speed, before the
    sample's operators. Off, the count is held at 0; the divider that times its steps runs on regardless, which is
    why the first step after the LFO is turned on again is short (`fm-lfo-restart`: off, on at another speed, and
    the speed changed again, 17,096 samples of 17,096; with the divider cleared while off, 16,852; with it stopped,
    16,382).
  - **The tremolo.** The count as a triangle, 126 at count 0 down by 2 a count to 0 at 63, then 0 at 64 up to 126
    at 127; shifted right by 3, 1 and 0 for AMS 1, 2 and 3 (at most 15, 63 and 126 of the attenuation's steps of
    0.09375 dB: 1.4, 5.9 and 11.8 dB, the manual's); added to the operator's attenuation with TL when its AMON bit
    is set, on a modulator as on a carrier. A triangle that rises first, or 126 − 2c without the repeated 0, fails
    from the first samples.
  - **The key code under vibrato** is the register's: with detune 3 just under the edge of key codes 18 and 19 and
    PMS 7, taking the key code from the modulated frequency gives 2,138 samples of 8,663, the register's all.
  - **Each operator's place.** S1 of channels 1 to 5 takes the count the next sample will have; S1 of channel 6 and
    S2-S4 of channels 1 to 5 this sample's; S2-S4 of channel 6 the one before. It is the same for the vibrato (all
    24 operators measured alone) and the tremolo (ten measured: S1-S4 of channels 1 and 6, S1 of 2 and 4); giving
    any of them a neighbouring count fails within 80 samples.
  - **The vibrato's table** (`lfo_tables.rs`) agrees with the board in every voice here: PMS 1 to 7 at frequency
    1,081 and PMS 7 at 1,151 and 1,500, 8,663 samples each.
- Not settled: **channel 3's special mode under the vibrato.** Without vibrato the special mode is the board's
  (8,656 of 8,656). With it, S1 at its own frequency keeps the board's samples to the 65th and 76% of them after
  (the two frequencies swapped: to the 25th, 83%); taking the offset from the channel's frequency, or none, fails
  within six. The board's phase then runs ahead by about one of its 1,024 steps in thirty samples, and no one
  frequency for each vibrato step explains its samples: the frequency moves within a step, which no normal channel's
  does. Nephrite gives S1-S3 the vibrato of their own frequencies.
- Recorded, not resolved: the manual's speeds are those of a counter one sample longer a step at the YM2608's rate
  (`Nephrite_LfoTables.md` §4); the board's figures are implemented.
- Conclusion: **measured on the board**, the special mode under the vibrato open.
- Pinned by: `voices.rs`'s `the_lfo_voices_are_the_boards_sample_for_sample` (59 voices exact, 574,600 samples; it
  fails with every operator given this sample's count, the triangle rising first, AMS 1 shifted by 2, the divider
  cleared while off, or the key code from the modulated frequency; and it holds the two special-mode voices to
  their first agreement and no further, so that a rule that mends them is noticed).
- Implemented in: the commit "Nephrite, stage 5: the LFO".
- 2026-10-06, at stage 5's SSG-EG and CSM step: ~~each operator's place~~ and ~~the special mode under the vibrato
  open~~ are replaced by D-24. With the chip given the board's writes, every operator takes the count before its
  sample's step, S1 is keyed a sample after the others, and the special mode's two voices are the board's (60 of the
  61 voices; `fm-lfo-restart` differs where its `$22` writes land, D-15). The per-operator counts above fitted the
  whole machine, whose 68000 timing and placement against the picture put the voices a sample from the board's
  against the LFO's divider.


### D-21. SSG-EG: a pass ends when the attenuation reaches `$200`, the output inverts about `$200`, the decay runs four times as fast, and a key-off keeps the level the output had
- Opened: 2026-10-06, at stage 5's SSG-EG step, by the YM2608 manual's eight shapes for `$90`-`$9E`, which give the
  envelope's form and nothing of the counting behind it.
- Documents read: the YM2608 manual §2-6 (the register: bit 3 the switch, bits 2-0 the shape; the eight shapes drawn;
  AR to be `$1F`). SpritesMind topic 386 as prose: Nemesis's SSG-EG section (2008; the bits as enable, attack,
  alternate and hold; the decay and sustain repeated; an inverted pass; the hold; a slow attack inside each pass) and
  his 2010 corrections (the decay's step four times the normal one, not six, in the decay, sustain and release; the
  pass ending at an attenuation of `$200`, not `$3FF`; the inversion "centred at `$200`", a two's complement and not a
  complement of bits; the SSG-EG steps taken each output sample, before the envelope's cycle). The posts' code blocks
  were withheld when the thread was read for this step; none was displayed. Of the page 28 code, two lines were
  displayed at stage 5 step 2 (`Nephrite_Native.md` §19.2); nothing here was taken from them.
- Test program: `mdboard.py`'s SSG-EG voices (62): S4 alone with each of the eight shapes (`fm-ssg8` to `fm-ssgf`),
  each released in its second pass and keyed again, each with an attack of rate 38 in every pass, a decay to SL 4 with
  a slower sustain, the fastest decay, a TL, the tremolo, a modulator (`fm-ssg-mod8`, `fm-ssg-mode`), the operators
  of channels 1 and 6 alone, a key-off inside a slow attack (`fm-ssg-ar-rel*`), and `$9C` written as the envelope
  runs (`fm-ssg-on`, `-off`, `-flip`, `-hold-off`, `-hold-att`, `-a-to-b`, `-b-to-a`).
- Referee: the board as in D-16, 19,000,000 cycles a voice; Nephrite's chip given each write at the master clock the
  board's 68000 made it (`examples/chip.rs`, the bus logged by `TB_ZBUS`), every sample compared absolutely; no RTL
  file opened.
- Conclusion: **measured on the board.** With bit 3 set:
  - In the decay, the sustain and the release each step adds four times its increment, and none once the
    attenuation is `$200` or more.
  - Each sample, after the output and the phase's advance, an attenuation of `$200` or more ends the pass: in the
    release it goes to `$3FF`; with hold, the alternate bit sets the inversion once and, outside the attack, an output
    not then inverted is held at `$3FF`; without hold, the alternate bit toggles the inversion or, clear, the phase
    restarts, and the envelope starts its attack again (the attenuation to 0 at once for a rate of 62 or 63).
  - The output is inverted where the attack bit and the toggled inversion differ, outside the release: `$200` less
    the attenuation, kept to ten bits, before TL and the tremolo are added.
  - A key-on clears the toggled inversion.
  - A key-off takes, as the release's attenuation, the level the envelope last put out: in an inverted pass the
    inverted level, and in any pass a step made in the cycle just before it set aside. This holds without SSG-EG
    too (`fm-csm-key`, D-22), and moves none of the earlier voices.
  - Each alternative was tried on the chip against the voices: a pass ending at `$3FF` fails 61 of the 62, the
    inversion as a complement of bits 40, the step six times all 62; the hold setting the inversion only outside the
    attack fails the slow attacks with hold, and the key-off converting the attenuation of its moment the voices
    keyed off just after a step (`fm-ssg-ar-relc-3`, `fm-ssg-ar-rele-3`, `fm-csm-key`).
- All 62 voices are the board's to the sample on the chip.
- Pinned by: `voices.rs`'s `the_chip_voices_are_the_boards_sample_for_sample`.
- Implemented in: the commit "Nephrite, stage 5: SSG-EG, CSM and the test register".

### D-22. CSM and the timers: timer A's overflow, and its load, key channel 3's operators for one sample in mode `10`; a load waits for the next tick, a count is taken a slot later, a flag reads set a fixed time before its sample
- Opened: 2026-10-06, at stage 5's CSM step, by the documents giving CSM's key in words and the timers' counting in
  samples, with nothing of where in a sample a load, a count or a flag falls.
- Documents read: the YM2608 manual §2-6 (`$27`'s mode bits; CSM speech synthesis keyed by timer A). SpritesMind topic
  386 as prose: Nemesis's CSM section (2008; mode `10` alone, not `11`; timer A to be loaded; the key-on and key-off
  at the overflow; only AR `$1E`-`$1F` sounding; a manual key masking the CSM key) and his 2010 corrections (the CSM
  key a flag set by the overflow and taken at the next update, OR'd with the manual key; held on by an overflow every
  sample). Sauraen's list of the test register's bits (2016), as prose: `$21` bit 2 counting the timers every
  internal clock. Plutiedev's register page for the timers' values.
- Test program: `mdboard.py`'s CSM voices (17: by period 1, 2, 3 and 100, every operator alone at its own frequency,
  all four, a slow attack, modes `11` and `10` without the load, the flags' enable bits, CSM turned off and on, a
  manual key held through it and its plain twin), the timer test bit's voices (`fm-test-timer`, at periods 7 to 200),
  `status_loop` (the status read every 23 of the 68000's clocks after a load: timers A and B at several periods, with
  and without the test bit, 103 runs with the loads moved across the sample by NOPs), the CSM sweep (the load moved by
  NOPs, 40 runs), and `sounds.rs`'s program polling timer A's flag, run on the board.
- Referee: the board as in D-21; the chip given the board's writes, and reads answered at the board's read strobes.
- Conclusion: **measured on the board.** In the frame of D-24, where a sample's deadline is the last moment a write
  is in time for it:
  - A timer's tick comes `LOAD_LEAD` (388 master clocks) before a sample's deadline, timer B's every sixteenth. A load
    bit rising before the tick is taken by it; the counter takes its value a slot later and counts from the next
    tick. An overflow raises the flag, which reads set from 276 master clocks before the deadline for timer A and 234
    for timer B, and the value comes back a slot later.
  - In mode `10`, the load and every overflow at a sample's tick key channel 3's four operators for that sample, the
    key OR'd with `$28`'s: a held manual key masks the CSM key-off, and CSM's key-on is lost on an operator already
    keyed. With an overflow every sample the key stays on, and a slow attack runs (`fm-csm-ar-p1`).
  - `$21` bit 2 makes every slot of the 24 a tick for both timers, timer B without its divider: a period of `n`
    counts takes `n` + 1 slots, and the CSM key comes only where an overflow falls on the sample's own tick.
  - `$27` reaches the timers at once, not with the key's lead.
- All 17 CSM voices and 9 of the timer test bit are the board's to the sample on the chip; the CSM sweep's 40 runs
  first sound in the board's sample; of the 103 status runs the timers' flags read as the board's at every read and
  the busy flag at all but one read (`sls-a30t-12`, at its last slot's edge). The polling program reads timer A's flag
  77 times on the board and on Nephrite placed as the board is (D-24); 79 at Nephrite's own placement.
- Pinned by: `voices.rs`'s `the_chip_voices_are_the_boards_sample_for_sample` and
  `keys_and_the_timers_load_land_in_the_boards_samples`; `ym2612.rs`'s
  `the_timers_overflow_at_their_periods_and_their_flags_clear_by_reset` and
  `the_test_registers_bit_2_counts_the_timers_by_slots`; `sounds.rs`'s `a_program_sees_timer_a_overflow_after_its_period`.
- Implemented in: the commit "Nephrite, stage 5: SSG-EG, CSM and the test register".

### D-23. The test register and the channel's sum: `$21`'s bits 1 to 5 as measured, `$2C`'s bit 5 putting the DAC on every channel, the DAC's data at `$80` from reset, and the carriers summed S1, S3, S2, S4, held to nine bits at each step
- Opened: 2026-10-06, at stage 5's test register step, by Sauraen's list of the test bits, which says what each bit
  reaches on the die and, for several, that it was not tested.
- Documents read: SpritesMind topic 386, Sauraen's test register posts (2015-16) as prose: `$21` bit 1 "some LFO
  control", bit 2 the timers per clock, bit 3 freezing the phase generator, bit 4 inverting the operators' top bit,
  bit 5 freezing the envelope generator, bits 6, 7 and 0 the test read; `$2C` bit 3 the DAC's ninth bit, bit 4 the
  test read's form, bit 5 the DAC over all channels, bits 6 and 7 the TEST pin. Code blocks withheld.
- Test program: `mdboard.py`'s test voices (29): each bit set before a key-on and as a voice sounds, the DAC over the
  channels with `$2B` on and off, the read bits; three voices whose carriers together pass nine bits (`fm-clip7`,
  `-5`, `-4`).
- Referee: the board as in D-21, the six channels read from the pins in their turns.
- Conclusion: **measured on the board**, for the bits that reach the output:
  - `$21` bit 1: the LFO's divider counts slots, 24 a sample, a count taking its rate's steps and one slot more; the
    tremolo is then four slots ahead of the vibrato, as S4 of channel 1 has them.
  - Bit 2: D-22. Bit 3: every phase starts each sample from nothing, so an operator sounds one increment's worth.
    Bit 4: the top bit of every operator's fourteen-bit output is inverted, so a silent channel reads -256. Bit 5:
    every envelope holds where it is, and puts out no attenuation.
  - `$2C` bit 5: every channel carries the DAC's level, whether `$2B` enables the DAC or not; bits 4, 6 and 7 change
    nothing on the pins.
  - The DAC's data reads `$80` from reset (channel 6 at 0 with the DAC enabled and never written).
  - A channel's sum: its carriers' top nine bits added one at a time in the order S1, S3, S2, S4, the sum held to
    nine bits at each addition (`fm-clip7` and the silent carriers under bit 4 tell the order; adding all and
    clamping once fails both).
  - Not built: the test read (`$21` bits 0, 6 and 7, `$2C` bit 4), whose status reads were not measured here; D-16's
    note on the bench's test read stands.
- 111 of the 120 voices of D-21 to D-23 are the board's to the sample on the chip; the nine others are this entry's,
  whose test register write lands a sample or more earlier on the board than at once (D-15), and differ there.
- Pinned by: `voices.rs`'s `the_chip_voices_are_the_boards_sample_for_sample` (the nine held to their counts).
- Implemented in: the commit "Nephrite, stage 5: SSG-EG, CSM and the test register".

### D-24. The operators' order and the key's moment: S1 is keyed a sample after S2-S4 and its output used at once, each channel's key is taken a slot after the last channel's, the LFO's count and the envelope's cycle are the same for every operator
- Opened: 2026-10-06, at stage 5's CSM step, by the CSM key, which reaches the four operators of channel 3 at once
  and did not reach S1 when Nephrite expected it to, and by the chip given the board's write times (D-21), which
  showed where the whole machine's 68000 timing had been absorbed by earlier fits.
- Documents read: those of D-17 and D-20.
- Test program: the key sweeps (`mdboard.py`'s `LANDING`: S1, S2, S3 or S4 alone on each of the six channels, its key
  moved across the sample by NOPs after the wait for the busy flag, 40 runs each, 960 in all), the CSM sweep, the
  envelope by operator (`fm-eg-c<n>s<k>`), and the LFO's 61 voices of D-20 run again with their buses logged.
- Referee: the board as in D-21.
- Conclusion: **measured on the board.**
  - A key written to `$28` is taken by a sample if it comes `KEY_LEAD` (472 master clocks) before the sample's
    deadline for channel 1, a slot less for each channel after it in the order 1, 2, 3, 4, 5, 6. S1 has its key a
    sample after S2-S4 (in all 960 runs S1 first sounds a sample after the others at the same place), and the CSM key
    of D-22 reaches all four at once.
  - S1's output is used in the sample it is made by S2, S4 and the sum, and by S3 in the next; with S1 a sample
    behind, this is D-17's first rule seen from the other side, and the voices of D-17 to D-19 are unchanged by it.
  - Every operator takes the LFO's count as it stood before its sample's step, and every channel the envelope's
    cycle of its sample. D-20's per-operator counts and channel 6's earlier count were S1's key and the whole
    machine's placement seen through the voices; with the chip given the board's writes, 60 of the 61 LFO voices are
    the board's under these rules, channel 3's special mode under the vibrato among them (D-20's open item), and the
    61st, `fm-lfo-restart`, differs only where its writes to `$22` land (D-15).
  - The frame: a deadline is now the last moment a write is in time for its sample, 472 master clocks after D-19's
    key moment. D-19's reset restarts the cycle with a key moment 87 after the assertion (`RESTART` 559), and its
    release rule counts from the key moment. The board's first deadline after power-on falls 537 master clocks into
    its clock (every key of the 960 runs and the CSM sweep's 40 in the board's sample with it there, and not with it
    7 either side). Nephrite's whole machine keeps D-2's placement against the picture: the key moment 793 after
    power-on, the deadline 1,265 (`POWER_ON`).
- Pinned by: `voices.rs`'s `keys_and_the_timers_load_land_in_the_boards_samples`, `the_lfo_voices_on_the_chip_are_the_boards`,
  `the_chip_voices_are_the_boards_sample_for_sample`, and the earlier voices' tests, which all pass under the frame.
- Implemented in: the commit "Nephrite, stage 5: SSG-EG, CSM and the test register".

### D-25. When each register's write is taken: by a slot edge after its sample's deadline, measured part by part, up to 64 edges after; each sample made once its last edge has passed
- *Split from D-15 on 2026-10-06: the first two entries are D-15's as they were written.*
- ~~Not measured: when a held write lands.~~ (D-15, 2026-10-05) Every voice compared since makes its writes before its
  key-on and at least 16 of the 68000's clocks apart, where applying a write at once and holding it for its operator's turn give the
  same samples. *The writer's step measured it for the key register alone: a key-on is in time for a sample if its
  strobe rises no later than 87 master clocks into the cycle the reset line's assertion started (D-19). Nephrite
  takes every other write by the same moment, which is argued.*
- Measured 2026-10-06, at stage 5's SSG-EG and CSM step, not implemented. `mdboard.py`'s `LANDING` sweeps move one
  write across the sample by NOPs after its wait for the busy flag, 40 runs 28 master clocks apart, and the chip,
  given the board's other writes, is compared without that write to find the first sample the board shows it in.
  Counted from that sample's deadline in D-24's frame, the write came:

  | Write (S4 of channel 1 unless named) | From the deadline, master clocks |
  |---|---|
  | The key, each channel (D-24) | 472 before for channel 1, a slot less for each after |
  | `$22` turning the LFO off; `$21` bit 3 | 469 after to 511 before |
  | `$27` setting channel 3's special mode | 455 to 1,435 before |
  | `$4C` TL, `$9C` SSG-EG, `$6C` AMON, `$B0` algorithm | 777 after to 203 before |
  | `$B0` feedback | 273 after to 707 before |
  | `$3C` multiple; `$A0` frequency | 231 to 1,211 before; 28 after to 952 before |
  | `$B4` AMS; `$B4` panning | 1,029 to 49 after; 1,785 to 805 after |
  | `$21` bit 5; bit 4; `$2C` bit 5 | 1,323 to 364 after; 1,477 to 497 after; 2,037 to 1,050 after |
  | `$2A` and `$2B`, the DAC | at once, in every run |

  A write a register reads late in the sample is taken by a sample whose deadline has passed, by up to two samples
  for `$2C`: the chip's registers are read at their own places in a pipeline longer than a sample, which Nephrite's
  sample-at-a-deadline unit does not have. The multiple's and the frequency's ranges include the sample a changed
  phase step takes to show. So the entry stays open: implementing it needs the unit to make each sample up to two
  samples after its deadline, with every register read at its place. Nephrite takes all of these at once.
- *2026-10-06, at stage 5's write pipeline step:* ~~`$2A` and `$2B`, the DAC: at once, in every run~~: the DAC's two
  sweeps were heard on channel 1, which the DAC does not reach, so they showed nothing. ~~So the entry stays open
  ... Nephrite takes all of these at once.~~ Measured, below, and implemented.
- Measured 2026-10-06, at stage 5's write pipeline step (`board-chip`'s `LANDINGS`: each sweep's writes at the
  master clocks the board's chip took them, from `ym_latches`, and each run's channel from its first sample not at
  rest, 320 samples, with how many samples after the first run's it leaves rest; the DAC's sweeps heard on channel 6,
  256 samples). The chip is given each run's writes and each register part's moment moved 7 master clocks at a time
  until all 40 runs are the board's. Every part has a window about 21 wide, the sweep's 28 less a step, and in every
  window falls one of the chip's slot edges, which D-13 placed 32 master clocks into each slot after a deadline:

  | Write (S4 of channel 1 unless named) | All 40 runs the board's from ... to ... after the deadline | The edge | Master clocks |
  |---|---|---|---|
  | `$4C` TL, `$3C` multiple | 782 to 803 | 18th | 788 |
  | `$9C` SSG-EG, `$6C` AMON, `$B0` algorithm | 777 to 798 | 18th | 788 |
  | `$B0` feedback | 275 to 296 | 6th | 284 |
  | `$A0` frequency, `$A4` with it | 1,039 to 1,060 | 24th | 1,040 |
  | `$B4` AMS; `$B4` PMS | 1,034 to 1,055; the same | 24th | 1,040 |
  | `$B4` panning | 1,791 to 1,812 | 42nd | 1,796 |
  | `$22` turning the LFO off | 1,477 to 1,498, and 1,502 | 35th | 1,502 |
  | `$21` bit 3; bit 5; bit 4 | 471 to 492, and 494; 1,327 to 1,369; 1,477 to 1,498, and 1,502 | 11th; 31st; 35th | 494; 1,334; 1,502 |
  | `$27` bit 6, channel 3's special mode | 557 to 578 | 13th | 578 |
  | `$2C` bit 5 | 2,034 to 2,055 | 48th | 2,048 |
  | `$2A`, `$2B`, the DAC's data and enable, on channel 6 | 2,712 (2,650 and 2,750 not) | 64th | 2,720 |
  | The key, channel 1 (D-24) | | 12th before | 472 before |

  Each edge was then run alone and every run was the board's at it. The first table's times differ from these where
  the part reaches the output a sample or more after the sample that takes it: the multiple, the frequency, channel
  3's mode and the LFO by one sample in Nephrite's unit, PMS by nearly two. `$6C`'s decay rate (its sweep fits from
  before 300 to 1,000 after, the envelope's cycle coming every third sample) and the key-off (D-24's lead) agree with
  the table. The DAC's level in a sample's output may be written up to the 64th edge after its deadline, 2.7 samples,
  where every other part is taken by the 48th.
- One sweep fits no moment: `lfo`, the LFO switched on just before the key-on, is unlike the board's in all 40 runs at
  every moment of `$22` from 1,500 before to 2,500 after, with the switch-on taken apart from the switch-off as well,
  and was before the pipeline too. Its output is a level louder than the board's in many samples from the attack on;
  it is the LFO's (D-20), not this record's.
- Implemented 2026-10-06 (`ym2612.rs`): every write but the timers' (`$24`-`$26`, `$27`'s timer bits, `$21` bit 2)
  waits in order with the deadline of the sample that takes it, each part of a register by its own edge (`takes`);
  a sample is made `TAKE_LAG`, the 64th edge, after its deadline, the writes it takes applied first; the timers tick
  at their own moment ahead of it and leave CSM's key for it; both edges of the reset line make every sample due by
  then before they act. Parts not measured are argued: the other operators' registers (`$30`-`$9F` but those above)
  by the 18th edge, the DAC's ninth bit (`$2C` bit 3) with the DAC, `$21`'s bits 0, 1, 6 and 7 and the rest of `$2C`
  at the deadline. Only S4 of channel 1 was swept; whether another operator's or channel's part is taken a slot
  apart, as the key is, is not measured, and every one is taken by the same edge.
- Result, on the chip given the board's write times: 20 of the 21 sweeps the board's in every run; the step's 120
  voices 114 (the test register's `fm-test-pg`, `fm-test-ugly-chain` and `fm-test-eg0` now among them); §23's 61 LFO
  voices all (`fm-lfo-restart` now among them). On the whole machine placed as the board is, the same: the sweeps'
  840 programs, rebuilt in `voices.rs` and checked against the board's by hash, 800 the board's (`lfo`'s 40 not), 114
  of the 120, all 61. Six test-register voices still differ (`fm-test-lfo-late`, `fm-test-pg0`, `fm-test-eg`,
  `fm-test-ugly`, `fm-test-2c`, `fm-test-2c-dac`); for `fm-test-lfo-late`, no moment of `$21` bit 1 from 100 edges
  before to 64 after changes a sample, so its difference is not when the write is taken. The others are not traced.
- Conclusion: **settled for the parts in the table, on S4 of channel 1**, and argued for the rest as above.
- Pinned by: `voices.rs`'s `every_write_is_taken_by_the_boards_sample` and `..._on_the_whole_machine`,
  `the_chip_voices_are_the_boards_sample_for_sample`, `the_lfo_voices_on_the_chip_are_the_boards`,
  `the_lfo_voices_are_the_boards_sample_for_sample`.
- Implemented in: the commit "Nephrite, stage 5: D-15's write pipeline".

### D-26. The picture's place against the 68000's start, and where the VDP takes the 68000's cycle: the VDP on line 159, 745 master clocks in, as the 68000 starts; its control port read and written 14 master clocks before the cycle ends; a write held for a full FIFO released 24 after its slot, on the 68000's own clock
- Opened: 2026-10-06, at stage 5's picture step, by the decision to remove stage 4's frame offset (`Nephrite_Native.md`
  §22.3): Nephrite's picture started 0.61 of a frame from the board's against the 68000's start, measured only to within
  an interrupt's latency.
- Documents read: none new. The 68000's manual, for the cycle's states and DTACK, as before.
- Test programs (run on the board and in Nephrite alike; every one starts at the reset vector):
  - **HV through the Z80's area** (`mdboard.py port-zbus 8`): the Z80's bus taken, H40 and mode 5 set, then 4,000 rounds
    of a word read of `$C00008` stored as two bytes into the Z80's RAM, with a shift of 1 to 8 between rounds so that
    the reads drift across the line. No access to the 68000's RAM, whose refresh's few long waits are not modelled
    (D-2); the bus refresh, which is, times the loop. The board's 5,806 stored bytes are read from its Z80 bus
    (`TB_ZBUS`).
  - **The status the same way** (`port-zbus 4`), reading `$C00004`.
  - **The same through the 68000's RAM**, in modes 4 and 5: abandoned, the RAM's waits of four and five clocks making
    the loop drift (36 of 750 writes).
  - `write-landing-*` and `dma-start-*`, now captured frame by frame from power-on (`mdboard.py landing-frames`).
- Referee: the board as in D-1: `TB_MEM`'s strobes, `TB_PICTURE`'s display enables and syncs, `TB_ZBUS`. No RTL file was
  opened.
- Measured:
  - **Power-on.** The board's VDP runs from power-on, its first display period starting 153,962.5 master clocks in; the
    68000's first bus cycle comes at 698,238, on line 159 of that frame.
  - **The VDP's place.** Nephrite's VDP placed at line 159 and moved a master clock at a time against the 68000's start,
    the HV program's bytes are the board's in all but 91 of 5,806 at 745 (and 744, the H counter's step being two clocks
    there), and in 391 or more at every other place from 735 to 756. The 91 are H values in horizontal sync one count
    from the board's, and the first half of a few vertical-blank lines two counts from it, left open.
  - **The first bus cycle.** The board's first cycle waits the bus refresh (its DTACK at 41 cycles of the bench's clock,
    not 13): the refresh runs from power-on, and the request before the 68000's start is pending at its first access,
    111.5 master clocks old. Nephrite had no wait there, so every later cycle of its was 14 master clocks early against
    the board's, which every placement fitted against the 68000's start had absorbed. With the wait, those placements
    are the same against the board's power-on and 14 later against the 68000's start: the bus refresh's 875 becomes 889,
    the main RAM's first request 1,414 becomes 1,428 and the YM2612's first deadline 910 becomes 924; `genesis.rs`'s RAM
    loops count from the first cartridge read's strobe, which the wait holds.
  - **The 68000's clock.** Of 3,000 bus cycles on the board, every one starts on the 68000's clock (the bench's cycle 4
    modulo 14). Nephrite's started off it after every wait on the VDP, the wait ending at a slot.
  - **A write held for a full FIFO.** In the write-landing program's set-up, 1,243 data-port writes wait for the FIFO;
    on the board every one's DTACK falls exactly 4 master clocks after the slot that frees an entry in Nephrite's
    tables, and the next cycle starts on the first of the 68000's clocks at least 24 after that slot (24 to 30 by the
    slot's place among the 68000's clocks, 177 or so of each).
  - **A refresh during the hold.** Where the bus refresh's request falls inside a held write, the next cartridge read on
    the board waits two clocks when the request came up to 118.5 master clocks before that read starts, one clock at
    125.5 and none from 132.5, in every hold the trace has: the refresh is done in the hold, the bus its own for some
    126 master clocks from the request in Nephrite's clock, and a read starting before then waits for it, two clocks at
    most, the wait of a pending request.
  - **The control port.** With the VDP placed from the HV program and the FIFO's releases from the board's DTACKs, the
    two disagree by 14 master clocks unless the HV counter is read 14 before the 68000's cycle ends, where Nephrite read
    it at the end; `register_writes_show_where_the_board_shows_them` then fails unless a write to the control port is
    taken there too; and the status program's bytes are the board's in all but 4 of 5,800 with the status read there, 22
    at the end. The data port's writes stay at the cycle's end, where D-11's start of 176 and the FIFO's releases fit
    them.
  - **Mode 4's H counter.** At power-on, before mode 5 is set, the board's HV read gives H `$00` on every read while V
    counts; Nephrite's H runs. Not built; mode 4's HV read is left open.
- Conclusion: **the picture moved.** At the 68000's start the VDP stands 745 master clocks into line 159 (`START_LINE`,
  `START`); the bus refresh's request before it is pending; the 68000's cycles keep its clock; a held write ends 24
  after its slot; a refresh in the hold is done there; the control port is taken 14 before the cycle ends (`HV_LATE`).
  The picture after Nephrite's first frame, which is lines 159 to 261 of the board's frame 0, is the board's frame 1, so
  that Nephrite's picture after n frames is the board's frame n-1 counted from 0.
- With all of these, the write-landing program's first 7,114 cycles from the reset vector, its set-up of 1,243 held
  writes among them, start where the board's do but for five: a cartridge read the board holds one clock where the
  refresh's request came 126 master clocks before it in Nephrite's clock, the edge of the rule above, which the board's
  half-clock phase leaves undecided. From the frame's last line on they part (below).
- Open: the refresh's one-clock overlap at its edge; the H counter in horizontal sync and on some vertical-blank lines;
  the status's horizontal-blank bit in 4 reads; the FIFO's count entering the frame's last line (one entry more in
  Nephrite than on the board, seen once, a write held 126 master clocks longer); mode 4's HV read; and the write-landing
  and transfer sweeps frame by frame, where 213 of 264 points are the board's to D-11's tolerance and the rest are a
  slot from it, the board's own alternation between two slots coming on the other frames in Nephrite
  (`write-landing-10`). Stage 4's tests, which hold Nephrite's value to those the board shows across its frames, all
  pass.
- Pinned by: every stage 4 test with the move; `genesis.rs`'s RAM loops; the voices' tests at the one placement.
- Implemented in: the commit "Nephrite, stage 5: the picture placed against the 68000's start".
