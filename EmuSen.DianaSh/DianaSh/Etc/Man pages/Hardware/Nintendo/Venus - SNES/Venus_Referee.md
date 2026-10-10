# Venus — the referee: an FPGA implementation read against the whole core

*Written 2026-09-20. Not a slice: no rule of Venus changed, no test was added and no code was touched. This
page carries all six of Venus's subsystems to a third implementation — the SNES_MiSTer core's VHDL — and
writes its answer beside Venus's and Mesen's.*

***§0 decides what the rest is worth, and its answer is the opposite of the N64 pass's.** `Mars_RdpReferee.md`
had to discount most of its referee's agreement, because that core's rasteriser carries angrylion's own
identifiers. This core mostly does not: its 65C816 is transcribed from the WDC datasheet, its SPC700 and PPU
are the author's own hardware shapes, and where it *does* borrow, the borrowing is visible and local. So here
**agreement is usually real support**, which changes what a disagreement means — and there are thirty-one of
them.*

---

## 0. What the referee is, and what its answers are worth

**The checkout.** `github.com/MiSTer-devel/SNES_MiSTer`, branch `master`, head `c61bfd4` (2026-09-17), cloned
to `~/Projects/snes-mister-reference`. About 25,000 lines of VHDL and Verilog in `rtl/`, by srg320. It is
synthesisable register-transfer logic that runs commercial cartridges in real time on an FPGA; it is **not** a
dump, a netlist or a decapping. No public dump of any SNES chip exists.

**Provenance, subsystem by subsystem.** This is the part that sets the weighting, and it had to be established
before any finding could be read:

| Subsystem | Lineage | What agreement is worth |
|---|---|---|
| 65C816 (`rtl/65C816/`) | WDC W65C816S **datasheet**: pin-level names (`VPA`, `VDA`, `VPB`, `MLB`, `ABORT_N`) no emulator carries; microcode rows are a generated cycle table | **Real support** |
| SPC700 (`rtl/SPC700/`) | The author's own microcoded datapath, same record shapes as his 65C816; DIV is a 9-iteration bit-serial divider, not bsnes's closed form | **Real support** |
| S-CPU system side (`CPU.vhd`) | Nintendo/fullsnes register mnemonics, two hand-written DMA/HDMA state machines | **Real support**, except the math unit (below) |
| — its multiply/divide unit | Literally bsnes's/Mesen's ALU algorithm | Nearly worthless |
| PPU (`PPU.vhd`) | anomie/fullsnes *register* documentation, hardware-shaped pipelines (fetch-slot table, 34-slot range pipeline, one shared 5-bit adder); cites fullsnes outright at §1190 | **Real support**, except the two mode-7 quirks it shares with bsnes |
| S-DSP (`DSP.vhd`) | Plumbing is the author's own 32-step schedule; **the arithmetic is blargg's**, expression for expression (BRR filter decomposition, the KON delay, the decay formula, the echo FIR's wrap) | **Weak** on formulas, **real** on the rate counter, which is independently derived |
| SA-1 (`chip/SA1/`) | Official Nintendo register mnemonics (shared with bsnes), but the structure — clock phasing, the bit-stream prefetch, the character-conversion machine — is the author's own; the divider is credited to a named third author | **Real support** on bit numbers and structure |
| GSU (`chip/GSU/`) | Instruction core is the author's own 25-entry microcode; **the pixel-cache shape is bsnes's**; the S-CPU ROM-lockout byte pattern is byte-for-byte Mesen's | Mixed — stated per finding |
| NEC DSP (`chip/DSP/DSPn.vhd`) | The author's own datapath over the same public opcode decode everyone uses | **Real support**, and it is the *only* second opinion — see §5.3 |
| Cartridge detection | **Not in this repo.** The HPS loader (`Main_MiSTer/support/snes/snes.cpp`) scores the header; that function is byuu's bsnes heuristic **with his comments verbatim** | Worthless |

**So the reading rule here is the inverse of the N64 pass's.** On the N64, agreement was cheap and
disagreement was the signal. Here most of the core was arrived at rather than copied, so agreement is
evidence — and the places where agreement is *not* evidence are few, named above, and mostly confined to the
S-DSP's formulas and the header scorer.

**What this pass cannot do.** It cannot measure a console. Nothing below is a hardware result. Every answer is
one more opinion, from a source with the advantage of having to satisfy real cartridges on real silicon in
real time, and the disadvantage of having been written by reading the same public documentation Venus was.

## 1. Method

Seven readings were run in parallel, one per subsystem, each asked for the same five things: the RTL's
provenance, the disagreements with `file:line` on both sides, what the RTL does not implement, what Venus does
not implement, and — required, not optional — the rules checked and found in agreement. Mesen 2
(`~/Projects/mesen-reference`) was consulted as the tie-break third reading throughout, because it is what
Venus's differential already grades against.

**Every finding below carries a verification mark, and the marks are not decoration.**

- **[V]** — the claim about *Venus's own code* was re-read at the source by the author of this page and holds.
  Nineteen findings carry it.
- **[R]** — reported by a reading and not independently re-read here. Treat as a lead, not a result.

That distinction was forced by the method, not chosen for rigour's sake: **two of the seven readings cited
Venus line numbers that do not exist** — `Sa1Dma.cs:279` in a 112-line file, `Sa1BitStream.cs:210` in a
64-line file, `Input/Input.cs:123-144` in a 102-line file. In each case the *rule* described was correct and
the real code was a short grep away, and in each case the RTL and Mesen citations checked out. The failure
mode is specific and worth recording: a reader that has read both trees can describe a behaviour accurately
while anchoring it to a line it half-remembers. **Every `[V]` citation below was re-derived from the file;
every `[R]` citation is the reader's and may be off by lines.**

## 2. The findings that matter most: both references agree against Venus

These are not referee disputes. In each, Mesen and the RTL — independently derived from each other in every
case marked "real support" in §0 — say the same thing, and Venus says something else. They are the strongest
class this pass produced, and four of them are defects of a kind that reading alone can establish, because
they are internal inconsistencies or dead code rather than judgements about hardware.

### 2.1 Two SPC700 absolute loads cost five cycles instead of four **[V]**

`MOV A,!a` (`$E5`) and `MOV Y,!a` (`$EC`) are 5 in `Spc700.OpcodeTable.cs:69-70`; `MOV X,!a` (`$E9`) is 4 at
`:188`. In the RTL's microcode all three have exactly four active rows — `MCode.vhd:3915`, `:3983`, `:4034`,
the remainder of each block being don't-care padding. Mesen builds all three from one `Addr_Abs()`, so they
cannot differ there either. The stores at 5 (`$C5`/`$CC`/`$C9`) are right; the loads are not.

**This is the exact signature of the `CMP Y,dp` defect that `Venus_APU.md` §1.7 records as found and fixed:**
one opcode in a family carrying a cost its siblings do not. That the same shape survived in a second family
suggests the §1.7 fix was applied where the symptom was, not swept across the table.

**Why the corpus did not catch it.** `Validation/Spc700SingleStepTarget.cs:101-117` parses only `initial` and
`final` from each TomHarte case and discards the per-cycle `cycles` array. The corpus therefore validates
every architectural result and no timing at all — which is worth stating plainly in `Venus_APU.md` §2.6,
because "255,106 of 256,000 pass" reads like a timing result and is not one.

### 2.2 The SA-1's `$2209` vector-select bits are in the wrong positions **[V]**

`Sa1.Memory.cs:50-51` gates the NMI override on bit 5 and the IRQ override on bit 4. Both references use
**bit 4 for NMI and bit 6 for IRQ** — Mesen at `Sa1.cpp:87-88` feeding `Sa1VectorHandler.h:24`, the RTL at
`SA1.vhd:1308-1309` feeding the `$00FFEA`/`$00FFEE` substitutions at `:1437-1447`.

A cartridge that sets bit 4 to redirect the S-CPU's **NMI** gets its **IRQ** vector redirected under Venus and
its NMI left pointing at the ROM vector. The failure is not subtle — it is a wrong vector on the first
interrupt of that kind. `Venus_SA1.md` §4.3 states bits 5/4 as fact, so the doc is wrong here too.

### 2.3 The SA-1's DMA destination is decoded from the wrong bit **[V]**

`Sa1Dma.cs:34` reads `_control & 0x08`; both references read bit 2 (Mesen `Sa1.cpp:161`, RTL `SA1.vhd:814`).
A normal DMA aimed at BW-RAM is therefore treated as an I-RAM transfer, which both mis-triggers it (on the
`$2236` write rather than `$2237`, per `Sa1Dma.cs:58-59`) and lands it in the 2KB the SA-1 keeps its stack and
direct page in. `Venus_SA1.md` §8 repeats "the destination in bit 3", so again the doc carries the error.

### 2.4 HDMA transfer mode 6 is four bytes wide **[V]**

`Dma.cs:143` gives mode 6 the pattern `{0,0,0,0}`. The RTL's `DMA_TRMODE_LEN` (`CPU.vhd:192`) encodes index 6
as `"01"` — two steps — and Mesen's `_transferByteCount` (`SnesDmaController.cpp:8`) is `{1,2,2,4,4,4,2,4}`.
Harmless for general-purpose DMA, where the same byte repeats; **wrong for HDMA**, where a mode-6 channel
consumes twice its table bytes per line and desynchronises the table from the first line onward.

### 2.5 The sprite time-over flag can never be set **[V]**

`Renderer.Sprites.cs:99-104` breaks the sliver loop at `sliversUsed >= 34`, so the test at `:133`
(`sliversUsed > 34`) is unreachable and `$213E` bit 7 is always 0. Range-over at `:53` fires when exactly 32
sprites are in range, where both references require a 33rd. `Venus_PPU.md` §6.1 claims both limits are
reproduced; one is off by one and the other is dead code.

Two further sprite-budget rules go the same way **[V]**: range evaluation at `:38-53` tests only Y, so a
sprite parked off-screen horizontally still consumes one of the 32 (both references exclude it, except at
X=−256), and the 34-sliver budget at `:97` counts tile columns that are off-screen, which both references
skip. The symptom of all three together is the same — visible sprites dropped on busy lines — and it is
exactly the class of bug that `sprites` was built to inspect.

### 2.6 Colour math with "half" clamps before halving **[V]**

`Renderer.cs:190-197` computes `Math.Min(255, main + sub)` and then `/2`. The RTL halves *instead of*
clamping, on 5-bit channels (`PPU_PKG.vhd:137-143`); Mesen computes `min((a+b)>>half, 31)`. For any pixel
whose channel sum saturates, Venus renders **half brightness where hardware renders full** — additive
translucency (water, fire, ghosts, spell effects) comes out markedly too dark.

### 2.7 ENDX is never set when a sample loops **[V]**

`DspVoice.cs:145-151` sets `Ended` only in the end-**without**-loop branch. Both references set the flag
whenever the end bit appears in a BRR header, loop or not (RTL `DSP.vhd:1024-1026`, Mesen
`DspVoice.cpp:317-323`). A driver that polls `$7C` to double-buffer streamed BRR never sees the flag.
`Venus_APU.md` §3.2 states Venus's rule as if it were hardware's.

Beside it, two more S-DSP host-visible gaps **[V]**: `$x8`/`$x9` return the last byte written rather than live
ENVX/OUTX (`SDsp.cs:96-110` special-cases only `$7C`), and FLG bit 7 (soft reset) is ignored entirely — only
the mute bit is read, at `:228`. A driver that silences everything by setting FLG bit 7 at a song change
keeps playing.

### 2.8 VRAM writes are not blocked during active display **[V]**

`Ppu.Registers.cs:128-145` writes unconditionally. Both references drop the write outside vblank and forced
blank (RTL `PPU.vhd:777-781`, Mesen `SnesPpu.cpp:2046-2068`). A game whose VRAM DMA overruns vblank corrupts
VRAM in Venus and does not on hardware. Related **[V]**: `$213B` (`Ppu.Registers.cs:284`) has neither the
high/low toggle nor the address increment that the write path at `:152` does have.

### 2.9 Clearing GO from the S-CPU leaves the GSU's cache stale **[V]**

`SuperFx.Registers.cs:74-79` clears only the prefix registers. Both references also clear CBR and invalidate
the code cache (RTL `GSU.vhd:546-551`, Mesen `Gsu.cpp:578-581`). A game that aborts the GSU and restarts it
without issuing `CACHE` executes stale cache lines.

## 3. Where the referee stands alone, and what that is worth

Findings the RTL raises where Mesen sides with Venus, or where Mesen implements nothing. These are weaker —
one implementation against two — but §0 says this core's agreement is usually earned, so they are not nothing.

- **Interrupt entry costs Venus no time at all [V].** `Cpu.Step()` (`Cpu.cs:169`) returns an instruction's
  master clocks, but `Nmi()` (`:242`) and `Irq()` (`:287`) are called from `VenusCore.cs:251` *outside* that
  accounting and contribute nothing. Hardware spends 8 cycles natively, 7 in emulation mode. Negligible per
  NMI; a per-scanline H-IRQ game loses on the order of 4% of a frame's clocks. Mesen charges it too, so this
  is really a §2-class finding that only the timing model hides.
- **A masked timer IRQ is dropped rather than held [V].** `Cpu.Irq()` returns false when `I` is set
  (`Cpu.cs:291`) and `VenusCore.cs:251,256` discards the result; nothing retries. Both references hold the
  line level until `$4211` is read. **The refinement the readings missed:** the SA-1 and SuperFX paths at
  `VenusCore.cs:324,336` *are* re-polled every scanline, so coprocessor IRQs effectively retry and only the
  H/V timer's is lost. A V-IRQ that comes due inside an NMI handler is gone for the frame.
- **HDMA costs no CPU time [V].** `PendingCpuCycles` is touched only by general-purpose DMA (`Dma.cs:227`).
  The RTL holds the 65816 off for every HDMA step; Mesen charges 8 per channel plus 8 per byte. With four to
  eight active channels a real machine loses several hundred master clocks per line and Venus gives all of it
  to the CPU — the same class of error `Venus_CPU.md` §8.7 chased for DRAM refresh, and a plausible
  contributor to the residual it left open.
- **`$4016`/`$4017` return zero in the upper bits [V]** (`Input.cs:71-92`), where both references return open
  bus in bits 7-2 for `$4016` and the peripheral `111` in bits 4-2 for `$4017`.
- **`HTIME` is still not compared [R].** `VenusCore.cs:246-258` raises an H-IRQ at every scanline start
  regardless of HTIME, and the both-enabled case falls into the V-only branch. Already recorded in
  `Venus_CPU.md` §8.5d and in project memory; the referee confirms the rule Venus is missing
  (`CPU.vhd:556-577`). Mid-line raster splits fire at H=0, up to a full line early.
- **Reads of `$F0`/`$F1`/`$FA`-`$FC` return the last value written [R]**, where both references return 0. A
  driver doing a read-modify-write on `$F1` would, on hardware, read 0 and therefore clear the timer-enable
  and IPL bits it meant to preserve.
- **The GSU pixel cache is keyed and addressed differently [R]:** Venus computes the target address when a
  pixel is plotted, both references at flush time from the live `SCBR`/`SCMR`/`POR`. A game that changes the
  buffer base with pixels pending writes up to 8 of them to the wrong buffer.
- **Clip-to-black is not implemented, and CGWSEL bit 6 is misused [R].** Venus reads bits 7-6 as colour-window
  inverters (`Renderer.Scanline.cs:361-363`); both references use them to force the main colour to black in
  four modes. Iris and spotlight effects and `CGWSEL=$C0` fades render wrong twice over.
- **Brightness is applied before colour math rather than after [R]**, so additive saturation happens at the
  wrong point during a fade. (The brightness *curve* is a genuine three-way split: Venus and Mesen use
  `b/15`, the RTL `(b+1)/16`.)

## 4. Where Venus and the referee agree against Mesen

Worth recording because Venus's differential grades against Mesen, so these are the places where matching the
grader would be the wrong move:

- **`COLOR`/`GETC` high-nibble handling [R].** Venus's deliberate bsnes reading (`SuperFx.Plot.cs`) expands to
  the RTL's expression over all four `POR` bit-2/3 combinations; Mesen keeps the old COLR high nibble in the
  HN=1/FH=0 case. `Venus_SuperFX.md` §9 lists this as open — it can now be closed in Venus's favour, as a
  reading, and the fork recorded as Mesen's.
- **The GSU cache window is indexed absolutely**, not relative to CBR, in both Venus and the RTL; Mesen and
  bsnes add CBR **[R]**. A real three-way with no hardware to settle it.
- **`$2137` latches on any read**, ungated by `$4201` bit 7, in both Venus and the RTL; Mesen gates it **[R]**.
- **An S-CPU write to R14 does not start a ROM fetch** in either Venus or the RTL; Mesen starts one **[R]**.
- **Hi-res interleave parity:** the RTL emits main on even dots and sub on odd, agreeing with Venus against
  Mesen **[R]**. This is recorded for completeness only — the parity decision is settled and is not to be
  revisited without hardware evidence.

## 5. The disputes this pass could not settle

Three-way splits, or two-way splits where neither side is hardware. Recorded so that nobody re-derives them.

### 5.1 SA-1 division semantics

Venus floors toward −∞ and returns dividend-as-remainder on divide-by-zero; Mesen floors and returns 0; the
RTL truncates toward zero, returns `|numer| mod denom`, and answers `$FFFF` on divide-by-zero. Three
implementations, three answers. Only a hardware test ROM settles it.

### 5.2 The GSU's `ALT` prefix and the `B` (WITH) flag

Venus and Mesen clear `B` on an `ALT` prefix; the RTL leaves it set (`GSU.vhd:685-693`), which makes
`WITH Rn : ALT1 : TO Rm` a `MOVE` there and a `TO` prefix here. `Venus_SuperFX.md` §4.2/§9 already frames
this correctly: a game that emits that byte pattern decides it. The referee is a second implementation on the
other side, not evidence — **the prediction registered in §9 stands unretired.**

### 5.3 The NEC DSP's `S1`/`OV1` after an overflow

The most consequential unsettled item on the page, and the one worth real effort. Venus's ALU is line-for-line
Mesen's, itself bsnes-derived, so **Venus and Mesen are one reading, not two**, and the RTL is the only second
opinion anywhere. They are incompatible: on a first overflow Venus sets `S1 = S0` and the RTL sets `S1 = ~S0`
(`DSPn.vhd:241-252`) — opposite — and after a non-overflowing operation the RTL keeps a stale `S1` where Venus
refreshes it. Both feed `SGN`, through which every clamped DSP-1 coordinate passes, and both feed the
`JS1`/`JOV1` branch conditions. `Venus_NecDSP.md` §5.2/§10.2 already flags this region as the most
error-prone in the chip and records no disagreement; it should now record one. Settling it needs a real
DSP-1B, not another emulator.

## 6. What each side does not implement

**The referee's own holes**, in its own voice: `$4213` RDIO is hardcoded to `$00`; the S-CPU-side writes to
the SA-1's `$220C-$220F` are accepted only because "some SMW hacks write inerrupt vectors here"
(`SA1.vhd:1248`, misspelling in the original) — a deliberate compatibility hack against the documentation; the
µPD96050's program counter is 11 bits, so the ST010's upper 8K is unreachable and the **ST011 is absent
entirely**; serial-condition branches decode to never-taken; the GSU's transparency test keys on the wrong
screen-mode bit; mosaic never restarts on a `$2106` write, the one place Mesen carries a game-driven
correction the RTL lacks. Its `TURBO`/`FASTROM` paths are MiSTer features, not hardware, and must never be
read as timing evidence.

**Venus's gaps that the referee has**, beyond §2 and §3: the SA-1's character-conversion DMA (both types),
BW-RAM banks `$50-$5F` and the `$60-$6F` bitmap banks, and BW-RAM/I-RAM write protection — all already
recorded in `Venus_SA1.md` §3.4/§8.1; the S-DSP's noise and pitch modulation (`Venus_APU.md` §4.1); the GSU's
RON/RAN arbitration and SFR bit 6 (`Venus_SuperFX.md` §2.1/§5.2); PPU interlace, overscan, and windows and
colour math inside the hi-res modes (`Venus_PPU.md` §8); the short scanline on V=240 of an odd
non-interlaced field; `$4201` bit 7 driving the counter latch (`Venus_PPU.md` §9).

**Chips the referee has and Venus has none of:** CX4, S-DD1, SPC7110, S-RTC, BS-X, Sufami Turbo, MSU-1, and
the ExHiROM mapping. `Venus_Memory.md` §2 already records all of these as unimplemented, so this pass adds
nothing but a confirmation that the list is complete and still accurate.

## 7. What the pass corroborated

Negative results, which in a comparison are half the product. The seven readings checked several hundred
rules and found the great majority in agreement. The ones worth naming:

- **The S-DSP's Gaussian table is byte-identical across all three implementations**, all 512 entries, with the
  same index mapping, the same three-term 16-bit wrap, the same final clamp and `& ~1`. So is the BRR decoder
  in every particular — shift 13-15 collapsing, all four filter coefficients, the clamp-then-double-with-wrap,
  the 12-entry ring. The envelope rate table and every GAIN mode match, which **closes `Venus_APU.md` §4.5's
  "not independently cross-checked" note on `(Gain & 0x7F) * 16`**: it now has two independent confirmations.
- **The SPC700's IPL ROM is byte-identical**, all 64 bytes, and 252 of 256 opcode cycle counts agree with the
  microcode — every conditional branch's taken/not-taken pair included.
- **`DIV YA,X` agrees over all 16,777,216 input pairs**, quotient, remainder and V flag, between Venus's
  closed form and the RTL's 9-iteration bit-serial divider. A reading produced an exhaustive result here
  because the RTL's algorithm is small enough to evaluate.
- **The OBC1 is a full match on all three implementations**, including the 2-bit merge on `$1FF4` writes and
  the base/index address arithmetic — and the RTL's expression is independently derived, which upgrades what
  was previously Venus-agrees-with-Mesen-because-Venus-copied-Mesen into real support.
- **A 16-bit GSU RAM access pairs `addr` with `addr xor 1`** in the RTL — an independent confirmation of
  `Venus_SuperFX.md` §5.1a.
- On the PPU: the two-latch scroll formula, VMAIN address rotation in all three modes, the per-mode BG/OBJ
  priority interleave including mode 1's BG3 bit and mode 7 EXTBG, the full window logic including the colour
  window's wiring, the direct-colour bit layout, mode 7's screen-over modes and flips, and the OBJ size table
  including the undocumented 16x32 and 32x64.
- On the S-CPU: the memory-speed table cell for cell, including the rule that FastROM never speeds up banks
  `$00-$3F`; DRAM refresh at 40 clocks near H≈538; the WRAM/`$2180` bus conflict in **both** directions; DMA
  A-bus blocking with its bank sensitivity; and the HDMA line-counter's repeat bit and reload arithmetic.

## 8. The witness list

A distinctive property of this referee: **its commit messages name the game or test ROM that forced each
fix.** That makes its history a checklist of rules that mattered enough to someone that a cartridge misbehaved
without them — a different and more practical axis than reading the code. The full extract is not reproduced
here; the ones bearing on rules Venus models are:

| Rule the author had to fix | Witness |
|---|---|
| NMI delayed one cycle after DMA | Battletoads in Battlemaniacs (intro) |
| Interrupt delay after DMA, and the IRQ path twice more | `emudetect` |
| HDMA disabled mid-line | Weaponlord |
| Reset timings | Kawasaki Superbike Challenge |
| SMP input ports latched on the falling edge of `PAWR_N` | Kawasaki Superbike Challenge |
| SMP I/O timings | Rendering Ranger |
| BG fetch during vblank | Pocky & Rocky (title) |
| Mode 7 parameter latch timing (twice) | Super Mario World glitch, Mode 7 tests |
| Mode 7 calculation | Tiny Toon Adventures: Wacky Sports Challenge |
| BRR decoder | 240p Test Suite (MDFourier) |
| `DIV` instruction | gilyon's test |
| SPC700 CPU behaviour | PeterLemon's test ROMs |
| GSU ROM preload | Doom |
| `SCBR` writable while the GSU runs | Dirt Trax FX |
| DSP `DR` written by both sides at once | Top Gear 3000 |

Several of these are rules §2 and §3 say Venus does not model. **Battletoads, Weaponlord, Pocky & Rocky and
Dirt Trax FX are therefore candidate test cases with a known-sensitive behaviour**, which is more than most
candidate games offer.

## 9. Corrections this pass owes the other pages

Recorded here rather than applied, so that each lands in a commit with its own evidence:

1. `Venus_SA1.md` §4.3 — `$2209`'s vector-select bits are 4 (NMI) and 6 (IRQ), not 5 and 4 (§2.2).
2. `Venus_SA1.md` §8 — the DMA destination is bit 2, not bit 3 (§2.3); and normal-DMA completion should not
   raise the S-CPU's flag **[R]**.
3. `Venus_SA1.md` §236 — the closing sentence still says SuperFX and the DSPs "remain unimplemented". Three
   coprocessors have shipped since. It is a stale prediction and should be retired in place, not deleted.
4. `Venus_PPU.md` §6.1 — the sprite range and time limits are not reproduced (§2.5).
5. `Venus_APU.md` §3.2 — ENDX is set on a looping end block too (§2.7).
6. `Venus_APU.md` §2.6 — say plainly that the SPC700 corpus validates no timing, because the target discards
   the per-cycle array (§2.1).
7. `Venus_APU.md` §260 — says the echo is "not present at all"; `SDsp.ProcessEcho` has existed since
   2026-07-25 **[R]**.
8. `Venus_Memory.md` §2.2 — the SRAM clamp gloss says 512KB where the code clamps at exponent 7, i.e. 128KB
   **[R]**.
9. `Venus_SuperFX.md` §9 — the `COLOR`/`GETC` item can be closed in Venus's favour (§4).
10. `Venus_NecDSP.md` §5.2 — record the `S1`/`OV1` disagreement, and that Venus and Mesen are one reading
    there, not two (§5.3).

## 10. What this pass did not establish

It did not run a single cycle of either implementation. Every finding is a reading, and the `[R]` ones are a
reading of a reading. Nothing here is graded, nothing is measured, and no game was observed misbehaving —
§2's findings are defect *candidates*, ranked by how little interpretation stands between the code and the
claim, and the four that rest on internal inconsistency or dead code (§2.1, §2.4, §2.5, §2.8) are the only
ones a reading can carry most of the way.

The natural next step is not more reading. It is the harness: §2.1, §2.4, §2.5 and §2.7 are each a short
WiseMan test, and §2.2 and §2.3 are each a unit test over a register write. Until those exist, this page is a
list of things to check, not a list of things that are wrong.
