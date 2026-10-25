using System;
using System.Globalization;
using System.IO;
using EmuSen.Galaxia.Library;
using Microsoft.Data.Sqlite;

namespace EmuSen.Crystal.Managed
{
    // states.db: the runtime's record of each save state it wrote, keyed by the state's path and stamped with its size and time, the rows of Mistress's records.db copied in once - see EmuSen_Backend.md §14.12 and D8.
    internal sealed class StateRecords : IDisposable
    {
        // Each entry takes the file from the version before it; append, never edit.
        private static readonly string[] Migrations =
        {
            """
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
            """,
        };

        public const string FileName = "states.db";

        // Mistress's file whose state rows are copied in on the first open.
        public const string RecordsFileName = "records.db";

        private readonly SqliteConnection _db;
        private readonly object _gate = new();
        private bool _closed;

        private StateRecords(SqliteConnection db) => _db = db;

        public static string DefaultPath => Path.Combine(DataStore.Library, FileName);

        public static StateRecords Open(string? path = null)
        {
            path ??= DefaultPath;
            Directory.CreateDirectory(Path.GetDirectoryName(Path.GetFullPath(path))!);
            var db = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = path, Pooling = false }.ToString());
            db.Open();
            try
            {
                int version = Convert.ToInt32(Scalar(db, null, "PRAGMA user_version"), CultureInfo.InvariantCulture);
                if (version > Migrations.Length) throw new InvalidDataException($"{FileName} is schema {version}; this build knows {Migrations.Length}.");
                for (; version < Migrations.Length; version++)
                {
                    using SqliteTransaction step = db.BeginTransaction();
                    Execute(db, step, Migrations[version]);
                    Execute(db, step, $"PRAGMA user_version = {version + 1}");
                    step.Commit();
                }
                Execute(db, null, "PRAGMA journal_mode = WAL");
                CopyFromRecords(db, Path.Combine(Path.GetDirectoryName(Path.GetFullPath(path))!, RecordsFileName));
            }
            catch
            {
                db.Dispose();
                throw;
            }
            return new StateRecords(db);
        }

        // D8's one migration: every row of records.db's state_record copied, none over a row already here, once.
        private static void CopyFromRecords(SqliteConnection db, string records)
        {
            if (Scalar(db, null, "SELECT COUNT(*) FROM copied WHERE source = $source", ("$source", RecordsFileName)) is long n && n > 0) return;
            if (File.Exists(records))
            {
                Execute(db, null, "ATTACH DATABASE $file AS records", ("$file", records));
                try
                {
                    using SqliteTransaction copy = db.BeginTransaction();
                    if (Scalar(db, copy, "SELECT COUNT(*) FROM records.sqlite_master WHERE type = 'table' AND name = 'state_record'") is long tables && tables > 0)
                        Execute(db, copy, """
                            INSERT OR IGNORE INTO state_record (path, state_bytes, state_modified, console, core, state_version, build, saved_at, rom_file, rom_md5, rom_bytes)
                            SELECT path, state_bytes, state_modified, console, core, state_version, build, saved_at, rom_file, rom_md5, rom_bytes FROM records.state_record
                            """);
                    Execute(db, copy, "INSERT INTO copied (source, at) VALUES ($source, $at)", ("$source", RecordsFileName), ("$at", DateTime.UtcNow.ToString("o", CultureInfo.InvariantCulture)));
                    copy.Commit();
                }
                finally
                {
                    Execute(db, null, "DETACH DATABASE records");
                }
                return;
            }
            Execute(db, null, "INSERT INTO copied (source, at) VALUES ($source, $at)", ("$source", RecordsFileName), ("$at", DateTime.UtcNow.ToString("o", CultureInfo.InvariantCulture)));
        }

        // The record while the state is the file it was written for; a pre-database sidecar is read in its place, as records.db read it.
        public StateRecord? Read(string statePath)
        {
            if (SaveLibrary.PackageRecordPathFor(statePath) is { } packaged && File.Exists(packaged)) return ReadPackaged(packaged, statePath);
            string key = Path.GetFullPath(statePath);
            if (Stat(key) is not { } stat) return null;
            lock (_gate)
            {
                if (_closed) return null;
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = "SELECT state_bytes, state_modified, console, core, state_version, build, saved_at, rom_file, rom_md5, rom_bytes FROM state_record WHERE path = $path";
                command.Parameters.AddWithValue("$path", key);
                using SqliteDataReader row = command.ExecuteReader();
                if (row.Read())
                    return row.GetInt64(0) == stat.Bytes && row.GetInt64(1) == stat.Modified ? From(row) : null;
            }
            return StateRecord.ReadSidecar(key);
        }

        public void Write(string statePath, StateRecord record)
        {
            if (SaveLibrary.PackageRecordPathFor(statePath) is { } packaged)
            {
                WritePackaged(packaged, statePath, record);
                return;
            }
            string key = Path.GetFullPath(statePath);
            if (Stat(key) is not { } stat) return;
            lock (_gate)
            {
                if (_closed) return;
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = """
                    INSERT INTO state_record (path, state_bytes, state_modified, console, core, state_version, build, saved_at, rom_file, rom_md5, rom_bytes)
                    VALUES ($path, $bytes, $modified, $console, $core, $version, $build, $saved, $rom, $md5, $romBytes)
                    ON CONFLICT(path) DO UPDATE SET state_bytes = excluded.state_bytes, state_modified = excluded.state_modified, console = excluded.console,
                    core = excluded.core, state_version = excluded.state_version, build = excluded.build, saved_at = excluded.saved_at, rom_file = excluded.rom_file,
                    rom_md5 = excluded.rom_md5, rom_bytes = excluded.rom_bytes
                    """;
                command.Parameters.AddWithValue("$path", key);
                command.Parameters.AddWithValue("$bytes", stat.Bytes);
                command.Parameters.AddWithValue("$modified", stat.Modified);
                command.Parameters.AddWithValue("$console", record.Console);
                command.Parameters.AddWithValue("$core", record.Core);
                command.Parameters.AddWithValue("$version", record.StateVersion);
                command.Parameters.AddWithValue("$build", record.Build);
                command.Parameters.AddWithValue("$saved", record.SavedAt.ToString("o", CultureInfo.InvariantCulture));
                command.Parameters.AddWithValue("$rom", record.RomFile);
                command.Parameters.AddWithValue("$md5", (object?)record.RomMd5 ?? DBNull.Value);
                command.Parameters.AddWithValue("$romBytes", record.RomBytes);
                command.ExecuteNonQuery();
            }
        }

        public void Forget(string statePath)
        {
            if (SaveLibrary.PackageRecordPathFor(statePath) is { } packaged && File.Exists(packaged)) File.Delete(packaged);
            lock (_gate)
            {
                if (_closed) return;
                Execute(_db, null, "DELETE FROM state_record WHERE path = $path", ("$path", Path.GetFullPath(statePath)));
            }
        }

        // A package's record of one of its states: one line of JSON beside it with the state's size and time, so that a package moved or renamed keeps it - see EmuSen_GamePackages.md §18.1.
        public static byte[] PackagedText(StateRecord record, long stateBytes, long stateModified)
        {
            var buffer = new System.Buffers.ArrayBufferWriter<byte>();
            using (var w = new System.Text.Json.Utf8JsonWriter(buffer))
            {
                w.WriteStartObject();
                w.WriteString("Console", record.Console);
                w.WriteString("Core", record.Core);
                w.WriteNumber("StateVersion", record.StateVersion);
                w.WriteString("Build", record.Build);
                w.WriteString("SavedAt", record.SavedAt.ToString("o", CultureInfo.InvariantCulture));
                w.WriteString("RomFile", record.RomFile);
                if (record.RomMd5 is null) w.WriteNull("RomMd5");
                else w.WriteString("RomMd5", record.RomMd5);
                w.WriteNumber("RomBytes", record.RomBytes);
                w.WriteNumber("StateBytes", stateBytes);
                w.WriteNumber("StateModified", stateModified);
                w.WriteEndObject();
            }
            return [.. buffer.WrittenSpan, (byte)'\n'];
        }

        private static void WritePackaged(string recordPath, string statePath, StateRecord record)
        {
            if (Stat(Path.GetFullPath(statePath)) is not { } stat) return;
            if (!AtomicFile.Write(recordPath, PackagedText(record, stat.Bytes, stat.Modified))) throw new IOException($"{recordPath} could not be written");
        }

        // A package's record while its state is the file it was written for, else null.
        public static StateRecord? ReadPackaged(string recordPath, string statePath)
        {
            if (Stat(Path.GetFullPath(statePath)) is not { } stat) return null;
            try
            {
                byte[] bytes = File.ReadAllBytes(recordPath);
                using (var doc = System.Text.Json.JsonDocument.Parse(bytes, new System.Text.Json.JsonDocumentOptions { CommentHandling = System.Text.Json.JsonCommentHandling.Skip, AllowTrailingCommas = true }))
                {
                    long? Number(string name)
                    {
                        if (doc.RootElement.ValueKind != System.Text.Json.JsonValueKind.Object) return null;
                        System.Text.Json.JsonElement? found = null;
                        foreach (var p in doc.RootElement.EnumerateObject())
                            if (string.Equals(p.Name, name, StringComparison.OrdinalIgnoreCase)) found = p.Value;
                        return found is { ValueKind: System.Text.Json.JsonValueKind.Number } v && v.TryGetInt64(out long n) ? n : null;
                    }
                    if (Number("StateBytes") != stat.Bytes || Number("StateModified") != stat.Modified) return null;
                }
                return System.Text.Json.JsonSerializer.Deserialize<StateRecord>(bytes, EmuSen.Galaxia.ConfigJson.Options);
            }
            catch (Exception ex) when (ex is IOException or System.Text.Json.JsonException or UnauthorizedAccessException)
            {
                return null;
            }
        }

        private static StateRecord From(SqliteDataReader row) => new()
        {
            Console = row.GetString(2),
            Core = row.GetString(3),
            StateVersion = row.GetInt32(4),
            Build = row.GetString(5),
            SavedAt = DateTime.Parse(row.GetString(6), CultureInfo.InvariantCulture, DateTimeStyles.RoundtripKind),
            RomFile = row.GetString(7),
            RomMd5 = row.IsDBNull(8) ? null : row.GetString(8),
            RomBytes = row.GetInt64(9),
        };

        private static (long Bytes, long Modified)? Stat(string path)
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

        private static object? Scalar(SqliteConnection db, SqliteTransaction? transaction, string sql, params (string Name, object Value)[] parameters)
        {
            using SqliteCommand command = db.CreateCommand();
            command.Transaction = transaction;
            command.CommandText = sql;
            foreach ((string name, object value) in parameters) command.Parameters.AddWithValue(name, value);
            return command.ExecuteScalar();
        }

        private static void Execute(SqliteConnection db, SqliteTransaction? transaction, string sql, params (string Name, object Value)[] parameters)
        {
            using SqliteCommand command = db.CreateCommand();
            command.Transaction = transaction;
            command.CommandText = sql;
            foreach ((string name, object value) in parameters) command.Parameters.AddWithValue(name, value);
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
