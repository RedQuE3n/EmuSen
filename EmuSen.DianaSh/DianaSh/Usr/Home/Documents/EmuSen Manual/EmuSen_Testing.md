# EmuSen_Testing — the testing studio

How EmuSen is tested: which units a change can affect, how they run, and what the record says. The studio is one
DianaSh command module, `emusen.testing` (crate `Platform/emusen-testing-module`), linked into the standalone `dianash`
program beside `emusen.dev` and never into the platform library, with one SQLite store and three data files. Its
commands are `units`, `impact`, `trial`, `results`, `ingest` and `queue`; each has a man page under
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

1. **recording**: a changed recording selects its owner (a `recording` line of `inputs.json`). A file that ends as a
   recording does (`.spec`, `.golden`, `.frame`, `.framesum`) and that no line owns selects the tier's whole set. This
   is the rule whose absence let a `.spec`-only commit select nothing on 2026-11-01.
2. **always**: a test class's own file selects the class; a unit that failed in the branch's last trial is selected
   again; a tier's `always` list.
3. **declared**: a `folder` line adds its units. Its select entries are WiseMan filter terms (a class is chosen when
   its full name holds the term, as `FullyQualifiedName~term` chose it), `cargo:<crate>`, `refs:<pathspec>` (the test
   files naming a type declared there, directly or through a fixture) and `lunap`.
4. **graph**: the file's owner and its reverse closure. Every crate reached is selected. A test project reached is
   narrowed to the units of the lines matching the file, or matching a reached crate's folder; a WiseMan file that is
   not a test class is narrowed to the classes naming its types; a project no line narrows is selected whole. This is
   the over-selection the design expected: until coverage (phase 3), the folder lines stand in for it.
5. **closed**: a file no rule claims selects the tier's whole set. A `none` line (documentation, ignore lists, CI
   files, the referee tools) claims a file and selects nothing.

The tier then removes its excluded tags (`nightly`) and keeps its `named_only` classes (WindowFitAuditTests at land)
only where a line names them, as blast.sh's filter did.

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

## 4. The data files

### 4.1 `Testing/tiers.json` — §3.3.

### 4.2 `Testing/inputs.json`

What no graph shows, one line per pattern with a dated reason. Its first 73 lines are `blast-map.txt`'s, each dated
by the tools repository's commit that wrote it (`git blame`) and given that commit's subject or the map's own dated
comment; twelve are recordings and two are nobody's input. Added on 2026-11-01: documentation (`\.md$`) and ignore
lists as nobody's input; any `.csproj` selecting `LeafAssemblyTests`; `LunaP.props` selecting `lunap`; `Testing/`
selecting the studio's own crate; and `EmuSen/Cores/Libretro/` selecting `LibretroHostTests`, from the replay of §6.
A term that names no class is reported on standard error: on 2026-11-01, `CrystalTwinScript` (in the map's
`^Platform/crystal` line) named none.

`blast-map.txt` is deleted when a month of the nightly's `miss` rows shows nothing the map would have caught that impact
did not (phase 3).

## 5. The backfill

`ingest --backfill`, run on 2026-11-01 over `~/.cache/emusen/tmp`, in 3.0 s: **155 trials** (129 green, 26 red),
**987 unit runs**, **87,354 test rows**, from 149 land folders (818 cargo logs, 147 dotnet logs), 38 transcripts (32
named a land folder's branch; 6 became trials of their own, their folders gone) and the two full-suite logs (47 m 08 s
green, 51 m 36 s red). Every cargo log's crate was matched to a crate of HEAD. Folders written in the last hour without
a `.ok` file are lands still running, left for the next backfill. Dotnet steps without TRX are recorded as one unit,
`dotnet:EmuSen.WiseMan`, with their counts and failed tests; per-class rows begin with the TRX files land.sh writes
since `testing-speed`.

## 6. impact beside blast.sh

For a week, land.sh runs impact beside blast.sh, logs both answers, and still decides by blast.sh's
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

## 7. Tests

`emusen-testing-module`'s 18 tests, run under nextest: the rules on a repository built in a temporary folder (a
workspace, a core built into a host by `RustCores.props`, a test project, a recording, the data files); keys before and
after commits; reuse, quarantine and migration against a temporary store; the readers of JUnit, TRX and both text logs;
the class finder; and the graph against `cargo metadata`.

Seeded faults, 2026-11-01, each reverted, each caught by one test:

| Fault | Caught by |
|---|---|
| the recording rule skipped (a `.spec` change selects nothing) | `a_recording_selects_its_owner` |
| reuse ignoring the key | `a_green_result_is_reused_only_under_its_own_key` |
| a flaky test passing unquarantined | `a_flaky_test_fails_unless_quarantined` |
| an unknown file selecting nothing | `an_unknown_file_selects_the_tier_and_a_doc_selects_nothing` |
| the key hashing the unit's own folder only | `a_changed_input_changes_the_key_and_nothing_else_does` |
| an expired quarantine still honoured | `a_quarantine_has_an_owner_a_cause_and_an_expiry` |

End to end on 2026-11-01: `trial` ran two Platform crates through nextest (51 tests, 6 s, trial 156), and a dry run
straight after reported both as reusable from it.

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
- Phase 3: coverage narrowing inside a project; `blast-map.txt` retired on a month of evidence.
- Not built in phase 1: `nextest archive` reuse of a build across runners; Sapphire's build counting as two slots.
- Phase 4: the dotnet adapter (`adapters.rs`' dotnet half) leaves with the last C# test project but LunaP's.
