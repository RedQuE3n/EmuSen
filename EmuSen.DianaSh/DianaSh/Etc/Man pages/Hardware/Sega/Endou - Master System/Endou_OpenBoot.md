# Endou_OpenBoot — the open boot program (stage 6)

*Written 2026-10-19, before any of stage 6's code, by a writer under the plan's replacement writers' rule
(`Endou_Plan.md` §1.4, §5.5), as the tester decided the same day (`Endou_Native.md` §12.14).* This page is stage 6's
plan: what the documents say a Master System's BIOS does that a game can see, what they leave unsaid and what Endou
chooses there, the program's steps and the bus it runs on, the player's own BIOS as an opt-in path per console model,
the steps with their oracles, the accuracy cost for the firmware's `replacement` words, and the questions only the
tester can answer. It takes the lesson of `Nephrite_OpenBios.md`, whose first finding stopped its stage at the plan:
here the documents give every number the program needs, and the assembler exists (`Beryl_Z80.md` §8), so the stage
goes on to build.

Marks as in the plan: **[D]** stated by a document, cited; **[R]** read on 2026-10-19 from the source named; **[M]**
measured on 2026-10-19 with the tool named; **[A]** argued; **[C]** a choice.

---

## 0. The clean room

The program is written without any real BIOS being seen. No BIOS image was read, disassembled, hashed, compared or
run for this page or for the program: not the tester's images in `~/Documents/BIOS/SMS/`, not the six firmware files in
the library's `mastersystem/` folder (four Master System BIOS images and two copier boot programs, whose names say so
and which every survey of this page leaves out by name), not anything in the probe's cache. The excluded documents of
`Endou_References.md` §5 stayed unopened: MacDonald's `smstech` §6, "BIOS information" (lines 434–458 of the file
were skipped when the rest was read), and SMS Power!'s `BIOS` page. No emulator's source was read.

**Read for this page** [R]: Sega's *Software Reference Manual for the SEGA Mark III Console* (SMS Power!'s
transcription, cited **MK3** by its section names), "The CPU", "Reset", "Interrupt system", "VDP register
description", "Memory management", "Memory enables", "Memory selection at power-on", "Memory maps", "The cartridge
header", "Input/Output (I/O) ports" and "How the trackball works"; MacDonald's `smstech` §1–§5 and §7 (**ST**);
Talbot-Watkins's VDP registers (**TW**); SMS Power!'s "Port $3E", "RAM", "Memory Map", "ROM Header", "Card Slot",
"Cartridge Slot", "Bus Signals", "Mappers", "Codemasters Header", "FM Chip Detection", "Peripheral Ports", "Reset
Button", "Interrupt Vectors", "VDP Registers" (**SP** with the page's name). Before the Mark III manual's sections were
read, they were screened mechanically for program text (lines beginning with a Z80 mnemonic): the power-on and header
sections hold none, and the manual's one listing near them is the two-instruction trackball initialisation quoted in
§1.4. The manual is a developer's document for a console Sega sold, not a firmware listing.

**Measured for this page** [M]: the headers of the library's 310 Master System game images (read in place, the six
firmware files left out by name), each one's stated checksum against sums computed by a Python reader over the range
its size code names, three ways (§1.6).

## 1. What the documents say the BIOS does

### 1.1 Which consoles have one

- The Master System and the Master System II have a "resident 8 Kilobyte ROM" at power-on (MK3, "Memory selection at
  power-on"; "base unit 8K ROM", MK3 "Memory enables"); the Japanese Master System has one too (ST §1's table). The
  Japanese Master System joins Endou at stage 9, with its BIOS.
- **The Mark III** is the disputed case: ST §1's table gives it a BIOS slot, while SP "Port $3E" calls the Mark III and
  earlier consoles the ones that "used a hardware method to activate only one slot", a precedence "reversed in
  BIOS-using systems", and SP "Card Slot" gives the Mark III's cartridge priority over a card where the Master System's
  card takes priority. The manual's boot (§1.3) checks the card first, so it describes a console whose card takes
  priority, the Master System. **Argued: the Mark III has no BIOS** [A], as `Endou_Plan.md` §5.5 had it; recorded as
  `Endou_Disputes.md` D-14.
- The SG-1000 has none (`Endou_Plan.md` §5.5).

### 1.2 Reset and the processor

- "The Z80A executes a RESET cycle when the base unit is turned on. This is the only Z80A reset mechanism
  implemented"; the Reset button is "attached to an input port for software polling" (MK3, "Reset").
- "The boot ROM executes the 'IM 1' instruction at power-on" (MK3, "INT").
- At power-on the Z80 "begins executing code at location $0000 in the resident 8 Kilobyte ROM" (MK3, "Memory
  selection at power-on").

### 1.3 Choosing the medium, and the hand-over

MK3, "Memory selection at power-on", in full as its steps: the system is initialised; "a small program is copied from
ROM into the RAM (at $C700)" and run there; "all of the system ROM spaces are disabled", so that the slots "can then be
checked one by one"; they are "individually enabled and checked for the presence of memory ... in the following order:
front card slot, top cartridge slot, external (rear) slot"; "if memory is detected in any of these three slots, the
slot is enabled and the Z80A executes a jump to location $0000"; "if no memory is detected, the console ROM is enabled
and an 'Insert a cartridge' message is put on the screen". "If both the card and cartridge are plugged in, the card
will be enabled because it is checked first." SP "Port $3E" says the same order, and that the BIOS "runs the code from
the first slot with valid data"; ST §4: "The BIOS only enables the cartridge slot after relocating itself to work RAM
and disabling the BIOS beforehand."

- **The value handed over.** Port `$3E` is `$AB` "when running a cartridge", on the snail maze BIOS's Master System and
  on the Alex Kidd Master System II (ST §4). Its bits (ST §4, MK3 "Memory enables"): 7 the expansion slot, 6 the
  cartridge, 5 the card, 4 the work RAM, 3 the BIOS, 2 the I/O chip, each enabled when clear; 1 and 0 unused. So `$AB`
  is the cartridge, RAM and I/O on, the rest off, with bits 1 and 0 set; the same rule gives the card `$CB` and the
  expansion slot `$6B` [A].
- **The value left at `$C000`.** SP "FM Chip Detection": the routine "requires memory address $c000 to contain the
  correct value to write to port $3e to enable the correct slot, RAM and controller inputs. This is ordinarily the case
  when running on any Master System with a BIOS." SP "RAM": "By default the RAM is always enabled by the Master System
  BIOS."

### 1.4 What the initialisation sets

- **The VDP's registers** (MK3, "VDP register description"): "The VDP registers are initialized at power on to the
  following values": R0 `$36`, R1 `$A0`, R2 `$FF`, R3 `$FF`, R4 `$FF`, R5 `$FF`, R6 `$FB`, R7 `$00`, R8 `$00`, R9 `$00`,
  R10 `$FF`. TW agrees for R2 ("the default name table address of $3800") and R5 ("a default sprite table base of
  $3F00"). Whether the chip itself or the boot ROM sets them is not said; the open boot program writes them, which
  holds under either reading, and the Mark III and SG-1000's power-on registers stay `Beryl_SmsVdp.md` §2.4's.
- **The I/O control port** (MK3, "How the trackball works"): before the trackball is used, `LD A,10100101b` and
  `OUT (03FH),A` "should be executed to set the STROBE bit directions to output, and initialize the strobe signals to
  1. NOTE: these instructions are executed in the power-on initialization." Read as the system's initialisation, port
  `$3F` is `$A5` at the hand-over: both TH pins outputs at a high level, both TR pins inputs. SP "Peripheral Ports"
  says the port "defaults to an input-only state ... on startup", which is the chip's reset, before any program
  [A]. The reading is argued, `Endou_Disputes.md` D-15.
- **The stack**: games "should" place it at `$DFF8` (MK3, "System RAM"), or `$DFF0` (SP "RAM").

### 1.5 The export check

SP "ROM Header": "The BIOS in non-Japanese Master Systems requires a valid header"; "The export Master System and Game
Gear BIOSes require [TMR SEGA] to be present to indicate valid data"; the checksum at `$7FFA` "gives the ROM checksum
for export SMS BIOSes. ... The BIOS checksum routines compare the calculated value to this to determine if the cartridge
data is valid"; the region code in the high four bits of `$7FFF`, `$3` SMS Japan and `$4` SMS Export among them, "Only
the export SMS BIOS actually checks this"; the size code in the low four bits "may be used by the BIOS to determine the
range over which to perform the checksum": `$A` 8 KiB, `$B` 16, `$C` 32, `$D` 48, `$E` 64, `$F` 128, `$0` 256, `$1`
512, `$2` 1 MiB, with `$D` and `$2` "buggy" because "some or all of the BIOSes that implement checksum validation have
errors in their implementations". The header may be at `$1FF0`, `$3FF0` or `$7FF0`. MK3, "The cartridge header": "The
presence of a cartridge is tested by checking for specific information in ROM. This information must be correct for
the cartridge to be recognized", the checksum being "a 16-bit checksum for the ROM (low-hi order). $7FFA and $7FFB are
not included in this checksum." SP "Mappers": Sega's mapper "is assumed to be there (for games larger than 48KB) by
the BIOS"; MK3, "128 kilobytes": banks are paged into `$8000`–`$BFFF` by the register at `$FFFF`. SP "Codemasters
Header": "the normal BIOS checksum routine does not work due to the different mapper".

### 1.6 Which bytes the sum covers, measured on the games

The manual's sentence leaves two readings of the range: every byte the size code names but the checksum's own two, or
those bytes less the whole 16-byte header. Sega mastered each export cartridge so that its BIOS accepted it, so a
game's stated checksum is evidence of the sum the BIOS makes. **Measured 2026-10-19** on the 305 game images whose
header names an export console (`$4`) or another code [M, a Python reader, headers read in place]:

| Rule | Images whose stated checksum it gives |
|---|---:|
| the size code's bytes but `$7FFA`–`$7FFB` (the manual's words, and stage 1's `media::checksum`) | 0 of 305 |
| the size code's bytes but the whole header, `$7FF0`–`$7FFF` | **300 of 305** |
| the same, read as the console reads it: `$0000`–`$7FFF` direct, then each bank from 2 paged into `$8000`–`$BFFF` by `$FFFF` | 301 of 305: the 300, and *Micro Machines* (E), on Codemasters' board |

*Micro Machines* states `$E352` with size code `$0` (256 KiB). A write to `$FFFF` reaches no register on Codemasters'
board, whose slot 2 holds bank 0 from power-on (`Endou_Native.md` §9.3), so a BIOS that pages banks 2 to 15 through
`$FFFF` reads bank 0 fourteen times; that sum is `$E352` exactly, while the image's own banks sum to `$7CC5`. The
cartridge was mastered to the sum the BIOS reads through its board, which is the measurement's strongest evidence that
the BIOS reads the first 32 KiB directly and pages the rest through `$FFFF` alone. Of the other four, three Brazilian
images (*Felipe em Ação*, *Teddy Boy e Geraldinho*, *Treinamento do Mymo*) and a Korean one (*Toto World 3*, size code
`$A`) give their stated sum under no range tried, from 8 KiB to the whole image; the two 32 KiB Brazilian images state
the same `$B351`, a header carried over unchanged from another game. And *Bank Panic* (UE), the one image with region
code `$2`, a code no document lists, states the sum of its first 32 KiB while its size code says 256 KiB.

## 2. What the documents leave unsaid, and what Endou chooses

| | Unsaid | Endou's choice | Mark |
|---|---|---|---|
| U1 | Port `$3E`'s value at reset with a BIOS | `$E3`: the BIOS, the work RAM and the I/O chip on, the three slots off. The BIOS must be the only thing in the slot area for the first instruction, since on the Master System the cartridge "has precedence" over the BIOS when both are on (ST §4) | [A] |
| U2 | What an empty slot or the BIOS reads as, and how the BIOS maps | An empty card or expansion slot, and an absent cartridge, drive nothing: the floating bus on the Master System, `$FF` on the Master System II (ST §4, "the last byte of the instruction which read memory"). The 8 KiB BIOS is mirrored through `$0000`–`$BFFF`, as a ROM with thirteen address lines whose chip enable is `#MREQ + PORT$3E[3]` is (SP "Bus Signals") | [D], [A] for the mirror |
| U3 | Both the BIOS and the cartridge on | The cartridge's byte on the Master System, the two ANDed on the Master System II (ST §4); the Master System II has no card or expansion slot, its bits 7 and 5 doing nothing | [D] |
| U4 | How "the presence of memory" is detected in a slot without a header | A slot is present when `TMR SEGA` is at `$7FF0`, `$3FF0` or `$1FF0`, or when 64 bytes read across the slot area are not all one value; an empty slot reads one value throughout, the same instruction's byte or `$FF` | [A] |
| U5 | The region code's test | Present and `$4`, "SMS Export"; `$3` refused as a Japanese game, any other as not this console's | [A] from §1.5 |
| U6 | The "buggy" size codes `$D` and `$2`, and codes no document lists | `$D` and `$2` summed as their sizes say, the bugs being unknown; an unlisted code refused | [A] |
| U7 | What the screen shows, and for how long | The program's own screen, its name in text and no logo, shown while it checks; a refusal or an empty console adds a line in words and waits | [C], `Endou_Plan.md` §5.5 |
| U8 | The RAM, the VRAM and the PSG left at the hand-over | RAM all zero but `$C000` (the value handed over), the program left at `$C700`, and `$DFFF` holding 2 where the sum paged slot 2, which is put back to bank 2 as the mapper powers on; VRAM and CRAM cleared; the PSG's four channels silenced | [C] |
| U9 | The processor at the hand-over | Interrupts disabled, mode 1, the stack pointer `$DFF0`, A the value handed over; R as the program's own fetches leave it, as any boot program's do | [C], mode 1 [D] |
| U10 | When in the frame the game starts | On the VDP's last line, as the console is at power-on (`Beryl_SmsVdp.md` §2.4): the program waits for the V counter to read `$FF` and then hands over, so that a game starts at the phase it starts at with no BIOS, some frames later | [C] |
| U11 | The VDP's pending flags | The status read once before the hand-over, so no interrupt is pending | [C] |
| U12 | The pace of the program's own VDP writes | At least 16 cycles apart with the display off and 29 with it on, MK3's rates, so that the program would run on a console whatever the slots of D-8 are | [D] |

## 3. The program

Z80 assembly in `src/boot/` of Endou's crate, assembled by `build.rs` with `beryl-z80`'s assembler into an 8 KiB image
in `OUT_DIR` (`open_boot.bin`), as Nephrite's `build.rs` builds its open BIOS. Its font is Nephrite's, moved to a Beryl
crate of its own (`beryl-font`), since two cores' firmware now draws it: one owner of the glyphs, each `build.rs` making
its own console's tiles from them (a Genesis's packed four-bit pixels, a Master System's four bitplanes).

### 3.1 In ROM, from `$0000`

1. `DI`, `IM 1`, the stack at `$DFF0`; `RETN` at `$0066` and `RETI` at `$0038`, though interrupts stay off.
2. `$3E` written `$E3` (U1); `$3F` written `$A5` (§1.4); the PSG's four volumes written off (U8).
3. The VDP: display off, the registers for its screen, VRAM's name table, the sprite list's end (`$D0` at `$3F00`), the
   font's 64 tiles and four CRAM entries, the program's name centred; then the display on.
4. The RAM program copied to `$C700` and entered.

### 3.2 In RAM, from `$C700`

5. Each slot in turn, card (`$CB`), cartridge (`$AB`), expansion (`$6B`), written to `$3E` with the BIOS off; the first
   one present (U4) is chosen. None: `$3E` back to `$E3`, and the ROM's "no game" screen.
6. The export check on the chosen slot (§1.5, §1.6, U5, U6): the header, its region, then the sum of its size code's
   range less the header, `$0000`–`$7FFF` read directly and each further bank paged into `$8000`–`$BFFF` by `$FFFF`,
   against the stated word. A failure: `$3E` back to `$E3`, and the ROM's refusal in words.
7. The hand-over: `$FFFF` back to 2 where it was written; the display off; VRAM and CRAM cleared; the registers to
   MK3's power-on values; RAM cleared but `$C000` and the program; the status read; the V counter awaited at `$FF`;
   the value written at `$C000` and to `$3E`; `JP $0000`.

### 3.3 The words

On its own screen, in capitals, the program's name, "ENDOU OPEN BOOT"; then "NO GAME IS IN THE CONSOLE.", or "THE
GAME CANNOT START." and one of: "IT HAS NO SEGA HEADER. IT MAY BE A JAPANESE GAME."; "IT IS FOR A JAPANESE CONSOLE.";
"IT IS NOT FOR THIS CONSOLE."; "ITS HEADER'S SIZE CODE IS NOT ONE SEGA LISTS."; "ITS CHECKSUM DOES NOT MATCH ITS
DATA." After a refusal or with no game it waits halted with interrupts off; a Pause press returns to the wait. *The
words as built, 2026-10-19; this page first gave them in a draft form.*

## 4. The machine around it

- **The bus** (`bus.rs`) gains the BIOS slot by U1–U3, and the card and expansion slots, empty. A machine has a
  **boot**: none (the cartridge from reset with `$3E` at `$AB`, the Mark III's and the SG-1000's way, and every machine
  stages 1–5 built), the open program, or the player's image. Only the Master System and the Master System II take one.
- **A console started empty** (`starts_empty`, `EmuSen_CoreAPI.md` §6.31) for `sms`: no cartridge, the region's
  console, the Americas for `auto` [C, an empty console being started to see its BIOS, which the Mark III lacks]; the
  open program shows "NO GAME IS IN THE CONSOLE." and a player's BIOS does what it does with no game.
- **The state** gains the boot and its image's CRC-32; a state of another boot is foreign, as one of another model is.

## 5. The player's own BIOS, per console model

*Decided 2026-10-19:* an opt-in accuracy upgrade per console model, the open program the default.

- **Settings** [C]: `bios_sms1` and `bios_sms2`, each "The open boot program" (`open`, the default) or "Your own BIOS"
  (`player`), `effect: none`, since the open program, not the hardware's exact path, is the default by decision, and
  the schema's `accuracy` effect requires the accurate value as the default (`EmuSen_CoreAPI.md` §6.13). The cost is
  carried by the firmware entry's `replacement` words and the setting's help.
- **Firmware** [C]: file 1, the Master System's BIOS (`bios_sms1.sms`), and file 2, the Master System II's
  (`bios_sms2.sms`), each 8,192 bytes, `required: false`, with the replacement's cost (§8); the Japanese Master
  System's takes file 3 at stage 9. Core info lists both for `sms`, and `firmware_for` answers both for a Master System
  image and none for an SG-1000 one. Machine info reports the model's file as `file` when the setting chooses it and the
  file was given, else `replacement`.
- **Checksums**: none listed at this stage. The one public description at hand (Gearsystem's `.info`) names one MD5 for
  "bios.sms (Master System BIOS)" without saying which console or version it is, and the page that would say (SMS
  Power!'s `BIOS`) is excluded for this writer. A question for the tester (§9).
- **Images other than 8 KiB** (the BIOSes with a built-in game) are refused with words at this stage; how their larger
  ROM is mapped is not in the documents read. A question for the tester (§9).
- **Tested only with a stand-in**: a Z80 program the test assembles, which behaves as a boot ROM's entry and hand-over
  does (§7). No real BIOS is read, hashed or run; the tester runs their own.

## 6. What is not built

The Japanese Master System's BIOS and its console (stage 9); the Game Gear's boot ROM (J2); `firmwarecheck.py` against
real images, which reads them and so is the tester's to run (`EmuSen_Firmware.md` §0).

## 7. The steps and their oracles

| Step | What it builds | Its oracle |
|---|---|---|
| 6.1 | `beryl-font`, Nephrite's `build.rs` on it | Nephrite's open BIOS byte-identical (SHA-256), its crate's tests and `NephriteTests` |
| 6.2 | The bus's BIOS, card and expansion slots; the boot; the open program and its `build.rs` | Bus tests of U1–U3 per model; the program's image rebuilt byte for byte; its SHA-256 recorded |
| 6.3 | The hand-over and the export check | Every Master System test program booted by the open program reaches the point it reaches with no BIOS, its results equal, with MK3's state at the hand-over (`$3E`, `$C000`, `$3F`, the VDP's registers, IM 1, interrupts off); synthetic cartridges refused in words for each reason; a Codemasters board summed through its own slot 2 |
| 6.4 | The player's image per model, the settings, the firmware entries, `starts_empty` | A stand-in boot ROM assembled by the test runs and hands over; the setting's absence, or the file's, runs the open program; machine info names the path; C4, C22 and C23 of the kit |
| 6.5 | The gate | §7 of the plan, below |

**The gate** (`Endou_Plan.md` §7, stage 6, as the tester set it): the open program boots every test program and a
sample of library games (read in place) to the same point as starting with no BIOS, with the documented state at the
hand-over; the export check refuses a Japanese cartridge on an export model in words; the player's-image path with the
stand-in; the kit; at least six seeded faults, made and reverted, any missed at first made a test; and the image
rebuilt byte for byte.

## 8. The accuracy cost, for the `replacement` words

`effect: accuracy`, its cost opening on one short sentence (`EmuSen_CoreAPI.md` §6.2, amended 2026-10-05):

"Games start without Sega's logo screen. The open boot program sets the console up as Sega's manual says the BIOS does,
checks an export cartridge's header as the documents describe, and hands the game over the same way; a game that reads
the BIOS's code or relies on what the documents leave unsaid may behave differently, the start-up takes a different
time, and the BIOS's built-in games are absent."

## 9. Questions for the tester, with recommendations

1. **The images the export check refuses** (§1.6). Under the default settings four of the library's images now meet the
   open program's export check and fail it (three Brazilian, one Korean), and would be refused by an export BIOS that
   checks as the documents say; they ran before stage 6, with no BIOS. *Recommended:* keep the documented check, the
   four running on the Mark III by the Model setting; the alternative, a check of the header's presence alone, would
   accept games an export console refuses.
2. **The BIOS checksums.** *Recommended:* at stage 7, a public list naming each image's console and version (No-Intro's
   Master System list, or SMS Power!'s `BIOS` page read by someone who writes no open boot program) fetched and cited,
   and each model's entry given its images' MD5s.
3. **BIOS images with a built-in game** (32, 128 and 256 KiB). *Recommended:* left to a later stage, with a document
   for how the console maps a BIOS larger than 8 KiB, or a screener's reading of the excluded pages.
4. **`firmwarecheck.py`** compares the open program with real images, which only the tester may run. *Recommended:* the
   tester runs it on their three images when convenient, and its numbers are recorded.

## 10. As built (2026-10-19)

The steps of §7 were built the same day; `Endou_Native.md` §13 is their record. In brief [M]: the image is 8,192
bytes (the ROM program 800 bytes from `$0000`, the font's 64 tiles at `$1000`, the RAM program's 669 bytes at `$1800`),
SHA-256 `625a92bd6aa6dd5ea523cd10a2eedef2947a0bba951d0a4cf6a0bb82f694f9f7`, the same from four builds in three target
directories; a 32 KiB check hands over in the 25th frame, a 128 KiB one in the 64th and a 256 KiB one in the 116th
(1.9 s); 29 of 31 export games from the library reach the same pictures 300 and 600 frames after the hand-over as with no
BIOS, the other two explained by the hand-over's phase and the Z80's R register; the four images of §1.6 that no range
explains are refused in words, as §9's first question says.
