-- The ROM catalogue - see EmuSen_Galaxia.md §7.
--
-- Galaxia owns this file because she owns where things live; she does not own the
-- driver that executes it. The schema is committed as SQL and the .db is a build
-- artifact, for the same reason the known-differences dictionary is (§3.49): a
-- binary cannot be reviewed, and a schema is the part worth reviewing.
--
-- This catalogue is a cache of what is on disk and is always safe to delete. It
-- is never the authority on anything: the ROM library is, and the library is read
-- and never written.

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS rom (
    path            TEXT    PRIMARY KEY,
    name            TEXT    NOT NULL,
    system          TEXT    NOT NULL,
    bytes           INTEGER NOT NULL,

    -- Null until something pays for it. Hashing 3,500 images is not free, and most
    -- questions do not need identity across a rename.
    md5             TEXT,

    -- What the loader resolved, not what the header claimed - the two differ on a
    -- large minority of any real library (Moon_Memory.md §2.2).
    board           TEXT,
    header_trust    TEXT    CHECK (header_trust IN ('clean', 'unverifiable', 'archaic')),
    region          TEXT,

    prg_bytes       INTEGER NOT NULL DEFAULT 0,
    chr_bytes       INTEGER NOT NULL DEFAULT 0,

    -- Whether a board exists for it today. False is a fact about this program, not
    -- about the image, so it is recorded rather than the row being omitted.
    playable        INTEGER NOT NULL DEFAULT 0 CHECK (playable IN (0, 1)),

    indexed_at      TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS rom_by_system ON rom(system);
CREATE INDEX IF NOT EXISTS rom_by_board  ON rom(system, board);
CREATE INDEX IF NOT EXISTS rom_by_name   ON rom(name);

-- Coverage by board, which is the question that prompted the catalogue: it took
-- four full walks of the library and a header re-parse each time to answer once.
CREATE VIEW IF NOT EXISTS board_coverage AS
SELECT system,
       board,
       COUNT(*)                          AS images,
       SUM(playable)                     AS playable,
       COUNT(*) - SUM(playable)          AS unplayable
  FROM rom
 GROUP BY system, board
 ORDER BY images DESC;
