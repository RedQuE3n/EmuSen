using System;
using System.IO;
using EmuSen.Crystal.Services;

namespace EmuSen.Crystal.Cheats
{
    public readonly struct CheatDatabaseInstallResult
    {
        public int Installed { get; init; }

        // Entries that were not .cht, or that tried to write outside the target.
        public int Skipped { get; init; }
    }

    // Downloads to the player's machine on request; EmuSen redistributes nothing - see `man cheat`.
    public static class CheatDatabaseInstaller
    {
        // What RetroArch's own "Update Cheats" pulls.
        public const string LibretroCheatsUrl = "https://buildbot.libretro.com/assets/frontend/cheats.zip";

        // CC BY-SA 4.0 requires attribution, so it goes in front of the player, not in a doc.
        public const string Attribution =
            "Cheat data comes from the libretro cheat database (github.com/libretro/libretro-database),\n" +
            "licensed CC BY-SA 4.0. The codes themselves were collected from community sources,\n" +
            "substantially GameHacking.org - credit belongs to their original authors.\n" +
            "EmuSen ships none of this data; it is downloaded to your machine at your request.";

        // Swapped in tests so the unpack path runs without a network.
        public static Func<string, Stream>? FetchOverride { get; set; }

        // The whole call's limit, redirects and body included.
        public static readonly TimeSpan FetchTimeout = TimeSpan.FromMinutes(5);

        public static Stream Fetch(string url) => FetchOverride is not null ? FetchOverride(url) : new MemoryStream(NativeDownload.Fetch(url, FetchTimeout));

        // Every .cht entry written under the target as .NET's zip reader would write it, by emusen-cheats - see EmuSen_Backend.md §15.3, §40.1.
        public static CheatDatabaseInstallResult Install(Stream zip, string targetDirectory)
        {
            byte[] bytes;
            using (zip)
            {
                if (zip.CanSeek) zip.Position = 0;
                var copy = new MemoryStream();
                zip.CopyTo(copy);
                bytes = copy.ToArray();
            }
            return CheatsNative.Install(bytes, targetDirectory);
        }
    }
}
