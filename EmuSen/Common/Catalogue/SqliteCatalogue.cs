using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Galaxia.Library.Catalogue;
using Microsoft.Data.Sqlite;

namespace EmuSen.Common.Catalogue
{
    // The catalogue's driver. Galaxia owns the contract and the schema; this owns
    // nothing but the engine that runs them - see EmuSen_Galaxia.md §7.1.
    //
    // It lives here because every consumer that wants a catalogue already
    // references EmuSen and nothing else is common to all of them. The one thing
    // that does not follow is EmuSen.DianaOS, which this project references rather
    // than the other way round: the shell takes an ICatalogue it is handed and
    // cannot construct one.
    public sealed class SqliteCatalogue : ICatalogue, IDisposable
    {
        private readonly SqliteConnection _db;

        private SqliteCatalogue(SqliteConnection db) => _db = db;

        public static SqliteCatalogue Open(string dbPath, string? schemaPath = null)
        {
            string schema = schemaPath ?? Path.Combine(
                AppContext.BaseDirectory, "Library", "Catalogue", "catalogue-schema.sql");

            Directory.CreateDirectory(Path.GetDirectoryName(dbPath)!);
            var db = new SqliteConnection($"Data Source={dbPath}");
            db.Open();

            using SqliteCommand create = db.CreateCommand();
            create.CommandText = File.ReadAllText(schema);
            create.ExecuteNonQuery();

            return new SqliteCatalogue(db);
        }

        public int Count
        {
            get
            {
                using SqliteCommand command = _db.CreateCommand();
                command.CommandText = "SELECT COUNT(*) FROM rom";
                return Convert.ToInt32(command.ExecuteScalar());
            }
        }

        public IReadOnlyList<RomEntry> All() => Query("SELECT * FROM rom ORDER BY name", _ => { });

        public IReadOnlyList<RomEntry> ForSystem(string system) =>
            Query("SELECT * FROM rom WHERE system = $system ORDER BY name",
                c => c.Parameters.AddWithValue("$system", system));

        public IReadOnlyList<RomEntry> ForBoard(string system, string board) =>
            Query("SELECT * FROM rom WHERE system = $system AND board = $board ORDER BY name",
                c =>
                {
                    c.Parameters.AddWithValue("$system", system);
                    c.Parameters.AddWithValue("$board", board);
                });

        public RomEntry? ByPath(string path)
        {
            IReadOnlyList<RomEntry> found = Query("SELECT * FROM rom WHERE path = $path",
                c => c.Parameters.AddWithValue("$path", path));
            return found.Count > 0 ? found[0] : null;
        }

        public IReadOnlyList<RomEntry> Missing()
        {
            var gone = new List<RomEntry>();
            foreach (RomEntry entry in All())
            {
                if (!File.Exists(entry.Path)) gone.Add(entry);
            }
            return gone;
        }

        public void Put(RomEntry entry) => PutAll(new[] { entry });

        // One transaction for the whole scan: a catalogue that is half written and
        // looks complete is worse than one that is obviously absent.
        public void PutAll(IEnumerable<RomEntry> entries)
        {
            using SqliteTransaction transaction = _db.BeginTransaction();
            using SqliteCommand command = _db.CreateCommand();
            command.Transaction = transaction;
            command.CommandText = """
                INSERT INTO rom (path, name, system, bytes, md5, board, header_trust, region,
                                 prg_bytes, chr_bytes, playable, indexed_at)
                VALUES ($path, $name, $system, $bytes, $md5, $board, $trust, $region,
                        $prg, $chr, $playable, datetime('now'))
                ON CONFLICT(path) DO UPDATE SET
                    name = excluded.name, system = excluded.system, bytes = excluded.bytes,
                    md5 = COALESCE(excluded.md5, rom.md5),
                    board = excluded.board, header_trust = excluded.header_trust,
                    region = excluded.region, prg_bytes = excluded.prg_bytes,
                    chr_bytes = excluded.chr_bytes, playable = excluded.playable,
                    indexed_at = excluded.indexed_at
                """;

            foreach (RomEntry entry in entries)
            {
                command.Parameters.Clear();
                command.Parameters.AddWithValue("$path", entry.Path);
                command.Parameters.AddWithValue("$name", entry.Name);
                command.Parameters.AddWithValue("$system", entry.System);
                command.Parameters.AddWithValue("$bytes", entry.Bytes);
                command.Parameters.AddWithValue("$md5", (object?)entry.Md5 ?? DBNull.Value);
                command.Parameters.AddWithValue("$board", (object?)entry.Board ?? DBNull.Value);
                command.Parameters.AddWithValue("$trust", (object?)entry.HeaderTrust ?? DBNull.Value);
                command.Parameters.AddWithValue("$region", (object?)entry.Region ?? DBNull.Value);
                command.Parameters.AddWithValue("$prg", entry.PrgBytes);
                command.Parameters.AddWithValue("$chr", entry.ChrBytes);
                command.Parameters.AddWithValue("$playable", entry.Playable ? 1 : 0);
                command.ExecuteNonQuery();
            }

            transaction.Commit();
        }

        private IReadOnlyList<RomEntry> Query(string sql, Action<SqliteCommand> bind)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = sql;
            bind(command);

            var found = new List<RomEntry>();
            using SqliteDataReader reader = command.ExecuteReader();
            while (reader.Read())
            {
                found.Add(new RomEntry(
                    reader.GetString(reader.GetOrdinal("path")),
                    reader.GetString(reader.GetOrdinal("name")),
                    reader.GetString(reader.GetOrdinal("system")),
                    reader.GetInt64(reader.GetOrdinal("bytes")),
                    Text(reader, "md5"), Text(reader, "board"), Text(reader, "header_trust"),
                    Text(reader, "region"),
                    reader.GetInt64(reader.GetOrdinal("prg_bytes")),
                    reader.GetInt64(reader.GetOrdinal("chr_bytes")),
                    reader.GetInt64(reader.GetOrdinal("playable")) != 0,
                    Text(reader, "indexed_at")));
            }
            return found;
        }

        private static string? Text(SqliteDataReader reader, string column)
        {
            int ordinal = reader.GetOrdinal(column);
            return reader.IsDBNull(ordinal) ? null : reader.GetString(ordinal);
        }

        public void Dispose() => _db.Dispose();
    }
}
