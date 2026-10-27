# Beryl_FmTables — the FM operator tables' record

`beryl-fm-tables`, in `EmuSen/Cores/Sega/Beryl-HW/`, created 2026-10-27. Two 256-entry ROMs, `LOGSIN` and `EXP`, and
nothing else; no dependencies, no state.

## 1. Why one crate

Endou's YM2413 took its operator ROMs to be the OPL2's (`Endou_Disputes.md` D-2) and Nephrite's YM2612 carries the
same pair (`Nephrite_OperatorTables.md`). A duplicate-code scan on 2026-10-27 found the two files identical entry for
entry (about 1,050 tokens). The tables are the same fact, so one crate owns them and both cores read it.

## 2. Contents

- `LOGSIN`: quarter-wave attenuation in 4.8 fixed point, entry x being round(-log2(sin((x + 0.5) * pi / 512)) * 256).
- `EXP`: 2^(x / 256) ascending with its leading one (1,024) implicit, entry x being round((2^(x / 256) - 1) * 1024).

Both are kept as written constants rather than computed at build time: the crate's tests recompute every entry from
the formulas above, so a changed entry fails, and a build script would make the table depend on the host's `libm`.

## 3. What stays in each core

Endou keeps the YM2413's instrument and rhythm ROMs (`src/opll/tables.rs`); Nephrite keeps its LFO and envelope tables.
Neither core's sound changed: the tables are the same values, and each core's audio digests are as before the move.

## 4. Tests

Five in the crate: both formulas, the printed spot values, the widths and monotonic shape, and the complemented
exponent index against 2^-(a + 1). Seeded faults of 2026-10-27 (an entry off by one, the two tables swapped, a table
shortened by one) were each caught by the crate's tests.
