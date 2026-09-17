# Mars — the test oracle, and what the machine has to do before the corpus will speak

*Written 2026-09-15, from a research pass plus direct reading of the corpus's own
source. `Mars_Gameplan.md` §3 rested the entire phase order on an unverified
assumption — that an N64 test ROM can report a text verdict headlessly, the way a
Game Boy one reports over the serial port. **The assumption holds.** It arrived with
three corrections and two landmines, and the landmines are the reason this is a
page rather than a line struck out of the plan.*

*Provenance is marked throughout: **[read]** means the claim was checked against the
corpus's own source in this session, **[reported]** means it comes from the research
pass and was not independently confirmed here.*

---

## 1. The corpus, and the repository that is no longer the corpus

`n64-systemtest` is the N64's blargg: a self-validating ROM that asserts against
real-silicon behaviour and reports per-assertion failures as text. **Its repository
moved.** `lemmy-64/n64-systemtest` is deprecated, and its final commit is a README
saying so; the successor is `thelemmy/nemu64-test`, still MIT, still the same crate
name and ROM filename, and active — HEAD `9a8b9f7`, 2026-09-08. **[read]**

Anything written about "n64-systemtest" before mid-2026 is about the old repository.
This project targets the new one.

**Coverage**, counted over its test registrations **[reported]**: roughly 1,125
registrations expanding to several thousand executed cases, weighted heavily toward
the CPU — arithmetic 396, RSP 243 (the whole vector unit, including the reciprocal
family), COP1 88, cache 53, TLB 57 across two sets, COP0 44, exceptions and
privilege around 80 between several groups, with smaller sets for SP and cart DMA,
PIF memory, MI and RDRAM registers.

**What it does not cover, which decides how much of Mars it can grade:** there is no
meaningful RDP rasterization coverage (a handful of registrations behind a feature
flag its own authors mark as not stable on hardware), and nothing for VI, AI or SI.
**Phases A, B and C have an oracle. Phases D and E do not get one from here** — §5.

## 2. The verdict channel

The ROM has **three** text backends and takes the first that answers, in this order:
**Emux → ISViewer → SC64 USB → silence**. **[read]** Only the second is what the
older documentation describes, and the third is flashcart hardware, irrelevant to a
headless emulator.

### 2.1 ISViewer, as the ROM actually drives it

Verified against `src/isviewer.rs` **[read]**:

| Address | Role |
|---|---|
| `0xB3FF_0014` | length register; a word write of N means "emit N bytes" |
| `0xB3FF_0020` | 512-byte text buffer |

`0xB3FF_xxxx` is the uncached window onto physical `0x13FF_xxxx`, inside the PI
cartridge address space. The ROM packs bytes big-endian into 32-bit words and issues
**word** stores — not byte stores, whatever the documentation says — polling the PI's
`io_busy` between each one, then writes the byte count to the length register. Output
longer than 512 bytes is chunked.

**Detection is a read-back**: the ROM writes `0x12345678` to the buffer base and
reads it back, and treats a mismatch as "no ISViewer present". **[read]**

### 2.2 Landmine one: a write-only stub fails silently

Because detection is a read-back, an ISViewer implemented as a write-only sink — the
obvious first implementation, and the shape our Game Boy serial sink has — **fails
detection and produces no output at all.** Not an error, not a warning: the ROM falls
through to SC64, then to silence, and runs to completion having printed nothing.

The region must be **readable RAM**. This is the single cheapest way to lose a day
on this subsystem.

### 2.3 Landmine two: the ROM executes Emux opcodes before anything else

`main()` calls `xioctl_fast()` **unconditionally**, and the panic handler calls
`xioctl_exit()` unconditionally. **[read]** These are raw-encoded COP0 CO-format
instructions in the `funct` range `0x20`–`0x3F`, which the Emux specification
reserves on the grounds that real hardware ignores them.

**If Mars raises a Reserved Instruction exception on COP0 CO `funct` `0x20`–`0x3F`,
the ROM traps on the first instruction of `main()`** and nothing else in this
document ever gets a chance to matter. That range must be a no-op unless Emux is
implemented.

### 2.4 Emux is the better channel, and the ecosystem is moving to it

The Emux specification (`n64brew.dev/wiki/Emux`, CC0 **[reported]**) defines
emulator extensions on those same unused COP0 opcodes. Three matter here: `xdetect`
returns a support bitmask — an emulator that no-ops the range reports zero and the
ROM safely falls through to ISViewer — `xlog` emits a string without touching the PI
bus at all, and `xioctl` carries `fast` and **`exit`**.

`exit` is the interesting one. The ROM calls it at the end of `main()` **[read]**,
which means **the ROM tells the harness when the run is over** instead of the harness
guessing a frame budget and hoping. Mercury's corpus never offered that; its harness
runs a fixed number of frames and reads what accumulated.

libdragon has deprecated its ISViewer logging in favour of the Emux channel
**[reported]**, so ISViewer is the installed base and Emux is the direction. Mars
should implement ISViewer first — it is what proves Phase A — and Emux shortly
after, for `exit` rather than for the logging.

### 2.5 What to parse

The summary format is not what the old README documents. Current form **[reported]**:

```
Finished in 12.34s. Base: Failed 3 of 3650 tests (99% success rate)
```

one line per enabled category, preceded as they happen by per-failure lines of the
form `Test '<name>' failed: <error>` or `… failed with exception: <Exception>`. A run
that executed nothing says so explicitly. Treat a nonzero failure count **or the
absence of a summary line** as failure — the second case is what a silent channel
looks like, and §2.2 is how it happens.

## 3. What Mars must do before the ROM will talk

In dependency order. None of this is Phase A's emulation proper; it is the cost of
admission to being graded at all.

1. **The boot handoff.** Copy the ROM's first `0x1000` bytes into SP DMEM and enter
   at `0xA4000040`. The corpus carries libdragon's open-source IPL3, so **no
   Nintendo PIF ROM or IPL3 dump is required** **[reported]** — which means the
   gameplan's HLE-boot decision (§4.1) is not merely defensible but is the path the
   corpus is built for.
2. **ISViewer as readable RAM** at physical `0x13FF_0000`, 4 KiB is enough, with the
   length register at `+0x14` and the buffer at `+0x20` (§2.1, §2.2).
3. **COP0 CO `funct` `0x20`–`0x3F` as no-ops** (§2.3).
4. **Three behaviours libdragon's IPL3 depends on** **[reported]**, each of which a
   young emulator typically gets wrong: a nonzero `RI_SELECT` lets it skip RDRAM
   initialisation; RSP DMA reading RDRAM above 8 MiB must return **zeros** rather
   than mirroring; and RSP DMA from IMEM longer than 4 KiB must **wrap within IMEM**
   rather than spilling into DMEM.

## 4. Acquiring it

The ROM is **built from source**, not downloaded from a release — neither repository
publishes releases **[reported]**. Two routes:

- **CI artifact**, which is how the copy in hand was obtained: 2,624,168 bytes, md5
  `addfe2727510ac2be28a6e98d833f96d`, header `80 37 12 40` (big-endian `.z64`), entry
  point `0x8000_6CF0`. All four verified locally **[read]**. Artifacts expire, so a
  mirrored copy is the point.
- **From source**, needing only rustup and `nust64` (pin `0.4.1`) — a pinned nightly
  and a custom MIPS target, **no MIPS GCC and no N64 SDK** **[reported]**.

**The copy in hand is at `TestRoms/n64/n64-systemtest.z64`**, which `git
check-ignore` confirms is ignored. It is MIT-licensed homebrew built from public
source, so it is not a commercial ROM and the repo rule that forbids committing
those is not in tension with keeping it — but `TestRoms/` is ignored wholesale and
it stays that way. Note that `HardwareTestRomLibrary` resolves its corpus under the
DianaOS sandbox's install directory rather than this folder; wiring N64 discovery is
Phase 0's job and this file is parked, not plumbed.

## 5. Phases D and E have no ROM oracle, and the RDP has no hardware oracle at all

This is the finding that most changes the plan's shape, and it is worse than §1's
coverage gap suggests.

> **Correction 2026-09-17: the corpus does grade part of Phase D.** Six groups test the display
> processor's command registers and its stream at the corpus's `RDPBasic` level, not behind the
> experimental flag §1 mentions, and they are graded against silicon (`Mars_Rdp.md` §0). What
> this section says about rasterization is unchanged: those six read one pixel of a fill, and
> nothing here grades an edge.
>
> **Update 2026-09-17: the differential this section describes now runs** against angrylion, with
> parallel-rdp as a cross-check, on dumps written by the test itself rather than by an emulator
> (`Mars_RdpDifferential.md`). This section's distinction survives its first use intact, and
> sharpened: agreement with angrylion turned out to include agreement with a validation workaround
> in its fork, and the two references disagreed on a case.

**For the RDP, the differential infrastructure is excellent and its ground truth is
not hardware.** `parallel-rdp` (MIT) defines an interchange format — `RDPDUMP2`: an
eight-byte magic, RDRAM and hidden-RDRAM sizes, then tagged records for commands, DRAM
updates, hidden-DRAM updates, VI register writes and frame boundaries **[reported]**.
Several emulators already emit it, and the same repository carries a headless
replay-and-diff tool plus roughly 150 generated conformance tests that must match
**bit-exactly in RDRAM**.

**But the reference those tests compare against is angrylion — a software
reimplementation, not silicon.** parallel-rdp's own author has said as much, noting
that running the tests against a real N64 would need someone to hook up a serial
connection first **[reported]**. So:

> **Bit-exactness against angrylion is agreement with the community's best model of
> the RDP, not with the hardware.** It is the strongest oracle available and it is
> not the same claim, and Mars's documentation must not quietly promote it.

That is precisely the distinction `Mercury_Gameplan.md` §3.2 drew when it refused to
treat gambatte as truth, and it survives here intact — with the difference that for
the Game Boy there was a better oracle available and for the RDP there is not.

**What this does to `Mars_Gameplan.md` §2.1's reopening condition.** The condition
was: if the RDP cannot be graded to pixel-exactness, LLE has lost its advantage.
Pixel-exact grading *is* available, so the condition does not fire. What has changed
is the meaning of a pass — it certifies agreement with angrylion, and the residual
risk that angrylion and hardware differ is now a known, named, unquantified quantity
rather than an assumption of correctness.

**No public golden-image corpus exists** and **no RSP dump format exists**
**[reported]**; dumps are generated by running content through a dumper-enabled
emulator, and the closest thing for the RSP is a developer fork of ares that emits a
causally-ordered RSP/RDP command log.

## 6. Negative results, recorded because they cost time to establish

- **No cross-emulator scoreboard exists.** The only figure confirmed at source is an
  ares issue reporting 1 failure in 3,650 cases on a release build, and 40 on a debug
  build owing to compiler floating-point rounding **[reported]** — which is itself a
  useful warning about Phase B.
- **ares is a poor choice for headless capture.** Its ISViewer region always detects,
  but output is discarded unless a debugger tracer is enabled in its UI **[reported]**.
  For a cross-check of our own verdict parsing, an emulator that prints ISViewer
  traffic to stdout unconditionally is the one to use.
- **Several high-ranking search results for N64 emulator projects are
  machine-authored repositories carrying self-reported perfect scores** **[reported]**.
  None of their claims were used here, and none should be.
- **Two corpora have unstated licences** — one containing third-party code used by
  permission, one a crash-condition suite **[reported]**. Neither is to be vendored.

## 7. What is still unverified

- Everything marked **[reported]** above, most consequentially the three IPL3
  behaviours in §3 and the summary text format in §2.5. Each is cheap to confirm the
  moment Phase A can execute instructions, and each should be confirmed rather than
  trusted.
- **The ROM has never been run.** Every claim here is from source reading and file
  inspection. The first real test of this page is the first time Mars prints a line
  of its output.
