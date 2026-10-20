# Jadeite_Native — the Game Gear's build record

Jadeite is the Game Gear's module of Endou, the Master System core, and the name of these pages. The Game Gear is a
system of Endou's library (`Endou_Plan.md` §2.2, form (a), decided 2026-10-19): one crate `endou` reports `sms`,
`sg1000` and `gg`, the Game Gear's I/O, window and scaling are its module `gg`, and its VDP is a variant of Beryl's
`beryl-sms-vdp`. The plan is `Endou_Plan.md` §2 and §7 (stages J1 to J3, and the LCD model, JL); the disputes are in
`Endou_Disputes.md`, from D-21; the shared crate's record is `Beryl-HW/Beryl_SmsVdp.md`. The tester's word of 2026-10-20,
"focus on gamegear", was the go for J1; the Master System's gate for it (G4) was met at stage 7.

---

## 1. Stage J1 in one paragraph (2026-10-20)

The Game Gear's VDP and its picture. Endou now lists a third system, `gg`, extension `.gg`, marked in development so
that no player is offered it (`EmuSen_CoreAPI.md` §27.4), and models a fifth console, the Game Gear: the 315-5378, NTSC
whatever the region setting, one controller port with a Start button. A Game Gear cartridge runs in GG mode unless its
header names the Master System (D-23); GG mode shows the LCD's fixed window, 160 by 144 of the VDP's picture, in 12-bit
colours written as words through CRAM's latch, and keeps the window's place in the frame when the display is 224 lines
(D-21). A Master System cartridge's picture is shrunk onto the panel's 160 by 146 as the documents and the photographs
they report describe: three columns into two by their colour elements, a fixed window of 218 lines of the frame into
146 rows, three lines into two with weights in sixths (D-22). The TMS9918's modes show CRAM's entries 16 to 31, which
the crate held since stage 4. The tester's six Game Gear games equal Genesis Plus GX's, Gearsystem's and PicoDrive's
pictures at frames 300 and 600; the Master System and the SG-1000 are unchanged, their kit reports identical but for the
routed-system case's list. Two pieces of stage J2 had to come forward, each shown by a game first: ports `$00`-`$06` no
longer reach `$3E` and `$3F` in GG mode, and Start makes no NMI there.

---

## 2. The system and the console

### 2.1 What was built

| Part | Built | Where |
|---|---|---|
| The system `gg` | core info's third system, "Sega Game Gear", `.gg`, NTSC only, one controller (`gg.pad`: the pad's six and Start, port 0), no firmware, `"development": true` | `v1::game_gear` |
| Routing | `create_as("gg")` makes a Game Gear; an image made unnamed is never one, since the stage-1 rule cannot tell a Game Gear cartridge from a Master System one (`media::Console::Gg`) | `media`, `v1::model_of` |
| The console | `Model::GameGear`: `Variant::GG` (the 315-5378), Sega's PSG, 8 KiB of RAM, `$FF` where nothing drives the bus [R SMS Power!'s "I/O Port Map - Game Gear"], no Reset button, no BIOS until J2, and no PAL (`Model::has_pal`) [R MacDonald's `msvdp` §14: "262 scanlines per frame", the machines "the same throughout the world"] | `model` |
| The mode | GG mode set at power-on by the cartridge, as D-23 reads it, and kept in the VDP's state block, which carried it since stage 3 | `Machine::with_boot`, `Media::gg_mode` |
| Machine info | 160 by 144, at most 160 by 146, aspect 4:3; one port; CRAM 64 bytes in GG mode, 32 in the Master System's | `v1`, `Machine::cram_len` |
| The model setting | **not** offered the Game Gear: a Master System cartridge on the Game Gear is reachable through `Machine::with_board` and `create_as("gg")` of an image whose header carries `$3` or `$4`, so that nothing of it reaches a player before J3 | `v1::MODELS` unchanged |

The state needs no new version: the console's and the model's bytes take new values (2 and 4), and the mode was in the
VDP's block already. No Endou state layout changed.

### 2.2 What J1 took from J2, and why

- **Ports `$00`-`$06` in GG mode.** Sonic the Hedgehog 2 showed its Sega logo and then a black screen [M
  2026-10-20]. Its Z80 was found at `$0038` for the whole frame with interrupts off: the cartridge read as `$FF`, an
  `RST $38` at every step. The game had written port `$06`, the Game Gear's stereo register [R SMS Power!'s "Game Gear
  Stereo"], which the Master System's decoding takes as port `$3E`, the memory control, at every even port of
  `$00`-`$3F`; the write switched the cartridge off. The Game Gear decodes `$00`-`$06` as its own in GG mode [R SMS
  Power!'s "I/O Port Map - Game Gear"]. J1 keeps those writes from `$3E` and `$3F` and gives them no meaning; their
  meanings (Start, the region and NTSC bits, the link port, stereo) are J2's. Reads of `$00`-`$06` stay the undriven
  `$FF`, which a game reads as Start released, an export console and **PAL** (port `$00`'s bit 5 [R Sega's Game Gear
  hardware notes, "I/O port 00H"]): none of the six games behaved differently for it at the frames compared, but it is
  wrong, and J2's.
- **Start.** In GG mode the Start button is a bit software reads at port `$00`, not the Pause NMI [D, `Endou_Plan.md`
  §2.1]. J1 stops the NMI in GG mode; the bit is J2's. In a Master System cartridge's mode the button is Pause, an NMI,
  as on the Master System.

### 2.3 The H counter's latch, measured and left for J2

FluBBa's *SMS VDP Test* run on the Game Gear model in a Master System cartridge's mode fails 9 of its 54 cases [M
2026-10-20], every one reading the H counter latched through port `$3F`'s TH; with the I/O chip's TH control given to
the Game Gear as the Master System II has it, all 54 pass. The VDP is therefore the 315-5246's in that mode, as the
documents say; whether the Game Gear's TH lines latch the H counter is an I/O question that no document read answers
(§8, question 3).

---

## 3. The picture on the LCD

### 3.1 GG mode

`gg::window` takes the VDP's picture, CRAM entries of 12 bits, from display line `gg_window_top(height)` and column 48,
160 by 144: lines 24-167 of a 192-line display [R Sega's Game Gear VDP manual: "Line 24 and line 167 are equivalent to
the top and bottom lines, respectively, on the LCD screen"; its name table figure, cells 102-121, columns 6-25], and
lines 40-183 of a 224-line one (D-21). Each entry is shown at sixteen equal steps a component (D-1's J1 note). The
scroll locks and the sprite limit off the window are the chip's, unchanged; the left column's mask has no effect on
the 315-5378 [R MacDonald §7], which the variant held since stage 3.

### 3.2 A Master System cartridge's mode

`gg::Shrink::draw` colours each of the picture's lines with the 315-5246's table, shrinks it to 160 columns once, and
makes the panel's 146 rows from four lines each, as D-22 concludes. The rows are kept in the `Shrink`, so a frame
allocates nothing (`a_game_gear_frame_allocates_nothing_in_either_mode`). Two consequences a player will see:

- The picture's frame is **160 by 146** in this mode against 160 by 144 in GG mode, since the panel lights all its rows
  here and the central 144 there [R SMS Power!'s "Pixel Aspect Ratio": "the top and bottom lines unused" in GG mode].
  Both are declared at an aspect of 4:3, so a host scaling every frame to 4:3 shows the 146 rows 1.4% shorter than
  the panel would: §8, question 1.
- **A one-pixel column is never white**, only two of its colour elements lit, as MacDonald describes; the colour
  fringes are the console's.

### 3.3 The cost

The desktop's mean a frame, release, the difference between runs of 3,600 and 600 frames over 3,000, the median of
three, the desktop loaded (load 5) [M 2026-10-20]: Sonic the Hedgehog 2 0.222 ms, Ecco the Dolphin 0.227, Mega Man
0.233 in GG mode; Sonic the Hedgehog (a Master System cartridge, 192 lines) 0.255 on the Game Gear against 0.239 as a
Master System, and Cosmic Spacehead (224 lines) 0.248 against 0.236: the shrink costs about 0.015 ms a frame. P1's
budget is 0.6 ms.

---

## 4. The gate (`Endou_Plan.md` §7, J1), measured 2026-10-20

### 4.1 The test programs

Three join `programs/` (`programs/images.rs`' `GG_PROGRAMS`), with WiseMan's `SyntheticSmsRom.GgPicture`:

- `gg.s`, GG mode: CRAM written as words through the latch, one latch written out to two entries by odd writes; the
  window's edge cells, the cells inside and outside it in three colours, the outside one never to be seen; a sprite
  across the window's left edge; a colour changed by the line interrupt at line 99; the V counter kept in the frame
  interrupt; from frame 30 the 224-line display (D-21); a write of `$FF` to port `$06`, and an NMI counter for Start.
- `ggtms.s`, GG mode in the TMS9918's Graphics I, codes 6 and 12 over a backdrop of 4 in CRAM's words 22, 28 and 20.
- `ggsms.s`, a Master System cartridge: every eighth line and column lit in each phase of three, a blue border, 192
  lines then 224 (D-22).

### 4.2 Results

| Gate item | Result [M] |
|---|---|
| GG test programs | `gg.s` at frames 10 and 40 **equal, 0 pixels**, to Genesis Plus GX, SMS Plus GX and PicoDrive up to a colour map, and to BlastEm at frame 10; the V counter in the frame interrupt `$C1` and `$E1` on all four and on Endou (D-21). Gearsystem differs by 144 pixels, the line interrupt's colour a line earlier as at stage 3 (`Endou_Native.md` §10.8), and shows the latch's second entry black. `ggtms.s` **equal, 0 pixels**, to Genesis Plus GX, SMS Plus GX, PicoDrive and BlastEm; Gearsystem shows another picture (11,520 pixels). WiseMan's `GgPicture` through the v1 adapter, every sampled pixel the word `$00F0` |
| Pictures against the references | the tester's six Game Gear games (Chuck Rock, Chuck Rock II, Ecco the Dolphin, Marble Madness, Mega Man, Sonic the Hedgehog 2), read in place, unpacked to a temporary folder and deleted: §4.3 |
| SMS mode's scaling against the references | the VDP's picture before the LCD, columns 8-247: `ggsms.s` at frames 10 and 40 **equal, 0 pixels**, to all five references and to Gearsystem's and SMS Plus GX's Game Gear modes for Master System cartridges, but BlastEm at frame 40 (7,680); Sonic the Hedgehog, Alex Kidd in Miracle World and Cosmic Spacehead at frame 600 **equal** to Gearsystem's and SMS Plus GX's Game Gear modes. The scaling itself: a second writing of D-22 from the documents (`lcd.py` in the step's scratch, Python), run on those references' pictures with their colours mapped to Endou's, gives Endou's 160 by 146 picture **byte for byte**, 23,360 of 23,360 pixels, on all five images at both heights |
| The Master System and the SG-1000 unchanged | the kit's corpus on `kit/sms-sample.txt` (33 images, 600 frames) with the base's library and this branch's: 33 of 33 pass on both, **the 33 image reports identical but for C24's list of named systems**, which now holds `gg`; `EndouTests` **19 of 19**; `CoreRegistrationEquivalenceTests`' golden recorded again, its three changed lines Endou's discovered systems gaining `gg`, and the classes that walk every system of core info (`CoreAdapterTests`, `FirmwareOverviewTests`, `ManagedCrystalCatalogTests`, `MemoryCardsOfferTests`, `SaveMediaTests`, `ConsoleScreenRuleTests`) passing; `endou`'s crate tests 98 of 98 and its allocation tests 4 of 4; FluBBa's *SMS VDP Test* on the four Master System models, every case |
| Nephrite unchanged | no common code of the crate moved: `compose` and `tms` are untouched, and `Vdp` gained `border` alone, a refactor of the blank display's colour; Nephrite's crate **194 of 194**, `NephriteTests` **22 of 22** |
| The kit | every case it runs passes (C1-C15, C17-C24) on `gg.gg`, `ggtms.gg` and `ggsms.sms`, 300 frames, C2 naming `gg` "in development, offered to no player" and C24 honouring it; in corpus form on the six Game Gear games, 6 of 6, though the corpus form makes each image unnamed, as a Master System cartridge (§6) |
| The crates' tests | `beryl-sms-vdp` **39 of 39**, `beryl-sms-colours` **3 of 3**, `endou` **98 of 98** and 4 of 4 |
| Seeded faults | §5: 12, every one caught |

Toolchains: rustc 1.99.0, .NET 10.0.112, on the desktop (Ryzen 7 7700X).

### 4.3 The tester's games against the references

Each game run to frames 300 and 600 with no input on Endou and on the five references as black boxes through the
probe, every option at its default, and the pictures compared up to a colour map, both ways (`compare.py` in the
step's scratch; BlastEm's whole frame of 256 by 275 searched for the window). No picture is committed.

| Game | Frame 300 | Frame 600 |
|---|---|---|
| Chuck Rock II, Chuck Rock, Marble Madness, Mega Man, Sonic the Hedgehog 2 | 0 pixels from each of the five | 0 from each of the five |
| Ecco the Dolphin | 0 from Genesis Plus GX, Gearsystem and PicoDrive; SMS Plus GX and BlastEm show one colour (1,456) | the same |

---

## 5. Seeded faults (2026-10-20)

Each made in the source, the crates' tests run (`endou`'s, and `beryl-sms-vdp`'s for a fault in the crate), the file
restored and compared byte for byte (`seed.py` in the step's scratch):

| # | Fault | Caught by |
|---|---|---|
| F1 | the first output pixel's blue from the first source pixel | `three_columns_become_two_by_their_colour_elements` |
| F2 | the even row's four lines from the group's first, not the line before | `a_224_line_display_s_lines_light_the_rows_benryves_measured`, the shrunk program's digests |
| F3 | the GG window at line 24 whatever the height | the GG program's frame 40, the crate's window test |
| F4 | an odd CRAM write not writing the latch | the crate's latch test, the GG and TMS programs |
| F5 | the TMS9918's codes in the 315-5246's fixed colours on the Game Gear | the crate's and the TMS program's tests |
| F6 | port `$06` decoded as `$3E` | the GG program, **after** `gg.s` was given the write that Sonic the Hedgehog 2 makes (§2.2) |
| F7 | Start an NMI in GG mode | the GG program, **after** `gg.s` was given its NMI counter |
| F8 | the Game Gear in PAL under the Europe setting | `a_game_gear_cartridge_runs_on_the_game_gear_in_the_mode_its_header_chooses` |
| F9 | region code `$4` taken as GG mode | `a_game_gear_cartridge_chooses_its_mode_by_its_header`, the v1 test |
| F10 | the border rows black | `a_192_line_display_is_letterboxed_by_its_border`, the shrunk program |
| F11 | the weighted sum truncated rather than rounded | the benryves test, the shrunk program's digests |
| F12 | CRAM's space 32 bytes in GG mode | the GG program, the v1 test |

---

## 6. Open, and named losses

- **D-22's last row** (line 221 on the 146th row against benryves's "no influence") and **the ratio of its two
  levels** are open; the border rows' colour at the frame's end is a named loss.
- **D-21** is argued; no game known uses a 224-line display in GG mode.
- **The 240-line display** on the Game Gear: the 315-5378 has none [R MacDonald §14], so the mode bits give 192 lines;
  SMS Power!'s LCD page says the console's display freezes "until the Game Gear is power cycled". Not modelled.
- **The I/O of GG mode** (§2.2) and **the H counter's latch** (§2.3) are J2's.
- **The kit's corpus form** makes each image unnamed, so the six Game Gear games ran its per-image cases as Master
  System cartridges; only the single-image form's C24 runs a `.gg` as the Game Gear. J3 needs the corpus form to name a
  system (§8, question 4).
- **The LCD itself**, its response, its grid and its colours, is the LCD model's stage (`Endou_Plan.md` §2.5, JL):
  what Endou draws is what the console sends the panel, not what the panel shows.

---

## 7. What was read for this stage

No emulator source, no SMS_MiSTer RTL, no firmware or firmware listing. Read: `Endou_Plan.md` §1.4-§1.8, §2, §3.3, §7,
§8, §10 Q1; `Endou_Native.md` §10-§11; `Endou_References.md`; `Endou_Disputes.md`'s rules, D-1 and its shape;
`Beryl_SmsVdp.md` §2; MacDonald's `msvdp` §5, §6, §7, §14 and §15; Sega's Game Gear hardware notes (SMS Power!'s
transcription): "Effective area and LCD display area", "Image display", "Standard VRAM mapping", the pattern name
table, "Color RAM", register 0 and "I/O port 00H"; SMS Power!'s "GG VDP", "Game Gear LCD", "Pixel Aspect Ratio",
"Palette", "I/O Port Map - Game Gear", "GG Cartridge", "ROM Header"; SMS Power!'s forum topic 9562, "Game Gear SMS
mode video scaling", whole, and benryves's two diagrams of a lit line's response (fetched 2026-10-20,
`Endou_References.md` §6g); FluBBa's *SMS VDP Test* readme, searched for the Game Gear; `EmuSen_CoreAPI.md` §27.4.
Not opened: the interactive demo the topic's last post links, a model's source. Of Sonic the Hedgehog 2's cartridge,
for §2.2, the bytes at `$0038`, `$0066` and `$2940`, as hexadecimal; nothing of it is kept.

---

## 8. Questions before J2

1. **One frame size for both modes?** GG mode's frame is 160 by 144, a Master System cartridge's 160 by 146, both
   declared at 4:3. Recommended: keep them as built until the LCD model (JL), which draws the panel at the display's size
   and can place GG mode's 144 rows on the panel's 146; a host's 1.4% squash in the rarer mode until then. **[decision]**
2. **The go for J2** as the plan has it: the Game Gear's I/O (Start, the region and NTSC bits of port `$00`, the link
   port's registers with nothing attached, stereo), the EEPROM boards, and its open boot replacement. Recommended: the
   go, with port `$00`'s NTSC bit first, since every Game Gear game reads PAL there today. **[decision]**
3. **The H counter's latch on the Game Gear** (§2.3): no document read says whether the Game Gear's TH lines latch it.
   Recommended: in J2, model port `$3F` as the Master System II's unless a document says otherwise, with a dispute
   entry, since that is what passes FluBBa's test. **[routine]**
4. **The kit's corpus form naming a system** (§6): recommended a `--system` option for the corpus form, a change to the
   kit proposed at J3, where the Game Gear's library is first run as players will run it. **[routine]**
