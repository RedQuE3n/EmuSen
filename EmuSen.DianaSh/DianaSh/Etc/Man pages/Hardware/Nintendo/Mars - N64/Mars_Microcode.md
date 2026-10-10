# Mars — the first commercial microcode, and the devices it waited for

*Landed 2026-09-17. Phase C's second "done when" condition — a commercial game's boot
microcode running to the point of emitting a display list — measured against the two
cartridges this project runs. The test is `MarsMicrocodeTests`; nothing in the core changed
for it. §5 is the reason to read this page: the condition could not be met by Phase C's work
alone, and the plan did not say so.*

---

## 1. The result

**Wave Race 64 runs its first graphics task to a display list, and hands that list to the
display processor one command at a time**: seventeen commands that set a colour image and a
Z image, fill a rectangle, and end in a full sync (§4). It does so only
when the harness raises two interrupts that no device in Mars can yet raise — the video
interface's and the serial interface's (§3). Without either, no graphics task is ever
submitted.

Super Mario 64 runs audio microcode under the same conditions and never submits a graphics
task within the budgets tried. That was not investigated further (§6).

## 2. How it was found, one measurement at a time

Each step below is a run, not a reading; the numbers are from those runs.

1. **As built: the RSP is never started.** Fifty million instructions each, and neither game
   clears the RSP's halt bit once. Both have unmasked all six RCP interrupt sources by then,
   the video interface's included, and both have programmed the video interface to interrupt
   at line 2. Mario then sits in its idle loop with interrupts enabled. This is the
   measurement behind the inference `Mars_FpuMath.md` §10 made from the loop alone.
2. **With a video interrupt pulsed once every 1.5 million instructions, the RSP runs.** In 150
   million instructions Mario starts it 89 times and Wave Race 88. Every task is the same
   kind — the word libultra's task loader leaves at DMEM `0xFC0` reads `2`, which the SDK's
   headers name an audio task — and every one runs to a `BREAK` with signal 2 raised. The
   display processor's start and end registers are never written.
3. **With a serial interrupt pulsed as well, Wave Race submits a graphics task** — the word at
   `0xFC0` reads `1` — at 63.7 million instructions. The microcode sets the display
   processor's start register, then advances its end register eight bytes at a time, and the
   task ends in a `BREAK`. Mario still submits only audio tasks.
4. **The words between start and end decode as display processor commands** (§4).

The serial interrupt matters because a libultra game's main loop reads its controllers every
frame and waits for the serial interface to say the read is done. That is a reading of why
step 3 differs from step 2, consistent with it but not separately tested.

## 3. The stand-ins, and what they are not

The test raises the video interrupt every 1.5 million CPU instructions and clears it 5,000
later, and does the same for the serial interrupt 700,000 instructions into each period. **The
numbers are arbitrary.** They were chosen to be roughly a frame apart and long enough for a
handler to run, and nothing about them is a measurement of the video interface's timing, of
when a controller read completes, or of anything Phase E will model. The serial "interrupt"
answers no exchange at all: PIF RAM is plain storage in Mars, so a controller read gets back
exactly what the game wrote to ask for it.

> **All of this was settled on 2026-09-17, and the prediction below held in full.** The serial half followed
> the video half the same day: `Mars_Serial.md` built the serial interface and the PIF's joybus, the serial
> pulse was deleted too, and **this test now runs with no stand-ins at all** — both of its assertions pass
> with two real devices where there were two arbitrary periods. What follows is the record of the video half,
> which came first.
>
> **The video half, 2026-09-17.** `Mars_VideoTiming.md` built
> the video interface's clock and interrupt, the video pulse was deleted from this test, and both of its
> assertions still pass — a commercial game's microcode still reaches its display list, now driven by a
> device rather than by an arbitrary period. Two things came out of the experiment that the prediction did
> not anticipate. With the stand-in *and* the real interface both raising the interrupt the test **fails**,
> because the stand-in's clear five thousand instructions later also clears the genuine interrupt; that
> failure is what shows the device is carrying the game rather than sitting beside something that was. And
> the serial pulse remained until later the same day, so for a few hours the sentence below was only half
> tested — which is why the paragraph above it exists.

They live in the test, not in the core. The core still cannot raise either interrupt, and a
frontend running Mars would still see nothing happen. The purpose is narrower: to show that
the RSP half of the machine does its part of a real game's frame once the rest of the machine
lets the game ask, and to keep that true while the rest is built. When Phase E builds the two
devices, the pulses should be deleted and the test should still pass; if it does not, the
difference is information about those devices.

## 4. The display list

The seventeen commands, by command number (the byte's low six bits):

| # | number | the community reference's name |
| --- | --- | --- |
| 1 | `0x27` | sync pipe |
| 2 | `0x3C` | set combine mode |
| 3, 4 | `0x2F` | set other modes |
| 5 | `0x00` | no-op |
| 6–8 | `0x2F` | set other modes |
| 9 | `0x2D` | set scissor |
| 10 | `0x3E` | set Z image |
| 11 | `0x3F` | set colour image |
| 12 | `0x37` | set fill colour |
| 13 | `0x36` | fill rectangle |
| 14 | `0x2F` | set other modes |
| 15 | `0x27` | sync pipe |
| 16 | `0x3F` | set colour image |
| 17 | `0x29` | full sync |

**Tier, per `Mars_Documentation.md` §6's policy: community.** The command numbers, names and
lengths are the n64brew wiki's. That page also records that one of the sync commands' opcodes
is stated wrongly in every document, so the test asserts numbers, never names — it checks
that every word is a documented command number, that the list sets a colour image and fills a
rectangle, and that it ends in `0x29`. It walks the list with the documented lengths rather
than eight bytes at a time, because triangles and texture rectangles are longer; this list
happens to contain neither.

**What the check establishes is well-formedness, not correctness.** A microcode defect that
produced different but valid commands would pass it. Nothing consumes the list — that is
Phase D — so nothing yet can say whether these seventeen commands are the ones hardware emits.

> **Update 2026-09-17: the list is now carried out** (`Mars_Rdp.md` §8). Its one fill is a depth
> clear, and the depth buffer holds it afterwards. That grades the display processor's handling
> of these commands, not the microcode's choice of them; the paragraph above stands.

## 5. What the plan did not say

`Mars_Gameplan.md` §4.3 set this condition for the RSP phase, and set the video and serial
interfaces two phases later in §4.5. **A commercial game does not submit microcode until those
interfaces have spoken**, so as written the condition could not be met until Phase E — and
nothing in the plan's §5 or §8, the sections that predict where it will be wrong, mentions it.

It is a relative of the finding recorded in `Mars_Gameplan.md` §4.2's progress notes — an
instrument cannot report on what it never reaches — with a commercial program as the
instrument and the RSP as what it never reached. The transferable form is specific to
conditions that name a program: **a "done when" that names a commercial
program inherits every device that program waits on**, and the only way to learn the list is
to run the program and watch what it waits for — which is what §2 did, and what should have
been done before the condition was written.

The test is the resolution chosen: it meets the condition as written, names the two devices
it had to stand in for, and becomes an ordinary test once they exist.

## 6. What this does not establish

- **Why Mario never submits a graphics task.** Its audio tasks run; its graphics never
  starts. Plausible causes include a controller or EEPROM exchange that zeros do not satisfy,
  and a wait on the display processor's interrupt. None was tested.
- **Why Wave Race submits only one.**

  > **Retired 2026-09-17, by measurement: the reading this bullet gave.** It said *"The list ends
  > in a full sync, and a game would normally wait for the display processor to report it done
  > before the next frame. Nothing raises that interrupt; this is a reading, not a measurement."*
  > The interrupt is now raised by the full sync and acknowledged by the game's handler, and over
  > 200 million instructions the game submits one graphics task and 122 audio tasks — the same
  > counts as on the bus without it (`Mars_Rdp.md` §8). Whatever the game waits for, that
  > interrupt is not all of it. Why only one graphics task is submitted remains open.
- **That the audio tasks are right.** They run to a break and signal completion. Their output
  goes to a buffer nothing plays.
- **Anything about timing.** The RSP still runs one instruction per CPU tick
  (`Mars_Rsp.md` §7).
