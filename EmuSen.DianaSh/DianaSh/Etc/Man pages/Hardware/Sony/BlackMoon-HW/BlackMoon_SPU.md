# BlackMoon_SPU — the SPU voice's record

*Written 2026-10-25, at Sapphire's stage S26.* `blackmoon-spu`, the part of the PlayStation's SPU that the PlayStation
2's SPU2 runs twice over: SPU-ADPCM decoding, the Gaussian interpolation, the ADSR envelope and the volume sweeps, the
noise generator, pitch modulation, each voice's state, and the reverb's algorithm with its work area and offsets as
parameters. Planned in `Sapphire_Plan.md` §9.3 and decided by the tester's Q10 (2026-10-24); moved out of Diamond's
`src/spu`, where it was written at Diamond's stage 8 (`Diamond_SPU.md`). Its users are Diamond's SPU and both cores of
Sapphire's SPU2 (`Sapphire_Native.md` §32).

The marks are the plan's: **[M]** measured on 2026-10-25 on the desktop (Ryzen 7 7700X, Fedora 44) with the tool named;
**[R]** read from a source, named; **[A]** argued; **[C]** a choice.

---

## 1. What is in the crate, and what is not

The rules are psx-spx's as `Diamond_SPU.md` §3 to §5 records them, with Diamond's readings where psx-spx stops (D-34,
D-35 and D-36 of `Diamond_Disputes.md`: the prediction's products shifted apart, the interpolation's products summed
before the shift, the envelope stepping after the sample it scales) [R]. The crate holds what a voice does sample by
sample and nothing that a console decides:

| In the crate | In each core |
|---|---|
| `voice`: a voice's registers as values, its block address, pitch counter, decoded samples, decoder history, envelope, phase, current volumes and output; key on and off; one sample (`Voice::tick`); the pitch step with modulation (`voice::step`); a volume register's cycle (`voice::sweep`) | the register map and its widths, where each value is read and written |
| `envelope`: the step that ADSR and the sweeps share | the clock: when a sample is made, and at what rate (44,100 Hz on the PlayStation, 48,000 Hz on the PlayStation 2) |
| `noise`: the generator's level and timer, one sample at a shift and step | which voices take the noise, and when its timer restarts (Diamond's D-37 rule is the core's, on its register write) |
| `reverb`: the formula, its accesses in psx-spx's order, the 39-tap filter at half the rate, the buffer address | the reverb's registers: their layout, their units, where the work area ends |
| `tables`: the interpolation's 512 entries, the filter's 39 taps, the five ADPCM filters | the mix, the routing, the transfers, the capture or output areas, external inputs |

**A core lends sound RAM** as a byte slice whose length is a power of two (512 KiB on the PlayStation, 2 MiB on the
PlayStation 2); every address wraps within it [C]. **Nothing keeps time**: a core calls `Voice::tick` once a sample.
**The IRQ address is the core's**: every access a voice or the reverb makes is reported through the trait `Touch`, which
the core implements with its own comparison (Diamond's in 8-byte units, Sapphire's by halfword for both cores at once);
`()` watches nothing. The crate is `no_std`, allocates nothing, and depends on nothing.

## 2. The interface's units

- A voice's **start and loop addresses** are 32-bit values in 8-byte units, PlayStation's SSA and LSAX as they are;
  its **block address** is in bytes. A core converts: Diamond's registers are already 8-byte units; Sapphire's halfword
  addresses lose their two low bits on the way in (`Sapphire_Disputes.md` D-268) [C].
- The **reverb** takes each sample's coefficients and offsets as `Params` (coefficients sign-extended, offsets in
  bytes from the buffer address, each pair left then right in psx-spx's order) and its work area as `Area` (start and
  end in bytes). The buffer address advances by a halfword after the right side, to the area's start when it reaches
  the end, and never below the start: psx-spx's `MAX(ESA, (BufferAddress+2) AND 7FFFEh)` with the area's end in place of
  the end of RAM [R psx-spx "SPU Reverb Formula"; A for the generalisation]. A degenerate area of less than a halfword is
  taken as one halfword [C].

## 3. The extraction, and its gate (measured 2026-10-25)

What moved: `voice.rs`, `envelope.rs` (with its five tests), `reverb.rs`'s algorithm and `tables.rs`; the noise step,
which was inline in Diamond's `Spu`, became `noise.rs`; the pitch step with modulation, inline in Diamond's sample,
became `voice::step`. What Diamond kept: `src/spu/mod.rs` (the registers, the transfer and its FIFO, the capture
buffers, CD audio, the mix and the clock), `state.rs`, a `reverb.rs` that holds the reverb registers' layout and turns
them into `Params` each sample, and a `voice.rs` that re-exports the voice and keeps the IRQ address's comparison.

**Every address Diamond computes is the one it computed before** [A]: a start or loop address of at most FFFFh units is
at most 7FFF8h bytes, inside 512 KiB, so the new masks (from the slice's length, 7FFFFh) change nothing; a loop start
flag's LSAX is the block's address over 8, below 10000h; the reverb area's end is RAM's end, so the new wrap rule is
psx-spx's. Diamond's state block keeps its layout: SSA and LSAX are still written as halfwords.

**The gate**, run before any Sapphire code used the crate [M, `cargo test --release` in `diamond` and `diamond-v1`,
with `EMUSEN_DIAMOND_PS1TESTS` naming ps1-tests' release 158 and `EMUSEN_DIAMOND_DISCS` the tester's bench discs]:

| What | Result |
|---|---|
| `tests/goldens/programs.txt`: every project program, 60 frames, picture, **sound**, state, VRAM and TTY digests | unchanged |
| `tests/goldens/ps1-tests.txt`: ps1-tests' 48 programs on the open kernel | unchanged |
| `tests/goldens/discs.txt`: the eight bench discs to frame 3,000, the **sound folded from the start** at every 300th frame | unchanged, 80 anchors |
| `diamond` lib (the SPU's 23 rule tests unchanged in `src/spu/tests.rs`) | 176 of 176 |
| `diamond` programs (`spu-voices.s`'s samples pinned, `spu-capture.s`'s records, the transfer's timing) | 84 of 84 |
| `diamond` shell; `diamond-v1` | 10 of 10; 5 of 5 |

The tests that need a GPU (`renderer`, `precise`, `scaled`, `filtered`, `colour`) were not run, the stage being CPU work
by the tester's limit of 2026-10-24; they reach no SPU code. The firmware tests that need the player's BIOS
(`EMUSEN_DIAMOND_FIRMWARE`) were not run either.

**Speed** [M, Diamond's `bench` on Spyro the Dragon, frames 300 to 1,500, three runs each, the base built from
66771c4f7 beside it, interleaved, other work on the machine]: mean 1.970–2.339 ms a frame before and 1.982–2.224 after,
every picture, sound and state digest identical (sound `486DDC01FC9337E2`). The change is inside the runs' spread.

## 4. Tests

`cargo test` in the crate: **6 of 6** — the envelope's five, moved from Diamond unchanged, and one added at S26 that
decodes a block through each of the five filters against psx-spx's "Pos/neg Tables" written out as numbers, so that a
coefficient changed in `tables.rs` fails in the crate itself and not only in Diamond's goldens (§5). Also under
`test-checked`; clippy clean.

## 5. Seeded fault (2026-10-25)

Filter 1's positive weight changed from 60 to 61 in `tables.rs`: caught by the crate's new filter test, by Diamond's
`the_capture_program_records_the_voices_as_the_referees_do`, `the_voice_scenes_play_to_their_end` and
`the_projects_programs_give_their_goldens` (the `spu-capture.s` state digest), and not by Diamond's lib tests, whose
filter test's values happen to round alike, nor by Sapphire's gate, whose record of a voice is this crate run apart
(`Sapphire_Native.md` §32.9). Restored.

## 6. Where the PlayStation 2's referee parts from the crate

PCSX2, the one lineage that reaches the SPU2 (`Sapphire_Referees.md` §4.3), departs from these rules in ways the
PlayStation's referees did not: a key on's first samples, the interpolation's rounding, the prediction's rounding, the
exponential attack above 6000h, rate 7Fh, and the noise's clock at slow rates; its noise sequence is the crate's. The
crate keeps psx-spx's and the PlayStation referees' readings; the evidence and the ruling are
`Sapphire_Disputes.md` D-269.

## 7. What was read

Diamond's `src/spu` whole (the project's own code, moved); `Diamond_SPU.md` §1 to §9; `BlackMoon_R3000.md` §2 and this
directory's `README.md` for the interface every crate keeps. No emulator's or library's source.
