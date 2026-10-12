# Diamond_OpenKernel_Phase2 — the open BIOS kernel, phase 2

*Written 2026-10-11, at stage 10, with the tester's go of the same day.* This page continues `Diamond_OpenKernel.md`,
whose §1-§11 record the kernel's first phase, and numbers its sections from §12 on so that the source can cite either
page by section alone. Phase 2 is the rest of `Diamond_Plan.md` §5.9's option B: (3) the CD boot, (4) the memory cards in
the on-card format, (5) the pads through the kernel's functions (phase 1's, now shared with the cards), and (6) a screen
with no disc. The marks are the plan's: **[M]** measured on 2026-10-11 with the tool named; **[R]** read from the source
named; **[A]** argued; **[C]** a choice where the documents are silent, each listed in §20.

It was written under the replacement writers' rules of the plan's §1.3, as phase 1 was: from psx-spx's descriptions of
the kernel's interface, its hardware chapters, ECMA-119 and test programs, never from a BIOS image, listing,
disassembly or trace, an emulator's source, OpenBIOS, PSn00bSDK's or PsyQ's libraries. The games were run as black boxes
on this kernel only, with no firmware folder. §21 lists every document opened and states the exposures.

---

## 12. The CD-ROM and its files

### 12.1 The controller, from the kernel

`kernel/src/cdrom.rs` drives the controller at its host interface, psx-spx's "CDROM Controller I/O Ports" [R]: the bank
in 1F801800h, a command written once BUSYSTS clears with its parameters first and the parameter FIFO cleared by
HCLRCTL bit 6, the response read from RESULT while RSLRRDY is set, every interrupt acknowledged by HCLRCTL 1Fh, HINTMSK
1Fh so that IRQ2 reaches I_STAT. An **operation** is a short list of steps, each a command with whether a second
response follows its first, or a data step that takes sectors on INT1 until a count is met; the next step is issued
when the last one's last response arrives, and an INT5 or INT4 ends the operation as failed [C on the structure]. One
operation runs at a time.

**Who services it** [C, §20 item 1]. While the kernel has an operation running it *owns* the drive: it sets I_MASK's
bit 2 (restored on release when it was clear), and its element at priority 0, the first function of
`EnqueueCdIntr`'s element (A(90h), `CdromIoIrqFunc1`), services the controller when IRQ2 is pending. A caller waiting
for an operation services the controller itself between interrupts, inside a short critical section, so that the
kernel's CD calls work with interrupts disabled too. When the kernel owns no operation, its element answers nothing and
acknowledges nothing, so that a game's own CD handler, which most games install through their libraries, sees every CD
interrupt. psx-spx says the original's `_96_remove` and `DequeueCdIntr` do not work and that its CD handlers stay in the
chain while games run ("A(56h) or A(72h)", "C(02h)") [R]; a handler that took IRQ2 from a game would break the game, and
one that takes it only while the kernel reads cannot [A].

**Sectors by DMA** [A]. psx-spx documents the kernel's CD reads as DMA-driven: its event F0000003h,10h is "cdrom DMA
finished", and the call table names `CdromDmaIrqFunc1` and `2` [R "CDROM Events", "A-Functions"]. On each INT1 the kernel
raises BFRD, acknowledges INT1, waits for DRQSTS ("RDDATA": "software must set the BFRD flag, then wait until DRQSTS is
set") and moves the sector by DMA channel 3 in its usual mode, CHCR 11000000h, BCR one block of the sector's words
(psx-spx "DMA Channels", "Commonly used DMA Control Register values") [R], DPCR's channel 3 enable set with its priority
kept [R "DPCR"], then clears BFRD; it waits for CHCR bit 24 itself, so the DMA functions A(91h) and A(93h) return 0.
The kernel's own reads use mode A0h, double speed and the whole sector but its sync [C, §20 item 2]: each sector's header
is compared with the sector wanted, an older sector is dropped, and a later one (a sector lost while interrupts were
held) re-seeks the read from the sector wanted, at most eight times. psx-spx says `CdReadSector` reads in mode 80h
("A(A5h)") [R]; the kernel's reads differ there and nowhere a program can see but in the mode Getparam would report
afterwards.

The init is psx-spx's own recipe, "To init the CD" [R]: the interrupts flushed, HCHPCTL 0, COM_DELAY 1325h, two Nops,
Init, Demute.

### 12.2 The CD calls

| Call | What it does here |
|---|---|
| A(54h), A(71h) `_96_init` | the priority 0 element, the kernel's five CD events (class F0000003h, specs 10h, 20h, 40h, 80h, 8000h, psx-spx "BIOS Event Summary") opened once a SetConf, the volume forgotten, the controller initialised; 1 [C on the value, the one `cd-kernel.s` expects of either kernel] |
| A(56h), A(72h) `_96_remove`, A(A3h) `DequeueCdIntr` | the element removed; the original's SysDeqIntRP defect is not reproduced, as in phase 1 (§4.2) |
| A(A2h) `EnqueueCdIntr` | the element at priority 0, "with prio=0 (fixed)" |
| A(95h) `CdInitSubFunc` | the controller's init alone |
| A(A5h) `CdReadSector(count,sector,buffer)` | the kernel's checked read; count, or -1 |
| A(A6h) `CdGetStatus()` | Getstat waited for; the status, or -1 |
| A(78h) `CdAsyncSeekL(src)`, A(7Ch) `CdAsyncGetStatus(dst)`, A(81h) `CdAsyncSetMode(mode)`, A(7Eh) `CdAsyncReadSector(count,dst,mode)` | each started and 1 returned; the end by F0000003h,20h (and 10h after a read's last sector), 80h on INT4, 8000h on an error; the read's sector size from mode bits 4 and 5, ReadS with bit 8, Pause after the count; the status byte stored at the end [R the entries; C that the read pauses] |
| A(94h) `CdromGetInt5errCode(dst1,dst2)` | the last INT5's two bytes, never reset, as documented |
| A(9Eh) `SetCdromIrqAutoAbort(type,flag)` | the flags kept; the kernel's reads end by their count [C] |
| A(A4h) `CdGetLbn(filename)` | the file's first sector, or -1; a device prefix passed over [C]. The documented -1 for "the first file in the directory" is the dot entry, which no name matches here either; the garbage for a missing file is not reproduced (§4.8's rule) |

### 12.3 ISO 9660 on "cdrom:"

`kernel/src/cdfs.rs`, from ECMA-119 2nd edition [R]: the primary volume descriptor at sector 16 (type 1, "CD001"), the
root's directory record at BP 157-190, the L path table's place at BP 141-144 (§8.4); directory records by §9.1, each
ended in its sector (§6.8.1.1), the identifiers 00h and 01h the directory and its parent. A path is resolved from the
root when it starts with a backslash, else from the directory `cd` set (psx-spx "B(40h)": "the directory name may be
ommited from filenames, but the device name must be still included") [R]; names are compared without case, LoadExec
upper-casing them anyway, and when the pattern gives no ";" the identifier's version is not compared [C]; "?" and "*" as
psx-spx's firstfile2 gives them, its "?" bug reproduced (`names.rs`). A one-sector buffer keeps the directory sector
read last.

The **disk id** psx-spx gives the FCB, a "checksum across path table", is the 32-bit sum of the words of the path
table's first sector [C on the sum]; a read on an FCB whose id is not the mounted volume's fails with 10h. The volume is
read again on `cd` (psx-spx: "the function does always load the path table") [R], at `_96_init`, and at an open after a
Getstat shows the shell opened.

The device ("cdrom", flags 14h, 800h-byte sectors, "CD-ROM", psx-spx "Device Control Blocks") [R]: open fills FCB[20h]
with the size and FCB[24h] with the first sector; read takes any length from any position, to the file's end at most,
error 16h at or past it [C: psx-spx's "per $0800" is read as the transfer's grain]; write returns 0 ("Writing to the
cdrom returns 0") [R]; firstfile and nextfile fill the direntry with the identifier whole, attribute 0, the size and the
first sector; chdir keeps the path without checking it, as documented, and returns 1. The CD and card devices are added
at boot with the TTY by `InstallDevices` (C(12h)), whose documentation lies in a section not opened (§21): the three
names psx-spx's DCBs list are the ones installed [A].

## 13. The memory cards' sectors

`kernel/src/card.rs`, from psx-spx's "Memory Card Read/Write Commands", "FLAG Byte", "Timings" and "BIOS Memory Card
Functions" [R]. A sector is read (52h), written (57h) or its FLAG taken (Info: 52h stopped at FLAG, "an incomplete
dummy read command") over SIO0 with `sio.rs`'s exchange, the one phase 1's pads used, now shared: the address 81h (82h-84h
for a multitap's slots by the port's low digit, "Multitap Support"), the command, the IDs 5Ah 5Dh, the sector, then the
data and checksum, the read's confirmed sector and end byte 47h checked, the write's end byte 47h, 4Eh or FFh. The read's
late /ACK after 5Ch is awaited longer than the others [R "Timings": "circa 31000 cycles"]. A card whose FLAG has bit 3
set, "the directory wasn't read yet", fails a read or write as a new card unless `_new_card` was called first, as
psx-spx describes ("B(50h)") [R].

**The pace** [R "Poor Memcard Performance"]: one request a slot, carried out by the element the pads and cards share at
priority 2 ("PadCardIrq"), on vblank, slot 1 on even frames and slot 2 on odd ones, as psx-spx describes the original
("accesses Slot 1 only on each second frame, and Slot 2 only each other frame"); so a sector takes two frames. A caller
waiting on a request carries it out itself, inside a critical section, when nothing will: the element not queued,
vblank masked, interrupts off, or no vblank seen for about three frames [C, §20 item 3]. The transfer runs inside the
vblank handler and polls each byte, about 240,000 cycles a sector [M, 140 bytes at the kernel's rate], where the
original is driven by IRQ7 at priority 1 ("CardSpecificIrq") [R "Priority Chains"]; the games' interrupts are delayed
by it during a card access, not lost [A].

**The calls**: `InitCARD2(pad_enable)` (the pads' flag, SIO0 reset), `StartCARD2` and `StopCARD2` (the shared element,
queued once, removed when neither pads nor cards use it), `_card_read`, `_card_write` (1, or 0 past sector 400h, which
psx-spx says the original accepts) and `_card_info`, each queued for its slot, `card_write_test` (sector 3Fh written with
the header frame's usual contents, "MC", zeros and 0Eh, psx-spx's "Write Test Frame"), `_new_card`, `_card_status` and
`_card_wait` with psx-spx's codes (01h ready, 02h, 04h and 08h busy, 11h failed or timed out, 21h an error), `_card_chan`
(the port of the request completed last [C]). Each low-level completion delivers F0000011h with spec 4, 100h (timed
out, psx-spx's "Event Specs"), 2000h (a new card) or 8000h (an error), and calls the matching `bufs_cb` through the A
table, which delivers F4000001h: A(A7h) 4, A(A8h) and A(AEh) 8000h, A(A9h) 100h, A(AAh) 2000h, the mapping psx-spx's
"Memory Card - Higher Level File/Device Events" gives [R]. *Found 2026-10-11*: Rayman calls `_card_info` and waits on
F4000001h's events, not F0000011h's; with only the lower-level event delivered it waited for ever on a black screen;
with the callbacks' event as well it plays its opening [M, `--calls` and a RAM dump of the EvCBs]. Before a request the
events of both classes are made busy again, psx-spx's note that 8001h alone "is NOT undelivered" read as the others
being so [A].

## 14. The file system on the card

`kernel/src/bu.rs`, from psx-spx's "Memory Card Data Format" and "BIOS Memory Card Functions" [R]. Each slot keeps block
0's frames 0-35: the header frame "MC" with its checksum, the fifteen directory frames (allocation state, size,
next-block link, name, checksum: the XOR of bytes 00h-7Eh at 7Fh), and the broken sector list (frames 16-35), through
which a file's sectors are redirected to frames 36-55 [R; C that the kernel never adds an entry].

- **Reading the directory**: frames 0-35 read with the new-card flag ignored, and when the card was new, the write test
  that clears its flag. A synchronous open first reads sector 0, "anytime when opening a file (in synchronous mode), the
  BIOS does additionally read sector 0" [R], and reads the directory again when that read finds a new card or a card
  formatted otherwise than the copy says; an asynchronous open and the search calls read it only when it was not read
  ("unless the directory wasn't yet read") [R].
- **Formatting** (`format`, B(41h), and auto format when `_card_auto(1)` and a card lacks "MC"): frame 0 "MC", frames
  1-15 free (A0h, link FFFFh), frames 16-35 an empty broken sector list (FFFFFFFFh), frames 36-62 FFh, frame 63 the
  header, each with its checksum, all written [R; C that free frames' links are FFFFh].
- **Creating** (open with bit 9, bits 16-31 the blocks, 1 when 0 [C]): 11h if the name exists, 1Ch without the blocks;
  the free blocks, freshly formatted ones first, then deleted ones [C], chained in block order, the first 51h with the
  size and the name, the middle ones 52h, the last 53h, each frame's link the next block less one, the last FFFFh [R].
- **Reading and writing**: lengths and positions in whole 80h-byte frames, 16h otherwise or at the end; the file's
  sectors found along its chain. With access mode bit 15 the call returns at once and a job carries the sectors on, one
  a turn, ending with DeliverEvent(fd, 4) ("0000000xh,4 card file handle (x=fd) done okay") and the callback's
  F4000001h event, or on a failure FCB[18h] = 10h and the failure's event [R].
- **firstfile2 and nextfile**: the frames whose state is 51h, or A1h in find mode 1, matching the pattern; the direntry
  the name (up to 20 bytes), the attribute 50h or A0h, the size and the first block's sector [R; C on the sector].
- **erase**: the chain's states made A1h, A2h, A3h; **undelete** the reverse, 11h if the name is in use; **rename** the
  first frame's name, 11h if the new one exists [R]. `card_clear_error` (A(6Fh)) clears FCB[18h], as its entry shows.
- `_bu_init` (A(55h), A(70h)) forgets both directories and turns auto format off; `_card_load` (A(ACh)) reads the
  directory as an asynchronous job ending with the higher-level events, 2000h for an unformatted card ("or unformatted
  (bad "MC" id)") [R].

The device: "bu", flags 14h, 80h-byte sectors, "MEMORY CARD" [R]; testdevice returns without doing anything, as psx-spx
says for every existing device.

## 15. The boot

**The shell's entry** (`boot.rs`, `shell.rs`). As in phase 1, the reset path ends by entering 80030000h with SR 401h, SP
801FFF00h and I_MASK clear; 80030000h now holds four instructions that jump to the shell in the ROM, so that a
side-loader taking over at 80030000h, as Diamond's machine does for a PS-EXE (`Diamond_OpenKernel.md` §3.3), finds the
same state as before and never runs the shell. The shell runs on a stack of its own in the kernel's RAM, which a
program loaded high cannot overwrite [C].

**The disc**: `_96_init`, then GetID until it answers, through the spin-up (INT5 with error 80h and the shell flag clear,
retried for about two seconds): a licensed data disc boots; no disc (INT5 08h 40h), an audio disc, an open lid or a
refusal goes to the screen of §16 [R "GetID"; C on the retry].

**SYSTEM.CNF** (psx-spx's "CDROM File Playstation EXE and SYSTEM.CNF") [R]: read from the root as "cdrom:\SYSTEM.CNF;1";
its lines ended by CR, LF or 00h, each "KEY = value" with the key's case and blanks ignored [C]; BOOT's name to its first
blank, the rest of the line, up to 127 bytes, copied to 180h as the program's argument; TCB, EVENT and STACK in
hexadecimal; what it does not give, or the whole file when the disc has none, the defaults with "cdrom:PSX.EXE;1" ("PSX.EXE
(Boot-Executable) (default filename when SYSTEM.CNF doesn't exist)"). Then, in the order the documents imply: SetConf
with EVENT, TCB and STACK, `_96_init` ("Before starting the boot executable, the BIOS automatically calls _96_init()"),
Load, the header's stack replaced by STACK with offset 0 ("the stack values in the exeheader seem to be ignored (replaced
by the SYSTEM.CNF value)"), and Exec(headerbuf, 1, 0) [R].

**The state at the jump** [R Exec's entry; C the rest]: PC the header's, GP the header's, SP and FP SYSTEM.CNF's STACK, a0 1
and a1 0, the memfill region zeroed, RA into Exec; SR 401h, I_MASK and I_STAT clear, the CD's element at priority 0 and
the five CD events open, the kernel owning nothing, the pads and cards not started. If the program returns, SystemError
("B", 38Ch) through the A table ("CDROM File Playstation EXE": "SystemErrorBootOrDiskFailure") [R].

**The warm boot**, A(A0h) `_boot`, "resets the kernel and reboots from CDROM" without the intro and without reading
SYSTEM.CNF again [R]: the settings SYSTEM.CNF gave are copied to the scratchpad, k1 is set to a marker and the reset
entry is entered with interrupts off; the reset path, which uses neither k1 nor the scratchpad, sets the kernel up anew,
and the shell, given k1, starts the same program with the same settings [C on the carrying].

**LoadExec** (phase 1's §4.11) now loads through the CD; its "Part 3", reloading the boot program, is still not
attempted, since psx-spx describes it as faulting at 6F726463h and then locking up, and the lock-up is kept [C].

## 16. The screen with no disc

`screen.rs`, `font.rs` and `manager.rs`. A plain screen and a minimal card manager, written for this kernel, with no
text, logo, layout or sound of Sony's shell (the plan's §5.9 item 6). The GPU is set as psx-spx's "GPU Display Control
Commands" give it [R]: GP1 reset, 320x240 NTSC at 15 bits, the display range X 600-3160 ("NTSC 320") and Y 10h-100h
(88h less and plus 120), the drawing area over it, display on. Text is the kernel's own 8x8-cell font, 5x8 glyphs for
ASCII 20h-7Fh drawn for it, uploaded once to VRAM at x 640 as a 15-bit texture (set texels 7FFFh, the rest 0000h,
transparent) and drawn as 8x8 textured rectangles, GP0(74h), blended with the text's colour, six pixels apart. A command
is sent once GPUSTAT bit 26 shows the GPU ready for one, as Diamond's GPU sets it (`src/gpu/mod.rs`) [R]. *Found 2026-10-11*: waiting for bit
26 before every word, parameters included, stalled the screen's first drawing past 900 frames, since Diamond's bit 26
reads clear while a command's parameters are awaited [M].

The manager lists both cards' saves by their titles, the title frame's Shift-JIS converted to ASCII for the full-width
letters, digits and the common punctuation of JIS X 0208's first rows ("Shift-JIS Character Set") [R], with each save's
blocks and the free blocks; Up and Down choose a save, Left and Right a card, Cross copies the save to the other card
(read whole into user RAM at 80100000h, a file of its blocks created there and written), Triangle twice deletes it,
Square twice formats the card. It reads the pads through InitPAD2 and StartPAD2, and the cards through the kernel's own
file calls, as a program would. Every two seconds it asks the drive with GetID and boots a data disc put in, and reads
each card's sector 0 to notice a card changed. On leaving it stops its pads and cards and masks vblank again.

*Found 2026-10-11*: StartPAD2 enabled vblank in I_MASK before queueing its element; with interrupts on and a vblank
already pending, the interrupt came before the element that acknowledges it existed, and the exception handler was
entered for ever [M, PCs sampled every few frames, all in the exception path, I_STAT and I_MASK both 001h]. The element
is now queued first. Games call StartPAD2 inside critical sections, which is why phase 1 never showed it.

## 17. The tests

`cargo test --release` in `kernel-harness/` [M]: 49 tests, the page test among them. Changed for phase 2:
`files_on_the_tty_and_the_errors_of_the_file_layer` (an unknown device is now "pcdrv:", and firstfile2 on "bu00:" with no
card answering), `the_card_calls_with_no_card_and_the_absent_character_sets` (replacing the phase 2 stubs' test:
`_card_read` queued, `_card_wait` carrying it out to 11h, a sector past 400h refused, Krom2RawAdd -1),
PrintInstalledDevices' three devices, and the `registers` test skipping the CD calls, which wait for a drive the harness
bus does not have.

In Diamond's `tests/programs.rs` [M], on the open kernel with no firmware, the discs and cards built in the tests
(`tests/support/iso.rs`, an ISO 9660 volume of mode 2 form 1 sectors):

| Test | What it shows |
|---|---|
| `the_open_kernel_boots_a_disc_by_its_system_cnf_and_reads_its_files` | `programs/kernel-cd.s` booted from `cdrom:\DIR\PROG.EXE;1` by SYSTEM.CNF: a0 1, a1 0, SP 801FF800h, the header's GP, "hello" at 180h, GetConf 14h, 6, 801FF800h; open, lseek, reads across sectors to the file's end and past it, close, CdGetLbn, CdReadSector of sector 16, firstfile2 and nextfile over the root ("DATA.BIN;1", "DIR", "SYSTEM.CNF;1"), cd and a relative open, a missing file's 02h, CdGetStatus; no call changing S0-S7, GP, SP or FP |
| `the_kernels_cd_functions_read_a_disc_on_the_open_kernel` | stage 7's `cd-kernel.s`, which ran on the player's BIOS, with the same expectations on this kernel |
| `the_open_kernels_card_files_keep_the_on_card_format` | `programs/kernel-card.s` on a blank card: format; a 2-block file created, written, read back; a second file; firstfile2 and nextfile; erase, the erased name 02h, find mode 1, undelete, erase, rename; an asynchronous read ended by F4000001h,4; `_card_read` ended by F0000011h,4; the empty slot's `_card_info` timing out (100h, status 11h); and the card itself: the header, each frame's checksum, the chain 51h-53h with its link, the erased file A1h with its name and size, twelve free blocks, the empty broken sector list, frames 36-62 FFh, frame 63 the header, the data in blocks 1 and 2 |
| `the_screen_with_no_disc_formats_a_card_and_boots_a_disc_put_in` | no disc and a blank card: the manager's background in VRAM, Square twice formatting the card, then the lid opened and closed on the boot disc and the disc booted |

**A negative control** [M]: a kernel whose directory frames were written without their checksum fails
`the_open_kernels_card_files_keep_the_on_card_format` at frame 0's checksum; the committed kernel passes.

Stage 3's seven ps1-tests programs, run by WiseMan's `DiamondTests` on the open kernel, still pass (§18.3).

## 18. The games, with no firmware folder

### 18.1 Booting

Each of the tester's seven copied discs (`~/.cache/emusen/probe/diamond/discs/`) given to the run tool with no `--bios`
and no `--boot`, so the open kernel boots it by its SYSTEM.CNF, pictures every 300 frames to frame 3,600 [M, at
`0efba49b`, with the SPU of stage 8 merged]. "The player's BIOS" is the result recorded for the same disc booted on the
player's BIOS through the kernel's own `LoadExec` (`Diamond_CDROM.md` §9.4, `Diamond_MDEC.md` §9, `Diamond_SPU.md` §11),
which skips that BIOS's intro, read from the pages and not run here.

| Game | The game's code starts | On the open kernel | On the player's BIOS (by `--boot`) |
|---|---|---|---|
| Crash Bandicoot | frame 207 | its title screen by frame 2,400 | its title screen by frame 2,100-2,400 |
| Spyro the Dragon | frame 229 | its title screen by frame 3,600; its library prints "CdRead: sector error" and "retry" about 400 times from frame 740 (§20 Q2) | its title screen by frame 3,300 |
| Ape Escape | frame 274 | its DUALSHOCK screen, then black at 1,500 and white from 1,800, the Analog button not pressed (§20 Q3) | waits at its DUALSHOCK screen |
| Castlevania: SotN | frame 252 | black throughout, "MDEC_out_sync timeout" on the TTY | black throughout |
| Rayman | frame 278 | the Ubi Soft logo and its opening movie | the same |
| Silent Hill | frame 157 | its content warning, the Konami and KCET logos, "no MEMORY CARD inserted", then black | the same |
| Tekken 3 | frame 376 | "NAMCO PRESENTS", its title screen and attract mode | the same |

The game's start is the first frame at which it calls the kernel through A0h, B0h or C0h (the run tool's `--calls`).
No game failed in a way that pointed at the kernel's internals.

### 18.2 The calls the games made

Counted by the run tool's `--calls`, which notes each entry of the A0h, B0h and C0h vectors, observing the game. Besides
phase 1's calls (memory, strings, events, the exception exits, ChangeClearRCnt, ChangeClearPAD, GetC0Table, GetB0Table,
SysEnqIntRP and SysDeqIntRP, the GPU's GPU_cw, the TTY): `_96_remove` (A(72h), every game), `_bu_init` (A(70h)),
`InitCARD2` and `StartCARD2` (B(4Ah), B(4Bh)), `_card_info` (A(ABh), Rayman and Silent Hill), `_card_auto` (A(ADh),
Rayman), and in Tekken 3's card menu `_new_card`, `_card_write`, `_card_load`, firstfile2, format, open, read, write and
close on "bu00:" (B(50h), B(4Eh), A(ACh), B(42h), B(41h), B(32h)-B(36h)). The boot itself uses the CD device and Load.
Ape Escape, Spyro, Silent Hill and Tekken 3 remove an element from a chain and add their own (C(03h), C(02h)); Ape
Escape's, read from the ExCBs at frame 600, replaces the shared pad and card element at priority 2 [M, a RAM dump].

### 18.3 A save, and its load

Tekken 3 on the open kernel with a blank card in slot 1 (the run tool's `--card` and `--press`): Option Mode, Memory
Card, Card Save, Start; the game asks "Initialize memory card?", Start; it formats the card through `format`, creates its
one-block file "BASLUS-00402TEKKEN-3" and writes it, and shows "SAVE OK!". Booted again with that card, its Card Load
shows "LOAD OK!", and at boot it opens and reads the file by itself [M, pictures]. The card written checks as psx-spx's
format: the header and every checksum, block 1 51h of 2000h bytes linked FFFFh, its title frame "SC" with three icon
frames, the empty broken sector list, frames 36-62 FFh and 63 the header [M, a script over the file]. The cards kept for
the cross-kernel step are listed in §20.4.

The manager of §16, with that card in slot 1 and a blank one in slot 2, formatted card 2, copied the save to it and
deleted it there; card 2 then held the save's frames with block 1 in state A1h [M, the run tool and the same script].

## 19. The call table

`Diamond_OpenKernel.md` §5 holds the whole table, printed from `kernel/src/table.rs`. With phase 2: 576 entries, 222
implemented, 2 absent from an open kernel (Krom2RawAdd and Krom2Offset, which address the ROM's character sets: the
kernel carries none, and returns -1), 346 no function as documented, 6 undocumented. No "phase 2 stub" is left.

## 20. Choices, questions, the cost and the image

### 20.1 Choices where the documents are silent

1. The kernel services the CD controller only while it has an operation of its own, and otherwise leaves every CD
   interrupt to the program (§12.1).
2. The kernel's reads use mode A0h and check each sector's header; `CdReadSector` too, where psx-spx gives 80h (§12.1).
3. A card request is carried out on its slot's vblank turn, by the waiting caller when nothing drives it (§13).
4. `_96_init` returns 1; `_card_chan` gives the port of the request completed last; the async CD read pauses after its
   count; `SetCdromIrqAutoAbort` keeps its flags only.
5. Names on the CD compared without case and without the version when the pattern gives none; the disk id a sum over the
   path table's first sector; a CD read of any length from any position (§12.3).
6. A new card file takes fresh blocks before deleted ones, chained in block order; the kernel never adds a broken
   sector; a direntry's sector for a card file is its first block's (§14).
7. SYSTEM.CNF's keys without case or blanks; the argument to the end of the line; GetID retried through the spin-up
   (§15).
8. The shell's own stack; the warm boot's settings carried in the scratchpad with a marker in k1 (§15).
9. The state at the boot program's entry beyond Exec's documented registers: SR 401h, I_MASK and I_STAT clear (§15).
10. The screen's layout, words, colours and font, and the manager's keys (§16).

### 20.2 Questions for the tester

1. **Q1, the screened sections.** `kernelbios.md`'s "BIOS Internal Boot Functions", "BIOS More Internal Functions" and
   "BIOS Versions" were not opened, as `DOCS.txt` still lists them for screening; the first two describe `InstallDevices`
   and the device functions' internals that §12.3 and §14 had to choose. *Recommended*: have them screened, and read the
   cleared parts at the next kernel step.
2. **Q2, Spyro's CdRead retries.** On the open kernel Spyro's library reports sector errors about 400 times while it
   still reaches its title. Whether it does on the player's BIOS is not in the record; Spyro's own reads go through its
   own handler and DMA, which this kernel leaves alone (§12.1). *Recommended*: the separate step with the player's BIOS
   counts the same lines; if they are absent there, a dispute is opened from D-41 on the controller's side, the kernel's
   being idle while they occur.
3. **Q3, Ape Escape's DUALSHOCK screen.** On the open kernel the game leaves the screen by itself at frame 1,500; on the
   player's BIOS it was recorded waiting there. The game replaces the kernel's pad and card element with its own, so
   neither kernel reads its pads afterwards [M, §18.2]. *Recommended*: the same step compares the two kernels' state at
   the game's entry (I_MASK, the chains, the events), outputs only.
4. **Q4, the exposure of §21.** One passage of `cdromdrive.md` lists the BIOS's own register accesses after an INT1; it
   was read before it was recognised. *Recommended*: add `cdromdrive.md` lines 714-747 to `DOCS.txt`'s exclusions for
   replacement writers.

### 20.3 The accuracy cost

In a player's words (the plan's §5.9, and `v1.rs`'s firmware entry since this stage): "Most games start and save as on a
console. A game that relies on an undocumented kernel call, reads the kernel's code, or depends on its exact timing may
behave differently, and the start-up screen is EmuSen's own. Games that use the console's own text characters (mostly
Japanese ones) show no text where they use them, as EmuSen does not carry Sony's fonts." Phase 1's sentence, that discs
and cards needed the player's BIOS, is withdrawn. The risks of `Diamond_OpenKernel.md` §8 stand, and phase 2 adds: card
I/O inside the vblank handler, about 240,000 cycles a sector; the CD handler that leaves interrupts it does not own to
the program; games that patch the card functions' code, which psx-spx's excluded section is said to describe and which
on this kernel find ROM, not RAM, at the tables' entries.

### 20.4 The image, and what the cross-kernel step needs

[M] 524,288 bytes, 92,440 of them before the last non-zero byte; the kernel's variables end at 8E70h, below DF80h; SHA-256
`0efba49b4d1bf011a5b59ce580e7fc29143ac68a0155012952c1791c72b8ef3f`, rebuilt byte for byte with
`DIAMOND_KERNEL_REBUILD=require`, and from a copy of `kernel/` at another path to the same digest.

**Left to a separate step**, by the stage's terms: the cards exchanged with the player's BIOS, and `firmwarecheck.py`.
The cards it needs are in `~/.cache/emusen/probe/diamond/cards/s10/` with a `README.txt`: `tekken3-saved-on-open-kernel.mcr`
(MD5 `cd4e7b491380fcc67d7ea4bd5da7dc6c`), Tekken 3's save made on this kernel on a blank card, with the presses that made
it, and `formatted-by-open-kernel.mcr` (MD5 `b57f32efd7912ebd349578d35ee7c565`), a blank card formatted by the manager.
The step loads the first in Tekken 3's Card Load on the player's BIOS, lets the same game save to a blank card on the
player's BIOS and loads that on this kernel, and gives the formatted card to the player's BIOS's own card manager; the
run tool's `--card`, `--card2` and `--press` serve it. It is also where Q2 and Q3 are looked at.

## 21. Reading log, and exposures

Every document and file opened for phase 2, 2026-10-11. **Never opened**: anything under
`~/.cache/emusen/probe/diamond/sysdir/` or `~/Documents/BIOS/`, any BIOS image, listing, disassembly or trace, OpenBIOS,
any emulator's source, PSn00bSDK's or PsyQ's libraries, ps1-tests' `common/`, `Diamond_References.md`, psx-spx's "BIOS
Patches" (`kernelbios.md` 2741-3333), its lines 147-152 and 2490-2503, `memorycontrol.md`, and the three sections still to
be screened (`kernelbios.md` 2229-2317 and 2647-2740). No reference emulator was run, and Diamond never with a firmware
folder.

**psx-spx** at `6d7d1bc`:

- `kernelbios.md`: its heading list (which prints the excluded and the unscreened sections' headings by name, their
  bodies unread); lines 32-146 (overview, memory map, table of tables, the garbage area's first lines), 160-176
  (parameters and registers), 270-364 and 430-503 (the A and B function summaries' CD, card and later rows, and the C
  functions), 542-1113 (file functions, execute, CDROM and memory card functions), 1114-1215 (interrupt handling, the
  priority chains, SysEnqIntRP and SysDeqIntRP), 1336-1483 (event classes, specs, modes and the event summary),
  2175-2228 (exit, `_boot`, the "no function" groups), 2570-2646 (the control blocks).
- `controllersandmemorycards.md` lines 2352-2735 (the card's commands, FLAG, timings, data format, images, notes).
- `cdromfileformats.md` lines 128-218 (the PS-EXE and SYSTEM.CNF).
- `cdromdrive.md`: its heading list; lines 29-226 (the host interface), 280-347 (BUSYSTS, Misc, "To init the CD",
  seek-busy), 500-640 (control and seek commands), 685-870 (read commands, the buffer, ReadTOC, "Setloc, Read, Pause",
  the status byte). **Exposure**: lines 714-747, inside the ReadN section, list the register accesses "the Kernel does"
  after an INT1, a trace of the BIOS's own I/O; they were read in sequence before they were recognised. Nothing was taken
  from them: the kernel's sector transfer follows the RDDATA and HCHPCTL descriptions and the DMA chapter (§12.1), and
  its enabling of DPCR's channel 3 is argued from the documented DMA events and functions; the passage is §20.2 Q4.
- `dmachannels.md` lines 1-172 (the registers, DPCR, DICR, the usual CHCR values).
- `graphicsprocessingunitgpu.md`: a search of its headings; lines 265-300 (rectangles), 376-404 (GP0(E1h)), 436-460
  (the drawing area and offset), 600-760 (the GP1 commands and display ranges).
- `DOCS.txt`, the exclusions, read first.

**ECMA-119**, 2nd edition, December 1987, as text by `pdftotext`: its contents and §9.1 (the directory record), with §8.4's
field positions from its contents and §6.8.1.1 found by a search.

**EmuSen's own**: `Diamond_Plan.md` §1.3, §5.9, §7 and §8; `Diamond_OpenKernel.md` whole; `Diamond_CDROM.md` whole
(including its exposure statement, which describes another writer's observation of a BIOS trace, not read here);
`Diamond_Pad.md` whole; `Diamond_Disputes.md`'s rules and headings and D-29; `Diamond_MDEC.md` §9; `Diamond_Native.md`'s
headings and §16; `Diamond_SPU.md`'s headings and a search of §6.3 and §11 for the games; the Diamond crate's sources
named here (`kernel.rs`, `machine.rs`, `examples/run.rs`, `tests/programs.rs`, `programs/cd-kernel.s`, parts of `v1.rs`,
`bus.rs`, `card.rs`, `sio0.rs`, `irq.rs`, `media.rs`, `exe.rs`, `gpu/mod.rs`'s status and FIFO, and `cdrom/commands.rs`
lines 100-200), the kernel and its harness whole, `emusen-mips`'s `asm.rs` (its encoder), `emusen-disc`'s signatures, and
WiseMan's `DiamondTests.cs` (a search for its firmware variable).

**Observed, the games as black boxes**: the calls they made by number; their pictures; RAM dumps of the kernel's own
structures while they ran (the EvCBs, the ExCBs, the card and pad variables); Spyro's own CD traffic through the
controller's trace and, for one run, the DMA registers it wrote (a print added to the bus for that run and removed).
