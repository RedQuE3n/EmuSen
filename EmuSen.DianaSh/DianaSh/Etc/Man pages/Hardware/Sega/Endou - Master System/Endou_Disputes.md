# Endou_Disputes — the disputes log

*Created 2026-10-19, at stage 0, with its first three entries, found while writing `Endou_Plan.md`.* This is the log
`Endou_Plan.md` §1.5 defines, in the shape `Nephrite_Disputes.md` keeps. An entry records a rule the documents and the
test programs leave open or in disagreement, the evidence taken, and the rule as implemented. It is the record of
which sources each dispute step read, so that the clean-room protocol of `Endou_Plan.md` §1.4 can be audited from
outside. A dispute in a Beryl crate goes to that crate's record page.

## The rules this log keeps

- **A dispute step is separate.** It ends with a rule stated in hardware terms, and a writer's step implements it and
  cites the entry.
- **The order of recourse is fixed:** the documents, then a test program written for the purpose, then a referee, run
  as a black box first. No emulator source is ever read.
- **Every RTL file opened is listed** with the lines read and what for. Nothing from it is transcribed, here or in the
  code.
- **An entry is never deleted.** A conclusion later overturned is struck through and followed by the new one.

## Entry shape

```
### D-<n>. <component>: <the rule, one sentence, in hardware terms>
- Opened: <date>, by <what left it open or showed the disagreement>
- Documents read: <title, section> for each
- Test program: <the program written or run, and what it showed>
- Referee: <what was run or read, and how>
- Conclusion: <the rule as implemented, in prose>; argued | measured | open
- Pinned by: <the test that fails if the rule is changed>
- Implemented in: <the commit that cites this entry>
```

## Entries

### D-0. The PSG: Sega's copy in the Mark III, the Master System and the Game Gear has a 16-bit noise register tapped at bits 0 and 3; the SG-1000's discrete SN76489AN a 15-bit one tapped at bits 0 and 1
- Opened: 2026-10-19, at stage 0, by two documents that disagree. MacDonald's *Sega Master System VDP documentation*
  says of the PSG inside every VDP of the family that "it is identical to the stand-alone version of the same chip"
  (§1). SMS Power's *SN76489* gives the shift register as "16 bits for the SMS (1 and 2), Genesis and Game Gear, 15 bits
  for SG-1000, OMV, BBC Micro, SC-3000H, ColecoVision and Tandy 1000; other systems need investigation", and lists the
  SG-1000 Mark I and II and the SC-3000 as carrying the discrete SN76489AN ("SN76489 sightings").
- Documents read: `msvdp.txt` §1 (11/12/02); SMS Power's *SN76489*, its "sightings" and LFSR sections (page modified
  2025-09-04, fetched 2026-10-04).
- Test program: none yet. At stage 5, periodic noise at each rate on each model, its period read from the output.
- Referee: none yet. The references run every model; their agreement is weak evidence, being emulators. A recording
  of an SG-1000's noise, if a published one is found, would settle the discrete side; no console is run for the
  project (`Endou_Plan.md` §10 Q11, decided 2026-10-19).
- Conclusion: **argued** for SMS Power's account. It is the later document, it carries MacDonald's own noise samples
  of three of the consoles (`Beryl_SN76489.md` §1), and it is specific where `msvdp.txt` is general; `beryl-sn76489`
  already implements both registers as `Variant::SEGA` and `Variant::DISCRETE` (§2 there). The Mark III is not in the
  document's list; it carries the 315-5124, as the first Master System does (`msvdp.txt` §1), and is argued to have the
  same PSG. Endou gives `SEGA` to every model from the Mark III on and `DISCRETE` to the modelled SG-1000.
- Pinned by: `beryl-sn76489`'s test of white noise's feedback in both variants (`Beryl_SN76489.md` §3); Endou's
  per-model choice gains its own test at stage 5.
- Implemented in: `beryl-sn76489` (2026-10-05) for the two registers; Endou's choice at stage 5.
- *At stage 5 (2026-10-19):* the test program promised above is `programs/voices.s`, its periodic noise at rates 0 and
  2 and from tone 2, the period read from the output: 16 shifts (436.9 Hz at rate 0) on the Mark III and both Master
  Systems, 15 (466.1 Hz) on the SG-1000, pinned by `programs::each_console_s_psg_sounds_its_own_periodic_noise`. The
  references, run as black boxes on the same images: Genesis Plus GX and Gearsystem give the SG-1000's image the 15-bit
  register and the Master System's the 16-bit one; PicoDrive, BlastEm and SMS Plus GX give both the 16-bit one, modelling
  no discrete chip (`Endou_Native.md` §12.6). Every reference that tells the consoles apart agrees with the conclusion,
  which stays argued: they are emulators, and no recording of an SG-1000 was found.

### D-1. The Master System's colour output: the level at which each value of a CRAM entry's two-bit component is shown
- Opened: 2026-10-19, at stage 0. `msvdp.txt` §5 gives CRAM's format, `--BBGGRR`, and "up to 64 possible colors",
  with no output levels. Nephrite measured the 315-5313's levels in mode 4 on Nuked-MD's board (red and green 0, 95,
  161, 255; blue 0, 109, 161, 255) and recorded that "no measurement of a console's mode 4 output is in the corpus"
  (`Nephrite_Disputes.md` D-7). The 315-5124 and 315-5246 are other chips with their own outputs, and the Genesis's
  board model is no evidence for them.
- Documents read: `msvdp.txt` §5; `Nephrite_Disputes.md` D-7.
- Test program: to be written at stage 3: the 64 colours in mode 4, as Nephrite's `mode4-colours-0` and `-32`.
- Referee: SMS_MiSTer's output levels are its author's choice, so weak. No console is run or captured for the
  project (`Endou_Plan.md` §10 Q11, decided 2026-10-19), so only a published measurement of a console's RGB output,
  made by others and read as a document, would settle it; failing one, the levels stay argued.
- Conclusion: **open.** Until it is settled the VDP crate outputs CRAM indices (`Endou_Plan.md` §3.3) and Endou shows a
  two-bit component at four equal steps, 0, 85, 170 and 255, the plainest reading of a two-bit DAC, named as a loss in
  the build record. The Game Gear's twelve bits (J1) are the same question at four bits a component.
- *Read at stage 3 (2026-10-19):* SMS Power!'s "Palette" page (fetched at stage 1, modified 2024-02-25) gives each
  colour of the 315-5246 as `#00`, `#55`, `#AA` and `#FF` a component, the equal steps above, and says that "the Mark III
  and SMS1's VDP has the first level of blue slightly boosted in brightness, affecting 16 colours", showing blue level
  1 as `#68` on those chips where the 315-5246's is `#55`. The page names no measurement, instrument or console behind
  the values, so they are a document's statement, not a published measurement. Endou keeps the equal steps on every
  model, as stage 0 settled them; whether to take the page's boosted blue for the 315-5124 is a question for the
  tester (`Endou_Native.md` §10.12). The crate outputs CRAM entries (`Beryl_SmsVdp.md` §1), so either answer is a table
  of Endou's alone.
- *Decided 2026-10-19 by the tester, at stage 4:* the page's boosted blue is taken for the 315-5124. **Conclusion, as
  implemented:** the 315-5246 shows each component at 0, 85, 170 and 255; the 315-5124 (the Mark III and the first
  Master System) the same but for blue's first level, `#68`. Both are **read from a document, not measured**: the equal
  steps are the page's values for the 315-5246, the boost its statement for the 315-5124. What a console puts out
  stays unmeasured, and no console is run for the project.
- Pinned by: ~~`machine::SMS_COLOURS` and `SMS1_COLOURS` through `machine::tests::the_315_5124_boosts_only_its_first_blue_level`~~
  `beryl-sms-colours`' `SMS_COLOURS` and `SMS1_COLOURS` through its `the_315_5124_boosts_only_its_first_blue_level`, the
  tables and the test moved there unchanged at stage 8 (2026-10-19, `Endou_Native.md` §15.5),
  `programs::a_program_s_picture_is_its_tiles_sprite_scroll_mask_and_mid_frame_colour` (a digest for each chip) and
  `EndouTests.The_vdp_draws_a_synthetic_program_s_picture`.
- Implemented in: the stage 3 commit, as the equal steps; the stage 4 commit, the 315-5124's blue.
- *Read at the Game Gear's stage J1 (2026-10-20):* the Game Gear's GG mode has four bits a component [R MacDonald §5;
  Sega's Game Gear VDP manual, "Color RAM": "Four bits each of the 12-bit data are assigned to R, G and B"]. No document
  read gives the LCD's levels for them. `beryl-sms-colours`' `gg_colour` shows each at sixteen equal steps, 17 apart,
  the four-bit form of this entry's equal steps, **argued**; a Master System cartridge's `--BBGGRR` entries on the Game
  Gear take the 315-5246's table, the chip being "identical to an SMS2 VDP" but for GG mode [R SMS Power!'s "GG VDP"].
  What the panel makes of either is the LCD model's question (`Endou_Plan.md` §2.5), not the VDP's. Pinned by
  `beryl-sms-colours`' `a_game_gear_word_is_red_green_and_blue_from_its_low_bits_at_equal_steps`.

### D-2. The YM2413's operator ROMs: the referee's log-sine and exponent tables are the OPL2's, entry for entry, wherever an entry can be seen
- Opened: 2026-10-19, at stage 0. *OPLx decapsulated* gives the YM3812's two 256-entry ROMs, read from its die.
  `Nephrite_OperatorTables.md` §4 measured that Nuked-MD's OPN2 computes with tables equal to them, with the caveat
  that the board is a model whose ROMs are evidence about a console only so far as they were read from a die, and
  recorded that the one statement about the OPLL seen was a search engine's summary of an SMS Power page, which was
  not opened and on which nothing relies. No document read for Endou says what the OPLL's ROMs hold.
- Documents read: *OPLx decapsulated*; `Nephrite_OperatorTables.md` §4.
- Test program: at stage 9, a single carrier at fixed attenuations on the OPLL, its samples compared with the referee's;
  then each of the 512 entries moved up and down by one in a build made for the purpose, as Nephrite's D-16 did for the
  OPN2.
- Referee: Nuked-OPLL run as a black box, allowed by `Endou_Plan.md` §10 Q4 (decided 2026-10-19); its agreement is evidence
  about a console only so far as its own tables came from a die. The SMS Power page the summary came from is a
  document, and is read at the fetch.
- Conclusion: **open.** P5 of `Endou_Plan.md` predicts equality. If the tables are the OPL2's, they move to a Beryl
  crate with their provenance page, Nephrite and Endou both reading them (`Endou_Plan.md` §3.2); if not, the OPLL's are
  Endou's own, with a provenance page of their own.
- Pinned by: —
- Implemented in: —
- *At stage 9 (2026-10-20):* the referee was not run: its bench is written (`Endou_Opll.md` §7), and building
  `opll.c` from the fetched tarball waits on the tester's approval. The SMS Power! page the stage-0 summary came from,
  "YM2413", was read: it says nothing of the ROMs, and links the reverse-engineering notes that carry code, which were
  not opened (`Endou_Opll.md` §0). Genesis Plus GX's `nuked` YM2413, run as a black box, gives levels that agree with
  the OPL2's tables to its resolution: a full sine at 4,111 against 510 nine-bit units scaled to it, and every 3 dB
  volume step to 0.1 dB down to 24 dB [M `volsweep.py`]; through its resampler no single entry can be told from its
  neighbour. **Still open**, the OPL2's tables used as argued; by `Endou_Plan.md` §3.2's condition they **do not move**
  to a Beryl crate, and Endou holds its own copy, `src/opll/tables.rs`, with its provenance in `Endou_Opll.md` §2.
- Pinned by: `opll::tests::the_tables_are_the_opl2_s_as_printed` (the copy), nothing yet against a chip.
- Implemented in: the stage 9 commit, as argued.
- *Measured on the referee (2026-10-20):* the tester approved building `opll.c`, and Nuked-OPLL now runs sample for
  sample (`Endou_Opll.md` §7).
  - **The test.** A build made for the purpose, never committed, moved one entry up or down by one through an
    environment variable. All 1,015 changes that stay inside a ROM's width were compared against the referee. The
    voices were the 54 exact synthetic voices, 40 modulated voices, and 640 voices of TL 0-4 by the four KSL codes
    across F-numbers and blocks.
  - **The result.** 1,010 changes alter at least one sample. Five do not: log-sine entries 0 and 1 moved down, 2 and 4
    moved up, and 6 moved down. **None of the five can alter any output pin** [M `d2.py`, `d2b.py`, `d2c.py`, and an
    enumeration over every attenuation]. Entries 0 and 1 moved down never change the twelve-bit output. The other
    three change it only in its lowest bit, at a few attenuations, and the chip drops that bit everywhere it goes: the
    DAC keeps the top eight bits, the carrier's modulation clears it, and feedback halves each output.
- Conclusion: **measured.** The referee computes with tables equal to the OPL2's in every entry an output can show.
  The caveat stays: the referee is a model, and it is evidence about a console only so far as its ROMs came from a die.
  By `Endou_Plan.md` §3.2 the tables should now move to a Beryl crate that Nephrite and Endou both read. That move
  touches Nephrite, so it waits on the tester (`Endou_Native.md` §16.15).
- Pinned by: `opll::tests::the_tables_are_the_opl2_s_as_printed`; against the referee, the synthetic voices of
  `examples/opllref.rs`, rerun as `Endou_Opll.md` §7 describes.
- Implemented in: the stage 9 commits.

### D-3. The I/O chip: on the Mark III a TH pin set as an output reads low and one left an input reads high
- Opened: 2026-10-19, at stage 2, by the documents and a referee disagreeing. SMS Power!'s "Peripheral Ports" says the
  TH function "is not present on the Mark III (there's no TH at all)", and its "Controller Port" gives pin 7 as "TH
  (Gnd on Mark III)", which would read low whatever port `$3F` says. Sega's Mark III manual lists port `$3F`'s TH
  directions and levels and `$DD`'s TH bits for the Light Phaser ("P1 TH (L.Gun Synch)", appendix B), as if the pin
  worked. The same "Peripheral Ports" gives the Japanese Master System's: with TH set as an output, `$DD`'s bits 7 and
  6 "are always zero regardless".
- Documents read: SMS Power!'s "Peripheral Ports", "Controller Port", "Region Detection"; Sega's Mark III manual,
  "Input/Output (I/O) ports" and appendix B; MacDonald's `smstech` §4.
- Test program: `programs/region.s` (SMS Power!'s region check) and `programs/pads.s`, on each model.
- Referee: the five libretro cores, as black boxes (`Endou_Native.md` §9.11). Genesis Plus GX's Mark III model finds a
  Japanese console by the region check and reads TH high with the pins left as inputs; SMS Plus GX and Gearsystem in
  their Japanese settings agree; PicoDrive and BlastEm find an export console. SMS_MiSTer is not run (no VHDL
  simulator, `Endou_Native.md` §8).
- Conclusion: **argued.** On the Mark III, TH as an output reads low, as the Japanese Master System's is documented to,
  which every account agrees software sees (a Japanese console); TH as an input reads high, as a pin no pad drives
  does on the other consoles and as the one referee with a Mark III model has it. A pin tied to ground would read low
  as an input too; nothing that reads TH as an input on a Mark III (the Light Phaser) is built before stage 11, which
  reopens the entry if its test program needs it. TR stays an input on the Mark III [A].
- Pinned by: `io::tests::the_export_consoles_read_th_back_and_the_mark_iii_does_not`,
  `programs::the_region_reads_back_as_each_console_s`, `programs::the_pads_reset_button_and_th_lines_read_through_the_joystick_ports`.
- Implemented in: the stage 2 commit.
- *At stage 3 (2026-10-19):* the VDP latches its H counter when TH rises on either port (`msvdp` §11). The rise is taken
  from the levels this entry gives each console, so on the Mark III a TH pin switched from an output to an input rises
  and latches as on the others; a Mark III whose TH is tied to ground would never latch. FluBBa's VDP test latches
  through port `$3F` and passes on the Mark III's model as on the others (`Endou_Native.md` §10.7). The Light Phaser's
  stage reopens this with the entry.

### D-4. Port $3E: a slot or the work RAM switched off drives nothing, and port $3E reads as an undriven port
- Opened: 2026-10-19, at stage 2, by the referees disagreeing with the document and with each other on
  `programs/port3e.s` (`Endou_Native.md` §9.11). MacDonald: "Writes to even addresses go to memory control register",
  reads of `$00`–`$3F` "return the last byte of the instruction which read the port" on the first Master System and
  `$FF` on the second; with a slot enabled and nothing in it, a read returns "the last byte of the instruction which
  read memory" (`smstech` §3, §4). SMS Power!'s "RAM": "Bit 4 of I/O Port $3E must be 0 for RAM to be enabled".
- Documents read: `smstech` §3–§4; SMS Power!'s "Port $3E", "RAM", "I/O Port Map - SMS"; Sega's Mark III manual,
  "Memory enables".
- Test program: `programs/port3e.s`: the cartridge switched off from a routine in RAM and `$0100` read, the work RAM
  switched off and written, port `$3E` read; on each model.
- Referee: BlastEm, Gearsystem and PicoDrive read the cartridge with its slot off and keep the write to work RAM
  switched off; Genesis Plus GX and SMS Plus GX did not finish the program; every one reads port `$3E` as `$FF`, on the
  first Master System's setting too. None agrees with another closely enough to be evidence against the document.
- Conclusion: **argued** from the document: a slot switched off drives nothing, and the read is the floating bus on the
  first Master System and the Mark III [A for the Mark III] and `$FF` on the Master System II; work RAM switched off
  takes no write and reads the same way (MacDonald "haven't checked" the read [R], so it is argued); port `$3E` reads
  as an undriven port. SMS_MiSTer would referee it when a simulator is allowed; a game shown to need otherwise reopens
  the entry.
- Pinned by: `bus::tests::port_3e_disables_the_cartridge_and_the_ram`, `programs::port_3e_takes_the_cartridge_and_the_ram_off_the_bus`.
- Implemented in: the stage 2 commit.

### D-5. Codemasters' mapper: every one of the three slots is banked by a write to its own addresses
- Opened: 2026-10-19, at stage 2, by two documents that disagree. MacDonald: "All SMS and GG games by Codemasters have
  a fixed 32K ROM area at $0000-$7FFF and a 16K bank at $8000-$BFFF. The bank can be selected by writing a byte to
  address $8000" (`smstech` §2, 2002). SMS Power!'s "Mappers" (modified 2024): the control registers "are mapped to all
  16KB of each bank and the first 1KB is no longer protected", writes at `$0000`, `$4000` and `$8000` banking slots 0,
  1 and 2, initialised to banks 0, 1 and 0, and bit 7 of slot 1's value mapping Ernie Els Golf's 8 KiB of RAM at
  `$A000`.
- Documents read: `smstech` §2; SMS Power!'s "Mappers", "Codemasters Header".
- Test program: `programs/codemasters.s`, banking slot 2 then slot 1 by their own addresses, then the RAM bit.
- Referee: PicoDrive, the one reference that took the synthetic image as Codemasters' board, banks slot 1 by a write
  to `$4000` and maps the RAM, its bytes equal to Endou's; the others ran it on Sega's mapper.
- Conclusion: **argued** for the later page: it is a superset of MacDonald's account (a game that banks only slot 2
  runs the same under both), it names the RAM board, and the referee agrees. Neither of the library's two Codemasters
  images wrote a bank register in the survey's 3,000 frames with no VDP [M `Endou_Native.md` §9.10], so the library
  has not yet told the two accounts apart; stage 3 runs them further.
- Pinned by: `cart::tests::codemasters_mapper_banks_every_slot_by_its_own_addresses`,
  `programs::codemasters_mapper_banks_by_its_slots_addresses_and_maps_its_ram`, `EndouTests.Each_mapper_banks_its_slots_as_its_board_does`.
- Implemented in: the stage 2 commit.

### D-6. The 315-5124's zoomed sprites: of N sprites on a line, the first N - 4 are widened, the rest only heightened
- Opened: 2026-10-19, at stage 3, by two documents that disagree. MacDonald: the 315-5124 "will only allow the first
  four sprites of the eight shown on a scanline to be zoomed horizontally and vertically, and the remaining four will be
  zoomed vertically" (`msvdp` §10). SMS Power!'s "Sprites" (modified 2025-10-29): "If N sprites on are a scanline, only
  the first N minus 4 are double-width", with a table (two sprites `VV`, four `VVVV`, six `FFVVVV`, eight `FFFFVVVV`),
  and "To get four sprites to zoom in both dimensions, four 'sacrificial' sprites will need to be added after them";
  its "VDP Registers" says the same of register 1's bit 0.
- Documents read: `msvdp` §10; SMS Power!'s "Sprites" and "VDP Registers".
- Test program: none run; the two readings agree with eight sprites on a line and differ with fewer.
- Referee: none. The references run a Master System II's chip by default; SMS_MiSTer is not run (no VHDL simulator).
- Conclusion: **argued** for SMS Power!'s account. It is the later document, it is specific where MacDonald's is
  general, its table and its advice to add sacrificial sprites describe a rule someone tested with fewer than eight
  sprites, and MacDonald's sentence describes the case of eight, where the two agree.
- Pinned by: `compose::tests::the_first_master_system_widens_only_n_minus_four_of_n_zoomed_sprites`.
- Implemented in: the stage 3 commit.
- *At stage 4 (2026-10-19):* the tester approved a VHDL simulator, and GHDL 6.0.0 was installed for the project
  (`Endou_References.md` §4a). SMS_MiSTer could not be run under it. Its VHDL imports cleanly as VHDL-93, but its
  console's top entity, `system`, and its `vdp` and `t80s` entities each need the resource library `altera_mf`, Intel's
  simulation models for the FPGA's memories, which is not free to fetch and not on the desktop, and its PSG, `jt89`, is
  Verilog, which GHDL does not elaborate [M: `ghdl -m`'s messages; nothing of the RTL was opened]. Writing stand-ins for
  `altera_mf`'s parts would need the RTL's use of them, which is reading the RTL outside a dispute step, so it was not
  done. The entry stays as it was.

### D-7. The background's first pixels under a fine X scroll show the backdrop
- Opened: 2026-10-19, at stage 3, by the references disagreeing with the document. MacDonald: the column counter starts
  drawing "offset to the right by the fine scroll value", and "the SMS VDP does not [show part of the column], and
  instead this area is filled with the backdrop color (and sometimes pattern data from sprite #0, but exactly how this
  is selected is unknown)" (`msvdp` §9).
- Documents read: `msvdp` §9; SMS Power!'s "Tilemap" ("glitches will be visible in the leftmost column").
- Test program: MacDonald's *Scroll Test Program*, frame 300, its middle band scrolled with the left column unmasked.
- Referee: the five references as black boxes: PicoDrive shows the backdrop there, as Endou does; Genesis Plus GX,
  Gearsystem and BlastEm show CRAM entry 0, the colour of the background pixel a wrapped column would give; SMS Plus GX
  was at another phase of the program's scroll. They disagree with each other, so they are no evidence against the
  document.
- Conclusion: **argued** from the document: the pixels before the column counter's first column are the backdrop.
  "Pattern data from sprite #0" is not modelled; no document says when it shows. The Genesis's garbage there
  (`Nephrite_Native.md` §15.2) is its own, a variant's choice at stage 4.
- Pinned by: `compose::tests::the_background_scrolls_by_its_column_counter_and_blanks_the_fine_scroll`.
- Implemented in: the stage 3 commit.
- *At stage 4 (2026-10-19):* the tester approved a VHDL simulator, and GHDL 6.0.0 was installed for the project
  (`Endou_References.md` §4a). SMS_MiSTer could not be run under it. Its VHDL imports cleanly as VHDL-93, but its
  console's top entity, `system`, and its `vdp` and `t80s` entities each need the resource library `altera_mf`, Intel's
  simulation models for the FPGA's memories, which is not free to fetch and not on the desktop, and its PSG, `jt89`, is
  Verilog, which GHDL does not elaborate [M: `ghdl -m`'s messages; nothing of the RTL was opened]. Writing stand-ins for
  `altera_mf`'s parts would need the RTL's use of them, which is reading the RTL outside a dispute step, so it was not
  done. The entry stays as it was.
- *At stage 4 (2026-10-19):* the Genesis's own rule, the wrapped column, is now a variant of the shared composition
  (`Variant::GENESIS`), which Nephrite draws with; nothing of either console's rule changed.

### D-8. OPEN. The CPU's access to VRAM and CRAM: when an access made faster than the chip serves it is lost
- Opened: 2026-10-19, at stage 3, by the plan's thin area (`Endou_Plan.md` §5.2, P3). Sega's Mark III manual: "The VDP
  chip cannot process data any faster than the following rates: 16 Z80A T-States during VBLANK, 29 Z80A T-States during
  active video. ... This constraint applies to Video RAM and Color RAM as well as the internal registers." MacDonald:
  "When writing to CRAM rapidly during the active display, I've found that sometimes data is written to the wrong
  address or that the data isn't written altogether. This happened with an SMS 2, but the same program runs fine on a
  Genesis and Game Gear. I think the cause is that the VDP can only allow so many CPU accesses ... and if you write when
  the VDP is busy processing a previous write, the data is simply gone" (`msvdp` §15). No document read gives the dots
  at which the chip serves the CPU.
- Documents read: those; `Endou_Plan.md` §5.2.
- Test program: `programs/slots.s`: 64 bytes written to VRAM at three rates during the active display with the display
  on (OTIR, 21 cycles apart; OUTI and JR, 28; OUTI, NOP and JR, 32) and at 16 during vertical blanking, read back slowly.
  And a survey: the library's 310 game images (read in place, 1,200 frames each with button presses, on a Master System
  II), every data port access timed against the one before it [M 2026-10-19, an instrumented build in scratch].
- Referee: the five references as black boxes keep every byte of `slots.s` at every rate. SMS_MiSTer is not run (no
  VHDL simulator, `Endou_Native.md` §8).
- Survey [M]: 122 of the 310 images access the data port during the active display, display on, less than Sega's 29
  cycles after the access before it; most at 26-28 cycles (Golden Axe: 523,271 such accesses in 1,200 frames, none
  closer than 28), After Burner at 16, Space Harrier at 11. These games run on the console, so a rule that lost every
  access closer than 29 cycles would break a third of the library; the manual's rates are a bound developers were
  given, not the chip's slots.
- Conclusion: **open; implemented as no access lost** [A]. The picture shows each write at its dot and nothing is
  dropped. What MacDonald saw, at a rate higher than games use, needs the slots' positions, which only a referee or a
  console can give, and no console is run for the project (`Endou_Plan.md` §10 Q11). SMS_MiSTer, run under a VHDL
  simulator with `slots.s` at a rate stepped from 11 to 29 cycles, would place them; installing a simulator is the
  tester's to allow (`Endou_Native.md` §10.12).
- Pinned by: `programs::data_port_writes_at_every_rate_land`.
- Implemented in: the stage 3 commit.
- *At stage 4 (2026-10-19):* the tester approved a VHDL simulator, and GHDL 6.0.0 was installed for the project
  (`Endou_References.md` §4a). SMS_MiSTer could not be run under it. Its VHDL imports cleanly as VHDL-93, but its
  console's top entity, `system`, and its `vdp` and `t80s` entities each need the resource library `altera_mf`, Intel's
  simulation models for the FPGA's memories, which is not free to fetch and not on the desktop, and its PSG, `jt89`, is
  Verilog, which GHDL does not elaborate [M: `ghdl -m`'s messages; nothing of the RTL was opened]. Writing stand-ins for
  `altera_mf`'s parts would need the RTL's use of them, which is reading the RTL outside a dispute step, so it was not
  done. The entry stays open, implemented as no access lost. The TMS9918A's CPU windows (TI's Table 2-2) are not modelled either (`Beryl_SmsVdp.md` §9.3).

- *At stage 6 (2026-10-19), the logged dispute step* the tester decided the same day: SMS_MiSTer's `vdp` entity alone
  under GHDL, with stand-ins for its `altera_mf` memories, reading only that entity's port list and those components'
  interfaces. **Files read: none of the RTL's text.** GHDL 6.0.0 analysed `dpram.vhd`, `vdp_cram.vhd`,
  `vdp_sprite_shifter.vhd`, `vdp_background.vhd`, `vdp_sprites.vhd`, `vdp_main.vhd` and `vdp.vhd` with
  `-fsynopsys -fexplicit -frelaxed-rules -fno-caret-diagnostics`, its messages giving a file, a line, a column and a
  rule and no source [M]. Under VHDL-93 `vdp.vhd` is refused at two case choices whose subtype is not locally static,
  `vdp_sprites.vhd` at five actuals not globally static and two such choices, and `vdp_main.vhd` at one actual; under
  VHDL-2008 `vdp_sprites.vhd` is refused at 26 case choices that are not locally static expressions while the rest
  analyse. GHDL keeps one standard to a library, so no standard analyses the entity whole, and making one do so means
  editing the lines named, which is reading the RTL beyond what the step allows. The step stopped before the port list,
  since nothing could be run with it, and before the `altera_mf` stand-ins were needed. The entry stays open,
  implemented as no access lost (`Endou_Native.md` §13.11). The next way is the tester's: nvc, another VHDL simulator,
  run as GHDL was, or a step allowed to read the lines GHDL names (§13.14 of the build record).

### D-9. The H counter's run and where a line's events fall, by FluBBa's test rather than MacDonald's Genesis
- Opened: 2026-10-19, at stage 3, by two documents that disagree. MacDonald's §11 gives the H counter's jump at
  `$8B`-`$ED` and the active display at `$00`-`$7F`, "taken from the TMS9918 manual and my tests on a NTSC Genesis. Be
  warned that a real SMS may have different results", and says "I don't have information about where events occur
  within a single scanline". FluBBa's *SMS VDP Test* (2009) holds the H counter's reads to a table running `$00`-`$93`
  and `$E9`-`$FF`, and its readme lists the interrupts, the V counter's step and the flags in HClock.
- Documents read: `msvdp` §11; the test's readme and source.
- Test program: the test itself, its 16 timing cases (`Beryl_SmsVdp.md` §3.2).
- Referee: Gearsystem and SMS Plus GX pass every one of the test's Master System cases; Genesis Plus GX all but
  "Offscreen Y, col"; PicoDrive fails the five H count cases; BlastEm did not start the test from a pad press [M,
  2026-10-19, frame 1,500, RAM `$C900` on].
- Conclusion: **measured** by the test program on Endou: the run of FluBBa's table, the events at the dots that pass its
  cases (`Beryl_SmsVdp.md` §3.2), six dots earlier than the readme's HClock, which counts from its own emulator's
  origin. The test's author's console was a Master System; which model is not said, and the 315-5124 and 315-5246 are
  given the same timing [A].
- Pinned by: `programs::flubba_s_vdp_test_passes_on_every_model` (with the corpus), `timing::tests`, `tests::the_h_counter_is_latched_on_th_rising_and_holds_its_value`.
- Implemented in: the stage 3 commit.

### D-10. The 315-5124 and 315-5246 in the TMS9918's modes show four sprites a line, as the TMS9918A does
- Opened: 2026-10-19, at stage 4, by the documents' silence. TI's manual gives the TMS9918A four sprites a line and a
  fifth sprite flag (§2.4.5, §2.3.3); MacDonald gives the Sega chips eight in mode 4 and says of the TMS9918's modes
  only that register 0's bit 3 moves their sprites too (§15); SMS Power!'s pages say nothing of the count.
- Documents read: TI §2.3.3, §2.4.5; MacDonald `msvdp` §10, §15; SMS Power!'s "Sprites", "VDP Registers".
- Test program: `programs/tms.s`, six sprites on lines 100-107, the fifth at X 100 and the sixth at X 130, run on the
  Mark III's model; with eight a line both would show.
- Referee: the references as black boxes on their Master System or Mark III settings (Genesis Plus GX's `mark-III`,
  Gearsystem's "Master System / Mark III", SMS Plus GX's "master system", PicoDrive's "Master System"): their pictures
  at frames 20, 45, 75 and 105 equal Endou's with four a line up to a colour map, 0 pixels each; and the library's 54
  SG-1000 images run on the Mark III's model agree with them as on the SG-1000 (`Endou_Native.md` §11.10) [M
  2026-10-19]. Their agreement is weak evidence, being emulators; SMS_MiSTer could not be run (D-6's note).
- Conclusion: **argued**, four a line on every chip in the TMS9918's modes, the fifth flagged in the status's bit 6 on an
  active line; the Sega chips' bits 4-0 stay undriven [R MacDonald §4].
- Pinned by: `tms::tests::four_sprites_a_line_the_fifth_named_and_208_ends_the_table`,
  `programs::an_sg_1000_program_s_pictures_status_and_4k_reads_are_the_tms9918a_s`.
- Implemented in: the stage 4 commit.

### D-11. The TMS9918A's sixteen colours: TI's colour difference levels as RGB
- Opened: 2026-10-19, at stage 4. The SG-1000's TMS9918A puts out composite video; TI's Table 2-3 gives its 16 colours as
  a luminance and a chrominance amplitude with no phase, and the TMS9928A's and TMS9929A's same colours as Y, R-Y and
  B-Y levels in volts. SMS Power!'s "Palette" gives RGB values taken from Young's document, which says they were "taken
  from mess/vidhrdrw/tms9928a.c in MESS" and computed from TI's levels by a formula it states.
- Documents read: TI's manual, Table 2-3 and its note that "the gray levels differ slightly for the TMS9918A"; Young §1.1;
  SMS Power!'s "Palette".
- Test program: `machine::tests::the_tms9918a_s_colours_are_ti_s_levels_as_rgb` recomputes the table from TI's levels.
- Referee: none; the references show the SG-1000's pictures in their own palettes, and the pictures are compared up to
  a colour map.
- Conclusion: **argued**: Endou derives the colours from TI's levels itself, by the procedure Young's document states (a
  colour difference's zero at 0.47 V, R = Y + (R-Y), B = Y + (B-Y), G from the NTSC luminance weights, each clipped and
  scaled to 255 and rounded), not from the emulator's table. The result equals SMS Power!'s values in 47 of 48
  components; cyan's green is `#EC` against the page's `#EB`, the page having rounded green to two decimals first. The
  colour difference levels stand for the TMS9918A's colours, whose own table gives no phase; transparent over no
  external video is black [R TI §2.4, its note].
- Pinned by: that test, and `EndouTests.The_tms9918a_draws_an_sg_1000_program_s_picture`.
- Implemented in: the stage 4 commit.

### D-12. The TMS9918A's 4K mode moves every access to the cell a 4K DRAM's row and column name
- Opened: 2026-10-19, at stage 4, by the references disagreeing with the documents. MacDonald (`sc3000h` §6, and his letter
  in Young §2.1): with register 1's bit 7 clear the chip puts out six bits of row and column with the address's bits 12
  and 13 above them, so that "even in 4K mode, all 16K of VRAM can be accessed", only at other cells; he tested a
  TMS9929A.
- Documents read: `sc3000h` §6; Young §2.1; TI §2.2.2 ("4/16K selection").
- Test program: `programs/tms.s` writes `$A5` at `$3C40` in the 16K mode and reads `$3C40` and `$3E00` back in the 4K mode.
- Referee: as black boxes, Genesis Plus GX reads `$00` and `$3F`, Gearsystem `$00` and `$00`, PicoDrive and SMS Plus GX
  `$A5` and `$00`; Endou, by the document, reads `$3E00`'s cell as `$A5` [M 2026-10-19]. No two references agree with
  each other, so none is evidence against the document.
- Conclusion: **argued** from the documents, the one source tested on a console. No game of the library has been seen
  to clear the bit; a game that shows otherwise reopens the entry.
- Pinned by: `tms::tests::the_4k_mode_puts_a_row_and_column_of_six_bits_with_the_high_bits_above_them`,
  `tests::the_tms9918a_s_4k_mode_moves_the_cpu_s_cells_and_its_own_alike`, and the program's test.
- Implemented in: the stage 4 commit.

### D-13. The audio control port with no FM unit: a read of `$F2` is the I/O chip's port, or with the I/O chip off the console's undriven bus; a write does nothing
- Opened: 2026-10-19, at stage 5, by two documents that disagree in part. MacDonald: "The SMS normally assigns reads
  from $C0-$FF to the I/O chip. You must disable the I/O chip by setting bit 2 of port $3E if you want to read port $F2
  to perform YM2413 detection", and "When bit 2 is set, all ports at $C0 through $FF return $FF on a SMS 2, Game Gear,
  and Genesis. For a SMS, these ports return the last byte of the instruction which read the port" (`smstech` §4, §7).
  SMS Power!'s "Audio Control Port": "If no YM2413 is present, reading from the audio control port returns varying
  results. It always returns %10 in the lowermost two bits on a non-japanese SMS. On a Mark III without FM unit, it
  returns the input from port 0. Writing to the audio control port has no effect if no YM2413 is present."
- Documents read: those; SMS Power!'s "FM Chip Detection" and "I/O Port Map - SMS" ("$F2 ... depends"); Sega's Mark III
  manual, "Memory enables" (bit 2, "disable joysticks").
- Test program: `programs/fm.s`, the detection as "FM Chip Detection" gives it (the I/O chip off, bit 0 written to
  `$F2` and read back seven times), each read kept, then a tone and writes to `$F2`, `$F0` and `$F1`.
- Referee: none run. The references' own FM units are on by default in three of them, which is the Japanese console's
  behaviour, not the one disputed.
- Conclusion: **argued** for MacDonald's account, which is a measurement of each port range on each export console;
  SMS Power!'s %10 agrees with it on the first Master System, whose floating bus returns the `IN A,($F2)` instruction's
  last byte, `$F2`, whose low bits are %10, and its "always" is taken as said of that console, the page naming no
  model. On the Mark III without the unit, "the input from port 0" is read as port `$DC` with the I/O chip on; with it
  off, the Mark III floats its bus as the first Master System does (§9.3 of the build record, argued). Either reading
  of either document finds no FM unit: a read of `$FF` or `$F2` never matches bit 0 written, and port `$DC` with the pad
  idle reads 7 in its low three bits. Writes to `$F0`-`$F2` do nothing.
- Pinned by: `programs::the_fm_unit_s_ports_are_nothing_without_it` (each read `$FF` on the Master System II and `$F2` on
  the Mark III and the first Master System; no FM; the tone sounding after the writes).
- Implemented in: the stage 5 commit; the bus's decoding is stage 2's, unchanged.

### D-14. The BIOS: the Master System and the Master System II have one, the Mark III none
- Opened: 2026-10-19, at stage 6, by documents that disagree. MacDonald's `smstech` §1 gives the Mark III a BIOS slot
  ("y" in its table). Sega's *Software Reference Manual for the SEGA Mark III Console*, "Memory selection at power-on",
  describes a resident 8 KiB ROM that checks the "front card slot" first, so that "if both the card and cartridge are
  plugged in, the card will be enabled". SMS Power!'s "Port $3E": the Mark III and earlier "used a hardware method to
  activate only one slot: the cartridge provides a bridge ... thereby disabling the card", a precedence "reversed in
  BIOS-using systems"; its "Card Slot": on a Master System the card takes priority, "on the Mark III, the cartridge".
- Documents read: those (`Endou_OpenBoot.md` §0).
- Test program: none can tell, no console being run for the project (`Endou_Plan.md` §10 Q11).
- Referee: none.
- Conclusion: **argued.** The manual's boot gives the card priority, which SMS Power! gives the Master System and
  denies the Mark III, whose priority is wired; so the manual, though named for the Mark III, describes the Master
  System's boot, and the Mark III has no BIOS, as `Endou_Plan.md` §5.5 had it. MacDonald's "y" is taken as the slot's
  enable bit, which no BIOS answers on that console [A].
- Pinned by: `openboot::a_japanese_cartridge_starts_on_the_mark_iii_and_is_refused_on_an_export_console`,
  `an_empty_master_system_shows_the_open_program_s_words` (the Mark III starts with no BIOS).
- Implemented in: the stage 6 commit (`model::Model::has_bios`).

### D-15. Port $3F at the hand-over: $A5, both TH pins outputs at a high level
- Opened: 2026-10-19, at stage 6. Sega's Mark III manual, "How the trackball works": before the trackball is used,
  `LD A,10100101b` and `OUT (03FH),A` "should be executed ... NOTE: these instructions are executed in the power-on
  initialization." SMS Power!'s "Peripheral Ports": the port "defaults to an input-only state (ie. with all the low
  bits set) on startup."
- Documents read: those.
- Test program: none can tell without a console.
- Referee: none.
- Conclusion: **argued.** The manual's note reads as the system's initialisation doing what the paragraph asks, as its
  "The boot ROM executes the IM 1 instruction at power-on" does; SMS Power!'s default is the I/O chip's reset, before
  any program runs. Both hold: the chip resets to `$FF`, the boot writes `$A5`. A pad reads the same either way, TH
  being unread by it and high both ways on an export console; region checks and the Light Phaser's games write the port
  themselves. Stage 11, which reads TH, reopens the entry if a game needs otherwise.
- Pinned by: `openboot::the_open_boot_hands_a_cartridge_over_in_the_state_sega_s_manual_gives`.
- Implemented in: the stage 6 commit (`src/boot/rom.s`).

### D-16. The export check's sum: the size code's range less the whole 16-byte header, banks past 32 KiB read through slot 2
- Opened: 2026-10-19, at stage 6, by the document's sentence against the games. Sega's Mark III manual, "The cartridge
  header": "A 16-bit checksum for the ROM (low-hi order). $7FFA and $7FFB are not included in this checksum." SMS
  Power!'s "ROM Header": the size code "may be used by the BIOS to determine the range", `$D` and `$2` "buggy"; "Mappers":
  Sega's mapper "is assumed to be there (for games larger than 48KB) by the BIOS"; "Codemasters Header": "the normal
  BIOS checksum routine does not work due to the different mapper".
- Documents read: those.
- Test program: the library's 305 export headers measured, read in place (`Endou_OpenBoot.md` §1.6): the manual's
  reading gives no stated sum; the range less the whole header gives 300; read as the console reads it, `$0000`-`$7FFF`
  directly and each further bank paged into slot 2 by `$FFFF`, *Micro Machines* on Codemasters' board as well, its
  stated `$E352` being bank 0 read fourteen times through slot 2. Four images agree with no range tried; *Bank Panic*'s
  first 32 KiB give its sum though its size code says 256 KiB.
- Referee: none; the games are the evidence, each having been mastered to pass an export BIOS.
- Conclusion: **measured on the games** for the range and the paging; **argued** for the codes the documents call buggy
  (`$D`, `$2`, summed as their sizes say), for refusing an unlisted code, and for requiring region code `$4`. *Bank
  Panic* suggests a BIOS that once summed 32 KiB whatever the code said; it runs on the Mark III by default, its region
  code naming no market, and is unchanged.
- Pinned by: `openboot::a_codemasters_board_is_summed_as_the_console_reads_it`,
  `a_cartridge_past_32_kib_is_summed_through_slot_2_and_slot_2_is_put_back`,
  `a_header_inside_the_range_is_left_out_of_the_sum`, `the_export_check_refuses_in_words`;
  `media::tests::the_checksum_leaves_out_the_header_and_stops_at_the_coded_size`.
- Implemented in: the stage 6 commit (`src/boot/ram.s`, `media::checksum`).

### D-17. The audio control port with an FM unit: with the I/O chip off a read of `$F2` gives three bits of a line counter, zeros, and the two bits written; the Mark III's unit keeps no bit 1; FM and PSG sound as the bits say; `$F0`-`$F2` are decoded exactly
- Opened: 2026-10-20, at stage 9, by what the documents leave open of D-13's other half. SMS Power!'s "Audio Control
  Port": with FM hardware, a read gives "Counter (see below)" in bits 7-5, "Always 0" in 4-2 and the "Last value
  written, 0 by default" in 1-0; the counter, MacDonald on its forum, is 12 bits "clocked by the C-Sync input into the
  315-5297", bits 11, 7 and 3 on D7, D6 and D5; writes of `$00`-`$03` choose the PSG, the FM, neither or both on the
  Japanese Master System, while on the Mark III "Bit 1 ... is forced to 0", so the two never sound together. MacDonald's
  `smstech` §7: `$F0` the register latch, `$F1` the data, `$F2` bit 0 for detection, reads of `$C0`-`$FF` the I/O
  chip's unless it is disabled; "It isn't known exactly what data is returned from ports $C0-$FF if the YM2413 is
  present."
- Documents read: those; the forum thread the "Audio Control Port" page cites, "FM/PSG mix: testing the audio control
  port", through a summarising fetch: a Japanese Master System heard as the page says (00 PSG, 01 FM, 02 nothing, 03
  both) and a Mark III with the unit as 00 PSG, 01 FM, 02 PSG, 03 FM, the poster's correction of 2020-03-25 included; no
  poster measured the two's levels.
- Test program: `programs/fm.s` (stage 5) finds the unit on both consoles, its seven reads giving bit 0 back, and the
  tone sounding after `$F2` = 3 on the Japanese Master System and not on the Mark III.
- Referee: none; the references' FM units are emulations of the same documents.
- Conclusion: **argued** from the documents and the posters' tests. The counter is taken as one pulse a line, 228 Z80
  clocks, counted from power-on: C-Sync's equalising and serration pulses in the vertical interval are not counted, which
  only a console could show and no game is known to read. `$F0`, `$F1` and `$F2` alone are the unit's ports, the
  documents naming no mirror; a read of `$F0` or `$F1` with the I/O chip off is the undriven bus, as with no unit.
- Pinned by: `bus::tests::the_fm_unit_s_ports_answer_on_the_consoles_that_carry_it`, `sound::tests::port_f2_chooses_what_sounds`,
  `programs::the_fm_unit_is_found_on_the_consoles_that_carry_one`.
- Implemented in: the stage 9 commit.

### D-18. The key scale's codes: KSL 1, 2 and 3 are 1.5, 3 and 6 dB an octave
- Opened: 2026-10-20, at stage 9, by the black box against the manual. The *YM2413 Application Manual*, Table III-4,
  gives D7 D6 = 1 0 as 1.5 dB an octave and 0 1 as 3, and its text lists "1.5 dB/oct, 3 dB/oct, 6 dB/oct, and 0 dB/oct"
  [R Maxim's HTML, checked against the scan].
- Documents read: those.
- Test program: a carrier's sine at F-number `$1A0` with each KSL, in blocks 7 and 5, on Genesis Plus GX's `nuked`
  YM2413 (`cmpshow.py`): KSL 1 lowers it by 10 dB and 7 dB, KSL 2 by 20 and 14, KSL 3 by more (28.5 in block 5), which
  are Table III-5's 3 dB an octave halved, as given, and doubled.
- Referee: the black box above, an emulation; its agreement is evidence only so far as its rule came from a chip.
- Conclusion: **measured on the black box**: the code read as a number, 1 is 1.5 dB an octave, 2 is 3, 3 is 6. The
  table's bit order is taken as a transcription's slip, the text's order agreeing with the black box.
- Pinned by: `opll::tests::the_level_key_scale_codes_are_1_5_3_and_6_db_an_octave`.
- Implemented in: the stage 9 commit.

### D-19. The DAC's small signals: sign and magnitude, a negative zero four units below the idle level
- Opened: 2026-10-20, at stage 9. The manual gives a nine-bit DAC and no more [D (II-1), (VI-5)]. On Genesis Plus GX's
  `nuked` YM2413, a carrier's sine at VOL 0 to 15 measures 69.3, 66.3, 63.3, … 46.1 (VOL 8), 43.5, 40.9, 38.7, 36.3,
  34.6, 32.6, 32.1 dB [M `volsweep.py`]: 3 dB a step to VOL 8, then less, VOL 14 and 15 nearly equal; its MAME core gives
  3.0 dB a step throughout. Endou's twelve bits shifted right by three give 3 dB a step to VOL 10 and then lose level
  faster. A release on the black box ends at about −37 dB and then falls to nothing at level 124 (`Endou_Opll.md` §3.5).
- Documents read: the manual.
- Test program: the volume sweep above; the waveform at VOL 15 on the black box shows four levels and no zero.
- Referee: the referee of `Endou_Opll.md` §7 would give the sums a sample, which a resampler hides.
- Conclusion: **open.** Endou keeps the nine-bit truncation; quiet notes are up to 8 dB quieter than the black box's.
- Pinned by: —
- Implemented in: —
- *Measured on the referee (2026-10-20):* the chip's two output pins, cycle by cycle, through the header's
  `OPLL_Clock` [M `p5`]. In a channel's turn, one pin carries the twelve-bit output's top eight bits plus one, and the
  other carries the sign as +1 or −1. A negative output enters as its ones' complement. An idle turn is +1 on both
  pins. So against the idle level, an output o ≥ 0 gives o >> 4, and o < 0 gives −((!o >> 4) + 4). A negative zero
  (−1), which the half wave and an output past the exponent's range both give, sits four units below the idle level,
  and a silent slot (level 127) gives a positive zero.
  - **This is the box's anomaly.** A quiet sine's negative half never rises nearer zero than −4, so a quiet note is
    louder than a truncation gives. At VOL 15 the waveform has four levels and no zero, as the box showed.
  - Endou now gives each channel's turn by this rule (`opll::dac`), and 54 synthetic voices are exact with it.
- Conclusion: **measured on the referee.** The rule replaces the nine-bit truncation. `sound::FM_GAIN` became 6, so
  that a full sine (255 units) keeps D-20's level of 0.59.
- Pinned by: `opll::tests::the_dac_is_sign_and_magnitude_offset_by_one`.
- Implemented in: the stage 9 commit "the YM2413 held to the referee sample for sample".

### D-20. The FM unit's level against the PSG's: argued, the references disagreeing
- Opened: 2026-10-20, at stage 9, as `Endou_Native.md` §12.3 left it: "the YM2413 ... whose level against the PSG's is
  that stage's measurement".
- Documents read: the forum thread of D-17, whose posters heard both at `$F2` = 3 and measured no level.
- Test program: a carrier's sine at full level on one channel and tone 0 at full volume, both sounding (`level.py`):
  the FM sine's amplitude over the tone's fundamental is 1.53 on Genesis Plus GX (either YM2413 core, at its default
  pre-amplifiers, 150 for the PSG and 100 for the FM), 0.557 on Gearsystem and 0.531 on SMS Plus GX [M].
- Referee: none; no recording of a Japanese Master System's output was found (`Endou_References.md` §6e's search
  stands).
- Conclusion: **argued**: 0.59 (`sound::FM_GAIN` 3 in nine-bit units, 6 since D-19's DAC rule of 2026-10-20), near the references' median, Genesis Plus GX's two cores being one
  program's choice of pre-amplifier, adjustable by its player. A recording would settle it as the PSG's analogue stage
  waits on one.
- Pinned by: `sound::tests::port_f2_chooses_what_sounds` holds the paths; the level is the constant `FM_GAIN`.
- Implemented in: the stage 9 commit.

### D-21. The Game Gear's VDP in GG mode: M1 and M2 give the 224-line display as on the 315-5246, and the LCD's window stays where it is in the frame, display lines 40-183
- Opened: 2026-10-20, at the Game Gear's stage J1, by a sentence open to two readings. MacDonald §6: "If the GameGear is
  in GG mode, it's resolution is fixed and does not change when the extended resolutions are used. However, the
  differences in sprite and background processing are still used. (e.g. end of sprite list marker doesn't work,
  background starts at $0700 offsets, etc.)" Read one way, the chip keeps a 192-line display (its counters and frame
  interrupt) and only the tables change; read the other, the chip makes a 224-line display and the LCD, whose size is
  what is "fixed", shows 160 by 144 of it. Neither says where the window falls in a 224-line display.
- Documents read: MacDonald's `msvdp` §6, §7 and §14; SMS Power!'s "GG VDP" (the window "the central 160×144 pixel
  section"); Sega's Game Gear VDP manual, "Effective area and LCD display area" ("Line 24 and line 167 are equivalent to
  the top and bottom lines, respectively, on the LCD screen", for the 192-line display) and its name table figure (cells
  102-121 of row 3, columns 6-25); SMS Power!'s "Game Gear LCD" (a Master System cartridge's mode: the LCD shows the
  same 218 lines of the frame whichever height, D-22).
- Test program: `programs/gg.s` asks for 224 lines from its frame 30 with a name table that marks the old window, and
  keeps the V counter read in its frame interrupt. **The first reading was built first** (the display held at 192, the
  window at lines 24-167). On the four references that run the Game Gear as a 160 by 144 picture (Genesis Plus GX,
  Gearsystem, SMS Plus GX, PicoDrive) [M 2026-10-20]: every one read the V counter as `$E1` in its frame interrupt, a
  224-line display's, against `$C1` on Endou, and showed lines 40-183, about 6,000 pixels from Endou's picture each;
  BlastEm, which shows its whole frame, placed the window otherwise (4,640 pixels from the conclusion's picture).
- Referee: none run; SMS_MiSTer cannot be elaborated (`Endou_Native.md` §11.11).
- Conclusion: **the second reading, argued:** GG mode is no exception to the 315-5246's heights (224 lines, not 240, as
  MacDonald §14 has it for both modes); what is fixed is the LCD's window, which keeps its place in the frame, so that
  a 224-line display, starting 16 lines higher in the frame than a 192-line one [R MacDonald §14: the 192-line display
  has "a 16 line overscan area at the top and bottom"], shows it from display line 40. It is the same rule as D-22's,
  that the LCD samples a fixed part of the frame, and it agrees with the four references. The first reading is
  retired: it was a guess that the references contradicted, and nothing in a document preferred it. No game known to
  this page uses the 224-line display in GG mode, so the rule matters to test programs and to games not yet found.
- Pinned by: `beryl-sms-vdp`'s `the_game_gear_s_own_mode_takes_the_224_line_display_and_its_window_stays_in_the_frame`;
  `endou`'s `gg::tests::a_game_gear_program_shows_its_window_in_twelve_bits_and_moves_it_for_224_lines`.
- Implemented in: the J1 commits (`beryl_sms_vdp::gg_window_top`, `endou::gg::window`).

### D-22. The Game Gear's LCD in a Master System cartridge's mode: 218 lines of the frame onto 146 rows, three lines into two with weights in sixths, three columns into two by their colour elements
- Opened: 2026-10-20, at stage J1, by three documents that disagree on the vertical and are thin on the weights.
  MacDonald §14: "I don't know how the GG modifies the display to pack 224 lines of graphics into just 144 lines, but it
  seems like some rows are skipped"; horizontally "only the middle 240 pixels are displayed", two of each LCD pixel's
  three colour elements standing for an SMS dot. SMS Power!'s "Game Gear LCD" (modified 2022-09-06): a 160 by 146
  panel; the 192-line display "padded with 13 rows of border color at the top and 13 at the bottom to give 218 lines";
  the 224-line display less three rows at each end, 218 lines; "scaled down to fit the 146 lines of the display";
  vertically "not incrementing the Y value at every third row, additionally, a duplicate pixel is rendered at Y + 1 for
  each third pixel"; horizontally `[RGB]1 [RGB]2 [RGB]3 = [RG1B2] [R2GB3]`. SMS Power!'s "Pixel Aspect Ratio": the
  central 240 by 192 "vertically four source pixels are spread across three output pixels (with some blurring)",
  "shown on the central 160x144 pixels of the screen".
- Documents read: those three; and the source the LCD page cites, SMS Power!'s forum topic 9562, "Game Gear SMS mode
  video scaling" (read 2026-10-20; `Endou_References.md` §6g): Maxim's account of the columns (2007-01-09, after
  MacDonald); **benryves's measurements from photographs of a Game Gear** (2007-01-10): on the 224-line display lines
  0-2 and 221-223 "do not appear to have any influence", line 3 "appears as the top line of the LCD", three lines make
  two rows of the panel and a lit line lights one row brightly and its neighbours dimly or two rows alike, by its place
  in the three; his two diagrams of a single lit line's response, one for each height; his weights, "a guess", of 1/6,
  1/3, 1/3 and 1/6 for one row and 1/3 three times (192) or 1/6, 1/3, 1/3, 1/6 (224) for the other; and **enik's
  comparisons with photographs** (2016-05-15): benryves's 192-line weights "do not look right" against a photograph of
  Altered Beast, whose one-line hyphen is shown "using 2 pixel height", which the 224-line weights give, and the guess
  that "the scaling is done by the LCD circuit without knowing which screen mode is set inside the VDP chip".
- Test program: `programs/ggsms.s`, every eighth line and column lit on black in each phase of three, a blue border,
  192 lines then 224.
- Referee: none. The black-box references show a Master System cartridge on a Game Gear unscaled, 256 by 192 or 224
  (Gearsystem's and SMS Plus GX's Game Gear modes for Master System cartridges, measured 2026-10-20), so they grade the
  VDP's picture before the LCD, not the scaling.
- Conclusion: **argued, from the documents**, each part reversible in `endou::gg`:
  1. The columns: 8-247, each three into two as `R1 G1 B2` and `R2 G3 B3`. The three documents and Maxim agree.
  2. The lines: a fixed window of the frame, 218 lines, which is the 224-line display's lines 3-220 and the 192-line
     display's lines -13 to 204, its border's 13 at each end. This is the LCD page's account, and with one rule for
     both heights it gives exactly enik's weights for the 192-line display (its line 0 on rows 8 and 9 alike, the
     hyphen's two rows) and benryves's 224-line ones; benryves's own 192-line diagram, which places each line half a
     row from that, is the one account it overrides, as enik found against photographs. It is also D-21's rule.
  3. The weights: of each three lines from the window's first, the panel's even row is 1/6, 1/3, 1/3, 1/6 of the line
     before and the three, its odd row 1/6, 1/3, 1/3, 1/6 of the three and the line after: benryves's 224-line
     weights, which reproduce both diagrams' two levels, "bright" 1/3 and "dim" 1/6, and conserve each line's light
     (2/3 of a row each, three into two). The ratio of the levels is **unmeasured**: the photographs show two levels.
     The arithmetic is on the components' 8-bit levels, rounded to nearest.
  4. The rows: 146, the panel's (the LCD page), not 144 (the aspect ratio page): with 144, line 220 could not light a
     row, which benryves saw it do. One disagreement remains and is **open**: with 146 rows, line 221 is bright on the
     last row, and benryves saw "no influence" from it; with 145 it would not be, and the LCD page's count would be
     wrong by one. The border's eight rows at each end of a 192-line display, benryves's "top 8 and bottom 8", come
     out of 146.
  5. The border rows take register 7's backdrop as the frame's picture ends (`Vdp::border`); a game that changes the
     backdrop between the display and the border shows the later colour throughout: a **named loss**.
- Pinned by: `gg::tests::three_columns_become_two_by_their_colour_elements`,
  `a_224_line_display_s_lines_light_the_rows_benryves_measured`, `a_192_line_display_is_letterboxed_by_its_border`,
  `a_master_system_cartridge_is_shrunk_onto_the_panel_in_both_heights`.
- Implemented in: the J1 commit (`endou::gg::Shrink`).

### D-23. Which mode a Game Gear cartridge sets: GG mode unless its header's region code names the Master System
- Opened: 2026-10-20, at stage J1. GG mode is "enabled by the cartridge hardware" [R SMS Power!'s "GG VDP"], a line of
  the cartridge connector that an image does not carry; a Master System cartridge in an adapter does not set it. Sega's
  header gives region codes `$3` and `$4` to the Master System and `$5`, `$6` and `$7` to the Game Gear [R SMS Power!'s
  "ROM Header"].
- Documents read: those; MacDonald §5 ("only available in GG mode").
- Test program: none can show a pin. The tester's six Game Gear cartridges carry `$6` (five) and `$7` (one) [M]. Of
  the references, only SMS Plus GX ran a `.gg` image whose header carries `$4` as a Master System cartridge, 256 by
  192; the other four ran it in GG mode [M 2026-10-20].
- Referee: none.
- Conclusion: **argued:** a cartridge routed as `gg` is in GG mode unless its header's region code is `$3` or `$4`, and
  one with no header is in GG mode; a Master System cartridge (`sms`) on the Game Gear model is in the Master System's
  mode. A cartridge whose header and pin disagree would need a table of its own, which the stage does not keep; none
  is known to this page.
- Pinned by: `media::tests::a_game_gear_cartridge_chooses_its_mode_by_its_header`;
  `v1::tests::a_game_gear_cartridge_runs_on_the_game_gear_in_the_mode_its_header_chooses`.
- Implemented in: the J1 commit (`Media::gg_mode`).

### D-24. OPEN. The YM2413's slot order within a sample: what channels 4-9 and the drums do differently from channels 1-3
- Opened: 2026-10-20, at stage 9, by the referee. Endou's chip computes all nine channels at one instant of a sample.
  The chip computes them in turn over eighteen cycles. On channels 1-3, every synthetic voice is equal to the referee's
  in every sample. Elsewhere it is not.
- Documents read: the manual, which gives no slot order.
- Test program: `lead.py`, `eglead.py` and `rfit.py` on the referee [M 2026-10-20]. The same voices were played on
  every channel. For each slot, the offsets of its tick count, its LFOs and its key-on were searched.
  - **Channels 4-9:** a modulator's attack at RATE 44 or more differs in a note's first 30-60 samples: 62, 31 and 19
    samples of 8,000 at RATE 45, 49 and 53. No offset makes it equal.
  - **Channel 9:** a carrier's decay steps a sample early at some ticks, 6-11 samples of 8,000.
  - **The drums:** all five start 1-3 samples before Endou's after a write to `$0E`. The bass drum's patch could not be
    fitted until that start is modelled.
  - **The bench:** where the bench's sample starts and where it puts a write change which sample a channel answers in.
    `Endou_Opll.md` §7 records the setting under which every channel lags alike.
- Referee: Nuked-OPLL, run as a black box. Its slot order is part of what it models, and it can only be observed.
- Conclusion: **open.** Endou keeps one instant a sample. What differs is a few samples at the start of a fast attack,
  single samples at decay ticks, and the drums' first samples. The chip's slot order, observed cycle by cycle on the
  referee's pins, is the next measurement.
- Pinned by: —
- Implemented in: —
