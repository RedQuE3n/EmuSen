# Testing

The testing studio's committed data, read by the `emusen.testing` DianaSh module (`Platform/emusen-testing-module`).
The design and its measurements are `EmuSen_Testing.md` in the manual; each command has a man page.

- `inputs.json` — what no build graph shows: folder lines that narrow a reached test project, recordings' owners,
  files that are nobody's input, and extra key inputs. Every line carries the date and reason it was added.
- `tiers.json` — what each tier (fast, land, nightly, games, hardware) selects, leaves out, builds with and reuses.
- `schema/` — the store's SQLite schema, numbered migrations applied forward by `user_version`.
- `esde-corpus/` — `esde-check`'s measure of ES-DE's listed themes, one TSV a measurement, named by its date; a row
  that worsens fails a landing until explained (`EmuSen_Qt.md` §4.4, §4.5).

The store itself is `~/.local/share/EmuSen/testing/studio.db`, trial files are under `~/.cache/emusen/testing/`, and
the household's machines are `~/.config/EmuSen/testing/runners.json`; none of them is committed.
