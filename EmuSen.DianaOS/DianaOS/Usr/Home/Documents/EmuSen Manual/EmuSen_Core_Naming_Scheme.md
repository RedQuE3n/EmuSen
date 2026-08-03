# EmuSen — Core Naming Scheme

*(Living document — companion to `EmuSen_Project_Overview_v2.md` and `EmuSen_Core_Gameplan.md` in this same `Man pages` folder. This one is purely naming/organizational convention: what a future core is called and where it lives, not what it does.)*

---

## 1. Where the project name comes from

**EmuSen** = **Emu**lator **Sen**shi, a reference to *Bishoujo Senshi Sailor Moon* ("Pretty Guardian Sailor Moon" — *senshi* 戦士 = "soldier"/"guardian"). Every core is named after a character from that series, grouped by the real-world console manufacturer.

**The organizing principle:** Nintendo — the project's own origin point — gets the heroes (Sailor Guardians). Every other manufacturer gets a villain faction. Dark Kingdom, Black Moon Clan, Death Busters, and Dead Moon Circus are all in use (§3-§6); Shadow Galactica (Sailor Galaxia's Animamates) is the one major faction not yet assigned to a manufacturer — a natural pick whenever another one gets added.

---

## 2. Nintendo consoles → Sailor Guardians

Folder: `Cores/Nintendo/`

### Inner Senshi — mainline handheld/console lineage

| Console | Codename | Folder | Status |
|---|---|---|---|
| NES | **Moon** | `Cores/Nintendo/Moon - NES/` | Active development — `EmuSen.Cores.Nintendo.Moon.*`. CPU validated against SingleStepTests `nes6502/v1` (2,560,000 cases, traces included); PPU, memory, five mappers, `ICore` and `IDebugTarget` all implemented. No audio synthesis, no PAL, scanline-granularity PPU. See `Man pages/Hardware/Nintendo/Moon - NES/` |
| SNES | **Venus** | `Cores/Nintendo/Venus - SNES/` | Active development — `EmuSen.Cores.Nintendo.Venus.*` (see §8) |
| Game Boy / Game Boy Color | **Mercury** | `Cores/Nintendo/Mercury - GB-GBC/` | Not started — reserved. Open question whether GB and GBC are different enough hardware to warrant two separate cores rather than one (undecided; revisit once Venus work is further along and there's a real basis for comparison) |
| Game Boy Advance | **Jupiter** | `Cores/Nintendo/Jupiter - GBA/` | Not started — reserved |
| N64 | **Mars** | `Cores/Nintendo/Mars - N64/` | Not started — reserved |

Sailor Moon herself (codename **Moon**, not the character's full title) anchors the console the project's own name-lineage effectively starts from (NES, the originator generation) — Venus/Mercury/Mars/Jupiter are her fellow Inner Senshi, extending outward to later Nintendo hardware in the order they line up with here.

### Outer Senshi — later/less-conventional hardware

| Console | Codename | Folder | Status |
|---|---|---|---|
| Virtual Boy | **Saturn** | `Cores/Nintendo/Saturn - Virtual Boy/` | Not started — reserved |
| GameCube | **Uranus** | `Cores/Nintendo/Uranus - GameCube/` | Not started — reserved |
| Wii | **Neptune** | `Cores/Nintendo/Neptune - Wii/` | Not started — reserved |
| Wii U | **Pluto** | `Cores/Nintendo/Pluto - Wii U/` | Not started — reserved |

**Saturn/Virtual Boy is a deliberate joke, not a coincidence pairing.** Sailor Saturn is the Guardian of Death and Destruction — a fitting, tongue-in-cheek match for the Virtual Boy, infamous even at launch for headaches, eye strain, and nausea in real players. The rest of the Outer Senshi (Uranus/Neptune/Pluto) fill out later Nintendo hardware in no particular thematic order beyond "the mainline Inner Senshi slots were already full."

**Note on "Saturn" the codename vs. "Saturn" the console:** Sega's own Saturn hardware is codenamed **Zoisite** (§3), not Sailor Saturn — the codename "Saturn" here refers only to the *character*, paired with Nintendo's Virtual Boy. No collision in practice since codenames are scoped per-manufacturer-folder, but worth being explicit about given how easy this is to misread at a glance.

### Guardian cats — handheld companion lineage

| Console | Codename | Folder | Status |
|---|---|---|---|
| Nintendo DS | **Luna** | `Cores/Nintendo/Luna - DS/` | Not started — reserved |
| Nintendo 3DS / New Nintendo 3DS | **Artemis** | `Cores/Nintendo/Artemis - 3DS-New3DS/` | Not started — reserved. Open question whether the 3DS and New 3DS are different enough hardware to warrant two separate cores rather than one (same shape as the GB/GBC question under Mercury — revisit once there's a real basis for comparison) |

**Luna and Artemis aren't Sailor Senshi themselves** — they're the two guardian cats who mentor and advise the Sailor Guardians throughout the series (Luna mentors Sailor Moon directly; Artemis mentors Sailor Venus and later the whole team). Deliberately kept as their own small group rather than folded into Inner or Outer Senshi above, since they're a different kind of character entirely — fitting company for the *handheld* wing of Nintendo's lineup specifically, companions alongside the main console generations the way the DS/3DS families themselves sit alongside (rather than replacing) Nintendo's home consoles.

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

---

## 4. Sony consoles → Black Moon Clan

Folder: `Cores/Sony/`

| Console | Codename | Folder | Status |
|---|---|---|---|
| PlayStation | **Diamond** | `Cores/Sony/Diamond - PlayStation/` | Not started — reserved |
| PlayStation 2 | **Sapphire** | `Cores/Sony/Sapphire - PlayStation 2/` | Not started — reserved |
| PlayStation 3 | **Rubeus** | `Cores/Sony/Rubeus - PlayStation 3/` | Not started — reserved |
| PSP | **Esmeraude** | `Cores/Sony/Esmeraude - PSP/` | Not started — reserved |
| PS Vita | **Wiseman** | `Cores/Sony/Wiseman - PS Vita/` | Not started — reserved |

The Black Moon Clan's five core leaders — Prince Diamond (Demande), his younger brother Sapphire (Saphir), Rubeus, Esmeraude, and the manipulative Wiseman pulling strings behind them — map cleanly onto Sony's five major-era systems. PS4/PS5 aren't assigned yet; the Ayakashi/Specter Sisters (Koan, Berthier, Calaveras, Petz) are the natural next pull from this same faction if/when they're needed.

---

## 5. Atari consoles → Death Busters (Witches 5)

Folder: `Cores/Atari/`

| Console | Codename | Folder | Status |
|---|---|---|---|
| Atari 2600 | **Eudial** | `Cores/Atari/Eudial - Atari 2600/` | Not started — reserved |
| Atari 5200 | **Mimete** | `Cores/Atari/Mimete - Atari 5200/` | Not started — reserved |
| Atari 7800 | **Tellu** | `Cores/Atari/Tellu - Atari 7800/` | Not started — reserved |
| Atari Lynx | **Viluy** | `Cores/Atari/Viluy - Atari Lynx/` | Not started — reserved |
| Atari Jaguar | **Cyprine & Ptilol** | `Cores/Atari/Cyprine & Ptilol - Atari Jaguar/` | Not started — reserved |

The Witches 5 (Eudial, Mimete, Tellu, Viluy, and the linked pair Cyprine/Ptilol) map to Atari's five major systems. Cyprine and Ptilol are canonically inseparable (a conjoined/split-personality pair depending on adaptation), which is why they share one folder/slot rather than being split across Jaguar and Jaguar CD — if a Jaguar CD core is ever wanted separately, it'll need a villain from outside this faction rather than trying to split the pair. Kaolinite and Professor Tomoe (the Witches 5's superiors) are the natural next pull from this faction.

---

## 6. Microsoft consoles → Dead Moon Circus (Amazoness Quartet)

Folder: `Cores/Microsoft/`

| Console | Codename | Folder | Status |
|---|---|---|---|
| Xbox | **CereCere** | `Cores/Microsoft/CereCere - Xbox/` | Not started — reserved |
| Xbox 360 | **JunJun** | `Cores/Microsoft/JunJun - Xbox 360/` | Not started — reserved |
| Xbox One | **PallaPalla** | `Cores/Microsoft/PallaPalla - Xbox One/` | Not started — reserved |
| Xbox Series X\|S | **VesVes** | `Cores/Microsoft/VesVes - Xbox Series/` | Not started — reserved |

The Amazoness Quartet (CereCere, JunJun, PallaPalla, VesVes — each named for an asteroid: Ceres, Juno, Pallas, Vesta) map one-to-one onto the four Xbox hardware generations.

---

## 7. NEC consoles → Dead Moon Circus (Amazon Trio)

Folder: `Cores/NEC/`

| Console | Codename | Folder | Status |
|---|---|---|---|
| PC Engine / TurboGrafx-16 | **Tiger's Eye** | `Cores/NEC/Tigers Eye - PC Engine/` | Not started — reserved |
| PC-FX | **Hawk's Eye** | `Cores/NEC/Hawks Eye - PC-FX/` | Not started — reserved |
| SuperGrafx | **Fish Eye** | `Cores/NEC/Fish Eye - SuperGrafx/` | Not started — reserved |

The Amazon Trio (Dead Moon Circus's other subordinate group, alongside the Amazoness Quartet used for Microsoft in §6) map onto NEC's three systems.

---

## 8. Spelling notes

Every codename in this doc has been checked against the Sailor Moon Wiki (and, for the Shitennou specifically, the official Naoko Takeuchi character-profile site) rather than assumed from memory:

- **Sega's Shitennou:** Jadeite, Nephrite, Zoisite, Kunzite — not Jadeite/Nepherite/Zoisite/Kuzite, an earlier draft's misspelling caught and corrected. Kunzite's name was changed to "Malachite" in the DiC English dub; Kunzite is the original/canon name and the one used here.
- **NEC's Amazon Trio folder names deliberately drop an apostrophe:** canon is "Tiger's Eye" and "Hawk's Eye" (with the possessive apostrophe), folders are `Tigers Eye` / `Hawks Eye` — a pragmatic filesystem/shell-quoting choice, not a spelling correction. The apostrophe is preserved in this doc's prose and in each folder's own `README.md` title.
- **Atari's `Cyprine & Ptilol` folder name uses a literal `&`** rather than "and" or a slash, matching how the pair is most commonly written together as a single unit.
- **Sony's PS Vita codename, "Wiseman" (§4), collides in spelling with `EmuSen.WiseMan`**, the unrelated committed xUnit test project (see `EmuSen_Debugging_Tools_Reference_v5.md` §3.18). Purely coincidental - `EmuSen.WiseMan` predates this doc's PS Vita assignment and was named for its own reasons, not after the Black Moon Clan character. No folder/namespace collision in practice (`Cores/Sony/Wiseman - PS Vita/` vs. the top-level `EmuSen.WiseMan/` project), but worth knowing about if either name ever shows up in a search and looks like it might refer to the other.

---

## 9. Current status: applied to the codebase

**Venus (SNES) — done, in two passes.** First: `Cores/Snes/` → `Cores/Nintendo/Venus/`, with namespaces `EmuSen.Memory`/`.Apu`/`.Processor`/`.Video`/`.Controllers` → `EmuSen.Cores.Nintendo.Venus.{Memory,Apu,Processor,Video,Controllers}`, and the two SNES-specific `Debug/` files (`StateDump.cs`, `SnesDebugTarget.cs`) moved from the generic `EmuSen.Debug` namespace to `EmuSen.Cores.Nintendo.Venus.Debug` (with an explicit `using EmuSen.Debug;` added so they can still reach the genuinely core-agnostic types — `IDebugTarget`, `WatchRegistry`, etc. — that live in the real `Debug/` folder and stay in `EmuSen.Debug`). Second: every core folder — `Venus` included — relabeled to `Codename - Console` (`Venus` → `Venus - SNES`, etc.).

**Folder name vs. C# namespace, a real constraint, not a choice:** the folder is `Venus - SNES/` but the namespace stays the plain `EmuSen.Cores.Nintendo.Venus.*` — C# identifiers can't contain spaces, dashes, or (for Atari's `Cyprine & Ptilol`) ampersands, so none of those are legal namespace segments. This is the one place folder path and namespace intentionally don't match 1:1 — relevant now to every reserved folder in §2-§7 too, not just Venus, whenever any of them gets real code.

**Every other core in this doc (§2's Outer Senshi, §3-§7 entirely) is a reserved placeholder folder with a `README.md` only — no code exists for any of them yet.** They exist so the full naming scheme is visible in the actual repo structure, not just in this doc.

**What deliberately did NOT change for Venus, and won't for future cores either:** class names, comments, and display strings describing *real hardware* — `SnesDebugTarget`, `Snes65816Disassembler`, the `CoreName` property returning `"SNES"`, console log lines, hardware-behavior comments citing the SNESdev wiki, etc. A codename governs the organizational layer (folder path, C# namespace) only; it never replaces factually-accurate hardware terminology. Renaming those too would reduce technical clarity for no benefit — "Venus65816Disassembler" would obscure that it's a 65816 disassembler, which is the actually-useful fact about that class.

**No dotnet SDK is available in the environment any of this was performed from**, so none of it could be built/compiled to verify after the fact — same limitation noted elsewhere in the Man pages (see the debugging tools reference's honesty notes on the disassembler and `ffmpeg` integration). Renames were done carefully and systematically (moved via `git mv` to preserve history, then verified via repo-wide `grep` that no old namespace/using references or stale unlabeled/pre-rename path comments remained), but **a real build on the project's own dev machine is the first actual verification any of this has had.** Report back if it doesn't compile clean.
