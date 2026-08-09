# Mercury — the Game Boy Color extensions

*Written 2026-08-08, Phase D of the Mercury gameplan, landed the same day as the
PPU it extends. Read `Mercury_Ppu.md` first: everything here is a modification to
the machine that page describes, not a second machine.*

---

## 1. When the core is in colour mode

Colour mode is decided **once, at construction**, from the `$0143` header byte
that `Cartridge.Cgb` already parsed (`Mercury_Memory.md` §2.1). Both
`CgbSupport.Enhanced` (`$80`) and `CgbSupport.Required` (`$C0`) select it. There
is no runtime toggle and no setting.

The consequence is that `MemoryBus` sizes its own memory in its constructor —
16K of VRAM instead of 8K, 32K of WRAM instead of 8K — and every colour register
either exists or does not for the whole session. A monochrome cartridge is
byte-for-byte the machine that existed before Phase D; the CGB register block
falls through to the generic `$FF00` array and reads back what a DMG's would.

Running an *enhanced* cart in colour mode rather than monochrome is what the real
console does, so it is what Mercury does. That choice matters more than it looks,
because of §1.1.

### 1.1 The A register is the console's name badge

A CGB-enhanced cartridge decides which of its two rendering paths to run by
testing `A` on entry: the colour boot ROM leaves `$11` there, the monochrome one
leaves `$01`. Mercury has no boot ROM (`Mercury_Cpu.md` §5), so it hardcodes the
post-boot register file — and until this was fixed it hardcoded the *DMG* one
unconditionally.

The failure that would have caused is worth writing down because it looks like a
video bug and is not. An enhanced cart would read `A = $01`, take its monochrome
path, and never upload a colour palette — while the PPU, which decided from the
header rather than from `A`, would render through colour palettes that no one had
written. Palette RAM initialises to white (§2.2), so the symptom is a blank white
screen on a game that runs perfectly, with correct tile data sitting in VRAM.

`Cpu.Reset(bool cgb)` now takes the flag and sets the whole CGB register file:
`AF=$1180`, `BC=$0000`, `DE=$FF56`, `HL=$000D`.

## 2. Colour palettes

Eight background palettes and eight sprite palettes, four colours each, held as
little-endian RGB555 in two 64-byte arrays. Neither is in the CPU's address space:
a game reaches them through an index register and a data port
(`$FF68`/`$FF69` for background, `$FF6A`/`$FF6B` for sprites).

Bit 7 of the index register makes a *write* to the data port step the index
afterwards, which is how a game pushes eight bytes of palette through one address
in eight consecutive stores. Reads never auto-increment, on hardware or here.

### 2.1 Five bits to eight

Each channel is expanded as `(c << 3) | (c >> 2)` — the top three bits replicated
into the bottom. The naive `c << 3` is wrong in a way that is easy to miss: it maps
full-scale `$1F` to `$F8`, so a game's pure white comes out slightly grey and
nothing on screen ever reaches full brightness.

**No colour-correction curve is applied.** The CGB's actual panel is dimmer and
more saturated than a modern display, and emulators that "look right" are applying
a transform on top of the hardware's output. Mercury emits what the hardware
produced, for the same reason `Mercury_Ppu.md` §6 emits neutral greys for a DMG:
the frame buffer is a record, and a display characteristic belongs in a shader
where it can be turned off.

### 2.2 Why palette RAM starts white

Hardware leaves palette RAM in an undefined state at power-on and every real game
writes it before enabling the LCD. Mercury fills it with `$FF` — white — rather
than zero, so that the failure mode of a game that draws before uploading is a
white screen rather than a black one. A white screen reads as "nothing has
happened yet"; a black screen reads as "the emulator is broken". This is a
debugging affordance, not a hardware claim.

## 3. What the CGB changes about rendering

Three things, and only three:

**Tile map attributes.** Every entry in a tile map has a second byte at the same
address in VRAM bank 1: palette in bits 0-2, tile bank in bit 3, X flip in bit 5,
Y flip in bit 6, priority in bit 7. A DMG background has none of these — it cannot
flip a background tile at all — so the renderer reads the attribute byte as a
literal zero in monochrome mode and the same code path serves both. That is the
whole reason there is one `RenderBackground` rather than two.

**Sprite priority becomes OAM order.** The DMG's leftmost-wins rule is gone; a CGB
resolves overlap by OAM index alone. Selection is unchanged — the first ten sprites
in OAM order still win the line, on both consoles — so the CGB simply skips the
sort described in `Mercury_Ppu.md` §5.1. Two orderings on a DMG, one on a CGB.

**LCDC bit 0 is reinterpreted, not extended.** On a DMG, clearing it blanks the
background and window (`Mercury_Ppu.md` §4.3). On a CGB it does nothing of the
kind: the background still draws, and what it loses is its *priority* — with the
bit clear, sprites are in front of everything regardless of any priority bit.

That last one is the only place in the whole colour extension where a register
means something different rather than something more, and it is worth naming as
such. `BackgroundWins` is where all three of the priority inputs meet: the
sprite's own attribute bit 7, the map attribute's bit 7, and LCDC bit 0 waiving
both.

## 4. HDMA

Two transfer modes behind one register block, distinguished by bit 7 of the byte
written to `$FF55`.

- **General purpose** (bit 7 clear) copies the whole length immediately and
  **charges the CPU for the time**, as of 2026-08-09 — see §4.1.
- **HBlank-driven** (bit 7 set) copies 16 bytes each time a visible line enters
  mode 0, and this one *is* timing-dependent, which is why it is real. The PPU
  calls `OnHBlankStarted` at the mode 3 → 0 transition and the bus moves one block.
  Games use it to stream tile data in during hblank without touching vblank, which
  is exactly the trick the mode exists for.

Reading `$FF55` reports bit 7 set when nothing is running and the blocks still
owed, minus one, while one is. Writing bit 7 clear *during* an hblank transfer
cancels it rather than starting a general-purpose one — the one case where the
same write means two different things depending on state.

The destination register only has bits 12-4; a transfer is always 16-byte aligned
and always lands in VRAM, in whichever bank `$FF4F` currently selects.

### 4.1 The stall, and why the copy is still instant

*Added 2026-08-09 with the cycle-granular bus (`Mercury_Cpu.md` §3).*

Hardware moves 16 bytes in eight machine cycles and the CPU is stopped for all of
them. Mercury splits those two facts apart: the **copy** happens at once inside the
`$FF55` write, and the **cost** is accumulated in `_stallCycles` and taken by
`MercuryCore` after the instruction, which ticks the rest of the machine by that
much before running another one.

Doing it this way rather than interleaving the copy with the clock is a real
decision and not laziness. What a program can observe about a general-purpose HDMA
is that time passed — the PPU has moved on, the timer has counted, an interrupt may
now be pending. What it cannot observe is the *order* the sixteen bytes arrived in,
because it is not running while they arrive. Splitting cost from copy buys the
observable half at a fraction of the complexity, and the half left behind is the
one with no observer.

Two details follow from the clock rather than from the transfer:

- **Double speed halves it.** The transfer is eight machine cycles either way, and a
  machine cycle is half as long, so the charge is 16 T-cycles a block rather than
  32. A test asserts both numbers, because getting this backwards is invisible in
  single speed.
- **The stall is taken once.** `TakePendingStall` clears as it reads. An accumulator
  that is read without clearing would charge every subsequent instruction for the
  same transfer, and the symptom would be a game that runs progressively slower
  after its first HDMA rather than an obvious fault.

**HBlank-driven transfers accumulate the same charge**, block by block, but the
block is moved from inside the PPU's tick rather than from a CPU write. The stall
is therefore drained at the end of whichever instruction happened to be running
when the line entered mode 0, which is up to one instruction late. Left as is: the
mode exists precisely so the copy hides inside hblank where the CPU has nothing to
do, so the misplacement lands in the window that was already idle.

## 5. Double speed

`STOP` is not a stop on a CGB. With bit 0 of `$FF4D` armed, it toggles the CPU
between 4.19 MHz and 8.39 MHz; with it not armed, it is the DMG's `STOP` and
Mercury still does nothing (`Mercury_Gameplan.md` §4).

The important part is what does *not* double. The CPU, the DIV counter and the
timer all run at the new rate. **The LCD does not** — the screen still refreshes at
59.7275 Hz, which is the entire point: double speed buys a game more CPU time per
frame, not a faster frame. `MemoryBus.StepOneCycle` therefore ticks the PPU on
every other cycle while double speed is on, and `MercuryCore.RunFrame` doubles its
LCD-off watchdog budget to match, since a frame is now 140448 CPU cycles.

Two known gaps: the switch is instantaneous here, where hardware spends about
2050 cycles in a stopped state performing it, and `KEY1`'s armed bit is cleared by
the switch but is not otherwise readable as hardware makes it. Neither is
observable to a game that uses the documented arm-then-`STOP` sequence.

## 6. What Phase D did not bring

- **No DMG-on-CGB compatibility palettes.** A monochrome cartridge runs as a
  monochrome cartridge, in greys. Real hardware would colourise it from a table
  the boot ROM picks by header checksum. That table is a boot-ROM artefact, not a
  hardware behaviour, and Mercury has no boot ROM.
- **`OPRI` (`$FF6C`) is accepted and ignored.** It selects DMG-style sprite
  priority on a CGB, and Mercury never enters the mode that would need it.
- **No infrared port (`$FF56`).** Four games use it, all for trading.
- **`.gbc` is still not a claimed extension.** That is a `CoreCatalog` change and
  it belongs with Phase B, which is what puts Mercury in front of a file picker at
  all.
