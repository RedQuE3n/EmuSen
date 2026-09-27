using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;
using Microsoft.Data.Sqlite;

namespace EmuSen.Mistress.Scraping
{
    public enum ScrapeState { Found, Unknown, Error }

    public enum ScrapePriority { Shown = 0, Console = 1, Library = 2 }

    // What a game's scrape kept: ScreenScraper's ids, the text of §3.7's metadata variants, and which media it offered.
    public sealed record ScrapedRecord(string Md5, long Bytes, ScrapeState State)
    {
        public long? GameId { get; init; }
        public long? RomId { get; init; }
        public int? SystemId { get; init; }
        public string? Name { get; init; }
        public string? Description { get; init; }
        public string? Developer { get; init; }
        public string? Publisher { get; init; }
        public string? Genre { get; init; }
        public string? Players { get; init; }
        public float? Rating { get; init; }
        public DateTime? ReleaseDate { get; init; }
        public string? Region { get; init; }
        public string? Language { get; init; }
        public string? MatchedBy { get; init; }
        public string? Offered { get; init; }
        public string? Detail { get; init; }
        public DateTime FetchedAt { get; init; }
    }

    public sealed record StoredMedia(string Type, string RelativePath, string? Region, string? Sha1);

    public sealed record QueuedGame(string Path, string System, ScrapePriority Priority, int Attempts);

    public sealed record QuotaDay(string Day, int Requests, int Ko, int? MaxPerDay, int? MaxKoPerDay, DateTimeOffset? StoppedUntil, string? StopReason);

    // media.db beside the media it describes: scraped games by hash, their files, the queue and the day's quota - see EmuSen_BigPicture.md §5.6 and §17.
    public sealed class MediaStore : IDisposable
    {
        // Each entry takes the file from the version before it; append, never edit.
        private static readonly string[] Migrations =
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
        };

        public static int SchemaVersion => Migrations.Length;

        public const string FileName = "media.db";

        private readonly SqliteConnection _db;
        private readonly object _gate = new();
        private bool _closed;

        private MediaStore(SqliteConnection db, string root)
        {
            _db = db;
            Root = root;
        }

        // The media folder: <root>/<es-de system>/<type folder>/<rom stem>.<ext>, and media.db in it.
        public string Root { get; }

        public bool IsOpen => !_closed;

        public static string DefaultRoot => DataStore.Media;

        public static MediaStore Open(string root)
        {
            Directory.CreateDirectory(root);
            var db = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = Path.Combine(root, FileName), Pooling = false }.ToString());
            db.Open();
            try
            {
                Migrate(db);
            }
            catch
            {
                db.Dispose();
                throw;
            }
            return new MediaStore(db, root);
        }

        // A file newer than this build is refused rather than written to, as games.db is (§4.32 of the settings reference).
        private static void Migrate(SqliteConnection db)
        {
            int version = Convert.ToInt32(Scalar(db, null, "PRAGMA user_version"), CultureInfo.InvariantCulture);
            if (version > Migrations.Length) throw new InvalidDataException($"media.db is schema {version}; this build knows {Migrations.Length}.");
            for (; version < Migrations.Length; version++)
            {
                using SqliteTransaction step = db.BeginTransaction();
                Execute(db, step, Migrations[version]);
                Execute(db, step, $"PRAGMA user_version = {version + 1}");
                step.Commit();
            }
        }

        // --- the queue ---

        // A game already queued keeps its place unless the new priority is higher; a new one goes to the back of its priority.
        public bool Enqueue(string path, string system, ScrapePriority priority)
        {
            lock (_gate)
            {
                if (_closed) return false;
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = """
                    INSERT INTO scrape_queue (path, system, priority, seq) VALUES ($path, $system, $priority, (SELECT COALESCE(MAX(seq), 0) + 1 FROM scrape_queue))
                    ON CONFLICT(path) DO UPDATE SET priority = MIN(priority, excluded.priority), next_try = CASE WHEN excluded.priority < priority THEN NULL ELSE next_try END
                    """;
                command.Parameters.AddWithValue("$path", path);
                command.Parameters.AddWithValue("$system", system);
                command.Parameters.AddWithValue("$priority", (int)priority);
                return command.ExecuteNonQuery() > 0;
            }
        }

        // The first due game by priority then arrival, skipping those another worker holds.
        public QueuedGame? NextDue(DateTimeOffset now, ICollection<string> held)
        {
            lock (_gate)
            {
                if (_closed) return null;
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = "SELECT path, system, priority, attempts FROM scrape_queue WHERE next_try IS NULL OR next_try <= $now ORDER BY priority, seq";
                command.Parameters.AddWithValue("$now", Stamp(now));
                using SqliteDataReader row = command.ExecuteReader();
                while (row.Read())
                {
                    if (held.Contains(row.GetString(0))) continue;
                    return new QueuedGame(row.GetString(0), row.GetString(1), (ScrapePriority)row.GetInt32(2), row.GetInt32(3));
                }
                return null;
            }
        }

        public DateTimeOffset? EarliestRetry()
        {
            lock (_gate)
            {
                if (_closed) return null;
                object? value = Scalar(_db, null, "SELECT MIN(next_try) FROM scrape_queue WHERE next_try IS NOT NULL");
                return value is string s ? DateTimeOffset.Parse(s, CultureInfo.InvariantCulture) : null;
            }
        }

        public void Retry(string path, DateTimeOffset nextTry, bool countAttempt) => Write(
            countAttempt
                ? "UPDATE scrape_queue SET attempts = attempts + 1, next_try = $a WHERE path = $path"
                : "UPDATE scrape_queue SET next_try = $a WHERE path = $path",
            ("$path", path), ("$a", Stamp(nextTry)));

        public void Dequeue(string path) => Write("DELETE FROM scrape_queue WHERE path = $path", ("$path", path));

        // A new run replaces what an interrupted one left; Resume is the only way to go on with that.
        public void ClearQueue() => Write("DELETE FROM scrape_queue");

        // "Scrape this game" asks again even where ScreenScraper once had nothing: an Unknown or Error answer is forgotten.
        public void ForgetUnfound(string path) => Write(
            "DELETE FROM scrape_game WHERE status <> 'Found' AND EXISTS (SELECT 1 FROM scrape_file f WHERE f.path = $path AND f.md5 = scrape_game.md5 AND f.bytes = scrape_game.bytes)",
            ("$path", path));

        public int QueueLength => (int)(long)(ReadScalar("SELECT COUNT(*) FROM scrape_queue") ?? 0L);

        public IReadOnlyList<QueuedGame> Queue()
        {
            var all = new List<QueuedGame>();
            lock (_gate)
            {
                if (_closed) return all;
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = "SELECT path, system, priority, attempts FROM scrape_queue ORDER BY priority, seq";
                using SqliteDataReader row = command.ExecuteReader();
                while (row.Read()) all.Add(new QueuedGame(row.GetString(0), row.GetString(1), (ScrapePriority)row.GetInt32(2), row.GetInt32(3)));
            }
            return all;
        }

        // --- the file hash cache ---

        public string? CachedMd5(string path, long bytes, long modified)
        {
            lock (_gate)
            {
                if (_closed) return null;
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = "SELECT md5 FROM scrape_file WHERE path = $path AND bytes = $bytes AND modified = $modified";
                command.Parameters.AddWithValue("$path", path);
                command.Parameters.AddWithValue("$bytes", bytes);
                command.Parameters.AddWithValue("$modified", modified);
                return command.ExecuteScalar() as string;
            }
        }

        public void RememberFile(string path, long bytes, long modified, string md5) => Write(
            "INSERT INTO scrape_file (path, bytes, modified, md5) VALUES ($path, $bytes, $modified, $md5) ON CONFLICT(path) DO UPDATE SET bytes = excluded.bytes, modified = excluded.modified, md5 = excluded.md5",
            ("$path", path), ("$bytes", bytes), ("$modified", modified), ("$md5", md5));

        // Every file last hashed to a game, for telling a renamed file from a second copy.
        public IReadOnlyList<string> PathsOf(string md5, long bytes)
        {
            var all = new List<string>();
            lock (_gate)
            {
                if (_closed) return all;
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = "SELECT path FROM scrape_file WHERE md5 = $md5 AND bytes = $bytes";
                command.Parameters.AddWithValue("$md5", md5);
                command.Parameters.AddWithValue("$bytes", bytes);
                using SqliteDataReader row = command.ExecuteReader();
                while (row.Read()) all.Add(row.GetString(0));
            }
            return all;
        }

        // --- games and media ---

        public void Record(ScrapedRecord r) => Write("""
            INSERT OR REPLACE INTO scrape_game (md5, bytes, status, game_id, rom_id, system_id, name, description, developer, publisher, genre, players, rating,
                release_date, region, language, matched_by, offered, detail, fetched_at)
            VALUES ($md5, $bytes, $status, $game, $rom, $system, $name, $description, $developer, $publisher, $genre, $players, $rating,
                $release, $region, $language, $matched, $offered, $detail, $fetched)
            """,
            ("$md5", r.Md5), ("$bytes", r.Bytes), ("$status", r.State.ToString()), ("$game", r.GameId), ("$rom", r.RomId), ("$system", r.SystemId),
            ("$name", r.Name), ("$description", r.Description), ("$developer", r.Developer), ("$publisher", r.Publisher), ("$genre", r.Genre),
            ("$players", r.Players), ("$rating", r.Rating), ("$release", r.ReleaseDate?.ToString("yyyy-MM-dd", CultureInfo.InvariantCulture)),
            ("$region", r.Region), ("$language", r.Language), ("$matched", r.MatchedBy), ("$offered", r.Offered), ("$detail", r.Detail),
            ("$fetched", Stamp(r.FetchedAt)));

        public ScrapedRecord? Game(string md5, long bytes)
        {
            lock (_gate)
            {
                if (_closed) return null;
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = GameColumns + " WHERE md5 = $md5 AND bytes = $bytes";
                command.Parameters.AddWithValue("$md5", md5);
                command.Parameters.AddWithValue("$bytes", bytes);
                using SqliteDataReader row = command.ExecuteReader();
                return row.Read() ? ReadGame(row) : null;
            }
        }

        // Every found game by the path last hashed to it, for a library that shows many at once.
        public IReadOnlyDictionary<string, ScrapedRecord> FoundByPath()
        {
            var all = new Dictionary<string, ScrapedRecord>(StringComparer.Ordinal);
            lock (_gate)
            {
                if (_closed) return all;
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = "SELECT f.path, g.md5, g.bytes, g.status, g.game_id, g.rom_id, g.system_id, g.name, g.description, g.developer, g.publisher, g.genre, g.players, g.rating, g.release_date, g.region, g.language, g.matched_by, g.offered, g.detail, g.fetched_at FROM scrape_file f JOIN scrape_game g ON g.md5 = f.md5 AND g.bytes = f.bytes WHERE g.status = 'Found'";
                using SqliteDataReader row = command.ExecuteReader();
                while (row.Read()) all[row.GetString(0)] = ReadGame(row, 1);
            }
            return all;
        }

        // One file's found game, read now rather than from a snapshot, for an editor waiting on its own scrape.
        public ScrapedRecord? FoundFor(string path)
        {
            lock (_gate)
            {
                if (_closed) return null;
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = "SELECT g.md5, g.bytes, g.status, g.game_id, g.rom_id, g.system_id, g.name, g.description, g.developer, g.publisher, g.genre, g.players, g.rating, g.release_date, g.region, g.language, g.matched_by, g.offered, g.detail, g.fetched_at FROM scrape_file f JOIN scrape_game g ON g.md5 = f.md5 AND g.bytes = f.bytes WHERE g.status = 'Found' AND f.path = $path";
                command.Parameters.AddWithValue("$path", path);
                using SqliteDataReader row = command.ExecuteReader();
                return row.Read() ? ReadGame(row) : null;
            }
        }

        // Every answered file by path: whether ScreenScraper knew it, and whether a cover was kept; the failover reads this to know where ScreenScraper had nothing.
        public IReadOnlyDictionary<string, (ScrapeState State, bool HasCover)> OutcomesByPath()
        {
            var all = new Dictionary<string, (ScrapeState, bool)>(StringComparer.Ordinal);
            lock (_gate)
            {
                if (_closed) return all;
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = """
                    SELECT f.path, g.status, EXISTS (SELECT 1 FROM scrape_media m WHERE m.md5 = g.md5 AND m.bytes = g.bytes AND m.type = 'cover')
                    FROM scrape_file f JOIN scrape_game g ON g.md5 = f.md5 AND g.bytes = f.bytes
                    """;
                using SqliteDataReader row = command.ExecuteReader();
                while (row.Read()) all[row.GetString(0)] = (Enum.Parse<ScrapeState>(row.GetString(1)), row.GetInt64(2) != 0);
            }
            return all;
        }

        public void RecordMedia(string md5, long bytes, StoredMedia media, DateTimeOffset now) => Write(
            "INSERT OR REPLACE INTO scrape_media (md5, bytes, type, path, region, sha1, fetched_at) VALUES ($md5, $bytes, $type, $path, $region, $sha1, $at)",
            ("$md5", md5), ("$bytes", bytes), ("$type", media.Type), ("$path", media.RelativePath), ("$region", media.Region), ("$sha1", media.Sha1), ("$at", Stamp(now)));

        public IReadOnlyList<StoredMedia> Media(string md5, long bytes)
        {
            var all = new List<StoredMedia>();
            lock (_gate)
            {
                if (_closed) return all;
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = "SELECT type, path, region, sha1 FROM scrape_media WHERE md5 = $md5 AND bytes = $bytes ORDER BY type";
                command.Parameters.AddWithValue("$md5", md5);
                command.Parameters.AddWithValue("$bytes", bytes);
                using SqliteDataReader row = command.ExecuteReader();
                while (row.Read()) all.Add(new StoredMedia(row.GetString(0), row.GetString(1), row.IsDBNull(2) ? null : row.GetString(2), row.IsDBNull(3) ? null : row.GetString(3)));
            }
            return all;
        }

        // ES-DE's Clear for one game: its answer and its pictures go, from media.db and from this store's folders only; the ROM, the player's covers and an ES-DE folder are never touched - see EmuSen_Settings_Reference.md §4.59.
        public IReadOnlyList<string> Forget(string path, string? system)
        {
            var files = new List<string>();
            lock (_gate)
            {
                if (_closed) return files;
                using (SqliteCommand find = _db.CreateCommand())
                {
                    find.CommandText = "SELECT m.path FROM scrape_file f JOIN scrape_media m ON m.md5 = f.md5 AND m.bytes = f.bytes WHERE f.path = $path";
                    find.Parameters.AddWithValue("$path", path);
                    using SqliteDataReader row = find.ExecuteReader();
                    while (row.Read()) files.Add(System.IO.Path.Combine(Root, row.GetString(0)));
                }
                using SqliteTransaction step = _db.BeginTransaction();
                foreach (string table in new[] { "scrape_media", "scrape_game" })
                {
                    using SqliteCommand drop = _db.CreateCommand();
                    drop.Transaction = step;
                    drop.CommandText = $"DELETE FROM {table} WHERE EXISTS (SELECT 1 FROM scrape_file f WHERE f.path = $path AND f.md5 = {table}.md5 AND f.bytes = {table}.bytes)";
                    drop.Parameters.AddWithValue("$path", path);
                    drop.ExecuteNonQuery();
                }
                using (SqliteCommand queue = _db.CreateCommand())
                {
                    queue.Transaction = step;
                    queue.CommandText = "DELETE FROM scrape_queue WHERE path = $path";
                    queue.Parameters.AddWithValue("$path", path);
                    queue.ExecuteNonQuery();
                }
                step.Commit();
            }

            if (!string.IsNullOrEmpty(system))
            {
                string stem = System.IO.Path.GetFileNameWithoutExtension(path);
                foreach (string folder in BigPicture.Scene.EsdeMediaFolder.Folders.Values)
                {
                    string dir = System.IO.Path.Combine(Root, system, folder);
                    if (Directory.Exists(dir)) files.AddRange(Directory.EnumerateFiles(dir, stem + ".*").Where(f => System.IO.Path.GetFileNameWithoutExtension(f) == stem));
                }
            }

            string root = System.IO.Path.GetFullPath(Root).TrimEnd(System.IO.Path.DirectorySeparatorChar) + System.IO.Path.DirectorySeparatorChar;
            var deleted = new List<string>();
            foreach (string file in files.Select(System.IO.Path.GetFullPath).Distinct(StringComparer.Ordinal))
            {
                if (!file.StartsWith(root, StringComparison.Ordinal) || !File.Exists(file)) continue;
                File.Delete(file);
                deleted.Add(file);
            }
            return deleted;
        }

        // --- the quota's day ---

        public QuotaDay Day(string day)
        {
            lock (_gate)
            {
                if (_closed) return new QuotaDay(day, 0, 0, null, null, null, null);
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = "SELECT requests, ko, max_per_day, max_ko_per_day, stopped_until, stop_reason FROM quota_day WHERE day = $day";
                command.Parameters.AddWithValue("$day", day);
                using SqliteDataReader row = command.ExecuteReader();
                if (!row.Read()) return new QuotaDay(day, 0, 0, null, null, null, null);
                return new QuotaDay(day, row.GetInt32(0), row.GetInt32(1), row.IsDBNull(2) ? null : row.GetInt32(2), row.IsDBNull(3) ? null : row.GetInt32(3),
                    row.IsDBNull(4) ? null : DateTimeOffset.Parse(row.GetString(4), CultureInfo.InvariantCulture), row.IsDBNull(5) ? null : row.GetString(5));
            }
        }

        public void SaveDay(QuotaDay d) => Write("""
            INSERT INTO quota_day (day, requests, ko, max_per_day, max_ko_per_day, stopped_until, stop_reason) VALUES ($day, $r, $k, $m, $mk, $s, $why)
            ON CONFLICT(day) DO UPDATE SET requests = excluded.requests, ko = excluded.ko, max_per_day = excluded.max_per_day,
                max_ko_per_day = excluded.max_ko_per_day, stopped_until = excluded.stopped_until, stop_reason = excluded.stop_reason
            """,
            ("$day", d.Day), ("$r", d.Requests), ("$k", d.Ko), ("$m", d.MaxPerDay), ("$mk", d.MaxKoPerDay),
            ("$s", d.StoppedUntil is { } s ? Stamp(s) : null), ("$why", d.StopReason));

        // --- plumbing ---

        private const string GameColumns = "SELECT md5, bytes, status, game_id, rom_id, system_id, name, description, developer, publisher, genre, players, rating, release_date, region, language, matched_by, offered, detail, fetched_at FROM scrape_game";

        private static ScrapedRecord ReadGame(SqliteDataReader row, int o = 0) => new(row.GetString(o), row.GetInt64(o + 1), Enum.Parse<ScrapeState>(row.GetString(o + 2)))
        {
            GameId = row.IsDBNull(o + 3) ? null : row.GetInt64(o + 3),
            RomId = row.IsDBNull(o + 4) ? null : row.GetInt64(o + 4),
            SystemId = row.IsDBNull(o + 5) ? null : row.GetInt32(o + 5),
            Name = Text(row, o + 6), Description = Text(row, o + 7), Developer = Text(row, o + 8), Publisher = Text(row, o + 9), Genre = Text(row, o + 10),
            Players = Text(row, o + 11), Rating = row.IsDBNull(o + 12) ? null : (float)row.GetDouble(o + 12),
            ReleaseDate = Text(row, o + 13) is string d ? DateTime.ParseExact(d, "yyyy-MM-dd", CultureInfo.InvariantCulture) : null,
            Region = Text(row, o + 14), Language = Text(row, o + 15), MatchedBy = Text(row, o + 16), Offered = Text(row, o + 17), Detail = Text(row, o + 18),
            FetchedAt = DateTime.Parse(row.GetString(o + 19), CultureInfo.InvariantCulture, DateTimeStyles.RoundtripKind),
        };

        private static string? Text(SqliteDataReader row, int i) => row.IsDBNull(i) ? null : row.GetString(i);

        private static string Stamp(DateTimeOffset t) => t.UtcDateTime.ToString("yyyy-MM-ddTHH:mm:ss.fffZ", CultureInfo.InvariantCulture);

        private static string Stamp(DateTime t) => t.ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ", CultureInfo.InvariantCulture);

        private void Write(string sql, params (string Name, object? Value)[] values)
        {
            lock (_gate)
            {
                if (_closed) return;
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = sql;
                foreach ((string name, object? value) in values) command.Parameters.AddWithValue(name, value ?? DBNull.Value);
                command.ExecuteNonQuery();
            }
        }

        private object? ReadScalar(string sql)
        {
            lock (_gate) return _closed ? null : Scalar(_db, null, sql);
        }

        private static object? Scalar(SqliteConnection db, SqliteTransaction? tx, string sql)
        {
            using SqliteCommand command = db.CreateCommand();
            command.Transaction = tx;
            command.CommandText = sql;
            return command.ExecuteScalar();
        }

        private static void Execute(SqliteConnection db, SqliteTransaction tx, string sql)
        {
            using SqliteCommand command = db.CreateCommand();
            command.Transaction = tx;
            command.CommandText = sql;
            command.ExecuteNonQuery();
        }

        public void Dispose()
        {
            lock (_gate)
            {
                if (_closed) return;
                _closed = true;
                _db.Dispose();
            }
        }
    }
}
