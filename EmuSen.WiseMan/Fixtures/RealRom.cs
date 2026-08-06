using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;

namespace EmuSen.WiseMan.Fixtures
{
    // Finds a real (never-committed) ROM in the user's own library - see EmuSen_Galaxia.md §3.3.
    public static class RealRom
    {
        public static string? Find(string console, string fileName)
        {
            foreach (string root in Roots())
            {
                string candidate = Path.Combine(root, console, fileName);
                if (File.Exists(candidate)) return candidate;
            }

            // A real library sorts by region under the console, so the flat path is only the first guess.
            foreach (string root in Roots())
            {
                string consoleDirectory = Path.Combine(root, console);
                if (!Directory.Exists(consoleDirectory)) continue;

                string? found = Directory
                    .EnumerateFiles(consoleDirectory, fileName, SearchOption.AllDirectories)
                    .FirstOrDefault();

                if (found is not null) return found;
            }

            return null;
        }

        public static bool Exists(string console, string fileName) => Find(console, fileName) is not null;

        // The configured library is the answer; the in-tree directories are only a fallback - see §3.3.
        private static IEnumerable<string> Roots()
        {
            if (AppSettings.Load().RomDirectory is { Length: > 0 } configured) yield return configured;

            yield return DataStore.Games;
            yield return Path.Combine(DataMigration.LegacyRoot, "Games");
            yield return Path.Combine(DataMigration.LegacyRoot, "Roms");
        }
    }
}
