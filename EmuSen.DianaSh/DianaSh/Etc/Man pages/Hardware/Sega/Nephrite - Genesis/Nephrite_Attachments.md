# Nephrite_Attachments — the Sega CD and the 32X behind a written interface

*Written 2026-10-15, before any code.* This page designs the surface between the Genesis and its attachments, so that
the Sega CD and the 32X stand behind a defined surface, as module trees of the one `nephrite` crate, while Nephrite
stays one library for three machines. It changes no code; the build is a separate step, taken after stage 9 merges, from §7 of this page.

It keeps the plan's three registers apart (`Nephrite_Plan.md`, opening):

- **Measured** means counted on 2026-10-15 in the tree at WiseMan `d32a00b3`, recounted at `fa1e4242` (§2), or measured
  in the build (§11), with the command named.
- **Argued** means reasoning from the code, the documents or other cores' records, with no measurement behind it.
- **Predicted** means a claim numbered A1–A5 (§9), kept so that the build can retire it.

---

## 1. The decision, its aims and what this page does not do

*Decided 2026-10-04 (`Nephrite_Plan.md` §1.1), and standing:* Nephrite is one library for three machines. The Sega CD
and the 32X are attachments of the Genesis, neither runs without one beneath it, and three system ids are reported.

*Decided 2026-10-15 (the tester's go):* each attachment stands behind a written attachment interface, in place of a
module folder that the Genesis side reaches into.

*Decided 2026-10-15: the interface, within the one `nephrite` crate; separate crates not now.* The page's first version
(the same day) designed four crates in a workspace; that layout is replaced by the module layout of §5, and the crate
split is recorded as a possible later step (§5.6). The Beryl crates (`beryl-m68k`, `beryl-z80`,
`beryl-sh2`, `beryl-sn76489`) and `emusen-disc` stay as they are: shared hardware and shared discs.

The aims, in order:

1. **The boundary held mechanically.** An attachment's internals are private to its module tree and the Genesis's to
   its own; an attachment reaches the Genesis only through the interface module; what privacy inside one crate cannot
   hold is held by two checks that fail a test run (§5.3).
2. **A Genesis-only build**, which compiles and tests with no attachment present, as a cargo feature (§5.4).
3. **The 32X begins on the interface** (stages 11–12): its SH-2s, system registers, frame buffers, overlay and PWM
   arrive as an implementation of the same traits the Sega CD implements, not as a second set of special cases in
   `genesis.rs`.
4. **The cartridge Genesis unchanged**: byte-identical in picture, sound and state, and no slower.

What the page does not do: it builds nothing, times nothing and reads no code outside Nephrite's own tree and the
pages it cites. No emulator's source, no RTL and no firmware was read to write it (§10).

---

## 2. The measured coupling

### 2.1 Method

Counted first in `EmuSen/Cores/Sega/Nephrite - Genesis/src/` at `d32a00b3`, and **recounted at `fa1e4242`**, after
stage 9 (`c8b10080`) and the save media (`Nephrite_Native.md` §61) merged, which is the tree the migration of §7 starts
from. The recount uses `grep -n` over the identifiers and gates that belong to the Sega CD (`mcd`, `MegaCd`, `RamCart`,
`CARTRAM`, `MCD_`, `RAM_CART`, `disc_print`, `System::Mcd`, `cdd::`, `open_disc`, `system_area`, `fingerprint`,
`cd_bios`, `CD_BIOS`, `.brm`, `.crm`, `STATUS_BAD_DISC`, `MEDIA_FILE`, and since the first count `BRAM`, `kept_apart`,
`KeptApart`, `backup_ram`, `ram_cartridge`, `attached`, `DISC_SIGNATURE`, `disc_offset`, `is_disc`, `CD_COST`,
`sound_to`, `cdda`), the J-Cart's (`jcart`, `JCART`) and the 32X's (`S32x`, `S32X`, `s32x`, `is_32x`); the script is
`~/.cache/emusen/probe/nephrite/interface/count.sh`. Each match was read and classified by hand.

A **site** is one line of production code that names an attachment's item, reads or writes its state, or decides by
whether an attachment is present. Lines inside a file's `#[cfg(test)]` module, and the test-only files, are counted
apart. Where one statement spans several lines, the line that reaches across is cited once; a declaration of several
constants is a site a line, which the first count did not always do (it cited `CD_BIOS_U..J` once).

### 2.2 The figures, recounted at `fa1e4242`

**131 production sites** on the Genesis side reach the Sega CD, where the first count found 96. By kind, the 35 more
are battery files and save media +13, media and discs +8, firmware +5, state +3, spaces +2, and one each in the map,
reset and power, settings and machine info. Most are the save media (`KeptApart`, `BRAM_ID`, the kinds of the battery
declarations and the flags at create, and the state's rule for a memory kept apart) and stage 9's transfer from Word RAM
and version bit (§2.8); the rest is the finer counting of §2.1 and lines the first count did not cite. Of the 131, 27
are in `genesis.rs`, 28 in `machine.rs`, 59 in `v1.rs`, 11 in `state.rs`, 5 in `media.rs` and 1 in `lib.rs`: the
console's own file holds a fifth of the reach, the assembly the rest.

The reverse is unchanged in kind: **two production lines** of `src/mcd/` reach outside it, `cdda.rs:55`
(`crate::sound::RATE`) and `mod.rs:319` (`crate::media::DISC_SIGNATURE`), and nine test lines do, in the two files that
run a whole machine (`verificator.rs` 5, 6, 13, 14, 21, 90, 91; `discs.rs` 53, 54). *The first count's figures*, kept for
the record: 96 sites, 48 lines naming `mcd`, and the reverse's 2 production and 10 test lines.

So the Sega CD is still far more self-contained than the Genesis is towards it, and most of the move's work is in the
assembly (`v1.rs`, `machine.rs`, `state.rs`), not in the console.

### 2.3 The Genesis side's reach into the Sega CD, by kind

| Kind | Sites | Where (file:line at `fa1e4242`) |
|---|---|---|
| Construction and ownership | 9 | `genesis.rs` 136 (`Hw::mcd`), 137 (`mcd_low`), 150, 159 (`with_mcd`), 190, 192 (`MegaCd::new` given `master_hz` and `START`); `machine.rs` 112, 130; `lib.rs` 23 |
| Map | 19 | `genesis.rs` 139 and 191 (the `attached` flag, shared with the J-Cart); the four guards 460, 480, 500, 521; the Sega CD's decode 545, 552, 564, 565; `mcd_read8` 574, 575, `mcd_read16` 582, 583, `mcd_write` 590, 591 (the RAM cartridge reached through `m.hw.cart`, the rest through `main_read*`/`main_write`); stage 9's transfer from Word RAM, 379, 380, 389 (§2.8) |
| Interrupts | 0 | the 68000's level is the VDP's alone (`genesis.rs` 803–807); the Sega CD's H-INT vector and IFL2 travel through the map (§2.4) |
| Clock and slices | 1 | `genesis.rs` 401: `m.run_to(self.clock)` in `catch_up`, after each 68000 instruction |
| Reset and power | 1 | `genesis.rs` 197: the version register's bit 5 cleared with an expansion unit (§2.8). Nephrite claims no `RESET` |
| Audio | 3 | `machine.rs` 265, 267 (`sound_to`), 268 (`take_with` lent the Sega CD's `Steps`) |
| Video | 0 | the Sega CD has no picture of its own; the VDP's transfers from Word RAM pass through the map (§2.5) |
| Pads | 0 | — |
| State | 11 | `state.rs` 17 (`OLDEST_MCD_STATE_VERSION`), 29 (`WrongMedia`), 54 (the system byte), 69, 70 (the block written), 76 (the oldest version by system), 105 (read), 109, 110 (the fingerprint held), 119 (applied); `machine.rs` 68 (`disc_print`) |
| Media and discs | 29 | `machine.rs` 107 (`open_disc`), 113 (`fingerprint`), 115 (no ROM for a disc); `media.rs` 144, 146 (`DISC_SIGNATURE`, `disc_offset`), 202; `v1.rs` 184 (`MEDIA_FILE`), 186 (`STATUS_BAD_DISC`), 213 (`is_disc`), 218, 219, 224, 226, 228, 229, 231 (`open_disc`, `system_area`), 355, 367, 368 (create's dispatch), 374, 585, 586 (`drives`), 593 (`media_lid`), 599, 606, 608, 609 (`media_insert`), 620, 624 (`create_disc`) |
| Firmware | 15 | `machine.rs` 126; `v1.rs` 172, 173, 174 (`CD_BIOS_U..J`), 245 (`CD_COST`), 248 (`cd_bios`), 335, 342 (`firmware_for`), 382, 383, 384 (create's choice of boot ROM), 427 (machine info), 627, 628, 629 (`create_disc`) |
| Battery files and save media | 21 | `machine.rs` 71, 76, 81, 84 (`KeptApart`), 208 (file 0 is `BRAM` on a disc); `media.rs` 228; `state.rs` 114 (kept apart on a load); `v1.rs` 182 (`RAM_CART`), 363 (`battery_files`), 579 (`battery`), 625, 631 (the flags at create), 640, 641 (loaded at create), 654, 656, 658 (`batteries`), 668, 669, 673, 674 (`backup_ram`, `ram_cartridge`) |
| Debugger and spaces | 13 | `machine.rs` 34 (`CARTRAM_ID`), 36 (`BRAM_ID`), 37 (`MCD_MEMORIES`), 124, 144 (no `ROM` on a disc), 147, 148, 150 (`spaces`), 166, 168, 176 (`bytes`), 192, 200 (`bytes_mut`) |
| Settings | 5 | `v1.rs` 189 (`RAM_CART_KEY`), 191 (`ram_cart_setting`), 348 (the schema), 635, 636 (applied at create) |
| Machine info | 4 | `media.rs` 17 (the id); `v1.rs` 308 (`MCD_EXTENSIONS`), 322 (the description), 325 (the `mcd` row, marked in development) |
| **Total** | **131** | |

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

**The J-Cart**, 26 sites at `fa1e4242`, all inside what will be the Genesis's module tree: `cart.rs` 23, 39, 42, 44, 61,
62 (the board known by its header); `io.rs` 150, 154, 199, 201, 206, 210, 211, 218, 219, 245 (players 3 and 4, the word,
TH); `genesis.rs` 160, 191, 195 (the flag copied and the `attached` check turned on), 541, 542, 549, 550, 558, 560 (the
two words decoded ahead of the cartridge); `v1.rs` 151 (machine info's players). The first count's 17 cited the same
places with a function's lines once.

**The 32X**, 17 sites, none of them hardware, since none is built: `machine.rs` 38 (`S32X_MEMORIES`), 123 (its three
placeholder memories made), 244 (`runs`: a 32X image counts frames only); `media.rs` 10, 18, 137 (`is_32x`), 205;
`state.rs` 55 (its system byte); `v1.rs` 175, 176, 177 (`S32X_68K..SLAVE`), 246 (`S32X_COST`), 262 (`s32x_boot`), 309,
326, 343, 428. The crate still depends on `beryl-sh2` and uses nothing of it.

### 2.8 What stage 9 and the save media added, as merged

The page's first version read stage 9 from its branch at its first step; at `fa1e4242` it is merged whole, with the
save media after it. On the Genesis side they added:

- **The version register's bit 5** (`genesis.rs` 196–199), cleared when a Sega CD is attached ("expansion unit
  present", `Nephrite_Native.md` §55.3): a reset-and-power fact, the interface's `EXPANSION` (§3.3).
- **A transfer from Word RAM a word late** (`genesis.rs` 376–394, §57 there): the VDP's transfer from Word RAM reads
  the word before the one addressed, where the 68000 reads it in time. This is the case §3.3 named in advance, a
  transfer from Word RAM behaving apart from a 68000 read, so the requester is added (§3.3, as amended).
- **The save media** (`Nephrite_Native.md` §61): `KeptApart` and `BRAM_ID` in `machine.rs`, the battery declarations of
  kind `console` and `removable` in `v1.rs`, and `load_state` skipping a memory kept apart. Facts for the frontend
  (§3.6), and one rule of the state (§3.5).
- **`MEDIA` and `MEDIA_SEEK` claimed** (§56 there): the drive's calls were already in place; nothing new crosses.

Inside `src/mcd/`, stage 9 added the graphics ASIC, the RF5C164, the fader's ramp and de-emphasis and the mix's levels,
the register watch `Sub::seen` and the drive's rules. Nothing of them reaches the Genesis's picture or its 68000's
interrupts, as §3.8 argued; the PCM sums into the Sega CD's one stream with CD-DA (`Sub::sound_to`, `Cdda::feed`).

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
   Sega CD keeps today's rule (it runs to every `to`, instruction by instruction); the 32X keeps the plan's §5.4 (to every
   access to its areas and every line, and otherwise in slices of a measured length, P5's kind of parameter, a
   setting with `effect: accuracy` only if the plan's §5.5 budget is missed). The bound lives in the attachment's module, so a
   change to it never touches the Genesis.

### 3.3 The bus side: `Attachment`

In the module `interface` (§5), the only one an attachment imports from the crate:

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

**As built (commit 3, §11.3).** `src/interface.rs` carries the bus side with the members the Sega CD uses, by the
rule just given, and one the design did not have:

```rust
pub trait Attachment {
    const EXPANSION: bool = false;
    fn read8(&mut self, a: u32, at: Master) -> Answer<u8>;
    fn read16(&mut self, a: u32, at: Master) -> Answer<u16>;
    /// A word the VDP's transfer reads from the 68000's bus, which an attachment may answer apart from the 68000's.
    fn transfer16(&mut self, a: u32, at: Master) -> Answer<u16> { self.read16(a, at) }
    fn write(&mut self, a: u32, v: u16, mask: u16, at: Master) -> bool;
    fn run_to(&mut self, to: Master);
    fn sound(&mut self, to: Master) -> Option<&mut Steps>;
}
```

- **`transfer16` is the requester** the paragraph above said would be added if a transfer from Word RAM behaved apart
  from a 68000 read. Stage 9 found that it does (§2.8): Word RAM answers the VDP a cycle late. The rule moved from the
  Genesis's `attached_dma_read` into the Sega CD's `transfer16`; the default is the 68000's read, so `Bare` and any
  attachment without such a rule pay nothing.
- **Left out until their first user**: `Slot` (the cartridge lent and the wait, the 32X's, stage 11), `Line`, `LINES`
  and `line` (the 32X's), `reset` (when Nephrite claims `RESET`), `peek` and the `Watch` given to `run_to` (the Sega
  CD's debugger step, which brings its processor). Each is added by the commit that first needs it, with the Sega CD
  brought along.
- **`sound` returns the one stream** rather than lending each stream to a callback, since the Sega CD has one and the
  mix's `take_with` takes one; CD32X's two streams (§3.10) change it when stage 12 builds them.

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

**As built (commit 3, §11.3)**, the parameter is on the Genesis and on what its buses reach, not on its devices:
`Genesis<A = Bare> { cpu, z80, hw: Wired<A> }`, where `Wired<A> { dev: Hw, att: A }` holds the Genesis's devices,
`Hw` as before and not generic, with the attachment beside them, and reads as `Hw` through `Deref`, a field projection
that costs nothing. `MainBus<'_, A>` and `Z80Bus<'_, A>` each hold `&mut Wired<A>`, one pointer, as they held `&mut Hw`;
the map's methods are `Wired<A>`'s, the devices' own (the refreshes, the HV counter's point, TMSS, the Z80's window's
address) stay `Hw`'s. `Hw::mcd`, `mcd_low` and `attached` are gone. The machine holds `enum Fitted { Md(Genesis<Bare>),
Mcd(Genesis<MegaCd>) }`, matched once per call by a macro, `fitted!`, that binds the Genesis with its attachment's type
known in each arm; the assembly and the tests reach the devices through `Machine::hw()`/`hw_mut()` and the processors
through `cpu()`/`z80()`, and the Sega CD through `mcd()`/`mcd_mut()`.

*A form tried and rejected, measured:* the attachment lent beside a borrowed `Hw`, `Map<'_, A> { hw: &mut Hw, att: &mut A
}`, which would have kept the machine's Genesis one type and every test's `m.genesis.hw` as it was. Each access then
reached the devices through a second pointer, and the cartridge Genesis cost 1.3 to 2.4% more on Phantasy Star IV and
Sonic the Hedgehog. With the Sega CD's instantiation removed as well the loss stood at 1.3 to 1.4%, so it was the form's,
not the second instantiation's. `Wired<A>`, one pointer, removed it (§11.3).

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
header (magic, version, system byte, frames) written by the assembly (§5.1); every writable space in id order, the
Genesis's (2–7) then the attachment's (from 16 for the Sega CD, from 32 for the 32X); the Genesis's block
(`Genesis::write_state`); then each attachment's block, the Sega CD's before the 32X's. This is today's order exactly
(`state.rs` 59–71): the spaces listing already puts the Sega CD's after the Genesis's, and its block follows the
Genesis's. The disc's fingerprint, today `Machine::disc_print` passed in (`state.rs` 69), becomes the Sega CD's own
field and is written as the block's last eight bytes, where it stands now (`mcd/state.rs` 102). The version is passed
to every reader; the sequence of versions and each system's oldest are the assembly's (§5.5), the branches on a
version inside a block its owner's, as `genesis.rs` 1079 and 1092 are today.

**How spaces and processors compose.** Each owner has a fixed range of ids, so ids never move between systems and a
configuration's lists are concatenations:

| Owner | Space ids | Spaces | Processor ids | Processors |
|---|---|---|---|---|
| Genesis | 0–15 | `M68KBUS` 0, `Z80BUS` 1, `WRAM` 2 … `ROM` 8 | 0–1 | `M68K` 0, `Z80` 1 |
| Sega CD | 16–31 | `S68KBUS` 16, `PRGRAM` 17, `WORDRAM` 18, `PCMRAM` 19, `BRAM` 20, `CARTRAM` 21 | 2 | `S68K` 2 |
| 32X | 32–47 | `MSH2BUS` 32, `SSH2BUS` 33, `SDRAM` 34, `FRAMEBUFFER` 35, `PALETTE` 36 | 3–4 | `MSH2` 3, `SSH2` 4 |

The space ids are the plan's (§4.3); the processor ids are proposed here (Q7). The ranges are the interface's, in
`interface`; each id within a range is its owner's constant. The debugger's `Hooks` are made with a width for each id up
to the highest the configuration lists (`machine.rs` 111 makes `[24, 16]` today). The Genesis's `peek_m68k` asks
`att.peek` first, which closes §2.6's first gap; in the move itself the Sega CD's `peek` answers `None` and its
processor list is empty, as today, so that the move changes nothing a debugger sees, and its processor arrives with the
Sega CD's debugger step.

**As built (commit 3, §11.3).** `AttachmentState` as designed, `write_state(&self, w)` with the fingerprint the Sega
CD's own field (`MegaCd::disc_print`, set at construction and by an insert), moved in this commit rather than the next
because the trait's writer takes no argument for it. The assembly composes the blocks in `state.rs` with one enum,
`FittedSaved`, holding the Genesis's block and the attachment's as read, so that a load still reads everything, checks
the fingerprint and only then applies the spaces, the Genesis and the attachment, in today's order. `AttachmentView` has
`spaces`, `bytes` and `bytes_mut`; the processors' members arrive with the Sega CD's debugger step, and `SpaceRef` and
the id ranges (`GENESIS_SPACES`, `MCD_SPACES`, `S32X_SPACES`) are `interface`'s.

### 3.6 Facts for the frontend: files, media, settings, machine info

Batteries, drives and discs, firmware, settings and the system rows are answered once per call between frames and are
read by one caller, the assembly, which knows each configuration's concrete types. They are **plain functions and
inherent methods of each attachment's module**, composed by the assembly, not trait members:

- **Firmware**: each attachment describes its images by name, label, size and its replacement's words (`mcd`:
  `bios_CD_U/E/J.bin` and `CD_COST`; `s32x`: the three boot ROMs and `S32X_COST`; the Genesis: TMSS). The ABI's
  file numbers (1–10 today) are the assembly's single table.
- **Battery files**: each attachment names its files by suffix and space (`.brm` on `BRAM`, `.crm` on `CARTRAM`) and loads
  them; the assembly numbers them (0 and 10 today) and answers `battery`.
- **Media**: `drives`, `media_lid` and `media_insert` call `MegaCd`'s inherent methods; opening a disc
  (`open_disc`, `is_disc`, `system_area`, `fingerprint`, `DISC_SIGNATURE`) is `mcd`'s, so the assembly no
  longer calls `emusen-disc` directly. Whether the library claims `MEDIA` (allowed for a machine with no drive
  since the core ABI's 1.6, `EmuSen_CoreAPI.md` §6.26) is the assembly's decision.
- **Settings**: each attachment lists its settings (`ramcart` in `mcd`, a slice bound in `s32x` if one is
  ever needed) and applies them at create; the assembly concatenates the schema.
- **Machine info**: the system id, extensions, the development mark, frame rate, ports and the description are the
  assembly's; spaces, processors, batteries and firmware are composed from the parts.

**As built (commit 4, §11.4)** in `src/mcd/frontend.rs`: the disc's signature and offset, `is_disc`, `open_disc`,
`fingerprint` and `system_area`; `bios_firmware(region, which)`; `ram_cart_setting` and `MegaCd::apply_settings`;
`SaveMemory { Backup, Cartridge }` with `battery(memory, which)` and `MegaCd`'s `save_memories`, `save_memory` and
`load_save_memory`; and the drive's `drive`, `lid_open`, `set_lid` and `insert`. Every number is the assembly's: files
2–4, the battery files 0 and 10 (`v1::save_file`), the companions from `MEDIA_FILE`, `STATUS_BAD_DISC`, and the host's
companions kept or borrowed (`v1::open_disc`).

### 3.7 Each kind, answered

| Kind | Through | Sega CD | 32X | CD32X |
|---|---|---|---|---|
| Map | `read8`, `read16`, `write`, `peek`; `Answer`; `Slot` | mode 2's `$000000`–`$3FFFFF`, the slot at `$400000` (RAM cartridge, or `$FF`, D-32), `$A12000`–`$A1203F`; mode 1 at `$400000` (stage 10) | its registers, its windows and the vector override; the cartridge beneath through `Slot::cart`; waits through `Slot::wait` | `Both`: the Sega CD asked first |
| Interrupts | none towards the 68000; into an attachment by the map and by lines | IFL2 and the H-INT vector by the map; the sub CPU's levels inside | V and H from lines, CMD by the map, PWM inside | each its own |
| Clock and slices | `run_to`, `Board`, `Slot::wait` | to every `to`, instruction by instruction (today) | the plan's §5.4 slices, the bound its own | both to the same `to`, the Sega CD first |
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

The module tree `s32x` holds two `beryl-sh2` processors on their own buses, the system registers, the comm ports, the
FIFO from the 68000, SDRAM, the frame buffers and the 32X VDP, PWM and the boot-ROM replacements. On the interface:

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

**The 32X's stages may amend the interface once their code exists.** What is written here for the 32X is argued from
the plan and the documents, before an SH-2 runs in Nephrite; where stage 11 or 12 finds the surface wanting (a
requester on map calls, a level towards the 68000, the overlay's form, the CD32X pair's decode), the change is a commit
to `interface.rs` with its reason recorded in `Nephrite_Native.md`, the Sega CD's implementation brought along in the
same commit and its digests held.

### 3.10 Both at once (CD32X)

`Both<A, B>` in `interface` composes two attachments with no knowledge of either: the map asks `A` (the expansion port's)
then `B` (the slot's); `run_to` takes `A` then `B` to the same `to`; lines go to both; `sound` lends both streams;
`EXPANSION` is either's; state, spaces and processors are `A`'s then `B`'s. The configuration is
`Genesis<Both<MegaCd, S32x>>`. If stage 12 finds an address both decode (in mode 2 the Sega CD fills `$000000`–
`$3FFFFF`, where the 32X's vector override also stands), the pair's rule goes in a type of the assembly that
replaces `Both` for that pair, since only the assembly may name both attachments; `s32x` never names `mcd` (§5.3's
source check). How a disc is known to want the 32X is stage 12's question, with its state (Q5).

*Named 2026-10-15:* CD32X's target is **Doom CD32X Fusion**, a homebrew disc that runs on the Sega CD and the 32X
together, with its music from the disc. Stage 12's gate includes it booting and played through `Genesis<Both<MegaCd,
S32x>>`, from the player's own copy; the project fetches or commits none of it unless its licence allows.

### 3.11 The J-Cart: a cartridge feature, not an attachment

*Recommended, with its reasons:* the J-Cart stays a feature of the cartridge, in the Genesis's module tree.

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
  implementing the traits by `match`. The Genesis is compiled once; every access pays a discriminant test, the
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

**None of this needs a crate boundary.** Monomorphisation is per type, not per crate: `Genesis<A>` in the `genesis`
module is instantiated in `machine` for each configuration exactly as it would be from another crate, and within one
crate, with one codegen unit and fat LTO in the release profile, the instances are optimised together as they are
today. The argument of the first version of this page carries over unchanged, and loses one of its risks: no inlining
is decided again across a crate boundary.

The choice between (a) and (b) is **local to the assembly**: the Genesis is generic in both, and (b) is one type
in the `machine` module implementing the traits by `match`. So the build starts with (a), and (b) is measured beside it only if
(a)'s costs cross the bounds of Q1.

**Decided by measurement (commit 3, §11.3): (a).** Each configuration's frame compiled apart, the cartridge Genesis
within the runs' spread of the baseline and faster on six of the seven runs, the disc's as well; the `dist` library
11.3% larger and a clean release build 13 to 23% longer, both far inside Q1's half. Form (b) was therefore not built.
One cost of (a) the page did not foresee: the Genesis's helpers called from both instantiations (the VDP's
`line_end`, `slot`, `control`, `data`, `status`; the PSG's write) are no longer called from one place, and the
compiler keeps ten of them out of line where it had inlined them; with `Wired<A>` this costs nothing measurable.

### 4.4 The bench that decides

The measurements of §54.1, before and after the commit that makes the Genesis generic (§7.2, commit 3), and again after
the files move into their module trees (commit 5), since a move should change no code and that is to be shown, not
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

## 5. The modules

*Decided 2026-10-15: the interface, within the one `nephrite` crate; separate crates not now.* The boundary is held by
module privacy inside the crate and by two checks that privacy cannot replace (§5.3). The library's file, its core id,
`RustCores.props`'s row, the sidecar and the kit's samples stay exactly as they are.

### 5.1 The layout

```
src/
  lib.rs           the module list and the features (§5.4)
  interface.rs     the interface: the three traits, Master, Board, Slot, Answer, Line, Watch, Refusal, SpaceRef,
                   Bare, Both, the ranges of space and processor ids; re-exports Cart
  genesis/         the Genesis: mod.rs (Genesis<A>, Hw<A>, its buses), console.rs (Console<A>: the frame with its
                   pads, picture, sound, cartridge battery file and debugger state), cart.rs, eeprom.rs, io.rs, vdp.rs,
                   render.rs, sound.rs, ym2612.rs, fm*.rs, lfo_tables.rs, fifo_records.rs, header.rs (the cartridge's
                   half of media.rs), debugger.rs; and the tests that run a cartridge (programs, pictures, voices,
                   sounds, board_*)
  mcd/             the Sega CD, as today, with its facts for the frontend (§3.6)       [feature "mcd"]
  s32x/            the 32X: a stub, then stages 11–12                                  [feature "s32x"]
  machine.rs       the assembly: Machine and Fitted, the composition of spaces, processors and batteries
  media.rs         System and the decision by contents
  state.rs         the header, the versions, the order of the blocks, StateError
  v1.rs            Core and the exports
```

**The assembly** in this page means the four top-level modules `machine`, `media`, `state` and `v1`: the only ones
that name a concrete attachment, and the only ones a host's call reaches.

| Today | Becomes |
|---|---|
| `genesis.rs` | `genesis/mod.rs`, generic over `A`; 136–139, 190–192, 379–381, 515–579 leave (the Sega CD's half into `mcd`, the J-Cart's into the cartridge arm) |
| `cart.rs`, `eeprom.rs`, `io.rs`, `vdp.rs`, `render.rs`, `sound.rs`, `ym2612.rs`, `fm.rs`, `fm_tables.rs`, `lfo_tables.rs`, `fifo_records.rs`, `debugger.rs` | the same files under `genesis/`, unchanged but for the J-Cart's arm, the line events' hook in the VDP loop and the debugger's composition, which goes to `machine` |
| `machine.rs` | split: `Console<A>` to `genesis/console.rs`; `Machine` with `Fitted` stays; `MCD_MEMORIES`, `CARTRAM_ID` to `mcd`, `S32X_MEMORIES` to `s32x` |
| `media.rs` | split: the header, markets, save RAM and copier images to `genesis/header.rs`; `System` and the decision stay; `DISC_SIGNATURE`, `disc_offset` to `mcd`; `is_32x` to `s32x` |
| `state.rs`, `v1.rs` | stay; the Sega CD's descriptors, setting and disc helpers to `mcd`, the 32X's to `s32x` |
| `mcd/` | stays; its tests build `Console<MegaCd>` directly, `verificator.rs` and `discs.rs` included |
| tests that run a machine | `Machine::new(&image, Media::read(&image))` becomes `Console<Bare>`'s cartridge constructor, mechanically |
| `examples/` | unchanged in place; they use the crate's `pub` items (§5.3) |

### 5.2 The visibility rules

- **The Genesis's internals are private to `genesis/`.** `Hw<A>`'s fields, the VDP's, I/O's, the sound's and the
  cartridge's fields are private or `pub(super)`/`pub(in crate::genesis)`. What the assembly needs of the Genesis is a
  short list of `pub(crate)` methods on `Console<A>`: build, run a frame plain or observed, pads in, picture and sound
  out, the cartridge's battery and patches, the Genesis's spaces, processors and registers.
- **An attachment imports from the crate only `crate::interface`.** `interface` re-exports `Cart`, whose fields stay
  private to `genesis/` and whose methods an attachment may call (`read8`, `read16`, `write8`, `write16`, `answers`)
  are `pub(crate)`.
- **An attachment's internals are private to its tree.** In `mcd/`, `Sub`, `Gate`, `Cdc`, `Cdd`, `WordRam`, `Cdda`,
  `RamCart`'s fields and every register are private or `pub(in crate::mcd)`; `MegaCd`, its three trait
  implementations and its inherent functions for the frontend (§3.6) are `pub(crate)`, for the assembly.
- **The generic Genesis cannot call an attachment except through the traits**: it is generic over `A` and names no
  concrete type.
- **`pub` is kept to what tools outside the crate use** (`examples/`, the conformance kit through the ABI): `machine`'s
  `Machine` and the constructors the examples call, and a named, documented accessor for a development tool such as
  `examples/disc.rs`'s register view and stage 9's `Sub::seen`.

### 5.3 What privacy enforces, what it cannot, and the two checks

**Enforced by the compiler:** no module outside `genesis/` reads or writes the Genesis's state (its clock, its
refreshes, the VDP's FIFO, the Z80's window), the assembly included; no module outside `mcd/` reaches the Sega CD's
devices, the Genesis and the assembly going through `MegaCd`'s `pub(crate)` surface; and the Genesis's code reaches
an attachment only by the traits.

**Not enforced by privacy, as a crate boundary would:**

- **The direction.** Rust's privacy is by subtree, and the assembly sits at the crate's top level: whatever is
  visible to it with `pub(crate)` is visible to every module. So `genesis/` could `use crate::mcd::MegaCd`, and `mcd/`
  could call `Console<A>`'s `pub(crate)` methods or anything else `pub(crate)` outside `interface`. Between crates
  Cargo forbids the first outright (no dependency) and the second is limited to `pub` items.
- **Cycles.** Modules of one crate may name each other both ways; crates may not.
- **`pub` items.** What `examples/` needs is `pub`, and so visible to every module.
- **Separate compilation.** The one crate is compiled whole; a crate would build and test the Genesis with no
  attachment's code present at all.

**The two checks**, which hold what privacy cannot and fail an ordinary test run:

1. **The feature-off builds** (§5.4). `cargo test --no-default-features` compiles neither attachment, so it fails if
   `genesis/` or `interface.rs` names `mcd` or `s32x`; `--no-default-features --features mcd` and `--features s32x`
   alone fail if either attachment names the other.
2. **A source check**, a test in the assembly (`the_attachments_reach_only_the_interface`), reading the files under
   `src/mcd/` and `src/s32x/` and failing on any path of the crate other than `crate::interface` (and `super::` within
   the tree), and reading `src/genesis/` and `src/interface.rs` and failing on `crate::mcd`, `crate::s32x`,
   `crate::machine`, `crate::media`, `crate::state` or `crate::v1`. It is a text test, deliberately simple; an
   exception is a reviewed line in it.

With both, the direction is held as surely as a crate boundary would hold it at each test run, though not at each
compilation, and cycles are refused by the source check.

### 5.4 The Genesis-only build: a feature, kept as a check

The features `mcd` and `s32x`, both default, gate `mod mcd`, `mod s32x`, their arms of `Fitted`, their systems' rows
and their settings. With `--no-default-features` the crate builds a library reporting `md` alone, with nothing of the
attachments and no `emusen-disc` linked (the dependency made optional with `mcd`).

**Whether it is worth having:** yes, as check 1 of §5.3, its cheapest enforcement of the direction, run in the crate's
test sweep with each change to `interface.rs` or the Genesis. Not as a shipped library: a player's library holds the
three systems, and a second artefact would need its own sidecar and kit run for no player (Q4).

### 5.5 One owner for each fact

| Fact | Owner | Read by |
|---|---|---|
| The pad's button bits (the order the I/O reads them) | `genesis` (`io`) | the controller descriptors in `v1` (`BUTTONS`) |
| The master clock's rates, the frame rates, `START` | `genesis` | attachments through `Board`; machine info |
| The output sample rate (48 kHz) and the output's scale | `genesis` (`sound`) | attachments through `Board`; `AUDIO_RATE` |
| System ids (`md`, `mcd`, `32x`), extensions, the state's system byte | `media`, `v1` | — |
| The state's magic, versions, oldest version per system | `state` | blocks are given the version |
| ABI file numbers (firmware 1–9, batteries 0 and 10) | `v1` | — |
| The ranges of space and processor ids | `interface` | every module |
| Each space's and processor's id within a range | that range's owner | — |
| The disc's signature, opening a disc, its fingerprint, the drive's number | `mcd` | the assembly (recognition, create, insert) |
| The 32X's header test | `s32x` | `media` |
| The J-Cart's boards and word | `genesis` (`cart`, `io`) | — |
| The open bus's rule (D-29) | `genesis` | attachments through `Answer::Open` |
| An attachment's mix gain against the Genesis's output | that attachment | — |

### 5.6 Separate crates, a possible later step

The first version of this page (`18e90a20`) laid the same design out as four crates in a workspace
(`nephrite-genesis`, `nephrite-mcd`, `nephrite-32x`, and `v1/` publishing `nephrite`, after Diamond's arrangement,
`Diamond_Enhancements.md` §24.1). It is not built now. **What would trigger it:** the boundary broken in review
despite §5.3's checks, or the checks needing exceptions to pass; a second consumer of an attachment's or the Genesis's
code outside Nephrite; the one crate's build time in development becoming a cost once the 32X's code lands; or a
Genesis-only library wanted as a shipped artefact. Because each attachment's tree imports from the crate only
`interface`, the split would then be a move of files (`interface` into the Genesis's crate, each tree into a crate
depending on it, the assembly into `v1/`), not a redesign.

---

## 6. Forms weighed

The design rules ask which form a feature takes before it is built: an extension of a surface, a module or plugin, or
a component of its own.

- **Chosen: an internal surface inside one library and one crate.** The interface is the module `interface` of
  `nephrite`, internal in the sense of `EmuSen_Surfaces.md` §6.1's rules (nothing promised across commits). It has no
  version of its own: a change to it is a reviewed diff of `interface.rs`, with the Sega CD's implementation brought
  along in the same commit. No outer surface changes: the core ABI v1 stays the only one a host sees, and no new
  dependency between components is added.
- **Not now: separate crates** (§5.6). *Decided 2026-10-15.* The boundary they would add over §5.3's is compile-time
  rather than test-time, and its costs (a workspace, a lock, `RustCores.props`'s row, cross-crate inlining to measure)
  buy nothing a player or a second consumer needs today.
- **Rejected: separate libraries per machine** (`libnephrite-md`, `-mcd`, `-32x`). Each would contain the Genesis,
  which the plan's §1.1 already gives as the reason the Sega CD is not a core of its own; three registrations, three
  sidecars and three system packs for one console; a CD32X library containing two attachments and a Genesis; and the
  state, the battery files and the debugger's numbering split three ways for one machine.
- **Rejected: a runtime plugin** (attachments loaded by Nephrite as dynamic libraries). Rust has no stable ABI, so the
  boundary would be a second C ABI, at the hottest boundary in the core: a call per 68000 instruction and per access,
  through a pointer, never inlined. The attachments share Rust types with the Genesis (`Cart`, `Steps`, `StateWriter`,
  `Hooks`), which a C boundary would have to copy or serialise; the framework already has one plugin boundary for
  cores, the core ABI, and nothing supplies attachments from outside the project.

**Is the interface ever published?** *No.* Its only implementers are Nephrite's modules, and its types are the
Genesis's internals (the cartridge, the master clock's units, the VDP's lines), which no other core shares. A console
with an attachment of its own (the PC Engine's CD-ROM², the Saturn's cartridge port) would copy the pattern, a bus side
asked first with a lent slot, not the trait: its buses and its clock differ.

---

## 7. The migration

### 7.1 When, and the baseline

After stage 9 merges into WiseMan, and before stage 10's first step (Q3): stage 9 edits exactly the files this move
rearranges, and stage 10's mode 1 is the first configuration the move makes easy. The steps run as Nephrite's others,
about three hours each with a check-in after each (A5).

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

The first version's sixth commit, the crate split, is dropped; the move into module trees takes its place as commit 5
and carries the measurement it carried.

| # | What | Checks | Measured |
|---|---|---|---|
| 1 | The baseline: `examples/digest.rs`; a pinned layout for a disc's machine; the before numbers of §4.4 recorded | all of §7.1 | the bench, the Sega CD, size, build time |
| 2 | The J-Cart into the cartridge's arm; `attached` guards the Sega CD alone | J-Cart tests; §7.1 | the bench, Micro Machines 2 |
| 3 | `interface.rs`; `Genesis<A>`, `Hw<A>`, `Bare`; `MegaCd` on the three traits (`genesis.rs` 518–579 and 379 moved in); `Console<A>` split from `machine.rs`; `Fitted` with `Md` and `Mcd`; state and spaces composed through the traits | §7.1 | **the decision of §4**: the bench, the Sega CD, size, build time |
| 4 | The Sega CD's facts behind its own module: firmware descriptors, `ramcart`, batteries, drives, the disc helpers and `DISC_SIGNATURE`, the fingerprint into `MegaCd`; `cdda.rs` given the rate by `Board` | §7.1; afterwards no `mcd` outside `src/mcd/` but the assembly's | — |
| 5 | The module trees and their visibility: the Genesis's files into `src/genesis/`, its internals and the Sega CD's narrowed (§5.2); the source check of §5.3 | §7.1; the source check | the bench and the Sega CD again |
| 6 | The 32X's stub: `S32x` in `src/s32x/` on the traits, answering `Pass`, with its three placeholder memories and descriptors; `Fitted::S32x`, still counting frames only (`machine.rs` 225) | §7.1; a 32X state byte-identical | — |
| 7 | The features `mcd` and `s32x`; the three feature-off builds of §5.3 in the test sweep; the kit's Genesis sample on `--no-default-features` | the sample, 64 of 64; the digests | the Genesis-only library's size, for the record |
| 8 | The record: a section of `Nephrite_Native.md`; this page's predictions retired; the folder's README | — | — |

Commits 3 and 5 are the two where the cost can move (3 changes the code, 5 moves it); each is measured before the next
begins, and a loss stops the migration until it is explained (§4.3's fallback is commit 3's alternative).

### 7.3 The state format

**No change, and no new version.** The move keeps every state byte-identical, at version 19 or whatever version stage 9
leaves, argued from the write order (§3.5): the header, the spaces in id order, the Genesis's block and the Sega CD's
block are written in today's order by today's code, moved; the one field that changes owner, the disc's fingerprint,
is written by its new owner at the same offset. A 32X machine keeps writing its three placeholder memories through the
stub (commit 6). The pinned layouts and the digests show it at every commit (A4).

What the format will need, and the move does not add: a configuration that the system byte does not decide. Today a
system byte decides the blocks (`md` none, `mcd` the Sega CD's, `32x` none). Mode 1 (a cartridge, `md`, with the Sega
CD's block) and CD32X (a disc with the 32X's block) break that. *Recommended (Q5):* a byte of the attachments present,
added to the header with the first stage that builds such a configuration, at that stage's version, and not before.

### 7.4 Risks

- **Stage 9's merge.** The move rewrites the files stage 9 edits; starting before stage 9 merges would make every one
  of its commits a conflict. Hence §7.1's order, and the move finished before stage 10 begins.
- **Monomorphisation's size and build time** (§4.3), bounded by Q1, with (b) as a local fallback.
- **Byte identity's subtle orders**: the VDP, attachment, Z80 order in `catch_up`; `run_to` before a register access;
  the open bus supplied by the Genesis; the fingerprint's offset. Each is a line of §3.4 and is held by the digests.
- **The checks are only as good as the runs that include them.** The feature-off builds and the source check hold the
  direction at test time, not at each compilation (§5.3); they must be in the crate's ordinary test sweep, not a
  separate script that can be forgotten.
- **Visibility churn.** Narrowing `pub` fields that tests, examples and stage 9's tools reach (`examples/disc.rs`
  reads the Sega CD's registers; `Sub::seen`) needs accessors or moved tests (§5.2); a field the assembly genuinely
  needs becomes a method of `Console<A>`, not a `pub(crate)` field.
- **CD32X's overlapping decode**, unknown until stage 12 (§3.10); contained by the assembly's pair type.
- **The 32X's amendments** (§3.9): a surface written before its second implementer exists will change; each change
  carries the Sega CD with it and its digests.

---

## 8. Questions for the tester

1. **Q1, the dispatch.** *Recommended:* one instantiation per configuration (§4.3, (a)), with the enum (b) measured
   beside it only if (a) grows the shipped library by more than half or a clean release build by more than half; the
   cartridge Genesis's frame cost may not rise beyond the runs' spread in either case.
2. **Q2, the Genesis's files.** Privacy by subtree needs the Genesis to be a subtree. *Recommended:* move its files into
   `src/genesis/` (commit 5), at the cost of git renames, rather than leave them at the top level, where nothing could
   be private to the Genesis without also being private from the assembly.
3. **Q3, when.** *Recommended:* the eight commits of §7.2 right after stage 9 merges, before stage 10's first step.
4. **Q4, the Genesis-only build.** *Recommended:* the features of §5.4, kept as a check in the test sweep, never shipped
   as a library of its own.
5. **Q5, a configuration in the state's header.** *Recommended:* a byte of the attachments present, added by the first
   stage that builds mode 1 or CD32X, at that stage's version; the move adds nothing.
6. **Q6, how far visibility is narrowed in the move.** *Recommended:* in commit 5, the Genesis's and the Sega CD's
   internals to their trees and the assembly's reach to `Console<A>`'s methods, so that the boundary exists when the
   move ends; any narrowing past that (the examples' `pub` items) later, each its own commit.
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
list (§2.4, §6.23–§6.26, §27.4); `Diamond_Enhancements.md` §24.1, for the first version's crate layout. Code, at `d32a00b3`: Nephrite's `src/` whole for the
count, and in full `genesis.rs`, `machine.rs`, `state.rs`, `v1.rs` (to its tests), `debugger.rs` (its non-test half),
`mcd/mod.rs`, `mcd/state.rs` (its writer), `mcd/cdda.rs`, the J-Cart's parts of `cart.rs` and `io.rs`, `sound.rs`'s
`take_with`, `examples/bench.rs`; `RustCores.props` and `EmuSen.csproj`'s cargo targets; Diamond's two manifests; the
stage 9 branch's diff of `genesis.rs` and `mcd/mod.rs`. Not read: any emulator's source, any RTL, any firmware.

---

## 11. As built (2026-10-15)

The migration of §7, commit by commit, from WiseMan `fa1e4242` on the branch `nephrite-interface`. Each commit's
checks are §7.1's four; what each proved is recorded here as it is made, and `Nephrite_Native.md` §62 summarises it.

### 11.1 Commit 1: the baseline

- **`examples/digest.rs`**: `digest <image>...` or `digest --list <names> --from <dir>` runs an image 1,200 frames, pad
  1's buttons pressed in turn as the kit presses them, and at frames 300, 600 and 1,200 prints FNV-1a (64-bit) over the
  state, the picture and every sample drained so far, and the state's length. A disc runs with `NEPHRITE_BIOS` as its
  boot ROM. The baseline is 71 runs: the kit's Genesis sample (64 images, its 32X cartridge among them), the bench's
  games not in it under their bench names (Sonic the Hedgehog 2, Thunder Force IV, Streets of Rage 2 and Micro Machines
  2's J-Cart image), the 240p disc as its 2,048-byte image and as its sheet with both audio tracks, each with
  mcd-verificator as the boot ROM, and the image again with no boot ROM. Run twice at `fa1e4242`, the two outputs equal
  byte for byte; kept as `~/.cache/emusen/probe/nephrite/interface/digests-base.txt`.
- **A disc's machine's layout pinned** (`v1.rs`, `a_disc_machines_layout_is_pinned`): a disc made through create with
  the backup RAM cartridge plugged in, every field from `Magic` to `DiscFingerprint` at its offset, 1,156,303 bytes.
  The cartridge's `the_layout_is_pinned` stands as it was.
- **The before numbers** (§4.4), `examples/bench`, both columns the same build, to show the runs' spread: each run
  under the shared timing lock (`~/.cache/emusen/probe/timing.lock`) on one core, the builds alternated, load 3.3 to
  3.5 from other work on the machine:

| Image | Best (ms a frame) | Same build again | Spread |
|---|---|---|---|
| Sonic the Hedgehog | 1.365 | 1.360 | 0.4% |
| Sonic the Hedgehog 2 | 1.254 | 1.249 | 0.4% |
| Thunder Force IV | 1.347 | 1.349 | 0.2% |
| Streets of Rage 2 | 1.164 | 1.152 | 1.0% |
| Phantasy Star IV | 1.420 | 1.416 | 0.3% |
| Micro Machines 2 (J-Cart) | 1.459 | 1.458 | 0.1% |
| The 240p disc, mcd-verificator (best of 5 × 1,200) | 1.599 | 1.585 | 0.9% |

- **Size and build time**: `libnephrite.so` built with the `dist` profile, 1,445,920 bytes; a clean `cargo build
  --release --lib` of the crate, its dependencies built, 4.87 to 4.96 s over three builds.
- **Tests**: the crate's 173 (172 and the new layout) pass with the programs' and discs' folders set.

### 11.2 Commit 2: the J-Cart in the cartridge's arm

The J-Cart's two words are decoded by two guarded arms ahead of the cartridge's own, inside `$000000`–`$3FFFFF`, in
each of `read8`, `read16`, `write8` and `write16`; the `attached` flag is set by a Sega CD alone, and `attached_read8`,
`attached_read16` and `attached_write` lose their J-Cart halves. Only an access to the cartridge's range pays the J-Cart's
test, and no access pays it on a console without one but the flag's (§3.11).

- **Byte identity**: the 71 digests equal the baseline's; the crate's 173 tests pass, the J-Cart's
  (`a_j_cart_gives_players_3_and_4_and_reads_back_th`) among them; the kit's sample 64 of 64 and the 240p disc with
  mcd-verificator as file 2 compliant (C1–C19).
- **Cost**, the build against the baseline's, alternated, best of 6 × 3 × 600 (the disc 5 × 1,200), load 3 to 4: Sonic
  the Hedgehog 1.356 ms (1.362), Sonic 2 1.241 (1.249), Thunder Force IV 1.339 (1.348), Streets of Rage 2 1.145 (1.146),
  Phantasy Star IV 1.412 (1.410), Micro Machines 2 1.450 (1.452), the 240p disc 1.590 (1.584). Every difference within
  the runs' spread of §11.1.

### 11.3 Commit 3: the interface, `Genesis<A>`, and the Sega CD on it

- **`src/interface.rs`**: `Master`, `Board`, `Answer`, `Attachment` (§3.3 as built), `Refusal`, `AttachmentState`,
  `SpaceRef`, the three ranges of space ids, `AttachmentView`, and `Bare`, every member of whose implementations is
  empty or `None` and inlined.
- **`genesis.rs`**: `Genesis<A = Bare>`, `Wired<A>`, `MainBus<'_, A>`, `Z80Bus<'_, A>` (§3.4 as built). The map's
  methods (`run_vdp`, `vdp_wait`, `dma_hold`, `dma_read`, `catch_up`, `read8`, `read16`, `write8`, `write16`, the VDP's
  ports, the Z80's space) moved, unchanged but for the attachment's calls, from `impl Hw` to `impl<A: Attachment>
  Wired<A>`; the four guards became `asked8`/`asked16` and `att.write`; `catch_up` calls `att.run_to`; `dma_read` asks
  `att.transfer16`. `attached_*`, `mcd_*` and `attached_dma_read` left for the Sega CD. Five lines that read one device
  while calling another through `Deref` name `self.dev` explicitly, as the borrow checker asks.
- **`mcd/`**: `MegaCd::new(board, bios, disc)`; `Attachment for MegaCd` (`EXPANSION`, its decode in mode 2, the RAM
  cartridge's slot, Word RAM's transfer a word late, `run_to` as it was, `#[inline(never)]` kept, and `sound`, its one
  stream); `AttachmentState` in `mcd/state.rs` with the fingerprint its own; `AttachmentView` with `MEMORIES`, `BRAM_ID`
  and `CARTRAM_ID`, which moved from `machine.rs` (re-exported there).
- **The assembly**: `Machine::fitted` and `fitted!` (§3.4 as built), `FittedSaved` in `state.rs` (§3.5 as built);
  `Machine::genesis` is gone, and the tests and examples that read `m.genesis.hw`, `cpu` or `z80` read `m.hw()`,
  `hw_mut()`, `cpu()` and `z80()`, a mechanical change of 140 lines.
- **Tests**: the Word RAM test moved to `mcd` at the interface (`a_transfer_from_word_ram_reads_a_word_late`, the
  attachment's `transfer16` and `read16`), and `genesis.rs` gained one with a stub attachment
  (`a_transfer_asks_the_attachment_apart_from_the_68000`) showing that the Genesis asks `transfer16` for a transfer and
  `read16` for the 68000 and falls to its own map where the attachment passes; one more checks each part's spaces stand
  in its range of ids. 175 pass.
- **Byte identity**: the 71 digests equal the baseline's; both pinned layouts unchanged; the kit's sample 64 of 64 and the
  240p disc with mcd-verificator as file 2 compliant (C1–C19); WiseMan's Nephrite, Genesis and Registration tests, 86
  pass.
- **Cost**, best of 6 × 3 × 600 (the disc 5 × 1,200), against the baseline's build, alternated, load 1.8 to 3.8: Sonic
  the Hedgehog 1.355 ms (1.358), Sonic 2 1.241 (1.248), Thunder Force IV 1.353 (1.347), Streets of Rage 2 1.147 (1.151),
  Phantasy Star IV 1.406 (1.414), Micro Machines 2 1.445 (1.451), the 240p disc 1.575 (1.588). A run with the load at
  5 to 14 from other work on the machine is not counted. `libnephrite.so` (`dist`) 1,609,128 bytes (1,445,920, +11.3%);
  a clean release build 5.49 to 6.12 s (4.87 to 4.96, +13 to 23%).
- **The form rejected on the way** (§3.4): `Map<'_, A>` cost the cartridge 1.3 to 2.4%; measured three ways (as it
  was, with the Sega CD's instantiation removed, and with `Hw`'s three removed fields put back as padding, 1.3%), so
  neither the second instantiation nor the devices' layout explained it. Not committed.

### 11.4 Commit 4: the Sega CD's facts behind its module

- **`src/mcd/frontend.rs`** (§3.6 as built): `DISC_SIGNATURE` and `disc_offset` from `media.rs`; `is_disc` from `v1.rs`;
  `open_disc`, `fingerprint` and `system_area` from `mcd/mod.rs` (re-exported there, as the examples name them);
  `bios_firmware` with the replacement's words from `v1.rs`'s `cd_bios` and `CD_COST`; the `ramcart` setting; the two
  save memories' declarations, kinds, media and labels, from `v1.rs`'s `backup_ram` and `ram_cartridge`, and their
  loading at create; the drive's calls from `v1.rs`'s `drives`, `media_lid` and `media_insert`, the fingerprint set by
  `insert`.
- **`Cdda::new` takes the mix's rate**, given by `Board::rate`, so `cdda.rs` no longer reads `crate::sound::RATE`. With
  `DISC_SIGNATURE` the Sega CD's own, **no production line of `src/mcd/` reaches the crate outside it but
  `crate::interface`** (§2.2 counted two).
- **After it**, no line outside `src/mcd/` names the Sega CD but the assembly's (`lib.rs`, `machine.rs`, `media.rs`,
  `state.rs`, `v1.rs`).
- **Byte identity**: the 71 digests equal the baseline's; the crate's 175 tests; the kit's sample 64 of 64 and the 240p
  disc compliant; WiseMan's 86. No code on a frame's path changed, so the bench was not run.
