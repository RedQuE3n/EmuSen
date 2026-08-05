using System.IO;
using EmuSen.Galaxia.Library;

namespace EmuSen.WiseMan.Fixtures
{
    // Finds a real (never-committed) ROM wherever the library still is - see EmuSen_Galaxia.md §3.3.
    public static class RealRom
    {
        // The new location and both old ones, since the library never migrated - see §3.3.
        public static string? Find(string console, string fileName)
        {
            foreach (string root in Roots())
            {
                string candidate = Path.Combine(root, console, fileName);
                if (File.Exists(candidate)) return candidate;
            }
            return null;
        }

        public static bool Exists(string console, string fileName) => Find(console, fileName) is not null;

        private static string[] Roots() => new[]
        {
            DataStore.Games,
            Path.Combine(DataMigration.LegacyRoot, "Games"),
            Path.Combine(DataMigration.LegacyRoot, "Roms"),
        };
    }
}
