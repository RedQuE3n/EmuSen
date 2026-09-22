using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Microsoft.Data.Sqlite;

namespace EmuSen.Mistress.Library
{
    // One game OpenVGDB knows: its canonical No-Intro name, its title, and the cover address it gives.
    public sealed record OpenVgdbMatch(string RomName, string? Title, string? CoverUrl, string System);

    // OpenEmu's game database, read only and never shipped: fetched on request into home/Library - see EmuSen_Settings_Reference.md §4.39.
    public sealed class OpenVgdb : IDisposable
    {
        public const string FileName = "openvgdb.sqlite";

        private readonly SqliteConnection _db;

        private OpenVgdb(SqliteConnection db) => _db = db;

        public static string DefaultPath => Path.Combine(EmuSen.Galaxia.Library.DataStore.Library, FileName);

        // Null when the file is absent or is not an OpenVGDB.
        public static OpenVgdb? Open(string path)
        {
            if (!File.Exists(path)) return null;
            try
            {
                var db = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = path, Mode = SqliteOpenMode.ReadOnly, Pooling = false }.ToString());
                db.Open();
                using SqliteCommand probe = db.CreateCommand();
                probe.CommandText = "SELECT COUNT(*) FROM ROMs JOIN SYSTEMS USING (systemID) LIMIT 1";
                probe.ExecuteScalar();
                return new OpenVgdb(db);
            }
            catch (SqliteException)
            {
                return null;
            }
        }

        // The file's name without its extension, which for a No-Intro set is the whole identification and costs no read of the file.
        public OpenVgdbMatch? ByName(string stem, IReadOnlyList<string> systems) => Find("r.romExtensionlessFileName = $key", stem, systems);

        // OpenVGDB's hashes are upper-case hex of the bytes the core's descriptor says it hashed.
        public OpenVgdbMatch? ByMd5(string md5, IReadOnlyList<string> systems) => Find("r.romHashMD5 = $key", md5.ToUpperInvariant(), systems);

        private OpenVgdbMatch? Find(string where, string key, IReadOnlyList<string> systems)
        {
            if (systems.Count == 0) return null;
            using SqliteCommand command = _db.CreateCommand();
            string[] names = systems.Select((_, i) => "$s" + i).ToArray();
            command.CommandText =
                "SELECT r.romExtensionlessFileName, l.releaseTitleName, l.releaseCoverFront, s.systemShortName " +
                "FROM ROMs r JOIN SYSTEMS s ON s.systemID = r.systemID LEFT JOIN RELEASES l ON l.romID = r.romID " +
                $"WHERE {where} AND s.systemShortName IN ({string.Join(", ", names)}) " +
                "ORDER BY l.releaseCoverFront IS NULL, l.releaseID LIMIT 1";
            command.Parameters.AddWithValue("$key", key);
            for (int i = 0; i < systems.Count; i++) command.Parameters.AddWithValue(names[i], systems[i]);
            using SqliteDataReader row = command.ExecuteReader();
            if (!row.Read() || row.IsDBNull(0)) return null;
            return new OpenVgdbMatch(row.GetString(0), row.IsDBNull(1) ? null : row.GetString(1), row.IsDBNull(2) ? null : row.GetString(2), row.GetString(3));
        }

        public void Dispose() => _db.Dispose();
    }
}
