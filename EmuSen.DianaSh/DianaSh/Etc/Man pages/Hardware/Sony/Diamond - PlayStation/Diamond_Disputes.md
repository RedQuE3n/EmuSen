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
- **Pinned by:** —
- **Implemented in:** —
