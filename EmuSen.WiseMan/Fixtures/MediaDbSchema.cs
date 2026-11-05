namespace EmuSen.WiseMan.Fixtures
{
    // The schema texts each version of media.db was written with, to build a file as an older build left it; the library's own are emusen-scrape's store.rs - see EmuSen_Backend.md §40.5.
    public static class MediaDbSchema
    {
        public static readonly string[] Steps =
        {
            """
            CREATE TABLE scrape_game (
                md5           TEXT    NOT NULL,
                bytes         INTEGER NOT NULL,
                status        TEXT    NOT NULL CHECK (status IN ('Found', 'Unknown', 'Error')),
                game_id       INTEGER,
                rom_id        INTEGER,
                system_id     INTEGER,
                name          TEXT,
                description   TEXT,
                developer     TEXT,
                publisher     TEXT,
                genre         TEXT,
                players       TEXT,
                rating        REAL,
                release_date  TEXT,
                region        TEXT,
                language      TEXT,
                matched_by    TEXT,
                offered       TEXT,
                detail        TEXT,
                fetched_at    TEXT    NOT NULL,
                PRIMARY KEY (md5, bytes)
            );
            CREATE TABLE scrape_media (
                md5         TEXT    NOT NULL,
                bytes       INTEGER NOT NULL,
                type        TEXT    NOT NULL,
                path        TEXT    NOT NULL,
                region      TEXT,
                sha1        TEXT,
                fetched_at  TEXT    NOT NULL,
                PRIMARY KEY (md5, bytes, type)
            );
            CREATE TABLE scrape_file (
                path      TEXT    PRIMARY KEY,
                bytes     INTEGER NOT NULL,
                modified  INTEGER NOT NULL,
                md5       TEXT    NOT NULL
            );
            CREATE TABLE scrape_queue (
                path      TEXT    PRIMARY KEY,
                system    TEXT    NOT NULL,
                priority  INTEGER NOT NULL,
                seq       INTEGER NOT NULL,
                attempts  INTEGER NOT NULL DEFAULT 0,
                next_try  TEXT
            );
            CREATE INDEX scrape_queue_order ON scrape_queue(priority, seq);
            CREATE TABLE quota_day (
                day             TEXT    PRIMARY KEY,
                requests        INTEGER NOT NULL DEFAULT 0,
                ko              INTEGER NOT NULL DEFAULT 0,
                max_per_day     INTEGER,
                max_ko_per_day  INTEGER,
                stopped_until   TEXT,
                stop_reason     TEXT
            );
            """,
            """
            CREATE TABLE store_move (
                from_path  TEXT    NOT NULL,
                to_path    TEXT    NOT NULL,
                how        TEXT    NOT NULL CHECK (how IN ('moved', 'copied', 'found')),
                at         TEXT    NOT NULL,
                PRIMARY KEY (from_path, to_path)
            );
            CREATE TABLE store_step (
                name     TEXT    PRIMARY KEY,
                done_at  TEXT    NOT NULL
            );
            """,
            """
            ALTER TABLE scrape_game ADD COLUMN kinds_known TEXT;
            """,
            """
            ALTER TABLE scrape_file ADD COLUMN crc32 TEXT;
            ALTER TABLE scrape_file ADD COLUMN sha1 TEXT;
            """,
            """
            CREATE TABLE scrape_game_v5 (
                md5           TEXT    NOT NULL,
                bytes         INTEGER NOT NULL,
                status        TEXT    NOT NULL CHECK (status IN ('Found', 'Unknown', 'Error')),
                game_id       INTEGER,
                rom_id        INTEGER,
                system_id     INTEGER,
                name          TEXT,
                description   TEXT,
                developer     TEXT,
                publisher     TEXT,
                genre         TEXT,
                players       TEXT,
                rating        REAL,
                release_date  TEXT,
                region        TEXT,
                language      TEXT,
                matched_by    TEXT,
                offered       TEXT,
                detail        TEXT,
                fetched_at    TEXT    NOT NULL,
                kinds_known   TEXT,
                fingerprint   TEXT    NOT NULL DEFAULT '',
                PRIMARY KEY (md5, bytes, fingerprint)
            );
            INSERT INTO scrape_game_v5 (md5, bytes, status, game_id, rom_id, system_id, name, description, developer, publisher, genre, players, rating, release_date, region, language, matched_by, offered, detail, fetched_at, kinds_known)
                SELECT md5, bytes, status, game_id, rom_id, system_id, name, description, developer, publisher, genre, players, rating, release_date, region, language, matched_by, offered, detail, fetched_at, kinds_known FROM scrape_game;
            DROP TABLE scrape_game;
            ALTER TABLE scrape_game_v5 RENAME TO scrape_game;
            CREATE TABLE scrape_media_v5 (
                md5          TEXT    NOT NULL,
                bytes        INTEGER NOT NULL,
                type         TEXT    NOT NULL,
                path         TEXT    NOT NULL,
                region       TEXT,
                sha1         TEXT,
                fetched_at   TEXT    NOT NULL,
                fingerprint  TEXT    NOT NULL DEFAULT '',
                PRIMARY KEY (md5, bytes, fingerprint, type)
            );
            INSERT INTO scrape_media_v5 (md5, bytes, type, path, region, sha1, fetched_at)
                SELECT md5, bytes, type, path, region, sha1, fetched_at FROM scrape_media;
            DROP TABLE scrape_media;
            ALTER TABLE scrape_media_v5 RENAME TO scrape_media;
            ALTER TABLE scrape_file ADD COLUMN fingerprint TEXT NOT NULL DEFAULT '';
            """,
        };

        // The version this build's library writes.
        public static int Version => Steps.Length;
    }
}
