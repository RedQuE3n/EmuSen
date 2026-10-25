# EmuSen_Disc — a disc as sectors

*Started 2026-10-11, at Diamond's stage 7.* `emusen-disc`, `EmuSen/Cores/Shared/emusen-disc/`: a compact disc as the
sectors a drive reads from it, built from a cue sheet and its track files, an `.iso`, a lone raw track or a CHD.
Planned in `Diamond_Plan.md` §9.2 and decided with its §11 Q9 on 2026-10-11, with the set module of
`EmuSen_CoreAPI_Extensions.md` §2 (decided the same day). Its first user is Diamond; Nephrite's paused stage 8 (the Sega
CD, `Nephrite_Plan.md` §5.8) is its second, and its interface was drawn for both.

The marks are `Diamond_Plan.md`'s: **[M]** measured on 2026-10-11 with the tool named; **[R]** read from a source;
**[A]** argued; **[C]** a choice.

## 1. Its one job, and what it leaves to its users

**In it** [R the plan's §9.2]: the geometry (minutes, seconds and frames, the absolute sector, BCD); raw 2,352-byte
sectors, read from the image or synthesised with their sync, header, EDC and parity (ECMA-130, 2nd edition, 1996); the
CD-XA modes 2 form 1 and form 2; subchannel Q, synthesised for every sector or patched from an `.sbi`; the TOC and the
lead-in's items; CD-DA as samples; the cue sheet parsed in full; the set module's two readers (a sheet's companion
files, an `.m3u`'s entries); CHD v5.

*Since 2026-10-12 (§9):* the kinds of backing a disc's bytes may have, owned, borrowed for a call or lent by the host
for the machine's life; opening a disc from an image and its companions; and the fingerprint a state is held to.

**Not in it**: where a disc's files are, or keeping them available, which are Galaxia's (`EmuSen_Galaxia.md` §3.8); any
console's drive controller, its seek or read timing, XA-ADPCM decoding, or any filesystem. Diamond's
controller is `src/cdrom/` (`Diamond_CDROM.md`); a Sega CD's CDC and CDD will be Nephrite's. The development tools that
look inside an ISO 9660 volume (Diamond's `run --boot`) read the directory records themselves.

**Dependencies**: the `chd` crate 0.3.4 (BSD-3-Clause), used as a dependency whose code is not read, as Q9 decided
[R]. Nothing else: not `emusen-native`, and never Galaxia, since a core builds and runs with no frontend.

## 2. The surface

| Item | What it is |
|---|---|
| `Msf`, `msf::{bcd, from_bcd, FRAMES, MINUTE, PROGRAM_START}` | a time and the absolute sector it names, counted from 00:00:00; 00:02:00 (sector 150) is where the first track's index 01 lies |
| `Disc::from_sheet(&Sheet, files, IsoForm)` | a disc from a parsed cue sheet and its files' bytes, in the sheet's order |
| `Disc::from_image(bytes, IsoForm)` | a disc from one file: a CHD by its `MComprHD` magic, a lone raw track by its sync pattern (data) or its length (audio), an `.iso` by its volume descriptor's `CD001` |
| `Disc::from_user_image(bytes, IsoForm)` | 2,048-byte user sectors as one data track, for a caller that knows them to be one without a volume descriptor |
| `Disc::apply_sbi(bytes)` | the `.sbi` beside the image (§3.5) |
| `Disc::read(at, &mut [u8; 2352]) -> Sector` | sector `at`'s raw bytes and whether it is data (its mode) or audio, stored or synthesised |
| `Disc::subq(at) -> SubQ` | the sector's subchannel Q, twelve bytes with the CRC |
| `Disc::toc() -> Toc`, `Disc::lead_in(pointer)`, `Disc::tracks()`, `Disc::track_at(at)`, `Disc::lead_out()` | the table of contents and the tracks on the time line |
| `samples(&raw)` | an audio sector's 588 stereo frames, little-endian, left first |
| `ecma130::{edc, mode1, mode2_form1, mode2_form2, parity, parity_holds, edc_holds, form}` | the sector codec |
| `cue::Sheet::parse`, `set::{companions, playlist, sbi_beside}`, `sbi::parse` | the readers |
| `Disc<B>`, for `B: AsRef<[u8]>` (§9.1) | the bytes behind a disc: a `Vec<u8>` (the default), a `&[u8]`, or a type that lends the host's own |
| `open::open(image, companions, IsoForm, Lone)`, `open::is_sheet`, `open::Refusal` (§9.2) | a disc from an image and the files beside it, as a host passes them |
| `Disc::fingerprint(sectors)` (§9.3) | FNV-1a of the tracks, the lead-out and the sectors the console names |

**`IsoForm`** is the console's choice of how a 2,048-byte sector is framed when it is read raw: mode 1, or mode 2 form 1
with a data subheader (`00 00 08 00`), which is what a CD-XA console's discs carry. Diamond passes `Mode2Form1`; a Sega
CD would pass `Mode1`. It is a parameter rather than a fact of the crate because the same `.iso` is either, by console
[A].

**One concrete type, not a trait object** [C]: the plan's §9.2 says "its interface is a trait: read sector n with its
subchannel". Both users need exactly `read`, `subq` and the TOC from the same type, and neither needs a second
implementation behind them; a trait was left out until one does. `read` takes `&mut self` because a CHD decompresses a
hunk into a cache.

## 3. The rules

### 3.1 Placing a cue sheet's tracks

psx-spx's ".CUE/.BIN (CDRWIN)" chapter [R]: each FILE's INDEX times are positions in that file; track 1's INDEX 01 lies at
00:02:00 (the two-second pause every disc opens with, stored or not), so the first file's frame 0 is at 150 less its
INDEX 01; a PREGAP is silence the file does not hold, inserted before the track and moving every later address; a
POSTGAP likewise after it; a new FILE begins where the previous track ended. A track's first sector (its `pregap`) is
its pause's start, its `start` is its INDEX 01, its `end` the next track's first sector; the last track's end is the
lead-out. Track 1's pause always starts at sector 0. A file is read at the byte its track's first INDEX names, each
earlier track in the same file taking its own sector size, so that a file mixing 2,048- and 2,352-byte tracks is
placed correctly.

### 3.2 Sectors the image does not hold

A pause or lead-out the image does not store is synthesised [C]: silence on an audio track, an empty sector of the
track's mode on a data one (mode 2 form 1 with a zero subheader, or mode 1), with a correct header, EDC and parity. The
lead-out continues the last track's kind. Track 1's first two seconds of a lone `.bin` or an `.iso` are the same
synthesised sectors. The getloc log of ps1-tests reads mode 2 headers in that pause on a console [R], which the choice
matches.

### 3.3 ECMA-130's sector codec

- **The EDC** (§14.3) is the CRC whose check polynomial is (x¹⁶ + x¹⁵ + x² + 1)(x¹⁶ + x² + x + 1), the least significant
  bit first: `D8018001h` reflected, the remainder stored little-endian so that x⁰ falls in byte 2,067's top bit. A test
  multiplies the two factors out and reflects the product, and another divides a message bit by bit by P(x) and
  compares the table-driven result [M, `cargo test`]. Mode 1 covers bytes 0–2,063; mode 2 form 1 the subheader and data,
  16–2,071; form 2 16–2,347 (psx-spx "CDROM Sector Encoding").
- **The RSPC** (Annex A): bytes 12 onward as 1,170 words, S(n) = MSB B(2n+13), LSB B(2n+12), the code applied to the MSBs
  and the LSBs apart; GF(2⁸) by x⁸ + x⁴ + x³ + x² + 1; P-vector Nₚ the words 43Mₚ + Nₚ, Mₚ = 0…25; Q-vector N_q the words
  (44M_q + 43N_q) mod 1118 for M_q = 0…42, then 1118 + N_q and 1144 + N_q. Each vector's two parity bytes are solved from
  H = [1 … 1; αⁿ⁻¹ … α 1]: p₀ = (S₀ + S₁)/(α + 1), p₁ = S₀ + p₀. `parity_holds` checks both rows of H on every vector, apart
  from the solution, so it catches an encoder that does not meet its definition.
- **Mode 2 form 1's parity** is computed with the header taken as zero (psx-spx's `encode_sector`), and a test shows that
  the check holds with it zeroed and fails without [M].
- **Known vectors** are derived from the standard, not from any other implementation [C]: the polynomial's product, a
  bitwise division, α⁸ = x⁴ + x³ + x² + 1, α generating all 255 non-zero elements, and the two checks on encoded sectors.
  The WiseMan builder (`Diamond_Native.md` §5.2) implements the same definitions in C#; the two were not compared
  sector for sector, which is left for a step that has a reason to (§7).

### 3.4 Subchannel Q

ECMA-130 §22.3 [R]: the control nibble (0 audio, 4 digital data; a cue sheet's FLAGS adds PRE, DCP and 4CH) over q-mode
1; in the user data area the track, the index, the time within the track and the absolute time, BCD. **In a pause the
relative time counts down to zero in its last sector**, which the standard states in two sentences that read
inconsistently ("set to the duration of the pause … equals zero in the last section"); the console's getloc log settles
it: absolute 00:01:68 reads relative 00:00:06, 00:01:67 reads 00:00:07, and 00:00:33 reads 00:01:41, all of them
`start − at − 1` [R the log; M, the crate's test holds the first of them]. In the lead-out the track is AAh, the index 01 and
the relative time counts up from the lead-out's start. **The CRC** (§22.3.6) is G(x) = x¹⁶ + x¹² + x⁵ + 1, most
significant bit first from zero, stored inverted and big-endian; a test divides the ten bytes and the uninverted CRC by
G(x) and finds no remainder.

The lead-in's items (§22.3.4) are made for GetQ: a track's pointer with its INDEX 01, A0h with the first track and the
disc type (20h when any data track is CD-XA), A1h with the last, A2h with the lead-out.

### 3.5 `.sbi`

psx-spx's "SBI (redump.org)" [R]: `SBI\0`, then entries of a BCD time, a format (1: ten Q bytes; 2: the relative time,
Q3–Q5; 3: the absolute time, Q7–Q9) and its bytes. The format cannot record the bad CRC that LibCrypt relies on, so every
listed sector's Q is the synthesised one with the recorded bytes laid over it and its CRC computed then XORed with
0080h, which fails [C, psx-spx: "assume that all sectors in the SBI file have wrong checksums"]. A drive that keeps the
last valid Q (Diamond's GetlocP) then answers as a console does on the protected sectors.

### 3.6 The CD-DA samples

Sixteen-bit little-endian stereo, left first, 588 frames a sector (ECMA-130's audio sector; psx-spx "Audio"). A CHD
stores them big-endian (§5), and `read` swaps them so that every source gives the same bytes.

## 4. One file, no sheet

**A lone raw track** (two-thirds of the tester's library, §8 of `Diamond_CDROM.md`): the bytes begin with the sync
pattern, so they are a data track of 2,352-byte sectors whose mode is the first header's (mode 1 or mode 2); bytes of a
multiple of 2,352 with no sync are an audio track. A partial last sector is left out. **An `.iso`**: a volume
descriptor's `CD001` at sector 16 makes 2,048-byte user sectors, a `MODE1/2048` track. Either way the crate parses the
one-line sheet it would have been given (`FILE "image" BINARY / TRACK 01 <mode> / INDEX 01 00:00:00`) and places it by
§3.1, so there is one placement rule for every source [C]. **Why here and not in the system pack** [A]: what a lone
track's bytes mean is a container format's fact (the sync pattern, the header's mode byte), the crate's by the plan's
§9.2; the pack is C#, and the kit and the probe, which open images without any C#, would otherwise need a second reader.

## 5. CHD v5

The CD layout a CHD holds is not in any document read here, and the `chd` crate's source and MAME's are not read. It
was measured from chdman 0.289's output (Fedora's `mame-tools`, unpacked user-local into the cache and run as a tool)
on synthetic sheets made for the purpose [M, 2026-10-11]:

- the header's unit is 2,448 bytes (2,352 and 96 of subcode) and its hunk a multiple of it (19,584 bytes, eight units);
- one `CHT2` metadata entry per track: `TRACK:n TYPE:t SUBTYPE:s FRAMES:f PREGAP:p PGTYPE:g PGSUB:s POSTGAP:q`, the
  types `MODE1`, `MODE1_RAW`, `MODE2_RAW`, `AUDIO` seen (`MODE2`, `MODE2_FORM1` and `MODE2_FORM_MIX` accepted by name and
  not measured);
- each track's units padded to a multiple of four (37 frames took 40), tracks following one another;
- a pregap stored in the image has a `PGTYPE` beginning with `V` and is counted in `FRAMES`; one not stored (a PREGAP
  line) has a plain `PGTYPE`, its frames not counted;
- audio samples stored big-endian; a cooked mode 1 track's 2,048 bytes at the start of its unit.

The crate's CHD path reads the metadata through the `chd` crate's `metadata_refs`, places the tracks by those rules,
and decompresses one hunk at a time into a cache. A test builds three sheets (two files with a stored pause, an
unstored PREGAP, an `.iso`), has chdman convert each, and requires every sector, its kind, its Q and the TOC to equal
the sheet's, through the lead-out [M]. It is opt-in, by `EMUSEN_DISC_CHDMAN` naming the chdman binary, because chdman
is not part of the build. The tester's library holds no CHD [M, a listing], so no real CHD was read.

## 6. Tests (2026-10-11)

`cargo test --release`: 11 unit tests (the codec's derivations and checks, MSF and BCD, the cue parser's three cases
and its refusals, the set readers, `.sbi`, the Q CRC) and 8 disc tests (a two-file dump read sector for sector, Q's
countdown and the lead-out, the TOC and lead-in, an unstored PREGAP, an `.iso` framed either way and meeting both
checks, a lone raw track, an `.sbi`'s failing CRCs, CHDs from chdman equal to their sheets) [M]. 19 of 19 with
`EMUSEN_DISC_CHDMAN`; 19 with the CHD test skipping without it. `rust-cores.yml` tests every crate in
`EmuSen/Cores/Shared/`, this one included.

## 7. What waits

- `.ccd`/`.img`/`.sub` (a full subchannel), `.ecm`, `.pbp` and `.gdi` (Kunzite): later, by the plan's §1.2.
- Q-mode 2 and 3 (the catalogue and ISRC), and the lead-in's TOC repeated in Q for a drive that reads it: no user needs
  them yet.
- The crate graded sector for sector against WiseMan's builder: both implement ECMA-130's definitions and each meets the
  checks; a direct comparison would show that the C# and the Rust read the definitions alike.
- Multi-session discs: one session only.

## 8. Reading log

Read for the crate: ECMA-130 (2nd edition, June 1996) §14 and Annex A, §22.1–§22.3; psx-spx at `6d7d1bc`: "CDROM Disk
Format", "CDROM Subchannels", "CDROM Sector Encoding" (its `encode_sector`'s header rule), "CDROM XA Subheader",
"CDROM File Formats" (".CUE/.BIN", "Cue Sheet", "SBI", ".chd"); the `chd` crate's generated API documentation (its
items and their doc comments, not its source); chdman's own `info` output. Not read: the `chd` crate's source, libchdr,
MAME, or any emulator's disc code.

**CHD and chdman, credited (2026-10-25).** CHD (Compressed Hunks of Data) is MAME's image format, and chdman is MAME's
tool for making, checking and extracting it. chdman's source file is under the BSD-3-Clause licence, copyright Aaron
Giles; MAME as a whole is distributed under the GNU General Public License, version 2 or later ("The MAME License",
MAME Documentation 0.289, <https://docs.mamedev.org/license.html>). chdman's commands, codecs and defaults are
documented in "chdman – CHD (Compressed Hunks of Data) File Manager", MAME Documentation 0.289,
<https://docs.mamedev.org/tools/chdman.html>, read 2026-10-25; every chdman run the project records was of release
0.289. EmuSen reads CHD through the `chd` crate (BSD-3-Clause, Ronny Chan; `THIRD_PARTY_NOTICES.md`), whose source is
not read, and MAME's source is never read.

## 9. Backings, and opening an image with its companions (2026-10-12)

*Decided 2026-10-12, with the core ABI's minor 1.3 (`EmuSen_CoreAPI_Extensions.md` §10.2.3): this crate owns what a disc
is and how its bytes are read, the kinds of backing among them, and takes from Diamond's `medium.rs` what was never the
PlayStation's. Written before the code as its design; the record of what was built and tested follows it.*

### 9.1 The backing

Until 1.3 a disc owned its bytes: `Disc::from_sheet` took `Vec<Vec<u8>>`, and a core copied every track at create. The
copy was the launch's cost (`Diamond_Players.md` §12.8), so the bytes may now stay where the host has them.

- **`Disc<B>` for any `B: AsRef<[u8]>`**, with `Vec<u8>` the default, so that `Disc` written alone means what it did
  and every earlier caller compiles unchanged. A CHD is `chd::Chd<Cursor<B>>`, which needs the same bound and nothing
  more.
- **Three backings are in use, and the crate knows none of them by name** [C]: `Vec<u8>`, owned; `&[u8]`, borrowed for
  one call, for a probe or a firmware question that opens a disc to read a few sectors and lets it go; and the core
  kit's `Kept` (`emusen_native::core`), which is the host's own mapped bytes where the host lends them and a copy
  otherwise. The kit's type reaches the crate through `AsRef<[u8]>` alone, so the crate depends on neither the kit nor
  any host.
- **Why a type parameter and not a boxed trait object** [A]: a `Box<dyn AsRef<[u8]>>` would need `'static`, which a
  slice borrowed for a call is not, and would put a second pointer behind every sector read; the parameter costs
  nothing at run time, and a core names its disc type once (`type Disc = emusen_disc::Disc<Kept>`).
- **One path reads every backing.** `read` takes the track's stored range out of `backing.as_ref()`, whatever the
  backing, so there is no borrowed reader to diverge from the owned one. The tests of §9.4 hold the equality anyway,
  format by format, because the claim is the exactness rule's: a game reads the same bytes from a mapping as from a
  copy.
- **What a borrowed backing asks of its owner**: that the bytes do not change and do not move while the disc lives.
  The crate cannot check either; `Kept`'s loan is the kit's to keep (`EmuSen_CoreAPI_Extensions.md` §10.2.4, item 2),
  and a `&[u8]` is the compiler's.
- **Reads touch only what is read.** Opening a sheet's disc reads no track byte: the tracks are placed from the
  sheet's times and the files' lengths. Opening a lone image reads its first sector and sector 16's five bytes;
  opening a CHD reads its header, map and metadata. Nothing walks an image.

### 9.2 Opening

`open::open(image, companions, iso, lone)` is what Diamond's `medium::open` did before it asked for the licence:

1. bytes that read as a cue sheet (`open::is_sheet`: text whose first command is `FILE`, `REM`, `CATALOG`, `TITLE` or
   `PERFORMER`) are parsed, and the sheet's files are the first companions, in its order; fewer than it names is
   `Refusal::Missing { needs, has }`;
2. anything else is one file: a CHD, a lone raw track or an `.iso` by §4's rules (`Lone::Detect`), or 2,048-byte user
   sectors with no volume descriptor where the console's own signature has shown them to be that (`Lone::User`);
3. an `.sbi` among the companions past the sheet's own, told by its magic, is applied (§3.5).

The image itself is taken by reference, since a sheet is parsed and not kept and a lone image is kept through the
caller's backing of the same bytes; the companions are taken as backings. **What stays with the console** is its
signature and its rules: Diamond reads the licence in sector 4 for the market and refuses a disc without it; a Sega CD
core will read `SEGADISCSYSTEM`. `Lone` is a parameter for the same reason `IsoForm` is one: whether headerless
2,048-byte sectors are a disc is the console's knowledge, not the container's.

### 9.3 The fingerprint

A core's state names the disc it was made on, so that a load over another disc is refused (`EMUSEN_WRONG_MEDIA`,
`EmuSen_CoreAPI.md` §6.23). `Disc::fingerprint(sectors)` is the arithmetic every disc core would otherwise repeat:
FNV-1a (64-bit) over each track's index 01 and kind, the lead-out, and the raw 2,352 bytes of each sector the caller
names, relative to the first track's index 01; never zero, zero meaning an empty drive. **It is bounded by
construction**: it reads the sectors named and no other, so its cost does not grow with the disc. Diamond names
sector 4 and sectors 16 to 23, as before 1.3, and the result is bit for bit what `medium::fingerprint` gave, so every
state made earlier loads; a test pins the value on a synthetic disc.

### 9.4 Tests (2026-10-12)

`cargo test --release`, 12 unit and 11 disc tests, all passing, with chdman named and without [M]. The three added:

- **`every_form_reads_the_same_from_borrowed_bytes_as_from_owned`**: each form opened over `Vec<u8>`, over `&[u8]` and
  over `Arc<[u8]>`, the last standing for the kit's lent bytes, and compared track for track, TOC for TOC, lead-in item
  for lead-in item, and sector for sector in kind, bytes and Q from sector 0 through ten sectors of the lead-out, the
  fingerprint too: a sheet of two files; one of three with FLAGS, PREGAP, a second index and a POSTGAP; one file with an
  unstored PREGAP; user sectors with audio, framed both ways; 2,336-byte sectors; a lone raw track, a lone audio track,
  an `.iso` and headerless user sectors, through `from_image`, `from_user_image` and `open`; and a sheet with its `.sbi`.
  `a_chd_reads_as_its_sheet` compares each CHD over borrowed bytes and through `open` with the owned CHD likewise.
- **`an_image_and_its_companions_open_or_are_refused_with_words`**: a sheet's files taken from the first companions, an
  `.sbi` no stand-in for a track, a file too short for its indexes, a sheet that does not parse, bytes that are no
  image, the console's word that headerless sectors are a disc, and a torn `.sbi`.
- **`the_fingerprint_is_the_tracks_the_lead_out_and_the_sectors_named`**: the value computed by hand as `medium.rs` did
  before 1.3 and pinned; every byte outside the named sectors changed, and the audio track silenced, without moving
  it; one byte inside, the lead-out or the sectors named moving it; never 0.

The cores' side was not changed by a test: Diamond's 157 unit and 76 program tests pass on the kept disc, with and
without ps1-tests, and every state made before 1.3 loads, its fingerprint being bit for bit the same.

## 10. Which file and byte hold a sector (2026-10-12)

*Decided 2026-10-12 with the core ABI's minor 1.4 (`EmuSen_CoreAPI_Extensions.md` §10.2.6): a disc core names where
its drive is going by a file and a byte, and the mapping of sector to file and byte is this crate's, since it alone
knows how a sheet's tracks lie in their files.*

### 10.1 The question

`Disc::place(sector) -> Option<Place>`, `Place { file, offset }`:

- **`file`** is the sector's file by its position among those a sheet names, the order a host numbers them from
  `EMUSEN_FILE_MEDIA`, or `None` where the disc was opened from one file that is the image itself (a lone raw track, an
  `.iso`, user sectors, a CHD). The crate records which by a flag set in `from_image` and `from_user_image`.
- **`offset`** is the byte where `read` begins to take the sector's stored bytes: `store.offset + n ×` the mode's
  sector size, so 2,352 for a raw sector, 2,336 for a mode 2 sheet's and 2,048 for user sectors.
- **None** for a sector the image does not store: a pause a sheet does not hold (`PREGAP`), a `POSTGAP`, the lead-out,
  and any place before the first track. It reads nothing of the image, so a core may ask it on every sector.

### 10.2 A CHD

A CHD stores hunks of eight 2,448-byte units, each compressed by one of four codecs, stored as it is, or a copy of
another hunk. There is no byte of the file where a sector's 2,352 bytes lie, but there is one the system must read
before the sector can be decoded: **the hunk's own first byte**, which the `chd` crate's map gives
(`Map::get_entry(hunk)`, `V5CompressedMapEntry::block_offset` for a hunk compressed or stored, and
`V5UncompressedMapEntry::block_offset` for a map of uncompressed hunks). `place` answers that, so that a host reading a
megabyte on from it reads the compressed bytes of the hunks that follow, which a compressed disc spends more slowly
than a raw one. A hunk that copies another (`CompressionSelf`, `CompressionParent`) names nothing, since its bytes are
elsewhere, and so does a map older than v5, whose entries were not measured here. The crate's source was not read for
this: the map's types are its documented API (`cargo doc`).

### 10.3 Tests (2026-10-12)

`a_sector_is_placed_in_the_file_and_at_the_byte_that_hold_it`, on every form the crate reads from files (a sheet of two
files, of three with FLAGS, PREGAP, a second index and a POSTGAP, one file with an unstored PREGAP, user sectors with
audio, 2,336-byte sectors, a lone raw track, a lone audio track, an `.iso` and headerless user sectors): every sector
from 0 through ten of the lead-out is either read from the image and placed, or synthesised and not; a placed sector's
read changes when the byte named is changed and not when the byte before it is; the numbers pinned on the sheet of two
files; and a place the same over borrowed bytes. `a_chd_reads_as_its_sheet`, with chdman named, shows each CHD's
places to be hunks of its own: with sixteen bytes at a place changed every sector placed there reads otherwise (or the
decoder stops on the hunk: `chd` 0.3.4 indexes past a slice for some changed hunks, which the test catches and counts)
and every sector placed elsewhere reads as before. `cargo test --release`: 12 and 12, with chdman named and without.


## 11. WAVE tracks (2026-10-14)

*Added at Nephrite's stage 8 (`Nephrite_Native.md` §51), its second user's first need of the crate.* A cue sheet's
`FILE "name.wav" WAVE` names a RIFF WAVE file whose data chunk holds an audio track's samples; the 240p Test Suite's
Sega CD build ships its two audio tracks so, and the crate refused it before.

- **What is read** [R: Microsoft's RIFF WAVE layout as the format is commonly documented]: `RIFF`, a length, `WAVE`,
  then chunks of a four-byte identifier, a little-endian length and a body padded to an even length. The `fmt ` chunk
  must give PCM (format 1), two channels, 44,100 Hz and 16 bits, which is CD-DA's own sample format; anything else is
  refused with the format's words, since a converted rate or width would not be the disc's samples. The `data`
  chunk's body is then the track's bytes, stored as a raw `BINARY` audio track stores them, little-endian, left first.
  Chunks before `data` (`LIST` and the like) are skipped.
- **Where it changes the crate**: `Sheet` records each file's kind (`FileKind::Binary` or `Wave`, the first the
  default, so that a sheet built as before means what it did); a `WAVE` file may hold only an `AUDIO` track. Placing a
  sheet's tracks takes a WAVE file's data chunk in place of the whole file, so a track's first stored byte is past the
  header, and `read` and `place` name bytes of the file itself: `place` answers the byte within the WAVE file, which is
  what a host reading ahead needs. Nothing else moves.
- **Tests** [M, `cargo test --release`, 13 and 13]: `a_wave_files_samples_are_found_and_others_refused` (a header, a
  `LIST` chunk of odd length before the data, a mono file, a truncated one), the cue parser's three cases, and
  `a_wave_files_track_reads_as_the_raw_track_it_holds`: the two-file dump of §6 with its audio as a WAVE file reads
  sector for sector, Q for Q and TOC for TOC as with the raw file, every audio sector placed 44 bytes further in.
  Diamond's crates, the crate's other user, pass unchanged (167 unit, 81 program and the rest).
