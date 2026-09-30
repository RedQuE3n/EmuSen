# VenusRT_Disputes — the disputes log

*Created 2026-09-30, at stage 0, before any VenusRT rule was written.* This is the log `VenusRT_Plan.md` §1.4 defines.
An entry records one disagreement between VenusRT and an oracle that the documents did not settle at once, the
evidence that settled it, and the rule as implemented. It is the only record of which sources a dispute step read,
so that the clean-room protocol of §1.2 can be audited from outside.

## The rules this log keeps

- **A dispute step is separate.** It changes no code. It ends with a rule stated in hardware terms, and a later
  writer's step implements that sentence and cites the entry (`VenusRT_Plan.md` §1.2).
- **The order of recourse is fixed:** the documents, then a test ROM written for the purpose, then the SNES_MiSTer
  RTL, and only then Mesen's source. An entry that cites Mesen's source says why the three before it were not enough.
- **Nothing read in a dispute step is typed into code in that step.** Files read are listed by path and line; nothing
  is transcribed here either, only the conclusion in prose.
- **An entry is never deleted.** A conclusion that is later overturned is struck through and followed by the new one,
  with the evidence that overturned it.
- **The weight of the referee** is `Venus_Referee.md` §0's table, subsystem by subsystem. Only §0 of that page is
  read; its later sections describe Venus's internals.

## Entry shape

```
### D-<n>. <component>: <the rule, one sentence, in hardware terms>
- Opened: <date>, by <what showed the disagreement: ROM, frame, space, the two values>
- Documents read: <title, section> for each
- Test ROM: <the ROM written or run to settle it, its result on VenusRT and on Mesen>
- Referee: <RTL files and lines read, and the weight of that subsystem per Venus_Referee.md §0>
- Mesen's source: <none> | <commit, each file path read, and why the three above did not settle it>
- Conclusion: <the rule as implemented, in prose>; argued | measured
- Pinned by: <the test that fails if the rule is changed>
- Implemented in: <the later commit that cites this entry>
```

## Entries

None. Stage 0 wrote no emulation rule, so nothing could disagree with an oracle (`VenusRT_Native.md` §1).

Two readings of the oracles were made at stage 0 that a later reader might take for disputes. They are not: each is
about what a suite's file format means, settled by the suite's own data and README, and neither says anything about
the hardware. They are recorded in `VenusRT_Native.md` §2.3: the SingleStepTests 65816 cases of MVN and MVP stop at
100 cycles in the middle of a move, and the SPC700 suite records dummy reads of memory a case does not list without
their data.
