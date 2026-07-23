# EmuSen — Core Naming Scheme

*(Living document — companion to `EmuSen_Project_Overview` and `EmuSen_Core_Gameplan` in this same `Man pages` folder. This one is purely naming/organizational convention: what a future core is called and where it lives, not what it does.)*

---

## 1. Where the project name comes from

**EmuSen** = **Emu**lator **Sen**shi, a reference to *Bishoujo Senshi Sailor Moon* ("Pretty Guardian Sailor Moon" — *senshi* 戦士 = "soldier"/"guardian"). Every core is named after a character from that series, grouped by the real-world console manufacturer.

---

## 2. Nintendo consoles → Sailor Guardians (Inner Senshi)

Folder: `Cores/Nintendo/`

| Console | Codename | Status |
|---|---|---|
| NES | **Sailor Moon** | Not started |
| SNES | **Venus** | In progress — currently `Cores/Snes/`, not yet moved/renamed (see §4) |
| Game Boy / Game Boy Color | **Mercury** | Not started — open question whether GB and GBC are different enough hardware to warrant two separate cores rather than one (undecided; revisit once SNES work is further along and there's a real basis for comparison) |
| Game Boy Advance | **Jupiter** | Not started |
| N64 | **Mars** | Not started |

Sailor Moon herself anchors the console the project's own name-lineage effectively starts from (NES, the originator generation) — Venus/Mercury/Mars/Jupiter are her fellow Inner Senshi, extending outward to later Nintendo hardware in the order they line up with here.

---

## 3. Sega consoles → Dark Kingdom / Shitennou

Folder: `Cores/Sega/`

| Console | Codename | Status |
|---|---|---|
| Master System | **Endymion** | Not started |
| Genesis / Mega Drive | **Beryl** | Not started |
| Game Gear | **Jadeite** | Not started |
| 32X | **Nephrite** | Not started |
| Saturn | **Zoisite** | Not started |
| Dreamcast | **Kunzite** | Not started |

**Spelling note:** corrected to the canon anime/manga spelling — Jadeite, Nephrite, Zoisite, Kunzite (not Jadeite/Nepherite/Zoisite/Kuzite) — verified against the Sailor Moon Wiki and the official Naoko Takeuchi character-profile site. Kunzite's name was changed to "Malachite" in the DiC English dub, but Kunzite is the original/canon name and the one used here.

---

## 4. Current status: not yet applied to the codebase

As of this doc, **no rename has happened yet.** The SNES core still lives at `Cores/Snes/` with `EmuSen.Cores.Snes.*`-shaped... actually still-generic namespaces (`EmuSen.Memory`, `EmuSen.Processor`, etc. — see `EmuSen_Project_Overview`'s §7 TODO on this pre-existing, separate namespace-catchup item). This naming scheme is recorded here so it's not lost, and applied at whichever of these turns out to be the natural moment:

- The existing planned "namespace catchup" pass (`EmuSen.Memory` → `EmuSen.Cores.Snes.Memory`, already noted as a deliberately-deferred drive-by cleanup) — do it once, correctly, as `EmuSen.Cores.Nintendo.Venus.*` instead of `EmuSen.Cores.Snes.*`, rather than renaming twice.
- Whenever a second core actually starts (the Multicore Gameplan's Phase 1 trigger) — `Cores/Snes/` → `Cores/Nintendo/Venus/` naturally happens alongside creating the new core's own manufacturer/codename folder, so both cores land in the right structure from day one instead of one needing a follow-up move.

**Until then:** every other Man pages doc, code comment, and console log line correctly refers to the current core as "SNES" / `Cores/Snes/` / `EmuSen.Cores.Snes.*`(-to-be) — that's not stale, it's just pre-rename. Don't "helpfully" start calling it Venus in code or docs before the actual rename happens, since a name used in some places and not others is worse than consistently using the old one until the real move.
