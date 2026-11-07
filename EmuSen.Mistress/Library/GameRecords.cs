using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;
using System.Text.Json;
using EmuSen.Crystal.Client.Library;

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

    // What an online cover lookup came to, as games.db keeps it - see EmuSen_Settings_Reference.md §4.39.
    public enum CoverOutcome { Found, Unknown, NoArt, Failed }

    // One of the player's own collections, with how many games it holds.
    public sealed record GameCollection(long Id, string Name, int Count);

    // The player's own library in games.db, through emusen_library.h's handle: kept, unlike the catalogue cache, so it migrates rather than rebuilds - see EmuSen_Settings_Reference.md §4.32, EmuSen_Backend.md §28.5 and §40.4.
    public sealed partial class GameRecords : IDisposable
    {
        private readonly GameLibrary _lib;

        private GameRecords(GameLibrary lib) => _lib = lib;

        // Galaxia owns the name (Platform/galaxia/src/stores.rs); StoreNamesTests holds the two equal until the C# front end goes.
        public const string FileName = "games.db";

        public static string DefaultPath => Path.Combine(DataStore.Library, FileName);

        public static GameRecords Load() => Open(DefaultPath);

        // A file newer than this build is refused rather than written to - see §4.32.
        public static GameRecords Open(string path)
        {
            try { return new GameRecords(GameLibrary.Open(path)); }
            catch (GameLibraryException e) when (e.Status == GameLibraryException.NewerFile) { throw new InvalidDataException(e.Message); }
        }

        // The library's handle, for the scan, identity and reclaim calls Mistress makes through it.
        internal GameLibrary Library => _lib;

        internal static string Time(DateTime when) => when.ToString("o", CultureInfo.InvariantCulture);

        public bool IsFavourite(string path) => Find(path)?.Favourite == true;

        public DateTime? LastPlayed(string path) => Find(path)?.LastPlayed;

        private static string Request(Action<Utf8JsonWriter> write)
        {
            var stream = new MemoryStream();
            using (var w = new Utf8JsonWriter(stream))
            {
                w.WriteStartObject();
                write(w);
                w.WriteEndObject();
            }
            return System.Text.Encoding.UTF8.GetString(stream.ToArray());
        }

        private static void Real(Utf8JsonWriter w, string name, double value)
        {
            if (double.IsFinite(value)) w.WriteNumber(name, value);
            else w.WriteString(name, double.IsNaN(value) ? "NaN" : value > 0 ? "Infinity" : "-Infinity");
        }

        private static double Real(JsonElement e) => e.ValueKind == JsonValueKind.String
            ? e.GetString() switch { "NaN" => double.NaN, "Infinity" => double.PositiveInfinity, _ => double.NegativeInfinity }
            : e.GetDouble();

        private JsonDocument Get(string what, Action<Utf8JsonWriter>? more = null) =>
            JsonDocument.Parse(Sqlite(() => _lib.Get(Request(w => { w.WriteString("what", what); more?.Invoke(w); }))));

        private string Change(string op, Action<Utf8JsonWriter>? more = null) =>
            Sqlite(() => _lib.Change(Request(w => { w.WriteString("op", op); more?.Invoke(w); })));

        // SQLite's refusal as the exception Microsoft.Data.Sqlite throws for it, its code read from its words.
        private static string Sqlite(Func<string> call)
        {
            try { return call(); }
            catch (GameLibraryException e) when (e.Status == GameLibraryException.BadRequest && e.Message == "Cannot store 'NaN' values.")
            {
                throw new InvalidOperationException(e.Message);
            }
            catch (GameLibraryException e) when (e.Status == GameLibraryException.Database)
            {
                string[] words = e.Message.Split(' ', 4);
                int code = words.Length > 2 && int.TryParse(words[2].TrimEnd(':'), out int c) ? c : 1;
                throw new Microsoft.Data.Sqlite.SqliteException(e.Message, code);
            }
        }

        private static GameRecord Record(JsonElement r) => new()
        {
            Favourite = r.GetProperty("favourite").GetBoolean(),
            LastPlayed = r.GetProperty("last_played").ValueKind == JsonValueKind.Null ? null : DateTime.Parse(r.GetProperty("last_played").GetString()!, CultureInfo.InvariantCulture, DateTimeStyles.RoundtripKind),
            PlayCount = r.GetProperty("play_count").GetInt32(),
            PlaySeconds = Real(r.GetProperty("play_seconds")),
        };

        private static string? Text(JsonDocument d) => d.RootElement.ValueKind == JsonValueKind.Null ? null : d.RootElement.GetString();

        public GameRecord? Find(string path)
        {
            using JsonDocument d = Get("record", w => w.WriteString("path", path));
            return d.RootElement.ValueKind == JsonValueKind.Null ? null : Record(d.RootElement);
        }

        public IReadOnlyDictionary<string, GameRecord> All()
        {
            using JsonDocument d = Get("records");
            var all = new Dictionary<string, GameRecord>(StringComparer.Ordinal);
            foreach (JsonProperty p in d.RootElement.EnumerateObject()) all[p.Name] = Record(p.Value);
            return all;
        }

        public void ToggleFavourite(string path) => Change("toggle_favourite", w => w.WriteString("path", path));

        public void Started(string path, DateTime now) => Change("started", w => { w.WriteString("path", path); w.WriteString("at", Time(now)); });

        public void Played(string path, TimeSpan time) => Change("played", w => { w.WriteString("path", path); Real(w, "seconds", time.TotalSeconds); });

        public string? KnownHash(string path, long bytes, long modified)
        {
            using JsonDocument d = Get("known_hash", w => { w.WriteString("path", path); w.WriteNumber("bytes", bytes); w.WriteNumber("modified", modified); });
            return Text(d);
        }

        public void StoreHash(string path, long bytes, long modified, string md5) =>
            Change("store_hash", w => { w.WriteString("path", path); w.WriteNumber("bytes", bytes); w.WriteNumber("modified", modified); w.WriteString("md5", md5); });

        public string? PathByHash(string md5)
        {
            using JsonDocument d = Get("path_by_hash", w => w.WriteString("md5", md5));
            return Text(d);
        }

        public void Identify(string path, string md5, long bytes) => Change("identify", w => { w.WriteString("path", path); w.WriteString("md5", md5); w.WriteNumber("bytes", bytes); });

        public IReadOnlyList<(string Path, string Md5, long Bytes)> Orphans(IReadOnlySet<string> present)
        {
            using JsonDocument d = Get("orphans", w => { w.WriteStartArray("present"); foreach (string p in present) w.WriteStringValue(p); w.WriteEndArray(); });
            return d.RootElement.EnumerateArray().Select(o => (o.GetProperty("path").GetString()!, o.GetProperty("md5").GetString()!, o.GetProperty("bytes").GetInt64())).ToList();
        }

        public void Move(string from, string to) => Change("move", w => { w.WriteString("from", from); w.WriteString("to", to); });

        public string? CoverLookup(string path)
        {
            using JsonDocument d = Get("cover_lookup", w => w.WriteString("path", path));
            return Text(d);
        }

        public void RecordCoverLookup(string path, CoverOutcome outcome, string? romName, DateTime now) =>
            Change("record_cover_lookup", w =>
            {
                w.WriteString("path", path);
                w.WriteString("outcome", outcome.ToString());
                if (romName is null) w.WriteNull("rom_name"); else w.WriteString("rom_name", romName);
                w.WriteString("at", Time(now));
            });

        public void ForgetCoverLookup(string path) => Change("forget_cover_lookup", w => w.WriteString("path", path));

        public IReadOnlyList<GameCollection> Collections()
        {
            using JsonDocument d = Get("collections");
            return d.RootElement.EnumerateArray().Select(c => new GameCollection(c.GetProperty("id").GetInt64(), c.GetProperty("name").GetString()!, c.GetProperty("count").GetInt32())).ToList();
        }

        public long? CreateCollection(string name, DateTime now)
        {
            using JsonDocument d = JsonDocument.Parse(Change("create_collection", w => { w.WriteString("name", name); w.WriteString("at", Time(now)); }));
            return d.RootElement.ValueKind == JsonValueKind.Null ? null : d.RootElement.GetInt64();
        }

        public bool RenameCollection(long id, string name) => Change("rename_collection", w => { w.WriteNumber("id", id); w.WriteString("name", name); }) == "true";

        public void DeleteCollection(long id) => Change("delete_collection", w => w.WriteNumber("id", id));

        public void AddToCollection(long id, string path) => Change("add_to_collection", w => { w.WriteNumber("id", id); w.WriteString("path", path); });

        public void RemoveFromCollection(long id, string path) => Change("remove_from_collection", w => { w.WriteNumber("id", id); w.WriteString("path", path); });

        public IReadOnlySet<string> Members(long id)
        {
            using JsonDocument d = Get("members", w => w.WriteNumber("id", id));
            return d.RootElement.EnumerateArray().Select(m => m.GetString()!).ToHashSet(StringComparer.Ordinal);
        }

        public IReadOnlySet<long> CollectionsOf(string path)
        {
            using JsonDocument d = Get("collections_of", w => w.WriteString("path", path));
            return d.RootElement.EnumerateArray().Select(m => m.GetInt64()).ToHashSet();
        }

        public void Dispose() => _lib.Dispose();
    }
}
