# Testing

The testing studio's committed data, read by the `emusen.testing` DianaSh module (`Platform/emusen-testing-module`).
The design and verification workflow are documented in [Testing_and_Verification.md](../Docs/EmuSen%20Manual/Testing_and_Verification.md); each command also has a man page in [`Docs/man/emusen.testing/`](../Docs/man/emusen.testing/).

- `inputs.json` — what no build graph shows: folder lines that narrow a reached test project, recordings' owners,
  files that are nobody's input, and extra key inputs. Every line carries the reason it was added.
- `tiers.json` — what each tier (`fast`, `land`, `nightly`, `games`, `hardware`) selects, leaves out, builds with, and reuses.
- `schema/` — the store's SQLite schema, numbered migrations applied forward by `user_version`.
- `esde-corpus/` — `esde-check`'s measure of ES-DE's listed themes, one TSV per measurement; a row that worsens fails a
  landing until explained (see [Qt_Frontend_and_Theming.md](../Docs/EmuSen%20Manual/Qt_Frontend_and_Theming.md) and [Testing_and_Verification.md](../Docs/EmuSen%20Manual/Testing_and_Verification.md)).

The store itself is `~/.local/share/EmuSen/testing/studio.db`, trial files are under `~/.cache/emusen/testing/`, and
the local machine roster is `~/.config/EmuSen/testing/runners.json`; none of them is committed.
