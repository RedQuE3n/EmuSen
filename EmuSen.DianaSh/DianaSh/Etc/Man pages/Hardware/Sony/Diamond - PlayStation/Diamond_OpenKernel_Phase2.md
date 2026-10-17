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
on this kernel only, with no firmware folder. §21 lists every document opened and states the exposures. §22, written
afterwards by another writer, records the cross-kernel step with the player's BIOS, for this kernel's writer.

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
bit 2 (restored on release when it was clear; *since §26.3* only for an asynchronous operation, a synchronous one
masking it), and its element at priority 0, the first function of
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

*Since 2026-10-17 (`Diamond_Shell.md` §1.3, §11.3) the screen and the manager are not the kernel's: they are the shell,
a PS-EXE of its own (`kernel-shell/`) that the kernel carries in its ROM and starts by Exec under hand-over v1, reaching
the kernel only through its call tables; the font and the GPU's primitives are the shared crate `diamond-guest`. What
this section describes, the shell does unchanged until its step S4.*

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
| Spyro the Dragon | frame 229 | its title screen by frame 3,600; its library prints "CdRead: sector error" and "retry" about 400 times from frame 740 (§20 Q2) | its title screen by frame 3,300; *since §22.3*, the same 402 retries |
| Ape Escape | frame 274 | its DUALSHOCK screen, then black at 1,500 and white from 1,800, the Analog button not pressed (§20 Q3) | ~~waits at its DUALSHOCK screen~~ *§22.4*: the same as on the open kernel |
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
kernel carries none, and ~~returns -1~~ *since §25.1* answers the full-width Latin letters, digits and punctuation from
its own 5x8 font, a blank space and a box for every other documented code), 346 no function as documented, 6 undocumented. No "phase 2 stub" is left.

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
   being idle while they occur. *Answered 2026-10-11 (§22.3)*: the game prints the same 402 pairs on the player's BIOS,
   and its CD commands are the same; not the kernel's, so no dispute was opened. *Settled 2026-10-12*
   (`Diamond_Disputes.md` D-43): the DMA's arbitration, which kept a channel waiting behind a list between its nodes;
   with it changed the game prints no retry in 3,600 frames on either kernel and reaches its demo, as the referees do.
3. **Q3, Ape Escape's DUALSHOCK screen.** On the open kernel the game leaves the screen by itself at frame 1,500; on the
   player's BIOS it was recorded waiting there. The game replaces the kernel's pad and card element with its own, so
   neither kernel reads its pads afterwards [M, §18.2]. *Recommended*: the same step compares the two kernels' state at
   the game's entry (I_MASK, the chains, the events), outputs only. *Answered 2026-10-11 (§22.4)*: with the present
   build the game leaves the screen by itself on the player's BIOS too, its pad exchanges the same on both kernels;
   the record of its waiting predates stages 8 and 9. The kernels' structures were not compared, only what the game
   sees (§22.5).
4. **Q4, the exposure of §21.** One passage of `cdromdrive.md` lists the BIOS's own register accesses after an INT1; it
   was read before it was recognised. *Recommended*: add `cdromdrive.md` lines 714-747 to `DOCS.txt`'s exclusions for
   replacement writers.
5. **Q5, `firmwarecheck.py`'s thresholds for a kernel-sized replacement** (§22.6). By the defaults the image fails on
   its zero fill and forced idioms. *Recommended*: thresholds for this image decided before it is measured again, with
   runs of one repeated byte excluded or a `--forced` image of the idioms, before G9 is counted. *Answered 2026-10-11*:
   fill excluded and the image judged against a control of unrelated compiled code; **PASS on all 18 images**
   (`EmuSen_Firmware.md` §0 gives the rule and the numbers).

### 20.3 The accuracy cost

In a player's words (the plan's §5.9, and `v1.rs`'s firmware entry since this stage): "Most games start and save as on a
console. A game that relies on an undocumented kernel call, reads the kernel's code, or depends on its exact timing may
behave differently, and the start-up screen is EmuSen's own. Games that use the console's own text characters (mostly
Japanese ones) show ~~no text~~ *since §25.1* the Latin letters, digits and common punctuation in EmuSen's own font
and a box for every other character where they use them, as EmuSen does not carry Sony's fonts." Phase 1's sentence, that discs
and cards needed the player's BIOS, is withdrawn. The risks of `Diamond_OpenKernel.md` §8 stand, and phase 2 adds: card
I/O inside the vblank handler, about 240,000 cycles a sector; the CD handler that leaves interrupts it does not own to
the program; games that patch the card functions' code, which psx-spx's excluded section is said to describe and which
on this kernel find ROM, not RAM, at the tables' entries (*since §25.3* the exception path runs from the kernel's RAM,
while the tables and C(06h) still name its ROM copies; *since §27* a game that stores and calls behind B(5Bh)'s entry
finds fill that returns, and behind any other entry the kernel's other code).

### 20.4 The image, and what the cross-kernel step needs

[M] 524,288 bytes, 92,440 of them before the last non-zero byte; the kernel's variables end at 8E70h, below DF80h; SHA-256
`0efba49b4d1bf011a5b59ce580e7fc29143ac68a0155012952c1791c72b8ef3f`, rebuilt byte for byte with
`DIAMOND_KERNEL_REBUILD=require`, and from a copy of `kernel/` at another path to the same digest. *Since §23 (2026-10-11):* SHA-256
`7af83c9be629827e850b4ce450e0d2fcf3400e428923ea2b676918393f5ea1b4`. *Since §25 (2026-10-12):* SHA-256
~~`593817b8fe850e30685a7feb6a40f6f3c474c519c7485624f441457566e66289`~~
`4e01c2ac1b8582e46fd002363f3a87f145fe96058c80fdc67222d5a533cfeb7b`, the variables ending at A310h. *Since §27
(2026-10-12):* 103,256 bytes before the last non-zero byte, SHA-256
`c1a002e88aa4e8b1be0257ca486bdb8309cd63f229e1f5070e36e4c72559c785`.

**Left to a separate step**, by the stage's terms, and taken in §22: the cards exchanged with the player's BIOS, and `firmwarecheck.py`.
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

## 22. The cross-kernel step (2026-10-11)

*Measured 2026-10-11, after stage 10's merge, by a writer who writes no kernel code.* This section closes the two items
§20.4 left to a separate step, the cards exchanged with the player's BIOS and `firmwarecheck.py`, and looks at Q2 and
Q3. **It is written for the kernel's writer**: it states only what a game sees at the documented interface (its
screen, its own text, its own RAM, its own accesses to the hardware registers, its kernel calls by number with their
arguments and documented results) and what the cards hold. Nothing here describes the player's BIOS's code, its
internal state or its own I/O; where a difference could only be explained by them, it says so and stops.

**How the player's BIOS was used.** Only as Diamond's firmware: the core's declared American file,
given to the run tool's `--bios`, and as `firmwarecheck.py`'s input (§22.5). Its intro does not boot a disc in
Diamond (D-29, open), so each game was started on it through the kernel's own `LoadExec` (the run tool's `--boot`,
`Diamond_CDROM.md` §9.4), and on the open kernel by its own SYSTEM.CNF boot (§15). The BIOS's own TTY was never read:
the run tool's output was discarded on its runs, and what a game prints was taken from the game's own calls (below).

**The run tool's `--game`**, added for this step (`examples/run.rs`), observes the game alone, from the first
instruction at its PS-EXE entry on, identically on both kernels: a call is counted only when it enters A0h, B0h or C0h
with RA in user RAM (1_0000h-1F_FFFFh), so that a kernel's calls to itself are never seen; its result is v0 when the
CPU comes back to that RA; the game's text is the bytes it passes to `write(1, ...)` (A(03h), B(35h)) and to putchar
(A(3Ch), B(3Dh)), read from its own arguments; and `--game-log` records the loads and stores of code in user RAM at the
CD-ROM controller, SIO0, the DMA and the interrupt registers, with the values read and written. A kernel's own code
lives below 1_0000h or in the ROM and is never decoded.

### 22.1 The cards, both ways

Tekken 3 (the tester's copy), with the run tool's `--card` and `--press` [M, pictures every 20 frames]. The path through
the game's menus is the README's (§20.4): Option Mode, Memory Card, then Card Save or Card Load and Start; on the
player's BIOS the presses are 60 frames later, its title appearing about 20 frames after the open kernel's.

| Card in slot 1 | Kernel | What the game shows |
|---|---|---|
| `tekken3-saved-on-open-kernel.mcr` (§20.4) | the player's BIOS | Card Load: **"LOAD OK!"** |
| a blank card (zeros) | the player's BIOS | Card Save: "Initialize memory card?", Start: **"SAVE OK!"** (the card kept as `tekken3-saved-on-bios.mcr`) |
| `tekken3-saved-on-bios.mcr` | the open kernel | Card Load: **"LOAD OK!"** |
| `formatted-by-open-kernel.mcr` (§20.4) | the player's BIOS | Card Load: **"NO TEKKEN 3 FILE!"**; Card Save: **"SAVE OK!" with no "Initialize memory card?"** |
| a blank card (zeros), the negative control | either kernel | Card Load: "LOAD ERROR!" |

Each kernel therefore reads the other's save, and the player's BIOS takes the open kernel's freshly formatted card for
a formatted card with no file, as the same game's three distinct answers show. **The BIOS's own card manager was not
reached**: with the drive empty and the formatted card in slot 1, the player's BIOS shows its first intro screen from
frame 360 to frame 2,400 and goes no further, as D-29 records for every revision. The formatted card's validity on that
BIOS is shown instead through its file functions, by Tekken 3, as above. The cards live in
`~/.cache/emusen/probe/diamond/xkernel/cards/`, outside the repository, with the run tool's presses beside the
README's.

### 22.2 The two kernels' cards, compared by the on-card format

By psx-spx's "Memory Card Data Format" [R], with a script over the files [M]; a card is the game's and the kernels'
data, not firmware.

- **The same**: the header frame (frame 0, "MC", zeros, checksum 0Eh); the directory frames 1-15 of the same save, byte
  for byte (block 1 in state 51h, size 2000h, link FFFFh, "BASLUS-00402TEKKEN-3", checksum; frames 2-15 free, A0h, link
  FFFFh, each with its checksum); the save's title frame "SC" and its three icon frames. The save's data frames (block
  1, frames 4-7) differ in 512 bytes, which is the game's own content from two different runs.
- **The broken sector list** (frames 16-35): FFFFFFFFh and checksum 00h on both; the player's BIOS's format writes bytes
  08h-09h as FFFFh, the open kernel's 00h. psx-spx: "Garbage (usually 00h-filled) (some cards have [08h..09h]=FFFFh)".
- **Frames 36-62**: the open kernel's format writes FFh ("usually FFh-filled"). The player's BIOS's format leaves them as
  they were: 00h on a blank card of zeros, 55h on a card filled with 55h before Tekken 3 formatted it, where the open
  kernel's format left no frame of block 0 at 55h [M]. On both, the save leaves 952 of blocks 1-15's 960 frames
  untouched; the game writes eight.
- **Frame 63**, the "Write Test Frame" ("Usually same as Block 0"): the open kernel's format writes a copy of the
  header; on the player's BIOS's cards it is not a copy of the header (its contents are not recorded here). A load
  changed frame 63 and no other frame, on either kernel.
- **A defect of the open kernel's write test** [M]: given a blank card of zeros, Tekken 3's Card Load on the open
  kernel leaves frame 63 holding not the header frame `card.rs`'s `test_frame` builds (§13) but 128 other bytes, whose
  first three words are the three the kernel keeps at 80h (its exception vector's jump) and the rest zeros but for
  02h at 60h and FFh at 68h; given a formatted card, the same load leaves the header copy. No game reads frame 63
  through the file functions, so nothing shows it; *for the writer*: the frame a write test sends on a new card.

No game reads these frames through the file functions, and both kernels' saves load on both. **Nothing for the writer
to change** to keep the exchange working. Should a game read block 0 raw, frames 36-62 and 08h-09h of frames 16-35 are
the two places where the kernels' formats differ, each within what psx-spx describes.

### 22.3 Q2: Spyro's "CdRead: sector error"

[M, both kernels to frame 3,600.] **The game prints the same on both kernels**: "ResetGraph:jtb=...", "CD_init:addr=..."
twice, and **402** pairs of "CdRead: sector error" and "CdRead: retry...", the first pair 514 frames after its entry
on the open kernel (entry at frame 229) and 517 on the player's BIOS (entry at frame 291). It writes them one byte a
call, 16,164 calls to B(35h) on each. Its CD-ROM commands and their parameters, from its own accesses to the controller
over the 700 frames after its entry, are the same sequence on both (183 and 182 commands, the last a Pause at the
window's edge); each sector it takes by its own handler: the three-word header by DMA channel 3, then 200h words, and
after a sector it rejects, Pause (09h) and a new read.

So Q2's condition for a dispute, that the lines be absent on the player's BIOS, is not met: **the retries are not the
kernel's**, and §20.2's Q2 needs no change to the kernel. Why the game rejects those sectors lies on the controller's or
the copied disc's side and is left to a CD-ROM step; no dispute number was taken.

### 22.4 Q3: Ape Escape's DUALSHOCK screen

[M, both kernels to frame 2,400, pictures every 60 frames.] **The game behaves the same on both kernels**: "Sony Computer
Entertainment America Presents", its "The DUALSHOCK analog controller is required to play this game." screen from about
630 frames after its entry to about 990, black, then white from about 1,405 frames after its entry, on the open kernel
(entry at 273) and on the player's BIOS (entry at 335) alike; the open kernel booted through `LoadExec` (entry at 250)
does the same. The record's "waits at its DUALSHOCK screen" on the player's BIOS (§18.1) was taken before stages 8
and 9 and with the Analog button pressed; with the present build it does not hold.

Its pad traffic, read from its own accesses to SIO0, is the same exchange for exchange on both: digital reads (42h:
41h 5Ah FFh FFh), the configuration of the DualShock (43h, 45h, 4Ch, 47h, 46h, and 43h 00h to leave it), then, 295
frames after its entry on the open kernel and 298 on the player's BIOS, 44h 01h 03h (analog mode, locked) and analog
reads (42h: 73h 5Ah FFh FFh 80h 80h 80h 80h) thereafter, port 2 answering FFh. The only difference is how many idle
digital reads precede the switch, three frames' worth, which is timing. The black and the white that follow are
therefore not the kernel's either; they are left to the steps that own the GPU and the MDEC.

### 22.5 The kernel calls, compared

The one-time calls the two games make in their first frames, side by side with their arguments and results [M,
`--game-log`]. The calls, their order and their arguments are the same on both kernels, but for register contents a
kernel leaves behind in a0-a3, which the documents do not define and which are not recorded. Where the documents give
a result, two differ:

- **C(0Ah) `ChangeClearRCnt(3, 0)`**, the first call of each game: **returns 1 on the player's BIOS and 0 on the open
  kernel** (the later calls return 0 on both). psx-spx: "The function returns the old (previous) flag value". So at a
  booted game's entry the vblank's flag reads 1 on the player's BIOS. *For the writer*: a game that restores the flag
  it was given would leave vblank IRQs acknowledged by the kernel on one kernel and not on the other; neither game
  examined does so. Both games were booted on the player's BIOS through `LoadExec`, its own boot being out of reach
  (D-29), so the value at the end of a console's own boot is not known.
- **B(08h) `OpenEvent`**: the games' first descriptor is **F1000000h on the player's BIOS and F1000005h on the open
  kernel**, after the games' own A(72h) `_96_remove`, then consecutive on both. psx-spx's B(08h) entry says five of the
  events are "internally used by the BIOS for CDROM functions", which the open kernel's five CD events (§12.2) follow;
  why none is in use on the player's BIOS at that point is **unexplained at the interface** (whether `_96_remove` or
  the boot path frees them is not visible to a game). Both games use the descriptors only as handles, and behave the
  same.

Results the documents do not define (B(19h), B(4Bh), B(5Bh), A(39h), A(3Fh), A(44h), A(72h), A(30h)) differ between the
kernels and are not recorded; so do the addresses B(56h) and B(57h) return, which the documents do not give. No game
examined depends on any of them.

### 22.6 `firmwarecheck.py`

Run as its docstring and `EmuSen_Firmware.md` §0 give it, with the default thresholds (at most 25% equal at the same
offset, no common run over 6 bytes), on `kernel/diamond-kernel.bin` (SHA-256 `0efba49b...`) against each of the 18
images in the firmware folder, the core's three declared files and fifteen others [M]. Its printed numbers only:

| Original | Equal at the same offset | Longest aligned run | Longest common run | Verdict |
|---|---|---|---|---|
| the declared Japanese file | 114,559 of 524,288 (21.9%) | 5,245 | 6,610 bytes, replacement offset 92,440 | FAIL |
| the declared American file | 160,929 (30.7%) | 111,108 | 111,108 bytes, replacement offset 92,440 | FAIL |
| the declared European file | 116,361 (22.2%) | 18,071 | 18,071 bytes, replacement offset 92,440 | FAIL |
| the fifteen others | 20.3% to 31.0% | 656 to 114,386 | 656 to 114,386 bytes, each at replacement offset 92,440 | FAIL, all |

Every longest run starts at the replacement's offset 92,440, where its last non-zero byte has ended (§20.4): it is the
image's zero padding meeting zeros in the original. Run again on the image's first 92,440 bytes, the part that holds
anything [M]:

| Original | Equal at the same offset | Longest aligned run | Longest common run | Verdict |
|---|---|---|---|---|
| each of the 18 | 12,669 to 14,135 of 92,440 (13.7% to 15.3%) | 113 at offset 143 | 200 bytes, replacement offset 56 | FAIL, all |

The equal share is within the threshold; the runs are not. The replacement's bytes at the 200-byte run are its own
zeros between its vectors (offsets 56-255), and among the runs the tool lists (it stops listing at its pair budget, so
the list is not complete) every run longer than 8 bytes lies in a run of the replacement's zero bytes, extended by at
most one byte at an edge; the three others of 8 bytes are a jump through a register with its delay-slot nop, twice,
and the two instructions that set SP to 801FFF00h, the reset path's value (§15). [A] These are forced by the
instruction set and the interface rather than taken from an image; but the policy's verdict is the tool's, and
**the verdict is FAIL**, plain and on the content alone.

The thresholds were fixed for VenusRT's 64-byte boot program (`EmuSen_Debugging_Tools_Reference_v5.md` §3.61), where
a run of six caught the case that mattered. For a 92 KB kernel of MIPS code the same rule cannot pass: two nops are 8
bytes, and so is `jr ra` with the nop in its delay slot. **Q5 for the tester**: whether a
kernel-sized replacement is measured against thresholds decided for it beforehand, for instance with runs of a single
repeated byte excluded, or against `--forced` with a generator of the forced idioms, before G9 is counted. ~~Until
then the gate's `firmwarecheck.py` item is **not met**.~~ *Decided 2026-10-11*: judged against a control (Q5's answer,
`EmuSen_Firmware.md` §0); **PASS on all 18 images**, and the item is **met**.

### 22.7 What this means for the gate

The stage 10 row's card item is **met** both ways (§22.1); `firmwarecheck.py` is **not met** by the policy's
thresholds (§22.6, Q5). **G9's open-BIOS condition is therefore not met** in full: every game claimed boots with no
firmware folder and runs on the player's image too, and their cards cross, but the repetition check fails by its
letter. Q2 and Q3 close with no change to the kernel: both games behave the same on the two kernels at every point
observed, and the two differences of documented results (§22.5) are for the writer to weigh.

### 22.8 Reading log, and exposures

Opened for this step, 2026-10-11: this page whole; `Diamond_Plan.md` §5.9, §7 and §8; `Diamond_Native.md` §18;
`Diamond_Pad.md` §8 and §11; `Diamond_CDROM.md` §9.4-§9.5; `Diamond_Disputes.md`'s headings and D-29; `EmuSen_Firmware.md`
§0; `EmuSen_Debugging_Tools_Reference_v5.md` §3.61 and `VenusRT_Native.md` §35 (how a firmwarecheck result is
recorded); `firmwarecheck.py`'s docstring; `DOCS.txt`; the cards' README; psx-spx `6d7d1bc`
`controllersandmemorycards.md` lines 2466-2540 (the card's data format) and `kernelbios.md`'s entries for B(08h),
B(4Bh), C(03h), C(0Ah) and B(56h)/B(57h) (lines 1020-1028, 1196-1210, 1277-1300, 1590-1600, 2096-2104) and its
function summary for names; the run tool's source. No emulator's source, OpenBIOS, BIOS listing or trace; no
reference emulator was run.

The player's BIOS was observed only as the screen, the games after its hand-over by the means of §22, the cards it
wrote, and `firmwarecheck.py`'s numbers. **Exposures**, stated plainly:

- A card comparison script printed frame 63 of the cards the player's BIOS had written, twice, as hexadecimal (about
  a dozen non-zero bytes), before it was recognised that a write-test frame may carry the writing kernel's memory, as
  the open kernel's does (§22.2). They were not studied further and are not recorded here; §22.2 says only that the
  frame is not a header copy.
- A search of `kernelbios.md` for function names printed eight lines of hits from inside "BIOS Patches" (lines 2741-3333,
  excluded for replacement writers by `DOCS.txt`), each a single line naming B(56h); nothing from them is used here.
  The search was then confined to lines before 2741.
- The call comparison printed the player's BIOS's undefined results and the argument registers it leaves behind
  between calls, a few dozen words. None is recorded; §22.5 names only which calls differ.

## 23. After the cross-kernel step (2026-10-11)

*Written by the kernel's writer, under the same rules; of §22 only its text was read.*

### 23.1 Frame 63, and the half-stored card request

**What frame 63 must hold.** psx-spx's "Write Test Frame (Block 0, Frame 63)": "Reportedly "write test". Usually same as
Block 0 ("MC", 253 zero-bytes, plus checksum 0Eh)" [R]. `card.rs`'s `test_frame` builds exactly that, and the format
and the directory read write it; the contents were never the defect.

**The defect** [M]. `submit` stored a slot's request field by field with interrupts on, and the vblank element, which
carries the request out, could run between two of the stores: it then took a request whose kind and sector were
already the new ones and whose buffer was not, or cleared the slot under the stores still to come. A write test so
sent sector 3Fh from address 0, where the kernel keeps its copy of the exception vector (the 80h words, then 02h at 60h
and FFh at 68h), which is what §22.2 found on frame 63; a request cleared under the stores was lost. Tekken 3's write
test on a new card hit the window; the stage's own tests, whose writes followed a vblank by most of a frame, did not.

**Shown first, then fixed.** `programs/card-write-race.s` makes one `_card_write` of sector 3Fh with interrupts on;
`a_vblank_inside_card_write_never_sends_a_half_stored_request` finds the instructions of the call that store into the
kernel's RAM and raises vblank at each instruction from 40 before them to 40 after, on both ports, checking frame 3Fh
for the call's bytes each time. On the stage 10 image it fails on port 10h at 1,488 instructions into the call, the
frame left zero; with `submit` storing the request inside a critical section, and only once the slot is free, it
passes at every instruction on both ports. The element's own chaining of a job's next request runs at interrupt level
and is not affected.

### 23.2 The two call results of §22.5

- **ChangeClearRCnt(3, 0)'s first result.** psx-spx gives what the call returns ("the old (previous) flag value") and
  not the flag a booted program finds; its "Vblank" section says vblank "is internally used by the BIOS for handling
  joypad and memory card accesses", which the priority 2 element does, and a vblank flag of 1 would make the priority 1
  handler acknowledge vblank and return before that element ever saw it [R "C(0Ah)", "Vblank", "Priority Chains"]. The
  documents therefore do not settle it, and `Diamond_OpenKernel.md` §9 item 4's default, 0, which the argument above
  supports, is kept. Recorded, not changed.
- **OpenEvent's first descriptor after `_96_remove`.** psx-spx's B(08h): "of the 16 events, only 11 events are available
  to the game", five "internally used by the BIOS for CDROM functions" [R]. The open kernel's five CD events keep that
  count; that the other kernel's first descriptor is F1000000h after `_96_remove` is not explained by any document read,
  and `_96_remove`'s entry describes only its chain element. Recorded, not changed.

### 23.3 The image and the tests

[M] 524,288 bytes, 92,504 of them before the last non-zero byte; SHA-256
`7af83c9be629827e850b4ce450e0d2fcf3400e428923ea2b676918393f5ea1b4`, rebuilt byte for byte with
`DIAMOND_KERNEL_REBUILD=require`. The kernel harness, the crate's tests with and without the ps1-tests corpus, and
WiseMan's `DiamondTests` with no firmware variable set, all passing (`Diamond_Native.md` §18.5).

**Read for this section**: psx-spx `kernelbios.md` lines 1277-1296 (OpenEvent) and 1545-1597 (the timer functions and
ChangeClearRCnt); `controllersandmemorycards.md`'s "Write Test Frame" again; this page's §22. Nothing else was opened,
no firmware folder was used, and no reference emulator was run.

## 24. Found in play, for the kernel's writer (2026-10-12)

*Fixed the same day: §25.* Three of the tester's games, driven into play on this kernel (`Diamond_Players.md` §13), meet the kernel at points
this page's design left open. Nothing in `kernel/` was changed; each is recorded with the game's own evidence (its code
and its calls, read from RAM; no firmware was read or run on Diamond), for a replacement writer's step.

1. **B(51h) Krom2RawAdd** (§19, "absent from an open kernel"). Castlevania: Symphony of the Night asks it for the
   Shift-JIS codes 8163h and 819Ah when it draws its file screen, takes the −1 it returns as a pointer, and reads a
   byte at FFFFFFFFh; the bus error never returns, and the game stops at a black screen after PRESS START BUTTON
   [M, `examples/run --game --game-log`, its code at 80106A30h–80106AC4h]. The game draws its English menus with the
   call, so "no text where they use them" (§19's player's words) is a stop for this game, not a blank. A kernel that
   carries no characters could still answer with the address of a blank glyph; whether that is a decision the plan's
   §5.9 allows is the writer's question.
2. **B(34h) read and B(35h) write in asynchronous mode** (§14). Spyro the Dragon opens its save asynchronously (access
   mode 8001h, after creating it with 10200h), then calls read or write in a loop that ends only when the call returns 0 (its code at
   80067018h–8006702Ch and 800672D4h–800672E8h). This kernel returns the length at once, as psx-spx's "Number of bytes
   actually read" reads; the game's second call then waits for the first job and returns C00h, and every call after it
   −1, so "CREATING SAVE FILE…" and "ACCESSING MEMORY CARD…" stand for ever [M]. The loop is the game's evidence that
   an accepted asynchronous request returns 0; psx-spx does not say what it returns.
3. **The time an interrupt takes** (§4.3's elements). In Crash Bandicoot's play, timer 2's IRQ, at its target every
   32,776 cycles, is acknowledged 5,800 to 8,800 cycles after its exception, and a vblank reaches the game's own handler
   36,434 cycles after its exception: 8,052 before the pads' element, 9,253 for port 1's five bytes, 9,800 for the empty
   port 2's timeout and 9,191 after [M, the kernel's stores to I_STAT and SIO0 timed at frame 6,660]. Being longer than
   timer 2's period, the vblank's path always finds timer 2 standing when the game's handler looks, and the game's
   handler, which counts such looks, prints "intr timeout(0040:004d)" at 2,049 and writes 0 to I_STAT. Nothing was seen
   to go wrong in that run; it is a cost of the kernel's paths, of which the port 2 timeout is the largest single part.

## 25. The three defects of §24, fixed (2026-10-12)

*Written by the kernel's writer under the replacement writers' rules of the plan's §1.3; of §24 and
`Diamond_Players.md` §13 only their text was read.* Each defect was shown by a test failing on the image before it, then
fixed, then taken back to the game; the marks are this page's, **[M]** measured on 2026-10-12. §25.6 lists what was read
and states the exposures.

### 25.1 Krom2RawAdd: a box for every character the kernel does not carry

**What the plan allows.** §5.9 has the open kernel carry nothing of Sony's, and §20.3's words for the player said that
games using the console's characters "show no text where they use them". What B(51h) answers in their place is not
decided anywhere [R the plan's §5.9, this page's §19 and §20.3]; it is **Q6** below.

**A blank glyph is not enough** [M]. With every documented code answered by one blank 16x15 glyph, Castlevania passes
the call but then never leaves its text routine: for each character other than its own space, the game shifts the glyph
left one column at a time while every row's first column is clear (its code at 801B2248h-801B22B4h), which ends only
on a set pixel. Its own fallback (80106A50h-80106AB4h) looks a code up in a short table of its own when the call answers
-1, and otherwise asks again for 819Ah, which is why -1 led to the read at FFFFFFFFh of §24.

**The answer** [C]. For every code of the two documented ranges, 8140h-84BEh (charset 2) and 889Fh-9872h (charset 3)
[R "BIOS Character Sets"], B(51h) returns a glyph in the kernel's ROM, at its KSEG1 address as the documents give the
character sets' (BFC6xxxxh): for 8140h, the ideographic space, a blank one; for every other code one box over columns
2-13 and rows 1-13, the conventional glyph for a missing character. Any other code answers -1, as documented. B(53h)
answers 0 for the space, 30 for the box (its offset within the kernel's two glyphs) and -1 otherwise. The glyph's form
(15 rows of two bytes, the left pixel in bit 7 of the first) was taken from Castlevania's own glyph table in its RAM at
800AC914h, whose first entry reads as a © in that order and in no other [M]. A rectangle is no font of the console's
and recreates none; `charset.rs` holds both glyphs.

**Shown** [M]. `krom2rawadd_answers_a_box_in_the_rom_for_every_documented_code_but_the_space` fails on §23's image (at
the first answer, -1) and on the blank-glyph image (at the box), and passes. Castlevania with the open kernel's formatted
card and its script: PRESS START BUTTON, the file screen with a row of boxes where its scrolling help text is drawn
through the call, NEW GAME, the name entry, "JOURNEY BACK TO 1792", "Now Loading" and Richter's stage from about frame
3,300 to the run's end at 9,000. The words for the player are now "show a box for each character where they use them"
(`v1.rs`'s firmware entry and §20.3).

**The kernel's own letters** (decided 2026-10-12, Q6 as recommended). The full-width Latin letters (8260h-8279h,
8281h-829Ah), digits (824Fh-8258h) and the punctuation of row 1 that the card manager already reads as ASCII (§16) are
drawn from the kernel's own 5x8 font of §16: each of its five columns doubled across into columns 3-12, its rows 0-6
doubled down and row 7 once, into rows 0-14. The shapes are that font's, scaled; nothing is drawn after a console's
characters. 8140h stays blank, every other code of the two ranges keeps the box, and any other code -1; `charset::ascii`
is now the one owner of the mapping, which the card manager calls too. The glyphs, 95 of them with the space and the
box, are built at compile time from the font into 2,850 bytes of the ROM; B(53h) answers each one's offset among them.
`krom2rawadd_draws_the_full_width_latin_from_the_kernels_font_and_a_box_for_the_rest` gives Ａ, １ and ！ row by row, the
box for 8141h, 8163h, 819Ah, 84BEh, 889Fh and 9872h, and Ｚ's and ｚ's own offsets; it fails on the box image
(`593817b8…`) at Ａ, and passes [M]. Castlevania's help line now reads "Select file to be loaded" and "Checking Memory
Card. Do not remove Memory Card", its kanji still boxes, and its text routine ends on every glyph: the game names its
file and plays its prologue and Richter's stage to frame 9,000 as before [M, pictures every 60 frames]. The words for
the player: games "show the Latin letters, digits and common punctuation in EmuSen's own font and a box for every other
character (mostly Japanese ones)".

### 25.2 Read and write in asynchronous mode

**What the documents say.** "Asynchronous access means that the BIOS function returns immediately after invoking the
access (which does then continue on interrupt level, and does return an event when finished)"; read's result is the
"Number of bytes actually read, -1 if failed" [R "Asynchronous Access", "A(02h) or B(34h)"]. Neither says what an
asynchronous call returns.

**The game's evidence** [M]. Spyro the Dragon's two loops (§24) are `while (read(fd, buf, len) != 0)` and the same
with write, each after `while (lseek(fd, pos, 0) != pos)`: the game takes any other result as a request not accepted
and asks again. So an accepted request returns 0, which also reads psx-spx's "actually read" literally: when the call
returns, nothing has been read yet [A].

**The fix.** An accepted asynchronous read or write returns 0 and moves the FCB's position past the bytes its job will
carry [C on the position]; a request refused for its length or position fails at once with -1 and its error, as in
synchronous mode; while a job runs on the slot, a new request waits for it and is then accepted (unchanged, [C]). The
job, its `DeliverEvent(fd, 4)` and the callbacks' F4000001h events are as §14 gives them, and the synchronous path is
unchanged.

**The harness's card.** `kernel-harness/src/sio0.rs` gives the harness SIO0's DATA, STAT and CTRL and a memory card in
slot 1 that answers psx-spx's "Memory Card Read/Write Commands" byte for byte, its FLAG 08h on insertion and cleared by
a write ("FLAG Byte"), each byte's /ACK raising IRQ7 400 of the rig's accesses later, the last byte none [R]. It models
the protocol, not the timing. `tests/card.rs`: an asynchronous write accepted with 0, the position past it, a second
write, close, the fd's event, and the file read back synchronously; a synchronous write read back asynchronously, with
every directory frame's checksum and the file's first block 51h; and the asynchronous calls refusing a part frame and
the file's end. The first two fail on the previous image (1024 for 0) and pass. Diamond's
`the_open_kernels_card_files_keep_the_on_card_format` now expects 0 from its asynchronous read, its event as before.

**Spyro the Dragon** [M], the open kernel's formatted card, its script to frame 18,000: "THE MEMORY CARD IN SLOT 1 DOES
NOT HAVE A SAVE FILE. CREATE SAVE FILE NOW?", Create File, the slot menu, a new game and the Artisans' world, two
dragons freed. The game creates `BASCUS-94228SPYRO` (10200h), writes its 1400h bytes asynchronously in one call
accepted with 0, closes it 80 frames later (40 sectors at two frames each, §13's pace), and reads the 1400h back
asynchronously, also accepted with 0. The card checks by psx-spx's format: the header, every checksum, block 1 in state
51h of 2000h bytes, its title frame "ＳＰＹＲＯ　ＴＨＥ　ＤＲＡＧＯＮ". Booted again with that card and the load script,
"ACCESSING MEMORY CARDS…" passes to the game's three slots, each EMPTY, as the script saved no game. **Not shown**: a
save of the game's progress, which the script never reaches.

### 25.3 The interrupt path's cost

**Measured first** [M]. The run tool's `--kernel-profile FILE --profile-frames A:B` (`examples/run.rs`) times every
exception from its entry at 80000080h to the return to the code it interrupted: per set of pending sources, the cycles
to the first instruction in user RAM, to the source's acknowledgement in I_STAT and to the return, and the cycles spent
at each PC outside user RAM. Crash Bandicoot by its script, frames 6,600-6,700, on §23's image:

| | to the game's code | to the acknowledgement | to the return |
|---|---|---|---|
| vblank (115) | 36,606 (5,647-39,126) | 36,804 | 43,796 |
| timer 2 (1,667) | 7,069 (3,856-14,513) | 7,644 (4,414-15,084) | 8,691 |
| syscalls (62) | — | — | 3,422 |

Medians, with the range. The kernel spent 185,658 cycles a frame in exceptions, a third of the frame. By PC, the cost
was the I-cache: the exception entry's first instruction of each 16-byte line took 128 cycles, its refill four words of
25 cycles each over the ROM's 8-bit bus (Diamond's DEV2 timing, `memctl.rs`), so the entry alone cost about 2,500
cycles an exception, the dispatch 21.9 cycles an instruction, and a game's code evicts the lines between interrupts. The
empty port's wait was 1,000 polls of the IRQ7 loop at 9.5 cycles each; the wait before acknowledging an old IRQ7 was 50
spins of 13.2 cycles, 660 cycles a byte.

**The exception path in the kernel's RAM.** psx-spx's RAM map places "Kernel Code/Data (relocated from ROM)" at 500h,
and its "BIOS Memory" says portions of the kernel "are relocated to the first 64K of RAM" for speed [R]. The linker
script's `.ramtext` now holds, at 80000500h-80001974h (5,236 bytes) and copied there with the variables at boot: the
exception entry, ReturnFromException and longjmp, the dispatch over the chains, the syscall, timer, vblank and default
handlers, DeliverEvent, the pads' element with its SIO0 exchange inlined, PAD_dr, and the CD element's first function;
`mem::call_far` calls the ROM from there through a register. The kernel's variables now end at A310h, below DF80h.

- **Every table entry and C(06h) still names a copy in the ROM** [M, C]. Crash Bandicoot, at its start, reads C(06h)
  through GetC0Table and writes 13 words into the handler it names (55Ch-590h in the first build, which put the handler
  in RAM), then calls FlushCache; the game stopped at once in SystemError. psx-spx: "reading C(06h) may be useful for
  patching the exception handler", and "most commercial games still include patches" for an old BIOS [R "C(06h)"]. On
  the ROM such writes do nothing, as with every earlier image. So the vector at 80000080h and the kernel's own exits use
  the copies in RAM, and C(06h), B(17h), A(14h), B(07h), B(16h) and A(90h) name the same code in the ROM;
  `reset.rs` checks that C(06h)'s copy and the vector's are the same bytes. A game that found the handler through the
  vector's own instructions would patch the RAM copy; none is known (**Q7**).
- **A jump cannot cross** [M]. The ROM, run at 9FC0xxxxh, and the RAM at 8000xxxxh lie in different 256 MiB regions,
  which J and JAL cannot cross, and the linker does not object: the first build's RAM code held `jal 8FC04900h`, the
  ROM's `card::perform` with its region's bits lost, once LLVM had turned a call through a constant pointer into a direct
  one. `every_jump_lands_in_the_part_of_the_kernel_it_lies_in` decodes every J and JAL of the ROM's code and of
  `.ramtext` in the rebuilt ELF and fails on that build at 80000AE4h; `call_far` hides the address from the optimiser.
- **Shown**: `a_vblank_runs_the_kernel_from_its_ram_and_fetches_nothing_from_the_rom` takes a vblank with the pads
  started from a loop in user RAM: 15,148 fetches from the ROM on the previous image, none now.

**An empty port given up 100 µs after its byte.** psx-spx: "The kernel's controller driver will time out if /ACK is
not pulled low by the device within 100 µs from the last SCK pulse", and a card's IRQ7 comes "circa 1500 cycles after
sending a byte (counted from the begin of the first bit)" [R "Controller and Memory Card Signals", "Timings"]; Diamond
gives /ACK 412 cycles after the last clock (D-22). `sio.rs`'s wait is now the byte's eight bits at BR 0088h and 3,387
cycles, at 9 cycles a poll (9.5 measured, taken low so that no wait ends early): 497 polls, about 4,700 cycles, three
times a device's latest /ACK. The wait before acknowledging an old IRQ7 is psx-spx's "100 cycles or so" ("Emulation
Note"): 8 spins, about 106 cycles. A card's late /ACK after a read's 5Ch byte keeps its own longer wait.

**After** [M], the same frames of Crash Bandicoot's script:

| | to the game's code | to the acknowledgement | to the return |
|---|---|---|---|
| vblank (109) | 15,881 (1,656-20,093) | 16,149 | 24,359 |
| timer 2 (1,716) | 1,296 (1,061-2,655) | 1,507 (1,261-3,104) | 1,781 |
| syscalls (100) | — | — | 774 |

The kernel spends 50,422 cycles a frame in exceptions, 9 % of it. Of a vblank, port 1's five bytes take about 7,500
cycles at the documented 1,500 a byte and the empty port about 4,700; the rest is the chains. `programs/kernel-irq.s`, a
pad on port 1, port 2 empty and timer 2 at Crash Bandicoot's 32,776 cycles from a loop in user RAM, shows it without a
game's code: a vblank returns after 23,351 cycles on §23's image and 13,730 now, timer 2 is acknowledged after 4,039 at
most and 1,107; `the_open_kernels_interrupt_path_fits_within_timer_2s_period` holds the bounds 16,000 and 2,000.

**Crash Bandicoot** by its script to frame 18,000 prints no "intr timeout", where it printed six, and plays the same
course by its pictures, to the hollow at about frame 6,600 where the camera rests to the run's end, as before.

**The cost to `timers`** [M]. ps1-tests' `timers` fails WiseMan's TTY gate on this image in two rows, and passes on
§23's. Both rows measure where an interrupt falls against the program's own loops, which the kernel's undocumented
timing decides:

- *Timer 0, system clock, 5,000 cycles*: the fifth sample reads 9,763, a vblank taken inside it. The program starts at a
  fixed time after reset, side-loaded at the hand-over: the hand-over now comes at 287,795 cycles, 44,578 later than
  before, the cost of copying `.ramtext` from the ROM, and the first row and its line take 111,673 cycles, 22,753 more
  (the ROM's code moved, and with it which lines share the I-cache). The row now runs from 530,715 to about 585,000 and
  meets the first vblank at 556,127; on §23's image it ended 40,000 cycles before it. Two to 32 spins added to every
  exception move the vblank to another sample of the same row.
- *Timer 1, hblank, a frame*: 262 in every sample against the log's 263. The program resets the counter a few dozen
  cycles after reading it, and the frame it counts is that much short of a frame; when the reset falls within those
  cycles after an hblank's edge, one edge is lost. Two spins added to every exception, or ReturnFromException's table
  entry pointed at the RAM copy, give 263.

The console's log is one draw of both phases, on a BIOS whose timing is not known. No change was made for them; it
is **Q8**. *Decided 2026-10-12, as recommended*: the gate drops one delay sample a row that is longer than its
tolerance allows, as one an interrupt landed in, and allows a frame's hblanks ±1, for every kernel image; the rule and
its reason are `Diamond_Native.md` §11.12a. With it `timers` passes on this image and on `7af83c9b…` [M].

### 25.4 The image and the tests

[M] 524,288 bytes, ~~97,288~~ 100,544 of them before the last non-zero byte; SHA-256
~~`593817b8fe850e30685a7feb6a40f6f3c474c519c7485624f441457566e66289`~~
`4e01c2ac1b8582e46fd002363f3a87f145fe96058c80fdc67222d5a533cfeb7b` since the kernel's letters (§25.1, Q6), rebuilt
byte for byte with `DIAMOND_KERNEL_REBUILD=require`. The images between: `afc27937…` after §25.1's box, `f277cce3…`
after §25.2 and `593817b8…` after §25.3.

- `cargo test --release` in `kernel-harness/`: 55 tests, all passing, with the rebuild required; new: the glyphs (§25.1),
  the card's three (§25.2), the jumps and the vblank from RAM (§25.3); changed: `reset.rs`'s vector and C(06h) and
  `exceptions.rs`'s hook structure, whose RA is now ReturnFromException's RAM copy.
- `cargo test --release` in Diamond's crate: 155 unit tests and 70 program tests, with and without
  `EMUSEN_DIAMOND_PS1TESTS`, the card round trips among them.
- WiseMan's `Diamond` tests under the test lock: ~~42 of 42 without the corpus, and with `EMUSEN_DIAMOND_PS1TESTS` set 41
  of 42, `timers` failing as §25.3 gives~~ *since Q8's rule* 43 of 43 with `EMUSEN_DIAMOND_PS1TESTS` set, the rule's own
  test among them, every program passing by its TTY; the gate's test on `7af83c9b…` passes `timers` too.
- The seven games by their scripts on the open kernel [M, pictures every 300 frames]: Castlevania past its file screen
  (§25.1), Spyro the Dragon's file created, written and read (§25.2), Crash Bandicoot without "intr timeout" (§25.3);
  Ape Escape to its first stage and the tutorial's boxes, Rayman to the Dream Forest, its save made on the card (block 2
  in use, block 1 its own erased first try), Silent Hill to the foggy street and the alley, Tekken 3 to its fights and
  "GAME OVER", each as `Diamond_Players.md` §13 records them.

### 25.5 Questions for the tester

6. **Q6, the characters.** *Decided 2026-10-12: the option, as recommended (§25.1).* Kept: a box for every documented code but the space. *Option*: the kernel's own 5x8 font of
   §16 drawn into the 16x15 cell for the full-width Latin letters, digits and common punctuation the card manager
   already converts (§16), the box for the rest, so that Castlevania's help line reads; it is EmuSen's own drawing,
   copies no console font, and costs about 2 KiB of ROM. *Recommended*: the option, if a font of EmuSen's own is
   acceptable in the kernel.
7. **Q7, the two copies.** *Decided 2026-10-12: kept, as recommended.* The tables and C(06h) name ROM copies so that a game's patch does nothing, while the vector
   runs the RAM copy. *Recommended*: keep, and treat a game that patches through the vector as a defect to record.
8. **Q8, `timers` in the gate.** *Decided 2026-10-12: the grading changed, as recommended (§25.3, `Diamond_Native.md`
   §11.12a).* Two rows depend on the kernel's timing modulo a frame and a line, which no document
   gives (§25.3), and each later change to the kernel re-draws them. *Recommended*: grade a delay row by its settled
   samples with any one an interrupt lands in dropped, and a frame's hblanks within 1, decided before the gate is
   counted; *or* keep the rule and accept that the open kernel's images pass it by phase.

### 25.6 Reading log, and exposures

Opened for this section, 2026-10-12: psx-spx `6d7d1bc` `kernelbios.md` lines 44-136 ("BIOS Memory", the ROM and RAM
maps, the table of tables), 542-592 (open, lseek, read, write, close), 939-1113 (the memory card functions,
"Asynchronous Access"), 1114-1140 (interrupt handling), 1235-1275 (C(06h), B(17h), the default elements, "No Nested
Exceptions") and 2521-2566 ("BIOS Character Sets"); `controllersandmemorycards.md` lines 40-178 (the overview, /ACK,
the emulation note, the signals) and 2352-2466 (the card's commands, FLAG, timings); `DOCS.txt`'s exclusions. EmuSen's
own: this page whole, `Diamond_OpenKernel.md` §1-§10, `Diamond_Plan.md` §1 and §5.9, `Diamond_Players.md` §13, the
disputes log's rules and D-22, `Diamond_Native.md` §11.12; the kernel and its harness, and of Diamond's crate
`examples/run.rs`, `tests/programs.rs`, `programs/kernel-pad.s` and `kernel-card.s`, `bus.rs`'s fetch path, `irq.rs`,
`memctl.rs`'s DEV2 figures, `machine.rs`'s stepping and `v1.rs`'s firmware entry; WiseMan's `TtyReader.cs` timers rule and
`DiamondTests.cs`; `psxplay.py` and the scripts; ps1-tests' `timers/main.c`, its `psx.log`'s first rows, and no file of
its `common/`. The games' own code and RAM on the open kernel only: Castlevania's text routine and glyph table, Spyro's
loops and calls, Crash Bandicoot's calls and its writes into the kernel's RAM.

**Never opened**: any firmware, `~/.cache/emusen/probe/diamond/sysdir/` or `~/Documents/BIOS/`, a BIOS listing,
disassembly or trace, OpenBIOS, any emulator's source, `Diamond_References.md`, psx-spx's "BIOS Patches" and the other
passages `DOCS.txt` excludes, the referees' outputs and pictures. No reference emulator and no `psxreferee.py` was run,
and Diamond only on the open kernel.

**Exposures**, stated plainly: Crash Bandicoot's 13 words written into the handler C(06h) named were printed once, as
hexadecimal, by a comparison of the kernel's RAM before and after the game's first frames; they are a game's patch of
the kernel, of the kind psx-spx's excluded "BIOS Patches" describes, and so may reflect the original's layout. They were
not decoded or studied, are not recorded here, and nothing in the kernel depends on them: the fix is to keep their
target in the ROM.

## 26. Mega Man Legends (2026-10-12)

*Written by the kernel's writer under the replacement writers' rules of the plan's §1.3.* The tester's copy of Mega Man
Legends stopped at a black screen on this kernel two frames after its entry at frame 201. The game was run on Diamond's
open kernel only, with no firmware folder, and read only through its own code and RAM, its calls and its accesses to the
logged registers (§22's `--game`). Three defects were found in turn, each shown by a test failing on the image before it
and fixed; the marks are this page's, **[M]** measured on 2026-10-12. §26.7 lists what was read and states the
exposures.

**Where the game stood** [M]. Its last logged accesses read I_MASK as 0Dh (vblank, CD and DMA enabled) and I_STAT as 0;
those reads came from the game's own vblank element, run by the exception that followed. The CPU was not waiting on an
interrupt: it was in SystemError, entered through A(40h) after a data bus error (Cause 1Ch) at 9FC03684h, inside the
kernel's own code, which the game had jumped into.

### 26.1 Addresses called past ChangeClearPAD's entry

**What the game does** [M, its library's code at 8001A2E4h-8001A85Ch, read from RAM at frame 203]. Its pad library takes
GetB0Table's table, reads B(5Bh)'s entry, and works on the code it expects past that entry: it stores zeros into nine
words at entry+62Ch and eleven at entry+594h, stores four words each at entry+3D8h and entry+4E0h, and calls FlushCache;
it keeps entry+884h, entry+894h and entry+7A0h as function pointers. Its pad start (8001A47Ch) calls StartPAD2,
ChangeClearPAD(0), then entry+884h; its pad stop (8001A4ACh) calls entry+894h before StopPAD2; entry+7A0h is returned to
the caller and not called in this game. On §25's image B(5Bh)'s entry was ChangeClearPAD at 9FC030C0h, so entry+884h was
9FC03944h, the middle of another function, which ran with the game's registers until a load faulted. This is the risk
the plan's §5.9 names (P7): games call and patch the kernel in ways psx-spx's excluded section describes, which a
replacement writer learns of only by a game failing.

*Withdrawn 2026-10-12, before merging; §27 replaced it* (§26.8). ~~**The answer** [C]. B(5Bh)'s table entry names a jump to ChangeClearPAD followed by 1000h bytes of `jr ra` and `nop`
pairs, so that every word from entry+4h to entry+FFCh is a function that returns at once, with v0 and every saved
register as the caller left them. What the original's code at those offsets does is not known, and is not sought; the
game needs nothing more from them than a return, as its play shows (§26.4). The stores land in the ROM and do nothing,
as Q7's decision keeps every table entry in the ROM (§25.3). The span covers the furthest offset this game uses, 894h,
with room for a library version that reaches further; one that addresses past 1000h, or computes addresses from another
entry, is not known (**Q9**). The sled costs 4 KiB of the ROM.~~

~~**Shown** [M]. `every_address_a_library_calls_past_change_clear_pads_entry_returns_at_once` (the harness's
`tests/kernel.rs`) makes the game's stores, calls FlushCache, then calls entry+7A0h, +884h, +894h and every word from
+4h to +FFCh with the saved registers set, and calls ChangeClearPAD at the entry. On §25's image it fails at the first,
+7A0h, the CPU in the exception path after 100 instructions; it passes.~~ The test was not merged with the answer.

### 26.2 LoadExec over its caller

**What the game does** [M]. With §26.1's fix the game shows its Capcom logo and "now loading…", then at frame 1211 calls
LoadExec("cdrom:\ROCK_NEO.EXE;1"): 823,296 bytes (C9000h) loaded at 80010000h, over the boot program SLUS_006.03 (4A000h
bytes at 80010000h) that calls it. Before the call the boot program resets its graphics, closes an event, stops its pads
and calls `_96_init`, but leaves in place its exit structure (HookEntryInt at 80046BB4h, whose RA, 80012ACCh, enters its
interrupt dispatcher), a vblank callback in its own table (80010C18h, a frame counter) and vblank enabled in I_MASK,
with ChangeClearRCnt(3, 0) and ChangeClearPAD(0) given, so that no kernel handler acknowledges vblank. Twenty-eight
frames into the load a vblank's exception ran the chains and then the exit; the dispatcher called 80010C18h, which by
then held ROCK_NEO.EXE's data, and the load it decoded faulted (Cause 1Ch at 80010C18h), into SystemError.

**What the documents say** [R "A(51h)", "B(19h)"]. LoadExec copies the name, "Enables IRQs via ExitCriticalSection",
loads by LoadTest and runs the file by Exec(headerbuf,1,0); a hook "is executed only if the ExceptionHandler has been
fully executed". Nothing is said of I_MASK or of the exit structure across LoadExec. A call that by its description
loads over its caller works only if the caller's handlers do not run while their memory is replaced [A].

**The answer** [C]. LoadExec, which never returns to its caller, masks every source in I_MASK and restores the default
exit (ResetEntryInt's) before it loads, since the caller's handlers lie in the memory the load replaces; it then enters
the program with I_MASK and I_STAT clear, the state §15 gives the boot program, so that the program enables what it
uses, as a boot program does. The kernel's own reads need no interrupt (§26.3). The chains' elements a caller leaves
queued are not removed: this game removes its own first, an element left in a chain is reached only through a source the
new program enables or an exception it takes, and SetConf, which psx-spx gives as the call that clears the chains, is
not LoadExec's.

**Shown** [M]. Diamond's `loadexec_starts_its_program_free_of_the_callers_interrupt_handlers` boots
`programs/kernel-loadexec.s` from a synthetic disc: an exit structure whose RA is a hook of its own at 80010008h, vblank
enabled, then LoadExec of `kernel-loadexec-second.s`, a 128 KiB program at 80010000h whose code at 80010008h marks a
word if it is ever reached. Each part of the answer was withdrawn in turn, §26.3's change kept: with LoadExec as on
§25's image the hook runs during the load; without the mask before the load the vblank stands unacknowledged and the
default exit re-enters the exception for ever (the run ends in the exception path at frame 1,200); without the exit
restored the new program's first vblank, with SetIrqAutoAck(0, 1), goes to the old structure and reaches the marker.
With all three the hook's count is the same at LoadExec and at the new program's entry, I_MASK and I_STAT read 0 there,
and the marker stays 0.

### 26.3 The kernel's synchronous CD operations and the program's handler

**What the game does** [M, the controller's trace and the game's accesses]. With §26.1 and §26.2 the game played its
intro, but on one build `_96_init` at frame 662 took 549 frames. Init's second response (INT2) arrived during a vblank's
exception after the kernel's priority 0 element had looked; the game's dispatcher, reached through its exit, read the
response and acknowledged it at the controller (its code at 80013C8Ch-80013D84h); the kernel's wait then ran its
4,000,000 polls out. Which builds meet it depends on where the response falls against the exception, so the delay came
and went with unrelated changes to the image. psx-spx's "Unstable IRQ Handling" describes the same hazard among the
original's handlers [R].

**The answer** [C]. ~~§12.1: while the kernel owns the drive it sets I_MASK's bit 2.~~ An asynchronous operation, which
the kernel's element carries on, unmasks IRQ2 as before; a synchronous one, whose caller services the controller itself
between interrupts (§12.1), masks IRQ2 for its duration and restores the bit on release, so that no program's handler is
offered a response of the kernel's. The responses still raise I_STAT's bit, which the kernel's servicing acknowledges.

**Shown** [M]. The same Diamond test has its hook OR the I_MASK it finds into a word while the program calls `_96_init`
and CdReadSector: on §25's image, and on this one with the change withdrawn, the word reads 5, IRQ2 enabled beside
vblank; with it, 1. With it Mega Man Legends' `_96_init` returns within a frame and LoadExec comes at frame 671.

### 26.4 Mega Man Legends, played

[M, `examples/run` with no presses, pictures every 300 frames to frame 6,000.] Frame 300: Capcom's opening animation, a
field of glowing cubes joined by lines. 600: the CAPCOM logo on white. 900 and 1,200: "now loading…" on black, the
second program loading. 1,500: the sky over the sea with the first line of the story, "In a world covered by endless
water…". 1,800 to 5,400: the story's text scrolling up over the sky and sea, an airship at the right, from "People are
forced to eke out a living on the small patches of land" through the quantum refractors, the Diggers and the "Mother
Lode" to "Seeking this Mother Lode, Diggers travel from one island to another in their flying machines". 5,700 and
6,000: "Hope… Despair… Charity… Greed… Duty… Power…" with a stone tower at the left.

[M, `psxplay.py diamond` by `EmuSen.WiseMan/Reference/analysis/psxplay/mega-man-legends.txt`, T 600, pictures every 120
frames to 7,200, the open kernel's formatted card in slot 1.] Start at T+1,400 skips the story; PRESS START BUTTON from
about 2,280; Start at 2,600 gives the menu, NEW GAME, CONTINUE, OPTION; Start at 2,900 begins a new game. From 3,360
Mega Man stands in a dark ruin and turns and moves with the script's presses; at 4,560 and 4,800 he lifts a shutter;
from 5,280 he walks a lit corridor away from and then towards the camera; from 6,000 the scene of the refractor on its
pedestal, which he takes up. An earlier run with Start every 300 frames from 2,000 reached the ruin's corridor by 3,900
and opened and closed the game's item menu (LIFE, ZENNY, TIME counting 00:00 to 00:02) to frame 12,000. Nothing wrong
was seen in the pictures. **Not shown**: play beyond the first ruin, and a save.

### 26.5 The image and the tests

~~[M] 524,288 bytes, 104,792 of them before the last non-zero byte, the variables ending at A310h; SHA-256
`fc1fb50726030c523bb39edaeaffb13b6cbc91e3a644241d10ee763d21f1f420`, rebuilt byte for byte with
`DIAMOND_KERNEL_REBUILD=require`. The images between: `6255e670…` after §26.1 and `7076b96e…` after §26.2.~~ That image
was never merged; the merged image, carrying §26.2's and §26.3's fixes with §27's stretch, is §27's,
`c1a002e8…`. The counts below are of the image this section built.

- `cargo test --release` in `kernel-harness/` with the rebuild required: 56 tests, all passing; new: §26.1's.
- `cargo test --release` in Diamond's crate: 155 unit tests and 71 program tests, with and without
  `EMUSEN_DIAMOND_PS1TESTS`; new: §26.2's.
- WiseMan's `Diamond` tests under the test lock with `EMUSEN_DIAMOND_PS1TESTS` set: 43 of 43, every ps1-tests program
  passing by its TTY and RAM records. `DiamondDebugTests`' read watch test compared a frame's store count with a
  quarter of its loaded bytes exactly, which holds only when the frame's edges fall between two of the loop's
  iterations: on this image the frame the watches see begins after the loop's first load and before its store (pc
  8001000Ch at frame 10 by the run tool), and the counts were 22,645 against 22,644; it passed on §25's image. The
  test now allows one iteration at the edges.
- The seven games by their scripts on the open kernel, `psxplay.py diamond` to frame 18,000 with pictures every 300
  frames, the formatted card (Spyro the Dragon a blank one) [M]: Crash Bandicoot's N. Sanity Beach and the hollow where
  the camera rests; Spyro the Dragon's card menus, the Artisans and "Entering Stone Hill"; Castlevania's file screen,
  "JOURNEY BACK TO 1792" and Richter's stage to the end; Silent Hill's opening, the foggy street and the alley; Ape
  Escape's first stage and the tutorial's boxes; Rayman's game chosen and named and the Dream Forest; Tekken 3's fights,
  "GAME OVER" and its attract mode, each as `Diamond_Players.md` §13 records them.

### 26.6 Questions for the tester

*Both decided 2026-10-12 as recommended* (§27.4).

9. **Q9, the returns past ChangeClearPAD.** The game's library calls three offsets past B(5Bh)'s entry; the kernel
   answers 1000h bytes of returns there and nothing else. Another library may anchor at another entry or reach further.
   *Recommended*: keep the construction to the entry a game is seen to use, extending it, entry by entry, when a game
   shows the need, as §26.1 did.
10. **Q10, LoadExec and the chains.** LoadExec now masks the sources and restores the exit, but leaves the chains'
    elements a caller queued. *Recommended*: leave them until a game is seen to rely on their removal, since a program
    may hand over with an element of its own kept in a part of memory the new program does not load over.

### 26.7 Reading log, and exposures

Opened for this section, 2026-10-12: psx-spx `6d7d1bc` `kernelbios.md` lines 1114-1240 (interrupt handling, the priority
chains, SysEnqIntRP and SysDeqIntRP, the critical sections, C(06h)), 1540-1680 (the timer and joypad functions), 700-840
(the file execute functions, SetConf, FlushCache), 900-945 (the CD's internal functions' entries) and 2085-2110
(GetB0Table); `DOCS.txt`'s exclusions. EmuSen's own: this page whole, `Diamond_OpenKernel.md` §1-§4, §6-§11,
`Diamond_Plan.md` §1 and §5.9, `Diamond_Players.md` §13, the disputes log's rules and headings; the kernel and its
harness, and of Diamond's crate `examples/run.rs`, `tests/programs.rs` and `programs/kernel-cd.s`; `psxplay.py` and its
scripts. The game's own code and RAM on the open kernel only: its pad library's calls and stores, its interrupt
dispatcher and callback table, its boot program's calls before LoadExec, the two executables' headers and its CD
handler's accesses.

**Never opened**: any firmware, `~/.cache/emusen/probe/diamond/sysdir/` or `~/Documents/BIOS/`, the runs under
`~/.cache/emusen/probe/diamond/mml/` and `realbios/`, a BIOS listing, disassembly or trace, OpenBIOS, any emulator's
source, ps1-tests' `common/`, `Diamond_References.md`, and the bodies of psx-spx's "BIOS Patches" and of the sections
still to be screened. No reference emulator and no `psxreferee.py` was run, and Diamond only on the open kernel.

**Exposures**, stated plainly:

- A search of `kernelbios.md` for ChangeClearPAD, not confined to the permitted lines, printed two lines from inside
  "BIOS Patches" (lines 3203 and 3236, excluded), each a part of a sentence naming B(5Bh) ChangeClearPAD, and an
  earlier search printed line 2296, inside "BIOS More Internal Functions" (not yet screened), the call's row in a list
  with a note that it serves the cards too. Nothing in them was used: §26.1 rests on the game's code, and
  ChangeClearPAD's meaning is unchanged from `Diamond_OpenKernel.md` §9 item 4. Later searches were confined to the
  permitted lines.
- The game's pad library is a patch of the kernel of the kind psx-spx's excluded section describes, and its offsets
  may reflect the original's layout. Only the offsets it calls and stores at are recorded here, as the fix needs them;
  the words it stores were seen in the disassembly, are not recorded, and nothing depends on them, since they meet the
  ROM.

### 26.8 Why §26.1's answer was withdrawn

Two findings came after this section was written, and the tester decided both on 2026-10-12.

- **The image's check.** `firmwarecheck.py` against the 18 images of the firmware folder, by `EmuSen_Firmware.md` §0's
  control rule, failed on all 18 with this section's image: 36,497-119,757 runs of 9 bytes or more shared with an image,
  against the control's 11,299-27,599 [M]. The runs were the 4 KiB of `jr ra` and `nop` pairs, a pair any compiled
  MIPS code holds at the end of each of its functions; with those 4 KiB blanked in a scratch copy the same image passed
  (6,778 runs against 11,315 on one image, 16,750 against 27,211 on another) [M]. Nothing was copied; the form of the
  answer fails a check whose purpose is to bound what a replacement shares with an original, and the form was changed,
  not the check.
- **The exposure above.** The three lines printed by the unconfined search all concern B(5Bh), the call §26.1 answers.

*Decided 2026-10-12:* §26.1's answer is withdrawn and rebuilt by another of the kernel's writers, who had read none of
this section, its branch or its scratch files, from the game's own code alone, in a form of fill and sparse returns
that the check passes (§27); §26.2's and §26.3's fixes stand, carried into §27's image unchanged; the exposure is
recorded here; and Q9 and Q10 are decided as recommended. §27's image passes the check on all 18 images
(`EmuSen_Firmware.md` §0).

## 27. The stretch behind B(5Bh)'s entry (2026-10-12)

*Written by the kernel's writer under the replacement writers' rules of the plan's §1.3. §26 is kept for the record of
the two fixes this section's image also carries, LoadExec's and the synchronous CD operations'.* Mega Man Legends (the
tester's copy, USA) showed a black screen on the open kernel. The marks are this page's, **[M]** measured on 2026-10-12;
E is the address the B table gives for B(5Bh), ChangeClearPAD. §27.8 lists what was read and states the exposures.

### 27.1 The game's evidence

**The stop** [M, the run tool's `--game` and a RAM dump, on the image built from §25's sources with §26's two fixes,
SHA-256 `b2d6c568…`, never committed]. The game enters at frame 201. At frame 204 it calls B(57h) GetB0Table three
times, each followed by A(44h) FlushCache, with B(5Bh) ChangeClearPAD(0), C(03h), C(02h), B(12h) InitPAD2, B(13h)
StartPAD2 and B(5Bh) again among them, and makes no call after. The CPU is then in SystemError's loop; the TCB holds
Cause 0000001Ch, a bus error on a data access, with EPC 9FC03684h and RA 9FC03960h, both in the kernel's ROM.

**The game's pad library** [M, its code in RAM, disassembled: 8001A3F0h-8001A860h in the program the disc boots,
80080518h-800806A4h in the program that one starts by LoadExec, the same instructions at other addresses]. Three
routines each call GetB0Table and load the word at 16Ch of the table it returns, entry 5Bh, into a register: E. None
calls through it. From E they compute addresses, store words at some and keep others to call:

| From E | What the game does there | When, by the run's frames |
|---|---|---|
| 3D8h-3E7h | stores four words, one instruction of its own and three zeros | 204 (the booted program), 859 (the second) |
| 4E0h-4EFh | stores the same four words | 204, 859 |
| 594h-5BFh | stores eleven zero words | 204, 859 |
| 62Ch-64Fh | stores nine zero words | 204, 859 |
| 7A0h | keeps the address, and holds a routine that calls it; nothing in either program's RAM calls or names that routine | never seen called |
| 884h | calls it, straight after StartPAD2 and ChangeClearPAD(0) | 204, 859 |
| 894h | calls it, straight before StopPAD2 | 663, eight frames before LoadExec |

Each run of stores is followed by FlushCache, so the game takes the addresses for instructions in RAM that it means to
change, and the two calls for routines that return [A]. Between them it removes an element from the priority 1 chain
and queues one of its own there (C(03h), C(02h)); before LoadExec it removes that element again (C(03h) at frame 663).
On that image E was 9FC030C0h, a Rust function among others in the ROM: E+884h, 9FC03944h, fell inside an unrelated
function, which ran from its middle, called on (the RA above) and came to the bus error.

**Everything it reaches** [M, the run tool's `--game-range 1FC00000:1FC80000`, added for this section: every load,
store and jump of the game's code into the ROM, over the 12,600 frames of §27.6's script]. No load. 28 stores at frame
204 and 42 at frame 859: the 28 of the table above each time, and at 859 fourteen more through GetC0Table's entry 06h
into the exception handler's ROM copy, the kind of patch §25.3 records for Crash Bandicoot, which the ROM ignores as it
does there. Three jumps: E+884h twice and E+894h once, as the table gives them. Nothing else, in play as before it.

### 27.2 The design

**The entry** [C]. B(5Bh)'s table entry is still ChangeClearPAD, now four instructions of assembly in `entry.rs`, so that
its place and what follows it are the kernel's to lay out: the flag (`pad.rs`'s `PAD_CLEAR`) loaded into v0 and the
argument stored, as before. E lies on a 16-byte boundary, a line of the I-cache.

**The stretch** [C]. From E+10h to E+A00h the ROM holds zero words, which the R3000 runs as `nop`, with `jr ra` in the
last word but one of every 100h bytes (E+F8h, E+1F8h, … E+9F8h) and a zero word, its delay slot, after it; one more
`jr ra` and zero word follow at E+A00h, so that a call at the last delay slot returns as well. 2,568 bytes in all. A call
at any word from E+10h to E+9FCh therefore runs `nop`s, one `jr ra` and its delay slot: it returns to its caller with
every register as it was, v0, HI and LO among them, touches no memory and takes no exception.

**The stores** [R §25.3, Q7]. The tables name the ROM copies of the kernel's code, so a store behind E is a store to the
ROM and changes nothing. That is kept, and it is what makes the game's one non-zero word harmless: in RAM a slide from
E+3D8h would run it.

**Why this form** [R `EmuSen_Firmware.md` §0]. The image is judged for what it repeats with fill left out, a run of
eight or more identical bytes; a return at every word pair would be one instruction pair repeated over three hundred
times, which is not fill and would count. Here each return is four bytes that are not fill, between two runs that are.

**Not an imitation** [C]. What the original holds at these offsets is not known and was not sought (§27.8). The game
shows only that it rewrites instructions there and calls two routines, once each as it starts and stops its pads; on
this kernel the routines do nothing, and the pads are read each vblank by the kernel's element as for any program.
Whether a game loses anything by that can only show in a game: this one reads its pad and plays (§27.6).

**How far** [A]. The furthest address the game reaches is E+894h. The 100h bytes holding it end at E+900h; one more
unit, to E+A00h, is the margin, so that a build of the same library with offsets up to 256 bytes greater would still
land in fill. No further: by Q9 below the construction follows what games are seen to do.

### 27.3 The cost

[M] `programs/kernel-pad-entry.s` on Diamond's bus, each call made inside a critical section, cold straight after
FlushCache and then again warm; the game's own calls by `--game-range`, which counts from the jump to the game's next
instruction:

| Call at | Instructions | Cycles, cold | Cycles, warm | In the game |
|---|---|---|---|---|
| E+884h | 31 | 798 | 31 | 984 at frames 204 and 859 |
| E+7A0h | 24 | 618 | — | not called |
| E+894h | 27 | 695 | — | 857 at frame 663 |
| E+FCh, a return's delay slot: the longest slide | 65 | 1,673 | 65 | — |

A cold line of the ROM costs 100 cycles at the memory control values the reset path sets (four words of 25) and 124
under the game's; the longest slide is one word of a line and sixteen lines. For scale, the game's call of B(5Bh)
through its vector takes 430 cycles straight after a FlushCache and 147 otherwise, and a frame about 566,000; the game
enters the stretch three times in 12,600 frames. With E
unaligned, as first built, each return's delay slot began a line of its own and each call cost a line more (872, 692,
769 and 1,748 cycles); hence the alignment.

**The spacing** [C]. Returns every 80h bytes would halve the longest slide to about 850 cycles and double the returns to
twenty. With calls made once as the pads start and stop, 1,673 cycles at worst is 0.3 % of one frame, and the sparser
form, ten returns and the one after the end, is kept.

In the ROM the stretch costs 2,568 bytes of 524,288; in RAM nothing.

### 27.4 Two decisions

9. **Q9, where the construction goes.** Other entries of the tables may be reached past in the same way by other
   games, and the stretch could be laid behind every entry, or behind every pad entry, in advance. *Decided 2026-10-12*:
   it goes only behind a table entry a game is seen to reach past, one entry at a time as games show the need. B(5Bh)
   is the first.
10. **Q10, LoadExec and its caller's chain elements.** LoadExec masks the caller's interrupt sources and restores the
    default exit before it loads (§26); it could also remove the elements the caller queued, whose functions the load
    overwrites. *Decided 2026-10-12*: LoadExec leaves any chain elements its caller queued, until a game is seen to
    need them removed. Mega Man Legends does not: it removes its own element before the call (§27.1).

### 27.5 The tests

Each failed on the image before and passes [M].

- `kernel-harness/tests/pad_entry.rs`, five tests: B(5Bh)'s entry found as the game finds it, naming the ROM and still
  ChangeClearPAD, through the vector and called at its address; a call at E+7A0h, E+884h and E+894h back within 64
  instructions with R1-R31, HI and LO as set, no store, no exception and RAM unchanged; the same at every word from
  E+10h to E+9FCh, the longest 65 instructions; the game's 28 stores all at ROM addresses, the ROM as it was and the
  calls still returning after them; and the form: E on a 16-byte boundary, zero words, `jr ra` at the end of each 100h
  and after the last. On `b2d6c568…` four of the five fail, the call at E+7A0h never coming back to its caller.
- Diamond's `calls_and_stores_past_b_5bh_s_entry_come_back_and_leave_the_pads_read` (`programs/kernel-pad-entry.s`): on
  Diamond's bus, the game's stores, FlushCache, InitPAD2, StartPAD2, ChangeClearPAD, then the calls of §27.3 with v0
  and S2-S7 checked after each, their instruction counts exact and their cycles bounded (1,000 for the game's three
  cold, 2,000 for the longest, 100 warm), and port 1's pad read by the kernel's element afterwards. On `b2d6c568…` the
  program never finishes.

### 27.6 The game

[M, `c1a002e8…`, no firmware folder.] With no presses, pictures every 300 frames to frame 6,000: a burst of green
particles (300), the CAPCOM logo (600), "now loading…" on black (900, 1,200), and from 1,500 the story's text scrolling
over sea and sky with the airship, "In a world covered by endless water…" to "Hope… Despair… Charity… Greed… Duty…
Power…" at 6,000.

`EmuSen.WiseMan/Reference/analysis/psxplay/mega-man-legends.txt` through `psxplay.py`'s Diamond side, T the CAPCOM logo
at frame 440, to frame 12,600, pictures every 60: Start skips the story; the title with PRESS START BUTTON from T+1,540;
Start, then Cross on NEW GAME; the opening scene left to play (the ruin, the refractor taken, the run from the closing
door) to about T+8,380; then the first ruin in play, the life gauge and radar drawn. A step forward brings the scene's lines
("Wait a second! There should be a Reaverbot around there somewhere…") and its instructions, read through with Cross;
"Read instructions again?" answered No with Right and Cross; then the corridor run down, "Turn right there.",
Reaverbots met, the life gauge turning red under their hits, explosions, the turns by R1 and L1 taken and a second
corridor walked. Every press acts, so the pads the kernel reads reach the game.

**Seen and not the kernel's** [M]: on the title the yellow figures that run across the screen leave copies of
themselves behind for a few hundred frames at a time (about T+1,760 to T+2,040 and T+2,740 to T+3,000). Whether the
game draws them so is not known here, no referee having been run; it is left to a step that owns the GPU.

### 27.7 The image and the gates

[M] 524,288 bytes, 103,256 of them before the last non-zero byte; SHA-256
`c1a002e88aa4e8b1be0257ca486bdb8309cd63f229e1f5070e36e4c72559c785`, rebuilt byte for byte with
`DIAMOND_KERNEL_REBUILD=require`; the variables still end at A310h. E is 9FC00670h in it. It carries §26's two fixes
with this section's. `firmwarecheck.py` was not run in this step; it is run apart, as each time the image changes.

- `cargo test --release` in `kernel-harness/` with `DIAMOND_KERNEL_REBUILD=require`: 60 tests, all passing, the five
  of §27.5 among them.
- `cargo test --release` in Diamond's crate: 155 unit tests and 72 program tests, with and without
  `EMUSEN_DIAMOND_PS1TESTS`, §26's and §27.5's among them.
- WiseMan's `Diamond` tests under the test lock, with `EMUSEN_DIAMOND_PS1TESTS` set and no firmware variable: 43 of 43.
- The seven games by their scripts on the open kernel with the formatted card, pictures every 300 frames: Crash
  Bandicoot to N. Sanity Beach and the hollow; Spyro the Dragon's card question, the slot menu and the Artisans' world;
  Tekken 3's fights, "YOU WIN!", "YOU LOSE" and the final round; Castlevania's file screen, "JOURNEY BACK TO 1792" and
  Richter's stage; Ape Escape's DUALSHOCK screen, the first stage and its tutorial's boxes; Rayman's slot menu and the
  Dream Forest; Silent Hill's warning, its title and the foggy street. Each as §25.4 records it.

### 27.8 Reading log, and exposures

Opened for this section, 2026-10-12: psx-spx `6d7d1bc` `kernelbios.md` lines 1598-1700 ("BIOS Joypad Functions", and
the GPU functions' first entries after it) and 2090-2104 (B(56h) and B(57h)), found by a search confined to lines
153-2228 for the two calls' names; it gives B(5Bh) a name in its summary and no description. `DOCS.txt`'s first 80 lines:
its exclusions, and past them the list of the documents fetched. EmuSen's own: this page whole, `Diamond_OpenKernel.md`
§1-§4 and §6-§11 with §5's head, `Diamond_Plan.md` §1 and §5.9, `Diamond_Players.md` §13, `EmuSen_Firmware.md` §0; the
kernel's `entry.rs`, `table.rs`, `pad.rs`, `main.rs`, `mem.rs` and its linker script, and §26's change to `exec.rs` and
`cdrom.rs`; the harness's sources and its `registers`, `image`, `reset`, `kernel` and `card` tests; of Diamond's crate
`examples/run.rs`, `kernel.rs`, `tests/programs.rs` and `programs/kernel-pad.s`; `psxplay.py`, the scripts' headings and
the earlier step's script that runs the seven games. The game on the open kernel only: its calls, the code of its pad
library in RAM at the addresses of §27.1 with two searches of its RAM for that code's callers, the TCB's saved EPC,
Cause and RA after the stop, its accesses to the ROM, and its pictures.

**Never opened**: any firmware, `~/.cache/emusen/probe/diamond/sysdir/` or `~/Documents/BIOS/`, a BIOS listing,
disassembly or trace, OpenBIOS, any emulator's source, `Diamond_References.md`, psx-spx's "BIOS Patches" and the other
passages `DOCS.txt` excludes or holds for screening, ps1-tests' `common/`, the referees' outputs and pictures, the
branch `diamond-kernel5` and its scratch folder. No reference emulator and no `psxreferee.py` was run, and Diamond
only on the open kernel.

**Exposures**, stated plainly:

- The game's pad routines were read as its own code, which the step required: they give the offsets behind B(5Bh)'s
  entry at which the game rewrites and enters the original, and the one instruction it stores there. That is the game's
  knowledge of the original's layout, of the kind psx-spx's excluded "BIOS Patches" describes. The kernel takes from it
  only where a call must return and a store must change nothing; the stored instruction is not recorded here and
  nothing in the kernel answers to it.
- The watch over the ROM printed, as hexadecimal, the fourteen words the game's second program stores into the
  exception handler's ROM copy at frame 859. They were not decoded or studied, are not recorded, and nothing depends on
  them: the handler the tables name stays in the ROM (§25.3).
- Listings of the probe's cache showed by name the folders this step may not enter (`sysdir`, `realbios`, `mml`,
  `kernel5`, the `fwcheck-` folders, the referees' folder under `play`); none was entered or listed.
- No search printed a line of an excluded passage.
