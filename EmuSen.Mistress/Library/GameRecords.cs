using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
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
