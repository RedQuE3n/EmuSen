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

**Mars runs at about a twelfth of the console's speed**, 5–8 million instructions a second against
93.75 million cycles. The games that run do so at that pace, with signal-processor and display work
inside it.

## 4. What this is not evidence for

- **That the frames are right.** Recognisable is not correct. Every rule that draws them is graded by
  its own slice; this page only shows that the rules compose into a picture a person would recognise.
- **Anything about input or saves, or what audio sounds like.** No button is pressed, the WAV is what the interface played rather than a comparison with a console, and nothing
  is saved.
- **Any game not in the folder, or any length of play past 150 seconds** — here, about ten seconds of
  the console's time.
- **Speed on another machine.** The numbers are this workstation's, in a Debug build, under the test
  host.
