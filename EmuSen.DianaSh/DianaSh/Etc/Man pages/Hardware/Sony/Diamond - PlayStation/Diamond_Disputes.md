# Diamond_Disputes — the disputes log

*Created 2026-10-11, at stage 1, with its first entry.* This is the log `Diamond_Plan.md` §1.4 defines, in the shape
`VenusRT_Plan.md` §1.4 gives and `Nephrite_Disputes.md` adapted for a core with no emulator to consult. An entry
records a rule the documents and the test programs leave open or in disagreement, the evidence taken, and the rule as
implemented. It is the record of which sources a dispute step read, so that the clean-room protocol of
`Diamond_Plan.md` §1.3 can be audited from outside. Disputes about the R3000A itself are kept in the R3000A crate's own
record (the plan's §9.1), since they belong to every core that uses the crate: D-1 to D-3 in `BlackMoon_R3000.md` §8.
*Since stage 5 (2026-10-11)* the GTE's are kept the same way, D-13 to D-16 in `BlackMoon_GTE.md` §12: the interlock
(D-13), the accumulators `test-all` settled (D-14), the write hazards (D-15) and the unlisted command numbers (D-16).
*The referee step of 2026-10-11* ran each open dispute's program on five reference emulators as black boxes; its runs
and their weighing are `Diamond_Referees.md`, and each entry below carries its ruling, D-2's in
`../BlackMoon-HW/BlackMoon_R3000.md` §8.
*Since the pad (2026-10-11)* the R3000A's D-17 to D-21, from amidog's `psxtest_cpx`, are in `BlackMoon_R3000.md` §8:
MTC0's interrupt latency (D-17), TAR on a branch not taken (D-18), CE (D-19), the debug unit (D-20) and SWCz's store
(D-21); D-22, SIO0's timing, is here.

## The rules this log keeps

- **A dispute step is separate.** Its only product is an entry. It ends with a rule stated in hardware terms, and a
  writer's step implements it and cites the entry. Nothing read in a dispute step is typed into code in that step.
- **The order of recourse is fixed:** the documents, then a test program written or run for the purpose, then
  ~~PSX_MiSTer run as a black box (on a device, or in simulation with only its README, licence, directory names, the
  simulation folder's scripts and its top level's ports read)~~ *decided 2026-10-11: no console is used for any
  dispute, now or later; the third rung is reference emulators run as black boxes, outputs only
  (`Diamond_Referees.md`), with PSX_MiSTer one of them if it is ever simulated*. **Its RTL is never read**, and no
  emulator's source is ever read, at any step (the plan's §1.3 and §1.5).
- **The reference emulators are witnesses, not evidence of the hardware.** ~~What a reference core does is recorded
  where it helps locate a disagreement; it settles nothing.~~ *Revised 2026-10-11:* where the console logs and
  pictures the suites ship do not reach, the referees rule, weighed by lineage and by how each scores against those
  logs (`Diamond_Referees.md` §3, §6): a ruling needs agreement between lineages, or names the strongest single
  witness as such; a majority alone rules nothing.
- **An entry is never deleted.** A conclusion later overturned is struck through and followed by the new one, with the
  evidence that overturned it.

## Entry shape

```
### D-<n>. <component>: <the rule, one sentence, in hardware terms>
- Opened: <date>, by <what left it open or showed the disagreement>
- Documents read: <title, revision, section> for each
- Test program: <the program written or run, on what, and what it showed>
- Referee: <which referees ran which program, and what each showed> | <not consulted>
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
- **The second step, 2026-10-11: the evidence weighed.** A dispute step of its own, which read the following on
  2026-10-11 and wrote no code. No emulator source, no PSX_MiSTer file and no firmware or firmware listing was opened,
  and no reference core was run.
  - *Every file opened.* Test programs: ps1-tests at `f727802`, its `README.md`, `timers/main.c`, `timers/Makefile`,
    `timers/psx.log` and the same log as first committed (`e13bd71`, 2019-09-22; the current one is from `f7cede5`,
    2020-09-10, and build-158's copy is byte-identical to it), the header of `timer-dump/psx.log`, the first lines of
    the other nineteen `psx.log` files, `gpu/bandwidth/main.cpp`'s `calculate()`, the `SetVideoMode` calls across the
    suite's sources (a search, lines only) and the repository's commit authors; PSn00bSDK `5d9aa2d`'s
    `examples/system/timer/main.c` and `examples/demos/n00bdemo/disp.c`, at their `GetVideoMode` calls only (the SDK's
    library was not opened); `tests/scanline-timing/scanline-timing.c` and its description and review thread in
    `pcsx-redux/nugget` pull request 51 (2026-09-28, MIT; a test program of PCSX-Redux's SDK repository, not the
    emulator); the PS1 240p test suite's `help.c` (`filipalac/240pTestSuite-PS1`, 2020-12-19), searched for rates, none
    given. Documents: psx-spx `stable` `6d7d1bc` "GPU Timings" and "Pinouts - CLK Pinouts" again, and the conversion's
    history for them: `9c9ef0c` (2020-09-24), which added the maintained subsections with no derivation or measurement
    cited, and `cafc0eb` (2026-09-28, pull request 137, merged into `master` as `5bd5d9b` and **not into the pinned
    `stable`**), which replaces 3,406/3,413 with 3,405/3,412.5; nocash's `psx-spx.htm` (as fetched 2026-10-11) at "GPU
    Timings", GP1(06h), "GPU Versions", "Pinouts - CLK Pinouts", "Mods - PAL/NTSC Color Mods" and the mainboard chip
    list's oscillator and GPU lines. Sony's service manuals (Sony documents, allowed by Q1, saved with their hashes under
    `docs/sony-service-manuals/`, `DOCS.txt` §C): **SCPH-5500 series** (PU-18), §2-1 block diagram; **SCPH-7500 series**
    (PU-22), §5-1 block diagram; **SCPH-9000 series, 3rd edition** (PU-23, 1999-05), §5 block diagrams, the IC204
    internal diagram, the check waveforms and the parts list's X201 and IC204; **SCPH-100 series, 5th edition** (PSone,
    PM-41, 2001-12), §4-1 block diagram and the parts list's X201 and IC204, in two scans. Also opened and of no use:
    the owner's manuals of the SCPH-7000, SCPH-7501 and SCPH-9001, a 2004 community repair guide
    (`Playstation_Service_manual.pdf` on psx.arthus.net, searched for clock figures: 53.203 MHz and 67.737 MHz on a PAL
    SCPH-1002), and a 2018 videogameperfection.com thread on 60 Hz PAL consoles, which gives no measured figure. Not
    read: consolemods.org's "PS1: Video Output Notes" and a psxdev.net timing thread, both refused (HTTP 403); the
    SCPH-1000/1001 service manuals, not found.
  - *What `timers/psx.log` measures* [R `main.c`]. `testTimerWithFrameDelay` writes a timer's mode (which zeroes it),
    waits for the vblank interrupt's callback to toggle a flag, reads the counter, adds 0xFFFF if the reached-FFFF bit
    is set, and repeats at once, eleven times, printing the last ten. Each figure is therefore one vblank-to-vblank
    period less `s`, the cycles from one iteration's counter read to the next one's mode write, with the
    interrupt's latency cancelling. The display is 320 × 240, so non-interlaced; `SetVideoMode(MODE_NTSC)` precedes every
    row ("Forced NTSC"), and the hblank row's 263 lines confirm NTSC mode [R the log]. "Detected NTSC" prints
    `GetVideoMode()` after `ResetGraph`; how the SDK sets it was not read, so it is **not evidence of the console's
    region**; every program in the suite forces NTSC the same way [R the search]. Neither log, nor the README, names the
    console; the commits carry a +0200 (central European) offset [R].
  - *The measured period* [A, arithmetic on the logs]. Dropping each row's first two printed values (still settling),
    the four CPU-clock rows agree: the system-clock rows unwrap as value + 458,753 (eight wraps, one counted as 0xFFFF)
    and the ÷8 row as (value + 1) × 8, giving a mean of **571,284.6** cycles (2020) and **571,297.0** (2019), each
    ±6. `s` is measured by the dot-clock row of the same function: psx-spx gives 426 timer-0 counts a line at 320
    pixels in NTSC mode (either line length), 263 × 426 = 112,038 a frame, and the logs read 112,030.6 and 112,032.5,
    so `s` is 7.4 and 5.5 dots, 37.7 and 28.0 CPU cycles (a dot is 8 video clocks, 5.09 CPU cycles). The period is
    then **571,322 (2020) and 571,325 (2019) CPU cycles, 571,324 ± 6**: the two builds differ in `s` and agree once it
    is added. The delay rows confirm the ratio independently: 5,011 CPU cycles hold 982.7 dots (320 mode); a 53.69 MHz
    video clock would give 991.7, a 53.20 MHz one 982.7.
  - *Sony's design figures* [R the four manuals]. **PU-18**: the GPU has its own "53MHz" crystal and the CPU its
    "67MHz" one, and the GPU hands FSC 3.58 MHz to the encoder. **PU-22**: one crystal, X201 "14.32MHz (7500, 7501,
    7503), 17.73MHz (7502)", feeds IC204 "CLK GEN", which gives the GPU "NTSC:53.69MHz / PAL:53.20MHz", the CPU
    67.73 MHz (33.86 MHz out of the CPU) and the encoder FSC directly. **PU-23**: the same, "NTSC : 53.693175MHz, PAL :
    53.203425MHz" to the GPU, 67.7376 MHz to the CPU, 33.8688 MHz on to the audio DAC; IC204 is a CY2081SL-500T
    (NTSC) or -509T (PAL), drawn as a reference oscillator, three PLLs and an EPROM-configured divider, with the CPU
    and GPU clocks on separate outputs; its pin 1 is labelled "53.63MHz (NTSC)" in that drawing alone, which the same
    manual's block diagram (53.693175) and its GPU check waveform ("NTSC:53.69MHz", IC203 pin 192) contradict, and
    53.63 is not 15 × fsc, so it is read as a misprint. **PM-41 (PSone)**: X'TAL 14.318182 MHz (NTSC) or 17.73447 MHz
    (PAL), IC204 PLL, GPU 53.693175 or 53.203425 MHz, CPU 67.7376 MHz. With nocash's PU-7/PU-8 X201 of 53.69 MHz and
    his "Mods" page (PU-7, PU-8, PU-16, PU-18 and PU-20 carry 53.20 or 53.69 MHz oscillators; PU-22 and later
    17.734 or 14.318 MHz crystals with constant dividers), **every NTSC board documented feeds its GPU 53.693175 MHz
    nominal, and every PAL one 53.203425 MHz**. On the one-crystal boards both are 15 × and 12 × the encoder's own
    subcarrier, and on the older ones the GPU derives the subcarrier by ÷15 or ÷12, so the subcarrier is exactly the
    video clock / 15 (NTSC) or / 12 (PAL) on every board [A]. nocash's pages disagree among themselves about PU-20
    (one crystal, says "CLK Pinouts"; separate oscillators, say "Mods" and the chip list); either way it is 53.69 MHz
    for NTSC.
  - *Where 11/7 comes from* [A]. nocash's figures for the PSone/PAL, 17.734 MHz × 3 for the GPU and × 3 × 2 × 7/11
    for the CPU, both marked "(?)", put the GPU at exactly 11/7 of the CPU there; with Sony's 67.7376 MHz the PAL ratio
    is 53.203425 / 33.8688 = 1.570868, 0.036% from 11/7. His 53.2224 MHz is that ratio applied to a CPU of 33.8688 MHz,
    and his NTSC frame rate applies it to NTSC as well, which no Sony figure supports.
  - *A public measurement* [R `cafc0eb` and pull request 137; the program R]. The `master` text reports a PAL-mode to
    NTSC-mode line ratio of 0.997802 on one console at a time, on an SCPH-1000, an SCPH-5501 and an SCPH-9002, which is
    3,405/3,412.5 (3,406/3,413 is 0.997949, 147 ppm away), and CPU cycles per line of 2,152.5 (NTSC console, NTSC
    mode), 2,147.8 (NTSC console, PAL mode), 2,167.5 and 2,172.3 (PAL console, PAL and NTSC mode), "to within crystal
    tolerance" of the nominal clocks; also that one SCPH-1001 stops all video timing in PAL mode. The program reads
    timer 2 on the system clock, timer 1 on hblank and timer 0 on the dot clock, unwrapped by polling, across 40
    vblank edges seen in I_STAT with interrupts masked, two windows per GP1(08h) mode, which measures what the report
    says. The consoles' raw outputs are not published, and the text is outside the pinned revision.
  - *The arithmetic* [A], CPU at 33.8688 MHz, the frame rate and the CPU cycles a field:

    | Video clock | Line (video clocks) | NTSC mode, 263 lines | NTSC, 262.5 (interlaced) | PAL mode, 314 lines | PAL, 312.5 (interlaced) |
    |---|---|---|---|---|---|
    | 53.693175 MHz (NTSC board) | 3,412.5 / 3,405 | 59.8261 Hz, 566,121 | 59.9401 Hz, 565,045 | 50.2195 Hz, 674,415 | 50.4605 Hz, 671,194 |
    | 53.693175 MHz | 3,413 / 3,406 | 59.8173 Hz, 566,204 | 59.9313 Hz, 565,127 | 50.2047 Hz, 674,614 | 50.4457 Hz, 671,391 |
    | 53.203425 MHz (PAL board) | 3,412.5 / 3,405 | 59.2804 Hz, 571,332 | 59.3933 Hz, 570,246 | 49.7614 Hz, 680,624 | 50.0003 Hz, 677,372 |
    | 53.203425 MHz | 3,413 / 3,406 | 59.2717 Hz, 571,416 | 59.3846 Hz, 570,330 | 49.7468 Hz, 680,824 | 49.9856 Hz, 677,571 |
    | 53.2224 MHz (CPU × 11/7) | 3,413 / 3,406 | 59.2929 Hz, 571,212 | 59.4058 Hz, 570,126 | 49.7646 Hz, 680,581 | 50.0034 Hz, 677,330 |

    The measured 571,324 ± 6 against each: an NTSC board, by either line, is 0.9% (5,100 cycles) short, **ruled out**.
    A PAL board's nominal clocks give a line of 3,412.45 video clocks: 3,412.5 is 14 ppm off, inside the tolerance of
    two separate crystals or a PLL's approximation, and 3,413 is 161 ppm off. An exact 11/7 gives 3,413.67, neither
    length; 3,413 under 11/7 would need `s` to be −73 cycles (−85 in 2019), which cannot be. The 2026-09-28 report's 2,172.3 cycles a
    line × 263 is 571,315, 9 cycles (16 ppm) from this log, on a PAL console, measured by a different program.
  - *What each piece supports* [A]. The `timers` log was taken on a **PAL-clocked console forced into NTSC mode**: it
    fits only a 53.20 MHz video clock (or nocash's 11/7 with a line no document gives), and Sony's manuals leave no
    NTSC board with a 53.2 MHz GPU clock. It is evidence for nocash's 59.29 Hz only as the rate of NTSC mode on a PAL
    console, which psx-spx's maintained table already lists (59.280 Hz). It favours 3,412.5 over 3,413 by its own
    arithmetic if the console's CPU clock is Sony's 67.7376 MHz / 2, and favours neither if it is nocash's PSone/PAL
    "(?)" figure. The `master` report favours 3,412.5 directly, by a ratio no crystal affects. The `stable` table's
    rates imply 3,412.5 but cite nothing. PCSX-ReARMed's 59.817333 agrees on the clock and uses 3,413 (a witness).
- **Conclusion:** ~~open. Until it is settled, nothing in Diamond depends on one reading without saying so.~~
  *Revised 2026-10-11 by the second step:* **the clock is settled, argued from Sony's design documents and consistent
  with every measurement read; the line length is provisional.**
  - **Clock, argued.** An NTSC console's GPU runs on **53.693175 MHz** on every board family, the separate-oscillator
    boards (PU-7, PU-8, PU-18, and PU-20 whichever way it is built) and the one-crystal ones (PU-22, PU-23, PM-41)
    alike; a PAL console's on **53.203425 MHz**; each console keeps its own clock in the other region's mode. nocash's
    53.2224 MHz (CPU × 11/7) is not an NTSC console's clock: at most it is a PAL PSone's, to 0.036%. Diamond's default,
    the 208-pin GPU (late PU-8 to PM-41), therefore runs NTSC at 53.693175 MHz whatever board it stands for, and the
    plan's §5.1 relation is the crystals' ratio, 53,693,175 / 33,868,800 = 715,909 / 451,584 (NTSC) and
    709,379 / 451,584 (PAL), not 11/7. (15 × 315/88 MHz is 53.693182 MHz, 0.13 ppm higher; Sony and psx-spx both print
    53.693175, and that is the figure taken.) P11 is consistent with this, and stays unretired until a console is
    measured.
  - **Line length, provisional: 3,412.5 video clocks a line on average in NTSC mode, 3,405 in PAL**, so the default
    NTSC console runs at **59.8261 Hz non-interlaced and 59.9401 Hz interlaced**, 566,121 and 565,045 CPU cycles a
    field, 2,152.55 a line; PAL 49.7614 and 50.0003 Hz. Against 3,413/3,406 this is 147 ppm (59.8173 Hz), and three
    readings favour it: the `master` report's crystal-free ratio, this log's arithmetic under Sony's CPU figure, and
    the `stable` table. None is a raw measurement the project holds on an identified console, so this part is not
    closed. Also open: how a half clock is spread over lines (alternate lines of 3,412 and 3,413, or otherwise), and
    262.5 lines a field interlaced, which no log read measures.
  - **The program that closes it**, at stage 3 or 4: the measurement above, on the tester's console with its model
    and board recorded (Q16). Per GP1(08h) mode, with interrupts masked and I_STAT polled, count CPU cycles (timer 2,
    system clock, unwrapped), hblanks (timer 1) and dots (timer 0) across 40 or more vblank edges, two windows per mode,
    and print the totals. It decides three things: the CPU cycles a field in NTSC mode (566,121 or 566,204 on an NTSC
    console; 571,2xx–571,4xx would mean a PAL clock), the **PAL-to-NTSC line ratio on the one console** (0.997802 for
    3,405/3,412.5, 0.997949 for 3,406/3,413; 147 ppm apart, which 40 fields resolve to about 1 ppm whatever the
    crystals), and the interlaced field (565,045 against 566,121 cycles). A fourth window, timestamping each hblank with
    timer 2 and averaging odd lines against even ones over many fields, shows whether lines alternate (a line of
    3,412 against 3,413 video clocks is 2,152.2 against 2,152.9 CPU cycles). The same program run on Diamond's timers
    against its known clock checks the implementation, not the hardware; the console decides. An NTSC console and a
    PAL one between them give every row.
- **Depends on it:**
  - the frame rate Diamond reports to the host, and the scheduler's second clock domain, which the plan's §5.1 relates
    to the CPU's by an exact rational (11/7 under nocash's reading, the crystals' ratio under the nominal one);
    *since the second step*: the crystals' ratio, with the line length provisional;
  - what timer 0 counts on the dot clock and timer 1 on the hblanks in a frame, which test programs and games read;
    *since the second step*: 263 hblanks a frame non-interlaced in NTSC mode and 426 dots a line at 320 pixels, both
    seen in the log, and 2,152.55 CPU cycles a line (2,152.87 at 3,413);
  - the CRT filter's PlayStation row (`Diamond_Plan.md` §4.6, `EmuSen_CRT.md` §3.2), which waits on this entry: with
    3,413 video clocks a line the NTSC subcarrier advances 8⁄15 of a cycle a line and 4⁄15 a frame, as §4.6 tabled it;
    with 3,412.5 it advances ½ a line and ½ a frame (263 lines), the broadcast standard's alternation; if the GPU runs
    at 53.2224 MHz while the encoder keeps a 3.579545 MHz subcarrier, a line of 3,413 clocks is 229.546 subcarrier
    cycles and the phases are no longer simple fractions. PAL: ⅚ a line at 3,406, ¾ at 3,405;
    *since the second step*: the 53.2224 MHz case falls away and the video clock is exactly 15 × (NTSC) or 12 × (PAL)
    the subcarrier on every board, so §4.6's video-clock row, the active picture (2,560 clocks, 47.68 µs NTSC, 48.12 µs
    PAL) and the cycles across it (170⅔, 213⅓) stand; only the phases wait on the line length, ½ a line and ½ a frame
    at 3,412.5 (an average: alternate lines of 3,412 and 3,413 would advance 7⁄15 and 8⁄15), ¾ a line and ½ a frame
    at 3,405 for PAL. `EmuSen_CRT.md` §20.6's row waited on the clock; whether it is written now with the provisional
    line or after the console's measurement is the tester's decision;
  - the audio is not affected: the SPU runs on the CPU's clock (768 × 44,100 Hz).
- **Pinned by:** `the_rates_are_d0s_provisional_ones` (`video` and `machine`, to four places), DiamondTests'
  `A_machine_shows_the_packs_screen_and_runs_as_its_market`, and ps1-tests' `timers` graded against its log on a
  PAL-clocked Diamond (`Diamond_Native.md` §11.12).
- **Implemented in:** `diamond: the bus and the machine of stage 3` (2026-10-11): the crystals' ratios, the lines of
  3,412.5 and 3,405 video clocks, 263 and 314 lines a field, interlaced fields of 263 and 262 (314 and 313) alternately
  [C]. *Stage 3's measurement, 2026-10-11* [M, `Diamond_Native.md` §11.5]: with the PAL clock and NTSC mode, Diamond
  runs the log's program to 263 hblanks a frame, 982–983 dots in 5,000 cycles and 571,303 cycles a frame unwrapped,
  against the log's 263, 982–983 and 571,285 ± 6; the program's own overhead differs by tens of cycles. This checks the
  implementation against the second step's arithmetic, not the hardware: the line remains provisional until the
  console's run of the closing program.
- **The referee step, 2026-10-11** (`Diamond_Referees.md` §4.1, `programs/video-timing.s`) [M]. The five referees
  divide as the documents do: Beetle PSX 3,412.5 and 3,405 video clocks a line, DuckStation and SwanStation (one
  lineage) 3,413 and 3,406, PCSX-Redux and PCSX-ReARMed no line at all (60 Hz flat). The `timers` log decides between
  them: as PAL consoles in NTSC mode, Beetle PSX times 571,334 cycles a frame and DuckStation 571,416 against the
  log's 571,324 ± 6, and DuckStation's dot clock there runs at the NTSC rate, 167 dots a frame more than the log's.
  Beetle PSX, the strongest witness, agrees with Diamond on both lines and on NTSC's interlaced fields of 263 and 262;
  **it does not on PAL's**, which it alternates 313 and 312 (312.5, psx-spx's figure, 50.0001 Hz) where Diamond
  alternated 314 and 313 (313.5, 49.84 Hz, against this entry's own 50.0003 Hz). ~~The program that closes it ... on
  the tester's console~~: no console is used (decided 2026-10-11).
- **Conclusion, the referee step:** the line is **3,412.5 video clocks in NTSC mode and 3,405 in PAL**, now witnessed
  by Beetle PSX and consistent with the one console log, not only argued; **changed**: an interlaced PAL field holds
  313 and 312 lines alternately. Still open: how the half clock is spread over lines, which no referee output shows.
- **Pinned by, since the referee step:** `interlaced_fields_alternate_about_262_5_and_312_5_lines`,
  `the_video_timing_is_the_referees_line_on_both_clocks`.
- **Implemented in, since the referee step:** `diamond: an interlaced PAL field holds 313 and 312 lines (D-0)`.

### D-4. Memory control's access times: a first bus cycle of the read delay plus 4 and each further one of the delay plus 2, recovery and floating, not psx-spx's formula

- **Opened:** 2026-10-11, at stage 3, by the console's `cpu/access-time` log disagreeing with psx-spx's "COM_DELAY"
  formula for five regions of six.
- **Documents read:** psx-spx `6d7d1bc`, "Memory Control" (the delay/size registers' bits, COM_DELAY, the formula given
  "somewhat like so", and "the total access time is fairly complex overall, and difficult to properly describe"); "I/O
  Map" for the kernel's usual register values.
- **Test program:** ps1-tests' `cpu/access-time` (its source read for what it measures: an 8-, 16- and 32-bit load in a
  cached loop, its cycles over a loop without it, by timer 2, as cycles a load in hundredths) and its `psx.log` from a
  console. With the usual values (DEV2 and DEV0 `0013243Fh`, DEV1 `00003022h`, DEV4 `200931E1h`, DEV5 `00020843h`,
  DEV8 `00070777h`, COM `00031125h`), the console's whole cycles a load, the load's own cycle included, and the
  formula's totals:

  | Region | Console: first, further | Formula: first, further |
  |---|---|---|
  | BIOS, DEV0 (read delay 3, floating) | 7, 6 | 8, 7 |
  | DEV8 (7, recovery, hold, floating) | 11, 15 | 16, 15 |
  | DEV1 (2, 16-bit, auto-increment) | 6, 4 | 6, 5 |
  | CD-ROM (4, pre-strobe) | 8, 6 | 9, 7 |
  | SPU (14, recovery, 16-bit) | 18, 21 | 22, 21 |

- **The argument** [A]: one rule in the register's terms fits every row: the first bus cycle takes the read delay plus
  4, and each further one the read delay plus 2 plus COM0 when the recovery bit is set plus COM2 when the floating bit
  is. Recovery, "between two operations", does not lengthen the first cycle; the pre-strobe's MIN does not reach it at
  COM3 = 3; the floating period's contribution to the first cycle cannot be told from a constant at COM2 = 1, the only
  value the log holds, and the simpler reading is taken. Write timings follow the same rule with the write delay [C,
  unmeasured]. The rule's register values are the kernel's; a program that changes them gets the rule's figures, which
  no console log checks.
- **A named disagreement**: SPUCNT's 32-bit row, 38.94 cycles on the console and 36.07 on Diamond, reads a word at
  `1F801DAAh`, which is not word-aligned: the load raises an address error, so the row times an exception's entry and
  return through the kernel's handler. Which part of that path costs the 2.87 cycles is open; a program of the
  project's own timing an exception round trip with its own handler, on a console, would say.
- **Referee:** not consulted. **Witnesses:** not run for this table (their TTY cannot be read, `Diamond_Native.md`
  §11.15). *Referee step, 2026-10-11* (`Diamond_Referees.md` §4.3) [M]: DuckStation and PCSX-Redux, whose TTY can be
  read, meet 11 of the 51 cells each within 0.2 cycles; the SPUCNT cell reads 82.4 and 6.0 on them, 38.94 on the
  console. Neither models memory control as the console times it, so neither bears on the rule.
- **Conclusion:** the rule above, with RAM at a 4-cycle stall, the internal registers at 2, the scratchpad and BCC at
  none (the same log). Measured: 50 of 51 cells within 0.20 cycles of the console's; the SPUCNT cell named. Open for
  the SPUCNT cell and for any register value other than the kernel's; the referees leave both open.
- **Depends on it:** every load's and uncached fetch's cost, so the BIOS's own speed and every timing measurement.
- **Pinned by:** `the_kernel_values_give_the_consoles_access_times`, `a_load_costs_its_regions_time`, and ps1-tests'
  `cpu/access-time` graded against its log (`Diamond_Native.md` §11.12).
- **Implemented in:** `diamond: the bus and the machine of stage 3`.

### D-5. The DMA's pieces and the CPU's stores on the bus: the gaps between blocks, nodes and chopped windows, and a RAM store's drain

- **Opened:** 2026-10-11, at stage 3: psx-spx gives the DMA's words per cycle but neither the time between blocks,
  list nodes and chopped windows nor the write queue's drain, and the plan predicted a dispute here (P4).
- **Documents read:** psx-spx `6d7d1bc`, "DMA Channels" (the transfer rates; "CPU Operation during DMA"; chopping's
  windows) and "Memory Map" ("Write queue").
- **Test programs:** ps1-tests' `dma/chopping` and `dma/chain-looping` (sources read for what they time; their console
  logs). `chopping` times 2,048 words to the GPU in mode 1 at block sizes 1 to 64 and 128, in mode 0, and chopped at
  every pair of window sizes; `chain-looping` times a RAM loop of 1,000 iterations with no DMA, with an empty chain,
  and with one- and two-node chains that never end.
- **What the logs say** [A, arithmetic]: mode 1 costs about 10 cycles a block beyond its words at 17/16 a word; a
  chopped burst about 6 cycles a window beyond its words and its CPU window, the CPU window reaching no further than 32
  cycles a word of the DMA window (rows with CPU windows of 32, 64 and 128 cycles for one-word DMA windows all take
  about 40 cycles a word); the RAM loop 16 cycles an iteration, and 1.598 times that under a never-ending chain.
- **Referee:** *2026-10-11* (`Diamond_Referees.md` §4.4, `programs/stores.s`) [M]: within 10% of the console on
  `chopping`'s 130 rows, DuckStation 17 and PCSX-Redux 0; on `chain-looping`, DuckStation 13,016 cycles with no DMA
  and 143,024 under a never-ending chain, PCSX-Redux 16,016 throughout with every chain finished; on `stores.s`, no
  referee has a write queue a load waits for (DuckStation N cycles for N stores, Beetle PSX N + 1, PCSX-Redux 3N).
  **Witnesses:** none run before the referee step.
- **Conclusion** [C, fitted]: 10 cycles between blocks and between nodes, 4 of a node's own, a chopped window's CPU
  window held to 32 cycles a word of the DMA's window plus 6; a RAM store drains in 8 cycles, and a load waits for the
  whole queue. Measured: `chopping`'s 130 rows within 10.5% of the console's, 129 within 10%; `chain-looping`'s within
  6.2%. **Open**: these are fits to two programs, not rules the documents give, and the drain's 8 cycles cannot be
  divided between the store and the load that follows it from one loop. ~~A program of the project's own that times
  stores in bursts of one to eight, a load of the stored and of another address after each, and a chain's slowdown
  of a cached loop against an uncached one, run on a console, separates them.~~ *Referee step, 2026-10-11:* that
  program (`stores.s`, without the cached-loop part) ran on every referee, and none models what it measures; still
  open, on the two console logs alone.
- **Depends on it:** every program that loads after storing, and the CPU's pace beside DMA.
- **Pinned by:** ps1-tests' `dma/chain-looping` graded against its log (10%); `a_gpu_list_runs_node_by_node_and_flags_its_end`.
- **Implemented in:** `diamond: the bus and the machine of stage 3`.

### D-6. Timer sync mode 2 reads zero outside the blank, where psx-spx says it pauses

- **Opened:** 2026-10-11, at stage 3, by the console's `timers` log.
- **Documents read:** psx-spx `6d7d1bc`, "Timers": mode 2, "Reset counter to 0000h at Hblank(s) and pause outside of
  Hblank".
- **Test program:** ps1-tests' `timers` (its source read in D-0's second step) and its console log: in every
  resolution, the mode-2 rows with the system clock read 0 between hblanks and count only within them (36, 159, 282,
  405, 528, then 0 at 320 pixels); a counter that paused outside the blank would hold the count it reached, about 543
  cycles' worth.
- **Referee:** *2026-10-11* (`Diamond_Referees.md` §4.5) [M]: on `timers`, DuckStation counts freely in mode 2 and
  PCSX-Redux reads zero always, within the blank too; neither models it as the console behaves, and the console log
  stays the evidence. **Witnesses:** none.
- **Conclusion:** the counter is reset at the blank's end as well as its start, so it counts within the blank and
  reads zero outside; measured on the console's log, argued for timer 1's vblank by analogy [A]. A second reading of the
  same log: **the reset's two-cycle hold is of the system clock only** [A]; applied in ticks of a slower source it
  swallowed one hblank in each frame's count (262 against the log's 263).
- **Depends on it:** programs that measure the blank with mode 2.
- **Pinned by:** ps1-tests' `timers` graded against its log (the hblank row exactly);
  `system_clock_over_8_and_a_stopped_timer_2`.
- **Implemented in:** `diamond: the bus and the machine of stage 3`.

### D-7. The rasteriser where psx-spx is silent: whole-pixel samples with the top-left rule, 12-bit gradients from vertex 0 truncated toward zero with a half added, and lines' ties by their major axis

- **Opened:** 2026-10-11, at stage 4: psx-spx gives the vertices' range, the size limit, polygons "up to <excluding>
  their lower-right coordinates" and lines "up to <including>", the quad's split, and that interpolation is affine; it
  gives no fill rule, no precision for the interpolators, no starting point and no line stepping.
- **Documents read:** psx-spx `6d7d1bc`, "GPU Render Polygon Commands", "GPU Render Line Commands", "GPU Rendering
  Attributes", "Perspective (in-)correct Rendering", "24bit RGB to 15bit RGB Dithering".
- **Test programs:** ps1-tests' `triangle`, `uv-interpolation`, `lines`, `quad` and `clipping`, their sources read for
  their vertices and colours, and their pictures, a console's VRAM. `uv-interpolation` draws 256 quads one line high and
  0 to 255 pixels wide, U stepping from 0 to 1 over a red and a green texel, and 256 Gouraud quads red to green, plain
  and dithered; `triangle` three Gouraud triangles; `lines` lines at every slope of a set, flat and shaded.
- **The argument** [M, by search]: of every pairing of 8 to 23 fractional bits with a bias in 1/4096 steps, one puts each
  of `uv-interpolation`'s 255 switches from red to green where the console does: 12 bits, the gradient truncated, a half
  added. Truncation toward zero, not flooring, is what the Gouraud rows need (0 pixels off against 158). Evaluating from
  vertex 0 gives `triangle` exactly and from the topmost vertex 3,227 pixels off. Sampling at the pixel's integer
  coordinates gives the apex row empty and one pixel on the next, as the picture shows; at the pixel's centre the next
  row would hold two. On lines, x-major ties step at the half and y-major ones do not: with the x-major rule for both,
  25 pixels are off, every one a y-major tie.
- **Referee:** not consulted. **Witnesses:** PCSX-ReARMed and SwanStation differ from Diamond by one five-bit level
  on some Gouraud pixels of Peter Lemon's shaded programs (5 to 232 a picture, `Diamond_GPU.md` §11), in his shaded
  quads and lines; on his shaded triangle, starting from vertex 0, from the leftmost vertex or from the topmost gives
  the same pixels.
- **Conclusion:** `Diamond_GPU.md` §6; measured against the console's pictures, exact on all five programs. Open
  where the pictures do not reach. No picture tells the starting vertex apart: where vertex 0 is not the leftmost (the
  second triangles of `texture-flip`'s quads) the gradients are whole numbers, and no shaded quad's second triangle
  covers a pixel (`uv-interpolation`'s quads are one line high). No gradient is near 12 bits' range. ~~A program
  drawing shaded triangles and quads at every vertex order, run on a console, decides the first.~~
- **The referee step, 2026-10-11** (`Diamond_Referees.md` §4.6, `programs/raster-order.s`) [M]: that program, a shaded
  triangle in its six orders, a shaded quad in four, and a triangle with two vertices in its leftmost column in six,
  ran on every referee. DuckStation, Beetle PSX, SwanStation and PCSX-ReARMed, three renderers of separate authorship
  by their READMEs, draw all sixteen shapes word for word alike, the first triangle and the quad the same in every
  order; Diamond, starting from vertex 0, differed by 2 to 4 pixels on seven shapes. Their rule is the **leftmost
  vertex, and of two in the leftmost column the one that follows the other in the command's order**: of the bases
  tried on Diamond, it alone gives all sixteen exactly, and ps1-tests' pictures stay exact with it, where the topmost
  start leaves `triangle` 3,227 pixels off. **Changed**, on agreement between lineages: the attributes start from that
  vertex. Still open: no gradient near 12 bits' range.
- **Depends on it:** every pixel of every polygon and line.
- **Pinned by:** `Ps1_tests_gpu_programs_draw_their_vram_pictures_exactly`; `a_quads_texture_steps_at_the_half_of_a_truncated_12_bit_gradient`,
  `shading_takes_vertex_0s_value_with_a_half_added` (*renamed* `shading_takes_the_base_vertexs_value_with_a_half_added`
  *at the referee step*), `a_triangle_owns_its_top_and_left_edges_only`,
  `a_line_draws_both_ends_and_rounds_a_half_by_its_major_axis`; *since the referee step*
  `the_base_is_the_leftmost_vertex_and_of_two_the_follower`, `a_shaded_triangle_reads_the_same_in_every_vertex_order`,
  `shaded_triangles_draw_in_every_vertex_order_as_the_referees_do`.
- **Implemented in:** `diamond: the GPU of stage 4, and VRAM graded against ps1-tests' pictures`; *the base vertex:*
  `diamond: a triangle's attributes start from its leftmost vertex (D-7), an x-flip from U with its low bit set (D-8)`.

### D-8. An x-flipped rectangle's texture starts at U+1, where psx-spx says only that it decrements

- **Opened:** 2026-10-11, at stage 4, by `texture-flip`'s picture.
- **Documents read:** psx-spx `6d7d1bc`, "Texture Origin and X/Y-Flip": the flip bits "cause the texture coordinates
  to be decremented (instead of incremented)".
- **Test program:** ps1-tests' `texture-flip` (source and picture): a 256x256 texture of each texel's own index, drawn
  as rectangles normal, x-flipped, y-flipped and both, at 256x256 and 64x64, and as quads, with U and V of 0. In every
  x-flipped rectangle the first column holds texel U = 1, the second U = 0, then 255, 254 and on; in every y-flipped
  one the first row holds V = 0, then 255. Decrementing from U, Diamond's first draw, left 134,144 pixels off.
- **Referee:** not consulted. **Witnesses:** PCSX-ReARMed and SwanStation do not flip at all (Peter Lemon's flipped
  rectangles, every differing pixel inside them, `Diamond_GPU.md` §11).
- **Conclusion:** U+1, U, U-1 and on for an x-flip; V, V-1 and on for a y-flip. Measured on one console's picture, with
  every flipped rectangle starting at an even column (260, 714). Open: whether the offset depends on the start column's
  parity or on U's (psx-spx's "the texture UV, as well as the texture width must be even" for rectangles hints at
  pairs); ~~a program drawing x-flipped rectangles at odd and even columns with odd and even U, run on a console,
  decides~~.
- **The referee step, 2026-10-11** (`Diamond_Referees.md` §4.7, `programs/xflip.s`) [M]: that program ran on every
  referee. Only Beetle PSX flips at all; DuckStation, SwanStation, PCSX-Redux and PCSX-ReARMed ignore both bits. Beetle
  PSX matches the console's picture where it reaches (U 0: 1, 0, 255 …), at even and odd columns alike, and with an
  odd U starts at U (U 1: 1, 0, 255 …; U 11: 11, 10 …) where Diamond started at U+1. **Changed, on the strongest
  single witness**, the only referee that flips and one that agrees with the console's picture: an x-flip starts from
  **U with its low bit set**, whatever the column; a y-flip from V, as before. psx-spx's "the texture UV, as well as
  the texture width must be even" is consistent with a texel pair as the unit [A].
- **Depends on it:** the 208-pin chip's flipped sprites.
- **Pinned by:** `texture-flip` exact; `an_x_flipped_rectangle_starts_one_texel_on` (*renamed*
  `an_x_flipped_rectangle_starts_from_u_with_its_low_bit_set` *at the referee step*);
  `flipped_rectangles_start_where_the_referee_starts_them`.
- **Implemented in:** `diamond: the GPU of stage 4, and VRAM graded against ps1-tests' pictures`; *since the referee
  step* `diamond: a triangle's attributes start from its leftmost vertex (D-7), an x-flip from U with its low bit set (D-8)`.

### D-9. GP0(E3h) to (E5h) act in their turn, not as they arrive

- **Opened:** 2026-10-11, at stage 4, by Peter Lemon's `RenderPolygonClip16BPP`.
- **Documents read:** psx-spx `6d7d1bc`, "GP0(E3h..E5h)" note: they "do not take up space in the FIFO, so they are
  probably executed immediately (even if there're still other commands in the FIFO) ... the new Drawing Area settings
  might accidentally affect older Rendering Commands in the FIFO".
- **Test program:** Peter Lemon's `RenderPolygonClip16BPP` (source read): it writes six groups of primitives by the CPU
  with no wait, a new drawing area before each group. Acting on arrival, Diamond drew seven of its primitives with a
  later group's area, which clipped them away.
- **Referee:** not consulted. **Witnesses:** PCSX-ReARMed and SwanStation draw each group within its own area, as
  does the program's bundled picture; its author tests on a console, by his README.
- **Conclusion:** the three take their turn in the FIFO; argued from the witnesses and the bundled picture, not
  measured on a console. psx-spx's sentence is a guess ("probably", "might") [R]. Open: whether they take a place in
  the FIFO (Diamond gives them one); a program filling the FIFO with them behind a slow command and reading GPUSTAT's
  bit 25 in direction 1 decides.
- **Depends on it:** every program that changes the area or the offset between primitives without waiting.
- **Pinned by:** `the_fifo_holds_what_arrives_while_a_command_draws`.
- **Implemented in:** `diamond: GP0(E3h) to (E5h) wait their turn in the FIFO`.

### D-10. GPUSTAT bit 28 is the FIFO's emptiness, set while a rectangle draws, where psx-spx clears it while a command executes

- **Opened:** 2026-10-11, at stage 4, by `gpu/gpustat`'s console log.
- **Documents read:** psx-spx `6d7d1bc`, "GPU Status Register" and its "Ready Bits": bit 28 "gets cleared when the
  command execution is busy (ie. once when the command and all of its parameters are received), however, for Polygon
  and Line Rendering commands, the bit gets cleared immediately after receiving the command word"; bit 26 "set when the
  GPU wants to receive a command".
- **Test program:** ps1-tests' `gpu/gpustat` (source and console log; the release has no binary): a 128x128 rectangle
  written by the CPU reads bit 26 clear and bit 28 **set** while it draws; with a second rectangle queued behind it by
  DMA, both clear. Rewritten as `programs/gpustat.s`.
- **Referee:** *2026-10-11* (`Diamond_Referees.md` §4.8, `programs/gpustat-shapes.s`, and `gpustat.s` on every
  referee) [M]: on the console log's twelve readings DuckStation and SwanStation score 9, Beetle PSX 6, PCSX-Redux 4,
  PCSX-ReARMed 3. While a polygon's or a line's vertices arrive, DuckStation and SwanStation keep bit 28 set, as
  Diamond does; Beetle PSX clears it from the command word to the last vertex, psx-spx's sentence for those commands;
  all three keep it set while a rectangle draws, as the log does. **Witnesses:** none before the referee step.
- **Conclusion:** bit 28 is set exactly when the FIFO holds no word, and a word goes to the processor at once while it
  is not drawing; bit 26 is set when, besides, nothing is half received or drawing. Measured: the log's twelve
  readings. Open: polygons and lines, which psx-spx singles out; the log has only rectangles. *Referee step,
  2026-10-11:* kept, on the strongest single witness on GPUSTAT (DuckStation, 9 of 12 on the log against Beetle
  PSX's 6); still open, since the lineages disagree there and psx-spx sides with the weaker. Pinned since by
  `gpustat_while_each_shape_arrives_and_draws`.
- **Depends on it:** DMA's pacing in modes 1 and 2 with direction 2, and every program that polls the two bits.
- **Pinned by:** `gpustat_reads_as_the_console_passes`; `the_fifo_holds_what_arrives_while_a_command_draws`.
- **Implemented in:** `diamond: the GPU of stage 4, and VRAM graded against ps1-tests' pictures`.

### D-11. The drawing time: fitted to `gpu/bandwidth`, with the GPU left busy after some long transfers unexplained

- **Opened:** 2026-10-11, at stage 4: psx-spx says "GPU polygon rendering time is unknown" and gives nothing of the
  fill's, the copies' or the transfers' rates.
- **Documents read:** psx-spx `6d7d1bc`, "DMA Channels" (its rate table and the sentence above).
- **Test programs:** ps1-tests' `gpu/bandwidth` (source and console log: fourteen rows of 400 full-screen primitives or
  transfers, timed in hblanks) and `dma/chopping` (its "(GPU busy)" mark, GPUSTAT bit 26 read after each transfer).
- **Referee:** *2026-10-11* (`Diamond_Referees.md` §4.9, `programs/gpu-busy.s`) [M]: after 2,048 words in blocks of 1
  to 128 or one burst, every referee reads the GPU ready at the first poll (PCSX-ReARMed alone sets bit 26 22,031
  cycles after its burst, with its blocks untimed); DuckStation's `chopping` busy marks equal Diamond's on all 130
  rows, the same 117 agreeing with the console and the same 13 not. **Witnesses:** none before the referee step.
- **Conclusion** [M, fitted]: `Diamond_GPU.md` §8's model, every `bandwidth` row within 0.6% of the console's.
  `chopping`'s busy mark agrees on 117 of 130 rows; the 13 others, after 128-word blocks, the unchopped burst and
  chopped windows of 16 to 128 words, read busy on the console and idle on Diamond. A write rate slower than the DMA's
  makes a full FIFO hold the DMA back, which the console's cycles rule out (at 2.38 cycles a word only 93 rows stay
  within 10%), so the console's GPU goes on working after a long transfer without slowing it: a deeper buffer between
  the FIFO and VRAM, or a write that ends late, would; neither is in a document. **Open.** ~~A program timing GPUSTAT
  bit 26 against the cycles after a burst of N words, on a console, would measure it.~~ *Referee step, 2026-10-11:*
  that program ran on every referee and none keeps the GPU busy; DuckStation independently supports the 117 rows, and
  nothing bears on the 13. Still open. Pinned since by `the_gpu_is_ready_at_once_after_each_transfer`.
- **Depends on it:** GPUSTAT's busy bits and DMA's pacing; the speed at which games draw.
- **Pinned by:** `bandwidth` graded against its log (2%); `a_gpu_cycle_is_a_video_clock`.
- **Implemented in:** `diamond: the GPU of stage 4, and VRAM graded against ps1-tests' pictures`.

### D-12. Two of ps1-tests' pictures cannot be met in full by the release's programs: `lines` shades to a colour it never sets, and `transparency`'s picture is from an older program

- **Opened:** 2026-10-11, at stage 4, by the VRAM gate.
- **Documents read:** none (a property of the test programs).
- **Test programs:** `lines`, whose `gouraudMultiLine` sets its second vertex's colour twice and its fourth never, so
  the segments from the third vertex shade toward whatever the stack held: the 64 pixels of the two such segments
  (one opaque, one semi-transparent) differ, every one of them on those segments, and nothing else does.
  `transparency`, whose picture is white outside its 320x240 screen where the release's program clears VRAM black: its
  picture was added on 2019-09-24 (`da6b472`) with a clear to white, and the program was changed to `clearScreen()`'s
  black on 2019-12-02 (`2fe1570`), the picture not regenerated [R, the repository's history].
- **Referee:** not consulted. **Witnesses:** none.
- **Conclusion:** `lines` is graded with those 64 pixels excluded, by name; `transparency` over its screen, where it is
  exact. Settled: the mismatches are the programs', not the GPU's.
- **Depends on it:** the gate only.
- **Pinned by:** `Ps1GpuTests.Programs`' exclusions.
- **Implemented in:** `diamond: the GPU of stage 4, and VRAM graded against ps1-tests' pictures`.

### D-22. SIO0's timing: a byte received when its eighth bit is sampled, /ACK 412 cycles after its last clock for 100 cycles, from figures psx-spx gives for other devices

- **Opened:** 2026-10-11, when the digital pad was brought forward from stage 6 (`Diamond_Pad.md` §2-§3): no document
  gives a pad's /ACK delay, and psx-spx gives the moment a received byte becomes readable only through the clock's
  edges.
- **Documents read:** psx-spx `6d7d1bc`, "Serial Interfaces (SIO)" (DR, SR, MR, CR, BR and the notes);
  "Controller and Memory Card Signals" (the edges, the kernel's 100-cycle and 100 µs window, /ACK of at least 2 µs);
  "Sony Mouse Hardware Bug on Power-On" ("around 100 clk cycles" low); "Memory Card Read/Write Commands", "Timings"
  (IRQ7 "circa 1500 cycles after sending a byte (counted from the begin of the first bit)").
- **Test program:** `programs/pad.s` on Diamond only (no console run): IRQ7 1,500 cycles after each acknowledged
  byte's first clock, as chosen. amidog's tests read the pad by polling, with no IRQ, sending a byte and reading DR
  after a delay loop of about 1,030 cycles without waiting for RXRDY; on Diamond the reply is there 1,020 cycles after
  the write, ten cycles before the read [M]; had it arrived at the byte's end, 1,088, the tests would have read every
  reply one byte late. That the console's tests work is weak evidence that the reply is readable before the last
  half bit ends.
- **Referee:** not consulted. **Witnesses:** none run (no reference emulator was run for this step).
- **Conclusion:** the reply readable 7½ bits after the first clock; /ACK from the card's figure for every device; the
  transfer starting at once rather than on the baud timer's next reload. Argued; open until a program of the project's
  own times IRQ7 and RXRDY with root counter 2 on a console.
- **Depends on it:** the kernel's pad and card functions' timeouts; games that read the pad by polling with fixed
  delays, as amidog's library does.
- **Pinned by:** `sio0::tests::a_pad_read_answers_with_an_ack_after_every_byte_but_the_last`,
  `the_status_follows_a_byte_through_the_transmitter_and_the_fifo`.
- **Implemented in:** `diamond: SIO0 and port 1's digital pad, brought forward from stage 6`.
