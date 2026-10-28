# EmuSen_Modding — modding tools on a Unix foundation

*Proposal, 2026-10-27; its shape decided by the tester the same day (§6).* Asked by the tester: modding tools — ROM
hacks, asset editing, live script mods, texture packs and enhancement patches, audio replacement, and mods shared and
stacked — "in the form of a Unix foundation", building up to the windows last; split so that no one thing becomes a
giant.

## 1. The foundation

- **Every tool is a DianaSh command first**, plain files and text in and out, one job each. **The commands come first
  and the windows are built up to**: each tool is usable and tested from the shell before a window is put over it.
- **The tools people already use are used.** EmuSen does not build a paint program or a text editor: pixel artists keep
  Aseprite, GIMP or Krita; translators and script writers keep their editors (or Pegasus). EmuSen's job is the bridge —
  out of the game into plain files, and back in, checked against what the hardware can hold.
- **A mod is files, never a ROM**: patches, tables, scripts, images, audio and a manifest; the player's own game is the
  base.
- **Mods are layers in a game's package** (`EmuSen_GamePackages.md` §3), declared in its manifest with kind, version and
  order, on or off per game, applied at load; the library file never changes.

## 2. Layer 1: formats, one owner each

| Owner | Holds |
|---|---|
| `emusen-patch` (built) | IPS, UPS and BPS: reading, applying, the source's checksum; and **making** a patch from two files |
| `emusen-tbl` (new) | table files, text encoding and decoding by a table, pointer tables |
| the debug-views capability (`EmuSen_DebugWindow.md` §2) | decoding a view, and an optional **encode** a core declares per view |
| a texture hash shared by Mars, Diamond and Sapphire | one name for a texture everywhere |
| the mod manifest, in Galaxia beside the package manifest | what a mod is: kind, version, order, files |

## 3. Layer 2: commands, a module per job

| Module | Commands | Its job |
|---|---|---|
| `emusen.patch` | `patch make <original> <modified> [--format bps]`, `patch apply <in> <patch> <out>`, `patch info` | changes into patches, and a patched file written where the player says (never into the ROM library) |
| `emusen.text` | `tbl`, `text dump <rom> --tbl t.tbl --range a-b`, `text insert` | translation and script text |
| `emusen.gfx` | `view get <view> --png <file>` (an indexed PNG with the view's real palette), `view put <view> <file>` (checked: size, colour count, palette, then encoded by the core) | graphics out to plain images and back |
| `emusen.texture` | `texture dump`, `texture pack check` | 3D texture packs, by hash |
| `emusen.mod` | `mod ls`, `mod add/rm <package> <mod>`, `mod on/off`, `mod order`, `mod pack`, `mod run <script>` | mods installed, layered, run and shared |

- **Patches are a layer kind.** `pkg patch add/rm` (`EmuSen_GamePackages.md` §15) become `mod add/rm` of a patch, the
  old names kept as aliases so nothing that uses them breaks.
- **Enhancement codes** (widescreen, 60 fps) are cheats, in the cheat format, from the cheats repository.
- **MSU-1 is not a tool** but a capability Venus gains (from the published MSU-1 specification); replacement audio is
  one more layer kind.

## 4. Layer 3: windows, thin, built last

- **Mods**, the mod workshop: a game's tiles, sprites, tilemaps and palettes
  browsed through the same renderers the Debugger uses, **exported** to an image editor and **imported** back with `view
  put`'s checks, small touch-ups in place, and **Save as Patch** for what was changed; later the translation work —
  tables, pointers, the script beside the screen — over `emusen.text`. The Debugger stays read-only.
- **Packages owns a game's layers**, since a mod is a layer inside a game's package
  (`EmuSen_PackageManager.md` §3b): patches, textures, scripts, audio and codes, on, off and in order; conflicts named
  (two layers writing the same bytes); a patch checked against the dump; mods installed from the emusen-mods
  repository; **Export Patched Game** to a folder the player chooses. The patch controls of the *Game Package* window
  move there.
- Each on `AdaptiveLayout`, with its big-picture form and the pad.

## 5. Stages

| Stage | What | Model |
|---|---|---|
| **X1** | `patch make/apply/info`; the mod manifest and layers; `mod ls/add/rm/on/off/order/pack`; `pkg patch` as aliases | routine |
| **X2** | Enhancement codes through the cheats; the emusen-mods repository | routine |
| **X3** | The encode side of debug views; `view get`, `view put` (after the Debugger's D0–D2) | careful: a stable surface |
| **X4** | `emusen-tbl`, `text dump`, `text insert` | routine |
| **X5** | `mod run`: live script mods, frame hooks | routine |
| **X6** | Texture dump and packs on the GPU path | careful, GPU windows with the tester present |
| **X7** | MSU-1 in Venus | careful: accuracy |
| **X8** | The layers in Packages, then the Mods workshop, on `AdaptiveLayout` | routine |

## 6. Decided 2026-10-27 by the tester

1. The Unix foundation: commands first, the windows built up to; mods as files and package layers, never a ROM.
2. The split in three layers (formats, commands, thin windows), the tools people already use kept rather than rebuilt.
3. The ROM patcher is not a window of its own: applying patches is Packages' (a patch is a layer), making them is
   `patch make` and Mods' Save as Patch, and a patched file is `patch apply` or Packages' Export Patched Game.
4. One owner for layers: Packages, where the packages already are.
5. Names, consolidated the same day: **Mods** (the mod workshop: graphics and, later, translation) and **Packages**
   (packages and their layers); no tool of its own for mods, for graphics or for translation.
6. An emusen-mods repository, each mod under its authors' licence with attribution.
