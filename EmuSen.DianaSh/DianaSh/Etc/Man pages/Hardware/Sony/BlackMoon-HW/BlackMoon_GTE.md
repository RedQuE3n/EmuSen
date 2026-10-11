# BlackMoon_GTE — the GTE's record

*Started 2026-10-11, at Diamond's stage 1.* `blackmoon-gte`, the Geometry Transformation Engine on the R3000A's
coprocessor 2. Planned in `Diamond_Plan.md` §5.3, §9.1 and §7's stage 5; the coprocessor's contract is `README.md`
beside this page.

## 1. Stage 1 (2026-10-11)

The registers, their names, the state block and the signatures of the register transfers and the command, each
`NotBuilt` while `BUILT` is false. Written from psx-spx's "Geometry Transformation Engine (GTE)" chapter, as names and
layout only.

## 2. The interface

- **`Registers`**: the 32 data registers (cop2r0–31) and the 32 control registers (cop2r32–63) as the hardware holds
  them, a word each. What a read returns differs from what is held for several of them (psx-spx gives IRGB's,
  SXYP's, LZCR's and H's quirks), so reads go through the transfers, not the array.
- **`DATA_REGISTERS`, `CONTROL_REGISTERS`**: psx-spx's names, VXY0 to LZCR and RT11RT12 to FLAG, which Diamond's
  machine info lists for processor 1, the GTE, from stage 1.
- **The transfers**: `data` (MFC2, SWC2), `set_data` (MTC2, LWC2), `control` (CFC2), `set_control` (CTC2).
- **`command`**: the COP2 instruction's low 25 bits; returns the cycles until its results can be read, the core owning
  the clock (README).

## 3. The oracle

ps1-tests' `test-all` (generated on a console, by its README's account of `gte-fuzz`) and amidog's `psxtest_gte`,
every case bit for bit (`Diamond_Plan.md` §5.3, G1). The r3000 single-step suite has no GTE cases (its README's
caveats). Both programs need the bus and the kernel, so they run at stage 5 through Diamond; no crate-level oracle
exists before then.

## 4. The state block

`impl State for Gte`: `Data` (32 words), then `Control` (32 words), 256 bytes, the layout pinned by
`the_layout_is_pinned_and_a_block_round_trips`. Diamond's state carries it as `Gte`. The command in flight and its
remaining cycles join the block at stage 5, which raises Diamond's state version.
