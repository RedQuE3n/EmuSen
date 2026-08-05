# Mercury (Game Boy) — cartridge, bus and boards

---

## 2. The header

Every Game Boy image carries its metadata at `$0100-$014F`. Mercury reads the title, the colour flag, the SGB flag, the cartridge type, and the ROM/RAM size codes.

The **title** is 16 bytes on a DMG cart but only 11 on a colour one, because the CGB reassigned the last five to a manufacturer code and the colour flag itself. Reading 16 on a colour cart appends garbage, so the length depends on `$0143`.

### 2.1 The colour flag

`$0143` is `$80` for "works on both, enhanced on a CGB" and `$C0` for "colour only"; anything else is a plain DMG cart. Modelled as `CgbSupport` from day one even though nothing branches on it yet — it is the seam colour support will hang on (`Mercury_Core.md` §1).

### 2.2 Sizes

ROM size code `n` means `2 << n` banks of 16K. Mercury reads it but treats the **file length as the authority**, because a trimmed or over-padded dump is common and the file is the thing actually being addressed.

RAM size codes are `$02`=8K, `$03`=32K, `$04`=128K, `$05`=64K — note that `$04` and `$05` are out of order, which is not a typo. Code `$01` was a 2K part that no released cartridge used.

**MBC2 is the exception:** it carries 512 half-bytes on the mapper chip itself and reports size code `$00`, so the size has to come from the cartridge type instead of the size byte.

### 2.3 The header checksum

`x = x - byte - 1` across `$0134-$014C`. A real boot ROM refuses to start when it disagrees. Mercury computes it and records the answer but never enforces it, because it never runs a boot ROM (`Mercury_Cpu.md` §5).

---

## 3. The memory map

| Range | What |
|---|---|
| `$0000-$3FFF` | ROM, bank 0 (or a banked window on MBC1 in advanced mode) |
| `$4000-$7FFF` | ROM, switchable bank |
| `$8000-$9FFF` | VRAM |
| `$A000-$BFFF` | Cartridge RAM, switchable |
| `$C000-$DFFF` | Work RAM |
| `$E000-$FDFF` | Echo of `$C000-$DDFF` |
| `$FE00-$FE9F` | OAM |
| `$FEA0-$FEFF` | Prohibited |
| `$FF00-$FF7F` | I/O registers |
| `$FF80-$FFFE` | High RAM |
| `$FFFF` | Interrupt enable |

**Echo RAM is not a copy.** The hardware simply does not decode the relevant address line, so `$E005` and `$C005` are the same storage cell. Games really do read through it.

The **prohibited region** reads back `$00` on a DMG rather than open bus.

---

## 4. The boards

### 4.1 No MBC

32K mapped flat, plus the optional unbanked 8K of RAM that types `$08`/`$09` add. Writes into ROM space do nothing at all.

### 4.2 MBC1

Two bank registers and a mode bit, and the mode bit is what makes this board confusing. `BANK1` is 5 bits and **cannot be zero** — a written 0 becomes 1, which is why bank 0 is unreachable through the high window and why a 32-bank cart has 31 usable high banks. `BANK2` is 2 bits and means either the top ROM bits or the RAM bank, depending on mode.

In mode 0 the low window is always bank 0 and RAM is always bank 0. In mode 1 the low window follows `BANK2 << 5`, which is the only way a 1MB+ cart reaches its upper half through `$0000-$3FFF`.

### 4.3 MBC2

The only board here whose RAM is on the mapper chip: 512 nibbles, echoed through the whole `$A000-$BFFF` window, with the high nibble reading back as open bus (`| $F0`). Its two registers are selected by **address bit 8, not the data** — `$0100` set means the ROM bank, clear means the RAM enable.

### 4.4 MBC3

Seven ROM bank bits (so 128 banks, still with the 0-becomes-1 rule) and a `$A000` window that points at either RAM or one of five real-time-clock registers, depending on whether the bank select is `$08-$0C`.

The clock is **latched**: writing 0 then 1 to `$6000-$7FFF` copies the running counters into the registers a game reads, so a game never sees seconds tick over mid-read. Bit 6 of the day-high register halts the clock; bit 7 is a sticky overflow that stays set until the game clears it. `Tick` advances it from the CPU clock.

### 4.5 MBC5

Nine ROM bank bits split across two registers (`$2000-$2FFF` low eight, `$3000-$3FFF` the ninth), and **the only board here that can genuinely select bank 0** through the high window — no 0-becomes-1 remap. Code written for MBC1 that relies on that remap breaks here.

On a rumble cart, bit 3 of the RAM bank register drives the motor instead of addressing a fifth RAM bank.

---

## 5. The timer

`DIV` at `$FF04` is the **upper 8 bits of a 16-bit internal counter** that increments every T-cycle. Writing to it zeroes the whole counter, not just the visible byte, which is also how a game deliberately resets the timer's phase.

`TIMA` does not count on a divided clock — it counts **falling edges** of one selected bit of that same counter, ANDed with the enable bit in `TAC`. The bit is 9, 3, 5 or 7 for `TAC & 3` of 0-3. Modelling it as an edge detector rather than "increment every N cycles" is what makes the DIV-write phase reset behave correctly, and games do use that.

On overflow `TIMA` reads **0 for four cycles** before `TMA` is loaded and the interrupt fires. Writing to `TIMA` during that window cancels the reload.

`MemoryBus.Tick` steps this one cycle at a time internally even when called with a whole instruction's worth, so no edge is skipped.

## 6. OAM DMA

Writing a page number to `$FF46` copies `$XX00-$XX9F` into OAM. Real hardware takes 160 machine cycles and locks the CPU out of most of the bus for the duration, which is why real games run the DMA trigger from a routine copied into HRAM. Mercury copies the whole page **immediately**. Nothing yet observes the difference; it will matter once the PPU exists.

## 7. The joypad

Eight buttons behind one register, read as two selectable nibbles. Only bits 4 and 5 of `$FF00` are writable — they select the d-pad half, the action half, both, or neither. A **pressed button reads 0**, not 1, and an unselected half reads all ones. Selecting both halves at once ANDs them together, which is real hardware behaviour and not a bug.

## 8. Named spaces

`ROM`, `VRAM`, `CARTRAM`, `WRAM`, `OAM`, `HRAM` and `CPUBUS` are what `mem`, `watch` and the cheat engine address. All but `CPUBUS` are direct array windows with wrapping; `CPUBUS` goes through the real decode, side effects included, which is why it is separate. `ROM` is read-only — a debug write must not corrupt the loaded image.

## 9. The battery save

Cartridge types carrying a battery (§4) keep `CARTRAM` across sessions in a `.srm` file, autosaved every 300 frames by `MercuryCore` and again on shutdown. `.srm` is the RetroArch spelling rather than the `.sav` most standalone Game Boy emulators use; EmuSen is internally consistent across its three cores instead.

**The path is still `Path.ChangeExtension(path, ".srm")` — beside the ROM.** Venus deliberately does not do this (`Venus_Memory.md` §2.4), and moving Mercury to match is a known, deliberately deferred change: it is user-visible and needs a copy-don't-move migration. See `EmuSen_Galaxia.md` §6.

Reads and writes go through `Galaxia`'s `AtomicFile`, so an interrupted autosave cannot truncate a live save (`EmuSen_Galaxia.md` §4). `LoadSram` copies whichever of the file and `Ram` is smaller; a cartridge with no battery or no RAM at all does neither.
