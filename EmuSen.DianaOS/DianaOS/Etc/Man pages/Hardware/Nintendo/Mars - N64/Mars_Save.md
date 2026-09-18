# Mars — the save chips, the Controller Pak, and how a cartridge's chip is named

*Phase E's save slice, landed 2026-09-18. `EmuSen/Cores/Nintendo/Mars - N64/Memory/` — `SaveChip.cs`, `Eeprom.cs`,
`Sram.cs`, `FlashRam.cs`, `ControllerPak.cs` — with the title table in `Rom/SaveTypes.cs` and the files in `MarsCore.cs`.
Tests in `MarsSaveTests` and `MarsSaveFileTests`; the games through the probe (`Mars_GameProbe.md`).*

---

## 0. What grades this, when nothing in the corpus does

**The hardware corpus has no save group.** Its source has no EEPROM, SRAM, FlashRAM or Controller Pak test, and no
access to the cartridge's second domain at all (`~/Projects/nemu64-test-reference/src/tests/`), so nothing here is
graded by a measurement of a console. What stands in for one:

- **The FPGA core is the source where the references disagree**, as it has been for every Phase E device: `rtl/PIF.vhd`
  for the EEPROM and the joybus routing, `rtl/PI.vhd` for SRAM and FlashRAM, `rtl/Gamepad.vhd` for the pak.
- **mupen64plus and Project64 were read beside it** for every device, and their disagreements with it and with each
  other are recorded where they fall. There are twelve; §2–§5 name the ones Mars had to choose between.
- **Two commercial games** are the end-to-end check: whether each still runs with the chips in place, and which chip its
  own behaviour names (§8).

The research that gathered the three references' behaviour, with a line citation for each claim, was done for this
slice and is summarised here rather than reproduced; every rule below cites the file it was checked in.

## 1. Naming the chip

A cartridge carries at most one save chip, and nothing in a commercial image says which (`Mars_Rom.md` §3). Mars asks
four questions in order and takes the first answer:

1. **The image's own word.** A homebrew image with the `ED` header declares its chip in the high nibble of byte 0x3F.
2. **The title table** (§6), keyed by the header's two checksums, which holds only 16 Kbit EEPROMs.
3. **The length of the save an earlier run wrote**: 2KB is an EEPROM, 32KB SRAM, 96KB banked SRAM, 128KB FlashRAM.
4. **What the game does first.** An undecided chip becomes an EEPROM when the game reads or writes a block on the
   joybus's fifth channel, FlashRAM when the processor touches the second domain, and SRAM when a transfer does.

**Asking is not using.** An EEPROM's info command does not decide the chip, because a probe may send it whatever the
cartridge holds; an undecided chip answers it as the smaller EEPROM, as all three references do for a cartridge they
have not classified, and stays undecided. Once the chip is named, the others are absent: a cartridge found to have SRAM
stops answering on the EEPROM channel.

**The rules of the fourth question are Project64's and mupen64plus's**, which agree on them — a processor access to the
second domain means FlashRAM, a transfer means SRAM. They hold if libultra's flash driver begins with a command the
processor writes and its SRAM driver only ever transfers, which is how both emulators' authors describe them; no libultra
source is on this machine to read, so that is recalled rather than checked, and §8 is the check that stands in for it. A
game that did otherwise would be named wrongly. The one chip that *cannot* be named by use is the 16 Kbit EEPROM, whose
driver sends the same commands the 4 Kbit one's does; Project64's own documentation says so, and it is why the table
exists (§6).

**A save from an earlier run is held until the chip is named**, and handed to whichever device the game turns out to
use. The third question usually makes this moot — the length names the chip before the game runs — but it is what
keeps a save that question could not read (a length Mars does not recognise) from being lost to the first guess.

## 2. The EEPROM

Eight-byte blocks on the joybus, behind every channel past the four controller ports:

| command | sends | receives | answer |
| --- | --- | --- | --- |
| `0x00` info, or `0xFF` | 1 | 3 | `00 80 00` for 4 Kbit, `00 C0 00` for 16 Kbit |
| `0x04` read | 2 | 8 | the block the second byte names |
| `0x05` write | 2 + data | 1 | stores the data, answers `00` |

- **Both sizes hold 2KB.** The FPGA core's EEPROM is one 2KB memory whatever size it reports (`PIF.vhd` 1201–1222),
  and addresses it with the whole block byte, so a 4 Kbit chip reaches past its 512 bytes. Project64 does the same;
  mupen64plus ignores blocks past the chip. A real 4 Kbit part probably decodes six block bits and repeats, which no
  reference models and nothing measures. A game whose driver bounds its block by the chip's size — libultra's is
  recalled to, unchecked here — cannot see the difference; Mars follows the referee, and its save file for either size
  is 2KB.
- **A write stores however many bytes it carries**, two fewer than its send length, wrapping inside its block: the
  FPGA core counts them with a three-bit address (`PIF.vhd` 1008, 1024). mupen64plus refuses any write that is not
  exactly ten bytes.
- **Every channel past the fourth reaches the cartridge.** The FPGA core routes channel 4 and above to the
  cartridge's decoder (`PIF.vhd` 815–823); mupen64plus has exactly five channels and Project64 reports a sixth as an
  error. Mars follows the referee; libultra only ever uses the fifth.
- **A cartridge without an EEPROM does not answer on its channel**, to any command. **This departs from all three
  references**, which answer reads and writes from an EEPROM that is not there — the FPGA core ignores the save type
  for those two commands (`PIF.vhd` 966–968, 999–1001), and Project64 even becomes an EEPROM on the first one. The
  reason is physical rather than measured: with no chip on the channel, nothing can drive a reply, and the PIF reports
  exactly that. A game that reads the EEPROM its cartridge lacks would see the difference; none is known to.
- **Erased is all ones**, in all three.
- **The real-time clock's commands (`0x06`–`0x08`) are not answered.** The references name one Japanese title for it; all three
  references answer its commands on every cartridge, which the same physical argument declines to copy. §9.

## 3. SRAM, and the second domain

32KB at `0x0800_0000`, read and written by the processor a word at a time and by the PI's transfers a byte at a time.

- **One bank repeats across the domain.** The FPGA core addresses 32KB SRAM with bits 14:2 of the address, so every
  32KB above it is the same memory (`PI.vhd` 466–474).
- **Banked SRAM is three 32KB banks picked by address bits 18 and 19**, at `0x0800_0000`, `0x0804_0000` and
  `0x0808_0000`, with nothing past the last. **This departs from the referee.** The FPGA core addresses its 96KB SRAM
  contiguously with bits 16:2 and drops bit 18, so `0x0804_0000` would land on the first bank. Project64 banks it, in a
  commit whose message is *"Support Dezaemon 3D saves (SRAM 96KB)"* (`cd2f3cf17`), and the flashcart convention that
  names the type calls it banked; that is a game's behaviour and a published convention against a construction whose
  reasoning cannot be read, because the FPGA clone has no history. No image in the library uses it, and a PI trace of
  that one game would settle it. The homebrew header's 1 Mbit SRAM is treated as four such banks, which is an
  extrapolation from the three-bank rule and nothing more.
- **A processor store writes the whole bus word**, whatever size the instruction named — the FPGA core writes SRAM
  with all four byte enables (`PI.vhd` 523–534) — so the second domain joins the signal processor's memories and PIF
  RAM as a window that takes only whole words (`Mars_Memory.md` §2.4).
- **Every store on the cartridge bus holds it**, not only a store to the ROM window: the FPGA core starts its I/O-busy
  latch for any processor write to the cartridge (`PI.vhd` 508–516), which `Mars_Memory.md` §7.7 had applied to the ROM
  window alone. **Only the ROM window hands the word back.** A read of the second domain returns the chip, and leaves
  the bus held (`PI.vhd` 486–489). Widening the latch moved nothing in the corpus, which never stores below the ROM
  window.
- **An empty second domain reads the low half of the address twice**, `0x1234_1234` at `0x0800_1234`: the FPGA core's
  open bus (`PI.vhd` 459), and mupen64plus's.
- **Erased is all ones.** mupen64plus fills SRAM with `0xFF`; Project64 starts from an empty file; the FPGA core starts
  from whatever its SDRAM holds.
- **Not modelled: the domain's timing registers.** The FPGA core charges them for transfers out of the chip only;
  mupen64plus takes a flat 0x1000 cycles; Project64 finishes at once, as Mars does for every transfer (`Mars_Memory.md`
  §7.1).

## 4. FlashRAM

128KB behind a command register. The command is the top byte of any processor write to the second domain except its
first word, and the work waits for an execute:

| command | effect |
| --- | --- |
| `0x4B` | remember the page in the low ten bits, for an erase |
| `0x78` | erase mode; status `1111_8008 00C2_001D` |
| `0xA5` | remember the page, for a program; status `1111_8004 00C2_001D` |
| `0xB4` | program mode |
| `0xD2` | **execute**: erase the remembered page to ones, or program it from the page buffer |
| `0xE1` | status mode; status `1111_8001 00C2_001D` |
| `0xF0` | read mode; status `1111_8004 F000_001D` |

The processor reads the status word's upper half at an address with bit 2 clear and its lower half with bit 2 set, in
every mode. A transfer out reads the status word in status mode, the array in read mode, and zeros in any other; a
transfer in fills the 128-byte page buffer, whatever the mode, wrapping every page.

**This is the FPGA core's model, and Project64's** (`PI.vhd` 536–567, 650–672, 844–855, 906, 923–925;
`FlashRam.cpp`). mupen64plus runs a different one, and the two differ in four places that Mars had to choose between:

1. **When the work happens.** mupen64plus erases on `0x78` and programs on `0xA5`; the referee waits for `0xD2`, and
   treats `0xD2` as execute rather than as status mode.
2. **How much an erase clears.** mupen64plus erases a 16KB sector and supports `0x3C`, chip erase; the referee and
   Project64 clear one page, and ignore `0x3C`. A game that erases a sector and then rewrites fewer than all of its
   pages would read stale data under the referee's model. No game in the library uses FlashRAM, so the difference is
   untested here, and the chip's datasheet — which would say what the silicon does — is not among the references.
3. **Which chip it claims to be, and 4. how a transfer's address is scaled.** These must be chosen together: each
   reference is consistent with itself, and the driver is recalled to scale its flash address by the chip it reads
   back, which would make a mixed pair read the wrong pages. mupen64plus and Project64 report the MX29L1100
   (`00C2_001E`) and double the cartridge offset; the referee reports the MX29L1101 (`00C2_001D`) and uses the offset as
   it is. Mars takes the referee's pair, whole.

**Execute with nothing pending does nothing**, in every model that has an execute. At power-on the flash is idle with a
status word of zero, so a game has to send `0xF0` before a transfer reads anything but zeros; mupen64plus starts in
read mode instead.

## 5. The Controller Pak

32KB in the first controller's slot, read and written thirty-two bytes at a time:

| command | sends | receives | answer |
| --- | --- | --- | --- |
| `0x02` read | 3 | 33 | the chunk and its CRC |
| `0x03` write | 35 | 1 | the CRC of what was written |

- **The address is its first two bytes with the low five bits dropped.** Those five bits are a CRC of the address,
  which none of the three references checks, and so none gives its polynomial; Mars ignores them too.
- **The data CRC is CRC-8 with polynomial `0x85`**, run over the thirty-two bytes and one more zero byte — which is the
  plain CRC-8 by another construction. All three references compute it the same way (mupen64plus's
  `pak_data_crc`, Project64's `Mempak.cpp`, the FPGA core's bit-serial `Gamepad.vhd` 640–657). The CRC of the bytes
  0 to 31 is `0x33`, of thirty-two ones `0x0A`, of thirty-two zeros `0x00`.
- **Above `0x8000` reads zero and keeps nothing**, in all three. That is the region where an accessory says what it is,
  and a pak's zeros are what tell a game it is not a rumble pak.
- **A new pak is formatted**, with mupen64plus's layout (`paks/mempak.c` 98–151): an identity block at 0x20 and its
  three copies at 0x60, 0x80 and 0xC0, each checksummed so that the sum and its complement add to `0xFFF2`; an index
  page with five reserved entries and 123 free ones, whose checksum is `0x71`; and a copy of the index. mupen64plus
  fills the identity's serial from the clock; Mars leaves it zero, so that every run makes the same pak. Project64
  writes a fixed image with a different identity block, and the FPGA core formats nothing.
- **An empty slot does not answer the pak's commands**, as the FPGA core's does not. mupen64plus answers with the
  CRC inverted and Project64 with zeros and a valid CRC.
- **The slot is filled by default.** A game that saves only to a pak cannot save at all without one, and a console
  owner of the period usually had one. What a frontend cannot yet do is empty it, or put a rumble pak there instead.

## 6. The title table

**Fifty-four rows, all 16 Kbit EEPROM**, keyed by the two checksums in the header at 0x10 and 0x14. They are the rows
that mupen64plus's `mupen64plus.ini` gives as `Eeprom 16KB` and Project64's `Project64.rdb` gives as `16kbit Eeprom`,
and no others; mupen64plus's licence is the GPL, version 2 or later, which this project's third version accepts.

**The table holds only what use cannot reveal.** A 4 Kbit EEPROM, SRAM and FlashRAM are each named by the game's first
move (§1), so a row for one would only restate what the machine finds out for itself; a 16 Kbit EEPROM is not, because
its driver behaves as the smaller one's does. Two database rows were left out on evidence:

- **Four 16 Kbit rows mupen64plus has and Project64 does not.** One database's word is not two.
- **One row the two databases disagree about**: `616B8494-8A509210` is *Kobe Bryant's NBA Courtside* at 16 Kbit in
  mupen64plus and an arcade board's game at 4 Kbit in Project64 — the same two checksums on two different images. A
  checksum key cannot tell them apart, which is the table's standing weakness, stated rather than solved.

Where the two databases both have a row they agree on 65 of 68, and the three disagreements are all arcade boards. That
agreement is not independence: both projects have been fitted to the same games for twenty years, and neither says
where its rows came from.

**For the three games in the library, both databases agree with what use found** (§8): mupen64plus gives Super Mario
64 and Wave Race 64 a 4 Kbit EEPROM and Ocarina of Time SRAM; Project64 has no row for any of them and finds them by
use, as Mars does.

## 7. The files

- **The cartridge's chip goes in `<rom>.srm`**, the one name `SaveLibrary` gives a save (`EmuSen_Galaxia.md` §5), in
  the N64's own byte order — byte 0 of the chip is byte 0 of the file. mupen64plus and Project64 store SRAM and
  FlashRAM word-swapped, as they sit in a little-endian host's memory, so their files are not Mars's; all three lay an
  EEPROM out byte for byte.
- **The Controller Pak goes in `<rom>.mpk`** beside it, one pak per game. A real pak moves between games, which is the
  point of it; one per game keeps one game from overwriting another's, and costs that.
- **Only what changed is written**, when a frontend asks (`SaveSram`) and every 300 frames without being asked, as the
  other cores do. ~~The every-300-frames write is covered by nothing but the code: a test would need 300 frames of a
  running machine.~~ The breakage round found it uncovered, and a case on very short fields now covers it (§8.1).
- **`--nobattery` is latched when the ROM is loaded**, as the other cores' cartridges latch it: nothing is read, nothing
  written, and the chip still works for the run.

## 8. What was measured

**Both games that boot name their chip by what they do, and name it the way both databases do.** The game probe
(`Mars_GameProbe.md`) ran each for sixty seconds of a Release build with an empty save and a formatted pak in the first
port, the machine a frontend gets:

| | chip named by use | written | pak |
| --- | --- | --- | --- |
| Super Mario 64 (Europe) | 4 Kbit EEPROM | yes, from frame 61 | untouched |
| Wave Race 64 (USA) | 4 Kbit EEPROM | yes | not written |

Neither game's picture, audio or frame count changed with the chips in place (198 and 193 framebuffers in the minute,
32kHz audio in both), and Wave Race draws its attract race with a pak in the slot as it did with none.

**A save made by one run is found by the next, and the game behaves differently for it.** That was shown by accident.
The first attempt at `Mars_GameProbe.md` §6's controller check ran Super Mario 64 twice in one save folder, expecting
the two runs to be identical until a button was pressed; they parted by frame 399. A standalone run found Mars
deterministic — two runs identical for 400 frames, side by side and one after the other — and then found the cause:
the game writes its EEPROM at frame 61, the every-300-frames write put it in `<rom>.srm`, and the second run's
`LoadRom` read it back, so the second boot was a boot with a save. It is the trap `--nobattery` exists for on the
other cores (`Venus_Memory.md` §2.4a), met from the other side, and the check now gives each run its own folder. What it is evidence for is narrow: a save written by Mars is read by Mars, and changes what a game does.
That the bytes are what a console would have written is not tested, since nothing here reads a console's EEPROM.

**Ocarina of Time did not boot when this slice was measured**, so SRAM has met no game here; the next boot of it is
the first such meeting (`Mars_Boot.md`).

### 8.1 The breakage round

Fifty-one rules were broken in turn — eleven on the EEPROM and its channel, eight on SRAM and the bus, eleven on
FlashRAM, six on naming the chip, ten on the pak and its format, five on the files — with `MarsSaveTests`,
`MarsSaveFileTests`, `MarsSerialTests`, `MarsCartridgeTests` and `MarsBusTests` run again, and the corpus as well for
the three rules that touch the bus every processor store crosses.

**Forty-seven were caught on the first run, and the four that were not now are**, each by a case added for it:

| survived | why nothing saw it | the case added |
| --- | --- | --- |
| a flash page number is ten bits, not eight | every page the tests used was below 256 | `A_page_number_reaches_past_255` |
| the page buffer wraps every 128 bytes | no transfer into the flash was longer than a page | `The_page_buffer_wraps_every_128_bytes` |
| above the pak reads zero rather than repeating it | the test read `0x8000`, whose repeat is block 0 — which a formatted pak leaves zero | `Above_the_pak_reads_zero_and_keeps_nothing`, now at `0x8020`, whose repeat is the identity block |
| a changed chip is written every 300 frames | no test ran 300 frames | `A_changed_chip_is_written_on_the_three_hundredth_frame_without_being_asked`, on fields a couple of thousand cycles long |

The first two were predicted when the list was written; the third was not, and is the more instructive — the case was
written to catch a pak that repeats, and chose the one address where repeating and not repeating read the same. The
fourth retires §7's sentence that nothing but the code covered it:
~~"The every-300-frames write is covered by nothing but the code: a test would need 300 frames of a running machine."~~

**The corpus moved for none of the three bus rules**, 46 each time. It never stores below the ROM window, so the
widened latch (§3), the whole-word store to the second domain and the rule that only the ROM window hands a word back
are each held by one named case and nothing else.

## 9. What is not here, and what this is not evidence for

- **The real-time clock**, which the references name one Japanese title for. The cheap version is mupen64plus's — a status reply, a
  control block, and the host's time in the second — and it would want the ED header's clock bit or a table row to gate
  it, since §2's argument says a cartridge without the clock should not answer.
- **Busy periods.** No reference models how long an EEPROM write or a flash erase keeps its chip busy. A driver that
  waits a fixed time after each cannot tell a chip that finished at once; one that polled would.
- **The rumble pak and the transfer pak**, which `Mars_Gameplan.md` §6 defers until something needs them.
- **None of it is a measurement.** Every rule here is a reading of an implementation, weighed against two others. The
  places Mars departs from all three (§2, the silent EEPROM channel and clock) or from the referee (§3, banked SRAM)
  are the first places a console would be asked.
