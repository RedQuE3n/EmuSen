# EmuSen_Testing — the testing studio

How EmuSen is tested: which units a change can affect, how they run, and what the record says. The studio is one
DianaSh command module, `emusen.testing` (crate `Platform/emusen-testing-module`), linked into the standalone `dianash`
program beside `emusen.dev` and never into the platform library, with one SQLite store and three data files. Its
commands are `units`, `impact`, `coverage`, `trial`, `results`, `ingest` and `queue`; each has a man page under
`EmuSen.DianaSh/DianaSh/Lib/dianash/emusen.testing/man/`.

Decided 2026-11-01 by the tester, with the six decisions of §8. Phase 1 (the store, the runner, impact v1) was built on
2026-11-01; phases 2–4 are §9.

## 1. Why

The review of 2026-11-01 (157 land records, 143 dotnet logs, 805 cargo logs, two full-suite runs) found that lands were
slow for three reasons: half of WiseMan's tests run on one thread; every Sapphire land relinked a 40,000-line crate with
fat LTO; and the hand-kept `blast-map.txt` (twelve of the tools repository's thirty commits were lines learned from
misses) widened 17 of 60 merges to the 2,798 tests of the Mistress namespace. Its items A–C (the `test-fast` profile,
the nightly trait, TRX and times) landed as `testing-speed` (`EmuSen_RustPlatform.md` §37.6,
`EmuSen_Debugging_Tools_Reference_v5.md` §3.68). The studio is where those items live from now on, and what replaces
the map with a derived graph.

## 2. The shape

| Job | Command | Surface |
|---|---|---|
| know the units | `units` | one line per unit, from the build graph and the test project's sources |
| choose the units | `impact` | `kind TAB unit TAB reason TAB via`, one unit a line |
| run them | `trial` | a selection in; a line per unit and a verdict out; result files under `~/.cache/emusen/testing/trials/<n>/` |
| keep the record | `results`, `ingest` | the store, `~/.local/share/EmuSen/testing/studio.db`; JSON lines by `results export` |
| share the machine | `queue` | lock files under `~/.cache/emusen/testing/queue/`; a command prefix for any heavy job |

The data files carry a format number each, and a reader ignores a field it does not know: `Testing/inputs.json`,
`Testing/tiers.json` and `~/.config/EmuSen/testing/runners.json`. The schema is `Testing/schema/001-studio.sql`.
The selection line and the verdict line are the two text formats; a change to either is a minor version of the module.

**Why a module of its own.** `emusen.dev` holds two commands with no state. The studio has a store, a queue and adapters
to runners; folded in, it would give `emusen.dev` a second job and couple its commands to SQLite. It needs no service,
so it runs in the standalone shell on a machine with no emulator.

## 3. The components

### 3.1 The catalogue

A **cargo** unit is a Cargo root: a crate outside any workspace, or a workspace counted once (Platform), whose member
crates are its sub-units. A **dotnet** unit is a test class of a test project (a project referencing xunit): a
non-abstract class with `[Fact]` or `[Theory]` tests of its own (any attribute ending in those names, such as
`[UinputTheory]`), merged across partial files, or one inheriting an abstract class's tests, as
`MoonDebugTargetContractOnMoonRt` does. `lunap` is the sibling toolkit's suite. On 2026-11-01 the catalogue held 53
cargo roots and 552 classes; against the `dotnet test --list-tests` listing of the review (549 classes), every listed
class was found, and the three more were classes added since.

A unit's **key** is git's hash of its inputs: the object name of every folder it is built from (its own, the crates and
projects in its forward closure, its workspace's `Cargo.toml` and `Cargo.lock`, the declared extras of
`inputs.json`), the toolchain (`rustc -V`, `dotnet --version`), the profile, `~/.cargo/config.toml` for a cargo unit
and the LunaP checkout's commit for a dotnet one. A unit with an uncommitted input has no key, so nothing is reused for
it. The extras are `library_key.py`'s `EXTRA` table, so the studio's key and CI's agree on what an input is; phase 2
makes `units --key` the one owner.

### 3.2 The graph and `impact`

The graph is read from git objects at the head revision, so a branch that is not checked out is read as it is: every
`Cargo.toml` (package name, workspace members, path dependencies in any dependency table, inherited workspace
dependencies), every `.csproj`, `.props` and `.targets` (references, imports, and any property naming a crate's folder,
which is how `RustCores.props` builds a core into `EmuSen.csproj`), `Platform.targets` building `emusen-platform`, and
the root `Directory.Build.*` files as inputs of every project. A workspace member depends on its workspace's folder,
which stands for the shared manifest and lock. The cargo half is held to `cargo metadata --no-deps` by a test: on
2026-11-01 the two agreed on every path dependency of the Platform workspace.

Reading `cargo metadata` itself was rejected for impact because land.sh asks before the merge, about a branch that is
not checked out; git objects describe that branch, the working tree does not.

The rules, the first reason for a unit kept:

1. **recording**: a changed recording selects its owner (a `recording` line of `inputs.json`), or, since 2026-11-03,
   when the owner is mapped by case (§3.7) and the map has the group the recording's name keys
   (`BigPicture-MainMenu-1280x800.spec` to `WindowFitAuditTests#BigPicture-MainMenu`), that group alone: two such
   specs had run the whole audit, 17.8 minutes, on 2026-11-02. A file that ends as a recording does (`.spec`,
   `.golden`, `.frame`, `.framesum`) and that no line owns selects the tier's whole set. This is the rule whose
   absence let a `.spec`-only commit select nothing on 2026-11-01.
2. **always**: a test class's own file selects the class; a unit that failed in the branch's last trial is selected
   again; a tier's `always` list.
3. **declared**: a `folder` line adds its units. Its select entries are WiseMan filter terms (a class is chosen when
   its full name holds the term, as `FullyQualifiedName~term` chose it), `cargo:<crate>`, `refs:<pathspec>` (the test
   files naming a type declared there, directly or through a fixture), `covered:<pathspec>` (since 2026-11-02: the
   classes the coverage map saw execute the files the pathspec names, plus the classes without a report; without a
   map it adds nothing and narrows nothing) and `lunap`. For a file the covered rule answers, only its `cargo:` and
   `lunap` entries apply: coverage says nothing about those units, and everything about the classes.
   Since 2026-11-02 a line matching the changed file (its own folder) narrows; a line on another crate the closure
   reaches adds that crate's units, and such lines narrow only when every crate through which the closure enters
   C# (an edge to a project, `RustCores.props` or `Platform.targets`) has one, since a line on an entry crate
   describes all of that crate's C# consumers and a line elsewhere does not. Before, any one reached crate's line
   narrowed the whole closure: a change to `Platform/serenity` (the ring) reached WiseMan through `emusen-platform`,
   with no line, and through `emusen-libretro-host`, whose line names `LibretroHostTests`; that one line narrowed
   the selection to `LibretroHostTests`, and nothing drawing the device picture ran. The other way round, a change
   to `emusen-gpu` (its `Cargo.lock`, land 49bd9f604) enters C# only through the cores that use it, each with a
   line, so it is narrowed to their classes rather than widened to the whole suite.

   `covered:` is how a Rust crate's C# surface, which coverlet cannot see from the Rust side, enters the map: the
   crate's `extern "C"` exports are named in C# by a few binding files (`git grep` for the symbol prefix finds them,
   and the line's reason records the search), and the classes that executed those files are the ones that call the
   crate. The mapping is declared, not derived: C# binds the platform library by symbol name through
   `NativeLibrary`, never by `DllImport("<library>")`, so no attribute names the crate. A check that every crate built
   into a C# host has such a line, by the exports its sources declare, is left for later.
4. **covered** (since 2026-11-02, §3.7): a changed `.cs` file the coverage map has seen selects the classes that
   executed it, and every class the map holds no report for. A `.cs` file the change adds (decided 2026-11-02 by
   the tester, after four of ten lands widened to the suite for a new file) selects the classes that executed the
   changed, map-seen files that name its types, directly or through other added files: new code is reached only
   through the edits that call it, which the map knows, or through a new test class, which selects itself. An added
   file nothing names, a file never executed by any test, or a file that is not C# (`.axaml`, JSON tables,
   resources, `.fs`) falls to the rules below. The map is used only when its revision is at most one day older than
   the change's merge base (`impact --map-age`); an older map is said on standard error and the graph answers as
   before. The toolkit's own changes can count (built 2026-11-03, off by default): with `--lunap-base REV` impact
   diffs the sibling LunaP checkout that `LunaP.props` names and each file enters as `lunap:<path>`, the name the
   map records the toolkit's 170 files under; a `lunap:` file the map has seen selects the units that executed it,
   and one it has never seen is claimed by no rule and closes to the tier's whole set. A LunaP change is otherwise
   invisible to impact: the flush-help-band land (LunaP c37edcc, `MenuPanel.cs`) went green with `EsdeMenusTests`,
   which measures that band and had executed the file, never selected; it failed in the next land. Held on
   2026-11-03 by the tester while LunaP's move into this repository (a subtree) is decided, which would make the
   sibling diff unnecessary; the replay case for c37edcc stands in §6 for whichever way it goes.
5. **graph**: the file's owner and its reverse closure. Every crate reached is selected. A test project reached is
   narrowed to the units of the lines matching the file, or matching a reached crate's folder; a WiseMan file that is
   not a test class is narrowed to the classes naming its types; a project no line narrows is selected whole. This is
   the over-selection the design expected: where the map has not seen a file, the folder lines stand in for it.
6. **closed**: a file no rule claims selects the tier's whole set. A `none` line (documentation, ignore lists, CI
   files, the referee tools) claims a file and selects nothing.

The tier then removes its excluded tags (`nightly`) and keeps its `named_only` classes (WindowFitAuditTests at land)
only where a line names them, as blast.sh's filter did, or where the covered rule saw them execute the file: that is
evidence, and the audit runs for every screen it draws.

### 3.3 Tiers

`Testing/tiers.json`: `fast` (depth 1, reuse), `land` (the closure less the nightly tag, reuse), `nightly` (all, no
reuse, `test-checked`), `games` and `hardware` (phase 2). Only `fast` and `land` are run by phase 1.

### 3.4 `trial`

Each cargo root runs once, its selected crates as `-p` arguments, through `cargo nextest run` on the tier's profile
where the root defines it (`test-fast` at land) and `release` otherwise, with a generated nextest profile: one retry for
every test, two for a quarantined one, no fail-fast, JUnit. Each test project runs once: `dotnet build`, then
`dotnet test --no-build` with the selected classes as the filter (only `Category!=nightly` when every class is chosen)
and a TRX logger; up to ten failed tests are rerun once by name, and a pass then records the test as flaky. Steps run
concurrently in the queue's CPU slots (two on the desktop), each in a capped scope (`CPUWeight=20`, `MemoryMax=10G` or
`EMUSEN_TESTING_MEMORY_MAX`, `MemorySwapMax=2G`).

**Reuse.** At a tier with `reuse` true, a unit whose key matches a green, not-reused result on the same runner within
`reuse_days` (7), for a unit with no flaky result in 30 days, is not run; the verdict says `reused` and names the
trial. The nightly never reuses, so no reused result is more than a week from a fresh one.

**The policy.** A failed test fails its unit. A flaky test (failed, then passed on a rerun) fails it too unless it is
quarantined by name; a quarantine has an owner, a cause and an expiry of at most 14 days, and a renewal needs a new
cause. nextest's own default, which counts a flaky test as a pass, never decides: the studio judges each test's row.

**Budget.** A trial runs in the foreground past the shell's ten-second budget, which it does not poll
(`EmuSen_DianaSh.md` §6.6): the first real dotnet trial, on 2026-11-01, was ended at ten seconds by the budget before
this was settled. Ctrl-C at a terminal reaches its steps' processes. Listing it under `ps` and ending it with `kill`, as
a job, is phase 2.

The verdict is printed before the store is written; a store failure is reported and never changes it, so a broken
database neither blocks a push nor fakes a green.

### 3.5 The store

One SQLite file opened through the `Platform/sqlite` crate (SQLite as the process has it, else `EMUSEN_SQLITE`, the
shipped `e_sqlite3` from the NuGet cache, or the system's), WAL, `foreign_keys = ON`, the schema applied forward by
`user_version`. Tables: `runner`, `unit`, `build`, `trial`, `selection` (why each unit ran, its key, the trial a reused
one came from), `unit_run` (a unit's outcome, counts and times in a trial: the row a log without per-test lines still
gives), `result` (per test, with attempts), `recording`, `coverage`, `miss`, `quarantine`, `queue`, and the view
`flaky`. `trial.source` names the file a backfilled trial came from, so a second backfill adds only what is new.

A test is named by its path within its binary or class; two binaries of one crate with a test of the same path keep the
first row. Backfilled cargo rows and nextest's rows are named alike.

### 3.6 The queue

Slots per class as lock files held with flock(2): `trial` takes a CPU slot per step; `queue ask cpu --gb 8` waits for a
free slot and prints `flock <slot> systemd-run --user --scope …`, so the job holds the slot while it runs.
Decided 2026-11-01: heavy jobs take their slots this way once this phase lands; until then they are agreed by hand. When 8bb-lease
arrives, the one function that takes a slot calls `lease run` instead.

### 3.7 The coverage map and `coverage`

Built 2026-11-02, after a menu change landed through 285 classes in 34 minutes (Moon's goldens, MarsRT's ABI and
Serenity's filters among them): the graph and the folder lines work at the level of a project or a folder, and any
change under `EmuSen.Mistress`, the tools or LunaP reaches `EmuSen.WiseMan` whole. The map works at the level of a
file: for each test class, the source files its tests executed, so a changed file selects the classes that ran it and
nothing else.

**The collector.** Three were weighed. Microsoft.CodeCoverage's collector writes a binary `.coverage` per run and
instruments dynamically, so one run says which files the run executed, not which class did. AltCover's call-context
tracking attributes each visited point to the test method on the stack, in one run; but half of WiseMan's tests hand
their work to the Avalonia headless dispatcher's thread, where no test method is on the stack, so the attribution
would lose exactly the windows the map is for. coverlet's collector, which WiseMan already references
(`coverlet.collector` 6.0.4), writes a cobertura report per `dotnet test` process; running each class in its own
process makes the attribution exact by construction, dispatcher threads included. That is the one chosen: the
simplest that is correct. Its cost is the fixed price of a process: instrumenting the 17 EmuSen assemblies and writing
a 34 MB report, measured on 2026-11-02 at 12 s for a class that runs bare in 2 s.

**The run.** `coverage run --bin EmuSen.WiseMan/bin/Debug/net10.0 --jobs 4` takes every class of the land tier
(`units` less the nightly tag; the nightly runs its classes whole and needs no map) through
`dotnet test <dll> --no-build --filter "(FullyQualifiedName~C.|FullyQualifiedName~C+)&Category!=nightly"
--collect:"XPlat Code Coverage"` with the runsettings of `coverage.rs` (`[EmuSen*]*` included, the test assembly
included, auto-properties skipped). coverlet rewrites the instrumented assemblies in place and restores them at the
end of the session, so processes cannot share a build folder: each worker gets a full copy beside it
(`bin/coverage-N/net10.0`), never a hard link (decided 2026-11-02). The first run that day hard-linked every file
but the `EmuSen*` assemblies, and coverlet, which matches its `[EmuSen*]*` filter without regard to case, rewrote
`emusen-shell.dll` (the `EmuSen.Shell` project's assembly) through an inode four workers shared with the build they
were mirrored from: one class lost its report to the lock (`emusen-shell.pdb` "used by another process"), the build's
copy was found altered and was restored from `obj/` under a fresh inode; that map holds no row for an `EmuSen.Shell/`
file (no class executes the shell program in its own process), so no attribution was lost. A build also rewrites the
rest of its output in place, which would change a linked copy mid-run; so, copies. A class whose process ends without
a report is run once more after five seconds, for the same lock. The copies sit under the test project, so a test
that walks up from its binaries to `EmuSen.sln` still finds the tree. Each process runs in the project's capped scope at CPU weight 20. A report is read as it arrives
(a file is executed when any of its lines has hits above zero), reduced to `covered` lines and deleted; the run
prints plain lines, one a line, that `coverage load` reads from a file or a pipe, so the nightly's run and its
loading are two steps that compose.

**By case.** `WindowFitAuditTests` is 981 cases (a view at two sizes, a recording read back) and 17 of the pausecard
land's 28 minutes; selected as a class it runs them all for any screen. Its 28 methods are no help, since the three
largest (`A_recording_reads_back_whole_and_holds_to_the_rules`, 477 cases; `Every_audited_window_state_is_as_its_
specification_records_it` and `Nothing_in_the_window_is_cut_off_past_its_panel_or_drawn_over_anything_else`, 164
each) cover every window. So `coverage run --by-case WindowFitAuditTests` (decided 2026-11-02 with the tester) maps
the class one group of cases at a time: the cases are listed (`dotnet test --list-tests`, theories enumerated at
discovery for that copy) and grouped by the first quoted argument of each display name less a trailing size
(`"PauseCard"`, `"PauseCard-1280x800.spec"` and `view: "PauseCard"` are one group; 581 groups and one more for the
37 cases with no quoted argument), each group a unit of its own, `Class#Key`, run by the filter
`(FullyQualifiedName~Class.&(DisplayName~"Key"|DisplayName~"Key-))` and read against a second baseline taken with
theories enumerated at discovery. A change to `MainWindow.PauseCard.cs` then selects the pause card's groups and
no other window's. The groups' run is merged into the whole map (`coverage load --merge`), which keeps one revision.
`PadSettingsWindowTests` (219 s, 17 methods) was left whole: every method drives the one window, so no split would
let a change skip any of them. Measured on 2026-11-02: the class's 272 groups in 22.5 minutes over four processes
(85 process-minutes: the class's 17 of cases and about 12 s of instrumentation per group), every group with a
report, 33,214 rows; the second baseline 227 files and 1,999 lines against the first's 67 and 455, which is the
theories' data evaluated at discovery, counted for nobody. Merged, the map holds 796 units; a class picked whole
(its own file changed) drops its groups from the selection, since it runs them all. What the groups buy, predicted
from the cases' durations in the land dc6bc0a67's TRX, one process: a change to `CoresWindow.cs` alone runs 10
units (5 classes, 5 groups) in 1.5 minutes where the audit whole made it 16.1; `ScraperPane.cs` 9 units in 1.4
against 16.7. What they cannot: `MainWindow.PauseCard.cs` is executed by 131 of the 272 groups, and those hold all
15.4 of the audit's minutes, since every view that opens the main window builds the pause card (`HomeCard.cs` the
same: 224 units, 25.4 minutes); a change inside the pause card's own drawing still runs the views that merely build
it. The lever beyond is a map by line: the hunks a change modifies against the lines each unit executed beyond the
baseline, with an insertion-only hunk (a new override the framework calls without a changed call site) still
claiming its whole file. The rows would be lines rather than files, some millions; not built on 2026-11-02,
proposed instead.

### 3.8 `shard`: the selection in four processes

WiseMan's Avalonia headless classes share one dispatcher per process, so 49 test-minutes took 34 wall minutes. Since
2026-11-02 land.sh deals the selection's dotnet units to four `dotnet test --no-build` processes (`shard`, the
units' seconds from the newest trial that recorded them, a land's or the map's; the longest item first into the
least loaded shard), runs them at once with a log and a TRX each (`dotnet-N.trx`, which the backfill reads together),
and sums their summary lines; `--dry-run` prints each shard's predicted minutes and the wall, both ways. Measured on
the pausecard land's selection (96 classes, 26.4 minutes in one process): 90 of them, 25.4 minutes, are in the
"Process globals" collection (294 of the 526 land classes carry it; 32.6 of the last whole land's 49.1 minutes), so
kept together that shard would be the wall, 25 minutes, and spread the prediction is 6.6. Decided 2026-11-02 by the
tester: the collection guards state inside a process (config, SDL, firmware: `EmuSen_Galaxia.md` §3.14), which
separate processes do not share, so its classes are spread (`EMUSEN_KEEP=1` keeps them together); what separate
processes do share is the machine, so the test project's files that show machine-wide state are found by a scan
(`shard --machine`: a virtual pad or uinput, a listener or a literal port, a fixed path under the home or `out/`
that the file writes, a lock on a fixed file, the GPU facts) and, since 2026-11-03, a unit is machine-global when
the map saw it execute such a file (a class without rows, when its own files are such); those units are dealt as
one item to one shard. `FixedRunFolder` was in the first scan as a lock and is not one: it claims the next free slot
(`/tmp/<name>/NNN.lock`, `FileShare.None`) per process, which is what lets the audit's groups run side by side; with
it counted, every one of the audit's 272 groups sat in the machine shard and the first four sharded lands ran 930 to
967 s against 5 to 6 minutes for the other shards. The land's own times go back to the store after each land
(`ingest --backfill` at the end of land.sh, a row per class and, for a class mapped by case, per group), so the
shards are dealt by real times rather than the map's, which run about 2.4 times longer for the audit. The two
coverage runs had already run all of the collection's classes four processes at a time: none failed the second
time, and the one failure the first time (`CoresWindowTests`, one of ten) never recurred in three runs of the class
alone under coverage nor bare; that run was the hard-linked one, in which other workers' sessions were rewriting
`emusen-shell.dll` under it, which the real copies ended. The first sharded land is the check: a spread class that
fails there and passes alone is one the scan missed, and its cause goes into the scan.

**Paths.** The pdbs carry the paths of the tree the build was made in (`--tree`, the repository by default); a path
under it is recorded relative to it, and a path under the LunaP checkout named by that tree's `LunaP.props` is
recorded as `lunap:<path relative to the checkout>`, so the map holds the toolkit's files for the day a change there
is asked about. Any other path is printed as a `# outside` line and dropped.

**The store.** A loaded run is one `trial` of tier `coverage` whose `tree` is the run's revision and whose verdict
`MAP` marks it complete; its `coverage` rows (`unit`, `file`, `recorded`) replace the previous map's in one
transaction, since the table's key is (unit, file) and a stale row would otherwise keep an older trial's number and
vanish from the new map. A class whose process ended without a report is recorded under the file `*`, and the covered
rule selects it for every covered change: a lost report widens the selection, never narrows it. The map is used when
its revision is at most one day older than the change's merge base; the nightly rebuilds it from its own tree's
build (`nightly.sh`, after the whole suite), so a land on the day after a nightly is covered and a land after a
missed nightly falls back to the graph.

**Discovery.** xunit discovers every test of the assembly before the filter applies. The first map of 2026-11-02
showed 227 files executed by every class: xunit v2 pre-enumerates theories at discovery, evaluating every
`MemberData` source, and the static constructors those reach (one for all of `MainWindow`'s partial files), so a
land that touched any theory's data file or any `MainWindow` partial selected the whole suite (the pausecard land,
525 classes). Two measures, decided 2026-11-02 with the tester: each copy gets an `xunit.runner.json` with
`preEnumerateTheories: false`, so a theory's data is evaluated when the class runs and counts for that class (227
files fell to 67, 455 lines: the test attributes' constructors, the GPU, uinput and shell probes, and what they
touch); and the run opens with a discovery baseline, two runs whose filter matches no test, the least hit count of
each line of each class block kept, and a file counts as executed by a class only where a line's hits exceed that
floor. Nothing is dropped outright: a line discovery hits that a class's tests hit again rises above the floor and
counts; a line discovery alone hits counts for nobody, since a change there breaks discovery for every class
alike and any land shows it. Hit counts are compared by file, class block and line, so a closure's block cannot
hide behind its method's. Noise in discovery can only widen: the floor is the least of two runs. The map sees
nothing in `.axaml`, JSON tables or resources: those keep the folder lines and the graph.

**Measured on 2026-11-02** (desktop, 4 workers, the land tier's 526 classes from the dc6bc0a67 build, the main tree's
assemblies at the land's revision). The first map, before the discovery measures and from hard-linked copies: 62
minutes wall, 178 class-minutes summed (a mean of 20 s per class, a median of 15 s, WindowFitAuditTests 17.5 min
against 15.4 bare, PadSettingsWindowTests 4.0 against 3.7); 524 classes gave a report, one lost it to the lock
above, one failed a test under coverage (`CoresWindowTests`, one of ten) and still reported; 154,530 rows over 1,470
files, 227 of them executed by every class. The second map, with pre-enumeration off, the baseline and full copies,
by `coverage run` itself: 43 minutes wall, 165 class-minutes, 525 classes with a report and none without (the retry
caught the lock), no failure under coverage; 45,939 rows over 1,467 files, 170 of them LunaP's; no file executed by
every class, one by nine tenths, 549 by one class only; `ConfigStore.cs` by 305 classes rather than all 524. What a
file selects: `MainWindow.PauseCard.cs` 89 classes, `CoresWindow.cs` 6, the broker's `CoreLibrary.cs` 155,
`SerenityNative.cs` 31, `PlatformRing.cs` none (no land class drives the ring's C# side). The tree the map is built
in needs `TestRoms/` for the classes that read it (`CommercialRomLibrary`, `HardwareTestRomLibrary`,
`N64TestRomLibrary`), or their commercial-ROM cases skip and the files only those cases execute go unseen: both maps
were built from a checkout without it, and the first run's counts, compared with the land dc6bc0a67 (285 classes in
both), showed one class short, `MercuryCommercialRomBatteryTests` at 1 of its 7 cases; it is loaded as a class
without a report, selected for every covered change, until a tree with `TestRoms/` builds the map. (The second
run's `ran` lines carried no counts, a slip in the runner's reading of the console line, fixed after the run.)

## 4. The data files

### 4.1 `Testing/tiers.json` — §3.3.

### 4.2 `Testing/inputs.json`

What no graph shows, one line per pattern with a dated reason. Its first 73 lines are `blast-map.txt`'s, each dated
by the tools repository's commit that wrote it (`git blame`) and given that commit's subject or the map's own dated
comment; twelve are recordings and two are nobody's input. Added on 2026-11-01: documentation (`\.md$`) and ignore
lists as nobody's input; any `.csproj` selecting `LeafAssemblyTests`; `LunaP.props` selecting `lunap`; `Testing/`
selecting the studio's own crate; and `EmuSen/Cores/Libretro/` selecting `LibretroHostTests`, from the replay of §6.
A term that names no class is reported on standard error: on 2026-11-01, `CrystalTwinScript` (in the map's
`^Platform/crystal` line) named none, and was dropped from `inputs.json` the same day.

`blast-map.txt` is deleted when a month of the nightly's `miss` rows shows nothing the map would have caught that impact
did not (phase 3). Since 2026-11-02 the folder lines narrow a project only where the coverage map has not seen the
changed file (§3.2, rule 3). Added 2026-11-02, for the first Rust-only change the map could not see (padloss-hang,
8b9c2c4c1, the ring and the broker): `^Platform/serenity/` selecting `covered:EmuSen.Serenity/Native/*.cs`,
`covered:EmuSen/Crystal/PlatformRing.cs` and `covered:EmuSen/Crystal/PlatformLend.cs`, the three files that name
`emusen_serenity_*` and `emusen_crystal_picture_*`; and `^EmuSen/Cores/Shared/emusen-core-broker/` selecting
`covered:EmuSen/Cores/Native/*.cs`, the two files that name `emusen_core_*`.

## 5. The backfill

`ingest --backfill`, run on 2026-11-01 over `~/.cache/emusen/tmp`, in 3.0 s: **155 trials** (129 green, 26 red),
**987 unit runs**, **87,354 test rows**, from 149 land folders (818 cargo logs, 147 dotnet logs), 38 transcripts (32
named a land folder's branch; 6 became trials of their own, their folders gone) and the two full-suite logs (47 m 08 s
green, 51 m 36 s red). Every cargo log's crate was matched to a crate of HEAD. Folders written in the last hour without
a `.ok` file are lands still running, left for the next backfill. Dotnet steps without TRX are recorded as one unit,
`dotnet:EmuSen.WiseMan`, with their counts and failed tests; per-class rows begin with the TRX files land.sh writes
since `testing-speed`.

## 6. impact beside blast.sh

**Since 2026-11-02 land.sh decides the dotnet tier by impact** (`impact --format blast`, with the covered rule of
§3.7), and by blast.sh's filter only when impact reports a file as closed (no rule claims it), where blast.sh's map
may still hold a line for it; `--also` terms go to impact. The cargo crates still come from blast.sh, and blast.sh's
unmapped-file stop stays. The comparison below is logged for one more week (`beside/`), after which blast.sh's dotnet
half goes. The replay with the covered rule is at the end of this section.

Until then (2026-11-01), land.sh ran impact beside blast.sh, logged both answers, and still decided by blast.sh's
(`~/.cache/emusen/tmp/studio-p1/land-beside.patch` in the tools repository's terms; each land's comparison goes to
`~/.cache/emusen/testing/beside/`, a line per land to its `ledger.txt`). Every disagreement is read and becomes an
`inputs.json` line or an impact fix. Then land.sh's blast, cargo loop and dotnet step become `impact | trial`
(`land.patch`), the `.ok` file kept for push.sh.

**The replay, 2026-11-01.** The last 60 first-parent merges of WiseMan (to `fe1360d70`), each as `blast.sh M^1 M^2`
with today's map and `impact --base M^1 --head M^2 --blast`, with today's `inputs.json`:

- **blast only: 0 units in 60 merges.** impact never selected less than blast.sh.
- 31 merges identical; 29 where impact selected more (6,159 units agreed, 4,573 impact only).
- 12 merges widened to all of WiseMan: 11 through a Platform crate that reaches the platform library, and so every C#
  test, with no folder line to narrow it (`emusen-library-module` twice, `platform` twice, `dianash-text` three
  times, `core-tool` twice, `core-entry`, `emusen-scrape-module`), and one through a WiseMan helper no test class
  names (`Mistress/NoToolCounts.cs`, which blast.sh left unmapped). These are the disagreements to read first: each
  either gets a folder line or is the graph telling the truth.
- The rest is the graph's width inside Platform: a change to one Platform crate selects its dependents
  (`library-tool` in 17 merges, `library` 13, `platform`, `scrape`, `savemedia`, `firmware`, `cheats`,
  `dianash-program` 10 each), and a change to `Platform/Cargo.lock` or `Platform/Cargo.toml`, which blast.sh claimed
  and selected nothing for, selects every member and the lines of each.
- blast.sh's two unmapped files (`EmuSen.WiseMan.csproj`, `NoToolCounts.cs`) were selected by impact, not stopped.
- Before the libretro line and the workspace edge, 20 merges widened to all of WiseMan; the libretro crates (five
  merges) were the largest single cause.

**The replay with the covered rule, 2026-11-02.** Every land folder with a TRX (22, since the TRX logger of
2026-11-01) and its failing classes, against the second map of §3.7 and this revision's `inputs.json`
(`impact --base M^1 --head M^2 --map-age 100000 --inputs`), the predicted minutes from each class's median duration
over the TRX files:

- **Recall: 10 of the 11 failing classes selected.** The pausecard land f3d6253aa (`WindowFitAuditTests`, the
  game page's "Slot 3" label), every other `WindowFitAuditTests` failure (9e984d1b8, 8ea1d89ff, 7f39a18d2), and
  b5d15c82e's four (`EsdeMenusTests`, `ScreensaverTests`, `ThemedFavouriteCursorTests`, `ControllersTests`) are
  all in. The one miss: `GraphicsSettingsWindowLayoutTests` under e7f33adca (the tool screens), which executes none
  of that land's files; it failed because another class in the same process reset Galaxia's statics under
  parallel execution, and 3a0c4f80e fixed that isolation on 2026-11-02 (land 99bc4bf1f). A failure that depends on
  the other classes in the process is not in a change's blast radius; the rule that selects a class that failed in
  the branch's last trial re-runs it on that branch regardless.
- **Size, the last ten lands, classes and predicted minutes, what ran against the selection:** pausecard f3d6253aa
  285 / 52 min to 96 / 28; the pad menu 7f39a18d2 178 / 43 to 94 / 28; the game page's recordings dc6bc0a67
  285 / 52 to 3 / 17 (`WindowFitAuditTests` and the two owners of the `.spec` files); the tool screens 853a16e79
  319 / 56 to 154 / 48; 99bc4bf1f 4 / 1 to 6 / 2. Four lands widen to the whole suite (521 to 528 classes, 60 min)
  because the branch added files the map had never seen (`EmuSen.Mistress/Pages/GamePageHost.cs` and 17 more in
  b5d15c82e and 0dc303808, `Fixtures/TestCollections.cs` in 678be6322, one in 3e7e676ba) and a new file falls to
  the graph by the rule above; and the cores land 49bd9f604 (`emusen-gpu`'s `Cargo.lock`) goes from 29 / 4 to
  199 / 29, narrowed by the cores' lines, whose terms are broad, with none of its 29 left out. In all, 340 minutes
  ran and 391 are predicted: a third where the map answers, the suite where a new file or a broad line does.
- **The padloss case** (8b9c2c4c1 against dc6bc0a67, Rust only, the ring and the broker): 214 classes, 36 min:
  `HostedCoreLaunchTests` (its own file), `LibretroHostTests` (the Libretro line), and through the two `covered:`
  lines every class that executed the broker's `CoreLibrary.cs` (155) or `SerenityNative.cs` (31): the Serenity
  ring, chain, device-lend and device-presenter classes, the hosted-core and core-ABI classes, the console screens
  that launch a core. Without the map the same change selects the suite whole (522 by graph), as any Rust change
  into the platform library with no line does.
- **Left out:** in 12 of the 22 lands the selection leaves out classes that ran then; none of them failed, and the
  largest sets are the lands above where blast.sh's namespace terms ran the Mistress or the cores namespace whole.
- **With the new-file rule** (the same day, recall unchanged): b5d15c82e's `EmuSen.Mistress/Pages/GamePageHost.cs`
  is claimed through `MainWindow.Library.cs`, which names it; the land stays at the suite for
  `EmuSen.WiseMan/Mistress/BigPicture/GameOptionsPictureTool.cs`, an added WiseMan file no test class names, which
  the graph answers with the project whole. 678be6322 stays at the suite for `Platform/platform/src/galaxia.rs`,
  a change in the platform crate, which has no line: a `covered:EmuSen.Galaxia/Native/*.cs` line for Galaxia's
  surface would narrow it the way the serenity line does, and is the next line to write.
- **With `WindowFitAuditTests` mapped by case** (§3.7, the third replay the same day): recall unchanged; the
  pausecard and pad-menu lands unchanged at 96 / 28 and 94 / 28, since both changed the audit's own files
  (`WindowFitAuditTests.PadMenu.cs`, `.CardContrast.cs`) and a test file selects its class whole; padloss 485 units
  (213 classes and all 272 groups, which every view's drawing through Serenity's surface earns), 36 minutes. The
  groups show on a change to a window of its own: `CoresWindow.cs` 10 units and 1.5 minutes, `ScraperPane.cs` 9
  and 1.4, against 16 with the audit whole; a `MainWindow` partial stays at 25, for the reason §3.7 gives.

The rule for new files is where the next third lies: a file added by the branch is reached only through the
branch's other edits, which the map does know, through a new test class, which selects itself, or through
`.axaml`, which the lines claim; selecting the classes that executed the modified files that name the new one
would keep those lands at the map's size. Not built: a change to a safety rule of this stage.

## 7. Tests

`emusen-testing-module`'s 27 tests, run under nextest: the rules on a repository built in a temporary folder (a
workspace, a core built into a host by `RustCores.props`, a test project, a recording, the data files); keys before and
after commits; reuse, quarantine and migration against a temporary store; the readers of JUnit, TRX and both text logs;
the class finder; the graph against `cargo metadata`; and since 2026-11-02 the covered rule on a loaded map (the
classes that executed a file, the class without a report, a folder line's classes stepping aside, a file the map never
saw), a `covered:` entry with and without a map and the entry-crate narrowing, a new file through the changed files
that name it, a group of cases as a unit and its merge into the map, the shards dealt by time with a collection
kept or spread, a run loaded and replaced, a cobertura report read with and without the discovery baseline, and a
stale map refused.

Seeded faults, 2026-11-01, each reverted, each caught by one test:

| Fault | Caught by |
|---|---|
| the recording rule skipped (a `.spec` change selects nothing) | `a_recording_selects_its_owner` |
| reuse ignoring the key | `a_green_result_is_reused_only_under_its_own_key` |
| a flaky test passing unquarantined | `a_flaky_test_fails_unless_quarantined` |
| an unknown file selecting nothing | `an_unknown_file_selects_the_tier_and_a_doc_selects_nothing` |
| the key hashing the unit's own folder only | `a_changed_input_changes_the_key_and_nothing_else_does` |
| an expired quarantine still honoured | `a_quarantine_has_an_owner_a_cause_and_an_expiry` |

Seeded faults, 2026-11-02, for the coverage map, each reverted, each caught by one test:

| Fault | Caught by |
|---|---|
| the classes without a report left out of the covered rule | `a_covered_file_selects_the_classes_that_executed_it_and_those_without_a_report` |
| a line with `hits="0"` counted as executed | `a_report_gives_its_executed_files_and_their_places` |
| the previous map kept on load (a row both maps hold keeps the old trial's number and vanishes) | `a_run_loads_into_the_store_and_replaces_the_previous_map` |
| the age comparison reversed | `a_stale_map_says_so_and_falls_back` |
| the map answering for files that are not C# | `a_run_loads_into_the_store_and_replaces_the_previous_map` |
| a folder line still naming classes for a covered file | `a_covered_file_selects_the_classes_that_executed_it_and_those_without_a_report` |
| a `covered:` entry narrowing a project with no map | `a_covered_entry_names_the_classes_that_executed_a_crates_surface_and_reached_lines_narrow_only_when_every_entry_has_one` |
| a line at the discovery floor counted as executed (`>=` for `>`) | `a_report_counts_a_file_executed_only_beyond_the_discovery_baseline` |
| any one lined entry crate narrowing, rather than every one | `a_covered_entry_names_the_classes_that_executed_a_crates_surface_and_reached_lines_narrow_only_when_every_entry_has_one` |

Nine faults rather than the stage's three to seven: the replay of the first map added two mechanisms (the discovery
baseline and the entry-crate narrowing) after the first seven were run, and each got one.

End to end on 2026-11-01: `trial` ran two Platform crates through nextest (51 tests, 6 s), and a dry run straight
after reported both as reusable from it; it ran one WiseMan class through the dotnet adapter in a fresh checkout
(build 5 m 03 s with the cores cold, the class 5 s, its TRX read back); and it ran this change's own blast radius, the
whole Platform workspace and the 72 DianaSh and Golden classes (1,690 tests, 4 m 59 s), where it found the six new man
pages missing from `man.golden`, which `GoldenCorpusTests.Every_manual_page_is_recorded` requires.

## 8. Decisions

Decided 2026-11-01 by the tester, as recommended: nextest is the cargo runner; green results are reused by key at the
fast and land tiers, never at nightly; a flaky test fails the land unless quarantined by name with an owner, a cause
and a 14-day expiry; heavy jobs take slots with `queue ask` once this phase lands, agreed by hand until then; the real-window
and device drivers stay Python inside declared jobs; the C# coverage pass runs on the Mint laptop's nightly. Also: the
`Testing/` folder, the store under `~/.local/share/EmuSen/testing`, results under `~/.cache/emusen/testing`,
`runners.json` under `~/.config/EmuSen/testing`, and this page.

cargo-nextest 0.9.150 was installed user-locally on the desktop (`cargo install --locked cargo-nextest`, 2026-11-01).

## 9. Left for later phases

- Phase 2: `golden` and commit-check's cause line; the runners' adapters (laptop, Mac, handheld) and declared jobs
  (real window, handheld smoke, GPU matrix); CI on `trial --tier ci`; `library_key.py` on `units --key`; the nightly's
  `miss` ledger; `studio`; `trial` as a job under `ps`/`kill`; a unit's needs matched against a runner's caps.
- Phase 3: coverage narrowing inside a project, built 2026-11-02 (§3.7); `blast-map.txt` retired on a month of
  evidence; the map asked about LunaP's own diff (the `lunap:` rows are recorded, no command reads them yet); `.fs`
  files under the covered rule once Pegasus has tests of its own.
- The shards' measure on a real land (§3.8 is predicted from TRX durations; the first sharded land is the measure),
  and the collection question it hangs on.
- Not built in phase 1: `nextest archive` reuse of a build across runners; Sapphire's build counting as two slots.
- Phase 4: the dotnet adapter (`adapters.rs`' dotnet half) leaves with the last C# test project but LunaP's.
