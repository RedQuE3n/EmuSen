using System;
using System.Collections.Generic;
using System.Globalization;
using Microsoft.Data.Sqlite;

namespace EmuSen.Mistress.Library
{
    // The player's metadata edits, one row per field, beside the records they belong to - see EmuSen_Settings_Reference.md §4.59.
    public sealed partial class GameRecords
    {
        public IReadOnlyDictionary<string, string> Edits(string path)
        {
            var edits = new Dictionary<string, string>(StringComparer.Ordinal);
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "SELECT field, value FROM game_edit WHERE path = $path";
            command.Parameters.AddWithValue("$path", path);
            using SqliteDataReader row = command.ExecuteReader();
            while (row.Read()) edits[row.GetString(0)] = row.GetString(1);
            return edits;
        }

        // Every game's edits, for a library that shows many games at once.
        public IReadOnlyDictionary<string, IReadOnlyDictionary<string, string>> AllEdits()
        {
            var all = new Dictionary<string, Dictionary<string, string>>(StringComparer.Ordinal);
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "SELECT path, field, value FROM game_edit";
            using SqliteDataReader row = command.ExecuteReader();
            while (row.Read())
            {
                if (!all.TryGetValue(row.GetString(0), out Dictionary<string, string>? edits)) all[row.GetString(0)] = edits = new Dictionary<string, string>(StringComparer.Ordinal);
                edits[row.GetString(1)] = row.GetString(2);
            }
            var shown = new Dictionary<string, IReadOnlyDictionary<string, string>>(StringComparer.Ordinal);
            foreach ((string path, Dictionary<string, string> edits) in all) shown[path] = edits;
            return shown;
        }

        // One save of the editor: a value is kept as the player's edit, a null returns the field to its scraped or default value.
        public void SaveEdits(string path, IReadOnlyDictionary<string, string?> changes, DateTime now)
        {
            using SqliteTransaction step = _db.BeginTransaction();
            foreach ((string field, string? value) in changes)
            {
                using SqliteCommand command = _db.CreateCommand();
                command.Transaction = step;
                command.CommandText = value is null
                    ? "DELETE FROM game_edit WHERE path = $path AND field = $field"
                    : "INSERT INTO game_edit (path, field, value, edited) VALUES ($path, $field, $value, $now) ON CONFLICT(path, field) DO UPDATE SET value = excluded.value, edited = excluded.edited";
                command.Parameters.AddWithValue("$path", path);
                command.Parameters.AddWithValue("$field", field);
                if (value is not null)
                {
                    command.Parameters.AddWithValue("$value", value);
                    command.Parameters.AddWithValue("$now", now.ToString("o", CultureInfo.InvariantCulture));
                }
                command.ExecuteNonQuery();
            }
            step.Commit();
        }

        public void ClearEdits(string path) => Write(path, "DELETE FROM game_edit WHERE path = $path");

        public void SetFavourite(string path, bool favourite) => Write(path,
            "INSERT INTO game (path, favourite) VALUES ($path, $value) ON CONFLICT(path) DO UPDATE SET favourite = excluded.favourite", favourite ? 1 : 0);

        // ES-DE's editor lets the player correct the two counters; they are Mistress's records, not edits laid over them.
        public void SetPlayStats(string path, int count, double seconds)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "INSERT INTO game (path, play_count, play_seconds) VALUES ($path, $count, $seconds) "
                + "ON CONFLICT(path) DO UPDATE SET play_count = excluded.play_count, play_seconds = excluded.play_seconds";
            command.Parameters.AddWithValue("$path", path);
            command.Parameters.AddWithValue("$count", Math.Max(0, count));
            command.Parameters.AddWithValue("$seconds", Math.Max(0, seconds));
            command.ExecuteNonQuery();
        }
    }
}
