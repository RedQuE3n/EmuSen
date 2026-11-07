# Endou_Pacing — the frame-pacing report of 2026-10-20

**The report.** "Smooth out the frame pacing on Master System, and SG 1000. On Wonder Boy The Dragon's Trap, the frame
pacing is very jittery." The game is the library's `Wonder Boy 3 - The Dragon's Trap (UE) [!].sms`. The report does not
say which machine; the handheld's logs (§3.2) show the game played there that night, docked to a 60 Hz television.

**The finding, in one paragraph.** Neither Endou nor the frontend's pacing treats the Master System or the SG-1000
differently from any other console, and nothing was changed. Endou reports 59.922743 Hz for this cartridge, the pacer
locks it to the television exactly as it locks Nephrite's Genesis at the same rate, and Endou's frame costs a third of
a millisecond, evenly. What the screen shows is the game's own cadence: *Wonder Boy III* moves its background four
pixels every second frame, so the picture changes on five frames of every eight, and Genesis Plus GX and Gearsystem
produce the same sequence frame for frame. On the handheld the only irregularity in the frontend's own counters is a
picture lost about every ten seconds while locked at one refresh a frame, which the NES, the SNES and the PlayStation
show at the same rate or worse in the same hour (`EmuSen_Settings_Reference.md` §4.87.14).

## 1. Hypotheses, written before measuring

Each was stated with the observation that would confirm it before any was measured.

- **H1, the region.** A "UE" dump might start as a European console, 49.70 Hz, which on a 60 Hz display is content-paced
  (the refresh 20% away) and shows five pictures in six refreshes. *Confirmed by:* `content 49.70` in the `[fps]` line.
- **H2, the rate's source.** Endou might report a rate other than its own, or the pacer might use 60 Hz for it.
  *Confirmed by:* a content rate other than 59.92 in the log, or a decision different from Nephrite's at 59.92.
- **H3, the audio.** Endou's 48 kHz output might starve or flood the queue, shedding or correcting outside the band.
  *Confirmed by:* shedding, or a ratio outside what Nephrite's run shows, on the same bench.
- **H4, the frame's cost.** Uneven work inside a frame (the VDP's or the PSG's catch-up bunched) might move frame
  starts. *Confirmed by:* frame-start intervals or frame costs with a spread beyond Nephrite's.
- **H5, the display sync.** The 59.92 Hz rate might be decided or deadlined differently from cores that pace smoothly.
  *Confirmed by:* hold distributions that differ from Nephrite's, which shares the rate.
- **H6, lag frames** (added once H1 to H5 were refuted). Endou might run the game slower than the console, so that the
  game itself drops updates. *Confirmed by:* a picture-change sequence that differs from the references' under the
  same inputs.

## 2. Instruments

- The `[fps]` lines Mistress writes once a second to `general.log`: frames run, pictures offered and pictures the render
  thread drew ("shown"), and the pacing decision (`EmuSen_Settings_Reference.md` §4.87.7). A second in which "shown" is
  below "offered" lost a picture.
- `CadenceBench` (WiseMan, §4.87.1): a real core through `FrameScheduler`, the real spin-wait and audio path, against a
  modelled vblank-locked presenter; its trace read with the display-sync `analyze.py`.
- `examples/cadence.rs` (this crate): one line a frame, whether the picture changed from the frame before, the VDP's
  X and Y scroll registers and the frame's cost. Its sequence is held against the probe's per-frame screen CRCs
  (`--sig`) from the pinned libretro references (`Endou_Native.md` §5.2), run as black boxes.

## 3. What was measured (2026-10-20)

### 3.1 The header and the rate (H1, H2)

The cartridge's header byte `$7FFF` is `$40`: region code 4, an export market, so `market_of` takes the Americas, the
model the Master System II, the rate 53,693,175 / 896,040 = 59.922743 Hz [M]. Every `[fps]` line of the tester's
sessions reads `content 59.92`. **H1 and H2 are refuted.**

### 3.2 The tester's sessions on the handheld

The handheld was docked to a 3840 × 2160 television at 60 Hz (gamescope: external display, `GAMESCOPE_FPS_LIMIT` 60,
variable refresh enabled) [M `xprop -root`]. Every session that night took the lock at 60.00 Hz, one refresh a frame.
Seconds locked, and seconds in which a picture was lost:

| Install | Console | Sessions | Locked seconds | With a lost picture | Per second |
|---|---|---|---|---|---|
| before 03:43 | Master System (Endou), Scanlines filter | 2 | 327 | 27 | 0.083 |
| before 03:43 | NES (MoonRT), CRT | 6 | 190 | 23 | 0.12 |
| before 03:43 | SNES (VenusRT), CRT | 2 | 118 | 14 | 0.12 |
| before 03:43 | PlayStation (Diamond), CRT | 2 | 47 | 14 | 0.30 |
| 37af18d7, from 03:43 | Master System (Endou), CRT | 4 | 805 | 5 | 0.006 |

Endou's frames cost 0.25 to 0.37 ms a frame in those lines, against 16.67 ms between frames. The Master System lost
pictures no more often than the other consoles in the same hour. The later sessions lost one in about 160 s. From
Endou's stage 7 (`a0b1c3b9`, the oldest build that can have offered the game) to `37af18d7`, the frontend's pacing and
presentation code is unchanged; the commits touch the CRT filter's tables (`EmuSen_CRT.md` §31) and the controller
pages. The filter was also changed by the tester at 03:55, so the improvement is recorded and not attributed. On 2026-10-19, with the display read at
240 Hz and the lock at four refreshes a frame, the Genesis lost a picture in 5 of 1,448 locked seconds.

### 3.3 The pacer: Endou and Nephrite at the same rate (H3, H4, H5)

`CadenceBench` on the desktop, 30 s from power-on, the first 3 s and the last second dropped; Endou on *Wonder Boy III*,
Nephrite on *Sonic the Hedgehog* (JUE), MoonRT on *Super Mario Bros. 3* for the 60.10 Hz case [M]:

| Modelled refresh, pacing | Core | Holds (vblanks: count) | Irregular /s | Frame-start sd, ms | Handed over to shown, mean (p95) ms |
|---|---|---|---|---|---|
| 60, locked | Endou | 1: 1554, 2: 1 | 0.04 | 0.008 | 3.06 (3.18) |
| 60, locked | Nephrite | 1: 1554, 2: 1 | 0.04 | 0.041 | 3.25 (4.06) |
| 60, locked | MoonRT | 1: 1554, 2: 1 | 0.04 | 0.011 | 3.38 (3.97) |
| 60, content | Endou | 1: 1552, 2: 2 | 0.08 | 0.002 | 8.85 (16.33) |
| 60, content | Nephrite | 1: 1553, 2: 2 | 0.08 | 0.002 | 8.86 (16.48) |
| 164.90, content | Endou | 2: 386, 3: 1170 | 14.86 | 0.002 | 3.54 (6.27) |
| 164.90, content | Nephrite | 2: 386, 3: 1170 | 14.86 | 0.002 | 3.52 (6.22) |

The one irregular hold in each locked run is the lock being taken. Endou's and Nephrite's rows agree to the count; the
164.90 Hz rows are the two-and-three cadence any 60 Hz content has on that display (§4.87.1). Neither run shed audio:
Endou's ratio stayed within 1.0016 to 1.0036 locked, Nephrite's within 1.0016 to 1.0037.

Endou's frame cost on the game's walking scene, 1,100 frames, release build [M]:

| Machine | p50 | p95 | p99 | max |
|---|---|---|---|---|
| Desktop | 0.262 ms | 0.277 | 0.294 | 0.465 |
| Handheld | 0.287 ms | 0.303 | 0.329 | 0.732 |

**H3, H4 and H5 are refuted**: the decision, the deadlines, the audio and the cost are those of a core that paces
smoothly.

### 3.4 The game's own cadence (H6)

With the same inputs (button 1 at frame 300, Down at 600, button 1 at 700, Right held from 1,900), `cadence` on Endou
and Genesis Plus GX v1.7.4 (`system_hw` Master System II, region `ntsc-u`) give the same changed-or-not sequence for
every frame from 301 to 2,390, 2,090 frames that cover the title, the menu, the story's text and the castle; they differ
only in the first 300 frames, where the two start-ups differ. Gearsystem 3.9.22 (region Export) gives the same
eight-frame pattern on the walk. While the player walks, register 8 steps by four pixels every second frame (`B4 B4 B0
B0 AC AC …`), the player's sprites stay at X `$74`, and the picture changes on five frames of each eight: the scroll on
every second frame and the walking animation on one frame between [M]. **H6 is refuted**: there are no lag frames, and
the background's 30 Hz step is the game's.

### 3.5 The SG-1000

Four cartridges from power-on, no input, the share of frames 600 to 1,790 whose picture changed: *Wonder Boy* 0.430 on
Endou, 0.429 on Genesis Plus GX; *Star Force* 0.695 and 0.682. *Zaxxon*'s attract mode reaches other scenes on Genesis
Plus GX, so it was held against Gearsystem instead: in five windows of 600 frames the two agree to within 0.002 [M]. No
SG-1000 cartridge measured runs slower on Endou than on a reference.

## 4. What this leaves

**Nothing in Endou or in its pacing was changed**, because no defect was shown. The decision table's rows for Endou's
two rates were added to `FrameSchedulerTests` (59.922743 Hz on 60, 119.90 and 164.90 Hz; 49.701459 Hz on 50 and 60 Hz)
so that its treatment is pinned beside the other consoles'.

**What the tester can check by eye.** On the same television, *Sonic the Hedgehog* (UE) on Endou changes its picture on
every frame of its running; if it looks smooth where *Wonder Boy III* does not, what is seen is the latter's 30 Hz
background, which the console draws the same way. The lost pictures of §3.2 are read from the log: in the `[fps]` line,
a second in which "shown" is one below "offered".

**Not offered.** A filter that blends or interpolates the 30 Hz steps would change the picture the console makes; it is
not pacing, and no form of it was built.

## 5. Seeded faults (2026-10-20)

Each built, run against the tests that guard the hypotheses above, and reverted:

| Fault | Caught by |
|---|---|
| Region code `$4` read as Europe (H1) | five of Endou's tests, among them `the_region_setting_chooses_the_market_and_the_rate` and `the_stub_reports_the_console_its_market_and_a_black_picture` |
| The NTSC frame counted as 263 lines (H2) | `the_rates_are_the_master_clock_over_a_frame`, `a_seconds_frames_make_a_seconds_samples`, `a_quiet_cartridge_is_black_and_silent_at_the_output_rate` |
| `FrameScheduler`'s tolerance narrowed to 0.5% (H5) | the new 49.701459 Hz on 50 Hz row, the tolerance's edge test and the wobbling-rate test |

## 6. What was read

`EmuSen_Settings_Reference.md` §4.21, §4.28 and §4.87; `FrameScheduler`, `FramePacer`, `DisplayClock`, Mistress's
emulation loop and `CadenceBench`; the platform library's frame info and speed; Endou's `machine`, `media`, `sound`,
`v1` and the VDP crate's timing; the handheld's own logs and configuration, read in place. No reference's source was
read; the references ran as black boxes through the probe.
