using System;
using System.Collections.Generic;
using System.Text.Json;

namespace EmuSen.Mistress.Library
{
    // The player's metadata edits, one row per field, beside the records they belong to - see EmuSen_Settings_Reference.md §4.59.
    public sealed partial class GameRecords
    {
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

        // Every game's edits, for a library that shows many games at once.
        public IReadOnlyDictionary<string, IReadOnlyDictionary<string, string>> AllEdits()
        {
            using JsonDocument d = Get("all_edits");
            var all = new Dictionary<string, IReadOnlyDictionary<string, string>>(StringComparer.Ordinal);
            foreach (JsonProperty p in d.RootElement.EnumerateObject()) all[p.Name] = Edits(p.Value);
            return all;
        }

        // One save of the editor: a value is kept as the player's edit, a null returns the field to its scraped or default value.
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

        // The editor's Clear; a cover borrowed from another game (§4.65) and a television standard chosen are the player's choices, not metadata, and stay.
        public void ClearEdits(string path) => Change("clear_edits", w => w.WriteString("path", path));

        // Q45: the game whose cover this one shows, or null to show its own again; nothing on disk but games.db changes.
        public void SetCoverChoice(string path, string? from, DateTime now) =>
            SaveEdits(path, new Dictionary<string, string?> { [GameMetadata.CoverFrom] = from == path ? null : from }, now);

        // The standard the player chose for this game's picture, or null where it follows the game - see EmuSen_CRT.md §26.
        public string? StandardChoice(string path) => Edits(path).TryGetValue(GameMetadata.TelevisionStandard, out string? standard) && standard.Length > 0 ? standard : null;

        public void SetStandardChoice(string path, string? standard, DateTime now) =>
            SaveEdits(path, new Dictionary<string, string?> { [GameMetadata.TelevisionStandard] = standard }, now);

        public string? CoverChoice(string path) => Edits(path).TryGetValue(GameMetadata.CoverFrom, out string? from) && from.Length > 0 ? from : null;

        public void SetFavourite(string path, bool favourite) => Change("set_favourite", w => { w.WriteString("path", path); w.WriteBoolean("favourite", favourite); });

        // ES-DE's editor lets the player correct the two counters; they are Mistress's records, not edits laid over them.
        public void SetPlayStats(string path, int count, double seconds) => Change("set_play_stats", w => { w.WriteString("path", path); w.WriteNumber("count", count); Real(w, "seconds", seconds); });
    }
}
