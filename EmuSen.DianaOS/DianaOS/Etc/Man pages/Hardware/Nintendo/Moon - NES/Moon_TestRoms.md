# Moon (NES) — Test ROMs

## 1. Why this exists

The 2A03 is the only part of this core validated against ground truth: `nes6502/v1`, 2,560,000 cases, final state *and* per-cycle bus traces (`Moon_CPU.md` §7). Everything else — the PPU, the APU, ten mapper boards — rests on synthetic fixtures in `EmuSen.WiseMan` and four commercial ROMs that were watched by eye.

That is a weak position, and the gap has a specific shape. The fixtures test what the code was written to do; they were written by the same reasoning that wrote the code, so they agree with it by construction. "83.8% of the local set boots four frames clean" is a *loading* result, not a correctness one — a core that renders nothing but the right nametable fetches would score the same.

The blargg suites close that gap cheaply. They are small, freely redistributable, and they were written against real hardware by someone who did not know this implementation existed. Most importantly they report a machine-readable verdict, so the whole thing runs headless with no human watching a screen.

This page is the protocol, the runner, and what the runner deliberately does not cover.

---

## 2. The `$6000` protocol

A blargg test ROM reports through the cartridge's PRG RAM window, not the screen:

| Address | Meaning |
|---|---|
| `$6000` | Status: `$80` running, `$81` awaiting RESET, `$00`-`$7F` finished with that result code |
| `$6001`-`$6003` | Signature `$DE $B0 $61` |
| `$6004`… | Zero-terminated ASCII describing the result |

`$00` is a pass. Any other completion code is a failure, and the code is *not* flattened to a boolean anywhere in the runner — a suite's own numbering is often the fastest way to find which sub-test broke.

The signature is the whole reason the status byte is trustworthy. PRG RAM powers on as zeroes here, and zero is also "passed"; without `$DE $B0 $61` sitting next to it there is no way to distinguish a passing test from a ROM that never wrote anything at all.

### 2.1 Why PRG RAM is read directly

`NesTestRomRunner.Read` goes through `MoonCore.ReadSpace(SpacePrgRam, …)` rather than `SpaceCpuBus`. Two reasons, and the second is the one that bites.

Reading `$6000` through the CPU bus routes to `IMapper.ReadPrg`, and a board may gate that: `Mmc3.ReadPrg` returns `0` when `_wramEnabled` is false. A test ROM that leaves WRAM disabled between writes would then read back as a passing test — the exact false positive the signature is meant to prevent, reintroduced one layer down. Reading the array directly observes what the ROM actually stored.

The lesser reason is that `CPUBUS` is the one Moon space declaring `HasSideEffects` (`Moon_Core.md` §4). Polling it every frame for the length of a test run is a lot of side effects to take on for a diagnostic read.

### 2.2 The RESET request

Status `$81` means the ROM wants the RESET button, and the protocol asks for at least 100 ms between the request appearing and the reset happening. The runner waits `ResetDelayFrames` (12, about 200 ms) and then calls `MoonCore.Reset()`.

That call did not exist before this harness — see `Moon_Core.md` §6 for what a soft reset keeps and what it clears. The requirement is exactly that PRG RAM survives it, because the ROM's own state machine remembers across the reset that it already asked once.

Resets are capped at `MaxResets` (4). A ROM that keeps asking is either wedged or reading a reset this core is getting wrong, and either way looping forever is the wrong answer.

**The stale request, which produced a wrong test result before it was found.** PRG RAM survives the reset — that is the whole point of `Moon_Core.md` §6 — so `$6000` still reads `$81` immediately afterwards. The ROM overwrites it when it resumes, but not instantly: `apu_reset/len_ctrs_enabled`'s reset routine writes eleven APU registers and then `delay 29830*2` before returning to the shell code that republishes the status. For two-plus frames the old request is still sitting there, and a runner that re-reads `$6000` in that window fires a **second** reset in the middle of the test.

That is exactly what happened. `len_ctrs_enabled` was recorded as a genuine emulation failure in the first baseline; it was the harness resetting the machine twice and clobbering the length counters it was about to check. Fixing the runner turned it green with no core change at all.

The rule now is that a reset request is only honoured if the running status has been seen *since the last reset* — `runningSeen` is cleared when the reset fires, and only `$80` sets it again. That also protects the completion path, which would otherwise read a stale verdict out of the same window. Any negative result from a ROM that uses the reset protocol is suspect until the runner is known to have reset it exactly once; `NesTestRomResult.Resets` is reported for that reason.

### 2.3 The half-written block, and why `runningSeen` exists

The runner samples PRG RAM once per frame, which means it can land between the ROM's write of the signature and its write of the status byte. At that instant the signature matches and `$6000` still reads `$00` — a pass, reported for a test that has not started.

So a completion code is only accepted after `$80` (or `$81`) has been observed at least once. Every real suite holds the running status for many frames, so this costs nothing; the synthetic fixture in §4 deliberately holds it for about eleven frames to exercise the same path.

The tradeoff is stated rather than hidden: a test that completes inside a single frame would never show `$80` and would be reported as `NoResult`. That is the safe direction to be wrong in — a missing verdict is visible, a false pass is not.

---

## 3. Running them

```sh
dotnet run -c Release --project EmuSen.Pharaoh -- --testroms /path/to/nes-test-roms
dotnet run -c Release --project EmuSen.Pharaoh -- --testroms /path/to/one.nes 900
```

The path is a single `.nes` or a directory, walked recursively. The optional second argument is the per-ROM frame budget, default 3600 (about 60 seconds of emulated time).

Output is one line per ROM:

- `[PASS]` — result code `$00`.
- `[FAIL]` — a non-zero code, with the ROM's own text indented beneath it.
- `[----]` — no verdict inside the budget. Either the ROM is screen-only (§6), or this core hung it.
- `[SKIP]` — the board is not implemented (`Moon_Memory.md` §4). Not counted as a failure; it is a tracked gap, not a regression.
- `[ERR ]` — a malformed image, or the core threw mid-run.

The exit code is 1 if anything failed, errored or produced no verdict. `--testroms` sets `CoreOptions.BatteryRamDisabled` for the whole run, so a suite directory is never written to; the same `.srm` reproducibility trap that bit the SuperFX work applies here and is worse, because these ROMs are shared fixtures.

`SkipRendering` is on. It drops framebuffer writes only — sprite evaluation, sprite 0 and every flag still run (`Moon_PPU.md` §3.5), so a PPU test is not silently exempted from the thing it is testing.

### 3.1 Where the ROMs come from

They are third-party and **not committed**, the same rule `nes6502/v1` follows. Fetch them from `https://github.com/christopherpow/nes-test-roms`, which collects blargg's suites among others.

The suites worth running first, roughly in the order they pay off:

| Suite | Tests |
|---|---|
| `instr_test-v5` | All 256 opcodes, official and undocumented |
| `instr_timing` | Per-instruction cycle counts |
| `cpu_interrupts_v2` | NMI/IRQ timing — the part of `Moon_CPU.md` §5.4 nothing currently validates |
| `ppu_vbl_nmi` | VBlank flag and NMI timing |
| `sprite_hit_tests` | Sprite 0, including the edge cases §3.2 of `Moon_PPU.md` lists |
| `apu_test` | Length counters, the frame sequencer, IRQ |
| `mmc3_test` | The IRQ counter, against `Moon_Memory.md` §4.6a's once-per-line approximation |

Expect failures. Several of them are already predicted by name in `Moon_PPU.md` §4 and `Moon_APU.md` §6 — per-dot timing, the odd-frame dot skip, the sprite overflow bug, DMC fetch alignment. A suite confirming a gap that was already written down is the harness working, not a new bug.

On this build machine the clone lives at `/home/red/nes-test-roms`.

---

## 4. Testing the harness itself

`EmuSen.WiseMan/Cores/MoonTestRomTests.cs` proves the runner with **no third-party data at all**. `SyntheticNesRom.BuildProtocolRom` assembles a real 6502 program that writes the signature, holds `$80` through a nested countdown, then writes a chosen result code and spins; `BuildResetProtocolRom` writes `$81`, marks PRG RAM `$6100`, and reports a pass only on the second pass through — which fails unless the soft reset genuinely preserved that byte.

That covers every branch the protocol has: pass, fail with a code, no verdict, reset-then-pass, unimplemented mapper. It is the same principle as `SyntheticRom` on the Venus side — the fixture builds its own images, and no commercial or third-party ROM is needed or committed.

**There is deliberately no WiseMan test that runs the real suites.** One was written and removed the same day. Two things were wrong with it. It took over ten minutes — 290 ROMs at a 3600-frame budget is not a unit test, and a suite nobody will wait for is a suite nobody runs. And there was nothing useful for it to assert: per-ROM pass and fail are *core results*, tracked in §5 and `EmuSen_Games_Tested.md`, so asserting them would paint the suite red for gaps that are already documented, while asserting only "nothing threw" duplicates Pharaoh's exit code at a hundred times the cost. The split is that WiseMan proves the harness and Pharaoh reports the ROMs.

---

## 5. The baseline — 2026-08-05

Against `christopherpow/nes-test-roms`, at a 2400-frame budget. **45 of the 63 ROMs that return a verdict pass**, from 31/62 the day the harness landed. A further 21 never speak the protocol; every one is a screen-only suite (§6), not a hang.

| Suite | Now | Harness landed |
|---|---|---|
| `apu_reset` | **6/6** | 4/6 |
| `instr_test-v5` | **16/18** | 15/17 |
| `apu_test` | **6/9** | 3/9 |
| `ppu_vbl_nmi` | **5/11** | 3/11 |
| `mmc3_test` | **3/6** | 0/6 |
| `instr_timing` | **3/3** | 3/3 |
| `cpu_reset` | **2/2** | 1/2 |
| `cpu_interrupts_v2` | **2/6** | 1/6 |
| `oam_read` | **1/1** | 1/1 |
| `ppu_open_bus` | **1/1** | 0/1 |

Three rounds. The first fixed the PPU open-bus latch, split the soft reset from power-on, and found one recorded failure was a harness bug (§2.2). The second rebuilt the APU frame counter and the CPU's interrupt sampling. The third made the PPU clock per dot and gave mapper boards real A12 edges.

**The methodology lesson, which cost more than any single bug.** Round two began by adjusting constants and re-running the suite — the sequencer period, the `$4017` delay parity, the tick order within a cycle — keeping whichever scored better. That produced two regressions of a passing test, a wrong explanation written confidently into `Moon_APU.md` §2.1, and no understanding. Every one of those questions was answered in minutes by reading the Mesen checkout instead:

- the frame sequence has **six** steps, not four, and the four-step IRQ spans three cycles;
- the `$4017` delay is 3 or 4 cycles on **CPU cycle parity**, mode applied after it, inhibit at once;
- a reset reads its vector **unclocked** then burns **8** cycles;
- a hardware interrupt entry is **7** cycles — two discarded reads;
- the branch quirk cancels a *just-arrived* IRQ rather than polling earlier;
- MMC3 counts A12 rises **per fetch**, and `$2006` writes drive A12 too.

None are fudge factors and none were reachable by search. **Read the reference before touching a constant** — the rule is [[feedback_use_mesen_before_guessing]] and `Moon_APU.md` §2.1 carries the retraction.

**A second lesson, from round three.** Putting real per-dot fetches underneath the line renderer broke three of the four test games, because the renderer had been quietly relying on `V` *not* advancing during a line. Adding correct behaviour under an approximation can break the approximation; `Moon_PPU.md` §1.3 has the fix (`RenderV`) and the shape of it.

**What is left.** `ppu_vbl_nmi` (6) and two of `cpu_interrupts_v2` need the NMI to land mid-instruction and sprite 0 to report at the pixel — the per-dot *renderer*, not the per-dot clock. `mmc3_test`'s remaining three want finer A12 timing still. `apu_test`'s two are the flat 4-cycle DMC fetch. `4-irq_and_dma` needs OAM DMA cycle-stepped. The `instr_test-v5` failure is `$AB` (`LXA`), unstable silicon.

---

## 6. Not covered

- **Screen-only ROMs.** Older suites print to the nametable and never touch `$6000`; they show as `[----]`. Reading them needs a framebuffer digest against a known-good capture, which is `--screenshot` and the Mesen probe's job, not this runner's. This is 21 of the ROMs in §5, and it is a real coverage hole rather than a rounding error: **`sprite_hit_tests` (11) and `sprite_overflow_tests` (5) are both entirely screen-only**, so the two suites that would test `Moon_PPU.md` §3.2's known sprite gaps are precisely the ones this runner cannot read. Wiring them up is the obvious next extension.
- **`nestest`.** It uses its own convention — entry at `$C000` in automation mode, results in `$02`/`$03` — and its real value is the cycle-accurate reference log, which belongs with `--singlestep` rather than here. The CPU is already the best-validated part of this core.
- **Per-sub-test granularity.** A suite's aggregate ROM reports one code for the whole run. Where a suite ships `rom_singles/`, the recursive walk picks those up too and each reports separately.
- **PAL suites.** There is no PAL timing to test (`Moon_Core.md` §7).
