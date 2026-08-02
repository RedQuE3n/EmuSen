using System;
using System.IO;
using System.IO.Compression;
using System.Net.Http;

namespace EmuSen.DianaOS.DianaOS.Var
{
    public readonly struct CheatDatabaseInstallResult
    {
        public int Installed { get; init; }

        // Entries that were not .cht, or that tried to write outside the
        // target directory - see CheatDatabaseInstaller.Install.
        public int Skipped { get; init; }
    }

    // Fetches and unpacks a .cht archive into the user's own cheat tree.
    // EmuSen ships no cheat data of its own and redistributes none - this
    // downloads to the user's machine, on their explicit request, from
    // upstream. See `man cheat`.
    public static class CheatDatabaseInstaller
    {
        // What RetroArch's own "Update Cheats" pulls.
        public const string LibretroCheatsUrl = "https://buildbot.libretro.com/assets/frontend/cheats.zip";

        // Shown before the download and again after it. CC BY-SA 4.0
        // requires attribution, and the codes themselves were aggregated
        // from elsewhere - both facts belong in front of the user rather
        // than buried in a doc.
        public const string Attribution =
            "Cheat data comes from the libretro cheat database (github.com/libretro/libretro-database),\n" +
            "licensed CC BY-SA 4.0. The codes themselves were collected from community sources,\n" +
            "substantially GameHacking.org - credit belongs to their original authors.\n" +
            "EmuSen ships none of this data; it is downloaded to your machine at your request.";

        // Swapped out in tests so the unpack path can be exercised without
        // a network - same override shape ConfigStore.OverrideDirectory uses.
        public static Func<string, Stream>? FetchOverride { get; set; }

        public static Stream Fetch(string url)
        {
            if (FetchOverride is not null) return FetchOverride(url);

            using var http = new HttpClient { Timeout = TimeSpan.FromMinutes(5) };
            // Buffered to a MemoryStream: ZipArchive needs to seek, and a
            // raw HTTP response stream cannot.
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

                // An archive entry naming ../../something must not be able
                // to write outside the target - the classic zip-slip.
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
