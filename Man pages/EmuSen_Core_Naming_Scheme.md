# EmuSen — Core Naming Scheme

*(Living document — companion to `EmuSen_Project_Overview` and `EmuSen_Core_Gameplan` in this same `Man pages` folder. This one is purely naming/organizational convention: what a future core is called and where it lives, not what it does.)*

---

## 1. Where the project name comes from

**EmuSen** = **Emu**lator **Sen**shi, a reference to *Bishoujo Senshi Sailor Moon* ("Pretty Guardian Sailor Moon" — *senshi* 戦士 = "soldier"/"guardian"). Every core is named after a character from that series, grouped by the real-world console manufacturer.

---

## 2. Nintendo consoles → Sailor Guardians (Inner Senshi)

Folder: `Cores/Nintendo/`

| Console | Codename | Folder | Status |
|---|---|---|---|
| NES | **Sailor Moon** | `Cores/Nintendo/SailorMoon - NES/` | Not started — reserved |
| SNES | **Venus** | `Cores/Nintendo/Venus - SNES/` | Active development — `EmuSen.Cores.Nintendo.Venus.*` (see §4) |
| Game Boy / Game Boy Color | **Mercury** | `Cores/Nintendo/Mercury - GB-GBC/` | Not started — reserved. Open question whether GB and GBC are different enough hardware to warrant two separate cores rather than one (undecided; revisit once Venus work is further along and there's a real basis for comparison) |
| Game Boy Advance | **Jupiter** | `Cores/Nintendo/Jupiter - GBA/` | Not started — reserved |
| N64 | **Mars** | `Cores/Nintendo/Mars - N64/` | Not started — reserved |

Sailor Moon herself anchors the console the project's own name-lineage effectively starts from (NES, the originator generation) — Venus/Mercury/Mars/Jupiter are her fellow Inner Senshi, extending outward to later Nintendo hardware in the order they line up with here.

---

## 3. Sega consoles → Dark Kingdom / Shitennou

Folder: `Cores/Sega/`

| Console | Codename | Folder | Status |
|---|---|---|---|
| Master System | **Endymion** | `Cores/Sega/Endymion - Master System/` | Not started — reserved |
| Genesis / Mega Drive | **Beryl** | `Cores/Sega/Beryl - Genesis/` | Not started — reserved |
| Game Gear | **Jadeite** | `Cores/Sega/Jadeite - Game Gear/` | Not started — reserved |
| 32X | **Nephrite** | `Cores/Sega/Nephrite - 32X/` | Not started — reserved |
| Saturn | **Zoisite** | `Cores/Sega/Zoisite - Saturn/` | Not started — reserved |
| Dreamcast | **Kunzite** | `Cores/Sega/Kunzite - Dreamcast/` | Not started — reserved |

**Spelling note:** corrected to the canon anime/manga spelling — Jadeite, Nephrite, Zoisite, Kunzite (not Jadeite/Nepherite/Zoisite/Kuzite) — verified against the Sailor Moon Wiki and the official Naoko Takeuchi character-profile site. Kunzite's name was changed to "Malachite" in the DiC English dub, but Kunzite is the original/canon name and the one used here.

---

## 4. Current status: applied to the codebase

**Done, in two passes.** First: `Cores/Snes/` → `Cores/Nintendo/Venus/`, with namespaces `EmuSen.Memory`/`.Apu`/`.Processor`/`.Video`/`.Controllers` → `EmuSen.Cores.Nintendo.Venus.{Memory,Apu,Processor,Video,Controllers}`, and the two SNES-specific `Debug/` files (`StateDump.cs`, `SnesDebugTarget.cs`) moved from the generic `EmuSen.Debug` namespace to `EmuSen.Cores.Nintendo.Venus.Debug` (with an explicit `using EmuSen.Debug;` added so they can still reach the genuinely core-agnostic types — `IDebugTarget`, `WatchRegistry`, etc. — that live in the real `Debug/` folder and stay in `EmuSen.Debug`). Second: every core folder — `Venus` included — relabeled to `Codename - Console` (`Venus` → `Venus - SNES`, etc.), so the console each codename maps to is visible directly in the folder name rather than only in this doc's tables.

**Folder name vs. C# namespace, a real constraint, not a choice:** the folder is `Venus - SNES/` but the namespace stays the plain `EmuSen.Cores.Nintendo.Venus.*` — C# identifiers can't contain spaces or dashes, so `Venus - SNES` is not legal as a namespace segment. This is the one place folder path and namespace intentionally don't match 1:1; every other segment of the path matches its namespace segment exactly.

Every other planned core's folder (§2, §3) was created alongside the first pass with a placeholder `README.md`, then relabeled in the second pass along with `Venus`, so the full structure — codename *and* console — is visible even though only Venus has real code.

**What deliberately did NOT change:** class names, comments, and display strings describing *real SNES hardware* — `SnesDebugTarget`, `Snes65816Disassembler`, the `CoreName` property returning `"SNES"`, console log lines, hardware-behavior comments citing the SNESdev wiki, etc. The codename `Venus` governs the organizational layer (folder path, C# namespace) only; it does not replace factually-accurate hardware terminology. Renaming those too would reduce technical clarity for no benefit — "Venus65816Disassembler" would obscure that it's a 65816 disassembler, which is the actually-useful fact about that class.

**No dotnet SDK is available in the environment either rename pass was performed from**, so neither could be built/compiled to verify after the fact — same limitation noted elsewhere in the Man pages (see the debugging tools reference's honesty notes on the disassembler and `ffmpeg` integration). Both were done carefully and systematically (moved via `git mv` to preserve history, then verified via repo-wide `grep` that no old namespace/using references or stale unlabeled/pre-rename path comments remained), but **a real build on the project's own dev machine is the first actual verification either has had.** Report back if it doesn't compile clean.
