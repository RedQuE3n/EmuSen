using System;
using System.IO;
using System.IO.Compression;
using System.Net.Http;

namespace EmuSen.DianaOS.DianaOS.Var
{
    public readonly struct CheatDatabaseInstallResult
    {
        public int Installed { get; init; }

        // Entries that were not .cht, or that tried to write outside the target.
        public int Skipped { get; init; }
    }

    // Downloads to the user's machine on request; EmuSen redistributes nothing - see `man cheat`.
    public static class CheatDatabaseInstaller
    {
        // What RetroArch's own "Update Cheats" pulls.
        public const string LibretroCheatsUrl = "https://buildbot.libretro.com/assets/frontend/cheats.zip";

        // CC BY-SA 4.0 requires attribution, so it goes in front of the user, not in a doc.
        public const string Attribution =
            "Cheat data comes from the libretro cheat database (github.com/libretro/libretro-database),\n" +
            "licensed CC BY-SA 4.0. The codes themselves were collected from community sources,\n" +
            "substantially GameHacking.org - credit belongs to their original authors.\n" +
            "EmuSen ships none of this data; it is downloaded to your machine at your request.";

        // Swapped in tests so the unpack path runs without a network.
        public static Func<string, Stream>? FetchOverride { get; set; }

        public static Stream Fetch(string url)
        {
            if (FetchOverride is not null) return FetchOverride(url);

            using var http = new HttpClient { Timeout = TimeSpan.FromMinutes(5) };
            // ZipArchive must seek, and an HTTP response stream cannot.
            byte[] bytes = http.GetByteArrayAsync(url).GetAwaiter().GetResult();
            return new MemoryStream(bytes);
        }

        public static CheatDatabaseInstallResult Install(Stream zip, string targetDirectory)
        {
            Directory.CreateDirectory(targetDirectory);
            string root = Path.GetFullPath(targetDirectory);

            int installed = 0, skipped = 0;
            using var archive = new ZipArchive(zip, ZipArchiveMode.Read);

            foreach (ZipArchiveEntry entry in archive.Entries)
            {
                if (entry.FullName.EndsWith('/') || entry.Length == 0 && entry.Name.Length == 0) continue;
                if (!entry.Name.EndsWith(".cht", StringComparison.OrdinalIgnoreCase)) { skipped++; continue; }

                // Zip-slip: an entry naming ../.. must not write outside the target.
                string destination = Path.GetFullPath(Path.Combine(root, entry.FullName));
                if (!destination.StartsWith(root + Path.DirectorySeparatorChar, StringComparison.Ordinal))
                {
                    skipped++;
                    continue;
                }

                try
                {
                    Directory.CreateDirectory(Path.GetDirectoryName(destination)!);
                    entry.ExtractToFile(destination, overwrite: true);
                    installed++;
                }
                catch
                {
                    skipped++;
                }
            }

            return new CheatDatabaseInstallResult { Installed = installed, Skipped = skipped };
        }
    }
}
