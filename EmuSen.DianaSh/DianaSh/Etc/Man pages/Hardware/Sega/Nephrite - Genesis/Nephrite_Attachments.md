# Nephrite_Attachments — the Sega CD and the 32X behind a written interface

*Written 2026-10-15, before any code.* This page designs the surface between the Genesis and its attachments, so that
the Sega CD and the 32X become crates of their own while Nephrite stays one library for three machines. It changes no
code; the build is a separate step, taken after stage 9 merges, from §7 of this page.

It keeps the plan's three registers apart (`Nephrite_Plan.md`, opening):

- **Measured** means counted on 2026-10-15 in the tree at WiseMan `d32a00b3`, with the command named; nothing in this
  page is a timing.
- **Argued** means reasoning from the code, the documents or other cores' records, with no measurement behind it.
- **Predicted** means a claim numbered A1–A5 (§9), kept so that the build can retire it.

---

## 1. The decision, its aims and what this page does not do

*Decided 2026-10-04 (`Nephrite_Plan.md` §1.1), and standing:* Nephrite is one library for three machines. The Sega CD
and the 32X are attachments of the Genesis, neither runs without one beneath it, and three system ids are reported.

*Decided 2026-10-15 (the tester's go):* each attachment becomes its own crate behind a written attachment interface,
in place of a module folder that the Genesis side reaches into. The Beryl crates (`beryl-m68k`, `beryl-z80`,
`beryl-sh2`, `beryl-sn76489`) and `emusen-disc` stay as they are: shared hardware and shared discs.

The aims, in order:

1. **The boundary enforced by the compiler.** The Genesis's crate cannot name an attachment; an attachment reaches the
   Genesis only through the interface's types.
2. **A Genesis-only build**, which compiles and tests with no attachment present.
3. **The 32X begins on the interface** (stages 11–12): its SH-2s, system registers, frame buffers, overlay and PWM
   arrive as an implementation of the same traits the Sega CD implements, not as a second set of special cases in
   `genesis.rs`.
4. **The cartridge Genesis unchanged**: byte-identical in picture, sound and state, and no slower.

What the page does not do: it builds nothing, times nothing and reads no code outside Nephrite's own tree and the
pages it cites. No emulator's source, no RTL and no firmware was read to write it (§10).

---

## 2. The measured coupling

### 2.1 Method

Counted in `EmuSen/Cores/Sega/Nephrite - Genesis/src/` at `d32a00b3` with `grep -n` over the identifiers and gates
that belong to the Sega CD (`mcd`, `MegaCd`, `RamCart`, `CARTRAM`, `MCD_`, `RAM_CART`, `disc_print`, `System::Mcd`,
`cdd::`, `open_disc`, `system_area`, `fingerprint`, `cd_bios`, `CD_BIOS`, `.brm`, `.crm`, `STATUS_BAD_DISC`,
`MEDIA_FILE`), the J-Cart's (`jcart`, `JCART`, `attached`) and the 32X's (`S32x`, `S32X`). The list is kept in
`~/.cache/emusen/probe/nephrite/attach/genesis-side.txt`. Each match was read and classified by hand.

A **site** is one line of production code that names an attachment's item, reads or writes its state, or decides by
whether an attachment is present. Lines inside a file's `#[cfg(test)]` module, and the test-only files, are counted
apart. Where one statement spans several lines, the line that reaches across is cited once.

### 2.2 The figures of 2026-10-15, reconciled

The count taken on 2026-10-15 is lines naming `mcd`: `genesis.rs` 17, `machine.rs` 10, `v1.rs` 23, `state.rs` 4,
`media.rs` 1, `lib.rs` 1, **55**. Read again: `v1.rs`'s 23 are 16 in production and 7 in its tests, so **48 lines of
production code name `mcd`**. The inventory below (§2.3) counts **96 production sites**, because it also counts the
places that reach the Sega CD without naming it: gates on `System::Mcd`, the disc helpers, the firmware and battery
numbers and the space ids.

The reverse figure, "about 20" lines of `src/mcd/` reaching `crate::`, is **25 lines** naming `crate::` or `super::`.
Thirteen name `mcd` itself by an absolute path or its parent (`use super::*` in the test modules, `crate::mcd::` in
`cdda.rs`'s tests, `discs.rs` and `verificator.rs`, `super::` in `state.rs`). Twelve reach outside the module, and of
those **two are production code**: `cdda.rs:26` (`crate::sound::RATE`) and `mod.rs:283` (`crate::media::DISC_SIGNATURE`).
The other ten are the two test files that run a whole machine (`verificator.rs` 5, 6, 13, 14, 21, 90, 91, 103;
`discs.rs` 53, 54).

So the Sega CD is far more self-contained than the Genesis is towards it: the Genesis side reaches in at 96 places and
the Sega CD reaches out at 2. Most of the work of the split is on the Genesis side, and most of that is in `v1.rs` and
`machine.rs`, not in the console.

### 2.3 The Genesis side's reach into the Sega CD, by kind

| Kind | Sites | Where (file:line at `d32a00b3`) |
|---|---|---|
| Construction and ownership | 9 | `genesis.rs` 136 (`Hw::mcd`), 137 (`mcd_low`), 150, 159 (`with_mcd`), 190, 192 (`MegaCd::new` given `master_hz` and `START`); `machine.rs` 93, 111; `lib.rs` 23 |
| Map | 18 | `genesis.rs` 139 and 191 (the `attached` flag, shared with the J-Cart); the four guards 438, 458, 478, 499; the Sega CD's decode 523, 530, 542; `mcd_read8` 553, 555, 557; `mcd_read16` 561, 563, 565; `mcd_write` 569, 571, 578 (the RAM cartridge reached through `m.hw.cart`, the rest through `main_read*`/`main_write`) |
| Interrupts | 0 | the 68000's level is the VDP's alone (`genesis.rs` 781–785); the Sega CD's H-INT vector and IFL2 travel through the map (§2.4) |
| Clock and slices | 1 | `genesis.rs` 379: `m.run_to(self.clock)` in `catch_up`, after each 68000 instruction |
| Reset and power | 0 | power-on is construction; Nephrite claims no `RESET` (`v1.rs` 312). Stage 9 adds one (§2.8) |
| Audio | 3 | `machine.rs` 246, 248 (`cdda.feed`), 249 (`take_with` lent the Sega CD's `Steps`) |
| Video | 0 | the Sega CD has no picture of its own; the VDP's transfers from Word RAM pass through the map (§2.5) |
| Pads | 0 | — |
| State | 8 | `state.rs` 16 (`OLDEST_MCD_STATE_VERSION`), 28 (`WrongMedia`), 68 (the block written), 75 (the oldest version by system), 104 (read), 108 (the fingerprint held), 116 (applied); `machine.rs` 66 (`disc_print`) |
| Media and discs | 21 | `machine.rs` 88 (`open_disc`), 94 (`fingerprint`), 96 (no ROM for a disc); `media.rs` 144–147 (`DISC_SIGNATURE`, `disc_offset`), 202, 228; `v1.rs` 184 (`MEDIA_FILE`), 186 (`STATUS_BAD_DISC`), 212 (`is_disc`), 218, 224, 228 (`open_disc`, `system_area`), 355, 365 (create's dispatch), 584 (`drives`), 591 (`media_lid`), 597, 604, 606, 607 (`media_insert`), 622 (`create_disc`) |
| Firmware | 10 | `machine.rs` 107; `v1.rs` 172 (`CD_BIOS_U..J`), 245 (`CD_COST`), 248 (`cd_bios`), 335, 342 (`firmware_for`), 380 (create's choice of boot ROM), 425 (machine info), 624, 629 (`create_disc`) |
| Battery files | 8 | `machine.rs` 189 (file 0 is `BRAM` on a disc); `v1.rs` 182 (`RAM_CART`), 361 (`battery_files`), 577 (`battery`), 636, 637 (loaded at create), 650, 654 (`batteries`) |
| Debugger and spaces | 11 | `machine.rs` 34 (`CARTRAM_ID`), 35 (`MCD_MEMORIES`), 125 (no `ROM` on a disc), 128, 129, 131 (`spaces`), 147, 149, 157 (`bytes`), 173, 181 (`bytes_mut`) |
| Settings | 4 | `v1.rs` 189 (`RAM_CART_KEY`), 191 (`ram_cart_setting`), 348 (the schema), 631 (applied at create) |
| Machine info | 3 | `v1.rs` 308 (`MCD_EXTENSIONS`), 322 (the description), 325 (the `mcd` row, marked in development) |
| **Total** | **96** | |

### 2.4 The Sega CD's reach into the Genesis

**Named**, two production lines: `cdda.rs:26` builds its step synthesiser at the Genesis's output rate
(`crate::sound::RATE`), and `mod.rs:283` tells a lone image of 2,048-byte sectors by the Genesis-side constant
`crate::media::DISC_SIGNATURE`, which is the Sega CD's own fact filed in the wrong module.

**Not named, but relied on**, five contracts the Sega CD keeps with the Genesis without importing anything:

- **The clock.** Every time the Sega CD is given is a master clock of the Genesis's (`mod.rs` 153 `new(master_hz, …,
  master_now, …)`, 186 `run_to(master)`, 214, 241, 254 the main side's `master`), and it converts both ways by an
  exact ratio (`to_sub`, `to_master`, 316–323). It never moves the Genesis's clock.
- **The 68000's map in mode 2.** The Sega CD decodes main-CPU addresses itself: the boot ROM and PRG-RAM's window by
  `a & 0x2_0000` below `$200000`, Word RAM above it (`mod.rs` 222–238), the registers from `$A12000` (216, 242, 256),
  and its RAM cartridge decodes `$400000`–`$7FFFFF` (104–149). It answers `None` where nothing drives the bus, and the
  Genesis supplies its open bus (`genesis.rs` 557, 565).
- **The 68000's interrupt vector.** The H-INT vector read at `$000070`/`$000072` (`mod.rs` 224–225) is the Sega CD's
  answer through the map, not an interrupt line.
- **The mix.** CD-DA's samples are placed at master clocks (`mod.rs` 386–390) and scaled against the Genesis's output
  (`cdda.rs` 15, `GAIN`, D-35), and the Genesis adds them after its output circuit (`sound.rs` 176–206).
- **The open bus and the requester.** A Z80 access through its window and a VDP transfer both reach the Sega CD
  through `Hw::read8`/`read16`, presented at the 68000's present (§2.5).

### 2.5 Paths that cross without naming either side

- **The VDP's transfers.** `dma_read` (`genesis.rs` 370–372) reads through `read16`, so a transfer from Word RAM to
  VRAM reaches the Sega CD by the map, at `self.clock`, the 68000's present, not the transfer's slot.
- **The Z80's window** (`genesis.rs` 801–823) reaches the 68000's map through `read8`/`write8`, also at the 68000's
  present. Both are today's behaviour and both are kept (§3.4).
- **The order of catching up.** `catch_up` brings the VDP to the present, then the Sega CD, then the Z80
  (`genesis.rs` 377–395). The order is part of the machine's behaviour and is kept.

### 2.6 What is missing, found by the count

- **The debugger cannot see the Sega CD.** `peek_m68k` (`debugger.rs` 94–102) answers the cartridge, the Z80's RAM and
  the main RAM and `$FF` elsewhere, so a disc's machine shows `$FF` where its boot ROM, PRG-RAM's window and Word RAM
  stand; `processors` (120–126) lists the 68000 and the Z80 alone, and `debug_disassemble` refuses any processor past
  the Z80 (`v1.rs` 567). The plan reserves `S68K` and `S68KBUS` (16) for it (`Nephrite_Plan.md` §4.3). Argued from the
  code, not run.
- **No reset line.** Nephrite does not claim `RESET`, so neither the Genesis nor an attachment is reset after power-on.
  A disc on the player's BIOS already asks for the button ("Press RESET to open the CD tray.", `Nephrite_Native.md`
  §55.4 on the stage 9 branch).

### 2.7 The J-Cart's and the 32X's coupling today

**The J-Cart**, 17 sites, all inside what will be the Genesis's crate: `cart.rs` 23, 39, 42, 61 (the board known by
its header); `io.rs` 150, 154, 199, 206, 210, 218 (players 3 and 4, the word, TH); `genesis.rs` 160, 191, 195 (the flag
copied and the `attached` check turned on), 519, 527, 536 (the two words decoded ahead of the cartridge); `v1.rs` 151
(machine info's players).

**The 32X**, 15 sites, none of them hardware, since none is built: `machine.rs` 36 (`S32X_MEMORIES`), 104 (its three
placeholder memories made), 225 (`runs`: a 32X image counts frames only); `media.rs` 10, 18, 137 (`is_32x`), 205;
`state.rs` 54 (its system byte); `v1.rs` 175 (`S32X_68K..SLAVE`), 246 (`S32X_COST`), 262 (`s32x_boot`), 309, 326, 343,
426. The crate already depends on `beryl-sh2` and uses nothing of it.

### 2.8 What stage 9 adds, read from its branch

At `nephrite-s9`'s first step (`889f497f`, merged with WiseMan as `77c8ba7f`), one new site on the Genesis side:
`genesis.rs` 196–199 clears the version register's bit 5 when a Sega CD is attached ("expansion unit present",
`Nephrite_Native.md` §55.3), a reset-and-power fact. The step's other additions are inside `src/mcd/` (a watch on the
registers, `Sub::seen`, and the drive's rules of D-36). The steps still to come in stage 9 (§49.4 there: the graphics
ASIC with level 1, the RF5C164 with its eight channels, the fader's ramp, de-emphasis and the mix's levels) add, by
their description, sound into the Sega CD's stream, a sub-CPU interrupt and memories; nothing in them reaches the
Genesis's picture or its 68000's interrupts. The interface below is written to take them without a change (§3.8).

---

## 3. The interface

### 3.1 The shape, from the hardware

The Genesis has two connectors that matter here: the **expansion port** on its side, where the Sega CD plugs in, and
the **cartridge slot**, where a cartridge, the Sega CD's RAM cartridge or the 32X plugs in, the 32X carrying a
cartridge (or, with a disc, nothing) in its own slot on top. The interface follows that chain:

- The Genesis decodes its own devices (the Z80's area, I/O, `$A11100`/`$A11200`, TMSS, the VDP, the main RAM) as it
  does now, and owns the **cartridge** as now (`Hw::cart`).
- An **attachment** is asked first for every 68000 access, exactly where the `attached` guard stands today
  (`genesis.rs` 438, 458, 478, 499). It answers, declines, or leaves the bus open.
- Where an attachment stands between the 68000 and the cartridge (the 32X), it is **lent the cartridge** for the call,
  so that it can pass an access on, bank it, or let an SH-2 read it.
- With both attachments (CD32X), the expansion port's device is asked before the slot's (§3.10).

Three small traits carry it, by job, rather than one wide one: `EmuSen_Surfaces.md` §4.6 records what a single
interface every engine implements whole costs (`IDebugTarget`), and the three here have different callers. The
**bus side** (`Attachment`) is the only one the Genesis's hot code calls. The **state** (`AttachmentState`) and the
**view** (`AttachmentView`: spaces, processors, the debugger) are called by the machine between frames. What a frontend
is told about files, media and settings is not a trait at all (§3.6).

### 3.2 The clock

**The bus owns the clock** (`Beryl-HW/README.md`), at every level:

- The **Genesis's** 68000 bus owns the master clock (`Hw::clock`), as now. Each attachment call carries `at`, the
  Genesis's present: the end of the 68000's bus cycle, as `main_read16(a, self.clock)` carries it today. Times given to
  an attachment never decrease.
- An **attachment's** processors' buses own their own clocks (the sub CPU's `SubBus` counts 12.5 MHz clocks; the 32X's
  SH-2 buses will count theirs), each converting to and from master clocks by its exact ratio, as `Sub::to_sub` does.
  The Genesis never reads an attachment's clock.
- An attachment **moves the Genesis's time only by a wait** it reports for the access it answers (`Slot::wait`, §3.3),
  which the Genesis's bus adds to the 68000's cycle, as it adds the Z80 window's `stall` today. The Sega CD reports
  none; the 32X's frame buffer and its contention for the cartridge will.

**A slice is bounded** (`Nephrite_Plan.md` §5.4) by three things the Genesis provides, and one the attachment keeps:

1. `run_to(to)` after each 68000 instruction, the Genesis's present as `to`, in `catch_up`'s present order (VDP,
   attachment, Z80).
2. A map call at `at`, before which the attachment brings itself to `at`, as `main_read16` calls `run_to` today.
3. For an attachment that takes line events (the 32X), each line's start, at its master clock.
4. **Inside the attachment**: how far it may lag. `run_to` is a bound, not an order: an attachment may stop short of
   `to` when nothing it decodes can have been observed, provided that every answer it gives at `at` is as at `at`. The
   Sega CD keeps today's rule (it runs to every `to`, instruction by instruction); the 32X keeps §5.4's (to every
   access to its areas and every line, and otherwise in slices of a measured length, P5's kind of parameter, a
   setting with `effect: accuracy` only if §5.5's budget is missed). The bound lives in the attachment's crate, so a
   change to it never touches the Genesis.

### 3.3 The bus side: `Attachment`

In `nephrite-genesis`, module `attach` (§5):

```rust
/// A time on the Genesis's master clock: master clocks since power-on.
pub type Master = u64;

/// What the console tells an attachment as it is built.
#[derive(Clone, Copy)]
pub struct Board {
    pub master_hz: (u64, u64), // Genesis::master_hz(pal), a fraction in Hz
    pub pal: bool,
    pub start: Master,         // genesis::START, the 68000's first clock
    pub rate: u32,             // sound::RATE, the mix's output rate
}

/// What the console lends for one call: the slot's cartridge, and the wait the access adds to the 68000's cycle,
/// set by the attachment and applied by the Genesis's bus.
pub struct Slot<'a> {
    pub cart: &'a mut Cart,
    pub wait: Master,
}

/// An answer to a read in the 68000's space.
pub enum Answer<T> {
    Pass,     // not decoded here: the Genesis's own map answers
    Open,     // decoded here with nothing driving the bus: the Genesis's open bus answers
    Value(T),
}

/// A line of the VDP's, delivered as it ends.
pub struct Line<'a> {
    pub number: u32,
    pub at: Master,
    pub vblank: bool,                                // the vertical blank begins with this line
    pub picture: Option<(&'a mut [u8], &'a [u64])>,  // the line as drawn (RGBA) and a bit a pixel where the
                                                     // backdrop shows; None in a frame not drawn
}

/// The debugger's side of a run; `Unwatched` compiles to nothing.
pub trait Watch {
    fn hooks(&mut self) -> Option<&mut emusen_native::debug::Hooks>;
    fn skip(&mut self, processor: u32) -> bool;      // the processor's next instruction runs unasked, once
}

pub trait Attachment {
    /// An expansion unit on the side port: the version register's bit 5 reads 0 (Nephrite_Native.md §55.3).
    const EXPANSION: bool = false;
    /// Takes the VDP's line events; the Genesis then brings its VDP to `at` before every call (§3.4).
    const LINES: bool = false;

    fn read8(&mut self, a: u32, at: Master, slot: &mut Slot<'_>) -> Answer<u8>;
    fn read16(&mut self, a: u32, at: Master, slot: &mut Slot<'_>) -> Answer<u16>;
    /// `mask` is $FF00, $00FF or $FFFF as the 68000 drives the lanes, a byte on both halves of `v`; false: not decoded.
    fn write(&mut self, a: u32, v: u16, mask: u16, at: Master, slot: &mut Slot<'_>) -> bool;
    /// A byte as a debugger reads it, without side effects; None where the Genesis's own peek answers.
    fn peek(&self, a: u32, cart: &Cart) -> Option<u8>;

    /// Brought towards `to`, never past it: (0, 0) when done, else a watch's stop and the processor it is on.
    fn run_to<W: Watch>(&mut self, to: Master, slot: &mut Slot<'_>, w: &mut W) -> (u32, u32);
    /// The console's RESET line, pulsed at `at`.
    fn reset(&mut self, _at: Master) {}
    fn line(&mut self, _line: Line<'_>, _slot: &mut Slot<'_>) {}
    /// Its sound brought up to `to`, each stream lent to the mix in turn, added after the Genesis's output circuit.
    fn sound(&mut self, _to: Master, _each: &mut dyn FnMut(&mut emusen_native::steps::Steps)) {}
}
```

**What is left out, and why.** No interrupt line towards the 68000: neither attachment drives one as built or as
planned, the H-INT vector and IFL2 being the map's (§2.4); if stage 10 or 12 finds a source, `fn level(&mut self, at)
-> u8` is added then, with its test. No "who is asking" on map calls: the VDP's transfers and the Z80's window are
presented as the 68000's accesses, as today (§2.5); if stage 9 or 10 finds a transfer from Word RAM behaving apart
from a 68000 read, the requester is added then. An addition to an internal surface costs one commit; an unused member
costs a reader's attention in every implementation.

### 3.4 What the Genesis offers in return, and promises

- **`Board`** at construction: the master clock's exact rate, the market, the 68000's first clock (power-on is
  construction) and the output rate. An attachment imports none of these as constants (`cdda.rs:26` stops reading
  `sound::RATE` directly).
- **`Slot`** on every call that can need it: the cartridge, lent, and the wait.
- **The open bus**: `Answer::Open` gives the Genesis's (`open8`, the prefetch word), so the rule of D-29 has one owner.
- **Line events**, with `LINES`, in time order, each after the line's pixels are final (after `vdp.line_end`) and
  before the next line's start; `picture` is `None` when the frame is not drawn, so an overlay changes no state and
  `skip_rendering_state_neutral` stays true. The Genesis's own picture is composed in place.
- **The order.** After each 68000 instruction: the VDP to the present, the attachment's `run_to`, then the Z80
  (today's). With `LINES`, the VDP is also brought to `at` before every map call, so that an attachment is never
  ahead of the last line delivered to it; without `LINES` (the Sega CD) nothing is added to the map's path.
- **The version register's bit 5** from `EXPANSION`, set once at construction.
- **The mix**: each stream the attachment lends is taken to the frame's end and added sample for sample after the
  Genesis's output circuit, clamped once, as `take_with` adds CD-DA today (`sound.rs` 202–203). The output scale is the
  Genesis's; each attachment's gain against it is its own (`cdda.rs` `GAIN`, D-35).

The Genesis's side becomes generic: `Genesis<A>`, `Hw<A>` (the attachment a field, `att`, beside `cart`),
`MainBus<'_, A>`, `Z80Bus<'_, A>`. `Hw::mcd`, `mcd_low` and `attached` go. The four guards become
`_ if let Answer::Value(v) = self.att.read16(a, self.clock, &mut slot) => v` with `Answer::Open` falling to the open
bus, and `catch_up`'s three lines become `self.att.run_to(self.clock, &mut slot, w)`. A Genesis with nothing attached
is `Genesis<Bare>`, whose methods are empty and inline, so that every attachment arm folds away (§4).

### 3.5 The state and the view: `AttachmentState`, `AttachmentView`

```rust
pub trait AttachmentState {
    type Saved;
    fn write_state(&self, w: &mut StateWriter);
    fn read_state(&self, r: &mut StateReader, version: i32) -> Result<Self::Saved, Truncated>;
    /// Whether a state read whole may be applied: a Sega CD's is held to the disc in its drive.
    fn check(&self, saved: &Self::Saved) -> Result<(), Refusal>;   // Refusal::WrongMedia { saved, here }
    fn apply_state(&mut self, saved: Self::Saved);
}

pub trait AttachmentView {
    fn spaces(&self, out: &mut Vec<SpaceRef>);                      // in id order, from its own range
    fn bytes(&self, id: u32) -> Option<&[u8]>;
    fn bytes_mut(&mut self, id: u32) -> Option<&mut [u8]>;
    fn read_bus(&self, space: u32, address: u32, out: &mut [u8]) -> bool; // its processors' buses, no side effects
    fn processors(&self, out: &mut Vec<Processor>);
    fn pc(&self, processor: u32) -> Option<u64>;
    fn registers(&self, processor: u32) -> Option<Vec<i64>>;
    fn disassemble(&self, processor: u32, space: u32, address: u32, count: u32) -> Option<Vec<Instruction>>;
    fn set_breakpoints(&mut self, processor: u32, pairs: &[i32]) -> bool; // an attachment keeps its processors'
}
```

**How the state composes.** The format is the library's, one magic and one version (`NPHR`, §7.3). In order: the
header (magic, version, system byte, frames) written by the assembly crate; every writable space in id order, the
Genesis's (2–7) then the attachment's (from 16 for the Sega CD, from 32 for the 32X); the Genesis's block
(`Genesis::write_state`); then each attachment's block, the Sega CD's before the 32X's. This is today's order exactly
(`state.rs` 59–71): the spaces listing already puts the Sega CD's after the Genesis's, and its block follows the
Genesis's. The disc's fingerprint, today `Machine::disc_print` passed in (`state.rs` 69), becomes the Sega CD's own
field and is written as the block's last eight bytes, where it stands now (`mcd/state.rs` 102). The version is passed
to every reader; the sequence of versions and each system's oldest are the assembly crate's (§5.4), the branches on a
version inside a block its owner's, as `genesis.rs` 1079 and 1092 are today.

**How spaces and processors compose.** Each owner has a fixed range of ids, so ids never move between systems and a
configuration's lists are concatenations:

| Owner | Space ids | Spaces | Processor ids | Processors |
|---|---|---|---|---|
| Genesis | 0–15 | `M68KBUS` 0, `Z80BUS` 1, `WRAM` 2 … `ROM` 8 | 0–1 | `M68K` 0, `Z80` 1 |
| Sega CD | 16–31 | `S68KBUS` 16, `PRGRAM` 17, `WORDRAM` 18, `PCMRAM` 19, `BRAM` 20, `CARTRAM` 21 | 2 | `S68K` 2 |
| 32X | 32–47 | `MSH2BUS` 32, `SSH2BUS` 33, `SDRAM` 34, `FRAMEBUFFER` 35, `PALETTE` 36 | 3–4 | `MSH2` 3, `SSH2` 4 |

The space ids are the plan's (§4.3); the processor ids are proposed here (Q7). The ranges are the interface's, in
`attach`; each id within a range is its owner's constant. The debugger's `Hooks` are made with a width for each id up
to the highest the configuration lists (`machine.rs` 111 makes `[24, 16]` today). The Genesis's `peek_m68k` asks
`att.peek` first, which closes §2.6's first gap; in the move itself the Sega CD's `peek` answers `None` and its
processor list is empty, as today, so that the move changes nothing a debugger sees, and its processor arrives with the
Sega CD's debugger step.

### 3.6 Facts for the frontend: files, media, settings, machine info

Batteries, drives and discs, firmware, settings and the system rows are answered once per call between frames and are
read by one caller, the assembly crate, which knows each configuration's concrete types. They are **plain functions and
inherent methods of each attachment's crate**, composed by the assembly, not trait members:

- **Firmware**: each crate describes its images by name, label, size and its replacement's words (`nephrite-mcd`:
  `bios_CD_U/E/J.bin` and `CD_COST`; `nephrite-32x`: the three boot ROMs and `S32X_COST`; the Genesis: TMSS). The ABI's
  file numbers (1–10 today) are the assembly crate's single table.
- **Battery files**: each crate names its files by suffix and space (`.brm` on `BRAM`, `.crm` on `CARTRAM`) and loads
  them; the assembly numbers them (0 and 10 today) and answers `battery`.
- **Media**: `drives`, `media_lid` and `media_insert` call `MegaCd`'s inherent methods; opening a disc
  (`open_disc`, `is_disc`, `system_area`, `fingerprint`, `DISC_SIGNATURE`) is `nephrite-mcd`'s, so the assembly no
  longer depends on `emusen-disc` directly. Whether the library claims `MEDIA` (allowed for a machine with no drive
  since the core ABI's 1.6, `EmuSen_CoreAPI.md` §6.26) is the assembly's decision.
- **Settings**: each crate lists its settings (`ramcart` in `nephrite-mcd`, a slice bound in `nephrite-32x` if one is
  ever needed) and applies them at create; the assembly concatenates the schema.
- **Machine info**: the system id, extensions, the development mark, frame rate, ports and the description are the
  assembly's; spaces, processors, batteries and firmware are composed from the parts.

### 3.7 Each kind, answered

| Kind | Through | Sega CD | 32X | CD32X |
|---|---|---|---|---|
| Map | `read8`, `read16`, `write`, `peek`; `Answer`; `Slot` | mode 2's `$000000`–`$3FFFFF`, the slot at `$400000` (RAM cartridge, or `$FF`, D-32), `$A12000`–`$A1203F`; mode 1 at `$400000` (stage 10) | its registers, its windows and the vector override; the cartridge beneath through `Slot::cart`; waits through `Slot::wait` | `Both`: the Sega CD asked first |
| Interrupts | none towards the 68000; into an attachment by the map and by lines | IFL2 and the H-INT vector by the map; the sub CPU's levels inside | V and H from lines, CMD by the map, PWM inside | each its own |
| Clock and slices | `run_to`, `Board`, `Slot::wait` | to every `to`, instruction by instruction (today) | §5.4's slices, the bound its own | both to the same `to`, the Sega CD first |
| Reset and power | `Board` (power-on), `reset`, `EXPANSION` | `EXPANSION`; the reset line | the reset line | both |
| Audio | `sound` | CD-DA, and the PCM from stage 9, one stream | PWM, one stream | two streams |
| Video | `LINES`, `Line` | none | its VDP's timing and the overlay | the 32X's |
| Pads | none | none | none | none |
| State | `AttachmentState`, after the Genesis's block | its block, the fingerprint, `WrongMedia` | its block | both blocks (Q5) |
| Media and discs | inherent, composed | the drive, the tray, the disc | none | the Sega CD's |
| Firmware | inherent descriptors, numbered by the assembly | three BIOS images | three boot ROMs | both sets |
| Battery files | inherent, numbered by the assembly | `.brm`, `.crm` | none | the Sega CD's |
| Debugger and spaces | `AttachmentView`, the id ranges | `S68K`, 16–21 | `MSH2`, `SSH2`, 32–36 | all |
| Settings | inherent lists, applied at create | `ramcart` | a slice bound, if needed | both |
| Machine info | the assembly's, composed | — | — | — |

### 3.8 The Sega CD on it

`MegaCd` implements the three traits by moving code, not by writing it: `read8`/`read16`/`write` are `genesis.rs`
518–579 with the J-Cart's half removed (§3.11) and the RAM cartridge reached as now; `run_to` is `mod.rs` 186 unchanged
(`#[inline(never)]` kept, so that a cartridge build's loop is as small as it was); `sound` is `machine.rs` 248's feed
and lends `cdda.steps`; the state is `mcd/state.rs` with the fingerprint its own. `EXPANSION` is true, which is stage
9's bit. `LINES` is false, so no map call gains a VDP catch-up.

What stage 9 adds fits without a change to the surface: the graphics ASIC and its level 1 are sub-side (its clock is
`run_to`'s, its memory Word RAM's); the RF5C164 sums into the Sega CD's one stream, with CD-DA, behind the fader's ramp
and de-emphasis, so that the mix's levels between the Genesis and the Sega CD (step 9.3) are the Sega CD's gains
against the Genesis's scale; the PCM's registers and memory are spaces it already lists. Stage 10's boot-ROM
replacement is an image in the boot ROM's place, which the interface does not see; mode 1 is a configuration (§3.12).

### 3.9 The 32X on it (stages 11–12)

`nephrite-32x` holds two `beryl-sh2` processors on their own buses, the system registers, the comm ports, the FIFO from
the 68000, SDRAM, the frame buffers and the 32X VDP, PWM and the boot-ROM replacements. On the interface:

- **Map**: it answers its registers and windows, takes the vector override, and passes the cartridge's addresses to
  `Slot::cart`, banked as its registers say; an SH-2's read of the cartridge goes to the same `Slot::cart` during
  `run_to`. Its accesses during display report their wait in `Slot::wait`.
- **Clock**: SH-2 clocks at 3/7 of the master clock, an exact ratio; the two SH-2s interleave inside `run_to` at the
  attachment's own bound.
- **Lines** (`LINES`): its VDP's line and frame timing, the frame buffers' swap at the vertical blank, and its SH-2s' V
  and H interrupts, are driven by the Genesis's lines; the overlay is composed in `Line::picture` from the Genesis's
  pixels and the backdrop mask, which the VDP computes only for a configuration with `LINES` (a `const` test, so the
  cartridge Genesis computes none).
- **Sound**: PWM as one stream.

The 32X is the budget's hardest load (`Nephrite_Plan.md` §5.5, P2), which is the strongest argument for §4's choice:
its `run_to` is called after every 68000 instruction and its map on every access to `$800000`–`$9FFFFF`.

### 3.10 Both at once (CD32X)

`Both<A, B>` in `attach` composes two attachments with no knowledge of either: the map asks `A` (the expansion port's)
then `B` (the slot's); `run_to` takes `A` then `B` to the same `to`; lines go to both; `sound` lends both streams;
`EXPANSION` is either's; state, spaces and processors are `A`'s then `B`'s. The configuration is
`Genesis<Both<MegaCd, S32x>>`. If stage 12 finds an address both decode (in mode 2 the Sega CD fills `$000000`–
`$3FFFFF`, where the 32X's vector override also stands), the pair's rule goes in a type of the assembly crate that
replaces `Both` for that pair, since only the assembly may name both crates; `nephrite-32x` never depends on
`nephrite-mcd`. How a disc is known to want the 32X is stage 12's question, with its state (Q5).

### 3.11 The J-Cart: a cartridge feature, not an attachment

*Recommended, with its reasons:* the J-Cart stays a feature of the cartridge, in the Genesis's crate.

- It is on the cartridge's board, selected by the header (`cart.rs` `JCARTS`), and its games are `md` games.
- Of the interface's kinds it has only a map (two words in the cartridge's own range) and the pads. It has no clock,
  processor, state block, space, battery, firmware or setting; TH is the pads' own and the state already holds it
  (`Nephrite_Native.md` §48.2).
- Its words are the console's pads 3 and 4 (`io.rs` 199–222). As an attachment it would need the console's pads lent
  to it, a member no real attachment uses.
- As an attachment, a J-Cart game would run as an attached configuration, with an attachment arm on every access:
  the cost stage 8 measured and removed (§4.1).

So its decode moves from ahead of every access (`genesis.rs` 519–521, 527–529, 536–541) into the cartridge's arm, where
only cartridge accesses pay its check, and `Io` keeps the word. Sonic & Knuckles' lock-on is the same kind of thing and
stays in `Cart`.

### 3.12 Mode 1 (stage 10)

A cartridge with the Sega CD beneath it is `Genesis<MegaCd>` with the cartridge in `Hw::cart` and the Sega CD decoding
`$400000`–`$7FFFFF` (its boot ROM at `$400000`) and its registers, declining `$000000`–`$3FFFFF` so that the Genesis's
cartridge arm answers. The Sega CD learns which mode it is in from its constructor, as `mcd_low` is decided today. Its
two consequences outside the interface are the assembly's: a battery numbering in which the cartridge's save RAM and
the backup RAM both exist (Q8), and a state whose system byte no longer decides its blocks (Q5).

---

## 4. Dispatch: keeping the call out of the cartridge's hot path

### 4.1 How often the interface is crossed (argued)

Per frame of the NTSC console: up to about 32,000 68000 bus cycles (`Nephrite_Plan.md` §5.5), each a map call; about
13,000 instructions (a 68000 clock of 128,000 a frame over a mean of about ten clocks an instruction, argued), each a
`run_to`; 262 lines; one `sound`. Stage 8 measured what a cost on the first of these is worth: the attachment's checks
on every access cost the cartridge Genesis **1.4%**, and one check for anything attached removed it
(`Nephrite_Native.md` §54.1). The cartridge Genesis must pay nothing new; an attachment must not pay an indirect call
per access or per instruction where a direct one is possible.

### 4.2 The three forms

- **(a) One instantiation per configuration.** `Genesis<A>` is compiled for `Bare`, `MegaCd`, `S32x` and
  `Both<MegaCd, S32x>`; the assembly holds the machine as `enum Fitted { Md(Console<Bare>), Mcd(Console<MegaCd>),
  S32x(Console<S32x>), Cd32x(Console<Both<MegaCd, S32x>>) }` and matches it once per ABI call. Every attachment call is
  a direct call, inlined where small. For `Bare` every arm folds away: the cartridge Genesis loses even today's one
  check, the J-Cart's moving into the cartridge arm.
- **(b) One instantiation over an enum.** `Genesis<Attached>`, where `Attached` is an enum of the four in the assembly
  crate implementing the traits by `match`. The Genesis is compiled once; every access pays a discriminant test, the
  same as today's `attached` flag; an attached configuration pays the `match` and a direct call.
- **(c) `Option<Box<dyn Attachment>>`, checked once.** The cartridge pays the `is_some` test (today's cost). An
  attached configuration pays an indirect call on every access to its areas and every instruction (about 13,000 to
  45,000 a frame, which at 1 to 2 ns each is argued at 0.03 to 0.09 ms, 2 to 6% of the Sega CD's 1.513 ms), none of
  them inlined. And the trait is not object-safe as written: `run_to` is generic over the watch so that the unobserved
  run carries no hooks, and an object needs `&mut dyn Watch`, a test per sub-CPU instruction in every plain frame.

### 4.3 Recommended: (a), with (b) as its fallback

(a) gives the cartridge less than today and the attachments direct calls, and keeps `run_to` generic, so that the
observed and the plain run stay two compilations as in Beryl's `step_observed`. Its costs are compile time and size:
the Genesis, with the 68000 and Z80 interpreters inlined into its buses, compiled for four configurations, each plain
and observed. (c) is rejected for the attachments' hot path, and for the 32X's budget most of all.

The choice between (a) and (b) is **local to the assembly crate**: the Genesis is generic in both, and (b) is one type
in `nephrite` implementing the traits by `match`. So the build starts with (a), and (b) is measured beside it only if
(a)'s costs cross the bounds of Q1.

### 4.4 The bench that decides

The measurements of §54.1, before and after the commit that makes the Genesis generic (§7.2, commit 3), and again after
the crates are split (commit 6), since fat LTO across crates should give the same code and that is to be shown, not
assumed:

- **The cartridge Genesis**: `examples/bench` on Sonic the Hedgehog, Sonic the Hedgehog 2, Thunder Force IV, Streets of
  Rage 2 and Phantasy Star IV, best of 6 × 3 runs of 600 frames, drawn, the two builds alternated; and Micro Machines 2
  for the J-Cart's arm.
- **The Sega CD**: the same tool on the 240p disc with mcd-verificator as `NEPHRITE_BIOS`, best of 5 × 1,200 frames
  (1.513 ms on 2026-10-14).
- **The costs of (a)**: the size of `libnephrite.so` built with the `dist` profile, and the wall time of a clean
  `cargo build --release` of the library.
- Where a bench moves past the runs' spread, `ipsample.py` on both builds before anything is concluded
  (`dotnet-trace` cannot attribute this thread).

---

## 5. The crates

### 5.1 Layout and dependencies

Diamond set the precedent on 2026-10-13 (`Diamond_Enhancements.md` §24.1): the machine's crate stays at the core's
root as an `rlib`, the published library moves to `v1/`, its package renamed and its library keeping the core's name,
so that `RustCores.props` changes its `Dir` alone. Nephrite follows it, and adds one Cargo workspace (Q2):

```
Sega/Nephrite - Genesis/
  Cargo.toml       package nephrite-genesis (rlib), and [workspace] members mcd, 32x, v1; the profiles
  src/             the Genesis: the console, the cartridge and its boards (J-Cart, EEPROM, lock-on), I/O, the VDP,
                   the sound, the Genesis's debugger, the cartridge's header, and attach.rs (the interface)
  mcd/             package nephrite-mcd (rlib): the Sega CD
  32x/             package nephrite-32x (rlib): the 32X, a stub until stage 11
  v1/              package nephrite-v1, library "nephrite" (cdylib + rlib): the machine the ABI drives
  kit/             the conformance kit's samples, unchanged
```

```
               nephrite-v1  (lib "nephrite": libnephrite.so)
               │      │      │
               │      │      └──────────────► nephrite-32x ──► beryl-sh2
               │      └──► nephrite-mcd ──► emusen-disc            │
               │              │    └──► beryl-m68k                 │
               ▼              ▼                                    ▼
              nephrite-genesis ◄──────────────────────────────────┘
                 │  ├──► beryl-m68k, beryl-z80, beryl-sn76489
                 └─────► emusen-native  (every crate above uses it: state, steps, debug, core types)
```

The arrows point one way: **no crate depends on `nephrite-v1`**, and **`nephrite-genesis` depends on no `nephrite-*`
crate**, so the Genesis cannot name an attachment and an attachment cannot name another. Only `nephrite-v1` implements
`Core` and exports the ABI. `beryl-sh2` moves from the Genesis's dependencies, where it is unused (§2.7), to the 32X's.

The workspace's one lock and one target directory change nothing for the build that ships: `EmuSen.csproj` already
passes `--target-dir` per crate under `obj/` (`EmuSenCargoTargetRoot`), and Cargo reads profiles only at the workspace
root, where Nephrite's already are.

### 5.2 What moves where

| Today (`src/`) | To | Notes |
|---|---|---|
| `genesis.rs` | `nephrite-genesis` | generic over `A`; 136–139, 190–192, 379–381, 515–579 leave (the Sega CD's half into `nephrite-mcd`, the J-Cart's into the cartridge arm) |
| `cart.rs`, `eeprom.rs`, `io.rs`, `vdp.rs`, `render.rs`, `sound.rs`, `ym2612.rs`, `fm.rs`, `fm_tables.rs`, `lfo_tables.rs`, `fifo_records.rs` | `nephrite-genesis` | unchanged but for the J-Cart's arm and the line events' hook in the VDP loop |
| `debugger.rs` | `nephrite-genesis` (the Genesis's processors, peeks, watches); the composition into `nephrite-v1` | |
| `machine.rs` | split: `Console<A>` in `nephrite-genesis` (the frame with its pads, picture, sound, cartridge battery file and debugger state); `Machine` with `Fitted` and the space and battery composition in `nephrite-v1` | `MCD_MEMORIES`, `CARTRAM_ID` to `nephrite-mcd`; `S32X_MEMORIES` to `nephrite-32x` |
| `media.rs` | split: the cartridge's header, markets, save RAM, copier images in `nephrite-genesis`; `System` and the decision by contents in `nephrite-v1` | `DISC_SIGNATURE`, `disc_offset` to `nephrite-mcd`; `is_32x` to `nephrite-32x` |
| `state.rs` | `nephrite-v1` (the header, the versions, the order, `StateError`) | the blocks are their owners' |
| `v1.rs` | `nephrite-v1` | the Sega CD's descriptors, setting and disc helpers to `nephrite-mcd`, the 32X's to `nephrite-32x` |
| `mcd/` | `mcd/src/` | `mod.rs` becomes `lib.rs`; its tests build `Console<MegaCd>` directly, `verificator.rs` and `discs.rs` included |
| tests that run a machine (`programs.rs`, `pictures.rs`, `voices.rs`, `sounds.rs`, the `board_*` files) | `nephrite-genesis`, on `Console<Bare>` | `Machine::new(&image, Media::read(&image))` becomes the cartridge constructor of `Console<Bare>`; mechanical |
| `examples/` | by what they run: `bench`, `dump`, `statediff`, `regs`, `wav`, `fmtrace` to `v1/`; `chip`, `dis`, `fifo` to the root; `disc` to `mcd/` | `bench` stays on the published path, so that it measures `Fitted`'s match too |

### 5.3 The Genesis-only build

Two forms, both kept:

1. **The Genesis's crate alone**: `cargo test -p nephrite-genesis` builds and runs every Genesis test with no
   attachment in the graph. This is the compiler's proof of the boundary and costs nothing to keep.
2. **A Genesis-only library**: `nephrite-v1`'s features `mcd` and `s32x`, both default; with
   `--no-default-features` the library reports the `md` system alone, `Fitted` has its `Md` arm alone, and nothing of
   `nephrite-mcd`, `nephrite-32x` or `emusen-disc` is linked. *Recommended (Q4):* a development build only, run with
   the kit's Genesis sample once per change to the interface, never shipped, since a player's library holds the three
   systems.

A separate assembly crate for the Genesis alone was weighed and rejected: it would repeat the systems table, the
settings and `Core`'s implementation, two owners for each.

### 5.4 One owner for each fact

| Fact | Owner | Read by |
|---|---|---|
| The pad's button bits (the order the I/O reads them) | `nephrite-genesis` (`io`) | the controller descriptors in `nephrite-v1` (`BUTTONS`) |
| The master clock's rates, the frame rates, `START` | `nephrite-genesis` | attachments through `Board`; machine info |
| The output sample rate (48 kHz) and the output's scale | `nephrite-genesis` (`sound`) | attachments through `Board`; `AUDIO_RATE` |
| System ids (`md`, `mcd`, `32x`), extensions, the state's system byte | `nephrite-v1` | — |
| The state's magic, versions, oldest version per system | `nephrite-v1` | blocks are given the version |
| ABI file numbers (firmware 1–9, batteries 0 and 10) | `nephrite-v1` | — |
| The ranges of space and processor ids | `nephrite-genesis` (`attach`) | every crate |
| Each space's and processor's id within a range | that range's owner | — |
| The disc's signature, opening a disc, its fingerprint, the drive's number | `nephrite-mcd` | `nephrite-v1` (recognition, create, insert) |
| The 32X's header test | `nephrite-32x` | `nephrite-v1` |
| The J-Cart's boards and word | `nephrite-genesis` (`cart`, `io`) | — |
| The open bus's rule (D-29) | `nephrite-genesis` | attachments through `Answer::Open` |
| An attachment's mix gain against the Genesis's output | that attachment | — |

### 5.5 What the compiler enforces, and what it does not

The dependency arrows are enforced: `nephrite-genesis` cannot be made to name `MegaCd`. Fields are another matter.
Today almost every field of `Hw` is `pub`, because `machine.rs`, `v1.rs` and the tests reach them. After the split the
assembly crate still reads the Genesis's devices directly (the VDP's frame, I/O's pads, the cartridge's battery), and
an attachment could reach `Slot::cart`'s public fields. *Recommended (Q6):* the move enforces the direction; the
fields are narrowed afterwards, crate by crate, to what the next crate up uses, each narrowing its own commit with no
effect at run time.

---

## 6. Forms weighed

The design rules ask which form a feature takes before it is built: an extension of a surface, a module or plugin, or
a component of its own.

- **Chosen: an internal surface inside one library.** The interface is a module of `nephrite-genesis`, of the kind
  `EmuSen_Surfaces.md` §6.1 calls rule **R**: an internal Rust API, a path dependency built from the same commit,
  promising nothing across commits. It is versioned by its crate: `nephrite-genesis`'s `version` moves (0.x minor) with
  any change to the three traits or their types, so that a change to the surface is a reviewed line in a diff, as the
  Beryl crates' contracts are kept (`Beryl-HW/README.md`). No outer surface changes: the core ABI v1 stays the only one
  a host sees, the library's file, core id, system pack and sidecar stay as they are, and no new dependency between
  components is added (every crate here is Nephrite's own, over `emusen-native`, `emusen-disc` and Beryl as today).
- **Rejected: separate libraries per machine** (`libnephrite-md`, `-mcd`, `-32x`). Each would contain the Genesis,
  which the plan's §1.1 already gives as the reason the Sega CD is not a core of its own; three registrations, three
  sidecars and three system packs for one console; a CD32X library containing two attachments and a Genesis; and the
  state, the battery files and the debugger's numbering split three ways for one machine. It is "a component of its
  own" three times over, for machines that cannot stand alone.
- **Rejected: a runtime plugin** (attachments loaded by Nephrite as dynamic libraries). Rust has no stable ABI, so the
  boundary would be a second C ABI, at the hottest boundary in the core: a call per 68000 instruction and per access,
  through a pointer, never inlined. The attachments share Rust types with the Genesis (`Cart`, `Steps`, `StateWriter`,
  `Hooks`), which a C boundary would have to copy or serialise; the framework already has one plugin boundary for
  cores, the core ABI, and nothing supplies attachments from outside the project.

**Is the interface ever published?** *No.* Its only implementers are Nephrite's crates, and its types are the
Genesis's internals (the cartridge, the master clock's units, the VDP's lines), which no other core shares. A console
with an attachment of its own (the PC Engine's CD-ROM², the Saturn's cartridge port) would copy the pattern, a bus side
asked first with a lent slot, not the trait: its buses and its clock differ. Publishing it would turn rule R into rule
A or C for no client.

---

## 7. The migration

### 7.1 When, and the baseline

After stage 9 merges into WiseMan, and before stage 10's first step (Q3): stage 9 edits exactly the files this move
splits, and stage 10's mode 1 is the first configuration the move makes easy. The steps run as Nephrite's others, about
three hours each with a check-in after each (A5).

Every commit keeps every test green and the cartridge Genesis byte-identical, and the Sega CD too. Byte identity is
checked four ways:

1. **The program tests** of the crate: the BCD verifier, the opcode sizes, the illegal instructions, VDPFIFOTesting,
   the board's FM and bus tests, the pictures, mcd-verificator's eighteen.
2. **The pinned layouts**: `the_layout_is_pinned` for a cartridge, and the same for a disc's machine, added in commit 1
   before anything moves, if stage 9 has not added it.
3. **The kit**: the Genesis sample, 64 of 64, and the 240p disc with mcd-verificator as its boot ROM; WiseMan's
   Nephrite tests as `Nephrite_Native.md` §53.5 lists them.
4. **Digests**: a tool written in commit 1 (`examples/digest.rs`) records, for each image of the kit's sample and for
   the 240p disc, FNV-1a over the state, the picture and the audio at frames 300, 600 and 1,200; recorded once at the
   baseline under `~/.cache/emusen/probe/nephrite/attach/` and compared at every commit.

### 7.2 The commits

| # | What | Checks | Measured |
|---|---|---|---|
| 1 | The baseline: `examples/digest.rs`; a pinned layout for a disc's machine; the before numbers of §4.4 recorded | all of §7.1 | the bench, the Sega CD, size, build time |
| 2 | The J-Cart into the cartridge's arm; `attached` guards the Sega CD alone | J-Cart tests; §7.1 | the bench, Micro Machines 2 |
| 3 | `attach.rs` in the crate; `Genesis<A>`, `Hw<A>`, `Bare`; `MegaCd` on the three traits (`genesis.rs` 518–579 and 379 moved in); `Console<A>` split from `machine.rs`; `Fitted` with `Md` and `Mcd`; state and spaces composed through the traits | §7.1 | **the decision of §4**: the bench, the Sega CD, size, build time |
| 4 | The Sega CD's facts behind its own module: firmware descriptors, `ramcart`, batteries, drives, the disc helpers and `DISC_SIGNATURE`, the fingerprint into `MegaCd`; `cdda.rs` given the rate by `Board`. Afterwards no `mcd` outside `src/mcd/` but `Fitted`'s arms | §7.1; a `grep` recorded | — |
| 5 | The 32X's stub: `S32x` on the traits, answering `Pass`, with its three placeholder memories and descriptors; `Fitted::S32x`, still counting frames only (`machine.rs` 225) | §7.1; a 32X state byte-identical | — |
| 6 | The split: the root package renamed `nephrite-genesis` (rlib) with the workspace; `mcd/`, `32x/`, `v1/`; `RustCores.props`'s `Dir` to `Sega\Nephrite - Genesis\v1\`; examples and tests sorted (§5.2) | §7.1; the kit on the library built by `EmuSen.csproj`; `cargo tree -p nephrite-genesis` naming no `nephrite-*` crate, as a test | the bench and the Sega CD again (LTO across crates) |
| 7 | The Genesis-only build: `nephrite-v1`'s features; the kit's Genesis sample on `--no-default-features` | the sample, 64 of 64; the digests | the size of the Genesis-only library, for the record |
| 8 | The record: a section of `Nephrite_Native.md`; this page's predictions retired; the folder's README | — | — |

Commits 3 and 6 are the two where the cost can move; each is measured before the next begins, and a loss stops the
migration until it is explained (§4.3's fallback is commit 3's alternative).

### 7.3 The state format

**No change, and no new version.** The move keeps every state byte-identical, at version 19 or whatever version stage 9
leaves, argued from the write order (§3.5): the header, the spaces in id order, the Genesis's block and the Sega CD's
block are written in today's order by today's code, moved; the one field that changes owner, the disc's fingerprint,
is written by its new owner at the same offset. A 32X machine keeps writing its three placeholder memories through the
stub (commit 5). The pinned layouts and the digests show it at every commit (A4).

What the format will need, and the move does not add: a configuration that the system byte does not decide. Today a
system byte decides the blocks (`md` none, `mcd` the Sega CD's, `32x` none). Mode 1 (a cartridge, `md`, with the Sega
CD's block) and CD32X (a disc with the 32X's block) break that. *Recommended (Q5):* a byte of the attachments present,
added to the header with the first stage that builds such a configuration, at that stage's version, and not before.

### 7.4 Risks

- **Stage 9's merge.** The move rewrites the files stage 9 edits; starting before stage 9 merges would make every one
  of its commits a conflict. Hence §7.1's order, and the move finished before stage 10 begins.
- **Cost from crossing crates.** Fat LTO with one codegen unit should give the same code across crates as within one,
  but inlining is decided again; small cross-crate functions need `#[inline]` for the unoptimised test builds' speed.
  Measured at commit 6.
- **Monomorphisation's size and build time** (§4.3), bounded by Q1, with (b) as a local fallback.
- **Byte identity's subtle orders**: the VDP, attachment, Z80 order in `catch_up`; `run_to` before a register access;
  the open bus supplied by the Genesis; the fingerprint's offset. Each is a line of §3.4 and is held by the digests.
- **Visibility.** Tests and tools that reach fields across what become crate boundaries (`examples/disc.rs` reads the
  Sega CD's registers; stage 9's `Sub::seen`) move with their subject (§5.2); a field the assembly genuinely needs
  stays `pub` until Q6's narrowing.
- **The lock.** One workspace lock in place of one: path dependencies alone, and `chd` through `emusen-disc`, are
  resolved again; the old lock is copied in, so that no version moves.
- **CD32X's overlapping decode**, unknown until stage 12 (§3.10); contained by the assembly's pair type.

---

## 8. Questions for the tester

1. **Q1, the dispatch.** *Recommended:* one instantiation per configuration (§4.3, (a)), with the enum (b) measured
   beside it only if (a) grows the shipped library by more than half or a clean release build by more than half; the
   cartridge Genesis's frame cost may not rise beyond the runs' spread in either case.
2. **Q2, the layout.** *Recommended:* the root stays the Genesis, renamed `nephrite-genesis`; `mcd/`, `32x/` and `v1/`
   beside `src/`, in one Cargo workspace with one lock. The alternatives were Diamond's separate locks per crate (no
   gain here, the crates being built together always) and moving the Genesis into `genesis/` (every file of the
   Genesis renamed in git, for symmetry alone).
3. **Q3, when.** *Recommended:* the eight commits of §7.2 right after stage 9 merges, before stage 10's first step.
4. **Q4, the Genesis-only library.** *Recommended:* a cargo feature on `nephrite-v1`, built for development and run
   with the kit's Genesis sample at each change to the interface, never shipped.
5. **Q5, a configuration in the state's header.** *Recommended:* a byte of the attachments present, added by the first
   stage that builds mode 1 or CD32X, at that stage's version; the move adds nothing.
6. **Q6, field visibility.** *Recommended:* the move enforces the dependency direction; `pub` fields are narrowed
   afterwards, crate by crate, each its own commit.
7. **Q7, processor ids.** *Recommended:* `S68K` 2, `MSH2` 3, `SSH2` 4, fixed whatever the configuration, so that an id
   never moves between systems as the plan's space ids do not.
8. **Q8, battery files in mode 1** (stage 10's, raised here because the numbering becomes the assembly's). *Recommended:*
   on a cartridge with the Sega CD beneath it, the cartridge's save RAM stays file 0 (`.srm`) and the backup RAM takes
   file 11 (`.brm`); on a disc, the backup RAM stays file 0, as now.

---

## 9. Predictions to be retired

- **A1.** Under (a), the cartridge Genesis's mean frame cost on the five bench games is equal to the baseline within
  the runs' spread, or lower.
- **A2.** Under (a), the Sega CD's cost on mcd-verificator's run is equal to the baseline within the runs' spread.
- **A3.** Under (a), the shipped library grows by less than half and a clean release build takes less than half as long
  again.
- **A4.** The move keeps every state byte-identical: no new state version, the pinned layouts unchanged.
- **A5.** The migration takes two steps: commits 1–4 in the first, 5–8 in the second.

## 10. Reading log

Read for this page: `Nephrite_Plan.md` (§1.1, §1.2, §4, §5.1, §5.4–§5.8, §6, §8, §9); `Nephrite_Native.md` §48 (the
J-Cart), §49–§54 (the Sega CD's split and stage 8), and the stage 9 branch's §55 (`nephrite-s9` at `77c8ba7f`, read
from git, not merged here); `Beryl-HW/README.md`; `EmuSen_Surfaces.md` §2.2, §4.6 and §6; `EmuSen_CoreAPI.md`'s section
list (§2.4, §6.23–§6.26, §27.4); `Diamond_Enhancements.md` §24.1. Code, at `d32a00b3`: Nephrite's `src/` whole for the
count, and in full `genesis.rs`, `machine.rs`, `state.rs`, `v1.rs` (to its tests), `debugger.rs` (its non-test half),
`mcd/mod.rs`, `mcd/state.rs` (its writer), `mcd/cdda.rs`, the J-Cart's parts of `cart.rs` and `io.rs`, `sound.rs`'s
`take_with`, `examples/bench.rs`; `RustCores.props` and `EmuSen.csproj`'s cargo targets; Diamond's two manifests; the
stage 9 branch's diff of `genesis.rs` and `mcd/mod.rs`. Not read: any emulator's source, any RTL, any firmware.
