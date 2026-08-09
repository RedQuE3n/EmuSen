# Mercury — the hardware test corpus as an oracle

*Written 2026-08-09, alongside the serial sink (`Mercury_Memory.md` §10). This page
is about where Mercury's correctness evidence comes from, and why for the Game Boy
it does not come from another emulator.*

---

## 1. The problem this solves

Every accuracy claim needs an oracle: some authority that says what the right answer
was. For Venus the answer is `Reference/` — a probe wrapping Mesen, a dump
comparator, and `known-differences.db` recording where the two legitimately diverge.
The GSU diffs to zero against it, and that is a strong statement.

That machinery does not transfer to the Game Boy, for three reasons:

1. **Mesen has no Game Boy core.** The backend that produces the richest dumps
   simply cannot load a `.gb`.
2. **A libretro core can be captured but not diffed sample-for-sample.** Its audio
   is reachable (`Mercury_Gameplan.md` §3.1), but two emulators mixing at different
   rates and phases never match exactly, so any comparison needs a tolerance — and a
   tolerance needs a justification, which is a research question of its own.
3. **Agreement is not correctness.** Matching gambatte proves Mercury and gambatte
   made the same choice. It does not distinguish a shared mistake from a shared
   truth, and for a console this well documented that is leaving evidence on the
   table.

The Game Boy has something the SNES does not: **a mature corpus of hardware tests
written against real silicon, which report their own verdict.** blargg's suite and
mooneye's suite were developed with logic analysers against DMG and CGB hardware.
They do not need an emulator to grade them. They grade themselves, and they say so
out loud through the link port.

This is what "reference-agnostic" means here, in the same sense
`EmuSen_Debugging_Tools_Reference_v5.md` §3 means it for the probe: **the oracle
names no emulator.** It is an assertion about a Game Boy.

## 2. Reading the verdict

`HardwareTestRomLibrary.ReadVerdict` takes the captured serial bytes and returns
`Passed`, `Failed`, or `NoVerdict`. The two corpora use different conventions and
both are read off the same byte stream.

**blargg** prints human-readable text: the test's name, a line per subtest, then
`Passed` or `Failed` and a number. The reader therefore checks for `Failed`
**before** `Passed`, and this ordering is load-bearing rather than stylistic — a
failing multi-part run prints the passing subtests first, so a log containing
`02:Passed 03:Failed` is a failure. A test asserts exactly that case.

### 2.1 Mooneye does not print words

Mooneye's tests end by writing the start of the Fibonacci sequence — `3, 5, 8, 13,
21, 34` — to the serial port, and by leaving the same values in `B` through `L`
alongside a `LD B,B` software breakpoint. Failure is six copies of `$42`.

Two details in the reader follow from this:

- Success is matched at the **end** of the log rather than anywhere in it, because a
  test may print other bytes first and the marker is what terminates it.
- `$42` is ASCII `'B'`, so "six identical `$42`s" would in principle collide with a
  ROM printing `BBBBBB`. Nothing in either corpus does, and the alternative — a
  register check through `MercuryDebugTarget` — would tie the harness to the
  debugger for no gain. Recorded because it is a real, if remote, ambiguity, and the
  first sign of it would be a text-printing ROM reported as a mooneye failure.

## 3. Where the ROMs live, and why none are committed

`TestRoms/hardware/`, searched recursively, gitignored, discovered by
`HardwareTestRomLibrary`. **The suite passes when the directory is absent** — the
same convention `CommercialRomLibrary` uses for commercial cartridges
(`Mercury_RealCartridges.md` §1) and for the same reason: a test that cannot run
without an artifact the repository does not carry must not fail the build for
someone who lacks it.

The rule these ROMs sit under is different from the commercial one, though, and the
distinction is worth being precise about. blargg's and mooneye's tests are freely
distributable homebrew with published source; the repo's ban is on *commercial*
ROMs. They are kept out anyway, because a test corpus is an input to the research
rather than part of it, and because vendoring binaries whose provenance the project
does not control is a habit worth not starting.

## 4. What the corpus is expected to say

These are predictions, recorded before the ROMs were obtained so that being wrong is
visible later:

- **`cpu_instrs` should pass now.** The instruction set and flags have been complete
  since Phase A and are covered by 18 synthetic tests.
- **`instr_timing` should pass now.** Per-instruction cycle counts are modelled even
  though their *distribution within* an instruction was not, until the
  cycle-granular bus landed (`Mercury_Gameplan.md` §3.3).
- **`mem_timing` and `mem_timing-2` are the reason the bus became cycle-granular.**
  Before that work they could not pass; they are the assertion that it landed.
- **`oam_bug` should fail**, and should keep failing until OAM access blocking
  exists. It tests a DMG hardware defect Mercury does not reproduce.
- **`dmg_sound` will probably partly fail.** `Mercury_Apu.md` §7 already lists the
  length-counter obscure behaviours as unmodelled, and this is the suite that names
  them individually.
- **`cgb_sound` is the first genuine `$C0` ROM to reach the core** and therefore
  also the first real exercise of the colour register file at reset — the defect
  recorded in `Mercury_Cgb.md` about `Cpu.Reset` was found by inspection, not by a
  colour ROM.

## 5. What this does not establish

- **It does not cover the mixer.** Every sound test here asserts register and
  counter behaviour. Whether Mercury's DAC output *sounds* right is untouched, and
  remains the open question from `Mercury_RealCartridges.md` §4.
- **It does not replace the commercial suite.** A test ROM exercises one behaviour
  deliberately; a shipped game exercises whatever it happens to. Both are needed,
  and `Mercury_RealCartridges.md` §6 still lists deep gameplay as uncovered.
- **A pass is not a proof of the whole subsystem.** These tests are thorough about
  the things they test and silent about everything else — passing `cpu_instrs` says
  nothing about the PPU.

## 6. The audio differential, and what its first run said

*Added 2026-08-09, the day the capture path was fixed (`Mercury_Gameplan.md` §3.1).*

Both halves now exist. The probe's libretro backend accumulates the samples a core
pushes it and writes the WAV itself, at the rate `retro_get_system_av_info` reports
— gambatte's is **32768 Hz**, not the 44100 an assumption would have used, and
mislabelling that would have made every later comparison wrong in a way nothing
would have caught. Mercury's side is `AudioCapture`, written by the commercial-ROM
suite when `EMUSEN_AUDIO_DUMP` names a directory and by nothing otherwise.

`Reference/analysis/audio.py` compares them as a **feature stream** rather than as
samples, for the reason §1 gives: two correct emulators never produce identical
samples. Windowed RMS, onsets, zero-crossing rate, a systematic gain offset, and a
lag search.

### 6.1 The first result: aligned events, disagreeing levels

Three DMG cartridges, 900 frames each with Start and A tapped from frame 150, the
same schedule replayed to gambatte through `--press`:

| ROM | gain | envelope over tolerance | onsets unmatched | best lag |
|---|---|---|---|---|
| Super Mario Land | +8.6 dB | 27.5% | 5.5% | +1 window |
| Tetris | +8.9 dB | 25.8% | **0.0%** | +2 windows |
| Dr. Mario | +8.8 dB | 21.9% | 4.7% | 0 windows |

Read carefully, this says three different things, and only the first is settled.

**Mercury runs about 9 dB hotter than gambatte, consistently.** This is a *level
convention*, not a defect. The Game Boy's output is analogue and how much of a
16-bit range to fill is the frontend's choice. It is now measured, reported, and
subtracted before anything else is judged — otherwise every ROM reads as divergent
forever and the comparator stops being read, which is exactly the failure the
module's own docstring warns about. Clipping was checked as the reference-free half
of the question and is **not** happening in any meaningful amount: the worst ROM
pins 0.063% of its samples and four of six pin none.

**The note events agree.** Onsets match to within a couple of windows, and Tetris's
match exactly. Whatever else is true, both emulators are running the same sound
driver at the same tempo and starting the same notes at the same times.

**The moment-to-moment level does not agree**, by about 7 dB mean absolute error
per 20 ms window after the gain offset is removed.

### 6.2 A negative result: it is not the four-frame boot offset

The obvious suspect was the documented boot skew — Mercury reaches the cartridge
about four frames ahead of gambatte (`Mercury_RealCartridges.md`), so the two could
simply be hearing the same music at different points.

**They are not.** `find_lag` searches ±60 windows and the best alignment is 0 to 2
windows (0 to 40 ms) on all three ROMs, with the residual barely improving there —
8.1, 7.2 and 7.2 dB against 9.6, 11.7 and 7.2 dB at zero lag. A tempo or timing
explanation would have shown a clear minimum at some non-zero lag. There is none.
Recorded because ruling it out is the useful part; the lag search was kept as a
permanent capability for exactly this reason, and it is tested with a
deliberately delayed signal so that it is known to find a shift when one exists.

### 6.3 What this does not license

**Do not tune Mercury's mixer until it agrees with gambatte.** §1 is explicit that
agreement is not correctness, and that argument does not weaken because a number is
now available. The plausible causes of a 7 dB per-window disagreement include
Mercury mixing at the output rate and aliasing where gambatte resamples properly
(`Mercury_Apu.md` §7, already recorded as a known simplification), different
channel weighting, and a different high-pass corner — and gambatte is not evidence
about which of those hardware does.

The instrument that *can* settle it is `dmg_sound` from the test corpus, which
asserts against real silicon. This differential's proper role is what it just did:
detect a difference, quantify it, and rule out one explanation. It is a tripwire,
not a judge.

**The Super Mario Land question from `Mercury_RealCartridges.md` §4 stays closed
for the reason it was closed.** SML genuinely does not call its music driver until
Start is pressed, which coverage proved without any reference. Nothing here reopens
it — the capture above has Start pressed from frame 150 and both emulators produce
61 and 64 onsets, which is two emulators agreeing that the music starts.
