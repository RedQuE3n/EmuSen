# Nephrite_OpenBios — the open Sega CD BIOS (stage 10)

*Written 2026-10-15, before any of stage 10's code, by a writer under the plan's replacement writers' rule
(`Nephrite_Plan.md` §1.3, §5.6).* This page is stage 10's plan: what the open BIOS must provide, what Sega's manuals
and the public documents say of it, what they leave unsaid, the steps of `Nephrite_Native.md` §49.5 with each one's
sources and oracle, the accuracy cost for the firmware's `replacement` words, and the questions only the tester can
answer. **Its first finding stops the stage at the plan** (§2.2): the manuals name every call and say what it does,
but neither the function numbers nor the sub CPU's entry addresses nor the backup RAM's format are in any document
the writer may read, and the 68000 assembler the plan names does not exist. §5's questions decide how the stage goes
on.

The page keeps the plan's registers apart: *documented* means stated by a document, cited by manual and section;
*argued* means reasoned from the documents with no test behind it; nothing here is yet *measured*.

---

## 1. What the BIOS is, by the manuals

The plan's §5.6 names five parts. The manuals describe them as follows (Sega, *Mega-CD BIOS Manual*, version 2.00,
1992-02-24, cited as **BM**; *Mega-CD Disc Format Specifications*, version 2.00, 1992-02-24, **DF**; the *Sega-CD
Technical Bulletins* collection, **TB**, cited by bulletin and its own page number; *Mega-CD Hardware Manual*, **HM**;
plutiedev's "Using the Sub-CPU in mode 1", **PD**).

### 1.1 The boot

- **Mode 2, from a disc** (BM §4-2). The main CPU boots from the boot ROM, initialises the Genesis, transfers the CD
  system program and the CD-boot program to the sub CPU's side, releases the sub CPU's reset, waits for the initial
  program (IP), runs it once it is transferred, waits for the application and runs it. The sub CPU initialises the
  system program and the boot program, begins interrupt processing, loads the IP and passes it to the main side, runs
  the system program (SP) once loaded, loads the application and passes the main CPU's part to it.
- **Mode 1, from a cartridge** (BM §4-1): the cartridge's own program transfers the CD system program and its sub CPU
  application, releases the reset and waits for the sub side; the sub CPU initialises the system program and runs the
  application. A cartridge finds the Sega CD by "SEGA" at `$400100` or bit 5 of `$A10001` clear (PD, "Check if a Mega
  CD is present"), which is how mcd-verificator finds it (`Nephrite_Native.md` §49.1).
- **The disc's system area** (DF §3-2, §3-3, appendices 2 and 3; TB 1992-06-16 "The U.S. BIOS"): sector 0 holds the
  system ID at `$000` (the identifier `SEGADISCSYSTEM` for a bootable disc, `SEGADATADISC` for a data disc, the volume
  and system names and versions; at `$030` the IP's disc address, load size, entry offset and work RAM size, at `$040`
  the same four for the SP), the disc ID at `$100` (hardware, company, titles, product code, I/O support, the region
  letters at `$1F0`), and the IP from `$200`. An IP larger than `$600` bytes gives as its address the first byte after
  the first `$600` and as its size the rest (TB "The U.S. BIOS"; TB "Creating Boot ID", p. 14). The SP follows from
  sector 1 or 2.
- **Where they go.** The IP runs on the main CPU from the start of work RAM, `$FF0000`, with the stack at `$FFFD00`
  (TB "The status after booting is completed", p. 15; the memory map, p. 12). The SP is loaded into PRG-RAM at
  `$006000` (BM §1-4; TB p. 16) and carries a user header (BM §4-3): an 11-character module name beginning "MAIN",
  a flag, version and type, a link pointer, the module's size, the offset of a table of four 16-bit offsets of
  `_usercall0` to `_usercall3` ended by zero, and its work RAM size.
- **The boot's order across the two CPUs** (TB "Boot diagram", p. 13): the main CPU resets the gate array, transfers
  the sub CPU's program and starts it; both show the logo; the sub side checks the disc, loads the IP into Word RAM,
  the main side copies it to work RAM and starts it; the sub side loads the SP into PRG-RAM; the application is loaded
  through the communication registers by the game's own programs. While the IP runs, the SP's load needs the main CPU
  to raise the sub CPU's level 2 every 16 ms (TB p. 18).

### 1.2 The sub CPU's call interface

- **Three entries**, `_cdbios`, `_buram` and `_cdboot` (BM §3, §6-1, §7-1), each called by `jsr` with the function
  number in `d0.w`, a parameter table's address in `a0.l` and other parameters in registers; `d0`, `d1`, `a0` and `a1`
  are not preserved; a result is in those registers or the carry (BM §2-1 "Call Method"). The entries and numbers are
  "provided by an include file" (BM p. 6; `CDBIOS.I`, BM §3 p. 11; `cabios.i`, BM §5-1).
- **The calls**, with their parameters and results: the drive (DRVINIT, DRVOPEN; BM §3-D), CD-DA (MSCSTOP, MSCPLAY,
  MSCPLAY1, MSCPLAYR, MSCPLAYT, MSCSEEK, MSCSEEK1, MSCSEEKT, MSCPAUSEON, MSCPAUSEOFF, MSCSCANFF, MSCSCANFR, MSCSCANOFF;
  §3-M), CD-ROM (ROMREAD, ROMREADN, ROMREADE, ROMSEEK, ROMPAUSEON, ROMPAUSEOFF; §3-R), the BIOS (CDBCHK, CDBSTAT with
  its 32-byte status table and the meaning of every bit of `bios_status`, CDBTOCREAD, CDBTOCWRITE, CDBPAUSE; §3-B), the
  fader (FDRSET, FDRCHG; §3-F), the CDC (CDCSTART, CDCSTOP, CDCSTAT, CDCREAD, CDCTRN, CDCACK; §3-C), subcode (SCDINIT,
  SCDSTART, SCDSTOP, SCDSTAT, SCDREAD, SCDPQ, SCDPQL; §3-S), the LEDs (LEDSET with its eight modes; §3-L), the boot
  (CBTINIT, CBTINT, CBTOPENDISC, CBTOPENSTAT, CBTCHKDISC, CBTCHKSTAT with its disc types; §6-1).
- **Their timing** (BM §2-1 "Caution"): the BIOS, LED, fader, subcode and CDC calls act at once; the drive, CD-DA and
  CD-ROM calls are taken by the level 4 interrupt every 13.3 ms; after initialisation levels 2 and 4 are enabled, and
  levels 5 and 6 by the CD-ROM calls; the drive's states and the calls that move between them are BM §2-4's chart.
  The loader BM gives as its sample (p. 44–45) shows the data path a game uses: ROMREADN, then CDCSTAT, CDCREAD with
  the destination set, CDCTRN into a buffer with its header, CDCACK, sector by sector.
- **The sub CPU's memory** (BM §1-4): vectors at `$000000`, the CD-BIOS identifier at `$000100`, the system program
  from `$000200` (the three write-protected), work from `$005400`, the heap and stack from `$005C00`, common work from
  `$005E80`, **the jump table from `$005EE0`**, the user header at `$006000`.
- **The user calls and exceptions** (BM §5-3, §5-4; TB p. 17–18): `_usercall0` (initialisation, before level 2 is
  enabled), `_usercall1` (the main routine, called with `_usermode` in `d0.w` and again after each level 2), `_usercall2`
  (level 2, after the system's own work), `_usercall3` (the user's, never called by the system); each jump table entry
  is six bytes, a `jmp`, `rts` or `rte`; the exception vectors in the protected area point at the jump table, whose
  `_level2`, `_level4`, `_level5` and `_level6` belong to the system.

### 1.3 The backup RAM interface

- **The calls** (BM §7-1, with TB #1's correction that BRMWRITE also takes `d1.l` zero): BRMINIT (a scratch area of
  `$640` bytes and a 12-byte display string buffer; carry clear with the size when a Sega-formatted RAM is present,
  carry set with `d1.w` 0 for no RAM, 1 unformatted, 2 another format), BRMSTAT (free blocks and the number of files),
  BRMSERCH, BRMREAD, BRMWRITE (an 11-character name of `0`–`9`, `A`–`Z` and `_`, a flag `$00` for normal or `$FF` for
  the protect function, a size in blocks of `$40` bytes or, protected, `$20`), BRMDEL, BRMFORMAT, BRMDIR (a template
  with `*`, files to skip and a buffer size) and BRMVERIFY.
- **The cartridge** is reached through a second entry, `_mburam`, with the same calls (TB #1, "How to access Back-Up
  Ram Cartridge"); its address is in `MEM_MAP.ASS`, which is not in the corpus.
- **What is said of the format**: of the total, `$40` bytes each go to "the last ID area", the directory area and "the
  leading reserve area" (TB "Backup RAM", p. 9); BRMDEL compresses the directory and the data again (same page); a
  file of 256 blocks or more is not deleted correctly (TB #2, item 2). Nothing more.

### 1.4 The main CPU's routines and tables

- **The main jump table** at `$FFFD00` (TB p. 15–16): six-byte entries `_reset`, `_mlevel6` (V), `_mlevel4` (H),
  `_mlevel2` (external), `_mtrap00`–`_mtrap15`, then the error entries (`_monkerr`, `_madrerr`, `_mcoderr`, `_mdiverr`,
  `_mtrperr`, `_mnocod0`, `_mnocod1`, `_mspverr`, `_mtrace`) and `_vint_ex`, then the VDP's registers 0 to 18 as words
  from `$FFFDB4` to `$FFFDD9`. A game sets an entry's address at the label plus 2 (the bulletin's sample stores the
  V-INT handler at `_mlevel6+2`). Work RAM above `$FFFD00` is the system's.
- **Routines** "in the boot ROM may also be used (please see mainent.i, rom_util.doc)" (TB p. 15), and `_CDPL_JMP`
  and `CDPL_RESET` return to the control screen (TB #2 and p. 9); none of those files is in the corpus.

### 1.5 The screen with no disc

The plan asks only for a simple open screen. The documents name what the real one does (the logo, the control screen,
"Press START"; Software Standards §I, §V), which an open screen need not copy.

---

## 2. What the documents give, and what they do not

### 2.1 Given

Every call's name, parameters, results and purpose; the status table's layout; the drive's state chart; the boot's
steps on both sides; the disc's system area to the byte; the sub CPU's memory map with the jump table's place; the
user header; the main jump table's entries, their order and addresses; the IP's and SP's places; the backup RAM's
calls. Enough for the boot, the drive and data driver's behaviour, and the main side's tables.

### 2.2 Not given (the gaps)

| | What | Where the manuals point instead | Needed by |
|---|---|---|---|
| G1 | **The function numbers** a game puts in `d0.w`, for all 57 calls (42 of `_cdbios`, 6 of `_cdboot`, 9 of `_buram`) | `CDBIOS.I` (BM p. 6, p. 11) | 10.2–10.4: every call |
| G2 | **The sub CPU's jump table**: which six-byte slot of `$005EE0`–`$005FFF` is `_cdbios`, `_buram`, `_cdboot`, `_usercall0`–`3`, `_level1`–`7`, the traps and errors; `_usermode`'s and `_cdstat`'s addresses | `cabios.i` (BM §5-1) | 10.1: the SP's header and its first call |
| G3 | **The backup RAM's format**: where the identifier, the directory and the free count stand, a directory entry's layout, how a protected file is encoded | none; TB p. 9 gives three `$40` areas | 10.4, and every save moved between the two BIOSes |
| G4 | **The main CPU's routines** a game calls by address, and `_mburam`'s address | `mainent.i`, `rom_util.doc`, `MEM_MAP.ASS` (TB p. 15, TB #1) | 10.5 |
| G5 | **The IP's security block**: what the boot ROM checks at the head of the IP before it runs it | Sega's data files `SECDB_*` (TB "The U.S. BIOS", TB "PAL conversion" §IV) | 10.1: whether to check |
| G6 | **The Beryl 68000 assembler** the plan's §5.6 names for the BIOS's source | the plan's stage 1 listed it as a development tool; `beryl-m68k` has a disassembler and no assembler | 10.1 onward |

G1 and G2 are the interface itself: a game's SP calls `jsr` to an address in the jump table with a number in `d0`, so
an open BIOS whose numbers or slots differ from the real one's runs no game. They are facts of the interface, not of
the BIOS's code, but no document the writer may read states them. G3 decides whether a save made under either BIOS
is readable under the other (§5.6, part 3), and the format of a new machine's memory (decided 2026-10-15 to wait for
this stage). G5 is a choice, not a fact the open BIOS needs (§4). G6 is work, not a question of sources.

**Searched for G1–G4, 2026-10-15:** the BIOS manual's 51 pages, read from their page images; the Software
Development Manual, the Disc Format Specifications, the Software Standards and the Technical Bulletins through their
text layers, with the bulletins' boot and memory pages read from their images; plutiedev's pages; railgun's wikitext;
every file of the corpus's `docs/` by name and text search, the excluded and unscreened files skipped by name. None
gives a function number, a sub-side slot or the format's layout.

---

## 3. The steps, as planned once §5's questions are answered

Each step is committed with its tests and its record in `Nephrite_Native.md` when its oracle is met.

| Step | What it builds | Sources | Its oracle |
|---|---|---|---|
| 10.0 | The 68000 assembler in `beryl-m68k` (a development tool behind a feature, as `emusen-mips`'s is): every instruction and addressing mode, labels, `dc`/`ds`/`even`/`org`/`equ`, expressions, branch sizing | Motorola's M68000PM §2–§4 and appendix tables | Every instruction form the single-step suites name assembled and disassembled back to the same words by `disasm.rs`; a build of the BIOS reproducible byte for byte |
| 10.1 | The BIOS as a 128 KiB image built at compile time from `src/mcd/bios/*.s`: the main CPU's vectors, the Genesis's initialisation, the gate array's reset (PD), the sub CPU's program copied to PRG-RAM and started, the main jump table at `$FFFD00` (TB p. 15), the sub CPU's vectors through its jump table (BM §5-4), its level 2 and 4 dispatch, the disc's system area checked and the IP loaded to `$FF0000` and started, the SP loaded to `$6000` and its user calls placed (BM §4-2, §4-3, §5-3; DF appendix 2), the screen with no disc | BM §1-4, §2-1, §4, §5; DF §3-2, §3-3, app. 2–3; TB p. 12–18; HM §3–§4 for the registers | The 240p suite's IP and SP running on the open BIOS to its first own picture; a disc's absence giving the open screen; the five discs' IPs starting |
| 10.2 | `_cdbios`'s drive, CD-DA, status, TOC, pause, fader and LED calls over the drive of §51, taken at level 4 as BM §2-1 says, the status table kept as BM p. 20–21 lays it out | BM §2, §3-D, §3-M, §3-B, §3-F, §3-L; HM §3-6 | The 240p suite's CD-DA tests and its menus' track playback; each call's effect on the drive's command stream in a crate test |
| 10.3 | ROMREAD, ROMREADN, ROMREADE, ROMSEEK, the ROM pauses, the CDC calls and subcode calls over §52's CDC | BM §3-R, §3-C, §3-S, the load sample p. 44–45; Sanyo's LC8951 manual | The 240p suite loading its files and PCM data; the five discs' data loads (their pictures after the hand-over) |
| 10.4 | `_buram` and `_mburam` in the real format, on the internal RAM and the cartridge; the format module (`ISaveMediumFormat`) as pure functions in the Genesis system pack | BM §7-1; TB p. 9, #1, #2; G3's source | Round trips through a game in both directions (save under one BIOS, load under the other) |
| 10.5 | The main CPU's routines and tables games use by address, each found by a game reaching it on the open BIOS and written from a document that describes it | TB p. 15; G4's source | The five discs to their games after the hand-over |
| 10.6 | The gate: the 240p suite; mcd-verificator in mode 1 and unchanged in mode 2; the five discs against the player's BIOS as black boxes; `_BURAM` round trips; the replacement made the default without the player's image | — | As §49.5 states |

Mode 1 (`Nephrite_Native.md` §49.6 Q4 and §62.4): a Create setting for cartridges, off by default, giving
`Genesis<MegaCd>` with the cartridge in its slot, the open BIOS at `$400000`, reported as an `md` machine with the
cartridge's `.srm` file 0 and the backup RAM's `.brm` file 11; it can be built at 10.1, needing none of G1–G4, since
mcd-verificator's mode 1 uses only the boot ROM's header and the hardware.

---

## 4. The accuracy cost, for the firmware's `replacement` words

*Argued, to be revised by the gate.* With the open BIOS present the firmware is `required: false` with a replacement
of `effect: accuracy`, in words like these:

"Nephrite's own Sega CD BIOS, written from Sega's manuals: the documented calls answer as the manuals describe, saves
use the console's own backup RAM format, and discs boot. A game that calls a routine no manual names, reads the BIOS's
code, or depends on the exact timing of its loops may run differently; discs of every region start, since the
security check is not made; the screen without a disc and the boot logo are Nephrite's own, with no CD player or
memory manager."

G5's choice is in those words: the real boot ROM refuses an IP without its region's security block; the open one
cannot test for Sega's bytes without carrying them, and carries none of Sega's code, so it starts any disc whose
sector 0 begins `SEGADISCSYSTEM`. The block itself is the game's own code and runs as part of its IP.

---

## 5. Questions for the tester, with recommendations

*Decided 2026-10-15: all five as recommended.* Each decision follows its question. The screening Q1 and Q2 call for is
made by a screener who neither writes nor reviews the BIOS; until its verdicts, the include file and the format's
public description stay unopened by the writer.

1. **Q1, where G1 and G2 come from.** The interface's numbers and slots are in no document the writer may read. Three
   sources are possible:
   - **(a)** A homebrew program's interface header: the 240p Test Suite's Sega CD build carries one
     (`segacdfiles/LukeProjectCD/_boot/_inc/cdbios.asm`, GPL-2.0, in the corpus's test-program sources). Test
     programs' sources are open to a writer by §1.3's table, but a header of BIOS constants may have been made from a
     listing. **It was not opened**: the writer's attempt to look at its length and first lines was refused before
     anything was shown. By §1.3's screening rule it should be screened by someone who does not write the
     replacement, who either clears it, in which case the writer reads only its constants, or adds it to `DOCS.txt`'s
     exclusions.
   - **(b)** From games, on the open BIOS: every jump into `$005EE0`–`$005FFF` trapped, with `d0` and the parameter
     table's shape recorded, each number matched to the one call in BM whose parameters fit (DRVINIT's two bytes
     `$01,$FF`, ROMREADN's two longs, the load sample's sequence). Clean of the BIOS, but slower, and a call no game on
     hand makes stays unknown.
   - **(c)** A public document listing them, fetched and screened like the corpus's others. None was found in the
     corpus.

   *Recommended:* (a) after screening, with (b) as its check: each constant used only once a game's call on the open
   BIOS confirms it, the confirmations recorded as disputes D-41 onward. Without an answer stage 10 cannot run a disc.
   *Decided 2026-10-15:* the numbers and slots come from the 240p suite's include file only once a screener has
   cleared it; each constant, once used, is confirmed by a game's call on the open BIOS and recorded as a dispute from
   D-41.

2. **Q2, where G3 comes from, and how a new machine's backup RAM is formatted.** No document gives the format beyond
   TB p. 9's three `$40` areas, and the replacement writers' rule (§1.3, §5.6) forbids reading what the player's BIOS writes. *Recommended:* a public
   description of the format (save-file converters and RAM cartridge tools publish them; to be fetched and screened as
   in Q1), the open BIOS writing it, graded only by round trips through a game. Once the format module exists and the
   round trips pass, *recommended* that a new machine's internal backup RAM be supplied formatted (both references
   start formatted, `Nephrite_Native.md` §60.5, and the open BIOS has no memory manager in which a player could format
   it), made by the format module's pure `format` function, so that the console's memory becomes Shared by default as
   decided.
   *Decided 2026-10-15:* the format comes from a screened public description and is graded only by round trips
   through games; once those pass, a new machine is given an internal backup RAM formatted by the format module, and
   the Sega CD's console memory becomes Shared by default.
   *Decided 2026-10-15, further:* the screened description is `docs/bram/emutos-mcd_bram-filesystem.md`, for the
   footer (the ID area, the free-block and file counts) and the odd-byte layout, with SpritesMind topic 826 for the
   cartridge's memory map, ID register and write protection; the rest of the format (directory entries, file names,
   allocation, the protected encoding) comes by a two-person method: an observer who does not write the BIOS runs games
   on the player's BIOS as a black box, examines only the backup RAM images the games leave, and writes the format in
   plain words; `_BURAM` is built from that specification alone.
   *Decided 2026-10-15 by the tester:* images built from the format already observed may be given to the player's BIOS as inputs, its handling of them being black-box evidence like the images games leave; the BIOS's code is still never read or watched.

3. **Q3, the security check (G5).** *Recommended:* none; every disc with `SEGADISCSYSTEM` boots, which §4's words
   state. The alternative, checking the disc ID's region letters against the console's, would refuse discs the real
   BIOS also refuses, but by a different test, and is not worth the difference.
   *Decided 2026-10-15:* no security check; any disc with `SEGADISCSYSTEM` in sector 0 boots, and the replacement's
   words say so.

4. **Q4, the main CPU's routines (G4).** *Recommended:* found game by game on the open BIOS (a jump into
   `$000000`–`$01FFFF` that no table explains), each written only from a document that describes it and recorded as a
   dispute; a routine no document describes stays unwritten, the game's failure recorded, which P7 predicted.
   *Decided 2026-10-15:* found game by game, written only where a document describes them, otherwise recorded as
   disputes.

5. **Q5, the assembler (G6).** *Recommended:* build it in `beryl-m68k` as step 10.0, for this stage and for the 32X's
   68000 boot ROM, graded by assembling and disassembling every instruction form back to the same words.
   *Decided 2026-10-15:* built now as step 10.0, a development tool of the crate as `Beryl-HW/README.md` describes,
   graded by assembling and disassembling every instruction form back to the same words and against the single-step
   suite's opcode coverage.

---

## 7. Step 10.0: the assembler (2026-10-15)

Built in `beryl-m68k` behind its `asm` feature, as Q5 decided: `Beryl_M68k.md` §8 is its record. Every opcode the
68000 decodes, 45,816 of them, assembles back from the disassembler's text to the words it came from; a branch or an
absolute address with no size is the one choice it makes for the writer (§8.2 there). One change to the disassembler
came with it: the PC-relative indexed mode now prints its target, as the PC-relative mode does.

## 8. The boot, the drive and the data reads, first proof (2026-10-15)

Built with Q1's constants (D-41) in `src/mcd/bios/` (`main.s`, `sub.s`, `font.txt`), assembled by `build.rs` into a
128 KiB boot ROM (`mcd::bios::OPEN_BIOS`): the main CPU's program from `$2000`, the font at `$E000`, the sub CPU's
program at `$10000`, copied to PRG-RAM.

- **The main CPU** (BM §4-2; TB p. 12-18): TMSS answered where the version register asks it; the Z80 held and the PSG
  silenced; the VDP set up with the BIOS's own registers kept at `$FFFDB4`; the gate array reset (HM §4-1), the sub
  CPU held, its program copied through bank 0, `$0000`-`$53FF` write-protected, Word RAM given to it, and the sub CPU
  released; the jump table at `$FFFD00` with `_reset`, the levels, the traps and the errors; H-INT through
  `_mlevel4`; a V-INT that raises the sub CPU's level 2 each frame (TB p. 18). Its own screen shows the drive's state
  ("NO DISC", "THE TRAY IS OPEN", "READING THE DISC", "AN AUDIO CD", "NOT A SEGA CD GAME DISC") in its own font, and
  START opens or closes the tray. When the sub CPU reports the initial program in Word RAM, it is copied to `$FF0000`
  and started with the stack at `$FFFD00`. `$0200`-`$1FFF` hold an RTS in each word but for `$0368` (D-42, D-43).
- **The sub CPU** (BM §1-4, §2-1, §2-4, §3, §4-3, §5): the vectors through the jump table at `$5EE0`-`$5FFF`; levels 2,
  4 and 5 enabled; the drive clocked and its TOC read through its reports (the disc's length, its tracks, each track's
  start); the system area read, `SEGADISCSYSTEM` checked (Q3: no security check), the initial program put in Word RAM
  from the system area and, beyond `$600` bytes, from its disc address; the system program read to `$6000` and its user
  calls placed from its header; `_USERCALL0` with level 2 masked, then `_USERCALL1` with `_USERMODE` after each level 2.
- **`_CDBIOS`**: every call of BM §3 dispatched by its number; the drive's, CD-DA's and CD-ROM's posted for the level 4
  exchange (CDBCHK busy until taken), the others at once. The drive's course: play, seek, pause and resume, scan,
  stop, the tray; MSCPLAY1's pause at the next track, MSCPLAYR's repeat, MSCSEEK1's play after its pause, the lead-out's
  stop and CDBPAUSE's stand-by. A read seeks two sectors short, turns the decoder on once the drive seeks, queues each
  wanted sector's block (five at most, the buffer's writes held when full and the sector asked for again), and
  ROMREADN's last sector pauses the drive. CDCREAD hands the head of the queue to the destination the caller set (2,048
  bytes, or 2,336 in CDCSETMODE's mode 2), CDCTRN reads it through the host data register, CDCACK drops it (D-44).
  The status table at `$5E80` as BM p. 20-21 lays it out; the fader's set and ramp; the LEDs by the drive's state or
  LEDSET. Subcode reads answer "no data", and `_BURAM` finds no RAM until step 10.4.
- **Found on the way**: the decoder raises its interrupt with the last header while the head travels in a seek, which a
  read took for a sector far past its target and sought again without end; a header that repeats the last is now no
  sector (argued: a real decoder in a seek reports blocks that fail their check, which Nephrite's does not model). And
  a BCD conversion that left a word of the caller's in the result's high half, which put every sector beyond reach;
  both shown by the drive seeking without end and fixed.

**Measured (2026-10-15).** The 240p Test Suite (Sega CD 1.32, the American disc) boots on the open BIOS in Nephrite:
the BIOS's screen reads "READING THE DISC" at frame 120, the initial program starts at frame 156, and at frame 1,200 the
suite's main menu is on screen ("Test Patterns", "Video tests", "Audio tests", "Hardware tools", "DRV VER: 0x00"),
its V-INT installed through `$0368`. `the_open_bios_boots_the_240p_suite_to_its_menu` holds it. The crate's 180 tests
pass, mcd-verificator's eighteen in mode 2 among them, unchanged.

### 8.1 The five discs, first runs (2026-10-15)

Each of the five discs, run on the open BIOS for 2,400 frames with no pad pressed, after two rules of the hand-over that
Sonic CD's program needs (D-45: interrupts open and every communication word 0; D-46: the Z80 running, its bus free):

| Disc | What the game shows by frame 2,400 |
|---|---|
| Chuck Rock II (USA) | its own pictures: a prehistoric landscape with palms and rocks, its intro under way |
| Lunar - Eternal Blue (USA) | its own message, "Backup RAM is not formatted! Please consult your system manual for details on how to format RAM before playing. Press any key" |
| Mickey Mania (USA) | a black screen at frames 1,200 and 2,400, its main program running |
| Sonic CD (USA, Europe) | its own message over Dr. Robotnik: "Internal RAM is not initialized. Use the SEGA-CD control panel to format the RAM." |

The two messages are the games' answers to `_BURAM`, which finds no RAM until step 10.4 (§5 Q2). Not yet compared with
the player's BIOS.

## 10. Step 10.4, the part the footer gives (2026-10-15)

`docs/bram/emutos-mcd_bram-filesystem.md` (allowed by the tester's decision of §5 Q2) gives the volume's footer, the
last `$40` bytes of the RAM as its bytes are counted on the odd addresses: eleven `$5F`, four `$00`, `$40`; the free
blocks as a big-endian word four times; the files the same way; `SEGA_CD_ROM`, `$00 $01 $00 $00 $00`;
`RAM_CARTRIDGE___`. A volume is Sega's when its last `$20` bytes match; blocks are `$40` bytes and an empty 8 KiB volume
has 125 free, the footer's three blocks counted out (the Technical Bulletins' "last ID area, directory area, leading
reserve area" of p. 9 are three such blocks); BRMINIT gives the size in `$1000` bytes; every byte before the footer of
a blank volume is 0. The document states its own origin plainly: the footer's layout was first taken from an
emulator's formatting function and then confirmed against a console's volume; it holds no code.

Built on the internal RAM: **BRMINIT** (carry clear and `d0.w` 2 for a Sega volume; else carry set, `d1.w` 2 where the
last line is `RAM_CARTRIDGE___` and the identifier differs, 1 otherwise; argued, BM §7-1 naming "other format" without
its test), **BRMSTAT** (the two counts), **BRMFORMAT** (zeros, then the footer with 125 free and no file). The
directory's calls (BRMSERCH, BRMREAD, BRMWRITE, BRMDEL, BRMDIR, BRMVERIFY) wait for §5 Q2's specification and fail;
the cartridge's `_mburam` waits for its address.

**Measured (2026-10-15)**, a blank volume made by the footer's rules given as the backup RAM: Sonic CD goes past its
"not initialized" message to "Internal RAM is full. Game will start without save option." (its write refused), and
Lunar to a message that its save data was destroyed (its search finding nothing a game can read), each the game's
answer to a directory call that fails. A new machine's RAM stays unformatted until the round trips pass (§5 Q2).

*Since 2026-10-16* the directory's calls are built from the observer's specification (§14); BRMFORMAT now writes the
footer alone, as the specification observed (`Nephrite_Bram.md` §4), and a new machine's RAM is formatted (§14.4).

## 11. Mode 1, the system pack's discs, and a conversion's high half (2026-10-15)

- **Mode 1** (the plan's Q4 of `Nephrite_Native.md` §49.6, `Nephrite_Attachments.md` §3.12 and Q8): the Create setting
  `mcd_beneath` (hidden while the Sega CD is offered to no player, "none" by default) gives a cartridge's machine the
  Sega CD beneath it: `Genesis<MegaCd>` with the cartridge in its slot, the Sega CD decoding `$400000`-`$7FFFFF` as
  mode 2's `$000000`-`$3FFFFF` and its registers, and declining the cartridge's area. The boot ROM is the player's
  BIOS of the console's market where given (files 2-4), Nephrite's own else; machine info names it, as `Replacement`
  without the player's. The machine stays of the `md` system; the cartridge's save RAM keeps file 0 and the backup RAM
  is file 11 (`.brm`, kind `console`, medium `mcd.backup`), which core info now lists for `md`. Its state carries a
  system byte of its own, 3, so that a plain Genesis refuses it and every other state keeps its bytes; this stands in
  for Q5's "byte of the attachments present", which would have changed every cartridge's state. The drive starts empty;
  a host puts a disc in it through `MEDIA`. The H-INT vector reads through the boot ROM's `$70` in mode 1 too (D-47).
- **The system pack** (`GenesisSystems.MegaCd`) claims `.cue` and `.chd` beside `.iso`, as does the core: a sheet is
  read by its first track, where `SEGADISCSYSTEM` tells a Sega CD's from a PlayStation's (whose pack also takes
  `.cue`), and a CHD, which neither pack can read by its first bytes, by its folder's name, as routing already does.
- **Found on the way**: the BCD conversion kept only the low half of a register its caller used for the sector being
  summed, so every time past 14:33 lost its high half; Sonic CD's title asked for track 26 and the drive played its data
  track. Shown by the drive's place and the title's silence, fixed, and the title's music then plays.

**Measured (2026-10-15).** mcd-verificator V1.02 as a cartridge with the Sega CD beneath it and the open BIOS at
`$400000`, the 240p disc in the drive: "CD hardware detected at 0x00400000", and RAM CART ("not present"), COLOR CALC,
VAR TESTS, IRQ TEST, REG X000, REG X002, REG 2006, REG X00C, REG 8030, CDC REGS, PROG RAM, WORD RAM, WRAM PMOD, CDC INIT,
CDC FLAGS, CDC DMA2, CDC DMA3 and CDC DMA1 all "OK", "Diagnostics complete." at frame 1,328
(`mcd_verificator_runs_in_mode_1_on_the_open_bios`). Its mode 2 run is unchanged at eighteen of eighteen. The crate's
182 tests pass, and those of the builds without each attachment.

## 12. The discs on the open BIOS against the player's BIOS, the Genesis unchanged, and the kit (2026-10-15)

**The comparison.** Each disc run in Nephrite for 6,000 frames, once on the player's BIOS (a model 1 American one, a
model 1 European one for Sonic CD's European disc) and once on the open BIOS, both with the same blank backup RAM made
by §10's footer rules and no pad pressed. Only the game was looked at: a picture was kept only from frame 1,500 on and
only while the main CPU ran outside the boot ROM, the sound's level only from there. The two boots differ in length,
so the games are compared by what they show, not frame by frame.

| Disc | On the player's BIOS | On the open BIOS |
|---|---|---|
| The 240p Test Suite | its main menu at frames 2,100 and 2,400 | the same menu, the two pictures equal byte for byte at both frames |
| The suite's sound test, CD-DA | track 3 plays, peaks 832, 760, 1,355, 1,399 a hundred frames apart | the same peaks in the same order, from about a hundred frames earlier |
| Chuck Rock II | its intro story, a landscape, then Chuck and the dinosaur, with sound | the same story with sound, further on at each frame |
| Lunar - Eternal Blue | its intro, a blue planet beside a moon, with music | its message that its save data was destroyed (a backup RAM call the open BIOS does not yet answer) |
| Mickey Mania | its opening, then its demonstration, Mickey and Pluto in the forest, with sound | the same: the Disney logo, then the forest with Mickey and Pluto, with sound |
| Sonic CD (USA) | the title, "Press Start", with its music | "Internal RAM is full. Game will start without save option.", then, START pressed, the title with its music |
| Sonic CD (Europe) | its opening film from frame 4,000 | the "RAM is full" message |

What differs is the backup RAM's directory, which waits for §5 Q2's specification. **Found by the comparison**: Sonic
CD's title asked for track 26 and heard nothing (§11's conversion).

**The Genesis unchanged.** The 71 digests of `Nephrite_Attachments.md` §11.1 (the kit's sample, the bench's games, the
240p disc with mcd-verificator as its image and its sheet, and without a boot ROM) equal the baseline's byte for byte.

**The kit** (`emusen-core-conform`, core ABI 1.9): the Genesis sample 64 of 64 with the core's cases; the 240p disc with
mcd-verificator as file 2 compliant; the 240p sheet and the five discs with no firmware file, on the open BIOS, 600
frames each, compliant. Two failures on the way, each fixed:

- **C14 on every cartridge**: core info lists file 11 for `md` (§11), and create refused it on a cartridge with no
  Sega CD beneath. A cartridge's machine now takes file 11 and uses it only with one.
- **C7 on Chuck Rock II**: a state saved during a seek's first exchanges loaded its travel count as 1 where it was 0,
  the load's "at least 1" guard meant for the travel that follows; save, load, save differed. The guard now applies
  only once the busy exchanges are over, where the count divides; the format is unchanged.

## 13. The cost, and a defect of the mix it found (2026-10-15)

Each disc for 6,000 frames with no pad pressed, the mean of each 500 frames in milliseconds, the picture drawn, on one
core under the timing lock (load 4 to 6), the same blank backup RAM on both BIOSes:

| Disc | Open BIOS | Player's BIOS |
|---|---|---|
| Chuck Rock II | 1.65 to 2.00 | 1.75 to 2.27 |
| Lunar - Eternal Blue | 1.34 to 2.29 | 1.37 to 2.46 |
| Mickey Mania | 1.25 to 2.73 | 1.87 to 2.41 |
| Sonic CD (USA) | 1.62 to 2.09 | 1.96 to 3.97 |

The two BIOSes' games are at different points at each frame (§12), so the ranges, not the columns, compare: the open
BIOS costs what the player's does, within the plan's 2.5 ms for the Sega CD but for a few windows of Mickey Mania's and
Sonic CD's on either. **Found on the way**: on the open BIOS Mickey Mania's cost first rose without end, a defect of
stage 9's mix that the open BIOS's timing exposed and the player's had not (D-48); fixed, with its test.

## 14. Step 10.4: `_BURAM` from the observer's specification (2026-10-16)

The source is `Nephrite_Bram.md` alone, the observer's page of §5 Q2's two-person method, with BM §7-1 and TB #1, #2 and
p. 9 for the calls' registers. None of the observer's images, runs or tools was seen (§9).

### 14.1 What was built

- **The file system** (`bios/bram.s`), written once for any volume: its size in `d7`, its first byte at `a5`, one byte
  every second address, its buffers past `a6`. `sub.s` includes it for `_BURAM` on the internal RAM (`$FE0001`, 8 KiB),
  its buffers at `$5650`-`$570D` of the BIOS's work area. Its calls, by `Nephrite_Bram.md`'s sections:
  - **BRMINIT** by §3's checks: `RAM_CARTRIDGE___` needs one of its first three longwords whole, then `SEGA_CD_ROM` and
    its zero exactly; another identifier of `0`-`9`, `A`-`Z` and `_` is another format (`d1.w` 2), anything else
    unformatted (`d1.w` 1). A volume of either format gives its identifier's 12 bytes to `a1`'s buffer (D-49).
  - **BRMSTAT**: the footer's counts, each the value three or four of its copies hold, else 0 (§3; read, not
    recomputed); `-1` and `-1` for a volume not Sega's (BM: negative counts mean formatting is needed).
  - **BRMSERCH**: `d0.w` the blocks, `d1.b` 0 or `-1`, `a0` the even address of the file's first block, `$FE0080` for
    block 1 as EM reports (§9).
  - **BRMREAD**: a normal file's blocks 64 bytes each; a protected file's blocks decoded, 32 bytes each (§7.1, §9).
  - **BRMWRITE**: §7.2: after the last file, the data first, then the directory block holding the new entry, coded
    whole, then both counts four times; refused when the blocks and any directory block the entry adds (§5.4) exceed
    the free count, or the data would reach the directory. A name already there: D-50.
  - **BRMDEL**: §7.3: the later files' data moved down over it, their entries one earlier with their first blocks
    lowered, the directory blocks from the deleted entry's rewritten, the vacated blocks and any directory block past
    the files left as they were, the counts as §5.4 gives them.
  - **BRMFORMAT**: §4: the footer alone, `S/64 - 3` free.
  - **BRMDIR** and **BRMVERIFY**: D-52, D-56.
- **The block code** (§6): the encoder from §6.1-§6.4's words; the decoder corrects one symbol error in each codeword,
  uses neither the low bits nor the CRC (§6.7), and reports a codeword past correcting (D-51).
- **`BRMSEEN`** (`$5710`), a bit for each function `_BURAM` is called with, as CALLSEEN records `_CDBIOS`'s.
- **The format module** (`EmuSen.Crystal/Systems/Genesis/SegaCdBramFormat.cs`), pure functions for both media,
  `mcd.backup` (8 KiB) and `mcd.ramcart` (Sega's 128 KiB; any power of two from 8 KiB to 512 KiB is read, §8): `Read`
  lists the entries, a save damaged where its directory block cannot be corrected or its blocks lie outside the data;
  `Copy` places a save as BRMWRITE does, its stored blocks copied as they are, which §7.2 observed of the BIOS's own
  copies; `Delete` as BRMDEL; `Format` as §4. The Memory Cards window and `memcard` now list and manage Sega CD saves.
- **Not built: `_mburam`.** TB #1 sends the reader to `MEM_MAP.ASS` for its address, and no document the writer may
  read gives it (D-55). `bram.s` takes its volume in registers so that the main CPU's program can include it over the
  cartridge's `$600001` once the address is known.

### 14.2 Where the specification is open

`Nephrite_Bram.md` §10 lists twelve points the evidence left open. Each is treated as unknown; the simplest or the
documented behaviour is chosen and recorded:

| §10 | Choice | Dispute |
|---|---|---|
| 1, writing an existing name | the same blocks and flag: written over in place; otherwise deleted, then written anew | D-50 |
| 2, block 0's pattern | the open BIOS never writes block 0 | D-56 |
| 3, register returns | BM's registers; the values §14.1 states | D-49 |
| 4, the read path's errors | a protected block past correcting fails BRMREAD; a directory entry is used as decoded | D-51 |
| 5, the unused directory slot | zeros, in a new block and after a delete | D-56 |
| 6, BRMDIR's buffer, BRMVERIFY's comparison | the entry's 16 stored bytes each; the blocks, the flag and the data compared | D-52, D-56 |
| 7, full RAM and large files | refused when the free count is short; no limit at 256 blocks | D-50 |
| 8, names | not checked | D-56 |
| 9, the footer's `5F...40` row | not checked | D-49 |
| 10, 11, other BIOS versions and cartridge sizes | one format for every size, as §8 found | — |
| 12, protected files | written and read as §6; no game was seen to write one | D-51 |

### 14.3 Measured (2026-10-16)

The crate's `openbram` tests run the sub CPU's program on a bare 68000 and call `_BURAM` as a game does: the four
vectors of §6.5 come out of the open BIOS's own BRMWRITE byte for byte (Sonic CD's and Lunar's directory blocks, the
protected block of zeros and the fox); a format leaves an image of `$A5` as it was below the footer, on 8 KiB and on a
128 KiB volume; BRMINIT and BRMSTAT decide each of §3's cases as the page records them; Lunar's four files land at
blocks 1, 15, 29 and 43 with 111, 96, 82 and 67 free, and are found at `$FE0080` onwards, read and verified; §7.3's
delete gives its result block for block; a protected file reads back through one corrupted symbol in every codeword
and flipped low bits, and fails with two in one codeword; BRMDIR lists by template, skips and reports a full buffer; a
121-block file on a 128 KiB volume leaves 1,924 free. WiseMan's `SegaCdBramFormatTests` check the C# module against the
same vectors and observations, independently written.

### 14.4 Shared by default, and a new machine formatted

As §5 Q2 and `EmuSen_SaveMedia.md` §10 Q2 decided: a console medium read by a format module now defaults to *Shared*
(`MemoryCards.DefaultMode`), so the Sega CD's internal backup RAM is one file every game uses, as on the console, and
Mistress offers once to import each game's own as it does for the PlayStation. A new machine's console memory with no
file yet is given the module's `Format()` (`MediaPlacement.Fresh`, read by `CoreEngine`), so a game meets a formatted
RAM though the open BIOS has no memory manager in which a player could format one. The core itself is unchanged: with
no file it still starts from zeros, which the kit and the digests rely on.

## 15. The gate (2026-10-16)

**The `_BURAM` round trips.** Each disc for 6,000 frames with no pad pressed, the player's BIOS a model 1 American one,
judged by the game alone: its pictures from frame 1,500 while the main CPU runs outside the boot ROM, its sound, and on
the open BIOS the `_BURAM` functions it called. No image the player's BIOS wrote was opened; each went straight from
one run into the next.

| Saved on | Loaded on | Sonic CD (USA) | Lunar - Eternal Blue |
|---|---|---|---|
| the open BIOS, from a formatted RAM | the open BIOS | its file written (BRMINIT, STAT, SERCH, READ, WRITE, VERIFY); the second boot reads it (INIT, STAT, SERCH, READ) and writes nothing | its four files written; the intro with music; the second boot reads them (INIT, STAT, SERCH, READ) and leaves the image as it was |
| the player's BIOS, from a formatted RAM | the open BIOS | INIT, STAT, SERCH and READ only: the file found and read, nothing written; the title with its music | the same four calls only; the intro, the blue planet and the moon, with music |
| the open BIOS, from a formatted RAM | the player's BIOS | the title with its music, then its opening film, as from a formatted RAM | the intro with music, as from a formatted RAM |

**Lunar reaches its intro** on the open BIOS, and **Sonic CD saves** on it: the RAM-full message of §10 is gone.
**Found by Lunar**: its intro streams with CDCREAD and reads the sector's header from the host data port before the
data, and the open BIOS sent the data alone, so the game asked for the same sector again and again; CDCTRN's header
buffer (BM §3-C) places the header first in a sub CPU read (D-53). CDCTRN now waits for DSR once and reads on without
polling, as the games themselves read the port. **Found by Sonic CD**: its opening film, which plays on the player's
BIOS, is black on the open one while its sound plays: the film's V-INT calls the boot ROM at `$02D4`, and its start
`$02B0`, routines no document describes (D-54, §5 Q4).

**The Genesis unchanged**: the 71 digests equal the baseline's. **The kit** (core ABI 1.9): the Genesis sample 64 of 64
with the core's cases; the 240p disc with mcd-verificator as file 2, the 240p sheet and the five discs on the open BIOS
at 600 frames, compliant. **mcd-verificator** in mode 1 and the 240p suite's boot pass as before (the crate's tests).

**Cost** on the open BIOS, the mean of each 500 frames, one core, under the timing lock: Lunar 1.36 to 2.13 ms,
Sonic CD 1.47 to 2.14 ms, now that both save and go on (§13's ranges for the others stand).

**The replacement's words** lose their last clause and say what the backup RAM does:

"Nephrite's own Sega CD BIOS, written from Sega's manuals: the documented calls answer as the manuals describe, saves
keep the console's own backup RAM format, and discs boot. A game that calls a routine no manual names, reads the
BIOS's code or depends on the exact timing of its loops may run differently; discs of every region start, since the
security check is not made; games cannot yet reach a backup RAM cartridge; the screen without a disc is Nephrite's
own, with no CD player or memory manager."

**What a player sees** with no BIOS of their own: a disc boots through Nephrite's screen into the game; Chuck Rock II,
Mickey Mania, the 240p suite and Lunar play, Lunar from its intro; Sonic CD saves and plays, but its opening film is
black with its music (D-54); saves move between the open BIOS and the player's in both directions; a RAM cartridge is
not offered to games. *Recommended:* keep the Sega CD's in-development mark until D-54 and D-55 are settled, since a
black film in the system's best-known game and a cartridge games cannot reach are what a player would meet first; the
backup RAM itself no longer stands in the way. The decision is the tester's.
*Decided 2026-10-16 by the tester:* the Sega CD is offered to players (`Nephrite_Plan.md` §7), D-54, D-55 and §16
open.

**Routing, once offered.** The Sega CD takes `.iso`, `.cue` and `.chd` beside the PlayStation. `SegaCdRoutingTests`
route the tester's library as it lists it, read only and with no development cores asked for: the 20 PlayStation
sheets and lone tracks under `psx`, Mega Man Legends 2's `(Track 1).bin` among them, to Diamond; the five Sega CD
sheets under `SCD` to Nephrite's `mcd`; the 946 cartridge `.bin` files under `genesis` to Nephrite's `md`; the Sega CD's
shelf is listed. A sheet's own tracks were left out, as the library lists them under their sheet: opened alone,
Rayman's 50 audio tracks go to the Genesis, an audio track holding neither a PlayStation licence nor a disc's sync,
which the cartridge's recogniser takes; this is as before the decision, since the Sega CD does not take `.bin`. The
library holds no `.chd`. A CHD cannot be told apart by content:
the probe reads its first bytes, the CHD's own compressed container, which neither console's recogniser reads, so a
CHD goes by its folder's ES-DE name, `psx` to Diamond and `segacd` to Nephrite, and in any other folder (the library's
`SCD` among them) it opens nowhere and the words say why. Before the decision every `.chd` went to the PlayStation,
the only system then offered for it.

## 16. The empty drive (2026-10-16)

`Nephrite_Bram.md` §11 records that the player's BIOS, with no disc from power-on, stays on its first picture and
ignores Start, and asks whether that is the drive. Answered from the drive's side only; the BIOS was not run without a
disc, since with no disc there is no game to observe.

- **The Mega-CD manuals** say nothing of what the drive answers with no disc. The Hardware Manual's CDD registers
  (§3-6) are marked "direct access ... is prohibited" and give no status codes; the BIOS Manual gives only the BIOS's
  own view: `bios_status` `$1xxx` "no disk" (§3-B, CDBSTAT) and the LED mode "CD ready & no disc" (LEDSET).
- **The public notes** the drive was built from (`Nephrite_Native.md` §51.2), krikzz's `cdd-cmd-info.txt` and `random
  notes.txt`, measured on a console: status `B` is "no disk", and report `F` is "busy: no disk or tray open or command
  in process".
- **Nephrite's drive** answers that way: from the host clock's first rise with no disc, every exchange reports status
  `B` with report `F` and flags 1; a report request (command 2) is taken but answered the same, since there is no TOC
  to report; opening and closing the tray gives `5`, then `B` again. The LL logs it was timed on were all made with a
  disc, so nothing measured covers an empty drive's answers beyond the notes' two lines.

So the drive answers as the only documents that speak of it say, and the stall is not explained by them. What they do
not settle, and a BIOS might wait on, is whether an empty drive ever answers a report request with that report's
number (Nephrite's answers `F` for ever), and the flags it gives. *Recommended:* the observer, who may watch the
player's BIOS, runs it with a probe build whose empty drive echoes a report request's number with zero data; if it then
passes its first picture, the question becomes one for a console. The open BIOS shows its own no-disc screen either way.

## 17. The replacement's words on the firmware page (2026-10-16)

The firmware page quotes a replacement's cost by its first sentence alone (`FirmwareItem.Change`,
`EmuSen_Settings_Reference.md` §4.89), and the big-screen menu shows that sentence, after "Using EmuSen's open
version.", as the focused row's footer, which wraps to two lines and no more. §15's words opened with a sentence of 174
characters, so the footer ran to 203. **Measured on 2026-10-16 [M]:** on the three BIOS rows the footer was cut with an
ellipsis at 1280 × 800 and at 1920 × 1200 (`WindowFitAuditTests.The_firmware_page_in_preferences_is_whole_at_both_sizes`).
The rows first reached the page when the Sega CD was offered to players, and nothing had measured them there before.

The words keep all that §15 says, in another order. The first sentence is the part a player choosing a BIOS needs, and
it is 63 characters: "Discs boot, and saves keep the console's own backup RAM format." The BIOS's provenance and the
documented calls move to the second sentence, and the caveats follow unchanged. The other open firmware opens the same
way (VenusRT's boot ROM: "Games may start a fraction of a second later."; Diamond's BIOS: "Most games start and save
as on a console."). With these words the audit finds nothing on any of the page's rows at either size.

Why the words were shortened and not the footer lengthened: the footer's two lines are the big-screen menu's for every
row of every page (`BigMenuForm`), and the rule that nothing is cut is met there by every other firmware row. One row's
long sentence is the outlier, and the desktop page, which shows the same sentence in full, reads better for it too.

## 9. Reading log (stage 10)

Opened and read: `Nephrite_Plan.md` whole; `Nephrite_Native.md` §49–§52, §53.1–§53.3, §53.5, §54.1, §54.2, §54.3 from
its second bullet, §54.4, §54.5, §56, §57.1, §57.3, §58, §59, §60.4–§60.7, §61, §62, and its headings; the
Nephrite pages' `README.md`; the Mega-CD BIOS Manual, every page, from page images rendered at 100 dpi (its text
layer is too damaged to cite); the Mega-CD Disc Format Specifications through its text layer, with appendix 2's two
pages from their images; the Mega-CD Software Development Manual through its text layer (the memory maps and the
register pages); the Sega-CD Technical Bulletins through their text layer, with the pages of "Backup RAM", the memory
map and "The status after booting is completed" from their images at 150 dpi; the Sega-CD Software Standards' first
300 lines of text; plutiedev's `subcpu-in-mode1` and `megacd-leds`; `DOCS.txt`'s sections X, A, C and D. Searched by
text, without opening files: the corpus's `docs/` for the calls' names, the excluded and unscreened files skipped by
name. Listed by name only: the test-program sources under `roms/_src/`.

*Since the plan was committed:* the 240p Test Suite's `segacdfiles/LukeProjectCD/_boot/_inc/cdbios.asm` whole, after
the screener cleared it (2026-10-15; DOCS.txt §X records the verdict); the same suite's `_boot/sp.asm`,
`_boot/ip-us.asm` and `segacdfiles/segacd.s` (its start-up's first 200 lines), a test program's own sources; the
headings of `Nephrite_Disputes.md` (by `grep -n '^#'`, which shows D-36's and D-40's rule titles) and its head and
D-32 for the entry shape. The security block the suite's initial program includes (`security/*.bin`) was not opened,
listed or disassembled.

*Later the same day:* plutiedev's `using-the-z80`; mcd-verificator's `test_std.c` (`testSTD` and `test2006`);
`docs/bram/emutos-mcd_bram-filesystem.md` whole, once allowed (used for the footer and the odd-byte layout alone; it
also shows one directory entry's bytes and where a first file's data lies, which were not used); the games' own code in
work RAM, disassembled with Beryl's disassembler where they called into the boot ROM or waited (Sonic CD's program at
`$FF0560`-`$FF064C` and `$FF1000`-`$FF104E`, and a byte search of its first 8 KiB, which showed the first eight bytes of
the security block, D-43). The player's BIOS was run as a black box with the five discs and the 240p suite: given to
Nephrite as a file, the games' pictures kept only from frame 1,500 and while the main CPU ran outside the boot ROM,
their sound's level from frame 1,500 (the sound's windows were not limited to frames outside the boot ROM, so a window may have held sound from before the hand-over; none of those levels was reported or used), and the frame's wall time; nothing of the BIOS's code, bytes, registers, screens or sounds, no
register watch, and no backup RAM it wrote was looked at. Read in code: `Cargo.toml`, the source
listing of `beryl-m68k`, and the firmware declaration in `src/mcd/frontend.rs`.

**Seen by accident.** `Nephrite_Native.md` lines 3997–4033 were printed to read §53.5–§54.2; the last line was the
first line of §54.3's first bullet, which is closed to this writer: "~~**The player's BIOS does not get past its
first screen** (§53.4).~~ *Found and fixed 2026-10-15 (§55).* Stage 9's first task, before the ASIC and the".
Nothing further of it was read. It records that a stall happened and where it was fixed, which §55's heading already
says, and nothing of what the BIOS did; it was judged not to disqualify the writer, and from then on each page with
closed parts was read by its headings' exact boundaries.

**Not opened, by the rule:** `spritesmind/t2061_p001.html`, `t1018_*`, `t3020_*`, `t458_*`, `t3218_*`,
`railgun/Trademark_Security_System.wiki`; mcd-verificator's `cd-log/BI-mcd-tst/` and `cd-log/sonic-bios/`; MiSTer's
`docs/mcd logs` and every RTL file; `Nephrite_Native.md` §53.4, §54.3's first bullet beyond the line above, §55,
§57.2, §60.1–§60.3; `Nephrite_Disputes.md`'s D-36 and D-40 beyond their headings; `~/.cache/emusen/probe/nephrite/s9/`;
`docs/bram/` but for `emutos-mcd_bram-filesystem.md` (read whole once allowed, used for the footer and the odd-byte
layout alone); the player's BIOS (`sysdir/scd/`), which was never read, and was run only as the black box described above; any emulator's source. The register
watch was run only with the open BIOS, never with a player's. *As of 2026-10-15, after the screening:*
`spritesmind/t458_*` and `t3218_*` are cleared and not yet opened; `t1018_*` is cleared but for named posts, and not yet
opened; `t3020_*` and `t2061_p001.html` stay closed.

*2026-10-16, step 10.4:* `Nephrite_Bram.md` whole, the only source of the format; the BIOS Manual's §7-1 and §3-C
pages and the Technical Bulletins' "How to access Back-Up Ram Cartridge" through their text layers; the Hardware
Manual's CDD register pages (§3-6) and krikzz's `cdd-cmd-info.txt` and `random notes.txt` for §16; `Nephrite_Native.md`
§51 by its exact boundaries; the games' own code in their RAM, disassembled with Beryl's disassembler, where they
stalled (Lunar's sub CPU at `$F000`-`$F1F0`, Sonic CD's sub CPU at `$7B20`-`$7C46` and `$300C0`-`$3024C`, Sonic CD's
main CPU at `$FF20E0`-`$FF2146`, `$FF2340`-`$FF23B4` and `$FF2500`-`$FF26E2`, and a search of its work RAM for calls
into the boot ROM). The player's BIOS was run as a black box with Sonic CD and Lunar, for the round trips and once to
10,000 frames to see Sonic CD's film: the games' pictures from frame 1,500 while the main CPU ran outside the boot ROM,
their sound's level; its backup RAM images went from one run into the next unopened, and were deleted afterwards.

**Not opened, added by the observer's method:** the observer's folder `~/.cache/emusen/probe/mcd-bram/` in any part
(its images, runs, tools and probe crate), and every backup RAM image the player's BIOS wrote, the observer's or the
round trips' own. Nothing on the excluded lists was opened at step 10.4.
