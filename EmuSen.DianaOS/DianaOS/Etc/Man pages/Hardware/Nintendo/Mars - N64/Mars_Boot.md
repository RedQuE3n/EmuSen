# Mars — the handoff, and the first real ROM to run

*Phase A's last slice, landed 2026-09-15. **The hardware corpus now boots through its
own bootcode and reaches its entry point**, which is the first time Mars has executed
anything not written by this project.*

---

## 1. What the handoff does

The real machine runs a boot ROM inside the PIF, which performs a security handshake
with the cartridge and then copies the cartridge's first four kilobytes into the
signal processor's data bank and jumps into it. Mars does the copy and the jump, and
skips the handshake.

`Mars_Gameplan.md` §4.1 argued for this on three grounds: the boot ROM is
copyrighted and most users do not have it, the handshake proves nothing about the
emulated machine, and a missing firmware image must never be fatal. The corpus
settles a fourth: it carries **libdragon's open-source bootcode**, so no proprietary
image is needed to run the one thing Mars is graded against.

## 2. The register state a bootcode is entitled to find

A stack pointer in the data bank, a television-standard flag taken from the ROM
header's destination code, a seed value, and a coprocessor-zero state with the
coprocessors enabled. These are the conventional post-boot values rather than
measured ones — no document the survey found states them
(`Mars_Documentation.md` §5), and they are what emulators agree on.

That agreement is exactly the kind of evidence this project distrusts elsewhere, and
it is recorded as such: **if a game misbehaves in a way that traces back to boot
state, this section is a suspect and not a foundation.**

> **The suspicion paid out, 2026-09-18.** *Ocarina of Time* is the first game to trace a
> failure back here, and the list above was short of what its boot code needs: it reads two
> registers this section never set (`t3` and `ra`), eight words of instruction memory it never
> filled, and a seed that is right only for one family of chips. §6 is what the handoff
> now leaves and why; §8 is the evidence. The sentence above stays, because it was the right
> caution — it is simply no longer hypothetical.

## 3. What running the real bootcode found

Three things, in the order they appeared.

**The cache instruction.** The bootcode invalidates caches in a tight loop before it
does anything else. Mars models no caches, so the instruction is a no-op — every
cache operation is complete before it starts. The corpus's own cache tests will fail
for the same reason and should, which is the correct relationship between a
simplification and the tests that cover it.

**Load-linked and store-conditional.** The corpus's README offers to let an emulator
fake these as ordinary loads and stores, and says plainly that doing so is wrong.
They are implemented properly instead — the load arms a flag, the store lands only if
nothing has broken it, and an exception breaks it — because the correct version is a
dozen lines and the fake would have to be undone later with a test suite already
depending on it.

**Nothing else.** The three admission requirements the test-oracle page named were
already in place, and the bootcode exercised all three for real rather than through
our own tests: the select register short-circuit, zero reads above installed memory,
and the transfer that wraps inside the instruction bank.

## 4. Where it stops, and why that is the right place

The corpus's very first instruction after boot is a **coprocessor-1 control move** —
it configures the floating-point unit before it does anything else. Mars has no
coprocessor 1, so that is a reserved-instruction exception, and it is where execution
stops today.

That is Phase B, which the plan already places next and already argues should begin
with a software floating-point implementation rather than host doubles
(`Mars_Cpu.md` §2.1, `Mars_References.md` §5.2). The boot slice therefore ends
exactly at the phase boundary the plan drew before any of this was written, which is
a small piece of evidence that the boundaries were drawn in the right places.

Two tests hold this position: one asserts the boot reaches the entry point without
faulting, the other asserts that the first instruction Mars cannot run is a
coprocessor-1 operation. The second inverts when Phase B lands, and is meant to.

## 5. What the boot does not do

- ~~**No security handshake**, and therefore nothing that depends on its results.~~ **No
  boot handshake** — IPL2's exchange with the CIC, which checks IPL3 itself, is still skipped.
  But its *results* are not all skippable: the seed the chip hands over decides whether IPL3's
  own checksum passes (§6.2), and one chip answers challenges from the game long after boot
  (§7). Both are modelled since 2026-09-18.
- **No PIF ROM path.** The firmware seam exists project-wide (`EmuSen_Firmware.md`)
  and Mars does not use it; a real boot ROM would be an alternative entry rather than
  a rewrite.
- **No reset types.** A cold boot is the only kind.
- **The bootcode is the cartridge's**, so a ROM carrying a different one gets a
  different boot. Nothing here is specific to the corpus except that the corpus is
  what has been run.

> **Retired, 2026-09-16.** §4's "where it stops" is no longer where it stops. The
> coprocessor-1 register file landed the following day and the ROM now runs past the
> boot handoff into its own test suite, printing verdicts — `Mars_Fpu.md` §9. The
> section stays because the prediction it made was right: the stop was a phase
> boundary rather than a defect, and the phase that followed it was the one the plan
> had already placed next.

---

## 6. The cartridge's security chip, and what IPL2 leaves for it

*Added 2026-09-18, with the fix for Ocarina of Time. The code is `Rom/Cic.cs`, `Boot.cs`, one
field on `RomImage` and, for §7, `Memory/SiInterface.cs`; the grading is `MarsCicTests` and an
opt-in test in `MarsCommercialRomTests`.*

§1 said Mars does the copy and the jump and skips the handshake. That remains true of the
handshake. What it missed is that the boot code a cartridge carries — IPL3 — is not the same
program on every cartridge, and that the program on the 6105 family reads state that only the
PIF's own boot code, IPL2, could have left. Four pieces of that state matter, and one decides
which chip is present.

### 6.1 Which chip: the boot code's word sum

**The header does not name the chip; the boot code does, by being different for each.** Both
software references identify it the same way: sum IPL3's 1,008 big-endian words (`0x40`–`0xFFF`)
into a 64-bit total and look the total up (mupen64plus `device/pif/cic.c`, Project64
`N64Rom.cpp`). The two tables agree on every cartridge chip, and Mars carries those entries:

| total | chip | seed (§6.2) |
| --- | --- | --- |
| `0xD0027FDF31`, `0xCFFB631223` | 6101 | `0x3F` |
| `0xD057C85244` | 6102 / 7101 | `0x3F` |
| `0x7C56242373` | libdragon's open IPL3, treated as a 6102 | `0x3F` |
| `0xD6497E414B` | 6103 / 7103 | `0x78` |
| `0x11A49F60E96` | 6105 / 7105 | `0x91` |
| `0xD6D5BE5580` | 6106 / 7106 | `0x85` |

The libdragon row is Project64's; mupen64plus reaches the same answer by falling back to the 6102
for any total it does not list, and Mars does that too — `CicChip.Unknown` boots as a 6102. The
row matters because it is the corpus's boot code, and the corpus's verdicts are identical before
and after this change (§8.4).

**The PAL twins are not told apart, because their boot code is the same.** *Ocarina of Time
(Europe, Rev A)* carries a 7105 and sums to `0x11A49F60E96`, the 6105's total. *Super Mario 64
(Europe)* and *Wave Race 64 (USA)* both sum to `0xD057C85244`.

**The referee has no opinion here.** N64_MiSTer takes the chip type from a menu setting
(`PIF.vhd`'s `CICTYPE` port) rather than from the image. Omitted: the 64DD, Aleck64 and iQue
entries, which name hardware Mars does not model.

### 6.2 The seed

**Each chip gives IPL3 one byte, and IPL3's checksum of the game starts from it.** The 6105's
IPL3 computes `seed × 0x5D588B65 + 1` as the starting value of the checksum it runs over the
game's first megabyte (IPL3 at DMEM `0x604`–`0x644`), then compares the result with header words
`0x10` and `0x14`. A mismatch sends it into `bal .` at DMEM `0x798` — which, since that part of
IPL3 runs from a copy in RDRAM, is `0x80000248` — for ever.

**The seeds are the one fact all three references agree on independently of each other's
code:** mupen64plus `cic.c`, Project64 `Register.cpp` and `PIF.vhd` §360–380 give `0x3F`, `0x78`,
`0x91` and `0x85`. Until this change Mars handed every cartridge `0x3F`, which is why the 6102
cartridges booted and the 6105 one could not have, even had it got that far (§8).

### 6.3 `t3` and `ra`

**IPL2 runs from the signal processor's instruction memory and jumps to IPL3 through `t3`.** It
therefore leaves `t3 = 0xA4000040`, IPL3's own address, and `ra = 0xA4001550`, an address inside
IMEM. The 6105's IPL3 reads both before it writes either: its third instruction is
`lw t2, 0x44(t3)`, and once its first loop is done it reaches `bltz ra` at DMEM `0x74`, which is
taken only because a sign-extended kseg1 address is negative. The 6102's IPL3 overwrites `t3`
before reading it; it does store and reload `ra`, and the two 6102 games run as before with the new
value (§8.3).

**Both are now set for every cartridge**, because IPL2 does not know which chip is fitted when it
jumps. mupen64plus does the same; Project64 sets `t3` for every chip and gives PAL consoles
`ra = 0xA4001554`. That disagreement is recorded rather than resolved: the referee boots a real
PIF ROM the user supplies, so its source has no value for either register, and *Ocarina of Time*
boots identically with both (§8.2).

### 6.4 Instruction memory: the key

**The 6105's IPL3 opens by decrypting part of itself with the contents of IMEM.** The loop at
DMEM `0x40`–`0x60` XORs the words at DMEM `0x84` onward with the words at IMEM `0x000` onward,
writes the result back into IMEM, and stops after the first IMEM word whose low twelve bits are
zero; two more words are then copied across as they are.
What IMEM holds at that moment is IPL2 — it is where IPL2 ran — and both software references
fill in its first eight words:

```
3C0DBFC0 8DA807FC 25AD07C0 31080080 5500FFFC 3C0DBFC0 8DA80024 3C0BB000
```

The eighth is the first that ends in `000`, which is why eight is enough (a test pins this).
Decrypted with them, the result is a coherent signal-processor program: wait for the DMA queue,
fetch IPL3's RDRAM copy of itself into IMEM `0x120`, wait, and jump to IMEM `0x1B8` — which is a
routine IPL3 carries at DMEM `0x7D0`–`0x870`. IPL3 then starts the signal processor on it
(`SP_STATUS ← 0xAD`, DMEM `0x540`–`0x550`) and carries on in parallel.

**What that routine is for was not established.** Read as far as it was: it takes the signal
processor's semaphore, waits until the CPU releases it — which IPL3 does once the game's first
megabyte is in RDRAM (DMEM `0x5FC`) — DMAs four kilobytes of the loaded game into DMEM, sums them
with `VADDC`, then writes DMA registers with values whose purpose is not clear, and breaks. Under
Mars it now runs to its `break` in 1,088 CPU steps.

**The key is not load-bearing for Ocarina of Time's boot on Mars**, and this is a measured
negative result rather than a guess: with `t3`, `ra` and the seed right and IMEM left empty, the
signal processor runs undecrypted bytes for 7.1 million steps, until IPL3 halts it after the
checksum (DMEM `0x6F0`), and the game boots anyway — reaching its entry point 56 steps *sooner*,
which is the seven passes of the decryption loop it did not make (§8.2). It is set because the boot code
is written against it and every reference sets it. What that result is *not* evidence for is that
nothing later in the game reads what the routine leaves behind.

**Where the references disagree:** Project64 gives the second word as `0xBDA807FC` on a PAL console.
Decrypted with that value, the program's second instruction becomes `addiu zero, s1, -2`, a no-op,
in place of the DMA-queue wait `bnez s1, 0`; the rest is unchanged, and the game boots identically.
Mars uses the value that decrypts to a coherent instruction, for every region — which is evidence
that IPL3 was *written against* that word, not that a PAL console holds it. Project64 also gives
most other registers values that look captured from a console (`a2 = 0xA4001F0C`,
`a3 = 0xA4001F08`, and per-chip values in `at`, `v0`–`a1`, `t4`–`t7`, `t8`, `t9`), and `s7 = 6` on PAL
where the referee's version bit gives 0; Mars sets none of them, and nothing observed depends on
them.

### 6.5 Two things the handoff still does not leave

- **PIF RAM is zero**, including `0x24`–`0x27`, where the referee and mupen64plus put the seed and
  the reset type at power-on. Reading mupen64plus's list of the steps its HLE boot skips, IPL2's
  last act is to ask the PIF to clear its RAM (bit 6 of `0x3F`), which `PIF.vhd`'s `CLEARRAM` does
  without rewriting those bytes — so zero is probably what IPL3 finds. That is an inference from
  comments and RTL, not an observation.
- **`osMemSize` is zero until the game sets it.** RDRAM's select register is stubbed nonzero
  (`Mars_TestOracle.md` §3), so both IPL3s take their warm-boot path and skip the memory sizing.
  The 6102's then writes nothing to `0x80000318`; the 6105's copies `0x800003F0` there, and nothing
  has written `0x800003F0`, so it copies 0. *Ocarina of Time* is unaffected — its first function
  after the entry point stores `osGetMemSize()`'s `0x00400000` over it (`0x800004E4`, twelve
  thousand steps in). Project64 writes the RDRAM size to `0x318`, or `0x3F0` for the 6105, on the
  first cartridge DMA; Mars leaves this to the RDRAM interface, whose registers are still stubs.
  *Closed 2026-09-20: §6.5.*

### 6.5 The memory size a cold boot would have left

*2026-09-20.* §6's last item came due. With the Expansion Pak in every frontend's machine (`Mars_Core.md` §7),
*Majora's Mask* started and *Donkey Kong 64* and *Perfect Dark* still said there was no Pak. A single-stepped run
from boot, printing every change to the two words, showed the difference. No boot code writes either word here:
the select register comes up nonzero, IPL3 takes its warm path, and the sizing is skipped. Majora's Mask does not
care — its own `osInitialize` probes memory and stores `0x00800000` at `0x80000318`, seven million steps in, as
Ocarina of Time stores its four. The two Rare games trust the word. On the 6105 the boot code copies `0x800003F0`
to `0x80000318` (at `0x80000224`), nothing had written `0x3F0`, and they read zero.

**The hand-off now leaves the size**, the length of RDRAM, at `0x318`, or at `0x3F0` when the chip is a 6105, whose
boot code then carries it across itself. It is the one result of the skipped sizing that a game can see, and what
Project64 supplies at the first cartridge DMA; mupen64plus needs no such thing because it models the RDRAM modules
and lets IPL3 size them, which is the honest route and is still not Mars's (`Mars_Memory.md` §8.5).
`The_hand_off_leaves_the_memory_size_a_cold_boot_would_have_left` holds both addresses at both sizes.

**Seen, not inferred.** With the Pak, Donkey Kong 64 plays its introduction and Perfect Dark reaches its language
menu over its 3D background; without it each shows its own screen — DK64's pink Pak in three languages, Perfect
Dark's "Expansion Pak not detected" — which is what a stock console does. What this does not cover: a game that
reads the RDRAM modules' own registers to size memory would still find the stubs.

## 7. The 6105's challenge, answered through the PIF

**The 6105 is the one chip a game can talk to after boot.** A game writes fifteen bytes of
challenge into PIF RAM `0x30`–`0x3E`, sets bit 1 of the command byte `0x3F`, and reads PIF RAM
back to find the chip's answer in their place. `Mars_Serial.md` §6 deferred this on the grounds
that "Mars boots through `Mars_Boot.md`'s handoff rather than through the PIF, so nothing asks";
that reason was wrong — the boot is not what asks.

### 7.1 The answer, and what grades it

**Thirty nibbles, high nibble first, each answered from a key and one bit of mode.** The key
starts at `0xB` and the mode at 0; each answer nibble is `(key + 5 × challenge) & 0xF`; the next
key is looked up from that answer in one of two sixteen-entry tables chosen by the mode; and the
next mode follows from the answer's sign and magnitude, with two overrides when the mode was
already 1. `0x2E` and `0x2F` are zeroed and the answer replaces the challenge.

**The grading is weaker than it looks, and here is why.** Three implementations agree:

- mupen64plus `n64_cic_nus_6105.c` and Project64 `PifRamHandler::CicNus6105` are the same code —
  X-Scale's, down to the identifiers.
- `PIF_cic6105.vhd` agrees with them on every one of 20,003 challenges (three fixed, 20,000
  random) when transcribed and run against mupen64plus's compiled C. But its identifiers
  (`cic_lut`, `cic_key`, `cic_mod`, `cic_mag`) follow X-Scale's, so under the referee rule this is
  agreement with a borrower, which is weak evidence.

**The only data named behind any of them is Project64's `pif2.dat`**, which X-Scale's header names as
the sole resource the algorithm was fitted to: one challenge from *Jet Force Gemini* and 267 from
*Banjo-Tooie*, attributed to Tooie and Azimer. The file was deleted from Project64 in `2c31b9486`
and recovered from `28c5a7e77`. Each line stores PIF RAM `0x30`–`0x3F` twice, before and after,
each time as two little-endian 64-bit words. **265 of the 268 pairs match; three do not, each in one byte:**

- *Jet Force Gemini*'s only pair records `B0` at `0x36`, where the algorithm gives `0B`;
- one *Banjo-Tooie* pair records `AB` at `0x37`, where it gives `A8`;
- one records `0x99` in `0x3F`, which every other pair records as `00` and which lies outside the
  answer.

Whether those three are errors in the file or in the algorithm was not established; the fitted
algorithm cannot referee its own training data. The tests use six pairs that match, chosen so
that each of the two overrides changes at least three of them — only 51 and 57 of the 265 pairs
depend on either override at all, and the first four chosen caught each on one pair only.

### 7.2 When the PIF answers, and for whom

**The references disagree on both, and Mars follows the referee.**

| | answers when | answers for | afterwards, `0x3F` |
| --- | --- | --- | --- |
| `PIF.vhd` | a serial read begins, before PIF RAM is copied out (`EVALREAD`, §654; `SI.vhd`'s `READ_WAITPIFPROC`) | a 6105 or 7105 only; any other chip has the bit cleared and RAM left alone | bit 1 cleared, the rest kept |
| mupen64plus | a write into PIF RAM completes, by DMA or processor store | any chip | bit 1 cleared |
| Project64 | a write, when the byte is exactly `0x02` | any chip | zeroed |

For a game that writes its challenge and then reads PIF RAM back by DMA — which is how the
protocol is used — all three give the same bytes. They differ only for a processor load of PIF
RAM between the two transfers, a command byte with other bits set, and a cartridge whose chip is
not a 6105. `SiInterface.AnswerChallenge` answers on the read, for a 6105 only, and clears bit 1
alone; a test pins each of the three.

**A side finding, recorded for the serial slice and not acted on:** `EVALREAD` is also the only
path in `PIF.vhd` to the joybus walk (`EXTCOMM_FETCHNEXT`), so the referee runs the controller
block when PIF RAM is read out, not when it is written in. `Mars_Serial.md` §2 and §5 say the
opposite, citing the guard at §777–781 — which, on this reading, is unreachable, because
`EXTCOMM` is entered only with `pifreadmode` set.

### 7.3 Who asks

**Not Ocarina of Time — at least not in its first 42.86 seconds.** Across 4.0 billion
instructions of the booted game, bit 1 of `0x3F` was never set (§8.3). *Jet Force Gemini* and
*Banjo-Tooie* are the games the recorded pairs came from; neither is in the folder, so this path
is graded only by its unit tests and has never met a game.

### 7.4 The mutation round

Twelve breakages, each caught by at least one test, and one survivor that is equivalent rather
than missed. Caught: dropping `t3`, dropping `ra`, handing back `0x3F`, leaving IMEM empty,
answering as the block comes in, answering for any chip, zeroing the whole command byte, leaving
`0x2F` alone, dropping either override, and dropping the sign fold or the second table. The
survivor is removing the `!toPif` guard: the check sits before the copy, so on a write it would
answer the *old* contents of PIF RAM, which the copy then overwrites entirely. The guard is kept
because it states the rule, and is recorded here as untestable because it is redundant.

## 8. Ocarina of Time: the diagnosis

### 8.1 Where it stopped

**At IPL3's third instruction.** On the unmodified handoff, `lw t2, 0x44(t3)` at `0xA4000048`
loads from virtual address `0x44` — `t3` was zero — which is a mapped address, and the TLB faults. The
exception vectors to `0x80000180`, where RDRAM holds zeroes, and the processor slides through
them as `nop`s for the rest of the run. That is the "two million instructions at two million
addresses" `Mars_GameProbe.md` §3 measured, and the physical address it reported is simply how
far the slide had got.

### 8.2 What each piece does, one at a time

Established with a scratch tracer — not in the repo — that applied each value over the
unmodified handoff, so each row isolates one condition; the row with IMEM empty and the seed right
was measured by clearing IMEM after the fixed handoff, which leaves the same state.

| `t3` | `ra` | IMEM key | seed | outcome |
| --- | --- | --- | --- | --- |
| — | — | — | `0x3F` | TLB fault at step 2 (§8.1) |
| set | — | set | `0x91` | `bltz ra` not taken; falls into the encrypted bytes; reserved instruction at `0xA4000084`, step 73 |
| set | set | — | `0x3F` | IPL3 runs; the signal processor runs undecrypted bytes until IPL3 halts it; the checksum fails and the CPU parks at `0x80000248` |
| set | set | set | `0x3F` | the signal processor stops at its own `break` after 1,088 steps; the checksum still fails, same place |
| set | set | — | `0x91` | **boots**: entry point at step 7,124,304 |
| set | set | set | `0x91` | **boots**: entry point at step 7,124,360; the game writes `0x08` to PIF RAM's command byte (libultra's end-of-boot write) at 8,432,504; a type-4 task at 20.3 million; first graphics task at 46.9 million |
| set | `…1554` | PAL word | `0x91` | **boots**, identically — Project64's PAL values (§6.3, §6.4) |

**So three things were necessary — `t3`, `ra` and the seed — and one was not.** The committed
handoff sets all four.

**Against the prediction.** `Mars_GameProbe.md` §3 named two suspects. The seed was right and
necessary, but it was the third failure a run would meet, not the first. The CIC challenge was
wrong: nothing asks during boot, and this game did not ask at all in the window measured (§7.3).
The failure that actually stopped the game — two registers IPL2 leaves — was not predicted by
anyone, including §2, which listed the registers it set and did not ask which others a boot code
might read.

### 8.3 How far it gets now

**To its title screen.** `Mars_GameProbe.md`, 120 seconds a game, in a **Release** build — which
is why every game executes about three times as many instructions a second as in §3 of that page,
whose numbers were a Debug build's:

| | Ocarina of Time (Europe) | Super Mario 64 (Europe) | Wave Race 64 (USA) |
| --- | --- | --- | --- |
| chip, seed | 6105 family, `0x91` | 6102 family, `0x3F` | 6102 family, `0x3F` |
| instructions a second | 19.58M | 19.19M | 17.51M |
| console time covered | 25.19s | 24.66s | 23.69s |
| graphics / audio / other tasks | 891 / 1,000 / 2 | 567 / 1,205 / 0 | 805 / 1,400 / 0 |
| framebuffers handed over | 564 | 564 | 449 |
| distinct addresses, last 2M | 35,169 | 4,244 | 4,117 |
| audio | 798,462 samples at 32 kHz | 778,949 at 32 kHz | 750,832 at 32 kHz |
| picture | the title logo over Hyrule Field, Link on horseback at the edge | Mario's face over the tiled logo | the attract race under the pier |

Before the change the same probe gave *Ocarina of Time* no task, no framebuffer and two million
distinct addresses. After it, the game runs its main loop, starts its first graphics task 46.9
million instructions in, and plays 32 kHz audio. The two 6102 games draw what `Mars_GameProbe.md`
§3 recorded, so the new register values and IMEM contents did not disturb them.

A separate run of the booted game for 4.0 billion instructions — 42.86 seconds of console time,
859 framebuffers — watched PIF RAM after every instruction for bit 1 of the command byte and never
saw it (§7.3).

### 8.4 What did not move

- **The corpus's verdicts are identical** — the full 1,740-line report, not just the
  tally, compared with and without the change, differing only in the line that reports how long
  the run took; `Failed 54 of 4637` stands. Its boot code is libdragon's, which gets the same
  seed; whatever it does with the new register values and IMEM words changes no verdict.
- **All 2,599 Mars tests pass**, including the new ones.

### 8.5 What this is not evidence for

- **That anything after boot is right.** The frames are recognisable, which `Mars_GameProbe.md`
  §4 already says is not the same claim.
- **That the game's later checks pass.** Whatever the signal-processor routine of §6.4 leaves
  behind, and any challenge a 6105 game sends later, are unexercised by a run with no input.
- **That a PAL console leaves what an NTSC one does.** §6.3 and §6.4 chose one set of values and
  showed the game does not distinguish them; that is a statement about this game.
- **A soft reset.** The warm-boot values of `s5`, and the RDRAM contents that survive one, are
  not modelled.
