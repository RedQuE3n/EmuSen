# Beryl_SmsVdp — the Master System VDP crate's record

`beryl-sms-vdp`, in `EmuSen/Cores/Sega/Beryl-HW/`, created 2026-10-19 at Endou's stage 3 (`Endou_Native.md` §10). The
315-5124 (the Mark III and the first Master System), the 315-5246 (the Master System II) and the 315-5378 (the Game Gear)
in mode 4; the TMS9918's modes, the TMS9918A itself and the Genesis's mode 4 are stage 4's (`Endou_Plan.md` §3.3, §7).
It keeps the contract of `README.md`'s "A chip that is not a processor", with the additions §3.3 of the plan made: the
caller owns the clock; VRAM is read through a trait by the composition, so a console whose VRAM lies elsewhere keeps its
map; the picture is CRAM entries, never colours, so that the console's DAC levels stay the console's; nothing is
allocated per line or per frame.

Marks as in `Endou_Plan.md`: **[M]** measured on 2026-10-19 with the tool named; **[R]** read from the source named;
**[A]** argued; **[C]** a choice.

---

## 1. Sources

- **Charles MacDonald, *Sega Master System VDP documentation*** (`msvdp-20021112.txt`): the ports and the command word
  (§2-§3), the status flags (§4), CRAM in both consoles' forms (§5), the modes and the taller displays (§6), the
  registers (§7), patterns, the background and sprites (§8-§10), the V counter's sequences (§11), the interrupts (§12),
  the Genesis's and the Game Gear's differences (§13-§14), and the lost writes of §15.
- **SMS Power!'s development pages** (`Endou_References.md` §1.2): "VDP Registers" (the 315-5124's mask bits over the
  table addresses), "Sprites" (the 315-5124's zoom by N - 4), "Tilemap", "Tilemap Mirroring", "VRAM Memory Map",
  "Scanline Counter", "SMS1/SMS2 VDPs", "GG VDP" (the window), "Palette" (D-1).
- **Sega, *Software Reference Manual for the SEGA Mark III Console***: the command word, the registers as Sega told
  developers to set them, the line counter's table ("If you load R10 with 94, a Raster Line Interrupt Request will be
  generated every time the 95th horizontal line finishes"), "Register 9 is latched in vertical blanking", and the
  access rate ("16 Z80A T-States during VBLANK, 29 Z80A T-States during active video").
- **FluBBa's *SMS VDP Test* 1.31**, its readme and source (`testmain.asm`), read as a test's source says what it
  expects: the H counter's table of reads, the dots of the interrupts, the V counter's step and the flags (its readme's
  HClock list), and the expectation of each of its 54 Master System cases.

No emulator's source and no RTL was opened (`Endou_Plan.md` §1.4).

## 2. The chip

### 2.1 Ports and the command word [R MacDonald §2-§4]

| Access | What it does |
|---|---|
| Control write, first byte | the address's low byte at once; the flag that the second byte is next |
| Control write, second byte | the address's high six bits and the two-bit code; code 0 reads VRAM into the buffer and steps the address; code 2 writes the first byte to register `n & $0F`, numbers 11-15 doing nothing |
| Control read | the status: the frame flag, the sprite overflow, the sprite collision in bits 7-5, each cleared by the read with the line interrupt's flag and the second-byte flag; bits 4-0 are not driven (`STATUS_BITS`), and the console gives them its undriven bus |
| Data write | CRAM with code 3 (32 bytes of six bits on the Master System; on the Game Gear in its own mode a latch at an even address and a word at an odd one), else VRAM; the buffer takes the byte; the address steps and wraps at `$3FFF` |
| Data read | the buffer, refilled from VRAM at the address, which steps |
| V counter read | §3.1 |
| H counter read | the value latched when TH last rose on either joystick port; the console reports the rise (`latch_h`) |

### 2.2 The variants

`Variant` is a set of flags, as `beryl-sn76489`'s is, so that stage 4's TMS9918A and Genesis are further constants:

| | `SMS1` (315-5124) | `SMS2` (315-5246) | `GG` (315-5378) |
|---|---|---|---|
| Register 2 bit 0, 3, 4, 5 bit 0 and 6 bits 1-0 mask mode 4's addresses [R "VDP Registers"] | yes | no | no |
| The 224-line display / the 240-line display [R MacDonald §6, §14] | no / no | yes / yes | yes / no |
| Sprites zoomed horizontally | the first N - 4 of N on a line (D-6) | all | all |
| Register 0 bit 5, the left column's mask [R MacDonald §7] | yes | yes | no effect |
| CRAM | 32 bytes | 32 bytes | 64 in GG mode, through the latch |
| The invalid text mode | M4 with M1 | M4 with M1 and M2 clear | as `SMS2` |

The Game Gear's window, the central 160 by 144 of the picture [R "GG VDP"], is `GG_WINDOW`, for the Game Gear's stage.

### 2.3 Modes and heights

Mode 4 is M4 set and not the invalid text mode; the TMS9918's modes and the invalid text mode show the backdrop until
stage 4 builds them, with no sprites [C]. The display has 224 lines where M2 is set with M1 alone and 240 where it is
set with M3 alone, on the chips that have them, else 192 [R MacDonald §6].

### 2.4 At power-on [C]

Registers, VRAM and CRAM clear, the address 0, every flag clear, on the frame's last line at its V counter's step. No
document read gives a console's power-on VDP state; every program and game Endou has run sets the registers it uses.
The chip starts on the last line so that a frame counted from power-on, as Endou counts its frames, holds one whole
picture and ends between two (§3.3).

## 3. Timing

### 3.1 The clock and the counters

The chip counts its own clock, the master clock over five, 684 a line, two a dot [R MacDonald §11: 342 pixels a line;
FluBBa's readme: "MClock (0->683)"]; the Z80's clock is three of them. Lines a frame: 262 NTSC, 313 PAL.

**The H counter** counts `$000`-`$127` and then `$1D2`-`$1FF`, 342 values, read as its top eight bits [M: FluBBa's test
reads it every 229 Z80 cycles and holds the reads to a table that runs `$00`-`$93`, `$E9`-`$FF`, two lines of it].
MacDonald's §11 gives the jump at `$8B`-`$ED` from a Genesis, and warns that "a real SMS may have different results";
the test's table is taken.

**The V counter** runs linearly and jumps back where MacDonald's §11 lists it, in each display height and standard; the
NTSC 240-line display, which "does not work", keeps its 262 lines and wraps through zero [A]. Its value is the line's,
the display's height at the time choosing the sequence.

### 3.2 Where a line's events fall [M]

A line is counted from the V counter's step. FluBBa's readme puts the frame interrupt, the line interrupt, the V
counter's step and the overflow flag at HClock -18, -17, -16 and -15. Its test's timing cases (the H counter's table,
the V counter's change, the frame and line interrupts' H counts, the frame flag's H count, the collision's and the
overflow's) measure each against an H counter latched through TH, so the events, the latch and the picture are held to
each other rather than to an absolute dot. They were fitted on Endou on 2026-10-19, varying each (the H counter at the
step over `$1E0`-`$1F4`, the latch's delay over 0-5 of the chip's clocks, the overflow's dot, the picture's first dot),
until every case passed:

| Event | Where, as the H counter reads it | Its case |
|---|---|---|
| The V counter steps; on the line after the last active one, the frame flag and INT | `$1E8` | "VCounter chg time", "Frame IRQ VCount", "Frame IRQ HCount", "VINT flag HCount" |
| The line counter steps or reloads, and its underflow raises the line flag; the X scroll is taken for the line | `$1E9` | "Line IRQ VCount", "Line IRQ HCount" |
| The line's sprites are found and the overflow flag set | `$1E8`, with the step | "Spr ovr correct HC" |
| Pixel 0 | `$00A`: the active display is H `$00A`-`$109` | "Spr col correct HC" |
| The H counter is latched | two of the chip's clocks, a dot, after TH rises | "HCounter correct" |

The fit holds the readme's order (the frame interrupt and the step together, the line interrupt a dot later) and moves
all three six dots earlier than its HClock, which is the readme's own emulator's count and not the H counter's. Every
case passes on the 315-5124 and the 315-5246, NTSC and PAL (§7).

**The line counter** steps as each line from 0 to the first after the display begins, and is reloaded as every other
line begins [R MacDonald §12: "decremented on lines 0-191 and 192"]; its underflow from 0 reloads it from register 10
and raises the line flag [R]. FluBBa's "Line IRQ VCount" reads the V counter in the handler as the line number register
10 names, which places the step as the line begins, not as it ends; Sega's manual agrees ("If you load R10 with 94 ...
every time the 95th horizontal line finishes").

**The frame interrupt** is raised as line `$C1`, `$E1` or `$F1` begins [R MacDonald §12; M "Frame IRQ VCount"]. INT is
held while a flag and its enable are set [R §12].

**The Y scroll** is taken as a frame's first line begins [R MacDonald §9; Sega's manual]; **the X scroll** for each
line at its line counter's dot [R the readme]; the name table's base, the patterns and CRAM are read as each pixel is
drawn [R the readme: "NameTable base is not latched"].

### 3.3 The frame

The chip powers on at its frame's last line, so that a console which runs a frame of 262 or 313 lines from power-on
ends each at the start of the chip's last line, after the picture is whole and before the next begins. Nothing of a
picture is then in flight at a frame's end, so a state saved there holds no pixels, and a frame whose picture is
skipped leaves the chip's state as a drawn one (§6, P9 of `Endou_Plan.md`).

## 4. The composition (`compose`)

The composition of a line, written to be shared: the sprites a line holds, their pixels, the background's pixels and
the mix, each a function over a `Vram`, the registers and the variant, writing CRAM indices into the caller's buffers.

- **The background** [R MacDonald §9; "Tilemap"]: register 8's coarse scroll subtracted from 32 gives the first name
  column, and the column counter's drawing starts the fine scroll's pixels to the right, those pixels showing the
  backdrop (D-7). The top two rows' lock is by the line (0-15), the right columns' by the column counter (24-31), so
  with a fine scroll it begins at pixel 192 plus the fine scroll [R §9: "it is the column counter, not the starting
  column value, that is checked"]. The 192-line display's table is 32 by 28, its Y wrapping at 224; the taller ones'
  32 by 32 at register 2's bits 3-2 plus `$700`. Entries are `---pcvhnnnnnnnnn`.
- **The 315-5124's masks** [R "VDP Registers", "Tilemap Mirroring"]: register 2's bit 0 clear forces the name table
  address's bit 10 (the row's bit 4) low; register 3's bits mask the pattern address's bits 13-6 for planes 0 and 1,
  register 4's bits 2-0 its bits 13-11 for planes 2 and 3; register 5's bit 0 masks bit 7 of the sprites' X and pattern
  fetches; register 6's bits 1-0 mask bits 12-11 of the sprite pattern address.
- **Sprites** [R MacDonald §10; "Sprites"]: the table is scanned for the line's V counter, a sprite on the lines from
  its Y plus 1, 8 or 16 tall, doubled by the zoom; `$D0` ends the table in the 192-line display only; a ninth sprite on
  the line sets the overflow flag and ends the scan, on an active line only (FluBBa's "Offscreen Y, no ovr"); X less
  eight with the early clock, no wrap at either edge, pattern index's bit 0 ignored when tall, register 6's bit 2 a
  global ninth bit. An earlier sprite's opaque pixel is over a later one's; two opaque pixels meeting set the collision
  flag at that pixel, while the display is on, on any line (FluBBa's "Offscreen Y, col"), never past pixel 255.
- **The mix** [R MacDonald §9-§10; Sega's manual]: a sprite's pixel where it is opaque, unless the tile has priority and
  its pixel is not colour 0 of either palette; then register 0's bit 5 puts the backdrop, CRAM entry 16 plus register
  7, over pixels 0-7.

### 4.4 A blanked display, and the modes stage 4 builds

Register 1's bit 6 clear shows the backdrop over the active display [A: MacDonald's §11 describes the border, not a
blanked picture; every reference shows the backdrop]; no sprite collides. The TMS9918's modes show the backdrop too
until stage 4.

## 5. The dot-stamped renderer

`Endou_Plan.md` §5.2's design. The chip runs a line's three events at their clocks (the line counter, the line's last
pixel, the next step); the pixels between are drawn when something could change them: before every register, VRAM or
CRAM write and every status read, the line is drawn up to the dot the chip is at, and the rest at the line's end. A line
with no write costs one composition; a write shows from the next pixel. The background's tile row is decoded once a
column. The sprites' pixels are made when the line's sprites are found, with the collision's first pixel, so that a
skipped picture still finds the collision at its pixel.

**Not modelled** [A]: the chip's fetches run ahead of the pixels they draw, so a name table or pattern write lands
some dots before its pixel on the console; no document read gives the fetch order, and no test program measures it.

## 6. The state block

`write_state`/`read_state`: the variant's byte, PAL and the GG mode, the registers, VRAM, all 64 bytes of CRAM, the
address, the code, the buffer and the latch, the five flags, the line counter, both scrolls, the latched H counter,
the clock, the line's start, the line, the pixels drawn and the height, the next event, and the line's eight sprites
(their X, width and planes), whose pixels are made again on loading. A block of another variant or standard, or of an
impossible line, is refused and changes nothing. No picture is in it (§3.3).

## 7. Tests (2026-10-19)

The crate's 23 [M `cargo test`, and on `test-checked`]: the timing tables (MacDonald's six V counter sequences, the H
counter's run); the composition (the Y plus 1 and `$D0`, the ninth sprite, sprite priority and collision, the 315-5124's
zoom, the background's scroll and blank, the Y wrap and the column lock, the 315-5124's masks, the priority); and the
chip (the command word, the registers, CRAM in both forms, the frame interrupt, the line counter over a frame, the H
latch, a write at its dot, the mask and a blanked display, sprites over the background with the overflow and the
collision at their dots, the scrolls' latches, the heights per variant, the state's round trip, and skipping's
neutrality). Sixteen faults were seeded across the crate and Endou (`Endou_Native.md` §10.9); the four the tests here
missed at first, or caught only through FluBBa's test, each gained a test.

On the machine, `Endou_Native.md` §10.7: FluBBa's *SMS VDP Test*, its 54 Master System cases, all pass on the
315-5124 and the 315-5246, NTSC and PAL, and Endou's own programs.

## 8. The access slots (D-8)

Sega's manual says the chip "cannot process data any faster than ... 16 Z80A T-States during VBLANK, 29 Z80A T-States
during active video"; MacDonald found CRAM writes lost or misplaced when written "rapidly during the active display" on
a Master System II, not on a Game Gear or a Genesis (§15). Which dots serve the CPU is in no document read. The crate
loses no access: `Endou_Disputes.md` D-8 gives the evidence, chiefly that 122 of the library's games access the data
port faster than the manual's 29 cycles during the active display, and every reference keeps every write of a program
written to find a loss.

## 9. For stage 4

- **The TMS9918's modes** and **the TMS9918A** as a variant: Graphic I and II, Text, Multicolor and the undocumented
  combinations, with the 315-5246's fixed palette, through `mode4()`'s complement; the invalid text mode.
- **The Genesis** as a variant, its composition through `compose` with a `Vram` of Nephrite's D-7 map: its differences
  are MacDonald's §13 (the first byte latched, CRAM by code bit 1, no buffer refill on a write, no zoom, no taller
  displays, no register 2 mask) and its left column's garbage under a fine scroll, which `background`'s fine-scroll
  blank then becomes a variant's choice.
