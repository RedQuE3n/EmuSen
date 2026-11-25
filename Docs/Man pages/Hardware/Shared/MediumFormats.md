# Save Medium Formats (`emusen-psx-card`, `emusen-n64-pak`, `emusen-mcd-bram`, `emusen-ps2-card`)

Live crates in `EmuSen/Cores/Shared/`:
- `EmuSen/Cores/Shared/emusen-psx-card` (`emusen_psx_card`, v0.1.0)
- `EmuSen/Cores/Shared/emusen-n64-pak` (`emusen_n64_pak`, v0.1.0)
- `EmuSen/Cores/Shared/emusen-mcd-bram` (`emusen_mcd_bram`, v0.1.0)
- `EmuSen/Cores/Shared/emusen-ps2-card` (`emusen_ps2_card`, v0.1.0)

These four `#![no_std]` crates define the binary filesystem layouts of console removable/console save media. Because they allocate nothing and depend only on `core`, the same format logic serves both host save-media tools (`Platform/systems`, `memcard`) and open console firmware verification (round-trip tested against Diamond's open PS1 kernel and Nephrite's open Sega CD BIOS).

---

## 1. Shared Design Contract

- **`#![no_std]`, zero allocation:** Every function reads a byte slice or edits a caller-supplied mutable image buffer in place.
- **Transactional refusal:** Validation (format checks, chain integrity, capacity, name collisions) runs before mutation; any error leaves the destination image untouched.
- **Byte-oriented identifiers:** Filenames and titles are exposed as raw stored bytes rather than decoded strings (character-set decoding, Shift-JIS/CP932 NFKC normalization in `Platform/systems/src/psx/cp932.rs`, and icon rendering live in the host). The single exception is `emusen_n64_pak::char_of`, which maps the N64 Controller Pak's fixed custom code table.

---

## 2. `emusen-psx-card` — PlayStation Memory Card (`psx.card`)

- **Geometry:** `CARD_LENGTH = 128 KiB` (`16` blocks of `8 KiB`, `64` frames of `128` bytes each). Block `0` holds the header and directory; blocks `1..=15` (`BLOCKS = 15`) store save data.
- **Header & directory:** Frame `0` starts with `"MC"` (`0x4D, 0x43`) and ends with an XOR checksum byte (`checksum`). Directory entries `1..=15` occupy frames `1..=15` of block 0:
  - Block states: `FIRST` (`0x51`), `MIDDLE` (`0x52`), `LAST` (`0x53`), `FREE` (`0xA0`), `DELETED_FIRST` (`0xA1`), `DELETED_MIDDLE` (`0xA2`), `DELETED_LAST` (`0xA3`).
  - Next-block link: 16-bit LE (`NO_NEXT = 0xFFFF`).
  - Filename: 21 bytes at offset `NAME_AT = 0x0A` (`NAME_LENGTH = 21`).
  - Title frame: 64 bytes at offset `TITLE_AT = 0x04` (`TITLE_LENGTH = 64`) inside the save's first data block (Shift-JIS / CP932 on console; decoded by host).
- **API:** `is_formatted`, `state`, `next`, `is_free`, `chain`, `saves`, `free_blocks`, `name`, `title`, `copy`, `delete`, `format`, `CopyError`.

---

## 3. `emusen-n64-pak` — N64 Controller Pak (`n64.pak`)

- **Geometry:** `PAK_LENGTH = 32 KiB`, `PAGES = 128` pages of `PAGE_LENGTH = 256` bytes.
  - Page `0`: ID sector with 4 redundant ID blocks at offsets `ID_BLOCKS = [0x20, 0x60, 0x80, 0xC0]`, each validated by `id_checksums` (sum and `0xFFF2 - sum`).
  - Pages `1` and `2`: Primary and backup inode (index) tables, validated by `index_sum` (`index_page` selects the first valid copy).
  - Pages `3` and `4`: Note directory (`NOTES = 16` notes of `NOTE_LENGTH = 32` bytes each).
  - Pages `5..128` (`FIRST_DATA = 5`, `DATA_PAGES = 123`): Allocatable save pages.
- **Inode markers:** `END_OF_NOTE = 0x0001`, `FREE_PAGE = 0x0003`.
- **Character encoding:** `char_of(u8) -> char` decodes the Controller Pak's 16-byte note name and 4-byte extension code table.
- **API:** `is_formatted`, `index_page`, `entry`, `note`, `start_of`, `in_use`, `chain`, `saves`, `free_pages`, `char_of`, `copy`, `delete`, `format`, `CopyError`, `DeleteError`.

---

## 4. `emusen-mcd-bram` — Sega CD Backup RAM (`mcd.backup`, `mcd.ramcart`)

- **Geometry:** `BLOCK_LENGTH = 64` bytes; valid image lengths (`size_ok`) are powers of two from `MIN_LENGTH = 8 KiB` (`0x2000`, internal BRAM) up to `MAX_LENGTH = 512 KiB` (`0x80000`, RAM cartridge).
- **Footer & directory:**
  - The final `64`-byte block holds the format signature (`format_name`, e.g., `"SEGA_CD_ROM"` / `"RAM_CART___"` variants classified by `Kind`), plus 4 redundant copies of the free-block and file counts (`counts`, `set_counts`).
  - Directory blocks grow backward from the footer (`directory_blocks(files)`), storing `NAME_LENGTH = 11`-byte raw filenames (preserved byte-for-byte on copy/delete even for bytes `>= 0x80`), start block, block length, and protection flag (`src/block_code.rs` encodes/decodes the Sega CD block protect code).
- **Damage safety:**
  - If a corrupted footer specifies a file count whose directory blocks reach below offset `0`, `entry()` returns `None` and edits return `EditError` rather than panicking.
  - `delete(image, index, edited)` reads from the unmodified source image while compacting entries into `edited`, preventing a damaged directory whose file ranges overlap the directory from corrupting reads mid-compaction.
- **API:** `size_ok`, `capacity`, `kind`, `format_name`, `counts`, `set_counts`, `directory_blocks`, `entry`, `entries`, `copy`, `delete`, `format`, `EditError`, `block_code`.

---

## 5. `emusen-ps2-card` — PlayStation 2 Memory Card (`ps2.card`)

- **Geometry:** Raw 8 MiB flash layout with spare bytes:
  - `PAGES = 16,384`, `PAGE = 512` data bytes + `SPARE = 16` spare bytes (`RAW_PAGE = 528`), `CARD_LENGTH = 8,650,752` bytes (`.ps2`, `.2.ps2`).
  - `PAGES_PER_CLUSTER = 2` (`CLUSTER = 1,024` bytes, `CLUSTERS = 8,192`), `PAGES_PER_BLOCK = 16` erase-block pages (`ERASED = 0xFF`).
  - Directory entries are `ENTRY = 512` bytes (`2` per cluster), with up to `NAME_LENGTH = 32` raw name bytes.
- **Superblock & FAT:**
  - Page `0` holds the superblock (`MAGIC = b"Sony PS2 Memory Card Format "`, `VERSION = b"1.2.0.0"`, `CARD_TYPE = 2`).
  - `format` writes the standard 8 MiB layout: indirect FAT cluster `8`, FAT clusters `9..=40`, first allocatable cluster `41`, end cluster `8,135`, backup blocks `1,023` and `1,022`, root directory at cluster `41` containing `"."` (`0x8427`) and hidden `".."` (`0xA426`), free FAT entries `0x7FFFFFFF`, post-allocatable FAT entries `0xFFFFFFFF`, flags `0x52`, and trailing 4 spare bytes zeroed.
  - **Free space count (`free_clusters`):** Matches the PS2 browser display: `7,999` minus clusters in use (`7,998` on a freshly formatted card, with the root taking 1 cluster; 1 cluster = 1 KiB).
- **Hamming ECC (`src/ecc.rs`):**
  - Each 512-byte page is split into four 128-byte chunks; each chunk carries a 3-byte (20-bit) complemented Hamming ECC code in the 16-byte spare area (an erased `0xFF` chunk encodes to `0x77, 0x7F, 0x7F`).
  - Every read checks ECC whenever the 3 code bytes are not erased (`0xFF`), regardless of the superblock's `CF_USE_ECC` flag bit: single-bit data errors are corrected transparently, single-bit ECC errors are ignored, and multi-bit errors return `Damage::Ecc`.
- **API & damage reporting:**
  - `Card::open`, `saves`, `find`, `read`, `clusters_of`, `free_clusters`, `format`, `mkdir`, `create`, `delete`, `copy_save`.
  - Structured `Damage` variants (`Length`, `Unformatted`, `Superblock`, `Ecc`, `Chain`, `Directory`), each with human-readable `words()`, guaranteeing panic-free handling on arbitrary corrupt inputs.
- **Known limits:** Bad-block lists in the superblock are respected during allocation, but new bad blocks are never marked; interrupted-write backup-block recovery is left to console firmware; non-512-byte page geometries are refused as `Damage::Superblock`; `icon.sys` and icon files are treated as ordinary files whose decoding belongs to the host.
