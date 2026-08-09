-- The dump signature store - see EmuSen_Stack.md §3.
--
-- A probe writes one CSV row per frame while it runs, plus one JSON manifest per
-- frame. That is the right shape for a stream being appended to, and it is not
-- changed by this schema: the database is an ingest of what is already on disk,
-- deletable and rebuildable at any time, exactly as catalogue-schema.sql argues
-- for itself. Nothing here is the authority on anything; the dump directory is.
--
-- The table exists because the comparison was always a join and never had a place
-- to say so. "How often does column X agree between two runs once one stream is
-- shifted by N frames" is a self-join on (frame + offset, column_name), and it was
-- written as nested dictionary lookups only because the data lived in a CSV.
--
-- This is not a performance change and none was measured. On a 75-frame set the
-- in-memory dictionaries were already fast. What it buys is that the manifests
-- stop being parsed by regular expression, the ingest/analysis seam becomes
-- explicit, and the join is written as one.

PRAGMA foreign_keys = ON;

-- One probe run. Identity is read once, from the first manifest and the CSV
-- header block, which do not always agree - see the precedence note in dumpdb.py.
-- Empty string means "this backend cannot say", which is a legitimate answer and
-- distinct from a disagreement: the identity gate requires both sides to have
-- spoken before it calls anything a mismatch.
CREATE TABLE IF NOT EXISTS dump_set (
    id              INTEGER PRIMARY KEY,

    -- The directory is the identity: re-ingesting one replaces it in place.
    directory       TEXT    NOT NULL UNIQUE,

    backend         TEXT    NOT NULL,
    system          TEXT    NOT NULL DEFAULT '',
    rom             TEXT    NOT NULL DEFAULT '',
    board           TEXT    NOT NULL DEFAULT '',
    region          TEXT    NOT NULL DEFAULT '',
    header_trust    TEXT    NOT NULL DEFAULT '',
    prg_bytes       INTEGER NOT NULL DEFAULT 0,
    chr_bytes       INTEGER NOT NULL DEFAULT 0,
    save_loaded     INTEGER NOT NULL DEFAULT 0 CHECK (save_loaded IN (0, 1)),
    screen_format   TEXT    NOT NULL DEFAULT '',

    ingested_at     TEXT    NOT NULL DEFAULT (datetime('now'))
);

-- The CRC32 stream: the whole of what a divergence search reads. One row per
-- (frame, column) rather than one per frame, because the two sides rarely expose
-- the same spaces and the comparison intersects on column names.
--
-- 'column' is reserved, hence column_name. WITHOUT ROWID because the primary key
-- is the whole row bar the value, and there are as many rows as frames times
-- columns.
CREATE TABLE IF NOT EXISTS signature (
    set_id          INTEGER NOT NULL REFERENCES dump_set(id) ON DELETE CASCADE,
    frame           INTEGER NOT NULL,
    column_name     TEXT    NOT NULL,
    crc             INTEGER NOT NULL,

    PRIMARY KEY (set_id, frame, column_name)
) WITHOUT ROWID;

-- What each frame's manifest listed. The probe has always written these and
-- nothing has ever read them back: the C# reader pulled nine scalars out of the
-- first manifest by regular expression and reconstructed every filename by
-- convention instead. They are recorded here so that a dump set can be checked
-- against its own manifest rather than trusted to follow the naming rule.
CREATE TABLE IF NOT EXISTS space (
    set_id          INTEGER NOT NULL REFERENCES dump_set(id) ON DELETE CASCADE,
    frame           INTEGER NOT NULL,
    name            TEXT    NOT NULL,
    bytes           INTEGER NOT NULL,
    file            TEXT    NOT NULL,

    PRIMARY KEY (set_id, frame, name)
) WITHOUT ROWID;

-- Null screens are legal: only the Rust probe emits one, for a backend that
-- cannot produce a frame buffer, so the row is simply absent for those frames.
CREATE TABLE IF NOT EXISTS screen (
    set_id          INTEGER NOT NULL REFERENCES dump_set(id) ON DELETE CASCADE,
    frame           INTEGER NOT NULL,
    width           INTEGER NOT NULL,
    height          INTEGER NOT NULL,
    bytes           INTEGER NOT NULL,
    format          TEXT    NOT NULL,
    file            TEXT    NOT NULL,

    PRIMARY KEY (set_id, frame)
) WITHOUT ROWID;

-- The agreement query reads one column across every frame of one set, which is
-- the opposite order to the primary key.
CREATE INDEX IF NOT EXISTS signature_by_column ON signature(set_id, column_name, frame);

-- Which frames a set actually carries, for the alignment search's bounds.
CREATE VIEW IF NOT EXISTS set_span AS
SELECT set_id,
       COUNT(DISTINCT frame) AS frames,
       MIN(frame)            AS first_frame,
       MAX(frame)            AS last_frame
  FROM signature
 GROUP BY set_id;
