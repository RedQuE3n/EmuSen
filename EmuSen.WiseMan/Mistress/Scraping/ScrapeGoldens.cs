using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using EmuSen.WiseMan.Crystal;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The scraping classes' goldens: a seeded media.db's rows kept as statements, and the test's folder written <root> - see EmuSen_Backend.md §40.5.
    internal static class ScrapeGoldens
    {
        // Which half a class runs on while both exist: the C# to record the goldens, the library otherwise.
        public static bool CSharp => Environment.GetEnvironmentVariable("EMUSEN_SCRAPE_GOLDEN_SIDE") == "csharp";

        public static string Plain(string text, string root) => text.Replace(root, "<root>", StringComparison.Ordinal);

        private static readonly string[] Tables = ["scrape_game", "scrape_media", "scrape_file", "scrape_queue", "quota_day", "store_move", "store_step"];

        private static string Literal(object value) => value switch
        {
            DBNull => "NULL",
            long l => l.ToString(CultureInfo.InvariantCulture),
            double d => Real(d),
            byte[] b => "X'" + Convert.ToHexString(b) + "'",
            _ => "'" + Convert.ToString(value, CultureInfo.InvariantCulture)!.Replace("'", "''") + "'",
        };

        private static string Real(double d)
        {
            string text = d.ToString("R", CultureInfo.InvariantCulture);
            return text.Contains('.') || text.Contains('E') ? text : text + ".0";
        }

        // Statements are kept apart by the record separator, which no seeded text holds.
        private const char Between = '\u001E';

        // Every row of a store as one INSERT, in the tables' order and each table's rowid order.
        public static IEnumerable<string> Rows(string db)
        {
            using var c = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = db, Mode = SqliteOpenMode.ReadOnly, Pooling = false }.ToString());
            c.Open();
            var rows = new List<string>();
            foreach (string table in Tables)
            {
                using SqliteCommand q = c.CreateCommand();
                q.CommandText = $"SELECT * FROM {table} ORDER BY rowid";
                using SqliteDataReader r = q.ExecuteReader();
                while (r.Read())
                {
                    string columns = string.Join(", ", Enumerable.Range(0, r.FieldCount).Select(r.GetName));
                    rows.Add($"INSERT OR REPLACE INTO {table} ({columns}) VALUES ({string.Join(", ", Enumerable.Range(0, r.FieldCount).Select(i => Literal(r.GetValue(i))))})");
                }
            }
            return rows;
        }

        // A seed recorded from the C#'s own calls: recorded into the golden under the variable, and otherwise laid into a store the library made.
        public static void Seed(ParityGolden golden, string key, string db, string root, Action csharp)
        {
            if (CSharp)
            {
                csharp();
                golden.Add(key, Plain(string.Join(Between, Rows(db)), root));
                return;
            }
            var recorded = ParityGolden.Read(golden.Name).Where(l => l.Key == key).Select(l => l.Answer).ToList();
            Assert.True(recorded.Count == 1, $"{golden.Name}: no seed {key}");
            golden.Add(key, recorded[0]);
            using var c = new SqliteConnection(new SqliteConnectionStringBuilder { DataSource = db, Pooling = false }.ToString());
            c.Open();
            using SqliteTransaction t = c.BeginTransaction();
            foreach (string statement in recorded[0].Split(Between).Where(s => s.Length > 0))
            {
                using SqliteCommand command = c.CreateCommand();
                command.Transaction = t;
                command.CommandText = statement.Replace("<root>", root, StringComparison.Ordinal);
                command.ExecuteNonQuery();
            }
            t.Commit();
        }
    }
}
