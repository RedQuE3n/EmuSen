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
  of an SG-1000's noise, if one is found or the tester has the console, would settle the discrete side.
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
- Referee: SMS_MiSTer's output levels are its author's choice, so weak; a capture of a console's RGB output, if the
  tester has a console (`Endou_Plan.md` §10 Q11), or a published measurement found at the fetch, would settle it.
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
- Referee: Nuked-OPLL run as a black box, if the tester allows it (`Endou_Plan.md` §10 Q4); its agreement is evidence
  about a console only so far as its own tables came from a die. The SMS Power page the summary came from is a
  document, and is read at the fetch.
- Conclusion: **open.** P5 of `Endou_Plan.md` predicts equality. If the tables are the OPL2's, they move to a Beryl
  crate with their provenance page, Nephrite and Endou both reading them (`Endou_Plan.md` §3.2); if not, the OPLL's are
  Endou's own, with a provenance page of their own.
- Pinned by: —
- Implemented in: —
