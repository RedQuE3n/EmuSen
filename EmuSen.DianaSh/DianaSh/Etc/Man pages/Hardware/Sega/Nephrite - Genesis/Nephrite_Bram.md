# Nephrite_Bram — the Sega CD's backup RAM format, observed

*Written 2026-10-15 by the observer of the two-person method decided in `Nephrite_OpenBios.md` §5 Q2: someone who
does not write, read or review the open BIOS runs the player's Sega CD BIOS as a black box, examines the backup RAM
images it leaves, and states the format in plain words. The writer builds `_BURAM` and `_mburam` from this page
alone.* Every point below is either **observed** (an image or a screen the player's BIOS produced, cited by the run
that produced it), **documented** (stated by a source in §2, cited), or **argued** (reasoned from those, marked so).
A point the page does not state is unknown; §10 lists the ones the evidence left open.

The page fills the gap `Nephrite_OpenBios.md` §2.2 calls G3. It covers the internal 8 KiB backup RAM and the RAM
cartridge, which were found to use one format.

---

## 0. Summary

**Settled.** The footer (§3), to the byte, with what the BIOS checks of it; a freshly formatted image of every size
observed (§4); the directory's place, its two-entries-per-block layout, the entry's fields and the free-block
accounting (§5); the **block code** that every directory block and every protected data block uses, reproduced
byte for byte on all 229 BIOS-written blocks collected (§6); allocation, the order of a write, and what a delete
leaves behind (§7); the cartridge's sizes and their footers (§8).

**Open.** What a write onto an existing name does; who writes block 0 during a game and why; every register-level
return value; the read path's handling of a bad CRC or bad low bits; `BRMDIR` and `BRMVERIFY` beyond the manual
(§10).

---

## 1. Method

### 1.1 What was run

- **Nephrite** at WiseMan `4e8be3d2`, built from a copy of the crate kept under
  `~/.cache/emusen/probe/mcd-bram/nephrite` (not in the repository) with three probe-only changes: the setting `ramcart` makes a cartridge of a size taken from
  the environment (for §8); a disc whose first sector is not `SEGADISCSYSTEM` is accepted (for the audio disc below);
  two save-memory accessors are public, so that a tool outside the crate can write and read the images. The
  cartridge's ID byte in Nephrite is `log2(size / 8 KiB)`; nothing else of the machine was changed.
- **The player's BIOS**: `Sega CD (U) - M1 V1.10` from the player's set, given to Nephrite as a file, for every
  observation on this page. No other version was run.
- **A probe tool** (`~/.cache/emusen/probe/mcd-bram/tool`) that runs a disc with chosen pad presses, starts from a
  given `.brm`/`.crm` image or saved state, and writes the pictures, the state and the two backup RAM images. A
  variant records the internal image on every frame it changes.
- **Two games** that save: Sonic CD (US) and Lunar – Eternal Blue (US), each booted on a formatted or an unformatted
  image and run to its title.
- **The BIOS's memory manager.** In Nephrite the player's BIOS stays on its first picture when the drive is empty
  from power-on (pressing Start does nothing; recorded as a negative result, §11). It was reached instead with an
  audio-only disc made from the player's Chuck Rock II tracks 2–3 (a cue sheet in the probe folder pointing at the
  library's files, read only): the BIOS shows its CD player, the panel's EJECT opens the tray, the disc is removed,
  CLOSE is pressed, and an OPTION button appears; OPTION shows "DATA STORAGE INFORMATION" and then the menu FORMAT /
  ERASE ITEM / COPY for the built-in memory and the cartridge. The path is scripted (`tomenu.sh`, `oracle.sh`,
  `copy.sh`, `erase.sh` in the probe folder).

### 1.2 What was examined

Only backup RAM images and the memory manager's screens. No memory of either CPU was read, no register, PC or bus
watch was set, and nothing of the BIOS's code was opened, dumped or traced.

### 1.3 Crafted images as inputs

Once the layout of §3, §5 and the high bits of §6 had been read off BIOS-written images, images built from that
layout were **given to the player's BIOS as inputs**: corrupted copies of a game's image (to see what the manager's
listing then shows), footers with single bytes changed (to see which the BIOS checks), and files whose directory and
data were built by the probe's own encoder (to have the manager copy them, so that the BIOS **re-encodes** chosen
data and leaves the result on the cartridge). The evidence is still only what the BIOS showed or wrote; the crafted
inputs are the probe's, and every format fact drawn from them is a fact of a BIOS-written output. This extends §5
Q2's method (which names games' images, the manager and the player's files) and is stated here so that the tester can
judge it. The images and their outputs are under `~/.cache/emusen/probe/mcd-bram/` (`images/` for inputs, `runs/<run>/`
for each run's `brm.bin`, `crm.bin`, screens and state).

### 1.4 Files the player already had

None. `~/.config/EmuSen/Saves`, the ROM library, `~/Documents` and the sandbox's `Usr/Home/Saves` hold no `.brm`,
`.crm`, `mcd.backup` or `mcd.ramcart` file. The only `.brm` files on the disk are reference emulators' outputs under
`~/.cache/emusen/probe/nephrite/` (stage 9's `s9/` and a run folder of Genesis Plus GX); they were found by name and
not opened, being neither the player's nor the BIOS's.

---

## 2. Sources and reading log

- **BM**: Sega, *Mega-CD BIOS Manual* v2.00, 1992-02-24, §7-1 (pp. 38–43), read from page images; the text layer of
  §5–§6 (pp. 33–37) was passed over on the way.
- **TB**: *Sega-CD Technical Bulletins*: "MEGA-CD BIOS Specifications" with BRMWRITE's correction and "How to access
  Back-Up Ram Cartridge" (PDF p. 16, bulletin #1), "Cautionary Notes when Using the Backup RAM" (PDF pp. 20–22,
  bulletin #2, 1993-01-12), "9. Backup RAM" (p. 9, PDF p. 33), from page images and their text layer.
- **SS**: *Sega-CD Software Standards*, the "BACK-UP RAM" section (its text layer): file names are registered with
  Sega, saves of one game share the first eight characters, the option command on the control screen formats.
- **EM**: `docs/bram/emutos-mcd_bram-filesystem.md` (EmuTOS-MCD), whole: used for the footer and the odd-byte layout,
  and quoted in §9 for register values it reports from a console.
- **SM**: SpritesMind topic 826, "CD Backup RAM Cart", both pages, as text: the cartridge's memory map, ID register
  and write-protect latch.
- **Nephrite, to run it**: `Nephrite_OpenBios.md` §1–§5 (the method's decision and the manual's summary);
  `Nephrite_Plan.md` §4.4 and `Nephrite_Attachments.md` §11.1 (where the battery files go); the Nephrite folder's
  `README.md`; the crate's `examples/disc.rs` and `Cargo.toml`. To drive the machine from a tool, also read: in
  `src/v1.rs` the button table, the battery and media calls, `create_disc` and `open_disc`; in `src/machine.rs` the
  public functions' signatures and `battery_id`/`battery_bytes`; in `src/mcd/mod.rs` the `RamCart` struct and the two
  lines that read and write the backup RAM by address; in `src/mcd/frontend.rs` the ram-cartridge setting and the
  save-memory accessors; in `src/state.rs` the state's header; in `emusen-native` the `Create`, `Insert` and `Core`
  signatures; lines of `src/v1.rs`'s tests that a text search printed. `build.rs` was searched for relative paths
  (no line matched) before the crate was copied. None of these holds anything of the backup RAM's format.
- **Corpus records**: `DOCS.txt` §X (the screening verdicts) and its fetch records for the three Sega manuals.
- **Listed by name only, not opened**: the other files of `docs/bram/` (they appeared in a directory listing); the
  `.brm` files named in §1.4.

**Not opened, by the rule:** the BIOS ROM or any part of it; `src/mcd/bios/*.s`, `src/mcd/bios.rs`,
`src/mcd/openbios.rs` (the probe's copy of the crate contains them, copied unopened and built as part of the crate);
`Nephrite_Native.md` (no section of it was read; its length was counted); `Nephrite_Disputes.md` (likewise);
`superctr_buram.c`, its README and `savefileconverter_*.js`; the `cd-log` logs, MiSTer's documents and RTL,
SpritesMind t2061 and t3020, the railgun TMSS page, and `s9/`; any other emulator's source.

---

## 3. The footer

The last 64 bytes of an image (offset `S − 0x40`, `S` the image's size):

| Offset from the end | Bytes | Meaning |
|---|---|---|
| `−0x40` | `5F` × 11, `00 00 00 00`, `40` | as written by a format; contents not checked by the BIOS where tested (below) |
| `−0x30` | free blocks, 16-bit big-endian, written **four times** | blocks of 64 bytes not used by files or the directory (§5.4) |
| `−0x28` | number of files, 16-bit big-endian, four times | |
| `−0x20` | `SEGA_CD_ROM` `00` | the format's identifier; also what the manager prints after "FORMAT IS:" |
| `−0x14` | `01 00 00 00` | |
| `−0x10` | `RAM_CARTRIDGE___` | present on the internal RAM too |

This confirms EM's layout in every byte. EM's statement that "a volume is Sega's if and only if the last 0x20 bytes
match" is **not** what the player's BIOS does; the observed checks are:

- **The identifier** `SEGA_CD_ROM\0` (12 bytes at `−0x20`) is compared exactly. Each of its bytes with bit 0
  flipped made the volume unusable: 9 of the 12 were shown as another format ("FORMAT IS:" with the changed name,
  0 items, 0 free), and the three that turned a character into one outside `0`–`9`, `A`–`Z`, `_` (`A`→`@` at `−0x1D`,
  `_`→`^` at `−0x1C` and `−0x19`) were shown as unformatted ("PLEASE FORMAT"). `SEGA_CD_RAM` showed as another format.
  *Argued:* the other-format/unformatted split follows the name's characters; the observations are consistent with
  that and test nothing else.
- **`01 00 00 00`** at `−0x14`: each of the four bytes flipped was accepted.
- **`RAM_CARTRIDGE___`**: the volume counts as formatted when **at least one of its first three longwords**
  (`RAM_`, `CART`, `RIDG`) is intact; the fourth (`E___`) is not checked. Observed: any single byte flipped,
  either half flipped, longwords {0,1,2,3} with any one of 0, 1 or 2 intact → accepted; only longword 3 intact, all
  16 bytes flipped, all 16 bytes zero, all 16 `A` → unformatted.
- **The `5F…40` row**: `−0x40` changed and `−0x31` changed were each accepted. The other 14 bytes were not tested.
- **The counts**: the manager shows a value held by at least three of the four copies (one copy changed, of either
  count, made no difference; three of four changed showed the new value). With two copies against two, or four
  different values, the manager showed 0 items and 0 free. The free count is **read, not recomputed**: all four set
  to 112 showed 112 although the files used 11 blocks.

Runs: `runs/ft_*`, `runs/fb_1fe0` … `fb_1fff`, `runs/rc_*`, `runs/rd_*`, each from `images/foot_*`, `fb_*`, `rc_*`,
`rd_*`, all edits of the Sonic CD image of §5.

---

## 4. A freshly formatted image

**Format writes the footer and nothing else.** The manager's FORMAT on an all-zero internal RAM changed only bytes
`0x1FC0`–`0x1FFF` (`images/int_before_fmt_zero.bin` → `images/int_fmt_over_zero.bin`, run `fmt5`). On an internal RAM
and a 128 KiB cartridge both filled with `A5`, FORMAT changed only their last 64 bytes; every other byte stayed `A5`,
the directory area included (`images/fill_a5*.bin` → `runs/fA5b/brm.bin`, `crm.bin`). A freshly formatted image is
therefore "the previous contents, then the footer"; a new machine's image built by the open BIOS is zeros then the
footer, which is what the BIOS produces from zeros.

The footer of a fresh image, with `F = S/64 − 3` as the free count:

```
5F 5F 5F 5F 5F 5F 5F 5F 5F 5F 5F 00 00 00 00 40
FH FL FH FL FH FL FH FL 00 00 00 00 00 00 00 00
53 45 47 41 5F 43 44 5F 52 4F 4D 00 01 00 00 00
52 41 4D 5F 43 41 52 54 52 49 44 47 45 5F 5F 5F
```

| Memory | Size `S` | Free `F` | Observed in |
|---|---|---|---|
| Internal | 8 KiB | 125 (`007D`) | `images/int_fmt_over_zero.bin` |
| Cartridge | 8 KiB | 125 (`007D`) | `runs/cs8192b/crm.bin` |
| Cartridge | 16 KiB | 253 (`00FD`) | `runs/cs16384b/crm.bin` |
| Cartridge | 32 KiB | 509 (`01FD`) | `runs/cs32768b/crm.bin` |
| Cartridge | 64 KiB | 1021 (`03FD`) | `runs/cs65536b/crm.bin` |
| Cartridge | 128 KiB (Sega's) | 2045 (`07FD`) | `images/cart128_fmt_over_zero.bin` |
| Cartridge | 256 KiB | 4093 (`0FFD`) | `runs/cs262144b/crm.bin` |
| Cartridge | 512 KiB | 8189 (`1FFD`) | `runs/cs524288b/crm.bin` |

The three reserved blocks are the footer, the first directory block and block 0 (§5.4, §7.5). The manager's info
screen read back each cartridge's free count as in the table.

---

## 5. The directory

### 5.1 Where it is

Directory blocks are 64 bytes each and grow **downward** from the footer: directory block `j` (from 0) occupies
`S − 0x80 − 0x40·j` to `S − 0x41 − 0x40·j`. Each block is one codeword of the block code (§6): 64 stored bytes that
carry 32 bytes of directory data, **two entries of 16 bytes**.

### 5.2 Which entry is where

Entries are numbered from 1 in the order the manager lists them, which was observed to be the order the files were
created (Lunar created `WD_LUNAR2_3`, `_2`, `_1`, `_0` in that order, `runs/lunarW`, and the manager lists them 1 to 4
in that order, `runs/mgrL2`). Directory block `j` holds entries `2j+1` and `2j+2`; in its 32 data bytes `U`:

- `U[16..31]` is entry `2j+1`;
- `U[0..15]` is entry `2j+2`.

### 5.3 An entry

| Bytes | Field |
|---|---|
| 0–10 | the name, 11 bytes of ASCII, as the game gave it (`SONICCD____`, `WD_LUNAR2_3`); no terminator |
| 11 | the flag: `00` for a normal file, `FF` for a protected one |
| 12–13 | the first data block, 16-bit big-endian |
| 14–15 | the size in 64-byte blocks, 16-bit big-endian; for a protected file the number of stored 64-byte blocks, each holding 32 bytes of the file |

Observed: Sonic CD's entry `SONICCD____`, `00`, `0001`, `000B` (`images/int_sonic1.bin`, manager: "SONICCD____ 11");
Lunar's four entries with starts `0001`, `000F`, `001D`, `002B` and size `000E` each (manager: "14" each); a crafted
protected file's `FF` kept by the BIOS when it copied the file (§6.6), its size field `0004` shown as "4" for four
stored blocks. The name is not stored as text in the image, since the block code spreads it across the stored bytes'
high six bits (§6.2); it is plain text once decoded. EM's remark that "the name is not stored as text" describes the
same thing.

### 5.4 Counting

With `n` files, the directory is accounted as **`(n + 2) div 2` blocks**, room for `n + 1` entries (BM's note to
BRMSTAT, "the free block are treated as one file", fits this), and

> free = `S/64` − 2 − (sum of the files' sizes) − `(n + 2) div 2`

(the 2 being the footer and block 0). Observed for 0 files (125, and every cartridge of §4), Sonic CD's one file
(128 − 2 − 11 − 1 = 114), Lunar's files one to four (111, 96, 82, 67 as each was written, `runs/lunarW`), the erase of
§7.3 (115 before, 117 after), and a 121-block file on the cartridge (2048 − 2 − 121 − 1 = 1924,
`runs/units0`). A directory block is **written** only when an entry is placed in it: with two files the second
accounted block was still unwritten (`runs/lunarW/w002254.brm`); it was written with the third.

### 5.5 The slot without an entry

When `n` is odd, `U[0..15]` of the last block holds no entry; the BIOS ignores it. Observed contents: sixteen zeros
(the first file on a fresh internal RAM, from Sonic CD and from Lunar, and every copy onto an empty cartridge), and once
**a copy of the entry in the same slot of the previous block**: when Lunar's third file started directory block 1,
its unused slot held `WD_LUNAR2_2`'s entry (start `000F`), which is block 0's `U[0..15]` at that moment
(`runs/lunarW/w002255.brm` at `0x1F40`). *Argued:* the BIOS keeps one 32-byte directory buffer and changes only the
entry it writes. Blocks beyond `(n+1) div 2` are left as they were: after an erase the old block stayed in place
(§7.3).

---

## 6. The block code (the "protect" encoding)

Every directory block, and every data block of a protected file, is 32 data bytes `U` coded into 64 stored bytes.
Normal files' data blocks are not coded (§7.1).

### 6.1 The message

`C` = CRC-16 of `U` with polynomial `0x1021`, initial value 0, bits in most-significant-first order, no reflection
and no final XOR (the parameters known as CRC-16/XMODEM). The message is 36 bytes:

> `M` = `C` (big-endian) ‖ `U[0..31]` ‖ `C XOR FFFF` (big-endian)

### 6.2 Six-bit symbols

`M`'s 288 bits, taken most-significant-bit first, are cut into 48 six-bit symbols `s[0..47]`. Stored byte `i`
(`0 ≤ i < 64`) carries symbol `s[i]` in its **bits 7–2**; its bits 1–0 are §6.4's check bits. Symbols `s[48..63]` are
§6.3's parity. Seen as an 8 × 8 grid, stored byte `i` is row `i div 8`, column `i mod 8`.

### 6.3 Eight Reed–Solomon codewords over GF(64)

The field is GF(64) with the polynomial `x^6 + x + 1`; a symbol's bit 5 is the coefficient of `α^5` and bit 0 of 1,
and `α` is the symbol `000010`. The 64 symbols form eight codewords `K = 0 … 7`, each six data symbols `d0 … d5` and
two parity symbols `p0`, `p1`:

| K | d0 | d1 | d2 | d3 | d4 | d5 | p0 | p1 |
|---|---|---|---|---|---|---|---|---|
| 0 | 0 | 9 | 18 | 27 | 36 | 45 | 48 | 56 |
| 1 | 1 | 10 | 19 | 28 | 37 | 46 | 49 | 57 |
| 2 | 2 | 11 | 20 | 29 | 38 | 47 | 50 | 58 |
| 3 | 3 | 12 | 21 | 30 | 39 | 8 | 51 | 59 |
| 4 | 4 | 13 | 22 | 31 | 40 | 17 | 52 | 60 |
| 5 | 5 | 14 | 23 | 32 | 41 | 26 | 53 | 61 |
| 6 | 6 | 15 | 24 | 33 | 42 | 35 | 54 | 62 |
| 7 | 7 | 16 | 25 | 34 | 43 | 44 | 55 | 63 |

(the entries are symbol indices: `d_k = K + 9k` for `k < 5`; `d5 = 45 + K` for `K < 3` and `9K − 19` otherwise;
`p0 = 48 + K`, `p1 = 56 + K`, so the parity fills rows 6 and 7 of the grid, column `K`.)

The codeword `c(x) = d0·x^7 + d1·x^6 + d2·x^5 + d3·x^4 + d4·x^3 + d5·x^2 + p0·x + p1` has the roots 1 and `α`:

> `S0 = d0 + … + d5`, `S1 = Σ d_k·α^(7−k)`; `p0 = (S0 + S1) / (α + 1)`, `p1 = S0 + p0`

(addition is XOR).

### 6.4 The column check (the low two bits)

Each grid column `c` (stored bytes `c, c+8, …, c+56`) carries 16 check bits, two in each byte's bits 1–0, computed
from that column's eight six-bit symbols (rows 0–7, parity rows included) by a 16-bit register `R` seen as eight
two-bit cells:

```
R = 0
for row r = 0 .. 7:
    o = R >> 14                       ; the top cell
    R = (R << 2) & 0xFFFF
    R ^= (o << 8) ^ (o << 6) ^ (o << 4) ^ o
    for each bit b (0 .. 5) set in the symbol at (r, c):
        R ^= T[b]
T = { 0x000E, 0x003E, 0x00FE, 0x03FE, 0x0FFE, 0x3FFE }   ; T[b] = 4^(b+2) − 2
after the loop, for each row r: stored byte (r, c) bits 1–0 = (R >> (14 − 2r)) & 3
```

(*Argued:* the register is a polynomial with two-bit cells reduced by `x^8 + x^4 + x^3 + x^2 + 1`; the table below is
what was measured, and the loop reproduces it exactly.) The same map as a table, each set bit `b` of the symbol in row
`r` contributing the value shown, all contributions XORed:

| row | b5 | b4 | b3 | b2 | b1 | b0 |
|---|---|---|---|---|---|---|
| 0 | 7F03 | BF3C | 4FCF | B30F | 8C3F | 83F3 |
| 1 | DF3C | 2FCF | D30F | EC3F | E3F3 | E000 |
| 2 | 37CF | CB0F | F43F | FBF3 | F800 | 3800 |
| 3 | CD0F | F23F | FDF3 | FE00 | 3E00 | 0E00 |
| 4 | F3BF | FC73 | FF80 | 3F80 | 0F80 | 0380 |
| 5 | FC13 | FFE0 | 3FE0 | 0FE0 | 03E0 | 00E0 |
| 6 | FFF8 | 3FF8 | 0FF8 | 03F8 | 00F8 | 0038 |
| 7 | 3FFE | 0FFE | 03FE | 00FE | 003E | 000E |

### 6.5 Test vectors (all written by the player's BIOS)

Sonic CD's directory block, `U` = 16 zeros ‖ `SONICCD____` `00` `0001` `000B` (`images/int_sonic1.bin` at `0x1F80`):

```
00 D4 63 00 00 03 03 00 03 03 03 00 03 00 00 03
00 03 00 00 00 03 03 00 53 D2 F5 3A 48 50 35 0F
47 14 F5 7E 5C D4 F3 03 00 03 12 00 0A FF CA A6
F5 27 ED 22 47 FA 22 96 6C A5 88 14 48 48 0A BB
```

Lunar's first two entries, `U` = `WD_LUNAR2_2 00 000F 000E` ‖ `WD_LUNAR2_3 00 0001 000E`
(`runs/lunarW/w002254.brm` at `0x1F80`):

```
1D 6F B7 5C 44 17 F6 30 56 53 E7 05 50 8D 27 7E
30 82 00 03 0F C0 00 39 55 D1 47 7E 4E 17 55 3A
42 56 22 CA 5C CF 33 00 00 03 12 00 0C B8 27 11
78 C6 1D 2B 53 F6 BB 74 AF EB 95 E7 21 FA E8 8B
```

A protected data block of 32 zeros (`runs/pcopy1/crm.bin`, block 1):

```
00 00 00 00 00 00 00 00 03 03 03 00 00 03 03 03
00 00 00 00 00 00 00 00 03 03 03 00 00 03 00 00
00 03 03 00 00 00 03 03 00 03 03 00 00 3C FF FF
44 09 09 00 00 00 00 00 78 F6 F6 00 00 00 03 03
```

A protected data block of `THE QUICK BROWN FOX JUMPS OVER T` (`runs/pcopy1/crm.bin`, block 4):

```
53 0C A6 50 4B 11 50 81 53 56 55 24 42 D2 B1 81
41 95 24 3F 56 D1 E1 81 44 90 F6 63 21 12 A6 55
4D 55 05 4E 22 11 F6 59 47 56 22 82 55 2B FF 14
24 99 99 CA 60 14 21 60 6C 30 BD A6 9F D8 8D E2
```

### 6.6 How it was established

1. **The data in the high bits.** Read as a most-significant-first stream of six-bit fields, the high bits of
   Lunar's first directory block spell `WD_LUNAR2_3` from symbol 24; the whole block decodes to two bytes, 16 zeros,
   the 16-byte entry, two bytes and twelve more (`runs/cpL1`). The two leading bytes satisfied CRC-16/XMODEM over the
   32 data bytes on all eight directory blocks then collected, of 32,768 polynomials searched in both bit orders,
   and the two after the data were their complement.
2. **Linearity.** The four Lunar entries copied each alone onto an empty cartridge (`runs/cpL1`–`cpL4`) differ only in
   the name's last character; their stored differences XOR as the characters do.
3. **Codeword membership**, from the manager's listing used as a decoder (`runs/orc/`): one six-bit symbol anywhere
   in Sonic CD's block corrupted (`XOR FC`) was corrected (the listing unchanged); eight consecutive symbols were
   corrected, twelve were not. Each of the 64 symbols was then corrupted together with an anchor symbol of its
   predicted codeword (56 runs: every listing changed) and with an anchor of the next codeword (64 runs: every listing
   unchanged). Two errors in one codeword are not corrected; one in each codeword is.
4. **The field and roots.** Pooled over all codewords, in §6.3's order, the symbol vectors span six dimensions over
   GF(64) with `x^6 + x + 1` and seven with each of the other five primitive polynomials (the eight directory blocks
   then collected, and later all 302 distinct codewords the BIOS wrote). Reed–Solomon parity with roots `α^0, α^1`
   fitted every codeword of every block only in §6.3's order; tried were the first five data symbols in increasing
   index with `d5` in each of the six places, and all eight in increasing index, each with both polynomial directions,
   both parity orders and all 63 first roots.
5. **The low bits by chosen data.** The probe's encoder (§6.1–§6.3, low bits zero) built protected files of 12 blocks
   each holding a zero block, the 256 single-bit blocks and 31 random ones (`images/craft_lo00` … `lo23`), given to the
   player's BIOS on the internal RAM; the manager's COPY to the cartridge read them and **wrote them back coded by the
   BIOS** (`runs/lo00` … `lo23`). Fifteen of the 24 copies kept the file protected (§11 for the other nine); their 180
   blocks, with the directory blocks, gave 567 distinct columns. The low bits are one linear function of the column's
   48 high bits, the same for every column, determined at full rank (49 of 49 with the constant, which is 0) and
   consistent on all 567. The register of §6.4 reproduces it.
6. **Check.** The complete encoder of §6.1–§6.4 reproduces, byte for byte, all 229 blocks with low bits that the BIOS
   wrote in these runs: the games' directory blocks, the manager's copies and erase, and the copied protected data.
   A second implementation written from §6.1–§6.4's words alone (`an/frompage.py`) reproduces the same 229; the only
   block it does not reproduce is a crafted input the BIOS never wrote (`runs/crA`).

### 6.7 What the read side was seen to do

Observed through the manager's listing only:

- **One symbol error per codeword is corrected**, anywhere in the block, parity symbols included; two in one codeword
  are not, and the listing then shows the stored data as it is (`SONICCD____` with symbols 24–35 corrupted showed
  `, 0 ICC; __`).
- **The low two bits are ignored**: all 128 flipped, alone or with a corrected symbol error, changed nothing.
- **The CRC is not checked**: a block whose data and parity were made consistent for `SONICCE____` showed
  `SONICCE____` with the old CRC as well as with the right one (`runs/ct_stalecrc`, `ct_goodcrc`).
- **Nothing is written back**: every corrected or uncorrected read left the image unchanged.

What `BRMREAD`/`BRMSERCH` return in these cases is not known (§10).

---

## 7. The data area

### 7.1 Blocks

Block `k` is bytes `64k` to `64k + 63` of the image. **Block 0 is never part of a file** (TB p. 9's "leading reserve
area"); files start at block 1. A normal file's blocks hold its bytes plainly, 64 per block, in order: Sonic CD's
11 blocks at `0x40`–`0x2FF` hold its records in the clear (`00 05 00 00` repeated, then `0B 0B 0B 00` repeated), and EM
reports the same of a console. A protected file's blocks each hold the block code (§6) of 32 bytes of the file, in
order.

### 7.2 Writing a file

- **Where**: the first block after the last file's last block. Observed: Lunar's four files at 1, 15, 29, 43; every
  copy onto an empty cartridge at 1; Sonic CD's file at 1.
- **Order**, frame by frame (`runs/sonicW`): the data blocks changed in one frame (1,337), the directory block and both
  counts in the next (1,338). Lunar's four files each changed the directory and the counts in one frame (2,252, 2,254,
  2,255, 2,258); their data blocks were zeros and showed no change.
- **The directory**: the block holding the new entry is coded and written whole (both of its entries), and the four
  copies of each count are written with the same value.
- **Copies** are position-independent: Sonic CD's file copied to the cartridge has the same entry bytes and data bytes
  as on the internal RAM, at the cartridge's own offsets (`runs/cartG`).

### 7.3 Deleting a file

The manager's ERASE ITEM of the second of three normal files (`images/craft_del3.bin`: `AAAA` blocks 1–3, `BBBB` 4–5,
`CCCC` 6–9, each block marked by its file and index; `runs/del2`):

- the third file's four blocks were moved down to 4–7, and its entry, now the second, has start `0004`;
- blocks 8–9 still hold the third file's old last two blocks: a delete does not clear what it vacates;
- directory block 0 was rewritten with entries `AAAA` and `CCCC`; directory block 1, now beyond the two files, was
  left as it was, still holding `CCCC` with start `0006`;
- files 3 → 2, free 115 → 117 (the two blocks freed; the directory still counts two blocks).

This is TB p. 9's "the directory and the data is re-compressed". That the manager's erase is `BRMDEL` is assumed
(*argued*: the manager is the BIOS's own program and TB names `BRMDEL` as the call that compresses).

### 7.4 What a game was seen to write besides its files

**Block 0 changes while a game runs.** Under both games, block 0's first 16 bytes became a 4-byte pattern repeated
four times, rewritten every 40 to 95 frames with its second word 2 lower: on a formatted RAM `00 00 FF FE`, then
`FF FC`, `FF FA`, …, from Sonic CD's frame 1,162 (`runs/sonicW`) and Lunar's 2,360 (`runs/lunarW`, where from frame 2,647 the first
word was `FF FE` too); on Sonic CD's second boot it went on from the value stored (`FF F2`, `FF F0`, …, `runs/sonicW2`);
on an unformatted RAM Sonic CD left `00 00 FF F8`. The memory manager never changed block 0. Who writes it, through
which call, and what it means is not known (§10).

### 7.5 Reserved blocks

Block 0, the first directory block and the footer are the three blocks a fresh volume subtracts (§4). Every further
directory block is taken from the free count as §5.4 states.

---

## 8. The RAM cartridge

- **Memory map, ID and write protection** (SM, MacDonald 2010 and later posts; *documented*, not observed here): the
  ID at `$400001`–`$4FFFFF` reads `04` on D7–D0 for Sega's 128 KiB cartridge, and the ID is the size, `1` 16 KiB,
  `2` 32 KiB, `3` 64 KiB, `4` 128 KiB, with 256 KiB and 512 KiB cartridges reported as `5` and `6` (Chilly Willy; Eke
  for 512 KiB as `6`); the RAM is on D7–D0 (odd bytes), Sega's one chip at `$680000`–`$6BFFFF` with room for a second at
  `$6C0000`–`$6FFFFF`; a write to
  `$700000`–`$7FFFFF` latches D0, `0` enabling writes and `1` protecting, reset to `0` by `/MRES` and not by the reset
  button. The cartridge is reached through `_mburam` with the same calls as `_buram`, by the main CPU (TB #1).
- **The image** (`.crm`, file 10) is the cartridge RAM's bytes in order, one per odd bus byte, as the internal
  `.brm` is the internal RAM's (block `k` at byte `64k`).
- **The format is the internal RAM's**: the same footer at its own end with `F = S/64 − 3` (§4), the same directory,
  block code and allocation; only the size differs (§4's table: 8 KiB to 512 KiB formatted and read back). Sizes other
  than 128 KiB were observed through Nephrite's cartridge with its size changed in the probe's copy, its ID being
  `log2(S / 8 KiB)` as Nephrite models it, which agrees with SM's codes for 16 KiB to 128 KiB and with Eke's 6 for
  512 KiB. The player's BIOS formatted and read every one.

---

## 9. The `_BURAM` calls

What BM and TB document, then what the images show. "Manager" means the BIOS's memory manager, whose calls cannot be
seen; a behaviour seen through it is attributed to a call only where marked *argued*. No register value was observed;
the values quoted from EM are EM's.

| Call | Documented (BM §7-1 unless noted) | Shown by the images |
|---|---|---|
| **BRMINIT** | `a0` scratch of `$640` bytes, `a1` a 12-byte string buffer; carry clear: a Sega-formatted RAM, `d0.w` its size, "`$2(000)`–`$100(000)` bytes"; carry set: `d1.w` 0 no RAM, 1 unformatted, 2 other format; `a1` the display strings. Call before the others (TB #1 "Add preceding BRMINIT"). With write protection on, an unformatted RAM is treated as absent. | The manager's three outcomes follow §3's checks: "FORMAT IS: SEGA_CD_ROM", "FORMAT IS: <name>" with 0/0 (other format), "PLEASE FORMAT". EM reports `d0` = 2 for the internal RAM, in units of `$1000` bytes. The manager's own initialisation wrote nothing. |
| **BRMSTAT** | `d0.w` free blocks, `d1.w` files; "the free block are treated as one file"; negative counts mean formatting is needed. | Free and files are the footer's (majority of four, §3), not recomputed. §5.4's accounting. EM reports 125 and 0 on a fresh internal RAM. |
| **BRMSERCH** | `a0` → 11-character name and a zero byte; carry clear if found; `d0.w` blocks, `d1.b` mode `0` normal or `−1` protected, `a0` the file's start address. | The entry's size and flag (§5.3). EM reports `a0` = `$FE0080` for a file at block 1 of the internal RAM, the data then on the odd bytes from `$FE0081`. |
| **BRMREAD** | name as above; `a1` buffer; returns `d0.w` blocks and `d1.b` mode; sizes in `$20` bytes protected, `$40` normal. | A protected file reads as 32 bytes per stored block, the decoded `U`s in order (seen in what the manager's copy wrote, §11). |
| **BRMWRITE** | `a0` → name, flag `$00`/`$FF`, `block_size.w` (`$40` or `$20` bytes per block by the flag); `a1` data; `d1.l` = 0 (TB #1's correction). Carry set if it cannot write. | §7.2: placement after the last file, data first, then the directory block and counts; §5.4's free accounting; the flag stored as `00`/`FF`; for a protected file the size field counts stored blocks. What it does when the name exists is not known. |
| **BRMDEL** | name; carry clear if deleted. The directory and data are compressed again; an interrupt during it may damage the file (TB p. 9). A file of 256 blocks or more may not be deleted correctly, and normal writing may then succeed only once (TB #2, item 2). | §7.3 (*argued* to be BRMDEL's). |
| **BRMFORMAT** | "First initializes the directory, then formats"; error if it cannot. With write protection on, returns an error. | §4: only the footer is written; nothing below it is cleared. |
| **BRMDIR** | `d1.l` high: entries to skip, low: buffer size in entries; `a0` → an 11-character template using `*`; `a1` buffer; carry set if the buffer is too small. | Nothing; the buffer's layout is not documented and was not observed. |
| **BRMVERIFY** | as BRMWRITE's parameters; carry clear if the RAM matches; else `d0.w` −1 data mismatch, 0 not found. With write protection on, "no match" unless the same name and contents were written. | Nothing. EM reports it answering "matches" for data altered in place, an anomaly it leaves unexplained. |

SS's rules for names (Sega registers them; a game's saves share their first eight characters) and TB #2's example
(`ALEX-KID-A0`, a name with `-` although BM lists `0`–`9`, `A`–`Z`, `_`) are the only statements on names beyond BM.

---

## 10. Gaps

Unknown, and not to be filled by guessing:

1. **Writing an existing name.** No game in the library saved twice in the frames run, and the manager refuses
   to copy onto an existing name by itself ("AN ITEM WITH THE NAME … ALREADY EXISTS. CANNOT COPY.", `runs/dupcopy`),
   so whether `BRMWRITE` overwrites in place, reallocates, or refuses was not seen. EM's console note that data can be
   changed through `BRMSERCH`'s pointer says nothing of `BRMWRITE`.
2. **Block 0's pattern** (§7.4): its writer, its call, its meaning, its starting value.
3. **Register returns** of every call (§9), including `BRMINIT`'s size units and `BRMSERCH`'s address for the
   cartridge.
4. **The read path's errors**: what `BRMREAD` or `BRMSERCH` return for a correctable or uncorrectable block, a wrong
   CRC or wrong low bits (§6.7 shows only that the listing ignores the CRC and the low bits, corrects one symbol per
   codeword and shows uncorrectable data raw).
5. **The unused directory slot** (§5.5) beyond the two cases seen.
6. **`BRMDIR`'s buffer** and **`BRMVERIFY`'s comparison**.
7. **Full RAM, failed writes, files of 256 blocks or more**: not exercised.
8. **Names**: whether the BIOS rejects characters outside `0`–`9`, `A`–`Z`, `_`.
9. **The footer's `5F…40` row**: 14 of its 16 bytes were not tested for being checked.
10. **Other BIOS versions**: everything here is from US Model 1 v1.10. SM reports Japan's v1.00 lacking support for
    the larger cartridges.
11. **Cartridge sizes other than 128 KiB** rest on Nephrite's cartridge model (§8), not on a cartridge.
12. **No game in the library writes a protected file**; the protected encoding's evidence is the BIOS re-encoding
    crafted files and its directory blocks.

---

## 11. Negative results

- **No player files** (§1.4).
- **The BIOS with an empty drive** stayed on its first picture in Nephrite for more than 2,000 frames, with or without the
  tray opened through the frontend, and ignored Start, A, B and C; with a game disc it showed "Press the START BUTTON" and
  booted the disc. The manager was reached only through an audio disc's panel (§1.1). Whether this is Nephrite's drive
  or the BIOS's behaviour with no disc was not examined.
- **The manager's COPY of a protected file** kept it protected, re-coded, for 20 of the 29 files of 4 to 12 blocks
  given to it (`runs/pcopy1`, `runs/size4` … `size12`, `runs/lo00` … `lo23`); for every file of 13 blocks or more
  (13, 14, 15, 16, 32, 48, 64, 96, 121, `runs/size*`, `runs/units*`) and for 9 of the 24 twelve-block files, it
  wrote a **normal** file of the same size whose first half held the decoded 32-byte blocks
  in order and whose second half held unrelated bytes. The choice depended on the data, not the timing (the same
  input with 1, 7 and 13 frames of delay gave the same result). Unexplained; the open BIOS has no memory manager, and
  only the decoded half was used as evidence (for BRMREAD, §9).
- **Hypotheses retracted on the way**: the low bits as a CRC-16 of the column (all 32,768 polynomials, four output
  orders), as GF(256), GF(16) or GF(4)-linear over the column's bytes, nibbles or pairs; codewords as grid columns,
  diagonals or symbols modulo 8 or 9 (modulo 9 fitted all but the column-8 symbols, which §6.3's `d5` places). A first
  test of a per-column low-bit function failed because a crafted, never-written block had entered the sample set;
  without it the function is exact.

---

## 12. Reproducing

The probe folder `~/.cache/emusen/probe/mcd-bram/` holds the tool (`tool/`), the scripts, the analysis (`an/`: the
encoder `enc.py`/`full.py`, the image builder `mkimg.py`, the fits), the input images (`images/`) and every run
(`runs/`), none of it in the repository. `python3 an/full.py` and `python3 an/frompage.py` re-check §6 against the
BIOS-written blocks.
