using System;
using System.Globalization;
using System.IO;
using EmuSen.Galaxia.Library;
using Microsoft.Data.Sqlite;

namespace EmuSen.Mistress.Library
{
    // records.db: what Mistress knows about files it fetched, each shader pack's build; its state_record rows are the runtime's to copy once (states.rs) - see EmuSen_Settings_Reference.md §4.64.
    public sealed class FileRecords : IDisposable
    {
        // Each entry takes the file from the version before it; append, never edit. Version 1's state_record is read only by the runtime's one copy into states.db.
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

        // Galaxia owns the name (Platform/galaxia/src/stores.rs); StoreNamesTests holds the two equal until the C# front end goes.
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

        // A file newer than this build is refused rather than written to, as games.db and media.db are - see §4.64.
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

        // --- downloaded shader packs ---

        // The build date recorded for the pack in this folder, or null when no pack is there; a pre-database stamp is imported and left - see §4.64.
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
