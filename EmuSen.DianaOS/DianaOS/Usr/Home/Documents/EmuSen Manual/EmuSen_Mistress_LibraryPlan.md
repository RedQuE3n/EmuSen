# Mistress: what it would take to model the library on OpenEmu

*Drafted 2026-09-21 from a reading of OpenEmu's source at `master`. A plan, not a record: nothing here is built.
It should be retired into `EmuSen_Settings_Reference.md` §4, section by section, as it is carried out or abandoned.
The research behind it is cited inline; where a claim is an inference rather than a reading, it says so.*

## 0. What OpenEmu's frontend actually is

Two things, and it is worth separating them because only one of them is worth copying.

**A catalogue with a face.** OpenEmu keeps a Core Data database of games, ROMs, save states, screenshots,
collections and cover images; it identifies each file against a shipped SQLite dataset (OpenVGDB) and pulls a title,
a description and a box image for it. The library window is a sidebar of consoles and collections beside a grid of
cover art. This is most of what a person means when they say a frontend "looks like OpenEmu".

**A deliberately shallow settings model.** Six preference panes, *none* of which contains a core option. Everything
per-system or per-core is reached either by right-clicking the console in the sidebar (default core, and nothing
else) or from the in-game heads-up bar (core, display mode, shader, scale, cheat, disc, audio device). Nothing at
all is persisted per game. Cores are native plugins whose capabilities are a fixed dictionary in their bundle, so
there is no generic core-variable editor by design, and the "display modes" menu is the single sanctioned escape
hatch for a core's own choices.

**Mistress is already ahead of OpenEmu on the second of those** and a long way behind on the first. The graphics
window (§4.26) offers per-console settings that a core declares — internal resolution, antialiasing, the Expansion
Pak — which OpenEmu has no equivalent of at all. Mistress can also be driven end to end from a controller (§4.29),
which OpenEmu cannot. So this is not a plan to become OpenEmu; it is a plan to take the catalogue.

## 1. Where Mistress stands

`RomLibrary.Scan` walks the ROM directory recursively, filters by the extensions the core catalogue declares, and
returns a `RomEntry` whose **title is the filename without its extension** and whose console is decided by that
extension alone. The list is a `LunaList<RomEntry>` under a `FilterBar` with a console facet and a search box
(§4.11, §4.23). There is no art, no description, no favourite, no play count, no collection, and no identity for a
file beyond its path.

Save states are eight numbered slots a game, written beside the ROM's name, with no screenshot, no name and no
record of which build wrote them.

**The thing to notice first:** `EmuSen.Galaxia/Library/Catalogue/` already holds an `ICatalogue` over SQLite whose
`RomEntry` carries `Md5`, `Region`, `Board`, `HeaderTrust`, `PrgBytes`, `ChrBytes` and `IndexedAt`. It was built
for cartridge analysis and **Mistress does not use it**. Any catalogue work here should extend that table rather
than start a second one; `EmuSen_Stack.md` §4 already argues that a catalogue is the strongest remaining candidate
for the database, and this is that candidate arriving.

## 2. The one idea worth taking outright

**Identification strategy as shipped data, not code.** OpenVGDB carries a `systems` table keyed by OpenEmu's own
system identifier, whose columns tell the *client* how to identify a file for that system: `systemhashless` (match
the filename, for arcade), `systemheader` (match a header string), `systemserial` (match a serial), and
`systemheadersizebytes` (how many bytes to skip before hashing). OpenEmu's own code carries no per-console hashing
special case; the policy is in the dataset.

That is exactly the shape EmuSen wants, because the alternative is a `switch` on console in the frontend, and this
project has spent a lot of effort keeping console knowledge inside cores. The rule generalises even if the dataset
does not get used: **a console's identification policy belongs beside its core's declaration, not in Mistress.**
`CoreCatalog` is where it would go.

Two details of OpenEmu's implementation are worth not repeating. Its identification hash is **MD5 only** — the
`crc32` column exists in the model and is commented out in the class, and no CRC32 or SHA-1 is computed anywhere in
the ROM path, although OpenVGDB stores all three. And it hashes **the whole file at import** but **the file minus
its header at lookup**, so a headered ROM's stored hash and its lookup hash are different numbers. That is a trap,
not a design.

## 3. The work, staged

Sizes are for someone who knows this codebase. Each stage is worth having on its own; none depends on a later one.

| # | What | Touches | Size | Depends on |
|---|---|---|---|---|
| 1 | **Favourites and recency.** A favourite flag and a last-played stamp per path, in `appsettings.json` or a small sibling. Y marks a favourite from the pad (EmulationStation's grammar, already half-built in §4.29); the filter bar gains "Favourites" and "Recent" as facets. | `MainWindow.axaml.cs`, `MainWindow.Pad.cs`, `AppSettings` | half a day | — |
| 2 | **Save states as objects.** Give each state a screenshot (the frontend already has the frame it just presented), a timestamp, and the core's name and version. Show them in a strip in the pad menu and a pane in the library. | `MainWindow` state code, a new `SaveStateIndex`, a LunaP tile list | 2 days | — |
| 3 | **The catalogue behind the library.** Point `RomLibrary` at Galaxia's `ICatalogue`: walk on first run, store path, size, MD5 and console, and re-walk only what changed. Keeps the list instant on a large tree and gives every later stage a stable identity for a file. | `EmuSen.Mistress/Library/RomLibrary.cs`, `Galaxia/Library/Catalogue/` | 2–3 days | — |
| 4 | **Cover art, local only.** An `Artwork/` folder beside the state directory, `<md5>.jpg` or `<title>.jpg`, shown in a grid; a placeholder with the title when absent. A grid/list toggle in the view menu and on the pad. | new `LibraryGrid` view, probably a new LunaP tile control | 3–4 days | 3 |
| 5 | **Collections.** User-made lists, plus the fixed ones OpenEmu has (All Games, Save States, Screenshots). A sidebar replaces the console facet. | `MainWindow.axaml`, `SidebarController` equivalent | 2 days | 1 |
| 6 | **An in-game bar for the pointer.** The pad menu of §4.29 already carries the entries; this is the same list as a fading bar for mouse users, with the scale and the console's settings on it. | `MainWindow.Pad.cs` generalised, a new overlay | 1–2 days | — |
| 7 | **Metadata, if wanted at all.** See §4. | — | — | 3 |

**Stage 3 is the keystone** and the one to do first if only one is done. Everything visual above it wants a stable
identity for a file, and the table already exists.

## 4. What not to copy, and why

**Do not copy the import pipeline.** OpenEmu copies ROMs into its own library folder, organises them into
`roms/<System Display Name>/`, gives disc games a directory each, renames on collision, and unlocks and re-locks
files around the copy. EmuSen's working rule is that **the ROM library is read and never written**. A frontend that
moves the user's files is the opposite of that rule, and no part of this plan needs it: a catalogue keyed by path
and hash gets every benefit of the organised tree without touching it.

**Do not build a network metadata fetch without asking.** OpenVGDB is fetched from GitHub releases, and cover art
is an HTTP download of a URL out of that database. Three problems, in order of weight: its provenance is
undocumented — the repository has no licence file and its GitHub licence field is null, and whether its hashes
derive from No-Intro, TOSEC or Redump is unstated; it turns a local, offline emulator into one that talks to the
network about which games a person owns, which is a decision for the user and not for this plan; and the fetch
would be a new dependency in a project that has been careful about those. **Stage 4 is local art on purpose.** If a
fetch is ever wanted, it should be off by default, one explicit action, and clear about what it sends.

**Do not copy the metadata model.** It is thinner than it looks: title, description, cover, a user star rating,
play count, last played, play time, serial, header and MD5. There is **no developer, publisher, year, player count
or region** — the `Genre`, `Credit` and `Contributor` entities exist in the schema and appear never to be populated,
since the lookup does not select them. Region *is* fetched, used only to break a tie between matching rows, and then
explicitly discarded. If EmuSen wants region it must get it elsewhere; Galaxia's catalogue already has a column for
it, filled from the cartridge rather than from a database.

**Do not copy Core Data's migration cost.** Nine model versions, five mapping models, three migration policies and
a third-party iterative migrator, for a library database. `EmuSen_Stack.md` §4.3 already names schema versioning as
the sharpest known gap in this project's storage design; adopting a library database is the moment to fix that, with
one `PRAGMA user_version` and a real migration path, not to import someone else's version history.

**Do not copy the settings model.** It is worse than what Mistress has. A hidden Debug pane unlocked by the Konami
code, no core options anywhere, and a per-core display-mode menu as the only escape hatch, would be a step back
from a graphics window that lists what a core actually declares.

**Do not copy per-game core selection.** OpenEmu's in-game "Select Core" is session-scoped and writes nothing; the
next launch goes back to the per-system default. Either persist it or do not offer it.

## 5. Licensing

OpenEmu has **no repository licence file**; licensing is per source file, and the files carry **BSD 3-Clause**
("Neither the name of the OpenEmu Team nor the names of its contributors may be used to endorse or promote
products derived from this software"). Its cores are separate submodules under their own licences. Nothing here
proposes taking any of its code, and none should be taken: the ideas above are architecture, and the one dataset
worth wanting — OpenVGDB — is the one whose licence is unstated.

## 6. Open questions

- **OpenVGDB's licence, schema and provenance** are unresolved. If stage 7 is ever wanted, this is the first thing
  to settle, and it may settle it negatively.
- **Whether a grid is wanted at all.** Mistress's list shows a title and a console tag and is legible on a handheld
  at 24 points (§4.29). A grid of placeholders, which is what a library with no art would be, is worse than a list.
  Stage 4 is worth doing only if art is actually present, and so it should follow the user's own art folder rather
  than lead it.
- **Whether the sidebar earns its width on a Steam Deck.** OpenEmu's is a desktop layout. Big-screen mode hides the
  menu bar for a reason; a sidebar would take the same space back.
