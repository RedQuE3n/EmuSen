using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Client.Library;

namespace EmuSen.Mistress.Library
{
    // GameRecords' every member over emusen_library.h, the library's handle on games.db - see EmuSen_Backend.md §28.5.
    public sealed partial class GameRecords
    {
        private readonly NativeRecords? _native;

        private GameRecords(NativeRecords native)
        {
            _native = native;
            _db = null!;
        }

        // games.db through the library, whichever the switch says; what the parity class opens.
        public static GameRecords OpenNative(string path)
        {
            try { return new GameRecords(new NativeRecords(GameLibrary.Open(path))); }
            catch (GameLibraryException e) when (e.Status == GameLibraryException.NewerFile) { throw new InvalidDataException(e.Message); }
        }

        // Whether this file is reached through the library.
        public bool ThroughLibrary => _native is not null;

        // The library's handle, for the scan, identity and reclaim calls Mistress makes through it.
        internal GameLibrary? Library => _native?.Lib;

        internal static string Time(DateTime when) => when.ToString("o", CultureInfo.InvariantCulture);

        private sealed class NativeRecords(GameLibrary lib)
        {
            public GameLibrary Lib { get; } = lib;

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
                JsonDocument.Parse(Sqlite(() => Lib.Get(Request(w => { w.WriteString("what", what); more?.Invoke(w); }))));

            private string Change(string op, Action<Utf8JsonWriter>? more = null) =>
                Sqlite(() => Lib.Change(Request(w => { w.WriteString("op", op); more?.Invoke(w); })));

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

            private static Dictionary<string, string> Edits(JsonElement e)
            {
                var edits = new Dictionary<string, string>(StringComparer.Ordinal);
                foreach (JsonProperty p in e.EnumerateObject()) edits[p.Name] = p.Value.GetString()!;
                return edits;
            }

            public IReadOnlyDictionary<string, string> Edits(string path)
            {
                using JsonDocument d = Get("edits", w => w.WriteString("path", path));
                return Edits(d.RootElement);
            }

            public IReadOnlyDictionary<string, IReadOnlyDictionary<string, string>> AllEdits()
            {
                using JsonDocument d = Get("all_edits");
                var all = new Dictionary<string, IReadOnlyDictionary<string, string>>(StringComparer.Ordinal);
                foreach (JsonProperty p in d.RootElement.EnumerateObject()) all[p.Name] = Edits(p.Value);
                return all;
            }

            public void SaveEdits(string path, IReadOnlyDictionary<string, string?> changes, DateTime now) =>
                Change("save_edits", w =>
                {
                    w.WriteString("path", path);
                    w.WriteStartObject("changes");
                    foreach ((string field, string? value) in changes)
                        if (value is null) w.WriteNull(field); else w.WriteString(field, value);
                    w.WriteEndObject();
                    w.WriteString("at", Time(now));
                });

            public void ClearEdits(string path) => Change("clear_edits", w => w.WriteString("path", path));

            public void SetFavourite(string path, bool favourite) => Change("set_favourite", w => { w.WriteString("path", path); w.WriteBoolean("favourite", favourite); });

            public void SetPlayStats(string path, int count, double seconds) => Change("set_play_stats", w => { w.WriteString("path", path); w.WriteNumber("count", count); Real(w, "seconds", seconds); });

            public GameCrop? Crop(string path)
            {
                using JsonDocument d = Get("crop", w => w.WriteString("path", path));
                JsonElement c = d.RootElement;
                return c.ValueKind == JsonValueKind.Null ? null : new GameCrop(Real(c.GetProperty("left")), Real(c.GetProperty("right")), Real(c.GetProperty("top")), Real(c.GetProperty("bottom")));
            }

            public void SetCrop(string path, GameCrop crop) =>
                Change("set_crop", w => { w.WriteString("path", path); Real(w, "left", crop.Left); Real(w, "right", crop.Right); Real(w, "top", crop.Top); Real(w, "bottom", crop.Bottom); });

            public void ClearCrop(string path) => Change("clear_crop", w => w.WriteString("path", path));
        }
    }
}
