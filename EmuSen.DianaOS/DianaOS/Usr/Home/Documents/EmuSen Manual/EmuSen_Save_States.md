# EmuSen — Save State Format

Covers `EmuSen/Common/StateSerializer.cs` and `VenusCore.SaveState`/`LoadState`. Rewind uses this same format in memory rather than on disk — see `EmuSen_Rewind_And_FastForward.md` §1.

---

## 1. What the format is

A save state is the reflection-walked contents of four object graphs — `Cartridge`, `Cpu`, `MemoryBus`, `Spc700` — written back to back with no field tags, preceded by a small header (§3). Within each type, fields are visited in **ordinal name order** so that write and read always agree, and arrays are filled in place rather than reallocated (every array in the graph is already correctly sized by its owner's constructor).

`Renderer` is deliberately not saved. It holds presentation handles that have no business in a save file, and it is fully re-derivable from PPU state — the next `RunFrame()` rebuilds it.

Three kinds of field are excluded via `[SkipInState]`: delegate dispatch tables (rebuilt identically on construction), back-references to an already-serialized owner, and transient non-game-state buffers such as `SDsp.AudioBuffer`.

**The format is still positional, not self-describing.** There is no field-name tagging, so adding, removing, or reordering any serialized field changes the layout. That is survivable now — but only by bumping the version in §3 and keeping a read path for the old layout, exactly as was done for §2. Doing it without that will silently misalign an older file into garbage rather than failing cleanly.

---

## 2. The aliased-array problem (`[AliasOfSerializedField]`)

`Spc700.Ram` is a single 64 KB array, but three different types hold a reference to it:

- `Spc700.Ram` itself — the real owner
- `SDsp._ram`, assigned by `SDsp.AttachMemory(ram)`
- `DspVoice._ram` × 8, assigned by the same call fanning out to each voice

All ten references point at the *same* array. The reflection walk had no way to know that, so it serialized each one as if it were independent data: **nine redundant 64 KB copies, 589,824 bytes, in every single save state.** A state that should have been ~584 KB was ~1160 KB, roughly half of it the same APU RAM over and over.

This cost rewind considerably more than it cost files, because a delta between two snapshots re-encodes every APU RAM change nine times — and APU RAM churns constantly whenever music is playing. Measured across 750 snapshots (50 seconds of history):

| | Before | After | |
|---|---|---|---|
| Raw state | 1160 KB | 584 KB | −49% |
| Super Mario World | 3979 KB | 2498 KB | −37% |
| Donkey Kong Country | 36.8 MB | 6.6 MB | −82% |
| Final Fantasy VI | 51.4 MB | 6.6 MB | −87% |
| Chrono Trigger | 64.1 MB | 8.6 MB | −87% |

At the default 96 MB rewind budget that takes Chrono Trigger's worst case from ~78 seconds of history to roughly **ten minutes**.

The fix is the `[AliasOfSerializedField]` attribute. It means "this field is a reference to an array some other field already writes" — skipped on write and on any modern read, but still **read** when parsing a pre-v1 file, which contains it. Only two fields carry it (`SDsp._ram`, `DspVoice._ram`), and both are re-wired by `Spc700`'s own `AttachMemory` call independently of serialization, so dropping them from the stream loses nothing.

Note the ordering detail that makes the legacy path work: `Spc700`'s fields sort as `Dsp` before `Ram`, so a pre-v1 file writes the aliases *before* the real array. Reading them back fills the same array twice with identical bytes — harmless, which is why the redundancy was invisible for so long.

---

## 3. The version header

As of state version 1, a state begins with:

| Offset | Size | Contents |
|---|---|---|
| 0 | 4 | magic `0x53454E53` (`"SNES"` little-endian) |
| 4 | 4 | format version, currently `3` |

### Version 2 — coprocessor state

v2 appends **one extra blob after the original four** (`Cart`, `Cpu`, `Bus`, `Spc700`), and only when the cartridge actually carries a coprocessor — currently the SA-1 (`Venus_SA1.md` §9). An ordinary cartridge writes a v2 file that is byte-identical to what v1 would have written.

The legacy read path this required is unusually cheap: `Cartridge.Sa1` is `[SkipInState]` so it never perturbs `Cart`'s own layout, and a v1 file **cannot** be an SA-1 game, because SA-1 games could not run when v1 was the current format. So "read the extra blob only when `version >= 2`" is the whole migration.

Adding it did need one `StateSerializer` change: an **interface-typed field** — `Cpu._bus`, which is a `MemoryBus` for the S-CPU and an `Sa1Bus` for the SA-1 — is now walked like a class. `Type.IsClass` is false for an interface, so it would otherwise have thrown on the first save.

`LoadState` peeks those eight bytes. If the magic matches and the version is in range, it reads the modern layout. If not, it rewinds the stream and reads the **pre-v1 layout** — no header, and aliases present inline. That is what lets save states written before this change keep working unchanged.

A file whose version is *newer* than the running build throws rather than guessing. A magic match with a nonsense version (`< 1`) is treated as a pre-v1 collision and falls back to the legacy path, since a pre-v1 file's first eight bytes are `TotalFrames` and could in principle collide — it would take a `TotalFrames` of ~5.6 billion (about three years of continuous play) to do so.

`LoadState` requires a seekable stream, because that peek-and-rewind is how the two formats are told apart. Every caller (a `FileStream`, or `RewindBuffer`'s `MemoryStream`) already is.

### Version 3 — NEC DSP state

v3 appends the NEC DSP's blob after the SA-1's and the GSU's, on exactly the same terms: only for a cartridge carrying one, and skipped otherwise. Its firmware is `[SkipInState]` — `LoadRom` re-reads that before any state load, the same way `Cartridge` handles ROM bytes — so what a v3 file actually carries is the chip's registers, its accumulator flags, and its data RAM. See `Venus_NecDSP.md` §7.

The v2 migration argument repeats verbatim one chip later: a v2 file **cannot** be a NEC DSP cartridge, because none could run when v2 was the current format. "Read the extra blob only when `version >= 3`" is again the whole of it.

### Migration

None needed — pre-v1 files load as-is, forever, via the legacy path. Re-saving over one converts it: it is written in v1 and roughly halves in size. Verified against the two states that predate the change:

- `LttP.state`: 1,194,692 → 604,875 bytes
- `SMW.state`: 1,188,546 → 598,730 bytes

Lossless in both directions — loading a pre-v1 file and loading its converted v1 equivalent produce identical `regs` + WRAM + VRAM + APU RAM dumps, and re-saving a v1 state twice is byte-identical.

---

## 4. Known limitations

- **Positional layout, per §1.** Any field change needs a version bump plus a legacy read path.
- **`Cartridge.SavePath` is serialized.** Loading a state therefore restores the SRAM path recorded when it was written, not the path the current session would have chosen. Pre-existing behavior, called out here because it is surprising: a state moved between machines carries the other machine's save path with it.
- **A few sub-frame `VenusCore` fields were never in the format** (`_lineCycles`, `_scanlineStarted`, `_spc700CycleRemainder`). Since states are only taken at frame boundaries these are sub-scanline quantities and are not observable in practice.
