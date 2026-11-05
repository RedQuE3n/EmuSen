using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Galaxia.Library;

namespace EmuSen.Mistress.Library
{
    // A game known by its contents, and rows that follow a renamed game, out of the window: the library's identity and reclaim - see EmuSen_Settings_Reference.md §4.37, EmuSen_Backend.md §28.5.
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
                using JsonDocument d = JsonDocument.Parse(_lib.Identity(System.Text.Encoding.UTF8.GetString(stream.ToArray())));
                return (d.RootElement.GetProperty("md5").GetString()!, d.RootElement.GetProperty("bytes").GetInt64());
            }
            catch (EmuSen.Crystal.Client.Library.GameLibraryException e) when (e.Status == EmuSen.Crystal.Client.Library.GameLibraryException.Io) { return null; }
        }

        // Rows whose file is not among games, each moved to the first game of its size and contents that has no row; only files of a missing one's size are hashed. How many moved.
        public int Reclaim(IReadOnlyList<string> games)
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
            using JsonDocument d = JsonDocument.Parse(_lib.Change(System.Text.Encoding.UTF8.GetString(stream.ToArray())));
            return d.RootElement.GetProperty("moved").GetInt32();
        }

        // Of games, those with a last-played time, newest first by the instant each names, at most limit.
        public IReadOnlyList<string> Recent(IReadOnlyList<string> games, int limit)
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
            using JsonDocument d = JsonDocument.Parse(_lib.Get(System.Text.Encoding.UTF8.GetString(stream.ToArray())));
            return d.RootElement.EnumerateArray().Select(e => e.GetString()!).ToList();
        }
    }
}
