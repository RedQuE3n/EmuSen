#!/usr/bin/env python3
"""Ingest a probe's dump directory into the signature database.

    dumpdb.py ingest <dumpDir> [<dumpDir> ...] [--db <path>]
    dumpdb.py list [--db <path>]

A dump directory is what `EmuSen.Pharaoh --probe ... --sig` and the Rust probe
both write: one `<backend>_sig.csv` carrying a CRC32 per space per frame, one
`<backend>_manifest_fNNNNN.json` per frame, and the raw `.bin` blobs those
manifests name. Neither probe was changed to produce a database; this reads what
they already write. See EmuSen_Stack.md §3 for why the ingest is a separate step
and not something the probe does itself.

The database is a cache. Deleting it loses nothing - re-run the ingest. It
defaults to `dumps.db` beside this script, and re-ingesting a directory replaces
that set in place, so running it twice is the same as running it once.

Two things about identity are worth knowing before trusting a row, because the
CSV header block and the first manifest both carry it and they do not always
agree. The C# reader this replaces resolved that collision inconsistently, and
the inconsistency is reproduced here deliberately rather than tidied, because the
comparator's verdicts were pinned against it:

  - text fields (backend, system, rom, board, region, headerTrust, format) take
    the manifest's value whenever it is present and non-empty, and the CSV's
    otherwise;
  - numeric fields (prg, chr) take the CSV's value, and only fall back to the
    manifest when the CSV said zero.

So a disagreement resolves one way for a string and the other way for a number.
Tidying that is a behaviour change and belongs in its own commit, with the
comparator re-verified against it.

An empty string means "this backend cannot say", which is not the same as a
disagreement - the identity gate requires both sides to have spoken before it
calls anything a mismatch. See EmuSen_Debugging_Tools_Reference_v5.md §3.48.
"""
import glob
import json
import os
import re
import sqlite3
import sys

SCHEMA = os.path.join(os.path.dirname(os.path.abspath(__file__)), "dumps-schema.sql")
DEFAULT_DB = os.path.join(os.path.dirname(os.path.abspath(__file__)), "dumps.db")

MANIFEST_NAME = re.compile(r"^(?P<backend>.+?)_manifest_f(?P<frame>\d+)\.json$")

# The CSV always carries a screen column, even for a backend with no frame
# buffer, so it is a column name like any other rather than a special case.
TEXT_FIELDS = ("backend", "system", "rom", "board", "region", "header_trust", "screen_format")


def open_db(path=DEFAULT_DB):
    """Open the store, creating it from the committed schema if it is not there."""
    parent = os.path.dirname(os.path.abspath(path))
    if parent:
        os.makedirs(parent, exist_ok=True)
    db = sqlite3.connect(path)
    db.row_factory = sqlite3.Row
    with open(SCHEMA, encoding="utf-8") as schema:
        db.executescript(schema.read())
    db.execute("PRAGMA foreign_keys = ON")
    return db


def read_signature(path):
    """Parse one <backend>_sig.csv into (identity, columns, {frame: {column: crc}})."""
    identity = {}
    columns = []
    rows = {}

    keys = {
        "backend": "backend", "system": "system", "rom": "rom", "board": "board",
        "region": "region", "headerTrust": "header_trust", "screenFormat": "screen_format",
    }

    with open(path, encoding="utf-8") as sig:
        for line in sig:
            line = line.rstrip("\n")
            if line.startswith("#"):
                if "=" not in line:
                    continue
                key, _, value = line[1:].partition("=")
                key, value = key.strip(), value.strip()
                if key in keys:
                    identity[keys[key]] = value
                elif key == "prg":
                    identity["prg_bytes"] = _int(value)
                elif key == "chr":
                    identity["chr_bytes"] = _int(value)
                elif key == "saveLoaded":
                    identity["save_loaded"] = 1 if value == "1" else 0
                continue

            fields = line.split(",")
            if len(fields) < 2:
                continue
            if fields[0] == "frame":
                columns = fields[1:]
                continue
            if not _is_int(fields[0]):
                continue

            frame = int(fields[0])
            # Short rows are tolerated the way the C# reader tolerated them: the
            # columns that are present are read and the rest are simply absent.
            rows[frame] = {
                columns[i]: int(fields[i + 1], 16)
                for i in range(min(len(columns), len(fields) - 1))
            }

    return identity, columns, rows


def read_manifest(path):
    """Parse one manifest. Real JSON, not the regular expressions this replaces."""
    with open(path, encoding="utf-8") as manifest:
        return json.load(manifest)


def _manifests(directory):
    found = []
    for path in glob.glob(os.path.join(directory, "*_manifest_f*.json")):
        name = MANIFEST_NAME.match(os.path.basename(path))
        if name:
            found.append((path, name.group("backend"), int(name.group("frame"))))
    return sorted(found, key=lambda entry: os.path.basename(entry[0]))


def ingest(db, directory):
    """Load one dump directory, replacing whatever was there for it before.

    Returns the new set id, or None when the directory holds no dump this can
    identify - which is what an empty or unrelated directory looks like.
    """
    directory = os.path.abspath(directory)

    identity, order, rows = {}, [], {}
    sig_files = sorted(glob.glob(os.path.join(directory, "*_sig.csv")))
    if sig_files:
        identity, order, rows = read_signature(sig_files[0])

    manifests = _manifests(directory)
    spaces, screens = [], []

    if manifests:
        first_path, filename_backend, _ = manifests[0]
        first = read_manifest(first_path)
        if not identity.get("backend"):
            identity["backend"] = filename_backend

        # Text fields: the manifest wins when it has something to say.
        for key, source in (("backend", first), ("system", first), ("rom", first)):
            identity[key] = _pick(source.get(key), identity.get(key))
        ident = first.get("identity") or {}
        identity["board"] = _pick(ident.get("board"), identity.get("board"))
        identity["region"] = _pick(ident.get("region"), identity.get("region"))
        identity["header_trust"] = _pick(ident.get("headerTrust"), identity.get("header_trust"))
        screen = first.get("screen") or {}
        identity["screen_format"] = _pick(screen.get("format"), identity.get("screen_format"))

        # Numeric fields: the CSV wins unless it said nothing.
        if not identity.get("prg_bytes"):
            identity["prg_bytes"] = _int(ident.get("prg"))
        if not identity.get("chr_bytes"):
            identity["chr_bytes"] = _int(ident.get("chr"))
        if "save_loaded" not in identity:
            identity["save_loaded"] = 1 if ident.get("saveLoaded") else 0

        for path, _, frame in manifests:
            report = read_manifest(path)
            for entry in report.get("spaces") or []:
                spaces.append((frame, entry["name"], _int(entry.get("size")), entry.get("file", "")))
            shown = report.get("screen")
            if shown:
                screens.append((frame, _int(shown.get("width")), _int(shown.get("height")),
                                _int(shown.get("bytes")), shown.get("format", ""),
                                shown.get("file", "")))

    if not identity.get("backend"):
        return None

    with db:
        db.execute("DELETE FROM dump_set WHERE directory = ?", (directory,))
        cursor = db.execute(
            """INSERT INTO dump_set (directory, backend, system, rom, board, region,
                                     header_trust, prg_bytes, chr_bytes, save_loaded, screen_format)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (directory, identity.get("backend", ""), identity.get("system", ""),
             identity.get("rom", ""), identity.get("board", ""), identity.get("region", ""),
             identity.get("header_trust", ""), identity.get("prg_bytes", 0),
             identity.get("chr_bytes", 0), identity.get("save_loaded", 0),
             identity.get("screen_format", "")))
        set_id = cursor.lastrowid

        db.executemany(
            "INSERT INTO signature_column (set_id, ordinal, name) VALUES (?, ?, ?)",
            [(set_id, ordinal, name) for ordinal, name in enumerate(order)])
        db.executemany(
            "INSERT INTO signature (set_id, frame, column_name, crc) VALUES (?, ?, ?, ?)",
            [(set_id, frame, column, crc)
             for frame, row in rows.items() for column, crc in row.items()])
        db.executemany(
            "INSERT OR REPLACE INTO space (set_id, frame, name, bytes, file) VALUES (?, ?, ?, ?, ?)",
            [(set_id,) + entry for entry in spaces])
        db.executemany(
            """INSERT OR REPLACE INTO screen (set_id, frame, width, height, bytes, format, file)
               VALUES (?, ?, ?, ?, ?, ?, ?)""",
            [(set_id,) + entry for entry in screens])

    return set_id


def set_for(db, directory):
    """The row for a directory, ingesting it first if it has not been seen."""
    directory = os.path.abspath(directory)
    row = db.execute("SELECT * FROM dump_set WHERE directory = ?", (directory,)).fetchone()
    if row is None:
        if ingest(db, directory) is None:
            return None
        row = db.execute("SELECT * FROM dump_set WHERE directory = ?", (directory,)).fetchone()
    return row


def columns(db, set_id):
    """The columns in the order the probe wrote them, which the report preserves."""
    return [r[0] for r in db.execute(
        "SELECT name FROM signature_column WHERE set_id = ? ORDER BY ordinal", (set_id,))]


def frames(db, set_id):
    return [r[0] for r in db.execute(
        "SELECT DISTINCT frame FROM signature WHERE set_id = ? ORDER BY frame", (set_id,))]


def signature(db, set_id):
    """The whole stream as {frame: {column: crc}}."""
    rows = {}
    for frame, column, crc in db.execute(
            "SELECT frame, column_name, crc FROM signature WHERE set_id = ?", (set_id,)):
        rows.setdefault(frame, {})[column] = crc
    return rows


def varies(db, set_id, column):
    """Whether a column ever takes a second value. Two rows is enough to know."""
    return db.execute(
        "SELECT COUNT(*) FROM (SELECT DISTINCT crc FROM signature "
        "WHERE set_id = ? AND column_name = ? LIMIT 2)", (set_id, column)).fetchone()[0] > 1


def agreement_rate(db, left_id, right_id, column, offset, through_frame=None):
    """The share of frames on which one column agrees once the right side is shifted.

    This is the operation the whole comparison is built on, and the reason the
    signature is a table at all: it is a self-join with the offset applied to the
    join key. Frames present on only one side, and a column present on only one
    side, drop out of the join rather than being counted as disagreement.
    """
    row = db.execute(
        """SELECT COUNT(*), COALESCE(SUM(a.crc = b.crc), 0)
             FROM signature a
             JOIN signature b
               ON b.set_id = ? AND b.frame = a.frame + ? AND b.column_name = a.column_name
            WHERE a.set_id = ? AND a.column_name = ? AND a.frame <= ?""",
        (right_id, offset, left_id, column,
         _MAX_FRAME if through_frame is None else through_frame)).fetchone()

    common, agreed = row[0], row[1]
    return 0.0 if common == 0 else agreed / float(common)


# Stands in for "no limit" in the agreement query, above any real frame number.
_MAX_FRAME = 2 ** 62


def _pick(candidate, current):
    """The manifest's value when it said something, otherwise what we had."""
    if candidate is None:
        return current or ""
    text = str(candidate)
    return text if text else (current or "")


def _int(value):
    try:
        return int(value)
    except (TypeError, ValueError):
        return 0


def _is_int(text):
    try:
        int(text)
        return True
    except ValueError:
        return False


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2

    db_path = DEFAULT_DB
    args = list(argv[1:])
    if "--db" in args:
        at = args.index("--db")
        if at + 1 >= len(args):
            print(__doc__)
            return 2
        db_path = args[at + 1]
        del args[at:at + 2]

    if not args:
        print(__doc__)
        return 2

    command, rest = args[0], args[1:]
    db = open_db(db_path)

    if command == "ingest":
        if not rest:
            print(__doc__)
            return 2
        failed = False
        for directory in rest:
            set_id = ingest(db, directory)
            if set_id is None:
                print(f"no dump found in {directory}", file=sys.stderr)
                failed = True
                continue
            count = db.execute("SELECT COUNT(*) FROM signature WHERE set_id = ?",
                               (set_id,)).fetchone()[0]
            row = db.execute("SELECT * FROM dump_set WHERE id = ?", (set_id,)).fetchone()
            print(f"{row['backend']}: {count} signature rows from {directory}")
        return 1 if failed else 0

    if command == "list":
        for row in db.execute("SELECT * FROM dump_set ORDER BY directory"):
            span = db.execute("SELECT * FROM set_span WHERE set_id = ?", (row["id"],)).fetchone()
            frames_text = (f"{span['frames']} frames {span['first_frame']}-{span['last_frame']}"
                           if span else "no signature")
            print(f"{row['backend']:8} {row['system']:5} {frames_text:28} {row['directory']}")
        return 0

    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
