using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Threading;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Views
{
    // ScreenScraper first and OpenEmu's sources as the failover, both only inside a run the player started - see EmuSen_Settings_Reference.md §4.60 and EmuSen_BigPicture.md §17.14.
    public partial class MainWindow : IScrapeHost
    {
        // Replaced by tests; nothing under the harness reads the developer's real file.
        internal static Func<DeveloperCredentials?> DeveloperSource = DeveloperCredentials.Load;

        internal static IScrapeClock ScrapeClock = SystemScrapeClock.Instance;

        // The confirm step before a run; a test answers it instead of a person.
        internal static Func<MainWindow, string, string, Task<bool>> ConfirmScrape =
            (owner, title, message) => owner.AskAsync(owner, title, message, "Scrape", "Cancel");

        private ScrapeProgress? _scrapeRun;
        private ScrapeChoices _scrapeChoices = new();
        private DeveloperOrigin? _developerOrigin;
        private bool _developerPresent => _developerOrigin is not null;
        private bool _scrapeClosed;
        private bool _scrapeRefreshPosted;
        private int _scrapeGeneration;
        private DispatcherTimer? _scrapeThemedRefresh;
        private IReadOnlyDictionary<string, (ScrapeState State, bool HasCover)> _scrapeOutcomes = new Dictionary<string, (ScrapeState, bool)>();
        private IReadOnlyDictionary<string, ScrapedRecord> _scrapedText = new Dictionary<string, ScrapedRecord>();
        private ScrapeStatusWindow? _scrapeStatus;
        private MemberAccount _member = new("", "");
        private ScrapeMember? _memberChecked;
        private string? _scrapeStatusLine;

        // The status window's cancel confirm; a test answers it instead of a person.
        internal static Func<Window, Task<bool>> ConfirmCancelScrape =
            owner => Dialogs.ConfirmAsync(owner, "Cancel Scraping", "Stop this run? What it has not reached stays queued, and Resume in Settings ▸ Library ▸ Scraping goes on from there.", "Cancel Scraping", "Keep Going");

        public event Action? ScrapeChanged;

        public ScrapeStatusWindow? ScrapeStatusShown => _scrapeStatus;

        internal bool ScrapeRefreshTimerRunning => _scrapeThemedRefresh?.IsEnabled == true;

        private string ScrapeMediaKey => $"{_scrapeGeneration}|{_appSettings.OpenEmuFallback}";

        // At start and when Preferences closes: the settings, whether the developer file is here, and media.db opened to read. It starts nothing.
        private void ApplyScraping()
        {
            if (_scrapeClosed) return;
            _scrapeChoices = new ScrapeChoices
            {
                Covers = _appSettings.ScrapeCovers, Screenshots = _appSettings.ScrapeScreenshots, Marquees = _appSettings.ScrapeMarquees,
                TitleScreens = _appSettings.ScrapeTitleScreens, Miximages = _appSettings.ScrapeMiximages, Region = _appSettings.ScrapeRegion,
                Language = _appSettings.ScrapeLanguage, RegionFallback = _appSettings.ScrapeRegionFallback, Threads = ScrapeRules.ThreadsWanted(_appSettings.ScrapeThreads),
                BackCovers = _appSettings.ScrapeBackCovers, Boxes3D = _appSettings.Scrape3DBoxes, PhysicalMedia = _appSettings.ScrapePhysicalMedia,
                FanArt = _appSettings.ScrapeFanArt, Manuals = _appSettings.ScrapeManuals, Videos = _appSettings.ScrapeVideos,
            };
            _developerOrigin = DeveloperSource()?.Origin;
            _member = MemberAccount.Load();
            _storeRefused = false;
            if (_native is { } open) open.Settings(NativeSettings());
            else if (File.Exists(Path.Combine(MediaStore.DefaultRoot, MediaStore.FileName))) OpenNative();
            _sources?.Settings(NativeSettings());
            ReadScrapeSnapshot();
            ScrapeChanged?.Invoke();
        }

        private bool OpenMediaStore() => OpenNative();

        private void ReadScrapeSnapshot()
        {
            if (_native is not { } n) return;
            _scrapeOutcomes = n.OutcomesByPath();
            _scrapedText = n.FoundByPath();
        }

        internal MediaSources MediaSourcesNow() => new(SourcesHandle(), CoverChoiceStamp);

        // Each packaged game's file and its package's folder, from the last scan of the whole library - see EmuSen_GamePackages.md §18.3.
        private IReadOnlyDictionary<string, string> PackagesByGame
        {
            get
            {
                if (!ReferenceEquals(_packagesScan, _allScan))
                    (_packagesScan, _packagesByGame) = (_allScan, _allScan.Entries.Where(e => e.Package is not null).GroupBy(e => e.FullPath, StringComparer.Ordinal).ToDictionary(g => g.Key, g => g.First().Package!, StringComparer.Ordinal));
                return _packagesByGame;
            }
        }

        private RomLibraryResult? _packagesScan;
        private IReadOnlyDictionary<string, string> _packagesByGame = new Dictionary<string, string>();

        // Why ScreenScraper cannot answer a run now, or null when it can.
        private string? ScreenScraperUnusable()
        {
            if (!_appSettings.Scraping) return "it is switched off";
            if (!_developerPresent) return DeveloperCredentials.NoneHere;
            if (_native is not { } n) return null;
            QuotaSnapshot q = n.Quota();
            return q.Gate == QuotaGate.Stopped ? q.Reason + (q.Until is { } stop ? $", until {stop.ToLocalTime():ddd HH:mm}" : "") : null;
        }

        private static string? SystemOf(RomEntry entry) => EmuSen.Mistress.Library.RuntimeCatalog.ShelfByName(entry.Shelf)?.EsdeSystem is { Length: > 0 } s ? s : null;

        // The game a single-game scope names; a wide run's games are chosen by the scrape module (ChosenGames).
        private List<RomEntry> ScopeGames(ScrapeScope scope)
        {
            IEnumerable<RomEntry> games = scope.Game is string one
                ? _allScan.Entries.Where(e => e.FullPath == one).DefaultIfEmpty(new RomEntry(one))
                : _allScan.Entries.Where(e => scope.Shelf is null || e.Shelf == scope.Shelf);
            games = games.Where(e => SystemOf(e) is not null && File.Exists(e.FullPath));
            if (scope.Game is null) games = games.Where(e => !ExcludedFromMultiScrape(e.FullPath)).Where(Meets(scope.Effective));
            return games.ToList();
        }

        // ES-DE's "Scrape these games": no metadata is no description; no game image is none of miximage, screenshot, title screen and cover (USERGUIDE "Scraper").
        private Func<RomEntry, bool> Meets(ScrapeCriteria criteria)
        {
            MediaSources? sources = criteria is ScrapeCriteria.NoGameImage or ScrapeCriteria.NoGameVideo ? MediaSourcesNow() : null;
            bool Has(RomEntry e, string type) => SystemOf(e) is string system && sources!.Locate(system, e.FullPath, type).Path is not null;
            return criteria switch
            {
                ScrapeCriteria.Favourites => e => _records.IsFavourite(e.FullPath),
                ScrapeCriteria.NoMetadata => e => MetadataFor(e.FullPath).DescriptionText is null,
                ScrapeCriteria.NoGameImage => e => CoverPathFor(e) is null && !Has(e, "miximage") && !Has(e, "screenshot") && !Has(e, "titlescreen"),
                ScrapeCriteria.NoGameVideo => e => !Has(e, "video"),
                ScrapeCriteria.NoCover => e => CoverPathFor(e) is null,
                _ => _ => true,
            };
        }

        // The confirm step's numbers: a bound of one lookup and one request a kind, and what §38's offer rates expect of the kinds turned on.
        public ScrapePlan Plan(ScrapeScope scope)
        {
            if (!OpenNative())
                return new ScrapePlan(0, 0, 0, null, TimeSpan.Zero, false, "the media store could not be opened", _appSettings.OpenEmuFallback);
            return _native!.Plan(ChosenGames(scope).Select(g => g.Path), scope.Game is not null, scope.Refresh, _scrapeChoices.Kinds(), _appSettings.Scraping, _appSettings.OpenEmuFallback);
        }

        public bool ScrapeRunning => _scrapeRun is { State: ScrapeRunState.Running };

        public ScrapeProgress? Progress => _scrapeRun;

        // Games an interrupted run left queued; offered as Resume, never resumed by itself.
        public int Interrupted => ScrapeRunning ? 0 : _native?.QueueLength ?? 0;

        // Asks the player with the count and the cost, then starts; false when they said no or there is nothing to do.
        public async Task<bool> ConfirmAndScrapeAsync(ScrapeScope scope)
        {
            if (ScrapeRunning || _scrapeClosed) return false;
            // ES-DE's single-game scraper follows "Overwrite files and data" too; a wide run's scope carries its own choice.
            if (scope.Game is not null && _appSettings.ScrapeRefresh) scope = scope with { Refresh = true };
            ScrapePlan plan = Plan(scope);
            if (plan.Games == 0)
            {
                StatusText.Text = plan.Describe();
                return false;
            }
            string title = scope.Game is not null ? "Scrape This Game" : "Scrape Games";
            if (!await ConfirmScrape(this, title, plan.Describe())) return false;
            return StartScrape(scope);
        }

        // One run over a scope: the queue is replaced by it, ScreenScraper asked where it can be, OpenEmu's sources where it has nothing.
        public bool StartScrape(ScrapeScope scope, bool resume = false) => StartScrape(scope, resume, null);

        private bool StartScrape(ScrapeScope scope, bool resume, ScrapedGame? chosen)
        {
            if (ScrapeRunning || _scrapeClosed || !OpenMediaStore()) return false;
            return StartNative(scope, resume, chosen);
        }

        // The status line as a run writes it; clicking it while it shows the run opens the status window.
        private void ScrapeLine(string text)
        {
            _scrapeStatusLine = ScrapeRedactor.Redact(text);
            StatusText.Text = _scrapeStatusLine;
        }

        private void SetUpScraping() => StatusText.PointerPressed += (_, _) =>
        {
            if (_scrapeRun is not null && StatusText.Text == _scrapeStatusLine) ShowScrapeStatus();
        };

        // Opens the status window, or brings it forward; it reads the run and asks no server - see EmuSen_Settings_Reference.md §4.57.
        public void ShowScrapeStatus()
        {
            if (_scrapeClosed) return;
            if (_scraperWindow is { } scraper)
            {
                SheetLayer.Activate(scraper);
                return;
            }
            if (_scrapeStatus is { } open)
            {
                open.Refresh();
                SheetLayer.Activate(open);
                return;
            }
            var window = new ScrapeStatusWindow(this);
            _scrapeStatus = window;
            window.Closed += (_, _) => { if (ReferenceEquals(_scrapeStatus, window)) _scrapeStatus = null; };
            _ = SheetLayer.Show(window, this);
        }

        public bool ScrapePaused => _scrapeRun is { State: ScrapeRunState.Running, IsPaused: true };

        // Pause is offered only while ScreenScraper's workers run; the failover's lookups are not paused.
        public bool CanPauseScrape => ScrapeRunning && _native is not null && _nativeWorkers;

        public void SetScrapePaused(bool paused)
        {
            if (!CanPauseScrape || _scrapeRun is not { } run || run.IsPaused == paused) return;
            _native?.Pause(paused);
            run.SetPaused(paused, ScrapeNow);
            ScrapeLine(run.Describe());
            ScrapeChanged?.Invoke();
        }

        public async Task<bool> ConfirmCancelAsync(Window owner)
        {
            if (!ScrapeRunning || !await ConfirmCancelScrape(owner)) return false;
            CancelScrape();
            return true;
        }

        public DateTimeOffset ScrapeNow => _native is { } n ? n.Now : ScrapeClock.Now;

        public ScrapeQuota? ScrapeLimits => _native is not null ? _nativeLimits : null;

        public MemberAccount Member => _member;

        public ScrapeMember? MemberChecked => _memberChecked;

        // Log In: one ssuserInfos request with what was typed; the account is kept only when ScreenScraper accepts it.
        public async Task<SignInAnswer> SignInAsync(string user, string password)
        {
            var account = new MemberAccount(user.Trim(), password);
            SignInAnswer answer = await CheckMemberAsync(account);
            if (answer.SignedIn) UseMember(new MemberAccount(account.User, account.Password) { Verified = DateTime.UtcNow }, answer.Member);
            return answer;
        }

        // Check: the stored account, as an old file from the two boxes has never been; kept whatever the answer.
        public async Task<SignInAnswer> CheckMemberAsync()
        {
            MemberAccount stored = MemberAccount.Load();
            SignInAnswer answer = await CheckMemberAsync(stored);
            if (answer.SignedIn) UseMember(new MemberAccount(stored.User, stored.Password) { Verified = DateTime.UtcNow }, answer.Member);
            return answer;
        }

        private async Task<SignInAnswer> CheckMemberAsync(MemberAccount account)
        {
            if (!account.IsSet) return new SignInAnswer(SignInResult.Incomplete, null, "Type your ScreenScraper name and password first.");
            if (_scrapeClosed || DeveloperSource() is not { } developer) return SignInAnswer.NoDeveloper();
            if (!OpenNative()) return new SignInAnswer(SignInResult.Unreachable, null, "The media store could not be opened, so nothing can be sent to ScreenScraper.");
            NativeScraping n = _native!;
            SignInAnswer said = await Task.Run(() => n.SignIn(account.User, account.Password));
            ScrapeChanged?.Invoke();
            return said;
        }

        private void UseMember(MemberAccount account, ScrapeMember? checkedAs)
        {
            _member = account;
            _memberChecked = checkedAs;
            ScrapeChanged?.Invoke();
        }

        // Log Out: the file is deleted, and a run in progress sends no ssid or sspassword from its next request.
        public string SignOut()
        {
            bool had = OpenNative() ? _native!.SignOut() : MemberAccount.Delete();
            _member = new MemberAccount("", "");
            _memberChecked = null;
            ScrapeChanged?.Invoke();
            return had
                ? "Signed out: screenscraper.json was deleted. Runs now use EmuSen's developer credentials alone."
                : "Signed out. There was no account file to delete.";
        }

        public bool ResumeScrape() => StartScrape(new ScrapeScope(), resume: true);

        public void CancelScrape()
        {
            if (!ScrapeRunning) return;
            _native?.Cancel();
            StopPump();
            _nativeRun++;
            EndRun(ScrapeRunEnd.Cancelled);
        }

        private void EndRun(ScrapeRunEnd end)
        {
            if (_scrapeRun is not { } run) return;
            run.SetPaused(false, ScrapeNow);
            run.Ended = ScrapeNow;
            run.State = end switch { ScrapeRunEnd.Done => ScrapeRunState.Done, ScrapeRunEnd.Stopped => ScrapeRunState.Stopped, _ => ScrapeRunState.Cancelled };
            run.Touch();
            if (!_scrapeClosed) ScrapeLine(run.Describe());
            PostCoverRefresh();
            ScrapeChanged?.Invoke();
        }

        private void PostCoverRefresh()
        {
            if (_scrapeRefreshPosted || _scrapeClosed) return;
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

        // The game a pad's "Scrape This Game" means: the themed gamelist's selection, else the library's.
        private string? GameToScrape => ThemedLibraryShown && LibraryView.IsVisible ? _themed?.SelectedGame?.File : SelectedLibraryEntry?.FullPath;

        bool IScrapeHost.HasDeveloperCredentials => _developerPresent;
        DeveloperOrigin? IScrapeHost.DeveloperOrigin => _developerOrigin;

        QuotaSnapshot? IScrapeHost.Quota => _native?.Quota();

        string IScrapeHost.Status
        {
            get
            {
                if (_scrapeRun is { State: ScrapeRunState.Running } running) return running.Describe();
                string idle = ScreenScraperUnusable() is string why ? $"ScreenScraper cannot be used: {why}." : "Nothing is scraped until you start it.";
                if (Interrupted > 0) idle += $" {Interrupted:N0} games are left from a run that did not finish; Resume goes on with them.";
                if (_scrapeRun is { } last) idle = last.Describe() + " " + idle;
                return ScrapeRedactor.Redact(idle);
            }
        }

        // The worker, its timer and media.db end with the window, before the HTTP client they use - see EmuSen_BigPicture.md §17.8.
        private void StopScraping()
        {
            _scrapeClosed = true;
            _scrapeThemedRefresh?.Stop();
            _scrapeStatus?.Close();
            _scrapeStatus = null;
            StopNative();
            ScrapeChanged = null;
        }
    }
}
