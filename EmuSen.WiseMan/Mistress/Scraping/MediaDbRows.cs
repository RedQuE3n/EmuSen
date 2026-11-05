using System;
using System.Globalization;
using System.IO;
using EmuSen.Crystal.Client.Scrape;
using EmuSen.Mistress.Scraping;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // media.db rows written as the store writes them, for a test that needs a store holding a game, a file or a queue before the scraper opens it; the store is made by the library first - see EmuSen_Backend.md §40.5.
    internal sealed class MediaDbRows : IDisposable
    {
        private readonly SqliteConnection _db;

        public MediaDbRows(string root)
        {
            string file = Path.Combine(root, MediaStore.FileName);
            if (!File.Exists(file)) MediaService.Open(root, "{\"developer_files\":false,\"test\":{\"offline\":true}}").Dispose();
            _db = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = file, Pooling = false }.ToString());
            _db.Open();
        }

        public static string Stamp(DateTimeOffset t) => t.UtcDateTime.ToString("yyyy-MM-ddTHH:mm:ss.fffZ", CultureInfo.InvariantCulture);

        private void Write(string sql, params (string Name, object? Value)[] parameters)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = sql;
            foreach ((string name, object? value) in parameters) command.Parameters.AddWithValue(name, value ?? DBNull.Value);
            command.ExecuteNonQuery();
        }

        public void RememberFile(string path, long bytes, long modified, string md5) => Write(
            "INSERT INTO scrape_file (path, bytes, modified, md5) VALUES ($path, $bytes, $modified, $md5) ON CONFLICT(path) DO UPDATE SET bytes = excluded.bytes, modified = excluded.modified, md5 = excluded.md5, fingerprint = ''",
            ("$path", path), ("$bytes", bytes), ("$modified", modified), ("$md5", md5));

        public void Record(ScrapedRecord r) => Write("""
            INSERT OR REPLACE INTO scrape_game (md5, bytes, status, game_id, rom_id, system_id, name, description, developer, publisher, genre, players, rating,
                release_date, region, language, matched_by, offered, detail, fetched_at, kinds_known, fingerprint)
            VALUES ($md5, $bytes, $status, $game, $rom, $system, $name, $description, $developer, $publisher, $genre, $players, $rating,
                $release, $region, $language, $matched, $offered, $detail, $fetched, $known, $fingerprint)
            """,
            ("$md5", r.Md5), ("$bytes", r.Bytes), ("$status", r.State.ToString()), ("$game", r.GameId), ("$rom", r.RomId), ("$system", r.SystemId),
            ("$name", r.Name), ("$description", r.Description), ("$developer", r.Developer), ("$publisher", r.Publisher), ("$genre", r.Genre),
            ("$players", r.Players), ("$rating", r.Rating), ("$release", r.ReleaseDate?.ToString("yyyy-MM-dd", CultureInfo.InvariantCulture)),
            ("$region", r.Region), ("$language", r.Language), ("$matched", r.MatchedBy), ("$offered", r.Offered), ("$detail", r.Detail),
            ("$fetched", r.FetchedAt.ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ", CultureInfo.InvariantCulture)), ("$known", r.KindsKnown), ("$fingerprint", r.Fingerprint));

        public void RecordMedia(string md5, long bytes, StoredMedia media, DateTimeOffset at) => Write(
            "INSERT OR REPLACE INTO scrape_media (md5, bytes, type, path, region, sha1, fetched_at) VALUES ($md5, $bytes, $type, $path, $region, $sha1, $at)",
            ("$md5", md5), ("$bytes", bytes), ("$type", media.Type), ("$path", media.RelativePath), ("$region", media.Region), ("$sha1", media.Sha1), ("$at", Stamp(at)));

        public void Enqueue(string path, string system, ScrapePriority priority) => Write("""
            INSERT INTO scrape_queue (path, system, priority, seq) VALUES ($path, $system, $priority, (SELECT COALESCE(MAX(seq), 0) + 1 FROM scrape_queue))
            ON CONFLICT(path) DO UPDATE SET priority = MIN(priority, excluded.priority)
            """, ("$path", path), ("$system", system), ("$priority", (int)priority));

        // A day's counts as the quota keeps them; day is Paris's, as ScreenScraper's.
        public void SaveDay(string day, int requests, int ko, int? maxPerDay, int? maxKoPerDay, DateTimeOffset? stoppedUntil, string? reason) => Write("""
            INSERT INTO quota_day (day, requests, ko, max_per_day, max_ko_per_day, stopped_until, stop_reason) VALUES ($day, $r, $k, $m, $mk, $s, $why)
            ON CONFLICT(day) DO UPDATE SET requests = excluded.requests, ko = excluded.ko, max_per_day = excluded.max_per_day,
                max_ko_per_day = excluded.max_ko_per_day, stopped_until = excluded.stopped_until, stop_reason = excluded.stop_reason
            """, ("$day", day), ("$r", requests), ("$k", ko), ("$m", maxPerDay), ("$mk", maxKoPerDay), ("$s", stoppedUntil is { } u ? Stamp(u) : null), ("$why", reason));

        public void Dispose() => _db.Dispose();
    }
}
