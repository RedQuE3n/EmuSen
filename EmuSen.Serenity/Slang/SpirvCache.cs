using System;
using System.Collections.Generic;
using System.IO;
using System.Security.Cryptography;
using System.Text;
using EmuSen.Galaxia.Library;
using Microsoft.Data.Sqlite;

namespace EmuSen.Serenity.Slang
{
    // Compiled SPIR-V kept in SQLite by a hash of everything a compile reads; a miss, a damaged row or an unusable database compiles instead - see EmuSen_Serenity.md §9.4.
    public sealed class SpirvCache : IDisposable
    {
        public const string FileName = "spirv-cache.db";
        public const long DefaultLimit = 64L << 20;
        public const int SchemaVersion = 1;

        // Beside the packs it serves, in Galaxia's data home - see EmuSen_Serenity.md §9.4.
        public static string DefaultPath => System.IO.Path.Combine(DataStore.Shaders, FileName);

        private const int SqliteBusy = 5, SqliteLocked = 6, SqliteCorrupt = 11, SqliteNotADatabase = 26;
        private static readonly uint SpirvMagic = 0x07230203;

        private readonly object _lock = new();
        private readonly long _limit;
        private readonly string _identity;
        private SqliteConnection? _db;
        private long _total;

        public string Path { get; }

        // Why the cache is off, or null while it works; builds go on compiling either way.
        public string? Problem { get; private set; }

        public bool Working { get { lock (_lock) return _db is not null; } }

        internal int Hits, Misses, Stored, Damaged, Skipped, Evicted;

        // Unix seconds, and how old a row's use must be before a hit writes it again; a test sets both.
        internal Func<long> Clock = () => DateTimeOffset.UtcNow.ToUnixTimeSeconds();
        internal long TouchAfter = 3600;

        public SpirvCache(string path, long limit = DefaultLimit, string? compilerIdentity = null)
        {
            Path = path;
            _limit = limit;
            _identity = compilerIdentity ?? SlangCompiler.Identity;
            _db = Open(path, out string? problem);
            Problem = problem;
            if (_db is not null) _total = TotalOrZero();
        }

        // The stage's SPIR-V, from the cache or compiled and kept; a stage that does not compile throws as SlangCompiler does, and is not kept.
        public byte[] Compile(string text, SlangStage stage, string name)
        {
            byte[] key = Key(_identity, text, stage, name);
            if (Find(key) is { } found) return found;
            byte[] spirv = SlangCompiler.Compile(text, stage, name);
            Store(key, spirv);
            return spirv;
        }

        // SHA-256 over each part, length-prefixed, so no two different inputs run together into the same bytes.
        internal static byte[] Key(string identity, string text, SlangStage stage, string name)
        {
            using var hash = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
            void Part(string value)
            {
                byte[] bytes = Encoding.UTF8.GetBytes(value);
                hash.AppendData(BitConverter.GetBytes(bytes.Length));
                hash.AppendData(bytes);
            }
            Part("emusen-spirv/1");
            Part(identity);
            Part(stage.ToString());
            Part(name);
            Part(text);
            return hash.GetHashAndReset();
        }

        private byte[]? Find(byte[] key)
        {
            byte[]? spirv = null, digest = null;
            long lastUsed = 0;
            lock (_lock)
            {
                if (_db is null) return null;
                try
                {
                    using SqliteCommand command = _db.CreateCommand();
                    command.CommandText = "SELECT spirv, digest, last_used FROM spirv WHERE key = $key";
                    command.Parameters.AddWithValue("$key", key);
                    using SqliteDataReader row = command.ExecuteReader();
                    if (row.Read()) (spirv, digest, lastUsed) = ((byte[])row[0], (byte[])row[1], row.GetInt64(2));
                }
                catch (Exception e) when (e is SqliteException or InvalidOperationException or InvalidCastException)
                {
                    Failed(e);
                    return null;
                }
                if (spirv is null) { Misses++; return null; }
            }

            if (!Intact(spirv, digest!))
            {
                lock (_lock)
                {
                    Damaged++;
                    Misses++;
                    Run("DELETE FROM spirv WHERE key = $key", ("$key", key));
                }
                return null;
            }

            lock (_lock)
            {
                Hits++;
                long now = Clock();
                if (now - lastUsed >= TouchAfter) Run("UPDATE spirv SET last_used = $now WHERE key = $key", ("$now", now), ("$key", key));
            }
            return spirv;
        }

        private static bool Intact(byte[] spirv, byte[] digest) =>
            spirv.Length >= 20 && spirv.Length % 4 == 0 && BitConverter.ToUInt32(spirv, 0) == SpirvMagic && SHA256.HashData(spirv).AsSpan().SequenceEqual(digest);

        private void Store(byte[] key, byte[] spirv)
        {
            byte[] digest = SHA256.HashData(spirv);
            lock (_lock)
            {
                if (_db is null) return;
                int added = Run("INSERT OR IGNORE INTO spirv (key, bytes, last_used, digest, spirv) VALUES ($key, $bytes, $now, $digest, $spirv)",
                    ("$key", key), ("$bytes", (long)spirv.Length), ("$now", Clock()), ("$digest", digest), ("$spirv", spirv));
                if (added <= 0) return;
                Stored++;
                _total += spirv.Length;
                if (_total > _limit) Trim();
            }
        }

        // Under the lock: the least recently used rows go until the sum is three quarters of the bound, since other processes may have added to it too.
        private void Trim()
        {
            _total = TotalOrZero();
            if (_total <= _limit || _db is null) return;
            long target = _limit / 4 * 3;
            try
            {
                using SqliteTransaction transaction = _db.BeginTransaction();
                var doomed = new List<(byte[] Key, long Bytes)>();
                using (SqliteCommand select = _db.CreateCommand())
                {
                    select.Transaction = transaction;
                    select.CommandText = "SELECT key, bytes FROM spirv ORDER BY last_used, key";
                    using SqliteDataReader row = select.ExecuteReader();
                    long left = _total;
                    while (left > target && row.Read())
                    {
                        doomed.Add(((byte[])row[0], row.GetInt64(1)));
                        left -= row.GetInt64(1);
                    }
                }
                foreach (var (key, bytes) in doomed)
                {
                    using SqliteCommand delete = _db.CreateCommand();
                    delete.Transaction = transaction;
                    delete.CommandText = "DELETE FROM spirv WHERE key = $key";
                    delete.Parameters.AddWithValue("$key", key);
                    if (delete.ExecuteNonQuery() > 0) { _total -= bytes; Evicted++; }
                }
                transaction.Commit();
            }
            catch (Exception e) when (e is SqliteException or InvalidOperationException)
            {
                Failed(e);
            }
        }

        private long TotalOrZero()
        {
            try
            {
                using SqliteCommand command = _db!.CreateCommand();
                command.CommandText = "SELECT COALESCE(SUM(bytes), 0) FROM spirv";
                return Convert.ToInt64(command.ExecuteScalar());
            }
            catch (Exception e) when (e is SqliteException or InvalidOperationException)
            {
                Failed(e);
                return 0;
            }
        }

        // Under the lock: rows changed, or -1 when the statement failed and was let go.
        private int Run(string sql, params (string Name, object Value)[] parameters)
        {
            if (_db is null) return -1;
            try
            {
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = sql;
                foreach (var (name, value) in parameters) command.Parameters.AddWithValue(name, value);
                int changed = command.ExecuteNonQuery();
                return changed;
            }
            catch (Exception e) when (e is SqliteException or InvalidOperationException)
            {
                Failed(e);
                return -1;
            }
        }

        // Under the lock: busy or locked by another process is let go for this statement, and the third time turns the cache off, as anything else does at once; a damaged file is set aside for the next process.
        private void Failed(Exception e)
        {
            int code = (e as SqliteException)?.SqliteErrorCode ?? 0;
            if (code is SqliteBusy or SqliteLocked && ++_busy < BusyLimit) { Skipped++; return; }
            Problem = $"{FileName}: {e.Message.Split('\n')[0]}";
            _db?.Dispose();
            _db = null;
            if (code is SqliteCorrupt or SqliteNotADatabase) TrySetAside(Path);
        }

        private const int BusyLimit = 3;
        private int _busy;

        private static void TrySetAside(string path)
        {
            try { SetAside(path); }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }

        private static SqliteConnection? Open(string path, out string? problem)
        {
            problem = null;
            try
            {
                return OpenOnce(path);
            }
            catch (SqliteException e) when (e.SqliteErrorCode is SqliteCorrupt or SqliteNotADatabase)
            {
                // A cache that cannot be read is replaced, not repaired - see EmuSen_Serenity.md §9.4.
                try
                {
                    SetAside(path);
                    return OpenOnce(path);
                }
                catch (Exception again) when (again is SqliteException or IOException or UnauthorizedAccessException or InvalidOperationException)
                {
                    problem = $"{FileName}: {again.Message.Split('\n')[0]}";
                    return null;
                }
            }
            catch (Exception e) when (e is SqliteException or IOException or UnauthorizedAccessException or InvalidOperationException or ArgumentException)
            {
                problem = $"{FileName}: {e.Message.Split('\n')[0]}";
                return null;
            }
        }

        private static SqliteConnection OpenOnce(string path)
        {
            string? directory = System.IO.Path.GetDirectoryName(System.IO.Path.GetFullPath(path));
            if (!string.IsNullOrEmpty(directory)) Directory.CreateDirectory(directory);
            var db = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = path, Pooling = false, DefaultTimeout = 1 }.ToString());
            try
            {
                db.Open();
                Execute(db, "PRAGMA busy_timeout = 250");
                Execute(db, "PRAGMA journal_mode = WAL");
                Execute(db, "PRAGMA synchronous = NORMAL");
                using (SqliteCommand version = db.CreateCommand())
                {
                    version.CommandText = "PRAGMA user_version";
                    if (Convert.ToInt32(version.ExecuteScalar()) != SchemaVersion)
                    {
                        using SqliteTransaction transaction = db.BeginTransaction();
                        Execute(db, "DROP TABLE IF EXISTS spirv", transaction);
                        Execute(db, Schema, transaction);
                        Execute(db, $"PRAGMA user_version = {SchemaVersion}", transaction);
                        transaction.Commit();
                    }
                }
                Execute(db, Schema);
                return db;
            }
            catch
            {
                db.Dispose();
                throw;
            }
        }

        private static void Execute(SqliteConnection db, string sql, SqliteTransaction? transaction = null)
        {
            using SqliteCommand command = db.CreateCommand();
            command.Transaction = transaction;
            command.CommandText = sql;
            command.ExecuteNonQuery();
        }

        private static void SetAside(string path)
        {
            SqliteConnection.ClearAllPools();
            File.Move(path, path + ".damaged", overwrite: true);
            foreach (string side in new[] { path + "-wal", path + "-shm" })
                if (File.Exists(side)) File.Delete(side);
        }

        private static readonly Lazy<string> LazySchema = new(() =>
        {
            using Stream stream = typeof(SpirvCache).Assembly.GetManifestResourceStream("EmuSen.Serenity.Slang.spirv-cache-schema.sql")
                ?? throw new InvalidOperationException("the SPIR-V cache's schema is not embedded");
            using var reader = new StreamReader(stream);
            return reader.ReadToEnd();
        });

        private static string Schema => LazySchema.Value;

        public void Dispose()
        {
            lock (_lock)
            {
                _db?.Dispose();
                _db = null;
            }
        }
    }
}
