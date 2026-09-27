using System;
using System.Globalization;
using System.IO;
using EmuSen.Galaxia.Library;
using Microsoft.Data.Sqlite;

namespace EmuSen.Mistress.Library
{
    // records.db: what Mistress knows about files it wrote or fetched, each save state's record and each shader pack's build - see EmuSen_Settings_Reference.md §4.61.
    public sealed class FileRecords : IDisposable
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
                rom_bytes       INTEGER NOT NULL DEFAULT 0,
                source          TEXT    NOT NULL CHECK (source IN ('written', 'sidecar'))
            );
            CREATE TABLE shader_pack (
                directory  TEXT PRIMARY KEY,
                built      TEXT NOT NULL,
                source     TEXT NOT NULL CHECK (source IN ('written', 'stamp')),
                recorded   TEXT NOT NULL
            );
            """,
        };

        public static int SchemaVersion => Migrations.Length;

        public const string FileName = "records.db";

        private readonly SqliteConnection _db;
        private readonly object _gate = new();
        private bool _closed;

        private FileRecords(SqliteConnection db) => _db = db;

        public static string DefaultPath => Path.Combine(DataStore.Library, FileName);

        public static FileRecords Load() => Open(DefaultPath);

        public static FileRecords Open(string path)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(Path.GetFullPath(path))!);
            var db = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = path, Pooling = false }.ToString());
            db.Open();
            try
            {
                Migrate(db);
                Execute(db, null, "PRAGMA journal_mode = WAL");
                Execute(db, null, "PRAGMA synchronous = FULL");
            }
            catch
            {
                db.Dispose();
                throw;
            }
            return new FileRecords(db);
        }

        // A file newer than this build is refused rather than written to, as games.db and media.db are - see §4.61.
        private static void Migrate(SqliteConnection db)
        {
            int version = Convert.ToInt32(Scalar(db, "PRAGMA user_version", null), CultureInfo.InvariantCulture);
            if (version > Migrations.Length) throw new InvalidDataException($"{FileName} is schema {version}; this build knows {Migrations.Length}.");
            for (; version < Migrations.Length; version++)
            {
                using SqliteTransaction step = db.BeginTransaction();
                Execute(db, step, Migrations[version]);
                Execute(db, step, $"PRAGMA user_version = {version + 1}");
                step.Commit();
            }
        }

        // --- save states ---

        // The record while the state is the file it was written for; a sidecar is imported on first sight and left where it is - see §4.61.
        public StateRecord? ReadState(string statePath)
        {
            string key = Path.GetFullPath(statePath);
            if (Stat(key) is not { } stat) return null;
            lock (_gate)
            {
                if (_closed) return StateRecord.ReadSidecar(key);
                using (SqliteCommand command = _db.CreateCommand())
                {
                    command.CommandText = "SELECT state_bytes, state_modified, console, core, state_version, build, saved_at, rom_file, rom_md5, rom_bytes FROM state_record WHERE path = $path";
                    command.Parameters.AddWithValue("$path", key);
                    using SqliteDataReader row = command.ExecuteReader();
                    if (row.Read())
                        return row.GetInt64(0) == stat.Bytes && row.GetInt64(1) == stat.Modified ? StateFrom(row) : null;
                }

                if (StateRecord.ReadSidecar(key) is not StateRecord sidecar) return null;
                try
                {
                    Upsert(key, stat, sidecar, "sidecar", replace: false);
                }
                catch (SqliteException)
                {
                }
                return sidecar;
            }
        }

        // Stamped with the state file's size and time, so a state rewritten without a record is not described by this one.
        public bool WriteState(string statePath, StateRecord record)
        {
            string key = Path.GetFullPath(statePath);
            if (Stat(key) is not { } stat) return false;
            lock (_gate)
            {
                if (_closed) return false;
                try
                {
                    Upsert(key, stat, record, "written", replace: true);
                    return true;
                }
                catch (SqliteException)
                {
                    return false;
                }
            }
        }

        public void ForgetState(string statePath) => Write("DELETE FROM state_record WHERE path = $path", ("$path", Path.GetFullPath(statePath)));

        // Where the record came from: "written" by this store, "sidecar" when imported from a pre-database file, or null.
        public string? StateSource(string statePath) => ReadScalar("SELECT source FROM state_record WHERE path = $path", ("$path", Path.GetFullPath(statePath))) as string;

        private void Upsert(string key, (long Bytes, long Modified) stat, StateRecord record, string source, bool replace)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = """
                INSERT INTO state_record (path, state_bytes, state_modified, console, core, state_version, build, saved_at, rom_file, rom_md5, rom_bytes, source)
                VALUES ($path, $bytes, $modified, $console, $core, $version, $build, $saved, $rom, $md5, $romBytes, $source)
                """ + (replace
                    ? """
                       ON CONFLICT(path) DO UPDATE SET state_bytes = excluded.state_bytes, state_modified = excluded.state_modified, console = excluded.console,
                       core = excluded.core, state_version = excluded.state_version, build = excluded.build, saved_at = excluded.saved_at, rom_file = excluded.rom_file,
                       rom_md5 = excluded.rom_md5, rom_bytes = excluded.rom_bytes, source = excluded.source
                      """
                    : " ON CONFLICT(path) DO NOTHING");
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
            command.Parameters.AddWithValue("$source", source);
            command.ExecuteNonQuery();
        }

        private static StateRecord StateFrom(SqliteDataReader row) => new()
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

        // --- downloaded shader packs ---

        // The build date recorded for the pack in this folder, or null when no pack is there; a pre-database stamp is imported and left - see §4.61.
        public string? PackBuilt(string directory, string legacyStampFile)
        {
            string key = Path.GetFullPath(directory);
            if (!Directory.Exists(key)) return null;
            if (ReadScalar("SELECT built FROM shader_pack WHERE directory = $directory", ("$directory", key)) is string built) return built;

            string stamp = Path.Combine(key, legacyStampFile);
            string? stamped;
            try
            {
                stamped = File.Exists(stamp) ? File.ReadAllText(stamp).Trim() : null;
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                stamped = null;
            }
            if (string.IsNullOrEmpty(stamped)) return null;
            TryWrite("INSERT INTO shader_pack (directory, built, source, recorded) VALUES ($directory, $built, 'stamp', $now) ON CONFLICT(directory) DO NOTHING",
                ("$directory", key), ("$built", stamped), ("$now", DateTime.UtcNow.ToString("o", CultureInfo.InvariantCulture)));
            return stamped;
        }

        public bool RecordPack(string directory, string built) => TryWrite(
            "INSERT INTO shader_pack (directory, built, source, recorded) VALUES ($directory, $built, 'written', $now) "
            + "ON CONFLICT(directory) DO UPDATE SET built = excluded.built, source = excluded.source, recorded = excluded.recorded",
            ("$directory", Path.GetFullPath(directory)), ("$built", built), ("$now", DateTime.UtcNow.ToString("o", CultureInfo.InvariantCulture)));

        // Where the pack's date came from: "written" by a download, "stamp" when imported from a pre-database file, or null.
        public string? PackSource(string directory) => ReadScalar("SELECT source FROM shader_pack WHERE directory = $directory", ("$directory", Path.GetFullPath(directory))) as string;

        // --- plumbing ---

        private bool TryWrite(string sql, params (string Name, object Value)[] parameters)
        {
            try
            {
                lock (_gate)
                {
                    if (_closed) return false;
                    Write(sql, parameters);
                    return true;
                }
            }
            catch (SqliteException)
            {
                return false;
            }
        }

        private void Write(string sql, params (string Name, object Value)[] parameters)
        {
            lock (_gate)
            {
                if (_closed) return;
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = sql;
                foreach ((string name, object value) in parameters) command.Parameters.AddWithValue(name, value);
                command.ExecuteNonQuery();
            }
        }

        private object? ReadScalar(string sql, params (string Name, object Value)[] parameters)
        {
            lock (_gate)
            {
                if (_closed) return null;
                return Scalar(_db, sql, null, parameters);
            }
        }

        private static object? Scalar(SqliteConnection db, string sql, SqliteTransaction? transaction, params (string Name, object Value)[] parameters)
        {
            using SqliteCommand command = db.CreateCommand();
            command.Transaction = transaction;
            command.CommandText = sql;
            foreach ((string name, object value) in parameters) command.Parameters.AddWithValue(name, value);
            return command.ExecuteScalar();
        }

        private static void Execute(SqliteConnection db, SqliteTransaction? transaction, string sql)
        {
            using SqliteCommand command = db.CreateCommand();
            command.Transaction = transaction;
            command.CommandText = sql;
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
