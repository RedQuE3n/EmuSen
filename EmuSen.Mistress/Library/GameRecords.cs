using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;
using Microsoft.Data.Sqlite;

namespace EmuSen.Mistress.Library
{
    // What Mistress remembers about one game it has seen - see EmuSen_Settings_Reference.md §4.32.
    public sealed class GameRecord
    {
        public bool Favourite { get; set; }
        public DateTime? LastPlayed { get; set; }
        public int PlayCount { get; set; }
        public double PlaySeconds { get; set; }
    }

    // One of the player's own collections, with how many games it holds.
    public sealed record GameCollection(long Id, string Name, int Count);

    // The player's own library in SQLite: kept, unlike the catalogue cache, so it migrates rather than rebuilds - see EmuSen_Settings_Reference.md §4.32.
    public sealed class GameRecords : IDisposable
    {
        // Each entry takes the file from the version before it; append, never edit - see §4.32.
        private static readonly string[] Migrations =
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
        };

        public static int SchemaVersion => Migrations.Length;

        private readonly SqliteConnection _db;

        private GameRecords(SqliteConnection db) => _db = db;

        public static string DefaultPath => Path.Combine(DataStore.Library, "games.db");

        public static GameRecords Load() => Open(DefaultPath);

        public static GameRecords Open(string path)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            var db = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = path, Pooling = false }.ToString());
            db.Open();
            using (SqliteCommand keys = db.CreateCommand()) { keys.CommandText = "PRAGMA foreign_keys = ON"; keys.ExecuteNonQuery(); }
            Migrate(db);
            return new GameRecords(db);
        }

        // A file newer than this build is refused rather than written to - see §4.32.
        private static void Migrate(SqliteConnection db)
        {
            int version = Convert.ToInt32(Scalar(db, "PRAGMA user_version"));
            if (version > Migrations.Length) throw new InvalidDataException($"games.db is schema {version}; this build knows {Migrations.Length}.");
            for (; version < Migrations.Length; version++)
            {
                using SqliteTransaction step = db.BeginTransaction();
                Execute(db, step, Migrations[version]);
                Execute(db, step, $"PRAGMA user_version = {version + 1}");
                step.Commit();
            }
        }

        public GameRecord? Find(string path)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "SELECT favourite, last_played, play_count, play_seconds FROM game WHERE path = $path";
            command.Parameters.AddWithValue("$path", path);
            using SqliteDataReader row = command.ExecuteReader();
            return row.Read() ? Record(row, 0) : null;
        }

        // Every stored row, for a library that shows many games at once.
        public IReadOnlyDictionary<string, GameRecord> All()
        {
            var all = new Dictionary<string, GameRecord>(StringComparer.Ordinal);
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "SELECT path, favourite, last_played, play_count, play_seconds FROM game";
            using SqliteDataReader row = command.ExecuteReader();
            while (row.Read()) all[row.GetString(0)] = Record(row, 1);
            return all;
        }

        public bool IsFavourite(string path) => Find(path)?.Favourite == true;

        public DateTime? LastPlayed(string path) => Find(path)?.LastPlayed;

        public void ToggleFavourite(string path) => Write(path,
            "INSERT INTO game (path, favourite) VALUES ($path, 1) ON CONFLICT(path) DO UPDATE SET favourite = 1 - favourite");

        public void Started(string path, DateTime now) => Write(path,
            "INSERT INTO game (path, play_count, last_played) VALUES ($path, 1, $value) ON CONFLICT(path) DO UPDATE SET play_count = play_count + 1, last_played = excluded.last_played",
            now.ToString("o", CultureInfo.InvariantCulture));

        public void Played(string path, TimeSpan time) => Write(path,
            "INSERT INTO game (path, play_seconds) VALUES ($path, $value) ON CONFLICT(path) DO UPDATE SET play_seconds = play_seconds + excluded.play_seconds",
            time.TotalSeconds);

        private void Write(string path, string sql, object? value = null)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = sql;
            command.Parameters.AddWithValue("$path", path);
            if (value is not null) command.Parameters.AddWithValue("$value", value);
            command.ExecuteNonQuery();
        }

        // --- Identity by hash, so a renamed file keeps its row - see EmuSen_Settings_Reference.md §4.37.

        // The cached hash of a file, while its size and modification time are the ones it was taken at.
        public string? KnownHash(string path, long bytes, long modified)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "SELECT md5 FROM file_hash WHERE path = $path AND bytes = $bytes AND modified = $modified";
            command.Parameters.AddWithValue("$path", path);
            command.Parameters.AddWithValue("$bytes", bytes);
            command.Parameters.AddWithValue("$modified", modified);
            return command.ExecuteScalar() as string;
        }

        public void StoreHash(string path, long bytes, long modified, string md5)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "INSERT INTO file_hash (path, bytes, modified, md5) VALUES ($path, $bytes, $modified, $md5) "
                + "ON CONFLICT(path) DO UPDATE SET bytes = excluded.bytes, modified = excluded.modified, md5 = excluded.md5";
            command.Parameters.AddWithValue("$path", path);
            command.Parameters.AddWithValue("$bytes", bytes);
            command.Parameters.AddWithValue("$modified", modified);
            command.Parameters.AddWithValue("$md5", md5);
            command.ExecuteNonQuery();
        }

        // Some file that had this hash when it was last looked at, for a state that names its ROM by contents.
        public string? PathByHash(string md5)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "SELECT path FROM file_hash WHERE md5 = $md5 ORDER BY path LIMIT 1";
            command.Parameters.AddWithValue("$md5", md5);
            return command.ExecuteScalar() as string;
        }

        // Gives a game's row the identity a rename cannot take away.
        public void Identify(string path, string md5, long bytes)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "INSERT INTO game (path, md5, bytes) VALUES ($path, $md5, $bytes) ON CONFLICT(path) DO UPDATE SET md5 = excluded.md5, bytes = excluded.bytes";
            command.Parameters.AddWithValue("$path", path);
            command.Parameters.AddWithValue("$md5", md5);
            command.Parameters.AddWithValue("$bytes", bytes);
            command.ExecuteNonQuery();
        }

        // Rows whose file is not among those present, and which carry the hash that could find it again.
        public IReadOnlyList<(string Path, string Md5, long Bytes)> Orphans(IReadOnlySet<string> present)
        {
            var orphans = new List<(string, string, long)>();
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "SELECT path, md5, bytes FROM game WHERE md5 IS NOT NULL AND bytes IS NOT NULL";
            using SqliteDataReader row = command.ExecuteReader();
            while (row.Read())
                if (!present.Contains(row.GetString(0))) orphans.Add((row.GetString(0), row.GetString(1), row.GetInt64(2)));
            return orphans;
        }

        // A renamed or moved game: its row and its collections follow it, and a row the new path already had is merged in.
        public void Move(string from, string to)
        {
            using SqliteTransaction step = _db.BeginTransaction();
            Execute(_db, step, "INSERT INTO game (path) VALUES ($to) ON CONFLICT(path) DO NOTHING", ("$to", to));
            Execute(_db, step, """
                UPDATE game SET
                    favourite    = MAX(favourite, (SELECT favourite FROM game WHERE path = $from)),
                    play_count   = play_count + (SELECT play_count FROM game WHERE path = $from),
                    play_seconds = play_seconds + (SELECT play_seconds FROM game WHERE path = $from),
                    last_played  = MAX(COALESCE(last_played, ''), COALESCE((SELECT last_played FROM game WHERE path = $from), '')),
                    md5          = COALESCE(md5, (SELECT md5 FROM game WHERE path = $from)),
                    bytes        = COALESCE(bytes, (SELECT bytes FROM game WHERE path = $from))
                WHERE path = $to
                """, ("$from", from), ("$to", to));
            Execute(_db, step, "UPDATE game SET last_played = NULL WHERE path = $to AND last_played = ''", ("$to", to));
            Execute(_db, step, "UPDATE OR IGNORE collection_game SET path = $to WHERE path = $from", ("$from", from), ("$to", to));
            Execute(_db, step, "DELETE FROM collection_game WHERE path = $from", ("$from", from));
            Execute(_db, step, "DELETE FROM game WHERE path = $from", ("$from", from));
            step.Commit();
        }

        // --- Collections the player makes - see EmuSen_Settings_Reference.md §4.38.

        public IReadOnlyList<GameCollection> Collections()
        {
            var all = new List<GameCollection>();
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "SELECT c.id, c.name, COUNT(g.path) FROM collection c LEFT JOIN collection_game g ON g.collection = c.id GROUP BY c.id ORDER BY c.name COLLATE NOCASE";
            using SqliteDataReader row = command.ExecuteReader();
            while (row.Read()) all.Add(new GameCollection(row.GetInt64(0), row.GetString(1), row.GetInt32(2)));
            return all;
        }

        // Null when a collection of that name, in any case, already exists.
        public long? CreateCollection(string name, DateTime now)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "INSERT INTO collection (name, created) VALUES ($name, $now) ON CONFLICT(name) DO NOTHING RETURNING id";
            command.Parameters.AddWithValue("$name", name);
            command.Parameters.AddWithValue("$now", now.ToString("o", CultureInfo.InvariantCulture));
            return command.ExecuteScalar() is long id ? id : null;
        }

        // False when another collection already has the name.
        public bool RenameCollection(long id, string name)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "UPDATE OR IGNORE collection SET name = $name WHERE id = $id";
            command.Parameters.AddWithValue("$name", name);
            command.Parameters.AddWithValue("$id", id);
            command.ExecuteNonQuery();
            return Collections().Any(c => c.Id == id && c.Name == name);
        }

        public void DeleteCollection(long id)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "DELETE FROM collection WHERE id = $id";
            command.Parameters.AddWithValue("$id", id);
            command.ExecuteNonQuery();
        }

        public void AddToCollection(long id, string path)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "INSERT OR IGNORE INTO collection_game (collection, path) VALUES ($id, $path)";
            command.Parameters.AddWithValue("$id", id);
            command.Parameters.AddWithValue("$path", path);
            command.ExecuteNonQuery();
        }

        public void RemoveFromCollection(long id, string path)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "DELETE FROM collection_game WHERE collection = $id AND path = $path";
            command.Parameters.AddWithValue("$id", id);
            command.Parameters.AddWithValue("$path", path);
            command.ExecuteNonQuery();
        }

        public IReadOnlySet<string> Members(long id)
        {
            var members = new HashSet<string>(StringComparer.Ordinal);
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "SELECT path FROM collection_game WHERE collection = $id";
            command.Parameters.AddWithValue("$id", id);
            using SqliteDataReader row = command.ExecuteReader();
            while (row.Read()) members.Add(row.GetString(0));
            return members;
        }

        public IReadOnlySet<long> CollectionsOf(string path)
        {
            var ids = new HashSet<long>();
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "SELECT collection FROM collection_game WHERE path = $path";
            command.Parameters.AddWithValue("$path", path);
            using SqliteDataReader row = command.ExecuteReader();
            while (row.Read()) ids.Add(row.GetInt64(0));
            return ids;
        }

        private static void Execute(SqliteConnection db, SqliteTransaction transaction, string sql, params (string Name, object Value)[] parameters)
        {
            using SqliteCommand command = db.CreateCommand();
            command.Transaction = transaction;
            command.CommandText = sql;
            foreach ((string name, object value) in parameters) command.Parameters.AddWithValue(name, value);
            command.ExecuteNonQuery();
        }

        private static GameRecord Record(SqliteDataReader row, int first) => new()
        {
            Favourite = row.GetInt64(first) != 0,
            LastPlayed = row.IsDBNull(first + 1) ? null : DateTime.Parse(row.GetString(first + 1), CultureInfo.InvariantCulture, DateTimeStyles.RoundtripKind),
            PlayCount = row.GetInt32(first + 2),
            PlaySeconds = row.GetDouble(first + 3),
        };

        private static object? Scalar(SqliteConnection db, string sql)
        {
            using SqliteCommand command = db.CreateCommand();
            command.CommandText = sql;
            return command.ExecuteScalar();
        }

        private static void Execute(SqliteConnection db, SqliteTransaction transaction, string sql)
        {
            using SqliteCommand command = db.CreateCommand();
            command.Transaction = transaction;
            command.CommandText = sql;
            command.ExecuteNonQuery();
        }

        public void Dispose() => _db.Dispose();
    }
}
