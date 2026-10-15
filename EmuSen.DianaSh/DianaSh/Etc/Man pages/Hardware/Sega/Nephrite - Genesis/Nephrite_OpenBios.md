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

## 6. Reading log (stage 10, to the plan)

Opened and read: `Nephrite_Plan.md` whole; `Nephrite_Native.md` §49–§52, §53.1–§53.3, §53.5, §54.1, §54.2, §54.3 from
its second bullet, §54.4, §54.5, §56, §57.1, §57.3, §58, §59, §60.4–§60.7, §61, §62, and its headings; the
Nephrite pages' `README.md`; the Mega-CD BIOS Manual, every page, from page images rendered at 100 dpi (its text
layer is too damaged to cite); the Mega-CD Disc Format Specifications through its text layer, with appendix 2's two
pages from their images; the Mega-CD Software Development Manual through its text layer (the memory maps and the
register pages); the Sega-CD Technical Bulletins through their text layer, with the pages of "Backup RAM", the memory
map and "The status after booting is completed" from their images at 150 dpi; the Sega-CD Software Standards' first
300 lines of text; plutiedev's `subcpu-in-mode1` and `megacd-leds`; `DOCS.txt`'s sections X, A, C and D. Searched by
text, without opening files: the corpus's `docs/` for the calls' names, the excluded and unscreened files skipped by
name. Listed by name only: the test-program sources under `roms/_src/`. Read in code: `Cargo.toml`, the source
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
§57.2, §60.1–§60.3; `Nephrite_Disputes.md` in any part; `~/.cache/emusen/probe/nephrite/s9/`; the player's BIOS
(`sysdir/scd/`), which was neither read nor run; any emulator's source. No register watch was run.
