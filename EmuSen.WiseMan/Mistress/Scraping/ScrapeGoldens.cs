using System;
using System.Linq;
using EmuSen.WiseMan.Crystal;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The scraping classes' goldens: a seeded media.db's rows kept as statements, and the test's folder written <root> - see EmuSen_Backend.md §40.5.
    internal static class ScrapeGoldens
    {
        public static string Plain(string text, string root) => text.Replace(root, "<root>", StringComparison.Ordinal);

        // Statements are kept apart by the record separator, which no seeded text holds.
        private const char Between = '\u001E';

        // A store's rows as the C#'s own calls seeded them before its deletion, from the recording, laid into a store the library made.
        public static void Seed(ParityGolden golden, string key, string db, string root)
        {
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
