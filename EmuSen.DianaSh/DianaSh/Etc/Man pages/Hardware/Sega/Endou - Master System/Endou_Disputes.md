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

### D-1. OPEN. The Master System's colour output: the level at which each value of a CRAM entry's two-bit component is shown
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
- Pinned by: —
- Implemented in: —

### D-2. OPEN. The YM2413's operator ROMs: whether its log-sine and exponent tables are the OPL2's, entry for entry
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
