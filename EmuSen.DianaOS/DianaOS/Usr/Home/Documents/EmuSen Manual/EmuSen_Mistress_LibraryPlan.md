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

**Identification strategy as shipped data, not code.** OpenVGDB carries a `SYSTEMS` table keyed by OpenEmu's own
system identifier, whose columns tell the *client* how to identify a file for that system: `systemHashless` (match
the filename, for arcade), `systemHeader` (match a header string), `systemSerial` (match a serial), and
`systemHeaderSizeBytes` (how many bytes to skip before hashing). The client asks the dataset which of the four to
use and then does it; OpenEmu's own code carries no per-console hashing special case. The whole database is four
tables — `ROMs`, `RELEASES`, `SYSTEMS`, `REGIONS` — and that first idea is the only part of it this plan wants.

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
| 0 | **Continue where you left off.** Write a state when a game is closed or the window shuts, and offer Resume or Restart the next time that game is started, with a "do not ask again" that remembers either answer. Mistress has eight manual slots and nothing automatic, so closing the window today loses the session. This is the cheapest thing on the list and the one a handheld wants most, since a Deck is suspended and resumed rather than quit. | `MainWindow` close path and `LoadRom`, one LunaP dialog | half a day | — |
| 1 | **Favourites and recency.** A favourite flag and a last-played stamp per path, in `appsettings.json` or a small sibling. Y marks a favourite from the pad (EmulationStation's grammar, already half-built in §4.29); the filter bar gains "Favourites" and "Recent" as facets. | `MainWindow.axaml.cs`, `MainWindow.Pad.cs`, `AppSettings` | half a day | — |
| 2 | **Save states as objects.** Give each state a screenshot (the frontend already has the frame it just presented), a timestamp, and **the core's name and its save-state version**. Show them in a strip in the pad menu and a pane in the library. OpenEmu keeps each state as a directory holding the data, a screenshot and a plist naming the core that wrote it, and refuses or switches on a mismatch. **For EmuSen that last part is the point, not the decoration:** Mercury is on save-state version 5, Mars's changed with the Expansion Pak, and a state written by an older build currently loads into whatever it happens to be compatible with. A state that names its writer can say so instead. | `MainWindow` state code, a new `SaveStateIndex`, a LunaP tile list | 2 days | — |
| 3 | **The catalogue behind the library.** Point `RomLibrary` at Galaxia's `ICatalogue`: walk on first run, store path, size, MD5 and console, and re-walk only what changed. Keeps the list instant on a large tree and gives every later stage a stable identity for a file. | `EmuSen.Mistress/Library/RomLibrary.cs`, `Galaxia/Library/Catalogue/` | 2–3 days | — |
| 4 | **Cover art, local only.** An `Artwork/` folder beside the state directory, `<md5>.jpg` or `<title>.jpg`, shown in a grid; a placeholder with the title when absent. A grid/list toggle in the view menu and on the pad. | new `LibraryGrid` view, probably a new LunaP tile control | 3–4 days | 3 |
| 5 | **Collections.** User-made lists, plus the fixed ones OpenEmu has (All Games, Save States, Screenshots). A sidebar replaces the console facet. | `MainWindow.axaml`, `SidebarController` equivalent | 2 days | 1 |
| 6 | **An in-game bar for the pointer.** The pad menu of §4.29 already carries the entries; this is the same list as a fading bar for mouse users, with the scale and the console's settings on it. OpenEmu's is a borderless child window that appears on mouse movement and fades after a second and a half. | `MainWindow.Pad.cs` generalised, a new overlay | 1–2 days | — |
| 6a | **A glyph when something happens.** A small icon flashed over the picture for a state written, a screenshot taken, fast-forward or rewind. On a handheld there is no status bar and no window title to say that a button did anything. | the same overlay as 6 | a few hours | 6 |
| 7 | **Metadata, if wanted at all.** See §4. | — | — | 3 |

**Stage 3 is the keystone** and the one to do first if only one is done. Everything visual above it wants a stable
identity for a file, and the table already exists.

## 4. What not to copy, and why

**Do not copy the import pipeline.** OpenEmu copies ROMs into its own library folder, organises them into
`roms/<System Display Name>/`, gives disc games a directory each, renames on collision, and unlocks and re-locks
files around the copy. EmuSen's working rule is that **the ROM library is read and never written**. A frontend that
moves the user's files is the opposite of that rule, and no part of this plan needs it: a catalogue keyed by path
and hash gets every benefit of the organised tree without touching it.

**Do not build a network metadata fetch without settling the licence first.** OpenVGDB is one SQLite file fetched
from GitHub releases, and cover art is an HTTP download of a URL out of it. Four findings, and the first is close
to disqualifying:

- **It has no licence.** No licence file, a twenty-eight byte README whose entire content is the project's own
  name, an empty wiki, and a `license` field of null. Its issue #43, "License?", was opened in September 2024 by
  someone asking whether they could use it in a GPL application, and has never been answered. Using it is a legal
  judgement, not a settled permission, and for a project whose output is academic that judgement should be made
  deliberately or not at all.
- **The cover art is not its art.** The four cover columns hold **hotlinks to GameFAQs**
  (`gamefaqs1.cbsistatic.com`). A frontend that caches them is redistributing someone else's scans. Two of the
  releases exist only because GameFAQs moved its host and every link broke.
- **It is frozen.** The newest release is v29.0, 11 November 2021. Anything dumped or renamed since is absent, and
  when the image host moves again there will be no upstream fix.
- **Provenance is per row, not per project.** There is a `romDumpSource` column; observed values include `Redump`,
  the maintainer has said in an OpenEmu thread that No-Intro DATs are the source for cartridge systems, and one
  release note credits a MAME 0.149 set for arcade. So the data is derived from the usual DAT projects without
  saying so anywhere in the project itself.

**Stage 4 is local art on purpose.** If a fetch is ever wanted it should be off by default, one explicit action,
clear about what it sends, and preceded by a decision about the licence that this plan does not make.

**Do not copy the metadata model.** What OpenEmu *keeps* is thinner than what it *could* keep: title, description,
cover, a user star rating, play count, last played, play time, serial, header and MD5. There is no developer,
publisher, year, player count or region, and the `Genre`, `Credit` and `Contributor` entities in its schema appear
never to be populated. This is not because the data is unavailable — OpenVGDB's `RELEASES` table carries
`releaseDeveloper`, `releasePublisher`, `releaseGenre`, `releaseDate` and three more cover columns, and OpenEmu's
own query selects none of them. Region *is* fetched, used only to break a tie between matching rows, and then
explicitly discarded. If EmuSen wants region it must get it elsewhere; Galaxia's catalogue already has a column for
it, filled from the cartridge rather than from a database.

**Do not copy Core Data's migration cost.** Nine model versions, five mapping models, three migration policies and
a third-party iterative migrator, for a library database. `EmuSen_Stack.md` §4.3 already names schema versioning as
the sharpest known gap in this project's storage design; adopting a library database is the moment to fix that, with
one `PRAGMA user_version` and a real migration path, not to import someone else's version history.

**Do not copy the settings model.** It is worse than what Mistress has. A hidden Debug pane unlocked by the Konami
code, no core options anywhere, and a per-core display-mode menu as the only escape hatch, would be a step back
from a graphics window that lists what a core actually declares.

**Do not copy the core installer.** OpenEmu downloads cores at run time from an XML feed, per-core Sparkle appcasts and a newer JSON manifest, and offers to install one when a game needs it. EmuSen's cores are built into the assembly and there is no plugin boundary to hang this on; `EmuSen_Assembly_Pruning` has been arguing the other way, towards fewer boundaries rather than more.

**Do not copy per-game core selection.** OpenEmu's in-game "Select Core" is session-scoped and writes nothing; the
next launch goes back to the per-system default. Either persist it or do not offer it.

## 5. Licensing

OpenEmu has **no repository licence file**; licensing is per source file, and the files carry **BSD 3-Clause**
("Neither the name of the OpenEmu Team nor the names of its contributors may be used to endorse or promote
products derived from this software"). Its cores are separate submodules under their own licences. Nothing here
proposes taking any of its code, and none should be taken: the ideas above are architecture, and the one dataset
worth wanting — OpenVGDB — is the one whose licence is unstated.

## 6. Open questions

- **OpenVGDB is settled, and settled badly**: unlicensed, unanswered on the question, frozen since 2021, and
  hotlinking someone else's cover scans (§4). Stage 7 should be read as "probably not, and certainly not without a
  decision taken on purpose". What survives from it is the idea in §2, which needs none of its data.
- **Whether a grid is wanted at all.** Mistress's list shows a title and a console tag and is legible on a handheld
  at 24 points (§4.29). A grid of placeholders, which is what a library with no art would be, is worse than a list.
  Stage 4 is worth doing only if art is actually present, and so it should follow the user's own art folder rather
  than lead it.
- **Whether the sidebar earns its width on a Steam Deck.** OpenEmu's is a desktop layout. Big-screen mode hides the
  menu bar for a reason; a sidebar would take the same space back.

## 7. Carried out (2026-09-21)

**The decision this plan was written against has changed.** The csproj header, `EmuSen_Launcher_Multicore_Gameplan.md` §0 and Path E of `EmuSen_LunaP_Adoption_Gameplan.md` all record that a polished frontend was to be a separate project and that Mistress would stay a bug-testing tool. On 2026-09-21 the user asked for Mistress itself to be modelled on OpenEmu. Those records are left as they were, since they were true when written; this section is where the change is recorded, and the csproj header is updated to point here.

**Stage 0 is built** (`EmuSen_Settings_Reference.md` §4.31, `EmuSen_Galaxia.md` §5.2), together with the first half of stage 2: every state written from the window now has its picture beside it.

**Stage 1's storage went to SQLite, not to a JSON sibling as the table proposed.** The table's "in `appsettings.json` or a small sibling" treated play records as settings. By `EmuSen_Galaxia.md` §7.2's own test (who is the author) they are data. They live in `home/Library/games.db`, which has schema versioning from its first row (§4.32). **Stage 3 is not a prerequisite for the grid after all:** art is matched by file name (`EmuSen_Settings_Reference.md` §4.33), so identity by hash is still worth having for renames but no longer holds up the visual stages.

**The open question in §6 about the grid** is answered by the request itself: the grid is wanted. It is a view beside the list, not a replacement, so the 24-point list that §4.29 made legible on a handheld stays one toggle away.

**Stages 1, 4, 5, 6 and 6a are built** the same day (`EmuSen_Settings_Reference.md` §4.33 to §4.36): the sidebar with Favourites and Recently Played, the cover grid with local art, the save-state and screenshot views, the bar over the game with its notices, and Preferences as OpenEmu's panes. Their four general controls went into LunaP (`docs/LunaP.md` §88), and the first screenshot of the result found that the grid's selection ring had never been painted (§88.11 there).

**Still open:** stage 2's second half (a state that names the core and version that wrote it), stage 3 (the hashed catalogue, now wanted for renames rather than for art), stage 7 (metadata, which §4 argues against), OpenEmu's user-made collections, and a pass on a real handheld and a real screen reader.
