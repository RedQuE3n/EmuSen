# Jadeite_OpenBoot — the Game Gear's open boot program (stage J2)

*Written 2026-10-20, before any of its code, under the plan's replacement writers' rule (`Endou_Plan.md` §1.4, §5.5),
as stage J2 of the plan's §7 asks.* This page is the plan for the Game Gear's open boot program, as
`Endou_OpenBoot.md` was for the Master System's: what the documents say a Game Gear's boot ROM does that a game can
see, what they leave unsaid and what Endou chooses there, the program and the bus it runs on, the player's own image
as an opt-in path, the steps with their oracles, the accuracy cost for the firmware's `replacement` words, and the
questions only the tester can answer. It follows the Master System's pattern exactly (decided 2026-10-19 for the
Master System, and asked for the Game Gear in J2's go of 2026-10-20): the open program is the default, the player's
image an opt-in found through the core ABI's firmware rules and never required.

Marks as in the plan: **[D]** stated by a document, cited; **[R]** read on 2026-10-20 from the source named; **[M]**
measured on 2026-10-20 with the tool named; **[A]** argued; **[C]** a choice.

---

## 0. The clean room

The program is written without any real boot ROM or BIOS being seen. No firmware image was read, disassembled,
hashed, compared or run for this page or for the program: not the tester's images in `~/Documents/BIOS/`, not any file
in the library's folders, not anything in the probe's cache. The excluded documents of `Endou_References.md` §5
stayed unopened: SMS Power!'s `BIOS` page and its `BIOSReplacement` page, and MacDonald's `smstech` §6, "BIOS
information" (lines 434–458 of the file, skipped; the lines read were 1–433 and 459 to the end). No emulator's source
was read.

**Read for this page** [R]: MacDonald's `smstech` §1 (the consoles' table and its notes), §3 (the Game Gear's port
maps in its two modes), §4 (port `$3E`, the Game Gear's paragraph among them) and §7 (the stereo register), cited
**ST**; Sega's Game Gear hardware notes, SMS Power!'s transcription, "Precautions for Hardware" (the CPU clock, the
unusable memory, "VDP initialization", "Interrupt"), "System control port" (ports `$00`–`$06`), "Using the ROM bank
switching and backup RAM" and the PSG manual's opening, cited **GG**; SMS Power!'s "Port $3E" (its section "GG BIOS
mapping"), "ROM Header", "I/O Port Map - Game Gear" and "Game Gear Stereo", cited **SP** with the page's name. None of
them lists or disassembles firmware.

## 1. What the documents say the boot ROM does

### 1.1 Which units have one

- "Some Sega Game Gears do not have a BIOS, others are reported to. All Majesco Game Gears have a BIOS." (ST §1, the
  table's note 2.)
- "The BIOS in non-Japanese Master Systems requires a valid header and later Game Gears will also not run software
  without one." (SP "ROM Header".)

The boot ROM is therefore a property of later units, not of a market: the documents read say nothing of a Japanese
unit with one, nor of an export unit without one beyond "some" (§9, question 1).

### 1.2 How it is mapped

- "Bits 7, 6, 5, and 2 have no effect on a GameGear. ... Bit 3 will enable a 1K BIOS ROM at $0000-$03FF when clear,
  and map the cartridge slot to the same space when set. When the BIOS is enabled, the rest of the address space
  ($0400-$BFFF) is mapped to the cartridge slot as well, so the BIOS can write to the cartridge mapper registers (if
  any) and read the header data as well." (ST §4.)
- "When enabled, the GG BIOS maps to $0000-$03FF; the rest of the lower 48KB maps to the cartridge as normal. When
  disabled, the first 1KB of the cartridge fills that space." (SP "Port $3E".)

So the image is **1,024 bytes**, and a program in it that writes bit 3 of port `$3E` set takes itself out of the
address space it runs from: the instruction after that write is fetched from the cartridge. The hand-over must run
from RAM.

### 1.3 What it checks

- "The export Master System and Game Gear BIOSes require [TMR SEGA] to be present to indicate valid data." (SP "ROM
  Header", "TMR SEGA".)
- The checksum "gives the ROM checksum for export SMS BIOSes. Game Gear and Japanese releases tend not to have a correct
  checksum there." (The same page, "Checksum".)
- The region code: "Only the export SMS BIOS actually checks this." (The same page, "Region code".)
- "The header can be at offset $1ff0, $3ff0 or $7ff0 in the ROM, although only the last of these seems to be used in
  known software." (The same page, "Location and size".)

Since Game Gear releases "tend not to have a correct checksum" and ran on units with a boot ROM, the boot ROM does not
check the sum [A]; it checks the signature alone.

### 1.4 What it leaves for the game

- **Port `$3E`**: "A Majesco Game Gear sets this to $A8 when running a cartridge." (ST §4.) Of `$A8`'s bits only 4 (the
  work RAM, on) and 3 (the boot ROM, off) mean anything on a Game Gear (§1.2).
- **Ports `$00`–`$06`** in GG mode: "Initial state is 'C0 7F FF 00 FF 00 FF'." (ST §3.) GG marks the same power-on
  values with its ☆ ("System control port"): port `$02` all ones, port `$05`'s bits clear, port `$06` all ones (its
  noise-right bit's mark missing in the transcription, the rest marked); port `$01` is "indeterminate" there.
- **The mapper's registers**: "The above registers are not initialized when the power is switched on. For this reason,
  be sure to initialize them program in 0 to 3FFFH. (The first 1 KByte is fixed for this purpose.)" (GG, "Using the ROM
  bank switching and backup RAM"); a game sets them itself.
- **The processor**: "IM 1 is executed at the beginning of the program" by the game, and the stack is placed below
  `$DFF0` (GG, "Interrupt", "Memory area unusable area").
- **The VDP**: "Sometimes, when the power is switched ON, the reset of the CPU is canceled while the VDP remains reset.
  In order to prevent this, confirm that the value of the V counter in the VDP has become B0H and then access the data."
  (GG, "VDP initialization.")
- **The PSG**: "The sound output goes OFF in the case of a power-on reset. Design the software so that the output goes
  OFF at the beginning of the program as well." (GG, the PSG manual's opening.)

### 1.5 What it shows

No document read describes the boot ROM's screen. `Endou_Plan.md` §5.5 calls it "a licence screen" from the plan's
reading; that reading is not repeated here, since the program shows a screen of its own with no maker's name or logo
in any case (§2, U5).

## 2. What the documents leave unsaid, and what Endou chooses

| | Unsaid | Endou's choice | Mark |
|---|---|---|---|
| U1 | Port `$3E`'s value at reset with a boot ROM | `$E3`, the value the Master System's program starts with (`Endou_OpenBoot.md` U1): bit 3 clear, the boot ROM on; bit 4 clear, the work RAM on; the other bits mean nothing on a Game Gear | [A] |
| U2 | Where the boot ROM looks for the header | `$7FF0`, `$3FF0` and `$1FF0`, as the Master System's program does; every one is cartridge space while the boot ROM is on (§1.2), and slot 1 holds bank 1 at power-on on Sega's mapper as Endou models it | [A] |
| U3 | Which mode the boot ROM runs in | Whichever the cartridge chose at power-on (D-23): the mode is a line from the cartridge, set before the first instruction. The program finds it by reading port `$00`: in GG mode bits 4–0 read 0 (ST §3, `$C0`), in a Master System cartridge's mode the port reads `$FF` (SP "I/O Port Map - Game Gear", ST §3); it writes CRAM in the mode's form, words of 12 bits or bytes of six | [A] |
| U4 | Whether the boot ROM checks a Master System cartridge | It does, alike: no document read says a Game Gear's boot ROM behaves otherwise in that mode, so a Master System cartridge without Sega's header (a Japanese one, most often) is refused by the open program. §9, question 2 | [A] |
| U5 | The screen | The program's own: its name, "ENDOU OPEN BOOT", and no logo; every line placed inside the LCD's window in GG mode (the name table's columns 6–25 and rows 3–20, `Jadeite_Native.md` §3.1), which a Master System cartridge's shrunk picture shows too; white letters on black | [C] |
| U6 | The RAM at the hand-over | All zero but `$C000`, which holds `$A8`, the value written to port `$3E`, as the Master System's program leaves its own (`Endou_OpenBoot.md` U8), and the hand-over's five bytes at `$C700`, where the Master System's program also stays | [C] |
| U7 | The VDP at the hand-over | VRAM and CRAM cleared, the registers at the Mark III manual's power-on values (`Endou_OpenBoot.md` §1.4), the 315-5378 being "a NTSC SMS2" to its software (`msvdp` §14) | [A] |
| U8 | The processor at the hand-over | Interrupts disabled, mode 1, the stack pointer `$DFF0`, A `$A8` | [C], mode 1 [D] |
| U9 | When in the frame the game starts | On the VDP's last line, the V counter read `$FF`, the status then read once, as the Master System's program does (`Endou_OpenBoot.md` U10, U11), so that a game starts at the frame's phase it starts at with no boot ROM | [C] |
| U10 | The pace of the program's VDP writes | At least 16 cycles apart with the display off and 29 with it on, the Master System's program's rule (`Endou_OpenBoot.md` U12) | [D] |
| U11 | Ports `$00`–`$06` | Not written: their power-on values (§1.4) stand at the hand-over | [C] |
| U12 | The PSG | Its four volumes written off at the start (§1.4) | [D] |
| U13 | A refusal | The words on the screen, then a halt with interrupts off; in a Master System cartridge's mode Start is the Pause button's NMI (`Jadeite_Native.md` §2.2), which returns to the halt | [C] |

## 3. The program

Z80 assembly in `src/boot/gg.s` of Endou's crate, assembled by `build.rs` with `beryl-z80`'s assembler into a 1 KiB
image in `OUT_DIR` (`open_boot_gg.bin`), beside the Master System's two. 1 KiB holds no font of 64 Master System tiles
(2 KiB), so the image carries the letters it uses as one bit a dot, five dots a row and seven rows a glyph, from
`beryl-font`, and the program writes each row into the tile's first bitplane, the other three clear.

1. `DI`, `IM 1`, the stack at `$DFF0`; `RETN` at `$0066` and `RETI` at `$0038`.
2. The V counter awaited at `$B0` (§1.4); the PSG's four volumes off.
3. The mode from port `$00` (U3), kept in a register pair's spare bit.
4. The VDP: display off, the Mark III manual's table addresses, the name table and sprite table cleared, the sprite list
   ended at `$3F00`, the glyphs written as tiles, CRAM's first two colours black and white in the mode's form, the
   program's name; the display on.
5. The header: `TMR SEGA` at `$7FF0`, `$3FF0` or `$1FF0` (U2). None: the refusal's words, and the halt (U13).
6. The hand-over: the display off; VRAM and CRAM cleared; the registers to the power-on values; RAM cleared; `$A8` at
   `$C000`; `OUT ($3E),A` and `JP $0000` at `$C700`; the V counter awaited at `$FF`; the status read; `JP $C700`.

**The words** (U5), each line at most 20 letters, the window's width: "ENDOU OPEN BOOT"; after a refusal, "THE GAME
CANNOT START.", "IT HAS NO SEGA", "HEADER."

## 4. The machine around it

- **The bus** gains the Game Gear's boot ROM slot by §1.2: with bit 3 of port `$3E` clear, `$0000`–`$03FF` reads the
  boot ROM and the cartridge answers everywhere else; with it set, the cartridge everywhere. Bits 7, 6, 5 and 2 of the
  port have no effect on a Game Gear, which stage J1 did not yet model (`Jadeite_Native.md` §9).
- **A machine's boot** on the Game Gear: the open program by default, the player's image when chosen and given, or none
  through `Machine::with_board`, the form every test of stages J1 and earlier used.
- **The state** carries the boot and its image's CRC-32 already (`Endou_OpenBoot.md` §4).

## 5. The player's own boot ROM

- **Setting** [C]: `bios_gg`, "Game Gear boot ROM": "Endou's open boot program" (`open`, the default) or "Your own boot
  ROM" (`player`), `effect: none`, for the reason `Endou_OpenBoot.md` §5 gives; one setting for both Game Gear models,
  since the boot ROM is a unit's, not a market's (§1.1).
- **Firmware** [C]: file 4, `bios.gg` (the name Genesis Plus GX's `.info` gives the image [R `Endou_Plan.md` §1.7]),
  1,024 bytes, `required: false`, with the replacement's cost (§8), listed for the `gg` system. Machine info reports
  file 4 as `file` when the setting chooses it and it was given, else `replacement`.
- **Checksums**: none listed at this stage; the public DAT at hand (libretro-database's copy of No-Intro's Game Gear
  DAT) holds no firmware rows, as its Master System copy held none (`Endou_References.md` §6f). §9, question 3.
- **Tested only with a stand-in**: a 1 KiB Z80 program the test assembles, which hands over from RAM as §1.2 requires.
  No real boot ROM is read, hashed or run; the tester runs their own.

## 6. What is not built

`firmwarecheck.py` against real images, which reads them and so is the tester's to run (`EmuSen_Firmware.md` §0); its
control is named there. The Game Gear to players is J3's.

## 7. The steps and their oracles

| Step | What it builds | Its oracle |
|---|---|---|
| 1 | The bus's boot ROM slot and port `$3E`'s bits on the Game Gear | Bus tests of §1.2 |
| 2 | `gg.s` and its `build.rs` | The image rebuilt byte for byte; its SHA-256 recorded; 1,024 bytes |
| 3 | The screen, the check, the hand-over | Each Game Gear test program booted by the open program reaches the point it reaches with no boot ROM, with U6–U9's state at the hand-over; a cartridge without the signature refused in words, in both modes |
| 4 | The setting, the firmware entry, the player's image | A stand-in assembled by the test runs and hands over; the setting's absence, or the file's, runs the open program; machine info names the path |
| 5 | G9 for the Game Gear | Every Game Gear game of the tester's library, with no firmware folder, reaches the pictures it reaches with no boot ROM, aligned at the hand-over; with the stand-in, the same |

## 8. The accuracy cost, for the `replacement` words

`effect: accuracy`, its cost opening on one short sentence (`EmuSen_CoreAPI.md` §6.2, amended 2026-10-05):

"Games start without the boot ROM's screen. The open boot program checks for Sega's header as the documents say the
Game Gear's boot ROM does and hands the game over the same way; a game that reads the boot ROM's code or relies on what
the documents leave unsaid may behave differently, and the start-up takes a different time."

## 9. Questions for the tester, with recommendations

1. **Units without a boot ROM.** Earlier Game Gears had none (§1.1); the open program runs by default, as on a later
   unit. *Recommended:* a third choice of `bios_gg`, "No boot ROM (earlier units)", the cartridge from reset, which is
   exact for those units; not built at J2, since the go asked for the Master System's pattern exactly. **[decision]**
2. **Master System cartridges without a header** (U4) are refused by the open program in the Game Gear's Master System
   mode. *Recommended:* keep the documented check; with question 1's choice, a player can still run such a cartridge
   as an earlier unit would. **[decision]**
3. **The boot ROM's MD5.** *Recommended:* at J3, No-Intro's Game Gear DAT fetched from DAT-o-MATIC with its firmware
   rows, as stage 7 fetched the Master System's, and the entry given its image's MD5. **[routine]**
4. **`firmwarecheck.py`** against the player's images, with the control named in `EmuSen_Firmware.md` §0. *Recommended:*
   the tester runs it when convenient, and its numbers are recorded here. **[routine]**
