# Diamond_Disputes — the disputes log

*Created 2026-10-11, at stage 1, with its first entry.* This is the log `Diamond_Plan.md` §1.4 defines, in the shape
`VenusRT_Plan.md` §1.4 gives and `Nephrite_Disputes.md` adapted for a core with no emulator to consult. An entry
records a rule the documents and the test programs leave open or in disagreement, the evidence taken, and the rule as
implemented. It is the record of which sources a dispute step read, so that the clean-room protocol of
`Diamond_Plan.md` §1.3 can be audited from outside. Disputes about the R3000A itself are kept in the R3000A crate's own
record (the plan's §9.1), since they belong to every core that uses the crate.

## The rules this log keeps

- **A dispute step is separate.** Its only product is an entry. It ends with a rule stated in hardware terms, and a
  writer's step implements it and cites the entry. Nothing read in a dispute step is typed into code in that step.
- **The order of recourse is fixed:** the documents, then a test program written or run for the purpose, then
  PSX_MiSTer run as a black box (on a device, or in simulation with only its README, licence, directory names, the
  simulation folder's scripts and its top level's ports read). **Its RTL is never read**, and no emulator's source is
  ever read, at any step (the plan's §1.3 and §1.5).
- **The reference emulators are witnesses, not evidence of the hardware.** What a reference core does is recorded
  where it helps locate a disagreement; it settles nothing.
- **An entry is never deleted.** A conclusion later overturned is struck through and followed by the new one, with the
  evidence that overturned it.

## Entry shape

```
### D-<n>. <component>: <the rule, one sentence, in hardware terms>
- Opened: <date>, by <what left it open or showed the disagreement>
- Documents read: <title, revision, section> for each
- Test program: <the program written or run, on what, and what it showed>
- Referee: <how PSX_MiSTer was run, and which of its non-RTL files were opened> | <not consulted>
- Witnesses: <what the reference cores do, as outputs only> | <none>
- Conclusion: <the rule as implemented, in prose>; argued | measured | open
- Depends on it: <what in the core or the framework changes with the answer>
- Pinned by: <the test that fails if the rule is changed>
- Implemented in: <the commit that cites this entry>
```

## Entries

### D-0. The GPU's video timing: which video clock an NTSC console's GPU runs on, and how many video clocks a line holds, which together set the NTSC frame rate

- **Opened:** 2026-10-10, at stage 0, by the plan's reading of psx-spx (`Diamond_Plan.md` §2.3); entered here on
  2026-10-11, at stage 1, once psx-spx was pinned and read again for it. The PAL figures share the question and are
  recorded beside the NTSC ones.
- **Documents read:** psx-spx, the maintained conversion, branch `stable` at `6d7d1bc` (Diamond_Plan.md §11 Q2):
  "GPU Timings" in `graphicsprocessingunitgpu.md`, both its maintained subsections ("Nominal Video Clock", "Vertical
  Video Timings", "Vertical Refresh Rates", added on 2020-09-24 by the conversion's history) and the "Nocash's original
  GPU Timings notes" kept below them; "Pinouts - CLK Pinouts" in `pinouts.md`; "PSX Dev-Board Chipsets" in
  `psxdevboardchipsets.md` for the oscillators on Sony's development boards. nocash's single-file edition
  (`problemkaputt.de/psx-spx.htm`, fetched the same day) was not compared.
- **The readings, as the documents give them:**
  1. *The maintained subsections* [R]. The nominal video clocks are 53.693175 MHz for NTSC and 53.203425 MHz for PAL;
     "Consoles will always use the video clock for its region, regardless of the GPU being configured in NTSC or PAL
     output mode". 263 lines a field non-interlaced and 262.5 interlaced for NTSC, 314 and 312.5 for PAL. Refresh
     rates: NTSC 59.826 Hz non-interlaced and 59.940 Hz interlaced; PAL 49.761 and 50.000 Hz; NTSC mode on a PAL clock
     59.280 and 59.393 Hz; PAL mode on an NTSC clock 50.219 and 50.460 Hz. It recommends an NTSC clock for NTSC content
     and leaves "Derivations for vertical refresh rates" as a to-do.
  2. *nocash's original notes* [R]. "The PSone/PAL video clock is the cpu clock multiplied by 11/7": 33.8688 MHz ×
     11/7 = 53.222400 MHz. 263 and 314 lines a frame; 3,413 video cycles a line for NTSC "(or 3413.6 or so?)" and
     3,406 for PAL "(or 3406.1 or so?)". Every dot clock is derived from 53.2224 MHz (5.32224 MHz for 256 pixels, and
     so on), and so are both frame rates: 53.2224 MHz / 263 / 3,413 ≈ 59.29 Hz for NTSC, 53.2224 MHz / 314 / 3,406 ≈
     49.76 Hz for PAL.
  3. *nocash's clock pinouts* [R]. The PSX/NTSC boards PU-7 and PU-8 carry a separate GPU oscillator, X201, 53.69 MHz
     ("div15 = NTSC color clock"), and the PAL boards PU-7, PU-8 and PU-18 a 53.20 MHz one; the CPU has its own
     67.737 MHz crystal on both. The PSone/PAL makes both clocks from one 17.734 MHz crystal: 53.202 MHz (× 3) and
     67.711636 MHz (× 3 × 2 × 7/11), both marked "(?)". The PSone/NTSC and the PU-20 use a 14.318 MHz crystal with a
     divider whose factors are "Unknown".
  4. *The development boards* [R]. The DTL-H2000 carries both an NTSC oscillator, 53.693175 MHz, and a PAL one,
     53.203425 MHz; the DTL-H201A a 53.693 MHz one.
- **What the readings imply** [A, arithmetic on the figures above]:
  - The maintained subsections' eight refresh rates are all reproduced, to their three decimals, by the nominal clocks
    over 3,412.5 video clocks a line for NTSC and 3,405 for PAL (263 × 3,412.5 at 53.693175 MHz is 59.8261 Hz; 262.5 ×
    3,412.5 is 59.9401 Hz; 314 × 3,405 at 53.203425 MHz is 49.7614 Hz; 312.5 × 3,405 is 50.0003 Hz; and the four
    cross-region figures likewise). Those line lengths are 227.5 and 283.75 subcarrier cycles, the broadcast
    standards' own lines. nocash's are 3,413 and 3,406. **The documents therefore disagree on the line length as well
    as on the clock**: the maintained text never states 3,412.5 or 3,405, but its rates require them.
  - The NTSC frame rate under each combination: 59.817 Hz (nominal clock, 3,413 a line), 59.826 Hz (nominal, 3,412.5),
    59.293 Hz (53.2224 MHz, 3,413). PAL: 49.747 Hz (nominal, 3,406), 49.761 Hz (nominal, 3,405), 49.765 Hz (53.2224 MHz,
    3,406). The PAL readings differ by 0.04% at most; the NTSC ones by 0.9%, which is the dispute.
  - nocash's 59.29 Hz contradicts his own pinouts for the PU-7 and PU-8, whose separate 53.69 MHz oscillator gives
    59.82 Hz whatever the line length. His statement of the 11/7 ratio is made for the PSone/PAL, where it would hold
    with the *CPU* clock off nominal (67.711636 / 2 = 33.856 MHz) if the "(?)" figures are right, and he then applies
    the ratio to NTSC as well. For the NTSC consoles that use one crystal (PU-20 and later, the PSone), the documents
    give no factors at all, so neither reading is established for them.
- **Test program:** none written yet. **Evidence found, not yet weighed:** ps1-tests' `timers` (build-158) ships a log
  from a console, `timers/psx.log`, which says "Detected NTSC system" and "Forced NTSC system" and then gives, among its
  rows, "System clock/8, frame delay" counts of 71,407–71,412 once settled [R the log]. Read as CPU cycles from one
  vertical blank to the next, that is 571,275 cycles a frame (± 16), or 59.286 Hz [A]. That is 0.01% from the 11/7
  reading (59.293 Hz) and 0.9% from both nominal-clock readings; it is also 0.01% from psx-spx's "NTSC mode on a PAL
  clock" (59.280 Hz), so the log alone cannot say whether the console was an NTSC one at about 11/7 or a PAL console
  forced into NTSC mode, and it names no console. Its "Hblank, frame delay" row reads 263 lines, and its "Dot clock,
  frame delay" row (112,028–112,034) is within ten of 263 lines of 426 dots (112,038), the 320-pixel count nocash's
  table gives, to which either line length rounds [A]. The step that weighs it reads `timers/main.c` (a test program's source, allowed by §1.3) to learn what
  "frame delay" counts and how the region is detected and forced.
  **The program that settles it**, at stage 3 or 4 (it needs the timers, the interrupts and the GPU's timing): on a
  console whose model and region are recorded, count CPU cycles (timer 2 at the system clock with its overflows) across
  many vertical blanks, timer 1's hblank count across a field, and timer 0's dot clock across a line, in each display
  mode, non-interlaced and interlaced, and report the totals on the TTY. The ratio of video to CPU clock, the lines a
  field and the line length follow directly. It is run on as many NTSC models as the tester can reach (PU-7/PU-8,
  PU-20 or later, a PSone; P11 predicts one answer for all) and on a PAL console for the PAL row; a console is the only
  referee, since PSX_MiSTer's timing is its author's reading of the same documents.
- **Referee:** not consulted.
- **Witnesses** [M, measured on 2026-10-11 through the probe]: PCSX-ReARMed r26 reports 59.817333 frames a second to
  its host when its fractional-rate option is enabled (60.000000 otherwise), which is 53.693175 MHz / (263 × 3,413):
  the nominal clock with nocash's line length. Beetle PSX and SwanStation were not run (`Diamond_References.md` §4).
- **Conclusion:** open. Until it is settled, nothing in Diamond depends on one reading without saying so.
- **Depends on it:**
  - the frame rate Diamond reports to the host, and the scheduler's second clock domain, which the plan's §5.1 relates
    to the CPU's by an exact rational (11/7 under nocash's reading, the crystals' ratio under the nominal one);
  - what timer 0 counts on the dot clock and timer 1 on the hblanks in a frame, which test programs and games read;
  - the CRT filter's PlayStation row (`Diamond_Plan.md` §4.6, `EmuSen_CRT.md` §3.2), which waits on this entry: with
    3,413 video clocks a line the NTSC subcarrier advances 8⁄15 of a cycle a line and 4⁄15 a frame, as §4.6 tabled it;
    with 3,412.5 it advances ½ a line and ½ a frame (263 lines), the broadcast standard's alternation; if the GPU runs
    at 53.2224 MHz while the encoder keeps a 3.579545 MHz subcarrier, a line of 3,413 clocks is 229.546 subcarrier
    cycles and the phases are no longer simple fractions. PAL: ⅚ a line at 3,406, ¾ at 3,405;
  - the audio is not affected: the SPU runs on the CPU's clock (768 × 44,100 Hz).
- **Pinned by:** —
- **Implemented in:** —
