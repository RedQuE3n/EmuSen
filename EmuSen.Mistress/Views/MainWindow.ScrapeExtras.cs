using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Views
{
    // Pass 8: the name search the game's page asks (EmuSen_BigPicture.md §50), and the cleanup of media whose game is gone; each only when the player presses for it - see EmuSen_BigPicture.md §38.
    public partial class MainWindow
    {
        // Requests this window's name searches sent, for the tests' count and the status window's.
        public int NameSearches { get; private set; }

        // The cleanup's confirm; a test answers it instead of a person.
        internal static Func<MainWindow, string, Task<bool>> ConfirmCleanUp =
            (owner, message) => Dialogs.ConfirmAsync(owner, "Clean Up Orphaned Media", message, "Move Them", "Cancel");

        // Why a name search cannot be sent now, or null.
        private string? NameSearchWhyNot => ScrapeRunning ? "a run is going; a search waits until it ends" : ScreenScraperUnusable();

        // One jeuRecherche for the text the player typed, paced and counted as every request is; nothing else is sent.
        public async Task<SearchAnswer> SearchByNameAsync(string path, string text)
        {
            string name = text.Trim();
            if (name.Length == 0) return new SearchAnswer(ScrapeStatus.BadRequest, [], null, "Type a name to search for.", TimeSpan.Zero);
            if (NameSearchWhyNot is string why) return new SearchAnswer(ScrapeStatus.Failed, [], null, $"ScreenScraper cannot be used: {why}.", TimeSpan.Zero);
            if (SystemOf(new RomEntry(path)) is not string system || !Scraper.SystemIds.ContainsKey(system))
                return new SearchAnswer(ScrapeStatus.BadRequest, [], null, "This game's console has no ScreenScraper system.", TimeSpan.Zero);
            if (!OpenMediaStore() || DeveloperSource() is null) return new SearchAnswer(ScrapeStatus.Failed, [], null, DeveloperCredentials.NoneHere, TimeSpan.Zero);
            NativeScraping native = _native!;
            SearchAnswer found = await Task.Run(() => native.Search(path, system, name)).ConfigureAwait(false);
            if (found.Status is not (ScrapeStatus.BadRequest or ScrapeStatus.DailyQuota) || found.Games.Count > 0) NameSearches++;
            if (found.Status is not (ScrapeStatus.Found or ScrapeStatus.NotFound))
                ErrorLog.Error("scraping", "A name search failed", null, ScrapeRedactor.Redact($"{found.Status}: {found.Detail}"));
            return found;
        }

        // The player's pick: a run of this one game whose lookup is the pick, so its pictures are paced and counted as a run's are.
        public bool ChooseFoundGame(string path, ScrapedGame game)
        {
            if (ScrapeRunning || _scrapeClosed) return false;
            bool started = StartScrape(ScrapeScope.ThisGame(path) with { Refresh = true }, resume: false, game);
            if (started) StatusText.Text = $"Keeping ScreenScraper's game {game.Id} for {Path.GetFileNameWithoutExtension(path)}";
            return started;
        }

        // --- orphaned media ---

        // The files a cleanup would move; nothing when the ROM folder is missing, so an unplugged drive never reads as every game deleted.
        public IReadOnlyList<string> OrphanedMedia()
        {
            if (SettingFolders.Roms(_appSettings) is not { } roms || !Directory.Exists(roms) || !OpenMediaStore()) return [];
            return _native!.Orphans(_allScan.Entries.Where(e => SystemOf(e) is not null).Select(e => (e.FullPath, SystemOf(e)!)));
        }

        // ES-DE's "Orphaned data cleanup" for Mistress's own store: asked first with the count and size, then each file moved to CLEANUP, never deleted.
        public async Task<string> CleanUpOrphansAsync()
        {
            if (ScrapeRunning) return "Wait for the run to end before cleaning up.";
            IReadOnlyList<string> orphans = OrphanedMedia();
            if (orphans.Count == 0) return "No orphaned media: every scraped file belongs to a game in the library.";
            long bytes = orphans.Sum(f => new FileInfo(Path.Combine(MediaStore.DefaultRoot, f)).Length);
            string message = $"{orphans.Count:N0} scraped file(s), {ScrapeSizes.Text(bytes)}, belong to games no longer in the library. "
                + $"They are moved to home/Media/{MediaStore.CleanupFolder}/, not deleted; delete that folder yourself when you are sure. The ROM folder is not touched.";
            if (!await ConfirmCleanUp(this, message)) return "Nothing was moved.";
            (int moved, long size, string folder) = _native!.CleanUp(orphans, DateTimeOffset.Now);
            _scrapeGeneration++;
            ReadScrapeSnapshot();
            ShowLibraryEntries();
            return $"Moved {moved:N0} file(s), {ScrapeSizes.Text(size)}, to {Path.GetRelativePath(DataStore.Media, folder)} in home/Media.";
        }
    }
}
