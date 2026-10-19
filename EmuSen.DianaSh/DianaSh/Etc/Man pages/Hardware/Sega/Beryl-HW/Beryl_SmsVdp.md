# Beryl_SmsVdp — the Master System VDP crate's record

`beryl-sms-vdp`, in `EmuSen/Cores/Sega/Beryl-HW/`, created 2026-10-19 at Endou's stage 3 (`Endou_Native.md` §10). The
315-5124 (the Mark III and the first Master System), the 315-5246 (the Master System II) and the 315-5378 (the Game Gear)
in mode 4; since stage 4 (`Endou_Native.md` §11) the TMS9918's modes on all of them, the TMS9918A itself (the
SG-1000), and the Genesis's mode 4 through `compose`, which Nephrite draws with (`Endou_Plan.md` §3.3, §7).
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

- **Texas Instruments, *TMS9918A/TMS9928A/TMS9929A Video Display Processors* data manual** (`TMS9918.pdf`, read at
  stage 4 as page images): the CPU interface and Table 2-1's transfers (§2.1), the registers (§2.2, Figure 2-3), the
  status register (§2.3), the display planes and modes (§2.4), Table 2-3's colours, the sprites (§2.4.5).
- **Sean Young, *Texas Instruments TMS9918A VDP*** (0.4.2): the undocumented modes (§3.5-§3.7), the Graphics II masks
  (§3.3), the fifth sprite number when no fifth sprite is flagged (§4.3), the 4K mode (§2.1, MacDonald's letter).
- **MacDonald, *Sega SC-3000 Hardware Notes*** (`sc3000h` §6): the TMS9929A's register writes, its read buffer and its
  4K mode, tested on a console.

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

`Variant` is a set of flags, as `beryl-sn76489`'s is:

| | `SMS1` (315-5124) | `SMS2` (315-5246) | `GG` (315-5378) | `TMS9918A` | `GENESIS` (`compose` only) |
|---|---|---|---|---|---|
| Register 2 bit 0, 3, 4, 5 bit 0 and 6 bits 1-0 mask mode 4's addresses [R "VDP Registers"]; registers 3 and 4 mask Graphics II's tables [R MacDonald §15; Young §3.3] | yes | no | no | Graphics II's | no |
| The 224-line display / the 240-line display [R MacDonald §6, §14] | no / no | yes / yes | yes / no | no / no | no / no |
| Sprites zoomed by register 1's bit 0 in mode 4 [R MacDonald §13] | the first N - 4 of N widened (D-6) | all | all | (no mode 4) | none |
| Register 0 bit 5, the left column's mask [R MacDonald §7] | yes | yes | no effect | — | yes |
| The fine scroll's first pixels | the backdrop (D-7) | the backdrop | the backdrop | — | the column before, wrapped |
| The right columns' vertical lock | by the column counter [R MacDonald §9] | the same | the same | — | by the pixel, from 192 (§9.4) |
| CRAM | 32 bytes | 32 bytes | 64 in GG mode, through the latch | none | the console's |
| A TMS9918 colour code shows | a fixed colour of the 64 [R SMS Power!'s "Palette"] | the same | CRAM's entry 16 plus the code [R MacDonald §14] | the code, the console's to colour | black (Nephrite's own) |
| The invalid text mode | M4 with M1 | M4 with M1 and M2 clear | as `SMS2` | — | — |

The `GENESIS` variant is never a `Vdp`: the 315-5313 is Nephrite's chip, and only its mode 4's composition is shared.

The Game Gear's window, the central 160 by 144 of the picture [R "GG VDP"], is `GG_WINDOW`, for the Game Gear's stage.

### 2.3 Modes and heights

Mode 4 is M4 set and not the invalid text mode; M4 clear selects the TMS9918's modes (§9), and the TMS9918A has no M4.
The invalid text mode, MacDonald's "identical to TMS9918 text mode" with CRAM's colours and tiles "repeated four
times", is not modelled: it shows the backdrop, with no sprites [C], a named loss, since no document read says how its
tiles repeat. The display has 224 lines where M2 is set with M1 alone and 240 where it is
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

`line` composes a span: the background, the sprites mixed over it, then the left column's mask. The Genesis's
variant differs only by the flags of §2.2.

### 4.4 A blanked display

Register 1's bit 6 clear shows the backdrop over the active display [A: MacDonald's §11 describes the border, not a
blanked picture; every reference shows the backdrop]; no sprite collides. In the TMS9918's modes the backdrop is
register 7's colour code [R TI §2.4], and the sprite system is inactive [R Young §1.2].

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
address, the code, the buffer and the latch, the five flags and whether the line's sprites are the TMS9918's, the
line counter, both scrolls, the latched H counter and the TMS9918A's fifth sprite number, the clock, the line's start,
the line, the pixels drawn and the height, the next event, and the line's eight sprites (their X, width and planes),
whose pixels are made again on loading by the rules that found them. Stage 4 added the second and fourth (two bytes). A block of another variant or standard, or of an
impossible line, is refused and changes nothing. No picture is in it (§3.3).

## 7. Tests (2026-10-19)

At stage 4 the crate has 37 [M `cargo test --profile test-checked`]: stage 3's 23, and fourteen for the TMS9918's modes
and the variants (the modes' selection, Graphics I, Graphics II's masks by chip, Text's high six bits and the 1+3
bars, Multicolor's nibbles, four sprites and the fifth's number, the early clock, the size, the magnification and the
colour 0 collision, the 4K map one to one, the TMS9918A's register write and lack of CRAM, the fifth sprite flag with
the frame flag, the codes and fixed colours drawn, the 4K mode on the CPU's cells, a TMS mode's state round trip and
skipping's neutrality, and the Genesis's composition). Stage 3's 23 [M `cargo test`, and on `test-checked`]: the timing tables (MacDonald's six V counter sequences, the H
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

## 9. The TMS9918's modes and the TMS9918A (stage 4, 2026-10-19)

### 9.1 The modes (`tms`)

M1 (register 1's bit 4), M2 (register 0's bit 1) and M3 (register 1's bit 3), named as MacDonald and Young name them
(TI's manual calls the last two M3 and M2), select [R TI §2.2-§2.4; Young §3; MacDonald §6]:

| Mode | The pattern plane | Sprites |
|---|---|---|
| Graphics I | 32 by 24 names; a pattern's byte a row, its set bits the high nibble of the colour byte of its group of eight names, its clear bits the low | yes |
| Graphics II | the screen's thirds each with their own 256 patterns and colour rows: register 4's bit 2 and register 3's bit 7 place the tables, their lower bits an AND mask over the third's index on the TMS9918A and the 315-5124 (§2.2) | yes |
| Text | 40 columns of 6 pixels between borders of 8, a pattern's high six bits [R TI §2.4.4: "the least significant 2 bits ... are ignored"; Young §3.2 says the highest, read as TI's bit numbering] in register 7's two colours | no |
| Multicolor | 4 by 4 blocks, two bytes a name chosen by the row, each byte's high nibble the left block [R TI §2.4.3] | yes |
| 1+2, 2+3 | Text and Multicolor with Graphics II's thirds and masks [R Young §3.5, §3.6] | no, yes |
| 1+3, 1+2+3 | forty columns of four pixels of the text colour and two of the backdrop [R Young §3.7] | no |

A code of 0 is transparent and shows the backdrop, register 7's low nibble.

### 9.2 Sprites in these modes

32 entries of four bytes at register 5 times `$80`: Y, X, the name, and the colour with the early clock bit, which moves
the sprite 32 pixels left [R TI §2.4.5]. A sprite shows from the line after its Y; `$D0` ends the table; 8 or 16 pixels
square (register 1's bit 1), the 16-pixel pattern four quarters from the name with its low two bits clear, doubled by
register 1's bit 0. **Four a line**: a fifth sets the status's bit 6, and on the TMS9918A its number goes in bits 4-0
[R TI §2.3.3]. The Sega chips keep four too (`Endou_Disputes.md` D-10) and shift sprites a further eight pixels by
register 0's bit 3 [R MacDonald §15]. An earlier sprite's set pixel shows over a later one's unless its colour is 0;
two set pixels meet as a collision whatever their colours [R TI §2.3.2, §2.4], among the four shown, never past pixel 255.

### 9.3 The TMS9918A

- **Registers**: eight. A second control byte with bit 7 set writes the register its bits 2-0 name, bits 6-3 ignored [R
  TI Table 2-1; `sc3000h` §6]; else bit 6 chooses a write or a read with its prefetch. No CRAM. The first byte is taken
  at once, as the 315-5124 takes it [A: no document read says otherwise].
- **Status**: F, 5S, C and the fifth sprite's number. 5S is set on an active line with a fifth sprite only while F is
  clear [R TI §2.3.3]; while it is clear the number bits hold where each line's scan ended, the terminator's number or
  31 [R Young §4.3, "not fully checked"]. A read clears F, 5S and C. Every bit is driven (`status_bits`).
- **No line interrupt**, no M4 and no counters at ports `$40`-`$7F` [R TI §2.2; SMS Power!'s "I/O Port Map - SG-1000"].
- **The 4K mode**: register 1's bit 7 clear makes the chip put out a 4K DRAM's row and column, so that every access,
  the CPU's and the display's, reaches the cell `tms::address_4k` names [R `sc3000h` §6, tested by MacDonald on a
  TMS9929A; Young §2.1]. `D-12` records that no reference models it so.
- **Colour codes** are the picture; Endou colours them by TI's Table 2-3 (`Endou_Disputes.md` D-11).
- **Timing** is the family's (§3) [A]: TI's manual puts the interrupt "at the end of each active-display scan" and gives
  no dot; the CPU's access windows (TI's Table 2-2, 2 to 8 microseconds in the Graphics modes) are not modelled, as the
  Sega chips' are not (D-8).

### 9.4 The Genesis's mode 4

`Variant::GENESIS` carries MacDonald's §13 differences that a composition sees (no zoom, no taller displays, no masks)
and two rules Nephrite drew before the move and keeps: the fine scroll's first pixels are the wrapped column, where the
board shows garbage (`Nephrite_Native.md` §15.2), and the right columns' lock begins at pixel 192 rather than at the
column counter's column 24. The second is not a measured difference: it is Nephrite's rule, kept so that the move
changed nothing, and whether the 315-5313 locks by the counter as MacDonald says of the Master System's chips is a
question for the board (`Endou_Native.md` §11.15). The chip's other differences (the first byte latched, CRAM by code
bit 1, no buffer refill) are Nephrite's VDP's, outside the composition.
