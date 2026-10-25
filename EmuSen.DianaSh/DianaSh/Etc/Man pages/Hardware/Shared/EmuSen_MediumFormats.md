# EmuSen_MediumFormats — the save media's file systems as crates without the standard library

*Built 2026-10-17, with the backend's step B3 (`EmuSen_Backend.md` §13).* Three crates in `EmuSen/Cores/Shared/`, one
per save medium whose file system EmuSen reads, and a fourth since 2026-10-25 (§6):

| Crate | Medium | Layout from |
|---|---|---|
| `emusen-psx-card` | the PlayStation memory card, `psx.card` | psx-spx, "Memory Card Data Format" |
| `emusen-n64-pak` | the N64 Controller Pak, `n64.pak` | N64brew, "Controller Pak/Filesystem" |
| `emusen-mcd-bram` | the Sega CD's backup RAM, internal (`mcd.backup`) and cartridge (`mcd.ramcart`) | `Nephrite_Bram.md` |
| `emusen-ps2-card` | the PlayStation 2 memory card, `ps2.card` (since 2026-10-25, §6) | ps2tek "SIO2 PS2 Memcard Filesystem"; Ross Ridge, "PlayStation 2 Memory Card File System" |

The marks are `Diamond_Plan.md`'s: **[M]** measured on 2026-10-17 with the tool named; **[R]** read from a source;
**[A]** argued; **[C]** a choice.

## 1. Why crates of their own, and why without the standard library

The same on-disk layout has two jobs: a host's card tool (the card manager, `memcard`, Mistress's Memory Cards
window) and an emulated console's firmware (Diamond's open kernel, `kernel/src/bu.rs`, and Nephrite's open BIOS,
`bios/bram.s`), and each was written twice (`EmuSen_Backend.md` F21). D14 of that page, decided on 2026-10-17, gives
the layout one owner: a crate with no standard library that an open firmware **may** build on, each core deciding
for itself, with a round trip holding the two equal until then [C]. The cores were not changed to adopt the crates.

So the crates allocate nothing and own no buffer: every function reads an image or edits one in place, and a
refusal leaves the image as it was [C]. A save is named by its place in the directory (an entry, a note, an index),
never by text: what a save is called, how its title reads and what words a refusal takes are the host's, in
`emusen-systems`' family modules (`psx::card`, `n64::pak`, `genesis::bram`), which reproduce the C# modules'
`ISaveMediumFormat` answers. The one exception is the Controller Pak's character set (`char_of`), which is how the
console itself writes a note's name [A].

## 2. The surface

Each crate gives its constants, `is_formatted` (or `kind` and `size_ok` for the backup RAM), the directory read as
an iterator of saves with their blocks or pages and whether each is damaged, the free count, `copy`, `delete` and
`format`. `copy` and `delete` check what the layout can check (formatted, a whole chain, room) and return an error
enum; the host checks first what only it can (a name, which it compares as `Encoding.ASCII` reads it) in the C#
module's order, so the refusal a player reads is the one the C# gave.

The backup RAM's `delete` takes the image and a copy to edit, `delete(image, index, edited)`, because a damaged
directory can place the files to be moved over the directory itself: the C# read every entry before it moved any
data, and reading from the untouched image is the same rule without an array of entries [C].

## 3. Damaged volumes, and what the C# did with them

Two cases were found while porting, both read from the C# and shown by the parity class [M]:

- **A backup RAM whose footer counts 255 or 256 files** (three of the four copies) puts the last entries' directory
  block below the volume: `SegaCdBramFormat.EntryOf` sliced at a negative offset and threw
  `ArgumentOutOfRangeException`, for a read as much as an edit. The crate returns `None` for such an entry and an
  error from an edit, the family module turns that into `Fault::Outside`, and the facade throws the same exception,
  so a player sees what they saw before. Whether a card tool should list such a volume rather than throw is a
  question for the tester, not answered here. *Decided by the tester on 2026-10-17 and built the same day
  (`EmuSen_Backend.md` §13.5): such a volume is listed as damaged, its readable entries shown with words saying why
  (`Problem`), a copy onto it or a delete from it refused, a save copied off it allowed, and nothing thrown; shown first
  by `SaveMediaParityTests.A_directory_reaching_outside_the_volume_is_listed_as_damaged_in_both` failing, in both
  halves.*
- **A delete over a directory whose later entries lie outside the volume** threw the same exception from the move's
  slice. The crate refuses it as `MoveOutside`, with the same outcome through the facade. The C#'s other two failure
  points in that statement (the target beyond the image, the target shorter than the range) cannot be reached, since
  a file's length is never negative and the target therefore lies at or below the range [A].

One was a defect, shown and fixed in both halves [M]: **the C# rewrote a name outside ASCII**. Its entry kept the name
as `Encoding.ASCII` reads it, and wrote that string back on every copy and on every delete that rewrote the entries
after the one deleted, so a byte past 7Fh became `?`. Shown on 2026-10-17 by `SegaCdBramFormatTests.
A_name_outside_ascii_is_copied_and_kept_byte_for_byte` failing on the C# module (a copied name's byte C1h written as
3Fh); the C# entry now keeps the stored bytes, the crate always did, and the name still *reads* with `?` as before.

## 4. A memory card's title: code page 932 and NFKC

The C# read a title frame with .NET's code page 932 and normalised it to NFKC, so a game's full-width title reads as
ASCII. Neither is in Rust's standard library, and no new dependency was taken [C]. Instead
`Platform/systems/src/psx/cp932.rs` holds three tables generated on 2026-10-17 from .NET 10.0.112 [M]:

- **every lead and trail byte** that decodes to something other than U+30FB, 9,603 pairs;
- **the compatibility decomposition** of each of the 332 characters that table decodes to whose decomposition is not
  itself;
- **the compositions** of the starters of those decompositions with their eight combining marks (U+0301, 0304, 0306,
  0308, 030A, 0338, 3099, 309A), closed over the composites, 249 entries.

.NET's decoder was measured at the same time [M]: a lead byte takes the next byte whatever it is, so an unmapped pair
(and a lead at the end) gives one U+30FB and consumes both bytes; 80h decodes to U+0080, A0h and FDh–FFh to U+F8F0 to
U+F8F3. `psx::shift_jis::nfkc` is the standard algorithm (decompose, order the marks by their classes, compose) over
those tables, and is complete for what code page 932 decodes to and for nothing else; it is not a general NFKC. The
parity class reads every lead with every trail, and every single byte, through a title on both halves.

## 5. The round trips with the firmware

- **Diamond's open kernel**: `kernel-harness/tests/card.rs`, `a_card_the_kernel_writes_reads_by_the_hosts_format_crate`,
  formats a blank card with the kernel's own `B(41h)`, makes and writes a one-block file, and reads the card with
  `emusen-psx-card`: one save under its name, its data, fourteen blocks free [M]. Beyond it, WiseMan's
  `DiamondShellTests` reads the cards the open kernel's card manager copied and deleted through the PlayStation's
  module, the library's unless `EMUSEN_SYSTEMS_NATIVE=0`.
- **Nephrite's open BIOS**: `src/mcd/openbram.rs`,
  `the_hosts_format_crate_reads_what_the_bios_writes_and_the_bios_what_it_copies`: the BIOS's format of zeros equals the
  crate's byte for byte; three files the BIOS writes (one protected) read with their names, starts, lengths and data;
  two copied in by the crate read back by `BRMREAD` and found protected by `BRMSERCH`; and a delete by the crate equals
  the BIOS's `BRMDEL` byte for byte [M].

Both are dev-dependencies of the cores' test rigs only; neither core's own code names a crate.

## 6. `emusen-ps2-card`, the PlayStation 2 card (2026-10-25)

*Built at Sapphire's stage S25, part A (`Sapphire_Native.md` §30; `Sapphire_Plan.md` §9.5). Its users are the ones §1
names: the host's card tools, through the card manager's format surface at Sapphire's part B (G11), and an open BIOS's
card module, which may build on it. Sapphire's own SIO2 does not use it: the console's card is raw pages, the file
system its BIOS's.*

### 6.1 What it reads

An image is the raw card: 16,384 pages of 512 bytes, each followed by 16 spare bytes, 8,650,752 bytes, which is what
other emulators' `.ps2` files hold [R ps2tek "SIO2 PS2 Memcards", its Get Specs; M: PCSX2 2.8.2's new cards are that
size]. The layout is Ridge's [R Ross Ridge, "PlayStation 2 Memory Card File System", public domain, at
<https://ps2savetools.com/ps2memcardformat.html>, read 2026-10-25; its first address, the University of Waterloo's
CS Club, redirects to a port no longer answering]: the superblock in page 0 (ps2tek's table gives the same fields);
clusters of two pages; a FAT of 32-bit entries, the top bit set for a cluster in use and the rest the next cluster,
all ones ending a chain, reached through the superblock's list of indirect clusters, which name the FAT's clusters;
directories of 512-byte entries (mode, length, created, first cluster, the "." entry's link to its parent, modified,
attribute, a name of up to 31 bytes), "." and ".." first, the root's "." standing for the root and holding its count
of entries; every cluster number relative to the first allocatable cluster.

### 6.2 What it does, and the choices

- **Read**: `Card::open` (the superblock checked, the root proved readable), `saves` (the root's directories past "."
  and ".."), `find`, `read` (a file's bytes from an offset, a cluster at a time), `clusters_of` (a save with everything
  in it), `free_clusters`.
- **Write**: `format`, `mkdir`, `create` (a file written whole), `delete` (a directory with everything in it: entries'
  EXISTS bit cleared, clusters freed), `copy_save` (a top-level save to another card, its modes and times kept). A
  refusal leaves the image as it was: room and names are checked first, and `copy_save` reads every cluster of the save
  before it writes one.
- **The free count** [A]: Ridge says Sony's drivers round the allocatable clusters down to a thousand, so that cards
  with different numbers of bad blocks show the same free space. The crate reads that as a limit on the clusters in
  use: a new card has 7,999 of its 8,000 free, the root taking one. What OSDSYS shows on a new card is to be compared at
  Sapphire's part B, when the player's BIOS formats one.
- **Format** [C, Ridge's "Standard Memory Card Layout"]: the superblock (magic, version 1.2.0.0, 512-byte pages, two
  a cluster, sixteen an erase block, 8,192 clusters, the first allocatable 41, its end 8,135, the backups blocks 1,023
  and 1,022, the indirect cluster 8, no bad blocks, type 2, flags 52h); the FAT in clusters 9 to 40, every entry
  7FFFFFFFh (Ridge requires only the top bit clear) but the root's, which ends its chain; the root at cluster 41 with
  "." (mode 8427h) and ".." (mode A427h, hidden); every other page erased. The modes and the free entry's value are
  choices to be compared with a BIOS-formatted card at part B.
- **Error correction** [R Ridge; M one lineage, `Sapphire_Disputes.md` D-226]: a 20-bit Hamming code for each 128-byte
  chunk in three bytes, the column parities' even groups in the first byte's low nibble and the odd in its high, the
  line parities' even and odd groups in the second and third, each parity stored complemented with the unused bits
  clear, so that an erased chunk's code is 77h 7Fh 7Fh. Every page read is checked unless its code bytes are erased
  (FFh), whatever the superblock's flags (Sony's 52h has Ridge's ECC bit clear): one wrong data bit is put right, one
  wrong code bit ignored, more refused as `Damage::Ecc`.
- **Damage**: `Length`, `Unformatted`, `Superblock`, `Ecc`, `Chain` (outside the card, a loop, a free entry in a chain,
  a chain shorter than its file) and `Directory` (a count its chain cannot hold, a directory whose first entry does not
  exist); each with `words()`, a sentence a host may show. Every refusal is a value: nothing panics on any image
  (§6.5).

### 6.3 What is not done

Bad blocks are honoured when allocating (no cluster in a listed block is given) but never found or added; the backup
blocks' recovery after a write cut short (Ridge "Backup Blocks") is the firmware's, not a host tool's, and is not
done; pages of 1,024 bytes and other geometries Ridge allows are refused as `Superblock`; names are bytes, their
character set the host's; the icon files a save carries (`icon.sys` and the icons) are read as files, their meaning
the host's at part B.

### 6.4 The referees' cards

PCSX2 2.8.2 makes a new card with every byte FFh (its `Mcd001.ps2` and `Mcd002.ps2`, MD5
914e26ce1e2661e14d39d87310d3cdd3, measured 2026-10-25), and formats it only through a BIOS; its folder card, read
through SIO2 from an IOP program, shows an empty folder as unformatted too. The crate reads such a card as
`Unformatted`. **A card formatted by a referee therefore needs a BIOS**, and the cross-check with one, both ways, waits
for Sapphire's part B (`Sapphire_Plan.md` §7, S25), as G11's test does.

### 6.5 Tests (measured 2026-10-25)

`cargo test --release` in the crate, **5 of 5**: the formatted card's layout, codes and free count; the code of an
erased chunk and every single-bit error in four bytes of a chunk put right, two refused, a code bit noticed; a save made
of a directory and eight files (the directory growing past its first cluster), read back at offsets, counted, listed
and deleted to the cluster; a save copied to another card and read back the same, a clash and a full card refused with
the target unchanged; damaged cards refused in words (the wrong length, PCSX2's erased card, a bad superblock, two
wrong bits in the FAT, a FAT loop through a copy) and 512 single-byte corruptions of the superblock, the FAT and the
root, each opened, listed and edited without a panic. Clippy clean on every target.

