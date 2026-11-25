# Sapphire_SpeedHacks — the references' speed hacks, and which have a general form

*Written 2026-10-29, a reading only: nothing was built or timed.* Decided 2026-10-29 by the tester: Sapphire is no
longer clean-room, and the source and game databases of PCSX2, Play! and DobieStation may be read
(`~/.cache/emusen/sealed/ps2-reference/`); for the PlayStation 2 and later, accuracy is to the player (games look, sound
and play the same; hardware timing need not be exact); per-game fixes stay banned in Sapphire, so a general mechanism is
the goal. Mechanisms are described here in the project's own words; no code was copied.

The question for each speed hack or speed-motivated setting: what it skips or approximates, why its authors made it
per-game or opt-in, and whether a form exists that is safe for every game. The compatibility fixes as a whole were
clustered earlier (the third sealed review, 2026-10-28, kept with the review notes); this page refers to its clusters
and does not redo them.

Marks: **[R]** read from the source named; **[M]** measured, with the section that measured it; **[E]** estimated;
**[A]** argued. Counts are PCSX2's `bin/resources/GameIndex.yaml` (12,832 serials), by serial, with titles counted by English name,
region and edition marks stripped; Play!'s are its `GameConfig.xml` (47 games). DobieStation has no game database.

---

## 1. Every hack, at a glance

"On / off" are the game entries that turn the hack on or off against its default. "General" is whether a form exists
that is safe for every game (§2 gives each yes and maybe).

| Hack (emulator) | What it skips or approximates | Default | Entries on / off (serials, titles) | General |
| --- | --- | --- | --- | --- |
| Wait-loop detection (PCSX2) | A loop that branches to itself and, by a register scan, does the same each turn: skips the EE to the next event. Also count-down loops (`addiu r, r, -k; bne r, zero`): jumps ahead, `r` reduced by the time passed | on | none: not a database key | **yes, built** (§2.1) |
| INTC spin detection (PCSX2) | Any read of INTC_STAT more than 8 cycles after the last event moves the EE to the next event, loop or not; disabled in PS1 mode, where it broke games | on | none | **yes, built** (§2.1) |
| INTC_STAT halt (DobieStation) | After 1,000 reads of INTC_STAT since the last INTC change, halts the EE until an IRQ | on | — | **yes, built** (§2.1) |
| Kernel idle loop halt (DobieStation) | The EE halted while its PC is in the kernel's idle loop (81FC0h to 81FE0h) | on | — | **yes, built** (§2.1) |
| Idle evaluator and idle blocks (Play!) | A block branching to itself with only loads, compares and flag tests, or the HLE kernel seeing the same semaphore waited 100 times or a thread rotating itself 500 times: the rest of the 4,800-cycle quantum dropped | on | 31 per-game idle blocks, 27 titles | **yes** for loops (built); **maybe** for loops through the kernel (§2.2) |
| Instant VU1 (PCSX2) | A microprogram started is run to its end at once (`CpuVU1->Execute`), the EE seeing VU1 done | on | on 11 (6); **off 76 (27)** | **maybe** (§2.3) |
| MTVU (PCSX2) | VU1 and VIF1's unpacks on a thread fed by a one-way ring; VU1's busy time the mean of its last four programs; its T and E interrupts seen whenever the thread gets there | on with 3 or more cores | **off 67 (30)**; 18 serials turn off both this and instant VU1 | **maybe**, deterministic but not exact (§2.4) |
| EE cycle skip (PCSX2) | With MTVU, adds VU1's estimated cycles times 1 to 3 to the EE's clock at each kick | off | none | no (§4) |
| EE cycle rate (PCSX2) | Scales the cycles charged per block, −3 to +3: under-clocks or over-clocks the EE against the board | 0 | +: 46 (15); −: 9 (5) | no (§4) |
| mVU flag hack (PCSX2) | Micro mode: status flags not computed in a block unless the block itself needs the non-sticky bits, and blocks linked ignoring the MAC flags' entry state. Macro mode: only the last flag writer before a CFC2, CTC2 or the block's end computes flags | on | **off 77 (30)** | **yes** (§2.5) |
| Fast CDVD (PCSX2) | Halves sector-read and read delays shorter than a full seek | off | none | no (§4) |
| MTGS, GS on its own thread (PCSX2) | The GS's drawing on another thread; SIGNAL, FINISH and LABEL answered on the EE's side at the packet's parse (`Gif_Unit.cpp`'s A+D handler) | always | — | **yes** (§2.6) |
| Hardware download mode (PCSX2) | Local-to-host transfers and GS readbacks skipped or not waited for | accurate | — (GS fixes are out of scope) | no (§4) |
| I-bit immediates as constants (PCSX2) | A microprogram's LOI immediates compiled as constants; `IbitHack` loads them at run time | constants | `IbitHack` 34 (7) | **yes** (§2.7) |
| XGKICK whole (PCSX2) | A PATH1 packet sent to the GIF at the kick, at once | whole | `XGKickHack` (accurate) 115 (75) | **maybe**, with §2.3 |
| VU runs ahead (PCSX2) | VU0 and VU1 run ahead of the EE by whole blocks | ahead | `VUSyncHack` 25 (13), `FullVU0SyncHack` 3 (3) | covered by §2.3 |
| GIF and VIF FIFOs not modelled (PCSX2, DobieStation) | Transfers bypass the FIFOs; DobieStation reports a GIF FIFO count of 16 while a transfer runs | not modelled | `GIFFIFOHack` 25 (11), `VIFFIFOHack` 24 (7) | not needed: Sapphire models them (cluster 8) |
| Rounds (DobieStation 32 cycles, Play! 4,800) | Every unit run for the EE's last span; the IOP in arrears | always | — | **yes, built** (`Sapphire_Speed.md` §62) |
| High-level kernel and IOP modules (Play!) | The EE kernel and the IOP's modules (CDVD, pad, memory card, sound library, SIF) run as host code | always | — | no (§4) |
| Fast boot (DobieStation) | The BIOS skipped to the game's entry | on for discs | — | out of scope: a boot path, not a speed lever |
| Skip MPEG (PCSX2, compiled out in DobieStation) | `sceMpegIsEnd` answered "ended", so videos are skipped | off | 5 (3), as compatibility | no (§4) |

PCSX2's EE cache (off, "interpreter only, for diagnostics") and its float clamping and rounding modes are also
approximations taken for speed; their entries are clusters 2 and 5 of the earlier review, and Sapphire does not make
them.

**The groups, by shared mechanism, and the condition that separates safe games from broken ones.**

- **Skipping the EE's waits** (wait loop, INTC spin, both DobieStation halts, Play!'s idle blocks). PCSX2 needs no entry;
  Play! needs 31, because its pattern list misses loops. The condition: the loop is *pure*, each turn the same, reading
  only what an event can change. PCSX2's INTC form ignores it and skips on any read, which is why it fails in PS1 mode
  [R, `HwRead.cpp`].
- **VU1 decoupled from the EE in time** (instant VU1, MTVU, cycle skip, XGKICK whole, VUs ahead): 76 + 67 − 18 = 125
  serials off, plus 115 for the accurate XGKICK and 28 for the VU sync fixes. The entries' own comments name the
  condition: something observes or feeds VU1 during its run. Either the EE watches VU1 (22 of the 67 MTVU entries say
  "bad T-Bit handling"; others say the game's frame rate halves), or VIF1 writes data the running program has yet to
  read. Play! special-cases the second for one studio: an MSCAL followed by an UNPACK the program needs, the program's
  start delayed (`Vif.cpp`, `StartDelayedMicroProgram`) [R]. Or the GIF reads a PATH1 packet after VU1 has changed it
  (the "SPS", spiky polygons, of most XGKICK entries).
- **Flags not computed**: 77 serials off. The condition: a later program, or a CFC2 in another block, reads sticky
  status bits or MAC flags the shortcut dropped (§2.5).
- **The EE's speed against the board** (cycle rate, cycle skip): no condition makes them safe, because they change what
  the game computes per frame. 55 serials use the cycle rate as a *compatibility* fix in both directions: PCSX2's EE
  timing is wrong for them (cluster 4).

## 2. The general forms

### 2.1 The EE's waits: built in Sapphire (S33)

`Sapphire_Native.md` §47 proves a loop by stepping one turn: every read is of something only an event can change, and
the turn comes back with the registers as they were. Its further turns are then passed, with counting loops tracked as
inductions (§47.5). This is the exact form of PCSX2's wait-loop and timeout-loop detection, its INTC spin detection,
DobieStation's two halts and Play!'s idle blocks. It recognises the safe case itself, by the proof, where PCSX2 relies on
a register scan and a pattern, DobieStation on a read count, and Play! on a list of 31. Measured 2026-10-26: Jak and
Daxter's loading window 3.6 times faster, OSDSYS 1.7 times [M, §47.11]; on the arena it passes 45 % of the EE's cycles
[M, `Sapphire_Speed.md` §60.2]. Nothing to build. What remains is §2.2 and the loops §47.7 refuses (shortlist item 5).

### 2.2 Waits through the kernel (from Play!): maybe

Play!'s evaluator calls the EE idle when its HLE kernel sees the same semaphore waited and signalled 100 times in a row,
or a thread rotating the ready queue to itself 500 times, or two threads bouncing [R, `Ee_IdleEvaluator.cpp`]. On
Sapphire's real kernel these waits are loops through SYSCALL and ERET that write the kernel's thread queues, which §47's
one-turn proof refuses. A general form would prove a period of two or more turns across the exception, the kernel's
writes coming back to the same values. **Unknown how often it applies**: §47.9's counts give the refusals by kind, and
reading them on the three windows decides whether this is worth building. Rough size: small to medium.

### 2.3 VU1 to its end (from PCSX2's instant VU1): maybe, exact where nothing intervenes

**Why the authors left it per-game.** Run at the kick, a program reads VU1's memory before VIF1's later writes and
hands the GIF its packets before PATH2 and PATH3 have had their turn. The 27 titles it breaks include *Jak X*, *Guitar
Hero World Tour*, *Valkyrie Profile 2* and *Hitman: Contracts*: hangs going in game, misplaced effects and spiky
polygons [R, `GameIndex.yaml`]. Twelve entries say turning it off "significantly improves game speed"; the source does
not say why.

**The general form.** Run VU1 as one call from its start to the first of these: its end, an observation by the EE
(`Sapphire_Speed.md` §62.1's list), the cycle of the next foreign access to its memory (VIF1's next write, the GIF's next
PATH1 read), or a bound on kicks outstanding. Each XGKICK is handed to the GIF with its own cycle. This is
`Sapphire_Redesign.md` §4.3 with two additions the references make plain:

1. **Bound the run by the kick queue, not the state.** §62.3 refused VU1 to its end because a whole program's kicks
   overflowed the state's 16 slots. Stopping the run at the kick that would fill the queue, and resuming when the GIF
   frees a slot, is deterministic and exact.
2. **The GIF and VIF1 resume at VU1's kick and end cycles as events.** §62.3's other failure was the VIF's next wait
   finding the GIF still holding the packets.

**Why it is safe for the player**: every foreign access lands at its own cycle, and every observation stops the run,
so the results are the cycle-ordered board's. It is exact, not a compromise; the instant-VU1 failures come from leaving
both out. **Speed**: VU micro mode's call path is about 80 ms of the arena's 500 [M, §62.4]; most of it is crossings that
whole runs remove, so **≈ 40 to 80 ms** [E]. PCSX2's interface calls instant VU1 "a modest speed improvement in most
games", and its database's entries that turn it on claim "about 15 % more FPS" [R]. **Needs**: the VU recompiler's
earliest-foreign-access check at each pair, the kick queue with cycles, the GIF and VIF1 at events. **Size**: large,
about 1,500 lines [E]. **Overlap**: this is §62.1 item 2, refused in its first form; build it once, as §4.3.

### 2.4 VU1 on a thread (from PCSX2's MTVU): maybe, deterministic but not exact

**Why the authors left it per-game.** The thread's progress depends on the host, VU1's busy time is a running average,
and its T and E interrupts arrive when the thread reaches them. Games that poll VU1 or stop it with the T bit break: the
Transformers and DreamWorks lines, *Marvel: Ultimate Alliance* and *The Sims*, among the 30 titles [R].

**The general form** is `Sapphire_Redesign.md` §4.7's split at fixed quanta. What crosses in quantum k is delivered at
k + 1, so the results do not depend on the host. It moves VU1, VIF1 and the GIF (about 150 ms of the arena's 500 by §62.4's
shares [E]) off the EE's thread, at the cost of seeing couplings up to 2Q late. **The references add a run-time
condition.** The entries' own comments show the breaking class is a game that watches VU1. So the split can fall back to
Q = 0 while VU1's T or D stop is enabled in FBRST, or while the EE has read VU1's bit of VPU-STAT in the previous quantum.
That is decided from emulated events alone, so it stays deterministic, and it is not per game. Whether that covers all
the 30 titles is not settled by the source. **Size**: large, already planned (R5's boundary). **Speed for the player**:
the largest on the handheld if the EE's thread is the bound; not exact, so the tester's decision of §4.7 stands.

### 2.5 Flags only where read (from PCSX2's mVU flag hack): yes

**Why the authors left it per-game.** Micro mode drops the status flags altogether in blocks that do not need the
non-sticky bits. It also links blocks while ignoring the MAC flags' state at entry [R, `microVU_Flags.inl`,
`microVU.h`'s quick lookup]. Macro mode keeps only the last flag writer before a read; the intermediate writers' sticky
bits are lost (`iR5900Analysis.cpp`, `COP2FlagHackPass`). Programs that read sticky bits or a MAC flag instance carried
over from an earlier block break: the Katamari series, *God of War II*, *Silent Hill 4*, *Tales of Rebirth* and *Full
Spectrum Warrior*, 30 titles. Their symptoms include falling through floors and enemies turning into squares [R].

**The general form**: a flag value is needed only where a later instruction of the same program can read it before it
is overwritten, and always at the program's end and at every observation. That liveness is exact within one program,
and it is `Sapphire_Redesign.md` §4.3's flag rule (C4 stays refused across programs). The one addition the hack's
failures point at: the **sticky bits are accumulated by every writer**, even where the full flags are not stored. That is
an OR of lane signs, zeros and the overflow and underflow results, cheap beside a full flag word. With that, the dropped
work is exactly the dead stores. **Speed**: `ops.rs`'s lane and flag wrappers are 5.9 % of the arena's emulation thread
[M, §62.4], about 30 ms; **≈ 15 to 30 ms** [E]. PCSX2 calls its hack "good speedup". **Needs**: a liveness pass in the VU
recompiler keyed by the program's entry state. **Size**: medium, about 500 lines [E]. **Overlap**: the same compiler work
as §2.3 and §62.6's recommended next step; build them together.

### 2.6 The GS's packet stream on its thread (from PCSX2's MTGS): yes

PCSX2 parses the GIF's output on the EE's side only far enough to answer what the EE can observe (SIGNAL, FINISH and
LABEL, and with them CSR), then hands the packets to the GS thread [R, `Gif_Unit.cpp`]. Since the observable answers
come from the parse, it is exact for the EE, and deterministic if the GS thread's results return only through what the
EE waits for. This is the fourth sealed review's pointer 5, Sapphire's GS workers taking the stream rather than
published jobs. **Speed**: the GS on the emulation thread is 42 ms of 500 [M, §62.4]; **≈ 25 to 35 ms** [E].
**Needs**: register state, CLUT loads and local-to-host transfers on the GS side, with the EE-observable registers kept on
the emulation thread. **Size**: medium to large. **Overlap**: none with §62.

### 2.7 Immediates loaded at run time (the exact side of PCSX2's I-bit fix): yes

PCSX2 compiles a microprogram's LOI immediates as constants, so a game that patches them between runs either recompiles
or runs stale code. `IbitHack`, 7 titles, loads them at run time. The general form keys a program's identity on its code
without those words and loads the words when the program runs (`Sapphire_Redesign.md` §4.3, "Program identity").
**Speed**: unknown until R2 counts recompiles. **Size**: small.

## 3. The shortlist for Sapphire

Ranked by player-visible speed, largest first, cheapest first on a tie. Every item is exact except item 2.

| # | Item (origin) | Speed [E] | Size | Overlap |
| --- | --- | --- | --- | --- |
| 1 | VU1 to its end, bounded by the kick queue, foreign accesses at their cycles (PCSX2 instant VU1; §2.3) | 40 to 80 ms of the arena's 500 | large | **§62.1 item 2** (refused form), `Sapphire_Redesign.md` §4.3, §62.6 Q1 |
| 2 | The VU1, VIF1 and GIF thread at fixed quanta, with Q = 0 while VU1 is watched (PCSX2 MTVU; §2.4) | up to about 150 ms of wall time on two threads; not exact | large | `Sapphire_Redesign.md` §4.7, R5 |
| 3 | The GS's packet stream on its thread (PCSX2 MTGS; §2.6) | 25 to 35 ms | medium to large | none |
| 4 | Flags only where read, sticky bits always accumulated (PCSX2 mVU flag hack; §2.5) | 15 to 30 ms | medium | with item 1 and §62.6 Q1 |
| 5 | Idle skipping over loops that poll a VU, passed to VU1's end once it is an event (§2.1, §47.7's `Coprocessor` and `Vu` refusals) | unknown; read §47.9's refusal counts first | small, after item 1 | needs item 1's end event |
| 6 | Waits through the kernel, proven over a period (Play!'s evaluator; §2.2) | unknown; same counts | small to medium | none |
| 7 | Program identity without immediates (PCSX2 I-bit; §2.7) | unknown; R2 counts | small | `Sapphire_Redesign.md` §4.3 |

Item 2 is ranked by its size, but it is a timing compromise and is the tester's decision (`Sapphire_Redesign.md` §11
Q4). Items 1 and 4 are one piece of compiler work, and §62.6's recommended next step, the VU call path, should take
them both.

## 4. The hacks with no general form

- **EE cycle rate and cycle skip** (PCSX2). They change how much the EE computes per frame. Under-clocked, an EE-bound
  game drops frames; over-clocked, it runs faster than the console. No detection makes either invisible to the player.
  A general shortcut that changes results may only ever be a player option; none is proposed.
- **Fast CDVD** (PCSX2). It shortens emulated load times, which the player sees, and it breaks games that move DMA while
  a read is pending: its own source names *Tales of the Abyss* [R, `CDVD.cpp`]. With exact idle skipping, the console's
  load time already costs the host little when the EE waits (§2.1), so the host gain is mostly had already.
- **High-level kernel and IOP modules** (Play!). Host code for the IOP's modules changes the IOP's timing and its
  answers' order, so it is not exact. The IOP's whole side is 8 ms of the arena's 500 [M, §62.4], so there is little to
  gain, and the firmware policy's open replacement is a separate track.
- **Skip MPEG** (PCSX2, DobieStation). It removes the game's videos.
- **Hardware download mode, no readbacks or unsynchronised** (PCSX2). The EE reads pixels other than the GS's.
- **The counting heuristics as such**: DobieStation's 1,000 reads, the 100 semaphore waits and 500 rotations of Play!,
  and PCSX2's 8-cycle INTC rule. Each is a guess at purity that §47's proof replaces.

## 5. Questions

1. **[decision] The run-time fallback for the VU1 thread** (§2.4): add Q = 0 while VU1's T or D stop is enabled or VU1 is
   polled to `Sapphire_Redesign.md` §4.7. *Recommended*: yes, when §4.7 is built; it is deterministic and is not per
   game.
2. **[routine] The idle refusals**: read §47.9's refusal counts on the short window, the dock and the arena (a counted
   run, no build) before items 5 and 6. *Recommended*: yes.

## 6. What was read

PCSX2 (`pcsx2/`): `Config.h` (`SpeedhackOptions`, `GamefixId`), `Pcsx2Config.cpp` (defaults, database names),
`VMManager.cpp` (MTVU by core count), `pcsx2-qt/Settings/EmulationSettingsWidget.cpp` and `AdvancedSettingsWidget.cpp`
(each setting's description), `HwRead.cpp` (INTC spin), `Interpreter.cpp` and `x86/ix86-32/iR5900.cpp` (wait-loop and
timeout-loop detection, cycle rate, Skip MPEG), `VU1micro.cpp` (instant VU1), `MTVU.cpp` (the VU1 thread, cycle skip),
`CDVD/CDVD.cpp` (fast CDVD), `x86/microVU_Flags.inl`, `x86/microVU.h`, `x86/microVU_Macro.inl` and
`x86/iR5900Analysis.cpp` (the flag hack), `Gif_Unit.cpp` (A+D handling), `bin/resources/GameIndex.yaml` (counted by a
script in the scratch folder). Play! (`Play-/`): `GameConfig.xml`, `PS2VM.cpp` and `PS2VM.h` (the quantum),
`ee/Ee_SubSystem.cpp`, `ee/EeBasicBlock.cpp`, `ee/Ee_IdleEvaluator.cpp`, `ee/Vpu.cpp`, `ee/Vif.cpp`. DobieStation
(`DobieStation/src/core/`): `emulator.cpp`, `scheduler.cpp`, `ee/intc.cpp`, `ee/emotion.cpp`. Sapphire's own:
`Sapphire_Speed.md` §60 to §62, `Sapphire_Redesign.md` §4.3 and §4.7, `Sapphire_Native.md` §47. No other emulator's
source was read; no firmware was listed or opened.
