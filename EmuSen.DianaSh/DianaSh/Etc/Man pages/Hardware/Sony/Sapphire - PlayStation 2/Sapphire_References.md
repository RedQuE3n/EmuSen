# Sapphire_References — the documents, the test suite, the exclusions and the BIOS database

*Written 2026-10-24, at stage S2.* The record `Sapphire_Plan.md` §2 and §3.1 point to: every document and test
program fetched for the PlayStation 2, with its source, revision, licence and SHA-256; ps2tek's BIOS chapter screened
for the writers of the open replacement, its exclusions by section and line; the public database the BIOS entries cite;
and the facts this stage was asked to read. Everything fetched is under `~/.cache/emusen/probe/sapphire/`, with
`PROVENANCE.txt` at its root and `docs/DOCS.txt` and `docs/SHA256SUMS` for the documents. **Nothing fetched is
committed.** No firmware, no firmware image and no commercial image was fetched, opened or hashed, and no emulator,
SDK or loader source was read.

Marks: **[M]** measured on 2026-10-24 with the tool named; **[R]** read on 2026-10-24 from the source named; **[A]**
argued; **[C]** a choice.

---

## 1. The documents

In `docs/`, each file's SHA-256 in `docs/SHA256SUMS` [M `sha256sum`]:

| File | Source, revision | Status | SHA-256 |
|---|---|---|---|
| `ps2tek/index.html`, `README.md` | git clone of `PSI-Rockin/ps2tek` at **`1c9166066c3a`** (2022-01-26 19:11 −0500, "Merge pull request #7"), 224 commits; `index.html` 278,696 bytes, 6,695 lines | No licence file; no statement of its sources; written by DobieStation's author; pinned by Q2 | `2227bab31bcf…` (`index.html`) |
| `toshiba/TMPR79xx-um-r2.0_200104.pdf` | Toshiba, *TX79 Core Architecture*, Rev. 2.0, 2001-04, 727 pages; bitsavers' mirror | A manufacturer's manual of Toshiba's derivative, not of the EE (P4, P6) | `b60c694bdd0a…` |
| `toshiba/TMPR79xx-um-r1.0_200111.pdf` | Toshiba, *TX79 Family TMPR7901*, 2001-11, 309 pages; bitsavers' mirror | As above | `210f04fc3275…` |
| `gcc/MIPS-Options.html` | GCC manual, "MIPS Options", the current page | Public | `f400dff18794…` |
| `sony-papers/hc99.s3.1.Kunimatsu.pdf` | Kunimatsu et al., "5.5 GFLOPS Vector Units for 'Emotion Synthesis'", Hot Chips 11 (1999), 24 slides | Published | `ea003264e4be…` |
| `ecma/ECMA-119_2nd_edition_december_1987.pdf` | ECMA-119, 2nd edition (ISO 9660:1988), copied from Diamond's cache, byte-equal | Public standard | `a5b915d0f38f…` |
| `ecma/ECMA-267_3rd_edition_april_2001.pdf` | ECMA-267, 3rd edition, April 2001, *120 mm DVD – Read-Only Disk*, 96 pages | Public standard | `3fc0e0545578…` |
| `itu/T-REC-H.262-200002-S.pdf` | ITU-T H.262 (02/2000), MPEG-2 video, 220 pages, the edition contemporary with the console | Public standard, free | `3d38fcd00967…` |
| `wikipedia/PlayStation_2_technical_specifications.wiki` | revision 1355643867 (2026-05-23), wikitext | CC BY-SA 4.0; secondary | `918a36fac516…` |
| `batocera/systems_ps2.txt` | Batocera wiki, `systems:ps2`, raw text | Public; secondary | `01f761c5d41f…` |
| `redump/Sony - PlayStation 2 - BIOS Datfile (115) (2022-12-12).dat` | Redump, *Sony – PlayStation 2 – BIOS Images (115) (2022-12-12)*, `redump.org/datfile/ps2-bios/` | Public database: names, sizes, CRC32, MD5, SHA-1; no byte of any image | `7d9ca6fd4e23…` |

**Read, not kept** [R]: Microprocessor Report, K. Diefendorff, "Sony's Emotionally Charged Chip", vol. 13 no. 5,
1999-04-19, through a course's copy (`docencia.ac.upc.edu`; the copy Wikipedia cites did not answer); a magazine's
article, cited by page. **Not fetched**: the IEEE Micro and IEEE JSSC papers (behind paywalls; the Hot Chips slides
stand in for the first); Sony's Linux-kit manuals (Q1 allows only the tester's own or a lawful copy, and none was found
on Sony's hosting; each is sought when a stage first needs it); psdevwiki's pages (HTTP 403).

## 2. ps2autotests

`ps2autotests/`, a git clone of `unknownbrackets/ps2autotests` at **`97469ffbed86`** (2019-11-16, "Merge pull request
#59"), ISC [R `LICENSE.md`]. 312 files, 10,474,724 bytes outside `.git` [M `du -sb`]; each file's size and SHA-256 in
`PROVENANCE.txt`. The plan's "1,850 KB" was GitHub's repository size, its compressed history.

- **78 programs with expected texts** [M, the tree]: 55 `.elf` for the EE and 23 `.irx` for the IOP, every expected
  text with its binary and every binary with its expected text, in the folders the plan's §3.1 lists (cpu/ee 6,
  ee_cop0 1, ee_fpu 9, ee_simd 6, iop 7, vu0_macro 2; dma/dmac 2, spr 3, vif 6; gs/label and gs/signal; kernel/ee 10,
  kernel/iop 16; memory/ee 1; vu/lower 5, upper 1, games 1). The plan's count stands.
- **How they report** [R README]: text on standard output compared whole with the `.expected` file; recorded on a
  console through ps2link with a ps2sdk set-up. Nothing was run; running is S4's (P2 stands untested).
- **Read**: the README, the licence, `.gitattributes`, `.gitignore`, the tree's names, and the first five lines of
  each file in `common/` to classify it. `gentest.py` (the runner that sends a test to ps2link) and the tests' sources
  were not read. One expected text was read for D-0 (§5.3).
- **Excluded for every writer** (Q3: ps2sdk's code is never read): `common/crt0-ee.s`, `common/common-ee.mk` and
  `common/common-iop.mk`, whose headers name ps2dev's project and copyright; `common/xprintf.c` and `xprintf.h`,
  "xprintf.c from ps2sdk/ee/libc/src adapted to IOP". The other files of `common/` carry no copied-from note in their
  first lines and are each checked whole for one before a writer reads past the header, as Diamond's ps1-tests
  `common/` files were. The binaries link ps2sdk; running them is allowed, and no one reads them.

## 3. ps2tek's BIOS chapter, screened

**The screening reader** [C, `Sapphire_Plan.md` §2.2]: stage S2's writer read the whole chapter, `index.html` lines
4564–6695 at `1c91660`, on 2026-10-24, and is therefore **a non-replacement writer**: never a writer of the open BIOS
nor of the stages that model the IOP's kernel interface (§1.4, Q3). Whoever writes the open BIOS is someone else. The
criterion: an interface description (names, numbers, arguments, results, effects a program can observe through the
interface, export tables, the formats of files on a game's disc) is cleared; the kernel's internals (its structures,
its algorithms, its internal call graph, the values it writes), its boot, bytes or banners of a BIOS image, and
emulators' code are excluded. Line numbers are `index.html`'s at `1c91660`; `docs/DOCS.txt` §X holds the same list.

### 3.1 Excluded for replacement writers

| Lines | Section | What |
|---|---|---|
| 4602–4608 | BIOS File Structure | a hex dump of an SCPH-39001 image's ROMDIR: bytes of a BIOS |
| 4616–4641 | BIOS Boot Process | whole (set by the plan, 2026-10-23) |
| 4685–4707 | BIOS EE Threading | the kernel's internal thread control block |
| 4710–4722 | BIOS EE Threading | the kernel's saved-context layout on a thread's stack |
| 4725–4734 | BIOS EE Threading | the kernel's internal semaphore structure |
| 4737–4754 | BIOS EE Threading | the scheduler's algorithm and the interrupt path's register save |
| 4773–4774 | BIOS EE Syscalls | which stack the kernel uses for negative and positive calls |
| 4795 | BIOS EE Syscalls | the sentence "Internally calls LoadExecPS2(…)" |
| 4840 | BIOS EE Syscalls | the sentence on the priority linked list |
| 4870, 4878, 4885, 4894, 4919, 4945, 4979 | BIOS EE Syscalls | each sentence "Used (called) internally by …": the kernel's internal call graph |
| 5034–5035 | BIOS EE Syscalls | SifSetDChain's register value |
| 5049–5066 | BIOS EE Patches | whole (set by the plan) |
| 5069–5099, 5103–5121 | BIOS PlayStation Compatibility | PS1DRV's and TBIN's internals and the hack table; the PS1 mode is out of scope (§1.2). Lines 5100–5101, the IOP's and SPU2's rates in that mode, are a hardware statement and cleared |
| 5210–5256 | BIOS IOP IOPBOOT | whole: the boot binary, its entry in one image, its configuration parser and that parser's bug, and an excerpt "pulled from an SCPH-39001 BIOS" |
| 5264–5265 | BIOS IOP SYSMEM | the allocator's internal block lists |
| 5310–5324 | BIOS IOP LOADCORE | `_start` and its parameter block, the boot-internal hand-over |
| 6081–6084 | BIOS IOP IGREETING | the BIOS's boot banner (§5.9's exposure rule) |
| 6162–6168 | BIOS IOP REBOOT | the reboot's internal sequence through the modules |
| 700 | EE RDRAM initialization | what the BIOS writes in early boot |
| 1680 | D_ENABLEW | what one BIOS expects of D_ENABLER at boot |
| 3529 | SIO2_CTRL | the value the BIOS's SIO2 module writes on a reset: a trace of the BIOS's I/O |

### 3.2 Excluded for every writer: an emulator's code

| Lines | Section | What |
|---|---|---|
| 701–753 | EE RDRAM initialization | "The following logic used in emulators:", an emulator's register model quoted as code. Read once by the screening reader while finding the block's extent; that reader does not write the memory controller's MCH registers |
| 4182–4208 | IOP Console | an emulator's hook on program-counter values inside a BIOS image. Lines 4183–4195 read by the screening reader |

### 3.3 An emulator's choice, not hardware evidence

Readable, never cited as a console's behaviour: line 3334 (PCSX2's delta for a contiguous read), 3364 ("PCSX2 uses
00020603h as the version", the Mechacon's version), 3416 (PCSX2's ForbidDVD result), 3421 (DobieStation's OpenConfig
result).

### 3.4 Cleared

The chapter's introduction and the ROMDIR entry format (4564–4600, 4610–4612; the same format an IOPRP image on a game
disc uses, 6170–6172); the threading limits, `ThreadParam`, `SemaParam`, the status values and the semaphores'
behaviour (4646–4684, 4756–4763); every EE syscall's number, signature and effect but for §3.1's lines (4769–5045),
including the SuspendThread bug, which a program observes; the IRX export and import tables and the linking contract
every module on a game disc relies on, and the module list (5125–5206); SYSMEM's and LOADCORE's exports; and every
other IOP module section (5360–6075, 6089–6161, 6170–6692). Outside the chapter, the exception sequence's pseudo-code
(813–828) is the MIPS architecture's rule, not attributed to an emulator, and cleared for every writer.

**What the exclusions cost the open BIOS** [A]: the IOP reboot's IOPBTCONF format (in 5210–5256) is a file on game
discs' IOPRP images; the open BIOS's writer learns it from a source that is not the BIOS's parser, or from the images'
own text at the stage that needs it. Nothing else excluded is an interface.

## 4. The BIOS database: Redump

**Chosen** [C]: Redump's *Sony – PlayStation 2 – BIOS Images (115) (2022-12-12)*, the database whose names the plan's
entries already use (`ps2-VVVVr-YYYYMMDD.bin`) and which Batocera's page names as the source of its recommended images
[R `batocera/systems_ps2.txt`]. Its 78 console images are each 4,194,304 bytes; each entry's description gives the
console models and a market letter, "(Version 5.0 02/20/08 J)" [R].

- **Retail** [C]: an image whose models are consumer products (SCPH consoles, the DESR "PSX" video recorders, the KDL
  television with a console built in); the 19 development and test consoles' images (DTL models, names ending in `d`)
  are left out.
- **The market** is the description's letter: J, A, E [R]. The four first Japanese images carry "T" and SCPH-10000 or
  DTL-H10000; the two retail ones are taken as Japan's by their name's `j` [A]. So the Asian (`h`), Korean (`k`) and
  Chinese (`c`) images are Japan's and the Russian (`r`) Europe's, as the plan's §5.9 expected.
- **The lists** [M, parsed from the datfile, never typed]: Japan 27 images, the Americas 15, Europe 16, no MD5 twice;
  in `src/bios.rs`, each with its name beside it. The canonical image is first and the rest newest first, since the
  store takes the checksums in the core's order (`EmuSen_Firmware.md` §7.4).

| Entry | Canonical name | MD5 [R Redump] | Models |
|---|---|---|---|
| Japan | **`ps2-0230j-20080220.bin`** | `80ac46fa7e77b8ab4366e86948e54f83` | SCPH-90000 |
| the Americas | `ps2-0230a-20080220.bin` | `21038400dc633070a78ad53090c53017` (equal to Batocera's) | SCPH-90001/90010 |
| Europe | `ps2-0230e-20080220.bin` | `dc69f0643a3030aaa4797501b483d6c4` | SCPH-90002/3/4/8 |

**ps2-0210j-20040917 is not the newest Japanese retail version** [R]: Redump lists it for the DESR-5500, 5700, 7500
and 7700, PSX video recorders, and lists newer Japanese consoles' images: 0220j of 2005-06-20 (SCPH-75000), 2006-02-10
(SCPH-77000) and 2006-09-05 (SCPH-79000/90000), and **0230j of 2008-02-20 (SCPH-90000)**, the newest. The Japanese
entry is renamed to `ps2-0230j-20080220.bin`, the name and the tests moving together (`Sapphire_Native.md` §12.3).
Version 0250 exists only for Europe, the KDL-22PX300 television of 2010.

**Against the tester's images, by name only** [M `ls`, nothing opened]: there is no `ps2-0230j` in `~/Documents/BIOS/PS2/`.
The newest SCPH Japanese image there by name is `ps2-0200j-20040614.bin` (SCPH-70000 by the database), and the newest
with Japan's letter is `ps2-0220h-20060905.bin` (SCPH-79006/79007/90006/90007). `Misc BIOS/ps2-0250j-20100415.bin` has
a name the database does not list, so no checksum describes it and the store will not recognise it by content; it was
not opened.

## 5. Facts read for the plan

### 5.1 The IOP's caches

- **ps2tek** [R "Memory Map"]: the IOP's map gives "FFFE0000h (KSEG2) Cache control" and no cache sizes; its IOP
  chapter states none, and the IOP is "Duplicate of the PSX's CPU" [R "PS2 Overview"].
- **Microprocessor Report, 1999-04-19** [R the PDF's page 4, the text and its Table 1, "Source: SCE"]: the IOP's core "will be identical
  to the current PlayStation CPU, except for some minor enhancements to the cache and a 4× increase in DMA transfer
  rates"; the table: "Characteristics: Enhanced cache, 4× PlayStation DMA", "Frequency 33.8 MHz or 37.5 MHz selectable".
  A pre-release article, from Sony's statements.
- **Wikipedia** [R revision 1355643867] gives no IOP cache; copetti.org's PS2 article [R, read 2026-10-24] gives none.
- **What follows** [A]: no public document gives the IOP's cache sizes. The PlayStation's are 4 KiB of instruction cache
  and 1 KiB of scratchpad (`Diamond_Plan.md` §1.1), and Sony said the IOP's cache was "enhanced" in some minor way
  without saying how. S21 starts from the PlayStation's figures, marked [A]; the enhancement is a dispute candidate for
  the IOP's programs and the referees, and ps2tek's cache control register at FFFE0000h is where it would show.

### 5.2 The SPU2's voices

**Two cores of 24 voices, 48 in all: confirmed** by three independent readings [R]:

- ps2tek's I/O map: "1F900000h 180h Core0 Voice 0-23 registers" and "1F9001C0h 120h Core0 Voice 0-23 start/loop/next
  addresses", repeated for core 1 at 1F900400h. Arithmetic [A]: 180h is 24 × 16 bytes, 120h is 24 × 12 (three addresses a
  voice, each a pair of 16-bit registers), both consistent with 24 voices a core.
- Microprocessor Report's Table 1: "SPU2 Sound Chip: Voices 48-channel + software voices on CPU", "Sampling Rates 44.1 KHz
  or 48 KHz selectable".
- Wikipedia: "up to 24 channels per core for a total of 48 channels".

### 5.3 For D-0, the clocks

- `cpu/ee_cop0/performance.expected` [R, the console's output, 13 lines] tests the counters' register mapping and
  which modes count; it records no count against a time base, so it gives nothing for D-0.
- Microprocessor Report gives the IOP at "33.8 MHz or 37.5 MHz selectable" and the GS at 150 MHz [R], and
  copetti.org the IOP at 37.5 MHz [R]: the design family again, entered in `Sapphire_Disputes.md` D-0.

## 6. What was read for this page

ps2tek's BIOS chapter whole, its I/O map's SPU2 rows, its memory map, and the passages located by search (the
emulators' mentions, the BIOS's mentions, the code blocks by pattern; §3.2 says which bodies were read);
ps2autotests' README, licence and tree, `common/`'s first lines and one expected text; Redump's datfile; Batocera's
page; Wikipedia's page; Microprocessor Report's article; copetti.org's PlayStation 2 article; the Hot Chips slides'
first page. No emulator, SDK or loader source; no firmware.
