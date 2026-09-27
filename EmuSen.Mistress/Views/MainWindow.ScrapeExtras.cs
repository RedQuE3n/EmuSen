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
    // Pass 8: Find by Name and its chooser, and the cleanup of media whose game is gone; each only when the player presses for it - see EmuSen_BigPicture.md §38.
    public partial class MainWindow : IFindByNameHost
    {
        private FindByNameWindow? _findByName;

        public FindByNameWindow? FindByNameShown => _findByName;

        // Requests this window's name searches sent, for the tests' count and the status window's.
        public int NameSearches { get; private set; }

        // The cleanup's confirm; a test answers it instead of a person.
        internal static Func<MainWindow, string, Task<bool>> ConfirmCleanUp =
            (owner, message) => Dialogs.ConfirmAsync(owner, "Clean Up Orphaned Media", message, "Move Them", "Cancel");

        private void ShowFindByName(string path, string title)
        {
            if (_scrapeClosed) return;
            if (_findByName is not null) { SheetLayer.Activate(_findByName); return; }
            _gameOptions?.Close();
            var window = _findByName = new FindByNameWindow(this, path, title, ScrapeRules.SearchName(Path.GetFileName(path)));
            window.Closed += (_, _) => { if (ReferenceEquals(_findByName, window)) _findByName = null; };
            _ = SheetLayer.Show(window, this);
        }

        string? IFindByNameHost.WhyNot => ScrapeRunning ? "a run is going; Find by Name waits until it ends" : ScreenScraperUnusable();

        int IFindByNameHost.KindsTurnedOn => _scrapeChoices.Kinds().Count();

        string IFindByNameHost.SystemOf(string path) => SystemOf(new RomEntry(path)) ?? "";

        // One jeuRecherche for the text the player typed, paced and counted as every request is; nothing else is sent.
        public async Task<SearchAnswer> SearchByNameAsync(string path, string text)
        {
            string name = text.Trim();
            if (name.Length == 0) return new SearchAnswer(ScrapeStatus.BadRequest, [], null, "Type a name to search for.", TimeSpan.Zero);
            if (((IFindByNameHost)this).WhyNot is string why) return new SearchAnswer(ScrapeStatus.Failed, [], null, $"ScreenScraper cannot be used: {why}.", TimeSpan.Zero);
            if (SystemOf(new RomEntry(path)) is not string system || !Scraper.SystemIds.TryGetValue(system, out int systemId))
                return new SearchAnswer(ScrapeStatus.BadRequest, [], null, "This game's console has no ScreenScraper system.", TimeSpan.Zero);
            if (!OpenMediaStore() || DeveloperSource() is not { } developer) return new SearchAnswer(ScrapeStatus.Failed, [], null, DeveloperCredentials.NoneHere, TimeSpan.Zero);

            ScrapeQuotaManager quota = _scrapeQuota!;
            _http ??= HttpFactory();
            var client = new ScreenScraperClient(_http, developer, MemberAccount.Load());
            SearchAnswer answer;
            try
            {
                if (!await quota.TakeTurnAsync(CancellationToken.None))
                    return new SearchAnswer(ScrapeStatus.DailyQuota, [], null, ScreenScraperUnusable() ?? "the quota stopped it", TimeSpan.Zero);
                NameSearches++;
                answer = await client.JeuRechercheAsync(systemId, name, CancellationToken.None);
            }
            catch (ObjectDisposedException) { return new SearchAnswer(ScrapeStatus.Failed, [], null, "Mistress is closing.", TimeSpan.Zero); }
            if (_scrapeClosed) return answer;
            quota.Observe(answer.Quota);
            if (answer.Status is ScrapeStatus.Found or ScrapeStatus.NotFound or ScrapeStatus.Malformed) quota.Counted(answer.Status == ScrapeStatus.NotFound);
            quota.Answered(answer.Status);
            if (answer.Status is not (ScrapeStatus.Found or ScrapeStatus.NotFound))
                ErrorLog.Error("scraping", "A name search failed", null, ScrapeRedactor.Redact($"{answer.Status}: {answer.Detail}"));
            return answer with { Detail = ScrapeRedactor.Redact(answer.Detail) };
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

        // Each system's games by the names the store files them under, for the systems the last scan found games of.
        private IReadOnlyDictionary<string, IReadOnlySet<string>> StoreNames()
        {
            var names = new Dictionary<string, HashSet<string>>(StringComparer.Ordinal);
            string? roms = _appSettings.RomDirectory;
            foreach (RomEntry entry in _allScan.Entries)
            {
                if (SystemOf(entry) is not string system) continue;
                if (!names.TryGetValue(system, out HashSet<string>? set)) names[system] = set = new HashSet<string>(StringComparer.Ordinal);
                string stem = Path.GetFileNameWithoutExtension(entry.FullPath), folder = GameFolders.Of(roms, entry.FullPath);
                set.Add(folder.Length == 0 ? stem : folder + "/" + stem);
                for (string f = folder; f.Length > 0; f = f.Contains('/') ? f[..f.LastIndexOf('/')] : "") set.Add(f);
            }
            return names.ToDictionary(n => n.Key, n => (IReadOnlySet<string>)n.Value, StringComparer.Ordinal);
        }

        // The files a cleanup would move; nothing when the ROM folder is missing, so an unplugged drive never reads as every game deleted.
        public IReadOnlyList<string> OrphanedMedia()
        {
            if (_appSettings.RomDirectory is not { Length: > 0 } roms || !Directory.Exists(roms) || !OpenMediaStore()) return [];
            return _mediaStore!.Orphans(StoreNames());
        }

        // ES-DE's "Orphaned data cleanup" for Mistress's own store: asked first with the count and size, then each file moved to CLEANUP, never deleted.
        public async Task<string> CleanUpOrphansAsync()
        {
            if (ScrapeRunning) return "Wait for the run to end before cleaning up.";
            IReadOnlyList<string> orphans = OrphanedMedia();
            if (orphans.Count == 0) return "No orphaned media: every scraped file belongs to a game in the library.";
            long bytes = orphans.Sum(f => new FileInfo(Path.Combine(_mediaStore!.Root, f)).Length);
            string message = $"{orphans.Count:N0} scraped file(s), {ScrapePreferencesPane.Size(bytes)}, belong to games no longer in the library. "
                + $"They are moved to home/Media/{MediaStore.CleanupFolder}/, not deleted; delete that folder yourself when you are sure. The ROM folder is not touched.";
            if (!await ConfirmCleanUp(this, message)) return "Nothing was moved.";
            (int moved, long size, string folder) = _mediaStore!.CleanUp(orphans, DateTimeOffset.Now);
            _scrapeGeneration++;
            ReadScrapeSnapshot();
            ShowLibraryEntries();
            return $"Moved {moved:N0} file(s), {ScrapePreferencesPane.Size(size)}, to {Path.GetRelativePath(DataStore.Media, folder)} in home/Media.";
        }
    }
}
