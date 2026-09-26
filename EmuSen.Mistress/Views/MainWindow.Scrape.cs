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
            (owner, title, message) => Dialogs.ConfirmAsync(owner, title, message, "Scrape", "Cancel");

        private MediaStore? _mediaStore;
        private ScrapeQuotaManager? _scrapeQuota;
        private Scraper? _scraper;
        private ScrapeProgress? _scrapeRun;
        private ScrapeChoices _scrapeChoices = new();
        private bool _developerPresent;
        private bool _scrapeClosed;
        private bool _scrapeRefreshPosted;
        private int _scrapeGeneration;
        private DispatcherTimer? _scrapeThemedRefresh;
        private IReadOnlyDictionary<string, (ScrapeState State, bool HasCover)> _scrapeOutcomes = new Dictionary<string, (ScrapeState, bool)>();
        private IReadOnlyDictionary<string, ScrapedRecord> _scrapedText = new Dictionary<string, ScrapedRecord>();
        private ScreenScraperClient? _scrapeClient;
        private ScrapeStatusWindow? _scrapeStatus;
        private MemberAccount _member = new("", "");
        private ScrapeMember? _memberChecked;
        private string? _scrapeStatusLine;

        // The status window's cancel confirm; a test answers it instead of a person.
        internal static Func<Window, Task<bool>> ConfirmCancelScrape =
            owner => Dialogs.ConfirmAsync(owner, "Cancel Scraping", "Stop this run? What it has not reached stays queued, and Resume in Preferences ▸ Scraping goes on from there.", "Cancel Scraping", "Keep Going");

        public event Action? ScrapeChanged;

        internal ScrapeStatusWindow? ScrapeStatusShown => _scrapeStatus;

        public Scraper? ScrapeWorker => _scraper;
        public MediaStore? ScrapeStore => _mediaStore;
        internal ScrapeQuotaManager? ScrapeQuota => _scrapeQuota;
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
                Language = _appSettings.ScrapeLanguage, RegionFallback = _appSettings.ScrapeRegionFallback, Threads = Math.Max(1, _appSettings.ScrapeThreads),
            };
            _developerPresent = DeveloperSource() is not null;
            _member = MemberAccount.Load();
            if (_mediaStore is null && File.Exists(Path.Combine(MediaStore.DefaultRoot, MediaStore.FileName))) OpenMediaStore();
            ReadScrapeSnapshot();
            ScrapeChanged?.Invoke();
        }

        private bool OpenMediaStore()
        {
            if (_mediaStore is not null) return true;
            try
            {
                _mediaStore = MediaStore.Open(MediaStore.DefaultRoot);
                _scrapeQuota = new ScrapeQuotaManager(_mediaStore, ScrapeClock);
                _scrapeQuota.Changed += OnScrapeQuotaChanged;
                return true;
            }
            catch (Exception ex) when (ex is IOException or InvalidDataException or UnauthorizedAccessException or Microsoft.Data.Sqlite.SqliteException)
            {
                StatusText.Text = $"The media store could not be opened: {ScrapeRedactor.Redact(ex.Message)}";
                return false;
            }
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

        // Why ScreenScraper cannot answer a run now, or null when it can.
        private string? ScreenScraperUnusable()
        {
            if (!_appSettings.Scraping) return "it is switched off";
            if (!_developerPresent) return "EmuSen's developer file is not on this computer";
            if (_scrapeQuota?.Check(out DateTimeOffset? until, out string? why) == QuotaGate.Stopped)
                return why + (until is { } u ? $", until {u.ToLocalTime():ddd HH:mm}" : "");
            return null;
        }

        private static string? SystemOf(RomEntry entry) => EmuSen.Cores.CoreCatalog.ShelfByName(entry.Shelf)?.EsdeSystem is { Length: > 0 } s ? s : null;

        // The games a scope names, from the library as last scanned.
        private List<RomEntry> ScopeGames(ScrapeScope scope)
        {
            IEnumerable<RomEntry> games = scope.Game is string one
                ? _allScan.Entries.Where(e => e.FullPath == one).DefaultIfEmpty(new RomEntry(one))
                : _allScan.Entries.Where(e => scope.Shelf is null || e.Shelf == scope.Shelf);
            games = games.Where(e => SystemOf(e) is not null && File.Exists(e.FullPath));
            if (scope.MissingArtOnly && scope.Game is null) games = games.Where(e => CoverPathFor(e) is null);
            return games.ToList();
        }

        public ScrapePlan Plan(ScrapeScope scope)
        {
            List<RomEntry> games = ScopeGames(scope);
            string? why = ScreenScraperUnusable();
            int notAsked = scope.Game is not null ? games.Count : games.Count(g => !_scrapeOutcomes.ContainsKey(g.FullPath));
            QuotaSnapshot? q = _scrapeQuota?.Snapshot();
            int? left = q?.MaxPerDay is int max ? Math.Max(0, max - q.RequestsToday) : null;
            return new ScrapePlan(games.Count, notAsked, notAsked * (1 + _scrapeChoices.Kinds().Count()), left,
                ScrapePlan.PerGame * notAsked, why is null, why, _appSettings.OpenEmuFallback);
        }

        public bool ScrapeRunning => _scrapeRun is { State: ScrapeRunState.Running };

        public ScrapeProgress? Progress => _scrapeRun;

        // Games an interrupted run left queued; offered as Resume, never resumed by itself.
        public int Interrupted => ScrapeRunning ? 0 : _mediaStore?.QueueLength ?? 0;

        // Asks the player with the count and the cost, then starts; false when they said no or there is nothing to do.
        public async Task<bool> ConfirmAndScrapeAsync(ScrapeScope scope)
        {
            if (ScrapeRunning || _scrapeClosed) return false;
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
        public bool StartScrape(ScrapeScope scope, bool resume = false)
        {
            if (ScrapeRunning || _scrapeClosed || !OpenMediaStore()) return false;
            MediaStore store = _mediaStore!;
            if (!resume)
            {
                store.ClearQueue();
                foreach (RomEntry game in ScopeGames(scope))
                {
                    if (scope.Game is not null)
                    {
                        store.ForgetUnfound(game.FullPath);
                        _records.ForgetCoverLookup(game.FullPath);
                        _fetcher?.Forget(game.FullPath);
                    }
                    store.Enqueue(game.FullPath, SystemOf(game)!, scope.Game is not null ? ScrapePriority.Shown : scope.Shelf is not null ? ScrapePriority.Console : ScrapePriority.Library);
                }
            }
            IReadOnlyList<QueuedGame> queued = store.Queue();
            if (queued.Count == 0) return false;
            QuotaSnapshot? before = _scrapeQuota?.Snapshot();
            var run = new ScrapeProgress(queued.Count, ScrapeClock.Now) { RequestsAtStart = before?.RequestsToday, DayAtStart = before?.Day };
            _scrapeRun = run;
            ReadScrapeSnapshot();

            if (ScreenScraperUnusable() is string why)
            {
                run.Why = why;
                FailoverFor(queued.Select(q => q.Path));
                if (run.FailoverPending.Count == 0) EndRun(ScrapeRunEnd.Stopped);
                ScrapeChanged?.Invoke();
                ShowScrapeStatus();
                return true;
            }

            _http ??= HttpFactory();
            _member = MemberAccount.Load();
            var client = new ScreenScraperClient(_http, DeveloperSource()!, _member);
            _scrapeClient = client;
            _scraper = new Scraper(client, store, _scrapeQuota!, () => _scrapeChoices, HasHandCover, TransformFor,
                result => Dispatcher.UIThread.Post(() => ScrapeArrived(result)), ScrapeClock) { Activity = run.Note };
            Scraper mine = _scraper;
            _scraper.Finished += end => Dispatcher.UIThread.Post(() => { if (ReferenceEquals(_scraper, mine)) ScraperFinished(end); });
            _scraper.Start();
            ScrapeLine(run.Describe());
            ScrapeChanged?.Invoke();
            ShowScrapeStatus();
            return true;
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
            if (_scrapeStatus is { } open)
            {
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
        public bool CanPauseScrape => ScrapeRunning && _scraper is not null;

        public void SetScrapePaused(bool paused)
        {
            if (!CanPauseScrape || _scrapeRun is not { } run || run.IsPaused == paused) return;
            if (paused) _scraper!.Pause();
            else _scraper!.Resume();
            run.SetPaused(paused, ScrapeClock.Now);
            ScrapeLine(run.Describe());
            ScrapeChanged?.Invoke();
        }

        public async Task<bool> ConfirmCancelAsync(Window owner)
        {
            if (!ScrapeRunning || !await ConfirmCancelScrape(owner)) return false;
            CancelScrape();
            return true;
        }

        public DateTimeOffset ScrapeNow => ScrapeClock.Now;

        public ScrapeQuota? ScrapeLimits => _scrapeQuota?.Limits;

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
            _http ??= HttpFactory();
            SignInAnswer answer;
            try { answer = await new ScreenScraperClient(_http, developer, account).SignInAsync(default); }
            catch (ObjectDisposedException) { return new SignInAnswer(SignInResult.Unreachable, null, "Mistress is closing."); }
            if (answer.Member?.Quota is { } quota && !_scrapeClosed) _scrapeQuota?.Observe(quota);
            return answer;
        }

        private void UseMember(MemberAccount account, ScrapeMember? checkedAs)
        {
            account.Save();
            _member = account;
            _memberChecked = checkedAs;
            if (_scrapeClient is { } client) client.Member = account;
            ScrapeChanged?.Invoke();
        }

        // Log Out: the file is deleted, and a run in progress sends no ssid or sspassword from its next request.
        public string SignOut()
        {
            bool had = MemberAccount.Delete();
            _member = new MemberAccount("", "");
            _memberChecked = null;
            if (_scrapeClient is { } client) client.Member = null;
            ScrapeChanged?.Invoke();
            return had
                ? "Signed out: screenscraper.json was deleted. Runs now use EmuSen's developer credentials alone."
                : "Signed out. There was no account file to delete.";
        }

        public bool ResumeScrape() => StartScrape(new ScrapeScope(), resume: true);

        public void CancelScrape()
        {
            if (!ScrapeRunning) return;
            Scraper? scraper = _scraper;
            _scraper = null;
            _scrapeClient = null;
            scraper?.Dispose();
            _fetcher?.Dispose();
            _fetcher = null;
            EndRun(ScrapeRunEnd.Cancelled);
        }

        // The failover, for games of this run ScreenScraper had no cover for or could not be asked about; never outside a run.
        private void FailoverFor(IEnumerable<string> paths)
        {
            if (_scrapeRun is not { State: ScrapeRunState.Running } run || !_appSettings.OpenEmuFallback) return;
            foreach (string path in paths)
            {
                var entry = new RomEntry(path);
                if (CoverPathFor(entry) is not null || run.FailoverPending.Contains(path)) continue;
                if (AskForCover(entry, EmuSen.Cores.CoreCatalog.ByDisplayName(entry.CoreDisplayName)))
                {
                    run.FailoverPending.Add(path);
                    run.FailoverAsked++;
                }
            }
        }

        private void ScrapeArrived(ScrapeResult result)
        {
            if (_scrapeClosed || _scrapeRun is not { } run) return;
            run.Count(result);
            if (result.Outcome is ScrapeOutcome.Found or ScrapeOutcome.Unknown or ScrapeOutcome.Error && !result.HasCover) FailoverFor([result.Path]);
            if (result.Outcome == ScrapeOutcome.Stopped) run.Why = result.Detail;
            if (run.State == ScrapeRunState.Running) ScrapeLine(run.Describe());
            PostCoverRefresh();
        }

        // What the failover said about a game of this run.
        private void FailoverArrived(CoverResult result)
        {
            if (_scrapeRun is not { } run || !run.FailoverPending.Remove(result.RomPath)) return;
            if (result.Saved is not null)
            {
                run.FailoverFound++;
                run.FilledByFailover(result.RomPath);
            }
            if (run.FailoverPending.Count == 0 && _scraper is null && run.State == ScrapeRunState.Running) EndRun(run.Why is null ? ScrapeRunEnd.Done : ScrapeRunEnd.Stopped);
            else ScrapeChanged?.Invoke();
        }

        // ScreenScraper's part is over: a stop hands what is left to the failover, and the run ends when the failover has answered too.
        private void ScraperFinished(ScrapeRunEnd end)
        {
            Scraper? scraper = _scraper;
            _scraper = null;
            _scrapeClient = null;
            scraper?.Dispose();
            if (_scrapeRun is not { State: ScrapeRunState.Running } run) return;
            run.SetPaused(false, ScrapeClock.Now);
            if (end == ScrapeRunEnd.Stopped)
            {
                string? why = null;
                _scrapeQuota?.Check(out _, out why);
                run.Why = why ?? run.Why;
                FailoverFor(_mediaStore?.Queue().Select(q => q.Path) ?? []);
            }
            if (run.FailoverPending.Count == 0) EndRun(end);
            else ScrapeChanged?.Invoke();
        }

        private void EndRun(ScrapeRunEnd end)
        {
            if (_scrapeRun is not { } run) return;
            run.SetPaused(false, ScrapeClock.Now);
            run.Ended = ScrapeClock.Now;
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

        // ScreenScraper's text for the themed view's metadata; the name stays the file's, as the library shows it.
        private SceneGame WithScrapedText(SceneGame game) => _scrapedText.TryGetValue(game.File, out ScrapedRecord? r)
            ? game with
            {
                Description = r.Description, Developer = r.Developer, Publisher = r.Publisher, Genre = r.Genre, Players = r.Players,
                ReleaseDate = r.ReleaseDate, Rating = r.Rating,
            }
            : game;

        // The game a pad's "Scrape This Game" means: the themed gamelist's selection, else the library's.
        private string? GameToScrape => ThemedLibraryShown && LibraryView.IsVisible ? _themed?.SelectedGame?.File : SelectedLibraryEntry?.FullPath;

        bool IScrapeHost.HasDeveloperCredentials => _developerPresent;

        QuotaSnapshot? IScrapeHost.Quota => _scrapeQuota?.Snapshot();

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
            Scraper? scraper = _scraper;
            _scraper = null;
            _scrapeClient = null;
            scraper?.Dispose();
            if (_scrapeQuota is not null) _scrapeQuota.Changed -= OnScrapeQuotaChanged;
            _mediaStore?.Dispose();
            _mediaStore = null;
            ScrapeChanged = null;
        }
    }
}
