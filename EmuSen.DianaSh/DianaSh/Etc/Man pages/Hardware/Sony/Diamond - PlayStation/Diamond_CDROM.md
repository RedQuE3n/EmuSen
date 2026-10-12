# Diamond_CDROM — the CD-ROM controller and the discs it reads

*Written 2026-10-11, at stage 7.* The plan's §7 row 7 (`Diamond_Plan.md`): the controller's commands, responses and
timing, reads, filters, XA-ADPCM, CD-DA, subchannel Q and `.sbi`, the lid; `.cue`/`.bin`, `.iso`, `.chd`; the routing.
The controller is `src/cdrom/` (the decoder's host registers and the command protocol in `mod.rs` and `commands.rs`, the
drive in `drive.rs`, XA-ADPCM in `xa.rs`, the audio output in `audio.rs`, the state in `state.rs`), the disc a create
opens is `src/medium.rs`, and the disc itself is the shared crate `emusen-disc` (`../../Shared/EmuSen_Disc.md`).

The marks are the plan's: **[M]** measured on 2026-10-11 on the desktop (Ryzen 7 7700X, Fedora 44), the tool named;
**[R]** read from a source; **[A]** argued; **[C]** a choice. The disputes this stage opened are D-24 to D-29 in
`Diamond_Disputes.md`.

---

## 1. The open replacement, and what it is written from

The controller is a Motorola HC05 running Sony's program beside the CXD1199/CXD1815 decoder [R psx-spx "CDROM
Drive"]. The plan's §5.6 decided that a model of the command protocol at the register interface, written from
psx-spx's description of the commands, is the open replacement for that program, and that no image, listing or
disassembly of it is asked for or accepted. Nothing of the HC05's program was read: psx-spx's chapters on its RAM, its
I/O map and its internal command set (`cdrominternalinfoonpsxcdromcontroller.md` lines 66–890 and its "Summary of Used
CX(xx) Commands", and `cdromdrive.md`'s "Read HC05 SUB-CPU RAM" and "CDROM Controller I/O Area and RAM Memory Map")
were left unopened, and the test commands that read the HC05's RAM answer zero (§4).

## 2. The machine around it

- **Registers**: the decoder's four banks of four 8-bit registers at 1F801800h–1F801803h on DEV5's timing, a 16-bit read
  of RDDATA taking two bytes, any other wider access repeating the byte register, as psx-spx's "Misc" says a 32-bit
  read of HSTS does with auto-increment off [R]. Each access brings the controller to the present first.
- **IRQ2**: I_STAT bit 2 rises when HINTSTS and HINTMSK first meet [R "HINTMSK"]; the line is kept so that only its
  rising edge sets I_STAT.
- **DMA channel 3** moves four RDDATA bytes a word, at DEV5's 24 or 40 cycles a word (the existing `dma.rs` rate), and
  is requested while the data FIFO holds bytes (DRQSTS), so that a transfer started before BFRD waits for it [A, psx-spx
  "RDDATA": "software must set the BFRD flag, then wait until DRQSTS is set"].
- **The disc** is held by the controller behind an `Arc<Mutex<_>>`, so that the copy of the board a state load works on
  shares it rather than duplicating up to 700 MB; it is not part of the state (§10).
- **Audio** leaves the controller at 44,100 Hz into a ring of 8,192 stereo frames, through the volume matrix; the
  machine takes one frame a sample (768 cycles) at each frame's end, which is where the SPU of stage 8 will take its CD
  input. Until then the frames are dropped, or kept by a test's tap. *Since stage 8 (2026-10-11)* the SPU takes one frame a sample at the sample's cycle, the
  controller brought to it first (`Diamond_SPU.md` §2).

## 3. The host interface

From psx-spx's "CDROM Controller I/O Ports" [R] unless marked:

- **HSTS**: the bank, ADPBUSY while XA frames are in the output, PRMEMPT, PRMWRDY (fewer than 16 parameters), RSLRRDY,
  DRQSTS, BUSYSTS while a written command waits to execute.
- **The parameter FIFO** holds 16; **the result FIFO** 16, padded with zeros past a response and wrapping. RSLRRDY clears
  once the response's last byte is read. **An acknowledge does not empty the result FIFO** [C]: psx-spx says "after
  acknowledge, the result FIFO is drained", but read literally that empties it before the bytes are read, and a
  PSn00bSDK program, which acknowledges first and reads after, then reads only the first byte; the FIFO is replaced by
  the next response instead (D-28 records what remains).
- **HCHPCTL**: BFRD's rising edge loads the data FIFO from the start of the sector the last INT1 delivered: 800h bytes
  from offset 24 (mode 2) or 16 (mode 1), or 924h from offset 12 in whole-sector mode; set again while the FIFO still
  holds bytes it changes nothing; cleared, it empties the FIFO [C]. *Found 2026-10-11*: a game's libcd writes BFRD twice
  after clearing HCHPCTL, and a reload on the second write restarted the DMA transfer begun after the first, so that the
  header arrived twice and the read was retried without end (`cdrom::tests::bfrd_set_again_…`). Past the sector's end
  RDDATA repeats the byte at [800h−8] or [924h−4].
- **HCLRCTL**: the INT bits, BFEMPT and BFWRDY acknowledged by their 1s; CLRPRM empties the parameter FIFO; CHPRST resets
  the decoder's FIFOs and flags and leaves a crashed program (commands 58h–5Fh) running again [C].
- **ATV0–ATV3 and ADPCTL**: the volumes are written apart and applied by CHNGATV; ADPMUTE silences XA alone (§7).
- **Not modelled**: the sound map (SMEN, WRDATA, CI: XA played from RAM), BFEMPT and BFWRDY.

## 4. The commands

Every command of psx-spx's "Command Summary" [R], with its first response, the second it defers and the delay of each.
A command waits until no interrupt is pending, as "First Response" says, then executes after its first response's
delay (D-24): 46,000 cycles with the motor on, 23,796 stopped, 75,000 for Init and ReadTOC. Error responses are INT5 with
the status's error bit and 10h (a bad value), 20h (a wrong count), 40h (an unknown command) or 80h (not ready: no disc,
the lid open, or the TOC not yet read while the drive spins up). The commands psx-spx lists as needing a disc (02h–09h,
0Bh–0Dh, 10h–16h, 1Ah, 1Bh, 1Dh) answer 80h without one.

| Command | Parameters | First | Second, and its delay |
|---|---|---|---|
| 00h Sync | — | INT5 40h | — |
| 01h Nop (Getstat) | — | INT3 status; the shell flag cleared once the lid is closed | — |
| 02h Setloc | mm ss ff, BCD | INT3; the target kept until a seek, read or play uses it; 10h for bad BCD | — |
| 03h Play | optional track | INT3 | — (reports and INT4, §7) |
| 04h Forward, 05h Backward | — | INT3 while playing, 80h otherwise | — (two seconds a press) |
| 06h ReadN, 1Bh ReadS | — | INT3 | INT1 per sector (§5) |
| 07h MotorOn | — | INT3, or 20h if the motor is on | INT2 after the spin-up |
| 08h Stop | — | INT3, the read bit cleared | INT2, the motor off: 13,863,626 single, 25,845,878 double, 7,547 stopped |
| 09h Pause | — | INT3, the read bit still set | INT2: 1,020,000 reading or playing, 7,666 held |
| 0Ah Init | — | INT3 (late); mode 20h | INT2 once the head holds at 00:02:00, at least 476,000 later |
| 0Bh Mute, 0Ch Demute | — | INT3 | — |
| 0Dh Setfilter | file, channel | INT3 | — |
| 0Eh Setmode | mode | INT3; bit 4 keeps the last sector size | — |
| 0Fh Getparam | — | INT3 status, mode, 0, file, channel | — |
| 10h GetlocL | — | INT3 the last data header and subheader; 80h before the first seek or read, while seeking, on audio | — |
| 11h GetlocP | — | INT3 the last valid Q's track, index and times | — |
| 12h SetSession | session | INT3 for 1, the head to the start; 10h for 0; two INT5 40h above 1 | INT2 |
| 13h GetTN | — | INT3 status, first, last | — |
| 14h GetTD | track, BCD | INT3 status, mm, ss of its INDEX 01 (00 the lead-out); 10h past the last | — |
| 15h SeekL, 16h SeekP | — | INT3 | INT2 at the seek's end; SeekL onto audio, or a seek past the disc's reach, INT5 with the seek error and 04h, the motor off (D-26) |
| 19h Test | sub-function | INT3 (below) | — |
| 1Ah GetID | — | INT3 | after 18,944: INT2 licensed for the console's region (§11), INT5 90h for an audio disc, INT5 08h 40h for an empty drive; INT5 80h with the lid open |
| 1Ch Reset | any | INT3; then commands wait an eighth of a second, the drive spins up again | — |
| 1Dh GetQ | adr, point | INT3 | INT2 the lead-in item and a peak of zero after 0.1 s; for a point not found INT5 80h after six seconds |
| 1Eh ReadTOC | — | INT3 (late) | INT2 after one second |
| 1Fh VideoCD, 50h–57h unlock | — | INT5 40h, parameters kept (psx-spx: unsupported) | — |
| 58h–5Fh | — | none: the program stops until CHPRST | — |
| 17h, 18h, 20h–4Fh, 60h–FFh | — | INT5 40h | — |

**Test (19h)**: 20h answers the firmware's date and version (§11); 21h the switches (POS0 when the head is before
00:02:00, the door); 22h the region string ("for Japan", "for U/C", "for Europe"); 23h–25h psx-spx's chip strings,
truncated to the FIFO's sixteen bytes; 04h and 05h the SCEx counters (1, 1 for a data disc); 03h stops the motor and
00h–02h and 10h–1Ah start it; 50h, 51h, 72h, 74h, 76h answer the status; 60h, 71h, 73h, 75h, which read the HC05's RAM
and the decoder's registers, answer zero [C, §1]; any other sub-function 10h.

**Second responses**: psx-spx's "Responses" says the controller keeps one flag for a second response and one for an
INT1 [R], so a second response replaces one not yet delivered [C]; INT4 and the lid's INT5 keep their own places. After
an acknowledge the next response comes 7,424 cycles later at the earliest (D-24).

## 5. The drive

- **The spindle**: a spin-up and the TOC's reading take 0.75 s (D-25). At power-on with a disc the drive spins up by
  itself and the shell flag is set, as psx-spx's tables show it on their first responses (0x11 rather than 0x01) [R].
  While spinning up, the motor bit reads clear and GetID answers INT5 01h 80h, as psx-spx's GetID table has it.
- **Seeks** (D-25): 400,000 cycles plus 300 ms × √(distance ÷ a 74-minute disc), or a sector period for each sector when
  fewer than 32 ahead. A seek that cannot reach its target fails at its end (D-26). A seek from a stopped motor spins up
  first.
- **The held head** steps through psx-spx's window after a seek or a pause: N−8…N single speed, N−5…N+2 double, a sector
  a period, its Q kept for GetlocP and, once a data header has been read since power-on, its header for GetlocL. After
  a spin-up or Init it holds at 00:02:00, the window 00:01:67–00:02:00, as the getloc log finds it.
- **Reads** deliver a sector a period, 451,584 cycles at single speed (psx-spx's "INT1 Rate": 33,868,800 × 930h ÷ 4 ÷
  44,100) and 225,792 at double, the speed taken from the mode at each sector; the first sector a period after the
  seek's end. **A Read with no new Setloc** begins where the last seek left the head or at the sector received last,
  "returning that sector once another time" [R "Setloc, Read, Pause"]. *Found 2026-10-11*: beginning at the last
  delivered sector instead made a Setloc–SeekL–ReadN sequence, the kernel's, read the wrong place.
- **The buffer**: a sector arriving while no interrupt is pending is delivered at once (INT1); otherwise it waits, the
  newest replacing the one waiting, and comes a little after the acknowledge [R "Sector Buffer", which shows the oldest
  then the newest]. A sector waiting when a Pause, Stop, Init or seek executes is dropped (psx-spx's tests show no INT1
  after Pause's first response). With the filter on, a waiting sector of another file or channel is dropped when its turn
  comes, psx-spx's "2nd delivery attempt".
- **The filter and XA**: with Setmode's XA bit, a mode 2 sector whose submode has audio and real time, and, with the
  filter bit, the chosen file and channel, goes to the XA decoder and makes no INT1; with the filter bit, other
  audio-real-time sectors are dropped; everything else is delivered as data [R "Data/ADPCM Sector Filtering"]. With
  Setmode's CDDA bit an audio sector is delivered as data; without it, none is.
- **The lid**: opening it delivers INT5 01h 08h whatever is pending (the disc-swap log's status, where psx-spx's text
  implies 11h) and stops the drive, the motor bit reading set until the spindle stops half a second later; GetID then
  answers 11h 80h. Closing it, on the same disc or another, spins the drive up; the shell flag stays set until a Nop
  reads it. *The disc change of the core ABI* (`EMUSEN_MEDIA`, `EmuSen_CoreAPI_Extensions.md` §3) waits for stage 11; the
  machine's `set_lid` is what it will call.

## 6. XA-ADPCM

`xa.rs`, from psx-spx's "CDROM XA Audio ADPCM Compression" [R]: a form 2 sector's eighteen 128-byte portions, 4-bit
(eight blocks of 28 samples, two a byte) or 8-bit (four blocks), mono or stereo (left the even blocks), each block's
header a shift (13–15 acting as 9) and a filter of four, each sample its value shifted to 16 bits and right by the
shift, the filter's prediction from the two before added and saturated; the two before kept per channel across sectors.
The 37,800 Hz output goes through psx-spx's 25-point zigzag interpolation, seven 44,100 Hz frames for every six, its
seven 29-tap tables transcribed from the page by a script rather than by hand. 18,900 Hz feeds each sample twice. The
rounding and the 18,900 Hz rule are D-27's choices; emphasis is not applied.

**Tested against the source** [M, `cdrom::xa::tests` and `cdrom::tests`]: blocks of filter 0 and shift 0 decode to their
nibbles × 1000h exactly, in every layout; a filter's prediction against the formula done by hand; a held level comes
out at each table's gain (psx-spx's tables sum to 73E5h–741Dh, so the gain is 0.905–0.907, not unity, D-27). Through
the controller: a stream on a synthetic disc interleaving channel 0's XA, channel 1's XA and data sectors, read with
the filter on, delivers only the data sectors as INT1 and plays channel 0, whose output equals its source put through
the interpolation formula written apart in the test, frame for frame over 4,000 frames.

## 7. CD-DA, reports and the volume matrix

Play seeks to the track, the Setloc or the current position and plays a sector a period, its 588 frames to the output;
a data sector plays as silence ("all sectors from data tracks are treated as 00") [R "Play"]. With Report, an INT1 of
eight bytes comes on each tenth frame of the second, the absolute time on frames 00, 20, 40, 60 and the time in the
track with bit 7 of the seconds on 10, 30, 50, 70, the peak the sector's largest left sample [R "Report"; the peak's L/R
bit is not modelled]. AutoPause pauses with INT4 when Q's track changes; the lead-out stops the drive with INT4 [R
"AutoPause"]. Forward and Backward jump two seconds a press [C].

**The matrix** (psx-spx "ATV0".."ADPCTL"): left = (L·ATV0 + R·ATV3) ≫ 7 and right = (R·ATV2 + L·ATV1) ≫ 7, saturated;
80h is unity, 40h each way mono. Mute silences both sources, ADPMUTE XA alone. **Tested** [M]: a track played on a
synthetic disc reaches the output frame for frame equal to the disc's samples at unity volume, with its reports and its
INT4; the matrix's unity, mono and saturation by their own test.

## 8. Discs at create, the system pack, and what the host owes

### 8.1 What the core opens

`medium.rs`: the image given to `create` is a PS-EXE (left to `media.rs`), a **cue sheet** (text whose first command is
FILE, REM, CATALOG, TITLE or PERFORMER) with its files as companions, **a CHD**, **an `.iso`**, or **a lone raw track**;
the last three are one file each and open through `emusen-disc`'s `from_image`, an `.iso` with no volume descriptor
through `from_user_image` when its sector 4 holds the licence. Companions are the create's files numbered from 65,536,
the sheet's nth file at 65,536 + n, then an `.sbi` found beside the image, told by its magic (`EMUSEN_FILE_MEDIA`,
`EmuSen_CoreAPI_Extensions.md` §2.4, decided 2026-10-11) [C]. **The number is defined in Diamond's `v1.rs` until it is
added to `emusen_core.h` and `emusen-native`**, which is the framework's step (§8.4). Every disc is CD-XA to the
crate (`IsoForm::Mode2Form1`). The market is the licence's second line in sector 4, as `media.rs` reads a raw image's;
a disc without the licence text is refused. Refusals: −9 not a PlayStation image, −10 a short PS-EXE, −11 a sheet's
tracks missing ("the cue sheet names 2 track files and 1 were given"), −12 a sheet or image the crate cannot read, each
with words.

### 8.2 The tester's library, and who owns each form

`~/Documents/Roms/psx/` holds 19 `.bin`, 2 `.cue` (each with its `.bin`), 17 `.zip` and one folder of a sheet with its
51 track files [M, a listing; nothing in it written]. Of the `.bin`s, 17 have no sheet: whole single-track dumps, or
"(Track 1)" files whose audio tracks are not there. The zips hold either a sheet with its tracks or "(Track n)" files
with no sheet. The forms, weighed by the project's rule (an extension of a surface, a module, or a component of its
own; one job per component):

| Form | What it needs | Owner, and why | Built here |
|---|---|---|---|
| A lone single-track `.bin` | a one-track sheet synthesised from its first sector | **`emusen-disc`** (an extension of its opener): the sync pattern and the header's mode are a container format's facts, and the kit and the probe need the same reading without C# (`EmuSen_Disc.md` §4) | yes |
| `.bin` routed to the PlayStation | telling it from a Genesis cartridge, which the Genesis pack claims by the same extension | **routing by content**, a runtime service over the packs' recognisers (Extensions §8, decided): the PlayStation's recogniser is the pack's | the recogniser, `PsxSystem.IsDisc`; the service is the framework's |
| `.iso` routed | the Sega CD's pack claims `.iso` | the same service | as above |
| `.cue` with its tracks | the host reading the sheet's FILE lines and passing the files | **the set module of `emusen-disc`**, reached by the runtime through the platform library (Extensions §2.3, decided), the runtime reading the companions | the set module and the core's side; the runtime's side is the framework's |
| `.chd` | nothing: one file, and no other pack claims it | the system pack's extension list | **routed**: `.chd` is in the pack's and the core's lists |
| `.zip` holding a disc | reading an archive's entries and choosing the image inside | **the frontend's image loader over Galaxia's file layer** — a component of its own, not the core (a core receives bytes, never paths) and not the disc crate (an archive is not a disc, and cartridge consoles' zips need the same reading); the system pack names what inside is an image (its extensions, a sheet's companions) | no: proposed (§8.4) |
| `.m3u` | the runtime expanding the list, the disc change | the set module (its `playlist` reader is built) and stage 11's `MEDIA` | the reader |
| `.sbi` | found beside the image | the runtime by name (`set::sbi_beside`), the core by its magic | the core's side |
| `.pbp` | later, unencrypted only (the plan's §1.2) | — | no |

### 8.3 The system pack

`PsxSystem` (`EmuSen.Crystal/Systems/Psx/`): the entry's extensions are `.psexe`, `.psx` and **`.chd`**, the core's list
the same three (held equal by `DiamondTests`); `DiscExtensions` keeps `.cue`, `.iso`, `.bin`, `.m3u` and `.pbp` declared
and not routed, for the reasons of §8.2; `IsDisc` is the recogniser routing by content will ask, the licence text's
first line in sector 4 of an `.iso` or a raw track, held to the core's decision on twelve synthetic images by
`DiamondTests` [M]. One fact, two owners, as the extensions already are: the core cannot read the C# pack.

### 8.4 What the framework owes, proposed

*All four built 2026-10-11, at stage 11, with the tester's approval of that day (`Diamond_Players.md` §1 and §2): the
constant in the header as minor 1.1, the runtime's companions and `.m3u` with the disc change, routing by content, and
archives. The pack now routes `.cue`, `.bin`, `.iso` and `.m3u`; `DiscExtensions` keeps `.pbp` alone.*

Built nothing beyond Diamond, the disc crate and the pack, as the stage's terms required; each item below is decided
in `EmuSen_CoreAPI_Extensions.md` but for the archive, which is new:

1. **`EMUSEN_FILE_MEDIA` (65,536)** in `emusen_core.h` and `emusen-native`'s constants, replacing Diamond's local
   `MEDIA_FILE` (Extensions §2.4). A header constant; no behaviour changes.
2. **The runtime's companions**: before create, for an image whose extension is a sheet's, the runtime calls the set
   module (`emusen_disc::set::companions`) through the platform library and passes each file as `EMUSEN_FILE_MEDIA + n`,
   then the `.sbi` by `set::sbi_beside`; the kit's `--images` form the same (Extensions §2.6). With it the pack moves
   `.cue` into its routed list.
3. **Routing by content** (Extensions §8.4): `SystemEntry` gains `Recognise`, the PlayStation's being `IsDisc`, and
   `SystemOf(path)` decides for `.bin` and `.iso` among the Genesis, the Sega CD and the PlayStation.
4. **Archives, a component of its own** [C, new]: the frontends' image loader opens a `.zip` through Galaxia (which owns
   paths, and would gain an archive's entry listing and reading as the way a path inside an archive is named), picks
   the entry the system pack's extensions name (a sheet, then a lone image), reads it and its companions from the same
   archive, and hands bytes to create as for a loose file. It is a component because it serves every console's zips
   (cartridges too), owns one job (naming and reading entries), and touches no core; ES-DE's convention of zipped
   discs makes it common. Inflating a 600 MB disc costs time at load (§9.7's copy measures the rest).

## 9. The gate

### 9.1 ps1-tests' `cdrom/getloc` and `cdrom/disc-swap`, and `cdrom/timing`

Run on the player's BIOS (as the release's programs are PSn00bSDK-built, they need a kernel) with one of the tester's
discs (Ape Escape, `.cue`), the TTY read from the program's heading on so that the BIOS's own messages are left out
[M, `tests/programs.rs`, opt-in by `EMUSEN_DIAMOND_PS1TESTS`, `EMUSEN_DIAMOND_FIRMWARE` and `EMUSEN_DIAMOND_DISC`]:

- **getloc**: "Test passed"; every GetStat and every GetlocL and GetlocP succeeds or fails where the console's log has it;
  the values differ where the disc differs (its lead-out 33:07 away rather than 3:34) and in the eighth byte of GetlocP,
  which the program reads as zero (D-28).
- **disc-swap**: every line of the console's log, the lid opened a second after the program asks and closed a second
  after it asks again, "Got IRQ 5 (expected 5), status 0x01" and the Getstats 12h, 10h, 00h as the console's; the log's
  last line, "Disc inserted?", is the source's older wording for the release's "Disc detected".
- **timing**: every figure within the console's range (D-24).

The release's `cdrom/terminal` is interactive and `volume-regs` is not in it; neither was run.

### 9.2 Synthetic discs read sector for sector

`cdrom::tests` [M]: a disc of a 600-sector mode 2 data track and a 300-sector audio track, built in the test, read with
ReadN at both speeds as user data and as whole sectors, each sector's FIFO bytes taken as DMA takes them and equal to the
disc's; sectors a period apart (451,584 and 225,792 cycles, within 300); a late handler receiving the oldest then the
newest; the TOC commands; a state taken mid-read continuing the same. `tests/programs.rs`'s `cd-read.s` polls the
kernel's read sequence of psx-spx's ReadN chapter on a synthetic disc with no kernel, and DMA channel 3 moves its
volume descriptor whole.

### 9.3 XA and CD-DA against their sources

§6 and §7.

### 9.4 Games on the player's BIOS

Seven of the tester's discs, copied or unzipped into the cache, each form represented: Crash Bandicoot and Spyro the
Dragon (lone `.bin`), Ape Escape (`.cue`), Rayman (a folder of a sheet and 51 tracks), Silent Hill and Castlevania:
Symphony of the Night (zipped sheets), Tekken 3 (a zipped "(Track 1)" with no sheet, opened as a lone track).

~~**Booted the console's way, none reaches its title**: every BIOS revision in the firmware folder draws the "Sony
Computer Entertainment" logo by frame 330, as SwanStation does, and stays on it, where SwanStation shows the second
screen at frame 630 (D-29, open).~~ *Settled 2026-10-12* (D-29, DICR's edge; D-43, the DMA's arbitration): on
`scph5501.bin` every disc passes the intro, its second screen at frame 660 where SwanStation's is at 612, and each game
then goes as far as on the open kernel. Of the folder's other revisions, all but the European `scph5502.bin` (D-44,
which draws nothing) pass the intro on Crash Bandicoot's disc.

**The seven discs, before and after, on both kernels** [M, the step's tool, frame boundaries only, pictures every 150
frames to frame 6,000; "before" WiseMan at `f5c30e69`, "after" with D-29's and D-43's changes; SwanStation and Beetle
PSX on `scph5501.bin` through `psxreferee.py pictures`, the referees' own pictures kept in the cache, never committed]:

| Game | Open kernel, before | Open kernel, after | Player's BIOS, before | Player's BIOS, after | SwanStation and Beetle PSX |
|---|---|---|---|---|---|
| Crash Bandicoot | title by 2,250, the attract demo by 4,350 | title by 2,100, the demo by 4,350, the pictures differing from frame 811 | the first logo, held | the intro, title by 3,000, the demo by 5,100 | the intro, title, the demo |
| Spyro the Dragon | title by 3,300, held to 6,000; 804 CdRead retries in 3,600 frames | title by 2,400, "Entering demo mode" and the demo by 6,500; no retry | the first logo, held | the intro, title by 3,150, the demo by 7,500 (to frame 8,000) | the intro, title, the demo by 6,000 |
| Tekken 3 | "NAMCO PRESENTS", title, attract fights; the opening movie skipped | the same | the first logo, held | the intro, then as the open kernel | the opening movie, then title and attract |
| Rayman | the Ubi Soft logo and its opening movie | the same, every picture identical | the first logo, held | the intro, the logo and the movie | the same |
| Ape Escape | its DUALSHOCK screen, then white from 1,500 | the same pictures | the first logo, held | the intro, then as the open kernel | Beetle PSX: the DUALSHOCK screen, then its polygon intro; SwanStation: its "not plugged into port 1" screen (§4.14 there) |
| Silent Hill | the content warning, Konami and KCET logos, the card warning, then black | the same | the first logo, held | the intro, then as the open kernel | the opening movie from 1,950 |
| Castlevania: SotN | black throughout | the same, every picture identical | the first logo, held | the intro, then black | the movie, the Konami logo, its title by 1,650 |

The three that stop do so the same way before and after and on both kernels, and **none shows a DICR storm**: in their
last 300 frames DICR stands with no flag and bit 31 up but in passing, and I_STAT's bit 3 is not held. Silent Hill and
Castlevania are streaming their movies at their stop, the drive thousands of sectors past their Setloc, DMA3 moving
blocks of 1F8h words (an STR sector's 2,016 bytes of data) and the MDEC idle, which is `Diamond_MDEC.md` §9's
malformed MDEC words, not this page's; Ape Escape stops in its polygon intro, with I_STAT's SPU bit standing unmasked
and the CD idle.

**Booted by the kernel's own loader**, a program side-loaded at the shell's entry that calls `_96_init` and `LoadExec`
on SYSTEM.CNF's BOOT line with its STACK (the run tool's `--boot`), skipping the intro and its licence check; the game
then starts about frame 335 [M, the run tool, pictures every 300 frames to frame 3,600 and every 30 for two of them]:

| Game | Form | Diamond | Why it stops, where it does |
|---|---|---|---|
| Crash Bandicoot | lone `.bin` | **its title screen**, from frame 2,100 | — |
| Spyro the Dragon | lone `.bin` | **its title screen**, by frame 3,300 | — |
| Ape Escape | `.cue` | its "DUALSHOCK analog controller is required" screen; then white with the Analog button pressed | its opening movie: the MDEC is a stub (stage 9) |
| Rayman | 51-track sheet | the Ubi Soft logo; CD-DA played with AutoPause; then a data stream read with ReadS | its opening movie (stage 9) |
| Silent Hill | zipped sheet | its content warning and the Konami logo | its opening movie (stage 9) |
| Castlevania: SotN | zipped sheet | the Konami logo and its "no memory card" warning | its opening movie (stage 9) |
| Tekken 3 | zipped track 1 | "NAMCO PRESENTS" | its opening movie (stage 9) |

**Against the referees** [M, `probe` with the player's BIOS by `--sysdir`; PCSX-ReARMed on its own kernel; pictures
compared on 160×120 grey]: the title screens of Crash and Spyro are the references' (a side-by-side picture kept in the
cache, never committed); the mean absolute difference between Diamond's title and the nearest frame of each is 5.3,
4.6 and 4.6 grey levels for Crash on Beetle PSX, SwanStation and PCSX-ReARMed, and 3.4, 1.4 and 1.6 for Spyro, the
residue being the animated characters' phase. SwanStation and Beetle boot both through the intro, PCSX-ReARMed without
it; DuckStation and PCSX-Redux were not run on games (their drivers take a program, not a disc and a picture).

### 9.5 Diagnostics the gate's step used, and their limits

To find D-29 without reading the BIOS, only the screen was observed while Diamond's own devices were varied, and single
sectors of a copy of a disc were changed (D-29's entry lists every variation). Once the BIOS hands over to a game, the
game's own CD traffic and the game's own code in RAM were looked at: the controller's trace from the game's entry
(`--trace-game`), and Ape Escape's wait loop disassembled from its RAM, which showed libcd's synchronisation with a
timeout and led to the BFRD defect of §3. No game data, picture or trace was kept in the repository.

### 9.6 What was not run

DuckStation and PCSX-Redux on the `cdrom/` programs: PCSX-Redux run with `-iso` and `-exe` together stopped at the
program's "Initializing CD-ROM controller" on both discs tried, and was not pursued. No console.

### 9.7 Load time, and the copy at create

[M, the run tool's `--disc` with one frame, `/usr/bin/time`, the files in the page cache]:

| Disc | Size | Opened in | The tool's peak memory |
|---|---|---|---|
| Ape Escape (`.cue`, one track) | 432 MB | 353 ms | 0.85 GB |
| Castlevania: SotN (`.cue`, two tracks) | 583 MB | 498 ms | 1.14 GB |
| Rayman (`.cue`, 51 tracks) | 617 MB | 566 ms | 1.21 GB |
| Spyro the Dragon (lone `.bin`) | 662 MB | 517 ms | 1.94 GB |

The time is the tool's read of the files from the page cache and the core's one copy of them. Through the ABI the host's
copy and the core's are both alive during create, about twice the disc, and the core's alone after; a lone image the
tool copies once more, hence Spyro's three. **On the desktop the copy at create is acceptable**: half a second, and
memory well within 8 GiB [A]. **What the handheld's measurement needs**, which the plan's §5.11 asks for and this step
did not take (nothing was installed or copied there): the run tool's `--disc` on a 660 MB disc from the handheld's own
library storage, once with a cold cache (after a reboot, since dropping caches needs root) and once warm, timing the
"disc opened" line, and the process's peak memory beside the free memory; a cold read from a microSD card at its
typical 90 MB/s would make the open about seven seconds, which would argue for the streaming read of Extensions §10.2.

### 9.8 The games that stopped at their movies, and the stream (2026-10-12)

The tester played the build on the handheld and reported "Slow and many games don't work or launch"; making the games
play was put ahead of disputes no game shows. Three of the seven discs stopped at their movies on the open kernel
(§9.4): Silent Hill went black after its memory card warning, Castlevania: SotN stayed black, Tekken 3 skipped its
opening movie. `Diamond_MDEC.md` §9 suspected the stream reaching the games' decoders.

**Where the stream was looked at** [M, the controller's trace and the game's own register accesses from its entry
(`--trace-game`, `--game-log`), and the game's RAM, on the open kernel; no BIOS runs]: Silent Hill starts its movie with
Setmode E0h (double speed, XA, whole sectors) and ReadS, reads 12 bytes of each sector by the CPU, then 8 words and
1F8h words by DMA channel 3. To frame 1,700 the drive delivered all 539 data sectors at once, none left waiting or
replaced, and sent the 76 XA sectors of file 1 to the decoder; of the last piece DMA channel 3 wrote at each of 474
places in RAM, 451 equal the disc's bytes at the FIFO's position, the 23 others being the headers the game rewrites
in place and buffers of earlier loads it has since reused. **The stream was right.** The game's own sector callback
then dropped every sector after its first frame's, waiting for the interrupt of the DMA piece that ends a frame, which
it enables by a byte store to DICR's third byte; Diamond wrote that byte as the whole of DICR, clearing every channel's
enable. That is D-45 (`Diamond_Disputes.md`), a rule of the bus's I/O widths, not of this controller.

**The seven discs, before and after**, on the open kernel to frame 6,000, pictures every 150 frames, against
SwanStation's on `scph5501.bin` (its BIOS's intro takes some 800 to 1,100 frames more, so its frames run later) [M,
WiseMan at `b5051abb` before; pictures kept in the cache, never committed]:

| Game | Before | After | SwanStation |
|---|---|---|---|
| Ape Escape | the DUALSHOCK screen, then white from 1,650 | **its polygon intro from 1,650, its title by 5,250** | its "not plugged into port 1" screen (§9.4), then the intro from about 3,450 |
| Castlevania: SotN | black throughout | **the movie from 750, the Konami logo at 1,080, its title at 1,320** | the movie at 1,780, the logo at 2,120, the title at 2,360 |
| Crash Bandicoot | its title, then the attract demo | every picture identical | the intro, title, the demo |
| Rayman | the Ubi Soft logo and its opening movie | every picture identical | the same |
| Silent Hill | the warnings and logos, then black from 1,500 | **"the fear of blood" screen from 1,110, the movie from 1,460**, playing at frame 6,000 | the screen at 2,180, the movie from 2,270 |
| Spyro the Dragon | its title, then "Entering demo mode" | every picture identical | the intro, title, the demo |
| Tekken 3 | "NAMCO PRESENTS", title at 1,920, attract fights; the movie skipped | **"NAMCO PRESENTS", the movie from 600, its title at about 6,600 after it, then the attract fights** (to frame 12,000) | the movie from 1,610, the title at about 7,500 |

Against SwanStation's pictures every 10 frames, on 80×60 grey at the best constant offset [M]: Silent Hill's movie
within 1.9 grey levels (the median of 211 pictures from 1,500 to 3,600, at 810 frames), Tekken 3's within 2.2 (256
pictures from 450 to 3,000, at 1,000 frames), Castlevania's movie, logo and title at 1,030–1,040 frames. Silent Hill's
"fear of blood" screen holds 350 frames on Diamond and 90 on SwanStation before the movie; why is not known.
Tekken 3's title comes later than before only because the movie before it now plays, as on the referee. Crash
Bandicoot's, Spyro's and Rayman's pictures are byte for byte the same, since none of the three makes the store.

## 10. The state

`DMND` version 6 (version 5 was the GPU's cache step): the controller's group, `Bus.Cdrom`, after the GPU's: the
disc's identity (an FNV-1a over the TOC; recorded, not checked), the registers and FIFOs, the data FIFO and both sector
buffers whole, the command and its time, the four deferred responses, the drive's activity, positions and times, the
header and Q, the XA decoder's history and interpolator ring, and the audio ring of 8,192 frames with its volumes. Not
in it: the disc, the scratch buffers, the trace flag. **3,788,042 bytes** with a DualShock on port 1 and no card, 40,236
more than version 5 [M, `the_version_6_layout_is_pinned`], the audio ring 32,768 of them. A state saved mid-read and
loaded into a fresh controller over the same disc delivers the same sectors at the same cycles
(`cdrom::tests::a_state_mid_read_…`).

## 11. SCEx, the region and the firmware's version

- **SCEx** (the plan's §5.6): a disc image does not hold the lead-in's wobble, so GetID reports a data disc licensed for
  the console's region, "SCEI", "SCEA" or "SCEE" by the machine's market, which is the disc's own unless the player's
  region setting differs; the disc type is the TOC's (20h for CD-XA). An audio disc is refused as psx-spx's table
  shows (INT5 90h, the ID error set).
- **The firmware's version** (19h, 20h), from psx-spx's table for the PU-18 boards that SCPH-5500 to 5502 carry: 10 Jan
  1997 vC2 (97h 01h 10h C2h) for the Americas and Europe, 12 Sep 1996 vC2 for Japan [R]; the region string "for U/C",
  "for Europe" or "for Japan".

## 12. The cost

[M, `run`, taskset to one core, five runs each]: ps1-tests' `timers` on the open kernel with no disc, 600 frames, took
3.73–3.88 s at `af622ba9` and 4.06–4.12 s after the stage, with identical emulation (the same cycle count); a build of
the stage without the frame's audio drain took 4.34 s, slower again, so the difference is the code's layout under fat LTO
rather than the controller's work, which with no disc schedules nothing. With a disc the drive holds an event every
sector period and reads a sector each, 75 to 150 a second. Games through the loader: Crash Bandicoot 4.69 ms a frame
and Spyro the Dragon 5.25 over their first 3,600 frames, against the plan's budget of 3.8 (stage 13's). No allocation
per frame: the sectors, the data FIFO and XA's frames use buffers made at create.

## 13. What waits

- D-29, booting through the intro; the SPU (stage 8) and the MDEC (stage 9), which the games' openings need. *Since
  2026-10-12* the seven discs play their openings on the open kernel (§9.8).
- The sound map; BFEMPT and BFWRDY; Report's peak's L/R bit; the emphasis filter; GetQ's lead-in seek scratch noise.
- `.m3u` and the disc change (stage 11); the framework's items of §8.4; LibCrypt on a protected disc, of which the
  library has none (the `.sbi` path is tested on a synthetic one).
- The handheld's measurement of §9.7; distributions for D-24's and D-25's figures.

## 14. Tests run (2026-10-11)

`cargo test --release`, after merging WiseMan at `e9ebe218`: `diamond` 101 unit tests (11 of them the controller's, 3
XA's, 1 the matrix's) and 41 program tests, `cd-read.s` among them and four that run only with the disc inputs (getloc,
disc-swap, timing and `cd-kernel.s`), all 41 passing with them; `emusen-disc` 19 with `EMUSEN_DISC_CHDMAN`. WiseMan
under the test lock with `EMUSEN_DIAMOND_PS1TESTS`: `DiamondTests`, `SystemPack`, `CoreAbi`, `CoreAdapter`,
`CoreDiscovery` and `CoreRegistration`, 94 of 94, the sheet with its companions and the pack's recogniser among them.

## 15. Reading log, and exposures

**Read**: psx-spx at `6d7d1bc`: `cdromdrive.md` but the HC05's RAM and I/O chapters (§1); `cdromformat.md`'s disk
format, subchannels, sector encoding, XA subheader, XA-ADPCM, System Area, SCEx and LibCrypt chapters;
`cdromfileformats.md`'s .CUE/.BIN, cue sheet, SBI and .chd sections; `soundprocessingunitspu.md`'s STATX, RAM_CTRL and
transfer sections; `memorycontrol.md`'s DEV5 and auto-increment; `kernelbios.md`'s overview, "BIOS CDROM Functions",
`_boot`, "BIOS Versions" and "Bootmenu/Intro Versions" (not its "BIOS Patches" or the excluded lines); `dmachannels.md`'s
CDROM rows; ECMA-130 §14, §22 and Annex A; ps1-tests' `cdrom/getloc`, `disc-swap` and `timing` sources (`main.cpp`) and
logs, and `common/measure.hpp`'s first lines (its header, which names no copied source); chdman's output.

**Not read**: any emulator's source; the BIOS; the HC05's program or psx-spx's chapters on it; PSn00bSDK's library;
`common/dma.hpp` (which getloc and disc-swap include, so that their binaries were run as black boxes).

**Exposures, stated plainly**: (1) the first runs of the BIOS printed its kernel's start-up banner to the terminal
before the run tool learned to print from a program's heading; (2) one trace of the controller, started at a fixed frame
rather than at the game's entry, showed the commands and responses of the kernel's `LoadExec` reading Ape Escape's
program (a Setmode 80h, a ReadN and its INT1s); the trace was moved to begin at the game's entry, and nothing of it was
used; (3) Ape Escape's own code around its wait loop was disassembled from RAM (game code, not firmware), and the
referees' RAM dumps of the BIOS's boot were written by the probe and deleted unread. No BIOS code, RAM or execution
trace was otherwise observed.
