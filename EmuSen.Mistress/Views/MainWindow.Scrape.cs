using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia.Threading;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Views
{
    // ScreenScraper as the library's first source of covers, media and game text, with OpenEmu's as the failover - see EmuSen_Settings_Reference.md §4.60 and EmuSen_BigPicture.md §17.
    public partial class MainWindow : IScrapeHost
    {
        // Replaced by tests; nothing under the harness reads the developer's real file.
        internal static Func<DeveloperCredentials?> DeveloperSource = DeveloperCredentials.Load;

        internal static IScrapeClock ScrapeClock = SystemScrapeClock.Instance;

        private MediaStore? _mediaStore;
        private ScrapeQuotaManager? _scrapeQuota;
        private Scraper? _scraper;
        private ScrapeChoices _scrapeChoices = new();
        private bool _developerPresent;
        private bool _scrapeClosed;
        private bool _scrapeRefreshPosted;
        private int _scrapeGeneration;
        private string? _scrapeStopShown;
        private string? _lastThemedScrape;
        private DispatcherTimer? _scrapeThemedRefresh;
        private IReadOnlyDictionary<string, (ScrapeState State, bool HasCover)> _scrapeOutcomes = new Dictionary<string, (ScrapeState, bool)>();
        private IReadOnlyDictionary<string, ScrapedRecord> _scrapedText = new Dictionary<string, ScrapedRecord>();

        public event Action? ScrapeChanged;

        internal Scraper? ScrapeWorker => _scraper;
        internal MediaStore? ScrapeStore => _mediaStore;
        internal ScrapeQuotaManager? ScrapeQuota => _scrapeQuota;
        internal bool ScrapeRefreshTimerRunning => _scrapeThemedRefresh?.IsEnabled == true;

        private string ScrapeMediaKey => $"{_scrapeGeneration}|{_appSettings.OpenEmuFallback}";

        // Called at start and whenever Preferences closes: the worker is rebuilt with the settings and the member account as they now are.
        private void ApplyScraping()
        {
            if (_scrapeClosed) return;
            StopScraper();
            _scrapeChoices = new ScrapeChoices
            {
                Covers = _appSettings.ScrapeCovers, Screenshots = _appSettings.ScrapeScreenshots, Marquees = _appSettings.ScrapeMarquees,
                TitleScreens = _appSettings.ScrapeTitleScreens, Miximages = _appSettings.ScrapeMiximages, Region = _appSettings.ScrapeRegion,
                Language = _appSettings.ScrapeLanguage, RegionFallback = _appSettings.ScrapeRegionFallback, Threads = Math.Max(1, _appSettings.ScrapeThreads),
            };
            DeveloperCredentials? developer = DeveloperSource();
            _developerPresent = developer is not null;
            if (_mediaStore is null && (_appSettings.Scraping && developer is not null || File.Exists(Path.Combine(MediaStore.DefaultRoot, MediaStore.FileName))))
            {
                try { _mediaStore = MediaStore.Open(MediaStore.DefaultRoot); }
                catch (Exception ex) when (ex is IOException or InvalidDataException or UnauthorizedAccessException or Microsoft.Data.Sqlite.SqliteException)
                {
                    StatusText.Text = $"The media store could not be opened: {ScrapeRedactor.Redact(ex.Message)}";
                }
            }

            if (_appSettings.Scraping && developer is not null && _mediaStore is not null)
            {
                _http ??= HttpFactory();
                _scrapeQuota = new ScrapeQuotaManager(_mediaStore, ScrapeClock);
                _scrapeQuota.Changed += OnScrapeQuotaChanged;
                var client = new ScreenScraperClient(_http, developer, MemberAccount.Load());
                _scraper = new Scraper(client, _mediaStore, _scrapeQuota, () => _scrapeChoices, HasHandCover, TransformFor,
                    result => Dispatcher.UIThread.Post(() => ScrapeArrived(result)), ScrapeClock);
                _scraper.Start();
            }
            ReadScrapeSnapshot();
            ScrapeChanged?.Invoke();
        }

        private void OnScrapeQuotaChanged() => Dispatcher.UIThread.Post(() => { if (!_scrapeClosed) ScrapeChanged?.Invoke(); });

        private bool HasHandCover(string system, string path)
        {
            string? console = EmuSen.Cores.CoreCatalog.ByExtension(Path.GetExtension(path))?.Console;
            return console is not null && _artwork.Find(console, Path.GetFileNameWithoutExtension(path)) is not null;
        }

        private static Func<byte[], byte[]>? TransformFor(string path) => EmuSen.Cores.CoreCatalog.ByExtension(Path.GetExtension(path))?.OpenVgdbBytes;

        private void ReadScrapeSnapshot()
        {
            if (_mediaStore is not { IsOpen: true } store) return;
            _scrapeOutcomes = store.OutcomesByPath();
            _scrapedText = store.FoundByPath();
        }

        internal MediaSources MediaSourcesNow() => new(
            (console, title) => _artwork.Find(console, title), MediaStore.DefaultRoot, _appSettings.EsdeMediaDirectory,
            _appSettings.OpenEmuFallback ? OpenEmuCoverDirectory : null);

        // ScreenScraper takes covers while it can be used and wants them; OpenEmu's failover takes them otherwise.
        private bool ScreenScraperCovers => _scraper is not null && _scrapeChoices.Covers && _scrapeQuota?.Usable == true;

        // A tile with no cover: queued for ScreenScraper when it has not answered, else handed to the failover.
        private void CoverWanted(RomEntry entry, CoreDescriptor? core)
        {
            if (ScreenScraperCovers && !_scrapeOutcomes.ContainsKey(entry.FullPath)
                && EmuSen.Cores.CoreCatalog.ShelfByName(entry.Shelf)?.EsdeSystem is { Length: > 0 } system)
            {
                _scraper!.Enqueue(entry.FullPath, system, ScrapePriority.Shown);
                return;
            }
            AskForCover(entry, core);
        }

        // The themed gamelist's selected game goes to the front of the queue as it is shown.
        private void ScrapeSelectedThemedGame()
        {
            if (_scraper is null || _themed?.SelectedGame is not { } game || game.File == _lastThemedScrape) return;
            _lastThemedScrape = game.File;
            if (_scrapeOutcomes.ContainsKey(game.File) || _themed.SelectedSystem is not { } system) return;
            _scraper.Enqueue(game.File, system.System.Name, ScrapePriority.Shown);
        }

        // The console in view first, then the rest; one explicit action, since a whole library is days of quota (§5.5).
        public void ScrapeWholeLibrary()
        {
            if (_scraper is null) return;
            string? current = _themed?.SelectedSystem?.System.Name ?? EmuSen.Cores.CoreCatalog.ShelfByName(_appSettings.SelectedCore)?.EsdeSystem;
            int queued = 0;
            foreach (RomEntry entry in _allScan.Entries.OrderBy(e => EmuSen.Cores.CoreCatalog.ShelfByName(e.Shelf)?.EsdeSystem == current ? 0 : 1))
            {
                if (_scrapeOutcomes.ContainsKey(entry.FullPath) || EmuSen.Cores.CoreCatalog.ShelfByName(entry.Shelf)?.EsdeSystem is not { Length: > 0 } system) continue;
                if (_scraper.Enqueue(entry.FullPath, system, system == current ? ScrapePriority.Console : ScrapePriority.Library)) queued++;
            }
            StatusText.Text = $"{queued} games queued for ScreenScraper.";
            ScrapeChanged?.Invoke();
        }

        private void ScrapeArrived(ScrapeResult result)
        {
            if (_scrapeClosed) return;
            if (result.Outcome == ScrapeOutcome.Stopped && result.Detail != _scrapeStopShown)
            {
                _scrapeStopShown = result.Detail;
                StatusText.Text = ScrapeRedactor.Redact($"ScreenScraper stopped: {result.Detail}." + (_appSettings.OpenEmuFallback ? " OpenEmu's sources fill in covers meanwhile." : ""));
            }
            if (_scrapeRefreshPosted) return;
            _scrapeRefreshPosted = true;
            Dispatcher.UIThread.Post(() =>
            {
                _scrapeRefreshPosted = false;
                if (_scrapeClosed) return;
                _scrapeGeneration++;
                ReadScrapeSnapshot();
                RefreshCovers();
                ScrapeChanged?.Invoke();
            }, DispatcherPriority.Background);
        }

        // New covers into the grid now; the themed view is rebuilt only when it is still, so a glide is never cut short.
        private void RefreshCovers()
        {
            if (LibraryView.IsVisible && LibraryGrid.IsVisible) LibraryGrid.Refresh(_shownEntries);
            if (!ThemedLibraryShown || !LibraryView.IsVisible) return;
            if (_themed?.NextChange(UiClock()) is null) ShowThemedLibrary();
            else
            {
                _scrapeThemedRefresh ??= new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
                _scrapeThemedRefresh.Tick -= OnScrapeThemedRefresh;
                _scrapeThemedRefresh.Tick += OnScrapeThemedRefresh;
                _scrapeThemedRefresh.Start();
            }
        }

        private void OnScrapeThemedRefresh(object? sender, EventArgs e)
        {
            if (_scrapeClosed || !ThemedLibraryShown || !LibraryView.IsVisible) { _scrapeThemedRefresh?.Stop(); return; }
            if (_themed?.NextChange(UiClock()) is not null) return;
            _scrapeThemedRefresh?.Stop();
            ShowThemedLibrary();
        }

        // ScreenScraper's text for the themed view's metadata; the name stays the file's, as the library shows it.
        private SceneGame WithScrapedText(SceneGame game) => _scrapedText.TryGetValue(game.File, out ScrapedRecord? r)
            ? game with
            {
                Description = r.Description, Developer = r.Developer, Publisher = r.Publisher, Genre = r.Genre, Players = r.Players,
                ReleaseDate = r.ReleaseDate, Rating = r.Rating,
            }
            : game;

        bool IScrapeHost.HasDeveloperCredentials => _developerPresent;

        QuotaSnapshot? IScrapeHost.Quota => _scrapeQuota?.Snapshot();

        string IScrapeHost.Status
        {
            get
            {
                if (!_appSettings.Scraping) return "Off.";
                if (!_developerPresent) return "EmuSen's developer file is not on this computer, so ScreenScraper cannot be used here.";
                if (_scraper is null || _scrapeQuota is null) return "Not running.";
                if (_scrapeQuota.Check(out DateTimeOffset? until, out string? why) == QuotaGate.Stopped)
                    return ScrapeRedactor.Redact($"Stopped: {why}" + (until is { } u ? $", until {u.ToLocalTime():ddd HH:mm}." : "."));
                int queued = _scraper.Pending;
                return queued == 0 ? "Running; nothing queued." : $"Running; {queued} games queued.";
            }
        }

        private void StopScraper()
        {
            if (_scrapeQuota is not null) _scrapeQuota.Changed -= OnScrapeQuotaChanged;
            _scraper?.Dispose();
            _scraper = null;
            _scrapeQuota = null;
        }

        // The worker, its timer and media.db end with the window, before the HTTP client they use - see EmuSen_BigPicture.md §17.
        private void StopScraping()
        {
            _scrapeClosed = true;
            _scrapeThemedRefresh?.Stop();
            StopScraper();
            _mediaStore?.Dispose();
            _mediaStore = null;
            ScrapeChanged = null;
        }
    }
}
