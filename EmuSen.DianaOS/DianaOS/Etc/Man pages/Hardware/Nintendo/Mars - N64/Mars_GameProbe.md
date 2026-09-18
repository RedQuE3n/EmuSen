# Mars — the game probe: how far a commercial title runs

*Added 2026-09-18. `EmuSen.WiseMan/Cores/MarsGameProbeTests.cs`. A measurement, not a test: it
asserts nothing, and it does nothing unless it is asked to.*

---

## 0. What it is for

Every other Mars instrument grades a rule. This one answers a different question — *how far does a
real game get, and how fast?* — which no rule-level test can, because a game runs only as far as its
weakest unbuilt device lets it. It exists so that the question can be asked again after each slice,
with the same numbers, instead of being answered by impression.

## 1. Running it

```
EMUSEN_MARS_PROBE=<output directory> dotnet test EmuSen.WiseMan --filter "FullyQualifiedName~MarsGameProbeTests"
```

- **`EMUSEN_MARS_PROBE`** names the directory the reports go to. Unset, the probe returns at once,
  which is why it can sit in the suite: it costs a normal run about five milliseconds. The name follows
  `EMUSEN_AUDIO_DUMP`, the other opt-in dump the harness has.
- **`EMUSEN_MARS_PROBE_SECONDS`** is the wall-clock time each game gets, 150 by default.
- **What it runs:** every `.z64` in `N64TestRomLibrary.Root` — the sandbox's `TestRoms/n64` — in name
  order, each on a fresh machine through `Boot.HandOff`. The ROMs are read, never written.

For each game it writes `<name>.txt`; `<name>.png` if the video interface is scanning out; and, since the audio
interface landed, `<name>.wav` of everything the game played, drained as the run goes.

## 2. What each line of the report means

- **stopped by** — the wall-clock limit, or the exception that ended the run.
- **speed** — instructions executed, instructions a second, and how much of the console's own time that
  was (the bus clock over 93.75MHz).
- **signal processor tasks** — tasks started, by the type word the operating system leaves in DMEM at
  `0xFC0`: 1 is graphics, 2 is audio.
- **framebuffers handed to the video interface** — how often `VI_ORIGIN` changed, sampled every 1,024
  instructions, with the first twelve times and addresses. A game that has finished a frame hands the
  interface a new buffer, so this is the plainest count of frames the game believes it drew.
- **distinct addresses over the next two million instructions** — a few thousand is a game in its main
  loop; a handful is a spin on a flag; two million is the processor walking through memory that holds
  no program.
- ~~**audio registers** — the six the audio interface would own. No device models them yet (§4); they
  read back what the game last wrote, so a nonzero line means the game is feeding audio to nothing.~~ **audio**
  (since 2026-09-18) — the stereo samples the audio interface played, the rate the game set, how many that is a
  second of console time, and the status register (`Mars_Audio.md`).
- **save chip** (since 2026-09-18) — the chip the game's own first move named, whether it was written, and whether the
  pak in the first port was (`Mars_Save.md` §1, §8). The probe runs the machine a frontend gets: no earlier save, and
  a freshly formatted pak.
- **picture** — the frame `Vi.Scan()` produces, saved as a PNG. **The raster's fourth byte is the
  pixel's coverage, not an opacity**, so the probe sets it to 255 before saving; without that, every
  frame looks bleached, because viewers composite a coverage of 7 as nearly transparent. The picture is
  one field at the raster's own 640 columns, so it looks stretched: a display would double it vertically.

## 3. The first measurement, 2026-09-18

After `a4fb0b9`, 150 seconds a game:

| | Super Mario 64 (Europe) | Wave Race 64 (USA) | Ocarina of Time (Europe) |
| --- | --- | --- | --- |
| instructions a second | 5.96M | 5.34M | 7.67M |
| console time covered | 9.58s | 9.01s | 12.30s |
| graphics / audio tasks | 190 / 452 | 236 / 519 | 0 / 0 |
| framebuffers handed over | 187 | 155 | 0 |
| distinct addresses, last 2M | 9,198 | 35,348 | 2,000,000 |
| picture | the title screen's face | the attract scene and logo | none |

**Two of the three run their main loops and draw recognisable frames.** Super Mario 64 shows Mario's
head over its tiled logo, and Wave Race its logo over the attract scene. Both cycle three framebuffers,
Super Mario 64 every 0.04 seconds, which is 25 frames a second on a 50Hz console. Both write to the
audio registers and carry on, so the missing audio interface does not stop either of them this far.

> **Update 2026-09-18: with the audio interface present**, both still run, and play 32kHz stereo from about a second
> in — 175,645 and 169,430 samples over 5.7 and 5.5 seconds of console time. `Mars_Audio.md` §5 has the numbers and what
> the recordings show.

**Ocarina of Time does not boot.** It starts no task and hands over no frame, and by the end the
processor is executing through physical `0x02E890F4`, past the end of RDRAM, where Mars reads zeroes:
two million instructions at two million addresses. Something early in its boot sent it somewhere with
no program. It is the only one of the three on the CIC-6105 boot chip, and Mars has two known gaps
that only a 6105 title would meet: `Boot.HandOff` leaves the seed `0x3F`, the 6102's, where the 6105's is
`0x91` (`Mars_Boot.md` §2 already calls those values a suspect), and Mars's PIF answers no CIC challenge,
which `Mars_Serial.md` deferred to the save devices. Those are suspects, not a diagnosis.

> **Diagnosed and fixed, 2026-09-18 — `Mars_Boot.md` §8.** Neither suspect was the first failure. The game
> stopped at its boot code's *third* instruction, which reads a register (`t3`) that IPL2 leaves and
> `Boot.HandOff` did not; the physical address above was only how far the processor had slid through empty
> memory after the resulting fault. The seed was the third thing it would have met, and necessary. The CIC
> challenge was not needed at all: the game never asks for it in 42.86 seconds of running. With the handoff
> fixed, the probe finds it at its title screen — 891 graphics tasks and 564 framebuffers in 25.19 seconds
> of console time, in a Release build (`Mars_Boot.md` §8.3 has all three games' numbers).

**Mars runs at about a twelfth of the console's speed**, 5–8 million instructions a second against
93.75 million cycles. The games that run do so at that pace, with signal-processor and display work
inside it.

## 4. What this is not evidence for

- **That the frames are right.** Recognisable is not correct. Every rule that draws them is graded by
  its own slice; this page only shows that the rules compose into a picture a person would recognise.
- ~~**Anything about input or saves, or what audio sounds like.** No button is pressed, the WAV is what the interface played rather than a comparison with a console, and nothing
  is saved.~~ **What audio sounds like, or whether a save is what a console would write.** Input and saves are in
  the report since 2026-09-18 (§2, §5), but the WAV is what the interface played rather than a comparison with a
  console, and a save is only ever read back by Mars.
- **Any game not in the folder, or any length of play past 150 seconds** — here, about ten seconds of
  the console's time.
- **Speed on another machine.** The numbers are this workstation's, in a Debug build, under the test
  host.

## 5. The controller check

`MarsControllerProbeTests`, opt-in by the same variable, is the last of the three things `Mars_Gameplan.md` §4.5 asks
of Phase E — *"a controller able to reach the game"*. It runs a game twice through `MarsCore`, the interface a frontend
drives, and presses Start in the second run only, through `SetButton`:

| | Start held | from frame | compared at | frame before the press | compared frame |
| --- | --- | --- | --- | --- | --- |
| Super Mario 64 (Europe) | ten frames | 700 | 800 | identical | different, in 886,224 of its bytes |
| Wave Race 64 (USA) | ten frames | 300 | 400 | identical | different, in 304,880 of its bytes |

**The frame before the press is the control.** Identical there, the two runs can only differ afterwards because of the
press. What each game shows for it: Super Mario 64 leaves Mario's head for a menu screen — the sound and language
select, which may be what the European version shows on a first boot with a blank EEPROM, and was not checked against
a console — and Wave Race 64 leaves its attract race for its title screen, *"Press START to begin"*.

**The check failed twice before it passed, and each failure was worth more than the pass.**

1. **The control failed.** The first version ran both boots in one save folder. Super Mario 64 writes its EEPROM at
   frame 61, the core's every-300-frames write saved it, and the second run booted with that save — different from
   the first run before any button was pressed. Mars itself was deterministic; the check was not. Each run now has its
   own folder (`Mars_Save.md` §8).
2. **The press changed nothing, and that was a Mars bug.** With the control fixed, Start held at frame 400 left frame
   520 identical. The reply in PIF RAM read `00 00` while Start was held: the serial slice ran the controller's block
   when it was written to PIF RAM, and a game that writes it once and reads it every frame got the buttons from that
   one write forever. The referee runs the block when PIF RAM is read out; Mars now does too (`Mars_Serial.md` §2).
3. **The press reached the game, and the game was not yet listening.** With the reply reading `10 00`, the Start bit
   reached the game's own pad state — the byte `0x80309262`, which reads as its player-one button-pressed edge, held
   `0x10` for one game frame — and Super Mario 64 still did nothing at frame 400. Pressed at frame 700 it answers within
   six frames. Where between the two it starts listening was not located; the head is on screen by frame 350, so the
   screen being visible is not the same as its input being read.

**What this is evidence for** is that a button pressed through `ICore` reaches a commercial game's logic and changes
what it does, in two games. **What it is not evidence for** is the timing of input: which frame a press is first seen
on, and whether a game polling faster than the joybus replies would see a stale one, are untested.
