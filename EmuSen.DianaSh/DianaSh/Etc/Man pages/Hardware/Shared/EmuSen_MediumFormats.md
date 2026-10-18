# EmuSen_MediumFormats — the save media's file systems as crates without the standard library

*Built 2026-10-17, with the backend's step B3 (`EmuSen_Backend.md` §13).* Three crates in `EmuSen/Cores/Shared/`, one
per save medium whose file system EmuSen reads:

| Crate | Medium | Layout from |
|---|---|---|
| `emusen-psx-card` | the PlayStation memory card, `psx.card` | psx-spx, "Memory Card Data Format" |
| `emusen-n64-pak` | the N64 Controller Pak, `n64.pak` | N64brew, "Controller Pak/Filesystem" |
| `emusen-mcd-bram` | the Sega CD's backup RAM, internal (`mcd.backup`) and cartridge (`mcd.ramcart`) | `Nephrite_Bram.md` |

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
