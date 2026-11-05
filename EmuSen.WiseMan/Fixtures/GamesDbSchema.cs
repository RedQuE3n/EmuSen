namespace EmuSen.WiseMan.Fixtures
{
    // The schema texts each version of games.db was written with, to build a file as an older build left it; the library's own are emusen-library's db.rs - see EmuSen_Backend.md §40.4.
    public static class GamesDbSchema
    {
        public static readonly string[] Steps =
        {
            """
            CREATE TABLE game (
                path          TEXT    PRIMARY KEY,
                favourite     INTEGER NOT NULL DEFAULT 0 CHECK (favourite IN (0, 1)),
                last_played   TEXT,
                play_count    INTEGER NOT NULL DEFAULT 0,
                play_seconds  REAL    NOT NULL DEFAULT 0
            );
            CREATE INDEX game_by_last_played ON game(last_played);
            """,
            """
            ALTER TABLE game ADD COLUMN md5 TEXT;
            ALTER TABLE game ADD COLUMN bytes INTEGER;
            CREATE INDEX game_by_md5 ON game(md5);
            CREATE TABLE file_hash (
                path      TEXT    PRIMARY KEY,
                bytes     INTEGER NOT NULL,
                modified  INTEGER NOT NULL,
                md5       TEXT    NOT NULL
            );
            CREATE INDEX file_hash_by_md5 ON file_hash(md5);
            """,
            """
            CREATE TABLE collection (
                id        INTEGER PRIMARY KEY,
                name      TEXT    NOT NULL UNIQUE COLLATE NOCASE,
                created   TEXT    NOT NULL
            );
            CREATE TABLE collection_game (
                collection  INTEGER NOT NULL REFERENCES collection(id) ON DELETE CASCADE,
                path        TEXT    NOT NULL,
                PRIMARY KEY (collection, path)
            );
            CREATE INDEX collection_game_by_path ON collection_game(path);
            """,
            """
            CREATE TABLE cover_lookup (
                path      TEXT PRIMARY KEY,
                outcome   TEXT NOT NULL CHECK (outcome IN ('Found', 'Unknown', 'NoArt')),
                rom_name  TEXT,
                checked   TEXT NOT NULL
            );
            """,
            """
            CREATE TABLE game_edit (
                path    TEXT NOT NULL,
                field   TEXT NOT NULL,
                value   TEXT NOT NULL,
                edited  TEXT NOT NULL,
                PRIMARY KEY (path, field)
            );
            """,
            """
            ALTER TABLE game ADD COLUMN crop_left REAL;
            ALTER TABLE game ADD COLUMN crop_right REAL;
            ALTER TABLE game ADD COLUMN crop_top REAL;
            ALTER TABLE game ADD COLUMN crop_bottom REAL;
            """,
        };

        // The version this build's library writes.
        public static int Version => Steps.Length;
    }
}
