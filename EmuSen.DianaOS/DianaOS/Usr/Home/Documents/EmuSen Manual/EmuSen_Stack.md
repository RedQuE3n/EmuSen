# EmuSen — the language and storage stack

*This revision: the first. The stack was settled in discussion on 2026-08-08 and had never been written down, which is the whole reason §4 exists — every entry there is drift that accumulated silently because there was no document to drift from.*

---

## 1. The four rules

| | Owns |
|---|---|
| **Rust** | The lower level: the reference probe, and anything C# cannot express accurately, safely and with performance at once. |
| **C#** | Everything else that should be written in C#. |
| **Python** | Testing, and accurate mathematical functions. |
| **SQLite** | Everything stored as config or as a table. |

These are stated as ownership, not as preference. A new subsystem does not get to argue its way into a language; it gets to argue that it is a different *kind* of thing than the rule assumes, and that argument goes in §4.

## 2. Where each boundary actually falls

### 2.1 Rust — the probe, and the reason it is not more than the probe

`EmuSen.WiseMan/Reference/probe-rs/` is the whole of it: three backends over one policy layer, driving Mesen through the `probe-c-api.patch` ABI and any libretro core through `dlopen`. See `EmuSen_Debugging_Tools_Reference_v5.md` §3.50 for why Mesen's own C ABI could not close the gap and §3.52 for the shape that replaced it.

The rule's phrasing matters: *accurately, safely, and with performance*, all three. The probe qualifies because it loads foreign C++ into its own address space and reads emulator-internal memory out of it — C# can do that, but not without either a marshalling layer that changes the timing being measured or an `unsafe` surface large enough to stop being reviewable.

**The cores are C#, permanently, and that is not an exception to the rule.** An emulator core in C# *is* the project's research claim; a Rust core would answer a question nobody asked here. `project_perf_investigation_settled` and §13.1 of `Venus_PPU.md` record four measured-and-rejected optimizations, none of which pointed at the language.

**The one arguable case is `EmuSen.Endymion/AudioPlayer.cs`,** the only `unsafe` in the tree — a thin SDL3 binding on the audio submit path. It stays C#. Moving it to Rust would *add* an FFI boundary rather than remove one: today there is one hop (C# → SDL3), and a Rust sink would make it two with no accuracy or safety gained. Recorded here so the next reader does not have to re-derive it from the `unsafe` keyword alone.

### 2.2 Python — offline analysis, not in-process tests

The rule says Python owns testing, and `CLAUDE.md` says tests run headless through `EmuSen.WiseMan`. Both are true, because they are about different things, and the seam is **whether the thing under test has to be alive**.

- **In-process verification stays C#/WiseMan.** A test that steps a CPU, drives `IDebugTarget`, renders an Avalonia control headlessly or drains `ICore`'s audio needs the live object graph. Nothing is gained by reaching it from another process, and the harness rule (`extend the harness, do not spawn a window`) is unaffected.
- **Offline analysis over artifacts is Python.** Once a run has produced files — dump sets, signatures, traces — the analysis is arithmetic over data and has no reason to be in the emulator's language. `gsudiff.py` was the first instance; §3 is the second and much larger one.

That seam is not arbitrary: it is exactly where the emulator dependency ends. `--compare` and `--verify-dictionary` never load a ROM, while `--probe` cannot avoid it.

**Dependency policy: standard library only.** No venv, no `requirements.txt`, no install step — `python3 <tool>` from a clean clone. This follows `gsudiff.py`, which is stdlib-only by instinct rather than by policy; the policy is now stated. `sqlite3`, `csv` and `json` are stdlib, and per-pixel screen work through `memoryview.cast('I')` runs in single-digit milliseconds per frame, so numpy would buy convenience rather than capability. Tests are `unittest`, run with `python3 -m unittest`.

**Where this does not generalise:** the rule is about *analysis* being mathematical, not about Python being the more accurate language. Nothing here claims a numeric result Python gets right and C# gets wrong. Both use IEEE 754 doubles and the port in §3 was required to be bit-identical in its verdicts before the C# was removed.

**A note on comments.** The one-line-maximum comment rule is about `.cs` files and is not widened by this document. The established pattern for this project's Python is `gsudiff.py`'s: a module docstring that *is* the manual page — synopsis, the exact command that produces the input, preconditions, a paragraph on why the analysis is shaped the way it is, and cross-references into these docs. A Python tool carries its own man page; a C# file points at one.

### 2.3 SQLite — the two databases that were already right

Both predate this document and both are the model:

- **The ROM catalogue** (`EmuSen.Galaxia/Library/Catalogue/catalogue-schema.sql`, driven by `EmuSen/Common/Catalogue/SqliteCatalogue.cs`) — see `EmuSen_Galaxia.md` §7.
- **The known-differences dictionary** (`Dictionary/schema.sql`) — see `EmuSen_Debugging_Tools_Reference_v5.md` §3.49. Its three triggers make `status = 'proven'` unreachable without a passing verification row, which is the project's evidence discipline expressed as something an author cannot talk past rather than as a convention they might forget.

Two structural rules they establish, and which anything new must follow:

1. **The schema is committed as `.sql`; the `.db` is a build artifact and gitignored.** A binary cannot be reviewed and a schema is the part worth reviewing. Note the `.gitignore` entries are individual filenames, not a glob — a new database needs a new line.
2. **The contract may live in a leaf; the driver may not.** `EmuSen.Galaxia` has no `PackageReference` at all, so `ICatalogue` and the schema live there while `Microsoft.Data.Sqlite` lives one layer up. The same applies to `EmuSen.DianaOS`, which is referenced *by* `EmuSen` rather than the other way round: the shell takes an `ICatalogue` it is handed and cannot construct one.

## 3. The signature store and the comparator

The largest instance of both rules landing on the same code.

`EmuSen.Pharaoh`'s comparator was 499 lines of C# statistics over a `frame → column → CRC32` mapping stored as one CSV plus one JSON manifest per frame, the latter parsed by regular expression rather than by a JSON parser. The mapping is a table by any reading, and the central operation — `AgreementRate`, the share of frames on which one column's CRC matches the other side's at a given stream offset — is a self-join with an offset applied to the join key. It was written as nested dictionary lookups because the data had no table to live in.

The migration is therefore one change, not two: the signature becomes a SQLite table, and the analysis over it becomes Python.

**The probes were not changed.** Both the C# `PeerProbe` and the Rust `dump.rs` still write `sig.csv` and per-frame manifests, byte-for-byte identically, and their parity tests were untouched. The database is an *ingest* of what is on disk — a deletable cache, exactly the argument `catalogue-schema.sql` already makes for itself. A probe writes a stream while it runs, and appending a row to a CSV is the right shape for that; wiring `rusqlite` into the probe to avoid a later ingest step would have bought nothing and cost the probe its dependency-free build.

**What SQLite did and did not buy here.** It removed regex-parsing of JSON, made the ingest/analysis seam explicit, and gave the join an honest expression. It did **not** make anything measurably faster: on a 75-frame set the in-memory dictionaries were already fast, and no speed improvement was measured or claimed. The performance argument only begins to apply at run lengths this project has not yet taken — thousands of frames across many columns — and is recorded here as a prediction, not a result.

### 3.1 The differential, and what it caught

Nothing was deleted on the strength of the port looking right. `PythonPortDifferentialTests` ran both implementations over the same argv and asserted byte-identical stdout and exit code across seventeen cases: each identity gate, both warnings, the column ratings, a located divergence, input blame, the stream-offset search, both screen formats, the vacuity gate, a vertical shift, a static reference, a missing screen, an empty signature, and the committed Mesen dump set. It passed, and it was then deleted along with the C# it verified — which is why the result is recorded here instead.

Beyond the fixtures, one real pair: `--probe --sig` over SMB3 frames 120–194 against `EmuSen.WiseMan/Reference/dumps/SMB3`. Both implementations produced the same output down to the byte, including the one-sided board warning, the real `PaletteIndex16` decode through the NES palette (5 colours, dominant 92.7%), 1536 differing pixels at 2.50%, and phase +2 — the two-frame boot offset §3.48 predicts.

**The differential earned its cost on one case.** The port was written believing that percentages needed special handling: .NET's `F` format was assumed to round half away from zero while Python rounds half to even, and the values are reachable — 384 differing pixels out of 256×240 is exactly 0.625%. `_fixed()` was accordingly implemented with a decimal round-half-away-from-zero. The test failed on precisely that case, **in the opposite direction to the prediction**. Measured against .NET 10 across eighteen values chosen to cover every dyadic midpoint at zero, one and two decimals: `F` rounds half to **even**, exactly as Python's `format()` does. The deliberate correction disagreed with .NET on seven of eighteen; doing nothing disagreed on none.

The prediction is retained in `compare.py`'s header rather than deleted. "Two languages must round differently" is a plausible claim that would otherwise be re-derived, believed and re-implemented by the next person to look at the file — and the belief was not idle, it was acted upon and shipped a bug that only a differential caught.

**Where this does not generalise:** a byte-identical differential was affordable because the tool's entire output is a text report and an exit code. A port whose output is a data structure, a rendered image, or a timing would need a different and weaker notion of "the same", and should not cite this as precedent for deleting the original.

## 4. Reasoned exceptions

Everything below violates a rule in §1 and stays that way on purpose.

### 4.1 Config stays JSON

The strongest-looking case for SQLite in the tree, and it does not survive contact with the dependency graph.

- **The leaf constraint forbids it.** `EmuSen.Galaxia` has no `PackageReference` at all; `EmuSen.DianaOS → Galaxia` and `EmuSen → DianaOS`. Neither Galaxia nor DianaOS can take `Microsoft.Data.Sqlite` without closing a cycle, so a settings database would need an interface in Galaxia, a driver in `EmuSen`, and injection through every frontend — for values that are read once at startup.
- **The path is late-bound and the tests pin it.** Every model holds a `static readonly ConfigFile<T>` created at type-initialization, while `ConfigFile.Path` is recomputed on every access so that `ConfigStore.OverrideDirectory` can move underneath it. Around thirty test fixtures swap that override between tests in one process, and `ConfigFileTests.Path_follows_the_override_when_it_moves` pins the behaviour directly. A connection opened at type-init breaks all of them.
- **Hand-editability is a stated design goal**, in `ConfigJson.cs`, `CheatFile.cs` and `AudioSettings.cs` independently — which is why the JSON reader accepts comments, trailing commas, case-insensitive names, and enums written as either names or numbers, and why the settings hubs clamp every value on the way in rather than trusting the file. SQLite removes the editing surface that machinery exists to serve.
- Roughly a dozen tests assert on the *bytes* of a config file, not on round-trip behaviour.

**Where this does not generalise:** none of the above is an argument that config is unsuited to a database in general. It is an argument that *this* config, behind *this* leaf assembly, with *this* hand-editing contract, is not worth the cycle-breaking. A frontend built without the Galaxia constraint would be free to decide differently.

### 4.2 Coverage maps, blobs, and the schemas themselves

- **`.covmap`** (`EMCV`) is a dense bitset of executed addresses. A bitmap is not a table, and storing one row per address would be strictly worse on every axis.
- **Raw dumps, firmware images and `.srm` saves** are opaque byte blobs whose structure belongs to the hardware, not to a schema.
- **The `.sql` schema files** are text on purpose, per §2.3.

### 4.3 Deferred, not rejected

Recorded so they are not re-derived from scratch:

- **The `.cht` cheat database** — 2,779 files and 16 MB in the user's tree today, re-walked by every `CheatDatabase` instance, whose own comment notes a real libretro tree is tens of thousands of files. This is catalogue-shaped and is the strongest remaining candidate. It needs the interface/driver split of §2.3 threaded through DianaOS.
- **Per-game cheat lists** (`cheats/<game>.json`) are genuinely tabular but small, and migrating them costs the same architectural split to make 253 rows queryable while losing the hand-editing of §4.1. Poor trade at present size.
- **`FrameRecorder`'s `frames.log`** is a four-column TSV that nothing in the repo reads back. The documented cross-reference against `ffmpeg -f framemd5` (`EmuSen_Debugging_Tools_Reference_v5.md` §3.15b) is a real join performed by hand — a good future Python tool, at which point the table becomes worth having.
- **Schema versioning.** No database here has `PRAGMA user_version` or any migration path. The catalogue re-runs its `IF NOT EXISTS` schema on every open, so a new column silently does nothing to an existing file — tolerable only because the catalogue is declared a deletable cache. The dictionary runs its schema *only when the file is absent*, so editing `schema.sql` has no effect on an existing `known-differences.db`, and that database accumulates `verification` rows that are the only thing making `proven` reachable. Deleting it to pick up a schema change would destroy the proof history. This is the sharpest known gap in the storage design.

## 5. Adding to the stack

- A new **database**: schema as committed `.sql` next to its owner, `IF NOT EXISTS` throughout, `PRAGMA foreign_keys = ON`, a header comment carrying the argument for the table's existence, a `<None … CopyToOutputDirectory="PreserveNewest">` item in the owning `.csproj` (it propagates transitively through `ProjectReference`; no consumer needs its own), and a new `.gitignore` line for the `.db`.
- A new **Python tool**: `EmuSen.WiseMan/Reference/analysis/`, stdlib only, `#!/usr/bin/env python3`, module docstring as the manual, exit code as the verdict, `unittest` beside it.
- A new **exception**: a section in §4, with the evidence, and a sentence on where it does not generalise.
