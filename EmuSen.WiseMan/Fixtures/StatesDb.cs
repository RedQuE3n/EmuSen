using System;
using System.Globalization;
using System.IO;
using System.Text.Json;
using EmuSen.Galaxia.Library;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Fixtures
{
    // A row of the runtime's states.db forged as the runtime writes one, and a record read back through the frontend API - see EmuSen_Backend.md §14.12 and §40.8.
    public static class StatesDb
    {
        private const string Schema = """
            CREATE TABLE state_record (
                path            TEXT    PRIMARY KEY,
                state_bytes     INTEGER NOT NULL,
                state_modified  INTEGER NOT NULL,
                console         TEXT    NOT NULL,
                core            TEXT    NOT NULL,
                state_version   INTEGER NOT NULL,
                build           TEXT    NOT NULL,
                saved_at        TEXT    NOT NULL,
                rom_file        TEXT    NOT NULL,
                rom_md5         TEXT,
                rom_bytes       INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE copied (source TEXT PRIMARY KEY, at TEXT NOT NULL);
            """;

        private static readonly JsonSerializerOptions RecordJson = new() { PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower };

        // The record of a state file that exists, keyed by its full path and stamped with its size and time, the database made at schema 1 when absent.
        public static void Write(string db, string statePath, StateRecord record)
        {
            var file = new FileInfo(Path.GetFullPath(statePath));
            Directory.CreateDirectory(Path.GetDirectoryName(db)!);
            using (var connection = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = db, Pooling = false }.ToString()))
            {
                connection.Open();
                using SqliteCommand version = connection.CreateCommand();
                version.CommandText = "PRAGMA user_version";
                if (Convert.ToInt32(version.ExecuteScalar(), CultureInfo.InvariantCulture) == 0)
                {
                    using SqliteCommand create = connection.CreateCommand();
                    create.CommandText = Schema + "PRAGMA user_version = 1;";
                    create.ExecuteNonQuery();
                }
                using SqliteCommand insert = connection.CreateCommand();
                insert.CommandText = """
                    INSERT OR REPLACE INTO state_record (path, state_bytes, state_modified, console, core, state_version, build, saved_at, rom_file, rom_md5, rom_bytes)
                    VALUES ($path, $bytes, $modified, $console, $core, $version, $build, $saved, $rom, $md5, $romBytes)
                    """;
                insert.Parameters.AddWithValue("$path", file.FullName);
                insert.Parameters.AddWithValue("$bytes", file.Length);
                insert.Parameters.AddWithValue("$modified", file.LastWriteTimeUtc.Ticks);
                insert.Parameters.AddWithValue("$console", record.Console);
                insert.Parameters.AddWithValue("$core", record.Core);
                insert.Parameters.AddWithValue("$version", record.StateVersion);
                insert.Parameters.AddWithValue("$build", record.Build);
                insert.Parameters.AddWithValue("$saved", record.SavedAt.ToString("o", CultureInfo.InvariantCulture));
                insert.Parameters.AddWithValue("$rom", record.RomFile);
                insert.Parameters.AddWithValue("$md5", (object?)record.RomMd5 ?? DBNull.Value);
                insert.Parameters.AddWithValue("$romBytes", record.RomBytes);
                insert.ExecuteNonQuery();
            }
            SqliteConnection.ClearAllPools();
        }

        // The runtime's state_record answer as the model, or null where it holds none.
        public static StateRecord? Read(EmuSen.Crystal.Client.CrystalRuntime runtime, string statePath) =>
            runtime.StateRecord(statePath) is string json ? JsonSerializer.Deserialize<StateRecord>(json, RecordJson) : null;
    }
}
