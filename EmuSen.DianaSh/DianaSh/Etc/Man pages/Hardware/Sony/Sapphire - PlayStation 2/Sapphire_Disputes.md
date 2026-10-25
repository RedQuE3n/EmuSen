# Sapphire_Disputes — the disputes log

*Created 2026-10-23, at stage 0, with its first entry.* This is the log `Sapphire_Plan.md` §1.5 defines, in the shape
`Diamond_Disputes.md` keeps. An entry records a rule the documents and the test programs leave open or in
disagreement, the evidence taken, and the rule as implemented. It is the record of which sources a dispute step read,
so that the clean-room protocol of `Sapphire_Plan.md` §1.4 can be audited from outside. Disputes about the IOP's
processor are kept in `blackmoon-r3000`'s record (`../BlackMoon-HW/BlackMoon_R3000.md` §8), since they reach Diamond
too; disputes about the R5900 are kept in its crate's record once it exists.

## The rules this log keeps

- **A dispute step is separate.** Its only product is an entry. It ends with a rule stated in hardware terms, and a
  writer's step implements it and cites the entry. Nothing read in a dispute step is typed into code in that step.
- **The order of recourse is fixed:** the documents, then a test program written or run for the purpose, then the
  reference emulators run as black boxes, outputs only (`Sapphire_Plan.md` §1.6). No console is used, now or later,
  and no emulator's source, SDK source or FPGA RTL is read at any step.
- **The referees are weighed, not counted** (`Diamond_Referees.md` §3, adopted by `Sapphire_Plan.md` §1.5): a ruling
  needs agreement between lineages, or names the strongest single witness as such, its strength measured against the
  console outputs ps2autotests ships; a majority alone rules nothing.
- **No firmware is read.** A dispute step never opens, traces or disassembles the BIOS or any other firmware; a
  referee's log that shows BIOS output exposes its reader (`Sapphire_Plan.md` §1.4).
- **An entry is never deleted.** A conclusion later overturned is struck through and followed by the new one, with the
  evidence that overturned it.

## Entry shape

```
### D-<n>. <component>: <the rule, one sentence, in hardware terms>
- Opened: <date>, by <what left it open or showed the disagreement>
- Documents read: <title, revision, section> for each
- Test program: <the program written or run, on what, and what it showed>
- Referee: <which referees ran which program, and what each showed> | <not consulted>
- Conclusion: <the rule as implemented, in prose>; argued | measured | open
- Depends on it: <what in the core or the framework changes with the answer>
- Pinned by: <the test that fails if the rule is changed>
- Implemented in: <the commit that cites this entry>
```

## Entries

### D-0. The system clocks: whether the EE runs at 294.912 MHz, the bus and the GS at 147.456 MHz and the IOP at 36.864 MHz on every model, or at the 300, 150 and 37.5 MHz figures, or the 299 MHz of later models, that other sources give

- **Opened:** 2026-10-23, at stage 0, by the plan's reading of its sources (`Sapphire_Plan.md` §2.3).
- **Documents read** (online, 2026-10-23):
  - *ps2tek* at `1c91660`: "EE Architecture" gives the EE "Speed: 294.912 MHz", and the VU chapter the same figure;
    "EE Timers" gives the bus clock as "~147 MHz" (TN_MODE's clock setting); "CDVD Reads and Seeks" gives
    "IOP_CLOCK is ~36,864,000 Hz"; the BIOS's PlayStation compatibility notes say the IOP "is underclocked to 33.8688 MHz,
    the original speed of the PSX" in that mode. No GS clock is stated.
  - *Wikipedia, "PlayStation 2 technical specifications"*: the EE "clocked at 294.912 MHz (299 MHz on newer
    versions)", cited to Hennessy and Patterson and to Diefendorff and tagged as failing verification; the GS at
    147.456 MHz with no citation, and the GIF bus and video memory at 150 MHz citing Hennessy and Patterson; a fill rate
    of "16 × 147 Mpix/s = 2.352 gigapixel/s" in one place and "2400 megapixels/sec" citing the GS user's manual in
    another; the IOP at 33.8688 or 36.864 MHz, its memory at 37.5 MHz citing Sony's *Emotionally Charged Chip* article.
  - *Kunimatsu et al., "Vector Unit Architecture for Emotion Synthesis", IEEE Micro 20(2), 2000*, through its abstract
    as a search engine quotes it: "5.5 Gflops at an operation frequency of 300 MHz". The paper itself not yet read.
- **What the readings imply** [A, arithmetic on the figures above]:
  - 36.864 MHz is exactly 768 × 48,000 Hz, the SPU2's output rate, as the PlayStation's 33.8688 MHz is 768 × 44,100 Hz.
    294.912 MHz is 8 × 36.864 MHz and 147.456 MHz is 4 × 36.864 MHz. ps2tek's three figures are therefore one crystal
    family with an exact audio ratio, and 33.8688 MHz in the PlayStation mode is the other family.
  - 2,400 Mpixel/s at the GS's 16 pixels a clock is 150 MHz; at 147.456 MHz the same pipeline gives 2,359.3 Mpixel/s,
    which Wikipedia's own second figure (2.352) approximates. The manual's figure is consistent with a 150 MHz design
    target and with the EE's 300 MHz in Sony's paper: both are the published design, 1.7 % above ps2tek's.
  - 299 MHz is 8.11 × 36.864 MHz, not an integer multiple. If later models run the EE at 299 MHz, their EE and SPU2
    clocks are no longer one family, which would change the scheduler's ratios for those models; if the figure is a
    rounding or a design number, nothing changes.
  - 37.5 MHz for the IOP's memory is 150 / 4, the design family again, against ps2tek's processor at 36.864.
- **Test program:** none yet. **Evidence to take**: ps2autotests' `cpu/ee_cop0/performance` expected text, if it records
  cycle counts against a time base the console fixes (read at S2); a program of the project's that counts EE cycles
  (COP0 Count) and bus-clock timer ticks across a number of vertical blanks, run on the referees, which can only say
  what their authors chose; and any console log a public suite ships that counts cycles per frame, as ps1-tests' did for
  Diamond's D-0.
- **Referee:** not consulted.
- **Conclusion:** open. Sapphire's scheduler is written on ps2tek's family (294.912 / 147.456 / 36.864 MHz, the SPU2's
  48 kHz as 6,144 EE cycles), the ratio a constant of one module, so that a ruling for any model changes one place.
- **Depends on it:** the scheduler's ratios (`Sapphire_Plan.md` §5.1); the EE timers' rate on the bus clock; the
  frame's length in EE cycles and every speed figure of §5.12; the CRT filter's timing (§4.6), with the PCRTC's own
  video clock beside it; whether a model setting must carry a clock.
- **Pinned by:** to be written at S6, when the scheduler exists.
- **Implemented in:** —
- **Addendum, 2026-10-24 (stage S2)** [R, `Sapphire_References.md` §5.3]:
  - ps2autotests' `cpu/ee_cop0/performance.expected`, the console's output, tests the counters' register mapping and
    which modes count; it records no count against a time base and gives nothing for this entry.
  - K. Diefendorff, "Sony's Emotionally Charged Chip", Microprocessor Report 13(5), 1999-04-19, its Table 1 ("Source:
    SCE"): the GS at 150 MHz, the IOP at "33.8 MHz or 37.5 MHz selectable", the IOP-to-EE bus at 150 MB/s. A
    pre-release article, so the design family again (150 and 37.5 = 150 / 4), against ps2tek's 147.456 and 36.864.
    copetti.org's article (read 2026-10-24) gives the IOP at 37.5 MHz, citing its own sources.
  - The conclusion stays open; the evidence still to take is the project's own counting program on the referees (S4).
