# `emusen-disc` — Optical Disc Sector Reader

Live crate: `EmuSen/Cores/Shared/emusen-disc` (`emusen_disc`, v0.1.0, dependency: `chd` 0.3.4).

`emusen-disc` models compact discs (CD, ECMA-130) and digital versatile discs (DVD, ECMA-267) as addressable sectors over arbitrary byte backings (`B: AsRef<[u8]>`). It owns disc geometry, cue sheet and playlist parsing, sector synthesis, subchannel Q generation, `.sbi` patching, CHD v5 decoding and corruption guards, sector-to-file placement, and media fingerprinting. It deliberately omits drive controllers, seek/read timing, XA-ADPCM decoding, ISO 9660 filesystems, and file discovery (which belong to individual console cores and `Platform/galaxia`).

---

## 1. Module Layout & Public Surface

| Module | Responsibility |
|---|---|
| `src/disc.rs` | `Disc<B>`, `DiscError`, `IsoForm`, `Place`, `Sector`, `Toc`, `Track`, `samples()` |
| `src/dvd.rs` | `Dvd<B>`, `DvdSector`, `CD_LIMIT` (449,850 sectors), `is_frames()` |
| `src/medium.rs` | `Kind` (`Cd`, `Dvd`), `Medium<B>` enum unifying `Disc<B>` and `Dvd<B>` |
| `src/open.rs` | `open()`, `open_medium()`, `is_sheet()`, `Lone` (`Detect`, `User`), `Refusal` |
| `src/msf.rs` | `Msf`, `bcd()`, `from_bcd()`, `FRAMES` (75), `MINUTE` (4,500), `PROGRAM_START` (150) |
| `src/ecma130.rs` | ECMA-130 sector codec: `edc()`, `mode1()`, `mode2_form1()`, `mode2_form2()`, `parity()`, `parity_holds()`, `edc_holds()`, `form()`, `gf_mul()`, `alpha()` |
| `src/ecma267.rs` | ECMA-267 Data Frame codec: `frame()`, `id()`, `ied()`, `edc()`, `holds()`, `sector_information()`, `physical()`, `Zone`, `TrackPath` |
| `src/subq.rs` | `SubQ`, Q-channel synthesis, lead-in TOC items, CRC-16/CCITT-FALSE (`ccitt`) |
| `src/cue.rs` | Full CDRWIN cue sheet parser (`Sheet::parse`, `FileKind::Binary` / `Wave`) |
| `src/wave.rs` | RIFF WAVE parser (PCM format 1, stereo, 44,100 Hz, 16-bit LE) |
| `src/sbi.rs` | Redump `.sbi` subchannel patch parser (`sbi::parse`) |
| `src/set.rs` | Companion file lists (`companions`), `.m3u` playlists (`playlist`), `.sbi` detection (`sbi_beside`), per-track file byte spans (`track_spans`) |
| `src/chd.rs` | CHD v5 reader for CD (`CHT2`) and DVD (`DVD `) images, hunk cache, and pre/post-decode corruption checks |

---

## 2. Backings, Opening, Placement & Fingerprinting

### 2.1 Generic Backings (`B: AsRef<[u8]>`)
Both `Disc<B>` and `Dvd<B>` default to `B = Vec<u8>` and accept borrowed slices (`&[u8]`) or host-lent memory-mapped backings (such as `emusen_native::core::Kept`) without copying image data. Opening a disc reads only metadata:
- Cue sheets read no track bytes on open; placement uses timestamps and file sizes.
- Lone images inspect sector 0 (sync/header) and sector 16 (`CD001` primary volume descriptor), plus sector `V + 16` for dual-layer DVD detection.
- CHDs read only header, map, and metadata tags.

### 2.2 Medium & Container Detection (`open::open`, `open::open_medium`)
- **Cue sheet:** Text whose first command is `FILE`, `REM`, `CATALOG`, `TITLE`, or `PERFORMER` (`open::is_sheet`). Track files come from the leading companion backings in sheet order; missing files return `Refusal::Missing { needs, has }`.
- **CHD:** Identified by `MComprHD` magic; unit size and metadata distinguish CD (`2,448` bytes, `CHT2`) from DVD (`2,048` bytes, `DVD `).
- **Lone raw CD track:** Leading 12-byte sync pattern identifies a 2,352-byte data track (mode 1 or mode 2 from the first header); a multiple of 2,352 bytes without sync is treated as an audio track (trailing partial sector ignored).
- **DVD Data Frames:** Leading 2,064-byte frame whose ID addresses `0x030000` with valid IED (`dvd::is_frames`).
- **ISO / user-sector image (2,048 bytes/sector):** Identified by `CD001` at sector 16 (`Lone::Detect`) or forced via `Lone::User` when console-specific headers identify headerless user sectors. `open_medium` chooses CD vs DVD from the caller's `Kind` hint, except any image exceeding `CD_LIMIT` (449,850 sectors — `99:59:74` minus the 150-sector lead-in pregap) is always treated as DVD.
- **Companion `.sbi`:** Any trailing companion matching `SBI\0` magic is applied automatically to a CD.
- **`IsoForm`:** Callers specify whether 2,048-byte CD user sectors synthesize raw frames as `Mode1` (e.g., Sega CD) or `Mode2Form1` with data subheader `00 00 08 00` (e.g., PlayStation CD-XA).

### 2.3 File Placement & Track Spans
- `Disc::place(sector) -> Option<Place>` and `Dvd::place(lba) -> Option<Place>` return `{ file: Option<usize>, offset: u64 }` for host read-ahead (`file` is the 0-based sheet file index, or `None` for single-file images). Unstored/synthesised sectors return `None`. For CHD v5, `offset` is the compressed or uncompressed hunk's `block_offset` (or `None` for self/parent-copy hunks).
- `set::track_spans(sheet)` computes the exact byte range of each track within its backing file (starting at its first `INDEX` — `00` if present — up to the next track's first `INDEX` or EOF; single-track `WAVE` files span the full file including header).

### 2.4 Media Fingerprinting
`Disc::fingerprint(sectors)` and `Dvd::fingerprint(sectors)` compute a non-zero 64-bit FNV-1a digest over the disc's track layout/lead-out (or DVD sector/layer layout) and the caller-selected sectors relative to track 1 index 01 (or LBA 0). Cost is strictly bounded by the requested sector list.

---

## 3. Compact Disc (ECMA-130) Specifications

### 3.1 Track Placement & Synthesised Sectors
- Track 1 `INDEX 01` is fixed at `00:02:00` (sector `150`, `PROGRAM_START`), with track 1's pause starting at sector `0`.
- `PREGAP` inserts unstored silence/empty sectors before a track and shifts all subsequent disc addresses; `POSTGAP` inserts unstored sectors after a track.
- Multiple tracks within one file advance offsets by each track's own sector size (`2,352`, `2,336`, or `2,048`), supporting mixed-mode single-BIN images.
- `FILE "..." WAVE` tracks accept uncompressed RIFF WAVE (PCM format 1, 2 channels, 44,100 Hz, 16-bit little-endian) and skip non-`data` chunks (such as `LIST`) before reading samples from the `data` chunk.
- Unstored pregaps, postgaps, and lead-out sectors are synthesised on demand: zeroed PCM for audio tracks, or properly framed empty sectors (sync, header, EDC, and RSPC P/Q parity) for data tracks. Lead-out inherits the final track's mode.

### 3.2 ECMA-130 Sector Codec (`src/ecma130.rs`)
- **EDC:** 32-bit CRC with polynomial $(x^{16}+x^{15}+x^2+1)(x^{16}+x^2+x+1)$ (reflected `0xD8018001`), LSB-first, stored little-endian.
  - Mode 1 covers bytes `0..2064`.
  - Mode 2 Form 1 covers subheader + user data, bytes `16..2072`.
  - Mode 2 Form 2 covers bytes `16..2348`.
- **RSPC (Reed-Solomon Product Code, Annex A):** Computed over bytes `12..2352` as 1,170 16-bit words (MSB and LSB lanes encoded independently) in $\mathrm{GF}(2^8)$ with polynomial $x^8 + x^4 + x^3 + x^2 + 1$.
  - P-vectors: $43 M_p + N_p$ ($M_p \in 0..26$).
  - Q-vectors: $(44 M_q + 43 N_q) \bmod 1118$ ($M_q \in 0..43$) plus parity words $1118 + N_q$ and $1144 + N_q$.
  - Mode 2 Form 1 parity is computed with the 4-byte header (`12..16`) treated as zero.

### 3.3 Subchannel Q & `.sbi` Patches (`src/subq.rs`, `src/sbi.rs`)
- **Q-channel (12 bytes):** Control nibble (`0` audio, `4` data, plus cue `FLAGS` bits `PRE`, `DCP`, `4CH`) over ADR/Q-mode `1`, BCD track/index/relative MSF/absolute MSF, and big-endian inverted CRC-16 over $G(x) = x^{16} + x^{12} + x^5 + 1$.
- **Pre-gap countdown:** Relative time inside an `INDEX 00` pause counts down to `00:00:00` on its final sector (`start - at - 1`).
- **Lead-out:** Track `0xAA`, index `0x01`, relative time counting up from lead-out start.
- **Lead-in TOC items (`lead_in`):** Per-track `INDEX 01` pointers plus `0xA0` (first track, disc type `0x20` when any data track is CD-XA), `0xA1` (last track), and `0xA2` (lead-out).
- **`.sbi` patches:** Header `SBI\0` followed by BCD time, format (`1`: 10 Q bytes, `2`: relative time Q3–Q5, `3`: absolute time Q7–Q9), and patch payload. Patched sectors overlay the recorded bytes onto synthesised Q and XOR the computed CRC with `0x0080` so the CRC intentionally fails, matching protected LibCrypt sectors.

### 3.4 CD-DA Audio
`samples(&raw)` yields 588 16-bit little-endian stereo frames per 2,352-byte audio sector (left channel first). CHD CD audio tracks are stored big-endian and byte-swapped on read so all backings return identical little-endian PCM.

---

## 4. Digital Versatile Disc (ECMA-267) Specifications

### 4.1 Data Frame Layout (`src/ecma267.rs`)
Each 2,064-byte ECMA-267 Data Frame consists of:
- **ID (4 bytes):** 1-byte Sector Information (`b24` layer, `b25` data type `0` = read-only, `b27..26` zone `00` Data / `01` Lead-in / `10` Lead-out / `11` Middle, `b29` reflectivity = `1` on dual-layer discs, `0` on single-layer) + 24-bit big-endian Physical Sector Number (PSN).
- **IED (2 bytes):** $I(x)\cdot x^2 \bmod (x + \alpha^0)(x + \alpha^1)$ over the same $\mathrm{GF}(2^8)$ field as ECMA-130.
- **CPR_MAI (6 bytes):** Preserved from stored 2,064-byte frame images; zeroed when synthesising from 2,048-byte user sectors.
- **Main Data (2,048 bytes):** Logical sector payload.
- **EDC (4 bytes):** MSB-first CRC-32 over the preceding 2,060 bytes with $G(x) = x^{32} + x^{31} + x^4 + 1$, stored big-endian.

Scrambling, ECC blocks, and EFMPlus modulation lie below the drive interface and are not modelled.

### 4.2 Dual-Layer Detection & Physical Sector Numbering
Because raw `.iso` / CHD images omit the physical lead-in, dual-layer breaks are detected from volume descriptors:
- If the ISO 9660 Primary Volume Descriptor at LBA 16 reports a Volume Space Size $V$ strictly smaller than total image sectors, $V$ is a multiple of 16, and LBA $V + 16$ contains a second valid Primary Volume Descriptor, Layer 1 begins at LBA $V$.
- Data Zone PSNs start at `0x030000` on Layer 0. Layer 1 defaults to **Opposite Track Path (OTP)**, starting at the 24-bit bitwise inverse of Layer 0's final PSN; callers may select **Parallel Track Path (PTP)** via `Dvd::set_track_path` (Layer 1 restarts at `0x030000`). Reads past the final sector synthesise Lead-out Zone frames with zeroed Main Data.

---

## 5. CHD v5 Format & Corruption Guards (`src/chd.rs`)

- **CD CHD:** 2,448-byte units (2,352 sector + 96 subcode), default 8-unit hunks (19,584 bytes), `CHT2` metadata tags (`MODE1`, `MODE1_RAW`, `MODE2_RAW`, `AUDIO`, plus `MODE2`, `MODE2_FORM1`, `MODE2_FORM_MIX`), tracks padded to 4-unit multiples, `V*` `PGTYPE` indicates stored pregap counted in `FRAMES`.
- **DVD CHD:** 2,048-byte units, default 2-unit hunks (4,096 bytes), `DVD ` metadata tag, generic codecs (`lzma`, `zlib`, `huff`, `flac`).
- **Pre/post-decode guards:** Because cores build with `panic = "abort"`, `emusen-disc` validates every compressed CHD v5 hunk before and after decoding:
  1. Follows `CompressionSelf` chains (bounded to 4 hops, no self-loops or out-of-range targets).
  2. Verifies `block_offset + block_size <= file.len()`.
  3. For framed CD codecs (`cdlz`, `cdzl`, `cdzs`), validates the leading frame-flag bytes (`ceil(units / 8)`) and big-endian compressed stream length (2 bytes for hunks `< 65,536` bytes, 3 bytes otherwise) against the block size before invoking the `chd` crate.
  4. Verifies the decoded hunk against `hunk_crc` using table-driven CRC-16/IBM-3740 (`x^16 + x^12 + x^5 + 1`, init `0xFFFF`, unreflected, uninverted).
- **Failure handling:** A failed hunk never panics during `read`; `read` returns synthesised empty/silent sectors and records the first diagnostic string in `Disc::damage()` / `Dvd::damage()`.
- **Known limits:**
  - Uncompressed CHDs (`-c none`) carry no per-hunk CRC; silent byte flips inside uncompressed hunks are undetected at read time.
  - Severe corruption inside a compressed v5 map's Huffman header (final bytes of the file) can still trigger a panic inside the upstream `chd` 0.3.4 `Chd::open` map decoder; archive verification tools run `chdman verify` ahead of open.
  - Multi-session CDs, Q-modes 2/3 (MCN/ISRC), `.ccd/.img/.sub`, `.ecm`, `.pbp`, and `.gdi` are not implemented.

---

## 6. Third-Party Attribution (CHD / `chdman`)

CHD (Compressed Hunks of Data) is MAME's disc image format, and `chdman` is MAME's tool for creating, verifying, and extracting CHD images. `chdman` is licensed under the BSD-3-Clause license (copyright Aaron Giles); MAME as a whole is distributed under the GNU General Public License, version 2 or later (<https://docs.mamedev.org/license.html>; `chdman` manual: <https://docs.mamedev.org/tools/chdman.html>). EmuSen reads CHD images via the `chd` Rust crate (BSD-3-Clause, Ronny Chan; see `THIRD_PARTY_NOTICES.md`) and ships unmodified `chdman` 0.289 (`mame0289`) where packaged.
