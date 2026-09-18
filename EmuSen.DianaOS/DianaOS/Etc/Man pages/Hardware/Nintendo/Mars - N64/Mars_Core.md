# Mars — the core behind `ICore`, and what it stands in for

*Landed 2026-09-18. Not a Phase E slice: the wrapper that lets a frontend load a Nintendo 64 ROM at all. The
code is `MarsCore.cs` and `Debug/MarsDebugTarget.cs`, a route in `CoreFactory` and a descriptor in
`CoreCatalog`, one counter added to `Vi/Vi.Timing.cs`, one property made public in `Vi/Vi.cs`, and one guard in
`EmuSen/Common/RewindBuffer.cs`; the grading is `MarsCoreTests`, a mutation round, and two commercial games
measured through the interface.*

***§0 is the part to read first.** Mars is registered here ahead of the condition its own plan set for
registration, and most of what `ICore` asks for is a deliberate stub. §0 says which, and what each one means for
somebody who opens a `.z64`.*

---

## 0. What this is, and what it departs from

**This registers Mars before Phase F.** `Mars_Gameplan.md` §4.6 put the `CoreCatalog` descriptor last, after a
debug target with disassemblers, save states and cheats, on the argument that *"a core reachable from the
frontends before its seams are tested is a core whose seams get tested by the user."* `Mars_Rom.md` §4 said the
same thing from the other end. Registration has been brought forward ahead of those seams, so this page does the
next best thing to meeting the condition: it tests the seams that do exist (§9), and it lists the ones that do
not, below, so that nobody has to find them by using them.

| `ICore` asks for | What Mars gives | What a user sees |
| --- | --- | --- |
| a picture | the VI's raster, copied, line-doubled when progressive, alpha set to 255 (§2) | the game's picture, several times slower than real time (§9) |
| a frame boundary and a rate | one VI field per frame, a cycle cap when the VI is idle (§3) | nothing; pacing follows the console's own clock |
| audio | ~~**nothing** — there is no audio interface (§4)~~ what the game plays, at the rate it set (§4) | ~~silence~~ the game's sound |
| input | ~~nine of the pad's fourteen buttons, and not its stick (§5)~~ every button, the stick, and the C buttons on the right stick (§5) | ~~no analog stick, no Z, no C buttons~~ the whole controller |
| battery saves | ~~**nothing** — there are no save devices (§6)~~ the cartridge's chip in `<rom>.srm` and a Controller Pak in `<rom>.mpk` (§6, `Mars_Save.md`) | ~~progress is lost when the ROM is closed~~ progress kept between runs |
| save states and rewind | ~~explicit saves refused, rewind given no history (§6)~~ the whole machine, 6.4MB a state (`Mars_SaveStates.md`) | ~~"Save State failed", and holding rewind freezes the picture~~ saving, loading and rewinding |
| a debug target | memory spaces, register readouts, a summary; nothing that halts (§8) | `watch`, `bp` and `framelog` accept arguments and never fire |
| cheats | the database folder is kept; no codec applies anything (§8) | the N64 cheat tab says it takes no format |

**What none of this is evidence for is that any game is playable.** Two commercial games reach their title
pictures through this interface (§9); ~~neither can be steered, heard or saved~~.

> **Update 2026-09-18: the table's first three struck rows were stale for a slice each.** The audio and input slices
> rewrote §4 and §5 and left this table saying *"nothing"* and *"no analog stick"*; the save slice found them while
> retiring the third. Both games can now be heard, steered and saved (`Mars_Audio.md`, `EmuSen_Input.md` §7,
> `Mars_Save.md`), and Super Mario 64 answers a pressed Start (`Mars_GameProbe.md` §5). Playable is still not the
> claim: at several times slower than the console, a game can be reached but not played.

## 1. What `MarsCore` is

**`LoadRom` is the handoff every Mars test already uses**: `RomImage.Load`, a new `MemoryBus`, a new `Cpu`, and
`Boot.HandOff` (`Mars_Boot.md`), so no firmware image is asked for and `GetFirmwareRequirements` keeps its
default of none. `Rom`, `Bus` and `Cpu` are public properties beyond the interface, the same escape hatch
`Moon_Core.md` §1 describes, and the debug target is built on them.

**`CoreName` is `"N64"`**, which is also the descriptor's console name. The two must be the same string —
Mistress assigns one to the other (`EmuSen_Input.md` §5.1) — and `MarsCoreTests` and `ConsoleBindingsTests`
both pin it.

## 2. The picture, and the size contract

**What every consumer assumes is that `GetFrameBufferRgba().Length == ScreenWidth × ScreenHeight × 4`, read
back to back after `RunFrame`.** `ICore.cs` says so in its comment on the method, and the consumers depend on
it rather than check it: Mistress reads the buffer and then the two properties (`MainWindow.axaml.cs`
1026–1027), Hotaru reads all three in one call (`GameWindow.axaml.cs` 431), `BmpFile.Write` indexes the buffer
to `width × height × 4` without a bound (`BmpFile.cs` 30–40), and Serenity hands the array to
`SKImage.FromPixelCopy` under an info built from the two dimensions (`GameFrameControl.cs` 128–129). **What
they do not assume is that the size is fixed**: every one takes the dimensions per frame, because Venus's width
already changes between 256 and 512 (`Venus_PPU.md` §8), and Venus keeps the contract by allocating a buffer of
exactly `FrameWidth × 224 × 4` on every call (`Renderer.cs` 49).

**So the answer is a variable height that is set together with the buffer it describes.** `ScreenWidth` is 640,
the VI's raster width. `ScreenHeight` is **480 for an NTSC signal and 576 for a PAL one**, and both it and the
buffer are replaced in the same method at the end of `RunFrame`, so no caller on the emulation thread can read
one without the other. Before a ROM is loaded, and after one is loaded but before its first frame, the
buffer is 640×480 of opaque black, which is what the VI describes before a game programs it.

**A progressive field is line-doubled; an interlaced one is copied as it is.** The VI's own frame is 240 or 288
rows for a progressive signal (`Vi.FrameHeight`), and every consumer presents a buffer at its own pixel aspect —
`ComputeLetterboxRect` scales width and height by the same factor (`GameFrameControl.cs` 39–51) — so a 640×240
buffer would be drawn at 8:3, squashed to half its height. Each raster row is therefore written twice. An
interlaced signal already holds both fields in its raster (`Mars_Video.md` §2.4), so it is copied row for row.

**Two consequences, one intended and one not.** The intended one is that the height does not change when a game
moves between an interlaced menu and progressive play; it changes at most between 480 and 576, which in
practice happens once, when a PAL game first programs `VI_V_SYNC` (§9 measures Super Mario 64 doing exactly
that between its first and second frames). The unintended one is that **a PAL picture is drawn 20% too tall**:
640×576 at square pixels is 10:9, where a PAL set shows 4:3. `ICore` has no channel for a pixel aspect, and
Venus has the same class of error (256×224 drawn at 8:7), so this is recorded rather than worked around.

### 2.1 The fourth byte is coverage, not opacity

**The VI's raster keeps each pixel's coverage — zero to seven — where an RGBA buffer keeps alpha**
(`Vi.Scanout.cs` 211, `Mars_VideoFilter.md` §1). Everything downstream reads that byte as alpha: Serenity and
`PngFile` both build their images as `Rgba8888` with `Unpremul` alpha (`GameFrameControl.cs` 128,
`PngFile.cs` 10), and `BmpFile` writes it straight out. Passed through unchanged, a correct picture is drawn at
seven parts in 255 of opacity. That was measured before this page existed, on Super Mario 64 and Wave Race 64
frames: sky `(45,99,179)`, water `(0,74,246)`, alpha 7.

**Mars copies the raster and writes 255 into every fourth byte.** The raster itself is left alone, because it is
what `MarsViDifferentialTests` grades and its coverage is real data there.
`Every_pixel_is_opaque_whatever_coverage_the_raster_carries` pins both halves: the raster it scans really does
carry a coverage of 7, and the buffer handed out carries 255 over exactly the colours the raster holds.

### 2.2 When the raster is scanned

**Once a frame, at the boundary that ended it.** `RunFrame` calls `Vi.Scan()` after its loop, which is the
moment the half-line count wraps, or the moment the cap ran out if the VI is not counting. A console scans continuously down the field, and nothing here says the origin
register a game has written by the wrap is the one hardware would have latched for that field; a game that
swaps its frame buffer mid-field would show the difference, and no test here has one.

**`SkipRendering` skips the scan and the copy, and nothing else.** The display processor still draws, because
what it writes to RDRAM a game can read. What goes stale is the VI's held-line bookkeeping
(`Mars_Video.md` §2.4), which no game can read either — the kind of render-derived state
`EmuSen_Rewind_And_FastForward.md` §2.2 allows fast-forward to leave behind.

## 3. The frame boundary

**A frame is one of the VI's fields.** `Vi.Timing.cs` already advanced the half line off the bus clock and wrapped
it at `VI_V_SYNC` (`Mars_VideoTiming.md` §1); this page adds `Fields`, a count incremented where it wraps, and
`RunFrame` steps the processor until that count changes. **That counter is the only change to what the VI
computes, and it changes nothing the VI does.** The other change inside Mars, making `Vi.Serrate` public for §2's
doubling, alters no behaviour either.

**A cycle cap ends a frame the VI never will.** Before a game programs `VI_V_SYNC` and `VI_H_SYNC`, the VI does not
count at all (`Mars_VideoTiming.md` §4), so a frame ending only on a field would never end. `RunFrame` also
stops after `CycleCap` = 4,100,000 processor cycles.

**The cap is set by the longest field the registers can describe, not by a nominal frame rate.** `VI_V_SYNC` has
ten bits and `VI_H_SYNC` twelve. Above 550 half lines the VI takes the PAL clock, so the longest field is 1023
half lines of 4095 interface clocks at 49.65653 MHz — **3,954,526 processor cycles**. At or below 550 half lines
it takes the NTSC clock, and the longest is 2,168,658. Four million one hundred thousand is above both, so **the
cap can only end a frame the VI would not have ended**. `The_cap_never_ends_a_frame_the_vi_would_have_ended`
programs the 1023-by-4095 case and checks the frame still ends on the field.

**The first design considered was wrong in a way worth recording.** A cap of one nominal NTSC frame, 1,562,500
cycles, was it, and it would have cut every PAL field short: a
PAL field is 1,874,400 cycles (§9), so the frame boundary would have fallen mid-field every frame, and §2.2's
scan with it. A cap chosen from the region in the ROM header is no better, because the header is not evidence of
the signal (`Mars_Rom.md` §2.2).

**What the cap costs is a slow boot.** A frame the cap ends is 43.7 ms of console time, so while a game has not
yet programmed its VI the core reports 22.87 Hz. Both commercial games measured spend exactly one frame that way.

**`FrameRateHz` is the rate of the frames `RunFrame` actually produced**: the processor clock over the cycles
the last frame took. It is 22.87 Hz before the first frame and during a capped one, and afterwards whatever the
VI's registers make it — 59.959 Hz for NTSC's 525 by 3093, 50.016 Hz for PAL's 625 by 3177. The first field
after a game programs its VI is a partial one (Wave Race's is 3,842,644 cycles, 24.4 Hz). **That is correct
rather than noise**: both frontends read the rate afresh before every frame (`MainWindow.axaml.cs` 969,
`GameWindow.axaml.cs` 388), so each wait is exactly as long as the console time the last frame emulated. The
boundary is instruction-granular, and the measured field lengths move by one cycle between frames.

**None of this establishes a console's rate.** 59.959 Hz is what `Mars_VideoTiming.md` §1.1 derives from three
published clock frequencies, and that page says in its §0 that nothing has measured it against hardware.

## 4. Audio: what the game plays, at the rate it asked for

*Rewritten 2026-09-18, when the audio interface landed (`Mars_Audio.md`). The version it replaces is kept below.*

`DequeueAudioSamples` drains the audio interface's queue — interleaved left and right, whole pairs, destructively — and
`AudioSampleRate` is the rate the game set the DAC to, rounded to the hertz (`Mars_Audio.md` §4). Before a game sets a
rate it is 44,100, which only opens a device.

**The version below saw a fork coming and named it**: when the interface landed it *"will have to resample to a fixed
rate or the contract will have to change"*. **The contract gave.** `AudioPlayer.Submit` already compares the rate it is
handed with the open device's and reopens on a mismatch (`EmuSen_Audio_Sync.md` §7.2), so a Nintendo 64 whose game sets
a rate at boot costs one reopen before any sample arrives, and a game that changes rate later costs another. A
resampler inside Mars would have stood in front of the one the sink already runs for drift. `ICore`'s comment on the rate
now says a core may change it when its machine does.

**What that version said about silence still holds.** The interface produces nothing while nothing plays, rather than
zeros the machine never made, so `audiosum` and `audiodump` count only what a game played.

**Wave Race's unchanging lit-pixel count from its eighty-third frame (§9) is still not traced.** The game probe shows it
handing the video interface new frames across five seconds of console time with the audio interface present
(`Mars_Audio.md` §5), which says the game runs; it does not say what that count was measuring.

> **Retired 2026-09-18: "Audio: nothing, at 44100 Hz".** It returned no samples and a provisional 44,100, argued from
> the frontends' wall-clock pacing, the sink's early return on an empty payload and `ICore`'s own permission that an empty
> queue stalls nothing, and rejected a queue of zeros as samples the machine never produced. It ended: *"What an absent
> audio interface does to a game is not established here — a game that waits on the audio interrupt may stall."* With
> the interface present both games that ran before still run, which settles it for those two and not in general.

## 5. Input: every input the controller has

*Rewritten 2026-09-18, when the generic controller template gave the contract an analog path (`EmuSen_Input.md` §7).
The version it replaces is kept at the end of this section.*

Mars takes the N64 controller off the generic template — the RetroPad's buttons and its two sticks:

| template | N64 controller | joybus (`Mars_Serial.md` §3.1) |
| --- | --- | --- |
| A, B, Start | A, B, Start | `0x8000`, `0x4000`, `0x1000` |
| Up, Down, Left, Right | the D-pad | `0x0800`, `0x0400`, `0x0200`, `0x0100` |
| L, R | L, R | `0x0020`, `0x0010` |
| **L2** | **Z** | `0x2000` |
| **left stick** | the stick | the two signed bytes, ±127 at full tilt |
| **right stick** | the four C buttons | `0x0008` up, `0x0004` down, `0x0002` left, `0x0001` right, each past half its travel |

The choices, and what each rests on:

- **Z on L2.** Z sits under the left index finger on the N64's own controller, which is where a modern pad's left
  trigger is. A user can bind anything else to L2.
- **The C buttons on the right stick.** They are four digital buttons in a cross on the controller's right. In Super
  Mario 64 they turn the camera, which is the right stick's job on a dual-stick pad; other games give them other jobs —
  Ocarina of Time puts items on them — and the mapping serves those equally, since it is only four directions standing
  for four buttons. Each is pressed once the stick is past **half** its travel on that axis; nothing measures that
  threshold, it is a choice, and it is a named constant so the choice is visible.
- **The stick reaches ±127.** The joybus carries it as a signed byte, and full tilt maps to the byte's full reach, as
  Project64's SDL input backend does (`MAX_AXIS_VALUE` 32767 over `N64DIVIDER` 258, `Project64-input/SdlInputBackend`)
  and as N-Rage's `N64_ANALOG_MAX 127` declares. A stock controller's stick is widely said to travel less than that;
  nothing in reach measures it, so Mars takes what the implementations in reach do and says so.
- **Up is positive on the N64**, and the template's stick is down-positive like the RetroPad's, so Mars turns the Y axis
  over on the way in.
- **X, Y, Select, R2, L3, R3 and both triggers' analog travel are dropped**, never moved onto another input — the rule
  `EmuSen_Input.md` §2 set for Moon.

**`port` indexes `Si.Controllers` directly.** Only the first port holds a controller (`Mars_Serial.md` §3.1), so a
press on port 1 is stored in a controller no game can see. A port outside 0–3 is ignored. The state lives on the
bus, so it resets when a ROM is loaded.

> **Retired 2026-09-18: "Input: nine of fourteen".** It mapped A, B, Start, the D-pad, L and R, and said that Z, the
> four C buttons and the stick *"cannot be reached at all, because `PadButton` has no member for them"*, so that *"a
> game that moves its character with the stick cannot be played"*, which included Super Mario 64. It was right, and it
> named the fix — *"a real addition to the contract rather than a fudge through the D-pad"* — which is the one
> `EmuSen_Input.md` §7 made.

## 6. Saves: ~~none of either kind~~ the battery kind, and not the state kind

~~**`SaveSram` does nothing**, because there is no save device to flush — no EEPROM, SRAM, FlashRAM or Controller
Pak (`Mars_Serial.md` §6). It writes no file beside the ROM, which `Flushing_save_data_writes_nothing` pins. A
game that saves loses it when the ROM is closed.~~

> **Retired 2026-09-18 by the save slice** (`Mars_Save.md`). `SaveSram` writes whatever the game changed — the
> cartridge's chip to `<rom>.srm` and the Controller Pak to `<rom>.mpk`, both in the data store's `Saves` folder
> rather than beside the ROM — and `MarsCore` does the same every 300 frames. `Flushing_save_data_writes_nothing`
> still passes, and now pins the narrower thing its name can still say: nothing changed, nothing written.

> **Retired 2026-09-18 by the save-state slice** (`Mars_SaveStates.md`). Everything from here to the end of the
> section describes the refusal that stood until then, and is kept because its argument about the two callers still
> explains why a path save checks for a ROM before it creates a file. The stream overloads now write and read the
> machine, the rewind buffer gets real history, and the tests that pinned the refusal were replaced.

~~**Save states are refused two different ways, because their callers fail two different ways.**~~

- **The path overloads throw `NotSupportedException` before touching the filesystem.** They are what an explicit
  save reaches, and both frontends catch and report the failure — Mistress's status bar says "Save State failed"
  (`MainWindow.axaml.cs` 478–480), Hotaru prints "[STATE] Save failed" (`GameWindow.axaml.cs` 539–541). Refusing
  before `File.Create` matters: a zero-byte `.state` that loaded as nothing would be a save that silently kept
  no progress.
- **The stream overloads write nothing and read nothing.** Their only routine caller is `RewindBuffer`, which
  both frontends enable by default (`MainWindow.axaml.cs` 70, `GameWindow.axaml.cs` 108) and feed from inside
  the emulation loop's `try` (`MainWindow.axaml.cs` 1007, `GameWindow.axaml.cs` 423). The `catch` stops the loop
  in Mistress (1056–1058) and closes the window in Hotaru (448–451). **A throwing stream overload would end every
  Mars session on its fourth frame.**

**This departs from `ICore`'s own comment**, which says the stream overloads write "the same bytes as the path
overloads" (`ICore.cs` 97–98). The departure is chosen rather than overlooked: an explicit save should fail
loudly and a routine snapshot must not fail at all, and one behaviour cannot be both.

**Recording nothing exposed a defect in `RewindBuffer`**, fixed here and described in
`EmuSen_Rewind_And_FastForward.md` §1.7: an empty snapshot was stored as history. Before the fix, 600 frames of
Wave Race left 149 empty deltas and 400 of Super Mario 64 left 99, and `Rewind()` answered true for each while
restoring nothing. **What a user sees is the same before and after** — holding rewind runs no frames, so the
picture holds — and what changed is the memory and the harness's `rewind` report, which no longer counts steps
that were never taken.

## 7. The Expansion Pak: off

**`CoreFactory` builds a 4 MB machine.** `Mars_Gameplan.md` §6 defers "the Expansion Pak as a default" until
something concrete needs it, and the commercial-game tests (`MarsCommercialRomTests`, `MarsMicrocodeTests`) all
run on 4 MB. The corpus harness does use the Pak, but because the corpus needs it (`Mars_Corpus.md` §8), which is
a fact about the corpus and not about the games a frontend opens. `new MarsCore(expansionPak: true)` exists for a
caller that wants 8 MB; nothing in the frontends passes it.

**What this does not settle is what a game that needs the Pak does.** A stock console shows such a game's own
"Expansion Pak required" screen; whether Mars gets that far, and whether a game's boot code detects 8 MB when
the Pak is on, has not been run.

## 8. The debug target and the catalogue

**A debug target is not optional**, although Phase F is where the plan put one. `CoreFactory.Bundle` throws for a core without one (`CoreFactory.cs` 91–92), `CoreBundle.DebugTarget` is not
nullable, and every frontend loads through it — Mistress by `EmulatorSession.LoadRom` (`EmulatorSession.cs` 49),
Hotaru (`GameWindow.axaml.cs` 321), Pharaoh (`Program.cs` 233). A route to `MarsCore` with no target would have
made every frontend throw on every `.z64`.

**`MarsDebugTarget` is the least that satisfies the interface.** It has five memory spaces — `RDRAM`, `DMEM`,
`IMEM`, `PIFRAM` and a read-only `ROM`, each the machine's own array with no side effects — the processor's
program counter, 32 registers under their o32 names, `HI`, `LO` and the cycle count, the fourteen VI registers,
and a summary. Everything else is empty, and the commands built on it say so: `disasm` reports "N64 target has
no disassembler", and `coretop` leaves out its tile sheet because `TilemapEntryStride` is 0.

**The stub most likely to mislead is the three registries.** `Watches`, `FrameLog` and `Breakpoints` exist so the
commands that use them do not fail, and nothing feeds them: `watch`, `framelog` and `bp` accept their arguments
and never fire. They become real with the rest of Phase F (`Mars_Gameplan.md` §4.6).

**No cheat codec is bundled**, so the Active Cheats window's N64 tab says the console has no cheat-code format and
disables its Add button (`ActiveCheatsWindow.axaml.cs` 199, 267). A `.cht` loaded from the database still
lands in the registry as raw pokes, and nothing applies them. **The libretro folder `Nintendo - Nintendo 64` is
claimed anyway**, so that `cheat db prune` keeps data a later build will use instead of deleting it.

**The descriptor is `Nintendo 64 (Mars)`**, console `N64`, Nintendo, 1996, claiming `.z64`, `.n64` and `.v64`. All
three are claimed because the container is decided by the magic word and not by the extension
(`Mars_Rom.md` §1.1), which `The_extension_does_not_decide_the_byte_order` pins with a little-endian image named
`.v64`. **Registering changed five things at once**, as `EmuSen_Multicore.md` §3 warns: N64 ROMs now appear in
Mistress's library and file picker, the input and cheat windows grow an N64 tab (and the three tests that
listed the consoles now list four), DianaOS accepts `core mars` and `core n64`, and logs go to an `N64` folder.

## 9. What the tests say, and what they do not

**Twenty-two tests in `MarsCoreTests`**, all on synthetic images except one. They cover the route for all three
extensions, the catalogue entry, a frame ended by the cap, a frame ended by a field and its rate, the cap against
the longest describable field, the size contract from no ROM through progressive NTSC, PAL and interlaced
signals, the opacity of every pixel against a
raster carrying coverage 7, a skipped frame, the joybus reporting what `SetButton` set, the buttons that are
dropped, both kinds of refused save state, the rewind buffer left without history, the empty audio queue, the
unwritten save file, the stock RDRAM size, the members that need a ROM, and the debug target's memory spaces.

**The rewind test failed before the fix it describes**, with a depth of 4 after five captures; that failure is the
evidence for §6's defect, not a reconstruction of it.

**Six mutations, each caught.** Keeping the raster's alpha failed the opacity test; dropping the line doubling
failed it and the size contract; ending frames on the cap alone failed both field tests. The last three — a cap
of one nominal PAL frame, a throwing stream save, and B wired to Z's bit — were applied together and failed four
tests, each of which only one of the three could reach: the longest-field test, the two save-state tests that
read the stream, and the joybus test.

**Two commercial games were measured through `CoreFactory.Load` outside the suite**, with the corpus read in place:

| | Wave Race 64 (USA) | Super Mario 64 (Europe) |
| --- | --- | --- |
| frame 0 | cap, 4,100,000 cycles, 640×480 | cap, 4,100,000 cycles, 640×480 |
| frame 1 | first field, 3,842,644 cycles | first field, 3,649,626 cycles, now 640×576 |
| every frame after | 1,563,557 cycles, 59.959 Hz | 1,874,399 or 1,874,400 cycles, 50.016 Hz |
| frames run | 600, 10.06 s of console time | 400, 8.04 s of console time |
| wall clock, Release build | 40.2 s, four times slower than the console | 43.3 s, about five and a half times slower |
| first frame with a lit pixel | 3 | 61; Mario's head on the title screen by 353 |
| changes of height | none | one, 480 to 576, at frame 1 |
| frames with a length mismatch, or an alpha other than 255 | none | none |
| samples dequeued | 0 | 0 |
| rewind depth at the end | 149 before §6's fix, 0 after | 99 before, 0 after |

Both games were run twice, before and after the rewind fix. The cycle counts, rates, heights and first lit
frames were identical between the two runs and the wall clocks within 3%; the whole-run checks for length and
alpha were made on the second run only.

**`Wave_Race_reaches_a_picture_through_the_interface_alone`** is the one test on a real game: it runs until the
first lit frame and checks the size contract on every frame, 480 lines, 59.95–59.97 Hz, and opacity. It passes
against the corpus in two seconds, and **it returns early where the corpus is absent** — in a checkout without
`TestRoms/n64` it is vacuous, and so are the two `MarsMicrocodeTests` that share its corpus. All three were run
against the corpus after the `Fields` counter went in, through a runner outside the repository that points
`N64TestRomLibrary.Root` at the corpus without copying it, and pass.

**What none of this establishes:**

- **That the picture is what a console shows at that frame.** This page copies the raster; the raster is graded
  in `Mars_Video.md`, and §2.2's moment of scanning is not graded anywhere.
- **The rate of a console** (§3).
- **That interlaced presentation is right for a game.** The size contract is tested with an interlaced signal;
  no interlaced game has been run.
- **That anything is playable** (§0).
- **Whether Wave Race's picture holds still after frame 83.** Its lit-pixel count reaches 245,622 there and is
  the same in every frame sampled up to 600, which suggests a still picture without showing one. If it is still,
  the missing audio interface is one candidate and the attract sequence another; neither has been looked at.
