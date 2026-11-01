using System.Collections.Generic;

namespace EmuSen.Tools.Library
{
    // One file of the library as ScreenScraper answered it: recognised with its game id and the hashes it was asked by, or not recognised.
    public sealed record ScrapedFile(string Path, bool Recognised, long? GameId, string Md5, long Bytes, string? Name = null);

    // What the library manager reads and changes in the host's games.db and media.db; a host lends it, and without one the tool says so - see EmuSen_LibraryTools.md §10.2.
    public interface ILibraryRecords
    {
        // The full paths of the games hidden from the library.
        IReadOnlySet<string> Hidden();

        // Hides a game from the library, or shows it again; only games.db changes.
        void SetHidden(string path, bool hidden);

        // Every file ScreenScraper has answered, recognised or not.
        IReadOnlyList<ScrapedFile> Scraped();
    }
}
