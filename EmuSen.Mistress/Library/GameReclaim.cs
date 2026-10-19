using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Galaxia.Library;

namespace EmuSen.Mistress.Library
{
    // A game known by its contents, and rows that follow a renamed game, out of the window: the library's identity and reclaim, and their C# twins - see EmuSen_Settings_Reference.md §4.37, EmuSen_Backend.md §28.5.
    public sealed partial class GameRecords
    {
        // A file's length and LastWriteTimeUtc ticks, or null where FileInfo says it does not exist.
        public static (long Bytes, long Modified)? Stat(string path)
        {
            try
            {
                var file = new FileInfo(path);
                return file.Exists ? (file.Length, file.LastWriteTimeUtc.Ticks) : null;
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                return null;
            }
        }

        public static string? TryMd5(string path)
        {
            try { return RomHash.Md5(path); }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException) { return null; }
        }

        // The file's hash from the cache or read, stored, and given to the game's row; null where it cannot be read.
        public (string Md5, long Bytes)? IdentifyFile(string path)
        {
            if (_native is { } n)
            {
                var stream = new MemoryStream();
                using (var w = new Utf8JsonWriter(stream))
                {
                    w.WriteStartObject();
                    w.WriteString("path", path);
                    w.WriteBoolean("record", true);
                    w.WriteEndObject();
                }
                try
                {
                    using JsonDocument d = JsonDocument.Parse(n.Lib.Identity(System.Text.Encoding.UTF8.GetString(stream.ToArray())));
                    return (d.RootElement.GetProperty("md5").GetString()!, d.RootElement.GetProperty("bytes").GetInt64());
                }
                catch (EmuSen.Crystal.Client.Library.GameLibraryException e) when (e.Status == EmuSen.Crystal.Client.Library.GameLibraryException.Io) { return null; }
            }
            if (Stat(path) is not { } stat) return null;
            string? md5 = KnownHash(path, stat.Bytes, stat.Modified);
            if (md5 is null)
            {
                md5 = TryMd5(path);
                if (md5 is null) return null;
                StoreHash(path, stat.Bytes, stat.Modified, md5);
            }
            Identify(path, md5, stat.Bytes);
            return (md5, stat.Bytes);
        }

        // Rows whose file is not among games, each moved to the first game of its size and contents that has no row; only files of a missing one's size are hashed. How many moved.
        public int Reclaim(IReadOnlyList<string> games)
        {
            if (_native is { } n)
            {
                var stream = new MemoryStream();
                using (var w = new Utf8JsonWriter(stream))
                {
                    w.WriteStartObject();
                    w.WriteString("op", "reclaim");
                    w.WriteStartArray("games");
                    foreach (string g in games) w.WriteStringValue(g);
                    w.WriteEndArray();
                    w.WriteEndObject();
                }
                using JsonDocument d = JsonDocument.Parse(n.Lib.Change(System.Text.Encoding.UTF8.GetString(stream.ToArray())));
                return d.RootElement.GetProperty("moved").GetInt32();
            }
            var present = new HashSet<string>(games, StringComparer.Ordinal);
            IReadOnlyList<(string Path, string Md5, long Bytes)> orphans = Orphans(present);
            if (orphans.Count == 0) return 0;
            var sizes = orphans.Select(o => o.Bytes).ToHashSet();
            IReadOnlyDictionary<string, GameRecord> rows = All();
            var candidates = new List<(string Path, long Bytes, long Modified, string? Md5)>();
            foreach (string path in games)
            {
                if (rows.ContainsKey(path) || Stat(path) is not { } stat || !sizes.Contains(stat.Bytes)) continue;
                candidates.Add((path, stat.Bytes, stat.Modified, KnownHash(path, stat.Bytes, stat.Modified)));
            }
            if (candidates.Count == 0) return 0;
            var hashed = candidates.Select(c => c with { Md5 = c.Md5 ?? TryMd5(c.Path) }).ToList();
            foreach (var c in hashed.Where(c => c.Md5 is not null)) StoreHash(c.Path, c.Bytes, c.Modified, c.Md5!);
            var claimed = new HashSet<string>(StringComparer.Ordinal);
            int moved = 0;
            foreach ((string path, string md5, long _) in orphans)
            {
                var match = hashed.FirstOrDefault(c => c.Md5 == md5 && !claimed.Contains(c.Path));
                if (match.Path is null) continue;
                Move(path, match.Path);
                claimed.Add(match.Path);
                moved++;
            }
            return moved;
        }

        // Of games, those with a last-played time, newest first by the instant each names, at most limit.
        public IReadOnlyList<string> Recent(IReadOnlyList<string> games, int limit)
        {
            if (_native is { } n)
            {
                var stream = new MemoryStream();
                using (var w = new Utf8JsonWriter(stream))
                {
                    w.WriteStartObject();
                    w.WriteString("what", "recent");
                    w.WriteNumber("limit", limit);
                    w.WriteStartArray("games");
                    foreach (string g in games) w.WriteStringValue(g);
                    w.WriteEndArray();
                    w.WriteEndObject();
                }
                using JsonDocument d = JsonDocument.Parse(n.Lib.Get(System.Text.Encoding.UTF8.GetString(stream.ToArray())));
                return d.RootElement.EnumerateArray().Select(e => e.GetString()!).ToList();
            }
            IReadOnlyDictionary<string, GameRecord> rows = All();
            return games.Where(g => rows.TryGetValue(g, out GameRecord? r) && r.LastPlayed is not null)
                .OrderByDescending(g => rows[g].LastPlayed!.Value.ToUniversalTime())
                .Take(Math.Max(0, limit)).ToList();
        }
    }
}
