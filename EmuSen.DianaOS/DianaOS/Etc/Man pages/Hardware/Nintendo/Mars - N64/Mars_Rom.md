# Mars — the ROM container and its header

*Phase 0, landed 2026-09-15. This is the first Mars code, and it emulates nothing:
it turns a file on disk into a normalised image and reads the 64 bytes at the front.
`EmuSen/Cores/Nintendo/Mars - N64/Rom/RomImage.cs`, with `SyntheticN64Rom` and
`MarsRomImageTests` in `EmuSen.WiseMan`.*

---

## 1. Three containers, one machine

The same cartridge dump circulates in three byte orders, a legacy of the dumping
hardware that produced them. `.z64` is the console's own order, `.v64` is the Doctor
V64's halfword-swapped order, and `.n64` is a word-reversed order.

### 1.1 The extension is not evidence

**The container is decided by the first word and never by the filename.** The header's
first word is `0x80371240` on hardware, so a file beginning `0x37804012` is halfword
swapped and one beginning `0x40123780` is word reversed. Anything else is not an N64
image and is refused with the offending word in the message.

This is not fussiness. The documentation survey found sources contradicting each
other about what `.n64` even means — the more careful of them describes it as
byte-swapped while several others call it little-endian — and the same source warns
that a file's byte order may not be the one its extension implies
(`Mars_Documentation.md` §6). A parser that trusts the extension inherits that
disagreement; one that reads the magic word cannot. `MarsRomImageTests` pins this
with a `.z64` file containing byte-swapped bytes.

### 1.2 A truncated image is refused, not half converted

Unswapping needs whole units, so an odd-length halfword-swapped file or a
non-multiple-of-four word-reversed one has a tail that cannot be converted. Mars
refuses both rather than converting as far as it can. A half-swapped tail is silent
corruption that surfaces much later as an unexplained bad read, and "this file is
truncated" is a better diagnostic than anything that failure would produce.

The big-endian path returns the caller's array rather than a copy, so the common case
allocates nothing.

## 2. The header

Sixty-four bytes, of which Phase 0 reads the clock rate, the entry point, the
libultra version, both check words, the twenty-byte title, and the four bytes of
game code — category, a two-character unique code, destination, and version.

### 2.1 What "long enough" means

An image shorter than the header plus the boot code that follows it — `0x1000` bytes
together — is refused. That is the least a bootable image can be, and it is the
bound the corpus ROM's own bootstrap assumes (`Mars_TestOracle.md` §3).

### 2.2 PAL is an inference, not a field

There is no video-standard field. The destination code is a field, and the mapping
from it to 50 Hz is a convention: `D`, `F`, `I`, `P`, `S`, `U`, `X` and `Y` are
treated as PAL and everything else as NTSC. It is recorded here as a convention so
that when a title is found that breaks it, the fix is understood as amending a
heuristic rather than correcting a parse.

### 2.3 The check words are read and not verified

Both are stored and neither is checked. Verifying them means implementing the boot
checksum over the boot code, whose algorithm the survey found documented nowhere —
the community wiki lists it as an open item (`Mars_Documentation.md` §5). Nothing in
the phases needs it: the real console's check happens in code Mars replaces with an
HLE boot (`Mars_Gameplan.md` §4.1). `SyntheticN64Rom` therefore writes recognisable
nonsense into both, which is safe precisely because nothing verifies them, and which
will fail loudly the day something does.

## 3. The save type, and the one case where the image answers

`N64SaveType` exists to record what the image says, and for a commercial cartridge
the honest answer is `Unknown`. There is no save-type field in the standard header;
the save chip is separate silicon on the cartridge bus and the ROM carries no
manifest of it. Two independent emulators ship per-title databases of thousands of
entries for this, and one states outright that 4 kbit and 16 kbit EEPROM cannot be
told apart by observation (`Mars_References.md` §5.1, `Mars_Documentation.md` §6).

The exception is the flashcart homebrew convention: an image whose unique code is
`ED` repurposes the version byte, with the save type in its high nibble and flags for
a real-time clock and region-freedom in the low bits. Mars reads it, which costs
nothing and is the only way an image can state its own answer.

**A commercial image returning `Unknown` is the correct behaviour, not a gap.** The
gap is the database that will eventually stand beside it, and Phase E owns that.

## 4. What Phase 0 deliberately does not do

- ~~**No `CoreCatalog` entry.** Nothing reaches this code from a frontend; `.z64` still
  fails `CoreFactory.IsSupported`. Registration is Phase F, for the reason
  `Mars_Gameplan.md` §4.6 gives.~~ **Registered 2026-09-18, ahead of Phase F**, in
  `Mars_Core.md`, whose §0 says what that departs from and which seams are still stubs.
- **No cartridge object, no memory map, no PI.** This reads a file; it does not model
  hardware. The boot handoff that copies the first `0x1000` bytes into SP DMEM is
  Phase A's, and it will consume `RomImage` rather than re-reading the file.
- **No 64DD disk images.** A different container entirely, deferred with the rest
  (`Mars_Gameplan.md` §6).

## 5. The corpus, and where it is found

`N64TestRomLibrary` mirrors `HardwareTestRomLibrary`: it looks in the gitignored
`TestRoms/n64` beneath the sandbox install directory — which resolves to the
repository root in a test run, because `ConfigRoot` walks up to `EmuSen.sln` — and
treats an absent corpus as a skip rather than a failure, so the suite passes on a
machine with no ROMs at all.

One test reads the real corpus ROM when it is installed, asserting its container,
entry point and internal title. It is the first real Nintendo 64 file this project
has parsed. Its value is that it is *not* synthetic: the fixture and the parser share
assumptions, and a real image is the only thing that catches an assumption they share
wrongly. Because the test returns early when the ROM is absent, it was verified by
mutation — a wrong expected entry point fails against the installed ROM — rather than
by trusting a green run.
