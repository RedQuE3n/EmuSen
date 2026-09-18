# Mars — cheat codes: the N64 GameShark, and where its writes land

*Landed 2026-09-18. Not a Phase E slice: the cheat seam `Mars_Core.md` §8 left as a stub, which its §0 table
described as "no codec applies anything". The code is `Cheats/N64GameSharkCodec.cs` and `Cheats/MarsCheatCodecs.cs`,
`MarsCore.Cheats.cs`, one line in each of `MarsCore.LoadRom` and `MarsCore.RunFrame`, and a read intercept in
`Memory/MemoryBus.cs`; the shared cheat model gained a test write for it, which is `EmuSen_Cheats.md` §7. The tests
are `MarsCheatTests`, the five Mars cases in `CoreFactoryCheatWiringTests`, and the test-write cases in
`CheatWriteModelTests`.*

***§0 is the part to read first**: nothing here is graded against a GameShark, and it says what stands in for one.
§2 is the five places the two reference emulators disagree about what a code does, and which reading Mars took. §5.1
is a defect the first version had — a cheat on from power-on hung Super Mario 64's boot — measured before it was fixed.*

---

## 0. What grades this, when nothing measures a GameShark

**There is no hardware measurement behind any rule on this page.** The hardware corpus
(`~/Projects/nemu64-test-reference`) tests the console, not a cheat cartridge plugged into it; there is no GameShark on
this machine; and the one device document either reference points at — mupen64plus's `cheat.c` cites kodewerx's
`hacking_n64.html` in its header — is not local and was not fetched. What stands in:

- **Two emulators, read for mechanism.** Project64: `Source/Project64-core/N64System/Enhancement/Enhancements.cpp` —
  `ApplyGameSharkCodes` (403–537), `EntrySize` (539–576), `ModifyMemory8`/`16` (578–624), `ApplyGSButton` (68–94).
  mupen64plus: `src/main/cheat.c` — `execute_cheat` (106–154), `cheat_apply_cheats` (208–307), `cheat_add_new`
  (374–427) — and `gs_apply_cheats` in `src/main/main.c` (1034–1047).
- **parallel-n64 is not a third opinion.** It is the libretro core the database's N64 folder is written for, and its
  `mupen64plus-core/src/main/cheat.c` is mupen64plus's with the locking removed (a `diff` of the two shows nothing else).
  Its frontend's `retro_cheat_set` (`libretro/libretro.c` 2962–3006) is evidence for one thing only: how a database code
  string is cut into lines (§5).
- **What the two agreeing is evidence for** is the community's reading of the format, which both were written from. It
  is not evidence of what the device does, and where they agree on something no store instruction can do (§2.4) the
  agreement is not followed.
- **The referee has nothing to say.** N64_MiSTer's `N64.sv` declares a *Cheats* menu entry and option (`"C,Cheats;"`,
  `"O[103],Cheats Enabled,Yes,No;"`, lines 302–303), but nothing in `N64.sv` or `rtl/` consumes the download or reads
  that status bit in this (shallow) checkout. There is no third implementation to consult.

**A census stands in for how much each decision matters.** Project64 ships its own database, `Config/Cheats/*.cht`:
671 files and 21,103 code lines. By type:

| type | lines | share | Mars |
| --- | ---: | ---: | --- |
| `80` 8-bit write | 10,464 | 49.6% | applied |
| `81` 16-bit write | 8,052 | 38.2% | applied |
| `D0` 8-bit equal | 916 | 4.3% | applied |
| `50` repeater | 883 | 4.2% | applied |
| `D1` 16-bit equal | 650 | 3.1% | applied |
| `89` 16-bit, GameShark button | 64 | 0.3% | refused (§4) |
| `88` 8-bit, GameShark button | 61 | 0.3% | refused (§4) |
| `A0` 8-bit uncached write | 9 | — | applied |
| `D2`, `D3` not equal | 2 each | — | applied |

No `A1`, `F0`, `F1`, `EE`, `DE`, `CC` or `FF` line appears at all. What the census is evidence for is which types a user
meets in a curated list, and how many real codes each disagreement below touches. What it is not is a sample of every
code in circulation: Project64's curators had reason to drop the types Project64 does not run (§4), so their absence
here says nothing about how common they were.

## 1. One format, one mechanism

**The GameShark is a RAM poke, re-applied every frame.** Each line is an eight-digit command and a four-digit value,
`TTAAAAAA VVVV`: the top byte `TT` is the type and the rest is an address. Some lines write, some test memory and guard
the line after them, and one repeats the line after it:

| type | does | registry form |
| --- | --- | --- |
| `80`, `A0` | write the value's low byte | one `Set` write, 1 byte |
| `81`, `A1` | write the value, high byte first | one `Set` write, 2 bytes, big-endian |
| `D0` / `D1` | the next code only if the byte / halfword equals the value | `IfEqual` test |
| `D2` / `D3` | … only if it differs | `IfNotEqual` test |
| `5000CCSS VVVV` | the next write, `CC` times, `SS` bytes and `VVVV` in value apart | the write, with a repeat run |

**Neither reference has a ROM-patch format for the N64**, and no type either decodes becomes one, so `CoreFactory`
bundles the GameShark in the auto-detect (RAM poke) slot and leaves the explicit slot empty; `CheatCodecsFor("N64")`
answers the same pair, which is what lets the Active Cheats window's N64 tab accept a code before a game is loaded.
Registry ROM patches are still supported, through the cartridge (§6); only a typed code cannot make one.

## 2. The decode, and where the references disagree

### 2.1 Writes

**An 8-bit write takes the value's low byte.** Both references cast before writing (Project64 `(uint8_t)Code.Value()`,
mupen64plus `(uint8_t)value`), and 182 census lines depend on it — `8033B9BC FFFF` writes `FF`. Mars stores the low byte
as the write's value, so the registry and a saved cheat file show what is actually written.

**`A0`/`A1` decode to the same write as `80`/`81`** (§3).

### 2.2 Tests

**A test guards the next code, and a run of tests is their AND** — in both references, by different routes. Project64
recurses (`ApplyGameSharkCodes` calls itself for the next entry only on a pass, and `EntrySize` counts a test and
everything it guards as one entry); mupen64plus sets a `cond_failed` flag that no later test clears and the next
non-test code consumes. Mars carries tests into the registry as `IfEqual`/`IfNotEqual` writes, and the registry's rule
is mupen64plus's: a failed test holds until the next write that is not a test, which it skips (`EmuSen_Cheats.md` §7).
`A_run_of_tests_must_all_pass` pins the order-independence — a passing test after a failed one does not undo it.

**Disagreement 1 — an 8-bit test's high byte.** Project64 compares the byte it read against the whole sixteen-bit value
(`bMemory == Code.Value()`, line 474), so a `D0` whose value has a nonzero high byte can never pass and such a `D2`
always does. mupen64plus casts the value to `uint8_t` first. The census has four such lines: `D00E7596 03D7`, twice, in
Ready 2 Rumble Boxing (U)'s *Sudden Death Mode* cheats, and `D216C556 FFFF` in both WWF No Mercy files. **Mars follows
mupen64plus**: it treats an 8-bit test's high byte the way both references treat an 8-bit write's, and it is the
reading of the libretro core the database is written for. Under Project64 the two Ready 2 Rumble cheats never fire;
under Mars they fire when the byte is `D7`. Neither is graded — the `03` looks like a typed `D1` more than either
reading.

### 2.3 The repeater

`5000CCSS VVVV` followed by a write: `CC` (bits 8–15) repetitions, the address advancing by `SS` (bits 0–7, unsigned)
and the value by `VVVV` each time. Both references agree on those fields and on a count of zero writing nothing; Mars
refuses a count of zero by name rather than keep a cheat that does nothing. The value accumulates at sixteen bits in
Project64 and at `int` in mupen64plus and is truncated on each write in both, so an 8-bit repetition writes the low byte
of the running sum — `50000201 0001` then `800001FF 00FF` writes `FF` then `00`, which
`An_8_bit_repeater_wraps_its_value_at_a_byte` pins, and which the registry's `ValueAt`/`ByteAt` already did.

**Disagreement 2 — bits 16–23.** mupen64plus only recognises a repeater whose command matches `0x5000xxxx`
(`(address & 0xFFFF0000) == 0x50000000`, line 399); any other `50` line is an ordinary code, which does nothing, and the
line after it then runs once on its own. Project64 reads the count and step whatever bits 16–23 hold. The census has two
such lines, `50010100 0000` in both Gauntlet Legends files, where a count of one and a step of zero make both readings
the same single write. **Mars follows Project64** — a code that says "once" is honoured either way, and nothing in the
census tells the readings apart.

**Disagreement 3 — what a repeater may repeat.** Project64 repeats only `80` and `81` (and Xplorer64's `10`/`11`); after
anything else it ignores the repeater, and `EntrySize` lets the next line run once by itself. mupen64plus expands
whatever follows when the cheat is added, so a `D0` becomes several tests and an `88` several button codes. In the
census a repeater is followed by `80` 511 times, `81` 366 times and `88` six times. **Mars repeats `80`, `81`, `A0` and
`A1`** — an uncached write is a write, and nothing about repeating one distinguishes it, where Project64 would drop the
repetition silently — **and refuses anything else**, so a repeater is never discarded without a word.

**Disagreement 4 — a test in front of a repeater.** Project64's `EntrySize` makes test, repeater and write one entry, so a
failed test skips every repetition. mupen64plus has already expanded the repeater into separate codes by the time it
applies anything, and its flag skips only the first of them. **Mars follows Project64**, and gets it without a special
case: a repeater is one registry write with a repeat run, so "the next write" is the whole run. The argument is the
format's own shape — both references treat the repeater and its line as one code everywhere else, and a test guards the
next code. The census has three such cheats: two in San Francisco Rush 2049 (E) (`D017055A 0004`, `50000C02 0000`, then
an `81` or an `80`) and one in Turok 2 (E) (`D00FC701 0020`, `50000601 0000`, `802FC99C 0030`). Under mupen64plus,
eleven of the twelve Rush writes happen whatever the test says.

**A step that walks off the end of the address** is handled differently again, which is recorded for completeness:
Project64 walks the virtual address (`0x80FFFFFF` + 1 is `0x81000000`, a physical address past RDRAM, skipped), and
mupen64plus walks the *code word*, so the type byte itself changes and an 8-bit write becomes a 16-bit one. Mars walks
the physical address and drops what lands past RDRAM (§3.1). No census code reaches the end.

### 2.4 A 16-bit code at an odd address

**Disagreement 5 is with the processor rather than between the references.** Both keep RDRAM as host-endian 32-bit words
and find a halfword by exclusive-OR with 2 (Project64 `MemoryPtr(VAddr ^ 2, …)` in `MemoryVirtualMem.cpp` 241–250,
mupen64plus `(address & 0xFFFFFF) ^ S16`). At an odd address that straddles two words: `81` at 4k+1 writes the value's
low byte at 4k and its high byte at 4k+7, and at 4k+3 it lands both bytes one address early. Neither writes the bytes the
code names, and no halfword store the VR4300 can make does either — a misaligned one raises an address error. What a
GameShark's own handler does with such a code is unmeasured.

The census has 49 such lines (`811BDC1B FFFF`, Carmageddon 64's *Max Credits*, is one) and one repeater that produces
them — Diddy Kong Racing (U) (V1.0)'s *Enable All Cheats*, `50000401 0000` then `810DFD9C FFFF`, whose second and fourth
repetitions are odd. Fifty codes in a curated list suggest they did something on the device their authors used; they are
also exactly what a mistyped last digit looks like, and the census cannot say which. **Mars refuses them by name**:
applying the references' tear would reproduce a write nobody intended, and applying them a byte at a time would invent
a device behaviour nothing here measured. This is the open question on this page most worth a hardware measurement.

## 3. Addresses: KSEG0, KSEG1, and the 24 bits a code can name

**The physical address is the command's low 24 bits.** Project64 ORs them onto `0x80000000` (`80`, `81`, the tests) or
`0xA0000000` (`A0`, `A1`) and translates through a map that sends both segments to `address & 0x1FFFFFFF`
(`MemoryVirtualMem.cpp` 79–83); mupen64plus masks with `0xFFFFFF` directly. Either way `80` and `A0` name the same byte,
and `The_type_byte_never_reaches_the_address` pins the mask at both ends.

**Why the device had two write types.** A GameShark's handler is code on the VR4300: an `80` write goes through KSEG0 and
the data cache, an `A0` write through KSEG1 around it. Mars models no cache (`Mars_Cpu.md` §13), and neither reference
models one for cheats — both write their RDRAM array directly — so the difference is not observable here and both decode
to the same RDRAM write. That is a statement about Mars, not about the device: where a dirty cache line would have
shadowed an `A0` write on a console, Mars cannot show it.

**Twenty-four bits reach sixteen megabytes; RDRAM is four, or eight with the Pak.**

**Byte order is the machine's own.** `MemoryBus.Rdram` is big-endian — byte 0 is the high byte of word 0 — so a 16-bit
write lands its high byte at the lower address, with no exclusive-OR. The references' `^ S8` and `^ S16` exist only
because their RDRAM is host-endian words. `A_16_bit_write_lands_high_byte_first_as_the_processor_reads_it` pins the two
literal bytes and the processor's own `Read32` of them (`0x12340000`).

### 3.1 Past installed RDRAM

- **Project64** refuses any physical address at or past its allocated RDRAM (`MemoryPtr`, `MemoryVirtualMem.cpp`
  201–227): the write is skipped, and a test fails whether it asked for equal or not-equal.
- **mupen64plus** does not check (`cheat.c` 65–99). Its RDRAM block is always allocated at eight megabytes
  (`RDRAM_MAX_SIZE`, `src/device/memory/memory.c` 224–226), so on a four-megabyte console a code in the upper half writes
  memory the game cannot see, and one past eight megabytes writes into the signal processor's memories and whatever
  block follows them.
- **Mars** drops the write and reads zero for the test — what `MemoryBus` gives the processor for RDRAM that is not
  installed (`Mars_Memory.md` §2.1). The argument is that the device's handler loads and stores through the same bus as
  the game, so Mars answers as that bus would. The cost is one disagreement with Project64: a `D0 … 00` there passes.

The census has 487 lines addressing four to eight megabytes (the Expansion Pak's half, and inert on the default
console, `Mars_Core.md` §7) and two past eight: `81DEC164 42C8` and `81DEC166 0000`, NFL Blitz 2001 (U)'s *Infinite
Turbo* for player 2, very likely a mistyped `0`. **The decoder cannot refuse these**, because it does not know the
console: it answers before any ROM is loaded, and a code for the upper half is right on an eight-megabyte machine.

## 4. What is refused, and why

**Every refusal is at decode and names its reason**, so none is silent: the Active Cheats window prints it in its status
line, `cheat add` prints it, and a `.cht` import counts the cheat as skipped. What that costs is stated plainly: a cheat
with one refused line is refused whole, because a cheat half-applied is a different cheat.

| type | Project64 | mupen64plus | Mars |
| --- | --- | --- | --- |
| `88`, `89` | only when its GameShark-button key is pressed (`ApplyGSButton`, via `SystemEvents.cpp` 181) | each frame while its GameShark event is active | refused: no GameShark button |
| `A8`, `A9`, `D8`–`DB` | Xplorer64 button writes (`A8`/`A9`), unknown (`D8`–`DB`) | button writes and button tests | refused: no GameShark button |
| `F0`, `F1` | not applied (unknown type) | written once, at the first frame (`g_gs_vi_counter == 0`) | refused: no boot moment |
| `EE` | not applied | rewrites `0x318`–`0x31B` to four megabytes each frame, with its own comment "most likely, this doesnt do anything" | refused: the Pak is chosen when the console is built |
| `DE` | not applied | inside its test range, and `execute_cheat` passes it: a test that always passes | refused: no effect on memory in either |
| `CC`, `FF` | not applied | not applied, and not reported | refused |
| `30`/`31`, `82`–`85` | 8- and 16-bit writes, present since the repository's first import in 2008 with no note of provenance | not applied | refused: the references disagree and nothing local says what they are |
| Xplorer64's `E8`/`E9`, `C8`/`C9`, `B8`–`BB` | decrypted and applied | not applied | refused: another device's format |

**The GameShark button (`88`, `89`)** is the only refused type the census holds in numbers — 125 lines, 0.6%. Both
references apply them only on a user action, and EmuSen's controller template has no control for a button on the cheat
cartridge; adding one for a sixth of a percent of the census is not done here. The message says to enter the code as
`80`/`81` to hold the value every frame instead, which is usually what such a code's author was approximating.

**The boot-time writes (`F0`, `F1`) have no moment to land at.** Mars applies cheats at frame boundaries, to a registry
`CoreFactory.Bundle` hands over *after* `LoadRom` has already run the handoff, so there is no point before the game's
own code runs at which a boot-time write could be made. Writing it at the next frame instead would be a different code,
and mupen64plus's "first frame" is not the device's boot either.

**`EE` names memory the machine already settled.** mupen64plus writes four megabytes into `0x80000318`, the word
libultra keeps the memory size in (`Mars_Boot.md` §6.5), on every frame. What a game does with that word is its own —
Ocarina of Time overwrites it with its own measurement twelve thousand steps after its entry point (`Mars_Boot.md` §6.5)
— so rewriting it every frame is not obviously what the device did either. Mars chooses the Pak when the core is built
(`MarsCore(expansionPak)`, `Mars_Core.md` §7), and the way to hide it here is not to install it.

**The shape faults are refused the same way**: Project64's `??` placeholder (`8125508A 00??` followed by a list of
options, which Project64 substitutes from a menu Mars does not have), a test with nothing after it, a repeater with no
line after it or a line it will not repeat, and an odd 16-bit address (§2.4).

## 5. Where codes come from, and when they apply

**Typed.** The N64 tab of the Active Cheats window takes a whole code — `D0000200 0005+81000100 1234` — as one cheat of
two writes. Lines may be joined by `+`, spaces or any other punctuation, and a run of twelve digits is read as one line,
so `cheat add D00002000005+810001001234 name` works from the shell, which splits its arguments on spaces.

**From the database.** A libretro `.cht` joins a code's lines with `+`. parallel-n64 splits the string at every non-hex
character and pairs the runs as address and value (`retro_cheat_set`); Mars's reader does the same, and accepts the
value run at one to four digits as that pairing does. A repeater spans a `+`, which is why `ChtFile` now hands a codec
that decodes whole codes the whole string (`EmuSen_Cheats.md` §7). **No N64 `.cht` from the libretro database is on this
machine** — `DataStore.Cheats` holds no `Nintendo - Nintendo 64` folder — so this path is tested on hand-written files
in the database's format (`A_cht_file_imports_each_code_whole_and_skips_what_it_refuses`), not on the database.

**When.** At the end of every `RunFrame`, after `TotalFrames++` — so once per VI field, or per cycle-cap frame while the
VI is unprogrammed (`Mars_Core.md` §3) — provided the game has interrupts enabled; and whenever `ApplyCheats()` is
called, which is Mistress's Apply button, through `MarsDebugTarget.ApplyCheats`, with no condition. Project64 applies at
every VI interrupt, but only while the status register's interrupt-enable bit is set (`N64System.cpp` 2400);
mupen64plus applies at every VI after holding off for the first sixty (`gs_apply_cheats`). The condition is Project64's,
and §5.1 is the measurement that made Mars need one.

**A disabled cheat is not undone.** Both references put back what a cheat overwrote when it is switched off —
Project64's `ResetCodes` (209–237), mupen64plus's `old_value` restored on the frame after `was_enabled` drops
(`cheat.c` 296–305). The registry has never done this for any core, and Mars does not start: a disabled *infinite
lives* leaves the last value written, which the game then changes or not.

### 5.1 The first application, and the boot code's checksum

**Applying at the end of every frame hung Super Mario 64's boot, and that was measured before it was fixed.** Mars's
first frame ends on the cycle cap, 4,100,000 cycles, because nothing has programmed the VI yet (`Mars_Core.md` §3). The
cartridge's boot code takes longer than that to reach the game. Measured to the header's entry point through `MarsCore`
alone: 5,753,305 cycles for Super Mario 64 (Europe), 5,773,226 for Wave Race 64 (USA), 7,124,366 for Ocarina of Time
(Europe) — 61 to 76 ms of console time. So the first frame boundary falls inside the boot code, after it has copied the
game's first megabyte into RDRAM and while it checksums it: when the frame ends the processor is in the boot code's
relocated loop, at `0x8000018C`, `0x80000150` and `0x80000118` respectively. A cheat inside that megabyte, applied
there, changes a byte the checksum has not yet read.

Project64's own *Infinite Lives* for this game, `803094DD 0064`, lies inside the megabyte the boot code copies to
`0x80241800`. Enabled from power-on and run for twelve frames through `RunFrame`: without it the game reaches its entry
point at 5,753,305 cycles and the VI's first field ends frame 1; with it the checksum fails and the boot code spins at
`0x800001CC` for all twelve frames, the VI never programmed. That is what a user whose saved cheat list is restored as the
game starts would see — a black screen — and nothing in the synthetic tests could have shown it, because their images
have no checksum.

**Both references hold off during the boot, differently.** Project64 applies at a VI only while the status register's
interrupt-enable bit is set (`N64System.cpp` 2400); mupen64plus skips the first sixty VIs (`gs_apply_cheats`). **Mars
takes Project64's rule**: the frame boundary applies cheats only while `Status.IE` is set, which the boot code never
sets. Measured at each frame end over 300 frames: IE is clear at frame 0 in all three games (and at frame 1 in Ocarina of
Time, whose boot is the longest), and after the game first sets it, clear at one frame end in 300 for Wave Race and for
Ocarina of Time and at none for Super Mario 64 — a critical section the frame happened to end in, whose cheats then land
a frame late. mupen64plus's sixty is not taken because it is a count rather than a condition: it holds every cheat for a
second to cover a boot that takes a sixteenth of one here, and it would not cover a boot longer than sixty fields.

`A_cheat_on_from_power_on_leaves_the_boot_checksum_alone` is the regression test. It needs the cartridge in
`TestRoms/n64` and passes vacuously without it; it failed against the ungated build with the processor at
`0x800001CC`, and passes with the gate. `The_frame_end_applies_nothing_while_interrupts_are_disabled` pins the gate on a
synthetic image, and the wiring tests' image now sets `Status.IE` before it spins, as a running game has.

**What the gate does not cover.** An explicit `ApplyCheats()` — Mistress's Apply button, through
`IDebugTarget.ApplyCheats` — is not gated, because it is the user's instruction and an Apply that silently does nothing
is `EmuSen_Cheats.md` §6's defect over again; pressed in the boot's first 61–76 ms of console time it can still break
it. The gate samples IE once, at the frame end. And a game that loads and checksums more code later would need the
same care; none is known, and it is untested.

## 6. ROM patches: supported, at the cartridge

**`MemoryBus.ReadCart32` is the one place a cartridge byte enters the machine after the handoff.** Processor loads
(`Load` → `CartridgeWord` → `Read32`) and the PI's transfers (`FromCartridge` → `CartridgeDmaRead8` → `Read8` →
`Read32`) both reach it, so one intercept there covers both — the arrangement `EmuSen_Cheats.md` §4 describes for Moon and
Mercury, installed by `LoadRom` and re-installed by the `Cheats` setter. Each of the word's four bytes is asked for by its
own ROM offset, and the address is that offset — the index into the big-endian image and into the debug target's `ROM`
space — so `cheat rompatch 001040 AD` patches what `peek ROM 0x1040` shows. `A_rom_patch_reaches_a_transfer_into_rdram`
is the test that matters: games copy their code out of the cartridge rather than run it in place.

**What a ROM patch means on this machine is narrower than on the NES.** It changes what the cartridge *says*, not what is
already in RDRAM: a patch takes effect at the next transfer that covers its byte, and code a game copied at boot stays
unpatched until the game copies it again. That is the hardware's semantics for a cartridge that answered differently,
and it makes a ROM patch a much blunter tool here than a RAM poke.

**Two places it does not reach.** The boot code's four kilobytes, which `Boot.HandOff` copies into DMEM straight from the
image, standing in for the PIF (`Mars_Boot.md` §1), before any registry is handed over; and — as consequence rather than
gap — a patch to the first megabyte in force while the boot code checksums it, which fails the checksum and stops the
boot, as it would on a console. `cheat rompatch` is the only way to make a patch for Mars.

**The cost is four calls per cartridge word**, each returning on one integer compare when no patch is enabled
(`CheatRegistry.TryPatchRom`), and the PI's transfers are the only bulk reader. Measured on a Release build, sixty
frames from power-on with the patcher installed and with it removed, alternating, after one warm-up pair: Super Mario 64
2,135 and 2,129 ms, Wave Race 64 2,767 and 2,763 ms, Ocarina of Time 6,518 and 6,419 ms — the last inside the 1% the
two unpatched Ocarina runs differed from each other by. The cycle counts were identical, as they must be. No cost is
distinguishable from noise at this resolution; a finer measurement was not made.

## 7. What the tests say, and what they do not

**`MarsCheatTests`** — 39 methods, 75 cases with the theories expanded — decode every applied type with literal
expected values, the refusals by the reason each names (twenty codes), the separators, and `CanDecode`'s shape rule; then,
on a `MarsCore` running a two-instruction spin, the bytes each code leaves in RDRAM — high byte first, the processor's own
`Read32` of them, tests guarding one write and no other, a run of tests as an AND, repeaters in both widths, a test before
a repeater, the edge of installed RDRAM with and without the Pak, the other writable spaces — the cartridge intercept for
a processor load and a PI transfer, the frame-end gate, `cheat add`, `cheat list` and `cheat export` through the shell,
and a hand-written database file. **`CoreFactoryCheatWiringTests`** gained the five Mars cases the other cores have:
a ROM patch and a RAM poke through the registry handed to `CoreFactory.Load`, a paused Apply through the debug target,
the bundle's own codec decoding a two-line code, and the path through `EmulatorSession`. **`CheatWriteModelTests`**
gained six for the test write itself. `MarsCoreTests`' two assertions that the bundle carried no codec now assert the
GameShark, and `ActiveCheatsWindowTests`' console-tab test asserts the N64 tab accepts it; three window tests add a
code, have one refused by name, and have a malformed one answered by the GameShark rather than a missing Game Genie slot
(`EmuSen_Cheats.md` §7).

**The breakage round: thirty-nine rules broken one at a time, and each reddened a named test.** In the codec: an `80`
written sixteen bits wide and an `81` eight; little-endian halfwords; an 8-bit value or test keeping its high byte; the
address mask keeping the type byte, and dropping bit 23; `D0` testing not-equal, `D2` equal, `D1` eight bits, `D3`
equal; the repeater's count read from bits 16–23, one short, swapped with the step; its value step dropped; a count of
zero accepted; any line accepted after it; an odd halfword accepted; a trailing test accepted; `88` and `F1` applied as
writes; `A0` refused; `DE` applied as a test; run-together lines not split; the `??` placeholder accepted. In the
registry: a passing test clearing an earlier failure, a failure outliving the write it skipped, a test ignoring its width
mask, a test applied as a write. In Mars: the frame end applying nothing, applying regardless of interrupts, gating on
`EXL` instead of `IE`; `LoadRom` installing no patcher; the setter not re-installing it; the four lanes of a cartridge
word patched in reverse; past-RDRAM reads returning `FF`; the debug target not forwarding Apply, or keeping its own
registry. In the import: a code string cut at its first `+`. The narrowest catches are single tests, and are worth
knowing: `The_type_byte_never_reaches_the_address` alone catches the dropped bit 23, `An_8_bit_test_compares_its_low_byte`
alone the test's high byte, and the refusal theory alone the count of zero, the odd halfword, the trailing test, the
accepted `F1` and the placeholder. `LoadRom` installing no patcher did *not* redden the wiring test, correctly: `Bundle`
hands a registry over and the setter installs the patcher itself, so only the tests that load a core directly see it.

**What none of this is evidence for.** No code was graded against a GameShark, so every rule in §2 is a reading of two
emulators, not a measurement (§0). The only commercial game a cheat has run on is Super Mario 64, for the boot question
in §5.1; that its infinite-lives value is still `0x64` after four frames shows the write landing and the boot surviving,
not that the game's lives counter is what that address holds. No libretro N64 `.cht` was imported (§5). The Controller
Pak, EEPROM and other save memories are not cheat targets; a code reaches RDRAM only, as the format's 24-bit address
does, and `cheat poke` reaches `DMEM`, `IMEM` and `PIFRAM` besides. And a database written in RetroArch's handler form
for this core would be read with its addresses in Mars's big-endian layout, where parallel-n64's RDRAM is host-endian
words — an address found by RetroArch's own cheat search would be off by the exclusive-OR of 3. No such file is on this
machine; this is noted, not tested.
