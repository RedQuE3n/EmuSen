using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Threading;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;

namespace EmuSen.Tools.Scraper
{
    // The Scraper's content, declared once on LunaP's manager frame: the games `scrape ls` chooses, marked, planned and run through the emusen.scrape module - see EmuSen_LibraryTools.md §2 and §9.
    public sealed class ScraperPane : ManagerLayout<ScrapeRow>
    {
        // How the command is asked and how a listing leaves the interface thread, which a test replaces.
        internal static Func<string, string> Ask = ScrapeCommand.Ask;
        internal static Func<bool> CommandAvailable = () => ScrapeCommand.Available;
        internal static Func<Func<string>, Task<string>> Background = work => Task.Run(work);

        public static readonly TimeSpan PollEvery = TimeSpan.FromMilliseconds(250);

        private readonly DispatcherTimer _timer;
        private readonly TextBlock _overview = new() { Name = "ScraperOverview", TextWrapping = Avalonia.Media.TextWrapping.Wrap };
        private readonly Dictionary<string, string> _outcomes = new(StringComparer.Ordinal);
        private readonly LunaAction _markAll, _theme, _videos, _overwrite, _stop;
        private readonly List<AdaptiveSection> _systems = new();
        private List<AdaptiveSection> _collections = new();
        private string? _notice, _endSaid;
        private long _since = -1;
        private bool _following, _listing;
        private int _listed;

        public ScraperPane()
        {
            Title = "Scraper";
            HasSearch = true;
            // The detail, a game's media as pictures, stands beside the list (EmuSen_Interface.md §15.1).
            DetailPlacement = AdaptiveDetailPlacement.Beside;
            ContentShare = 0.6;
            Header = new AdaptiveHeader("SCRAPER");
            SectionWord = "System";
            SearchHint = "Search games";
            RowKey = r => r.Path;
            Column(new LunaColumn<ScrapeRow>("Game", r => r.Title) { Sort = (a, b) => string.Compare(a.Title, b.Title, StringComparison.CurrentCultureIgnoreCase) }, bigScreen: true)
                .Column(new LunaColumn<ScrapeRow>("System", r => r.System) { Width = "Auto" })
                .Column(new LunaColumn<ScrapeRow>("Media", ScraperPlan.Media) { Width = "Auto", Sort = (a, b) => a.Have.CompareTo(b.Have) }, bigScreen: true)
                .Column(new LunaColumn<ScrapeRow>("Missing", ScraperPlan.Missing))
                .Column(new LunaColumn<ScrapeRow>("Last Run", r => OutcomeOf(r) ?? ""));
            MarksFor = ScraperPlan.MarksFor;
            SearchKey = r => $"{r.Title} {r.Path}";
            DetailFor = r => r is null ? Overview() : Detail(r);
            Summarize = ScraperPlan.Summary;
            Plan = PlanText;
            Applying += Run;

            _markAll = new LunaAction("Mark All", () => MarkWhere(_ => true)) { HelpText = "Marks every game shown to be scraped; in Missing Media, every game lacking a kind asked." };
            _theme = new LunaAction("Theme", _ => { }) { IsCheckable = true, HelpText = "Asks exactly the kinds the active big picture theme draws for each system." };
            _videos = new LunaAction("Videos", _ => { }) { IsCheckable = true, HelpText = "Asks for videos too; they are off by default." };
            _overwrite = new LunaAction("Overwrite", _ => { }) { IsCheckable = true, HelpText = "Fetches again and replaces what is there (ES-DE's Overwrite files and data)." };
            _stop = new LunaAction("Stop", StopRun) { HelpText = "Stops the run; the games not reached are not asked." };
            // The options are rows of the plan's question and the desktop toolbar's toggles; a change relists, since what is missing depends on the kinds asked.
            WatchOption(_theme, relist: true);
            WatchOption(_videos, relist: true);
            WatchOption(_overwrite, relist: false);
            SetToolActions([_markAll, LunaAction.Separator(), _theme, _videos, _overwrite, LunaAction.Separator(), _stop], bigScreenButtons: []);
            StopAction = _stop;
            SetListActions([_markAll]);
            SetPlanOptions([_theme, _videos, _overwrite]);
            MarksChanged += UpdateActions;
            SectionChosen += section => { _endSaid = null; _ = Refresh(); };
            FilterChosen += filter => { _endSaid = null; _ = Refresh(); };

            _timer = new DispatcherTimer { Interval = PollEvery };
            _timer.Tick += (_, _) => Poll();
            DetachedFromVisualTree += (_, _) => _timer.Stop();
            FillSections();
            UpdateActions();
        }

        public ScrapeOptions Options => new(_theme.IsChecked, _videos.IsChecked, _overwrite.IsChecked);

        // The last refusal or failure the command gave, in its words; null when there was none.
        public string? Notice => _notice;

        // Whether a run is being followed, this window's or one started elsewhere.
        public bool Following => _following;

        // Raised on the interface thread when a run this window followed has ended, so that a host can show what it fetched.
        public event Action? RunEnded;

        public string? OutcomeOf(ScrapeRow row) => _outcomes.TryGetValue(row.Path, out string? said) ? said : null;

        private void Say(string? words)
        {
            _notice = words;
            SetNotice(words);
        }

        // The command's answer, or null with its refusal said in the apply bar.
        private string? Asked(string request)
        {
            try { return Ask(request); }
            catch (ScrapeCommandException e)
            {
                Say(Refusal(e));
                return null;
            }
        }

        // A refusal in the player's words: a held store and a run going are said plainly.
        public static string Refusal(ScrapeCommandException e) => e.Status switch
        {
            ScrapeCommand.StoreInUse => ScraperPlan.Sentence(e.Message),
            -256 => $"Scraping here is done by EmuSen's platform library, which is not in use ({ScrapeCommand.Report}).",
            _ => ScraperPlan.Sentence(e.Message),
        };

        // Lists the chosen section's games through `scrape ls`, off the interface thread; then follows a run that is going.
        public async Task Refresh()
        {
            if (_listing) return;
            if (!CommandAvailable())
            {
                Say($"Scraping here is done by EmuSen's platform library, which is not in use ({ScrapeCommand.Report}).");
                SetRows([]);
                return;
            }
            _listing = true;
            int listing = ++_listed;
            UpdateActions();
            if (!_following) SetProgress(null, "Choosing the games…");
            string request = ScrapeCommand.Request(ScraperPlan.Line("ls", SelectedSection, SelectedFilter, Options));
            string? answer = null;
            ScrapeCommandException? refused = null;
            try { answer = await Background(() => Ask(request)); }
            catch (ScrapeCommandException e) { refused = e; }
            _listing = false;
            if (listing != _listed) return;
            if (!_following) Finish(clearMarks: false);
            if (refused is not null || answer is null)
            {
                Say(refused is null ? null : Refusal(refused));
                SetRows([]);
            }
            else
            {
                (List<ScrapeRow> rows, List<string> notes) = ScrapeRow.Listing(answer);
                Say(_endSaid ?? (notes.Count > 0 ? string.Join(" ", notes.Select(ScraperPlan.Sentence)) : null));
                SetRows(rows);
                int lacking = rows.Count(r => r.Media.Count > 0 && r.Missing.Count > 0);
                Header = new AdaptiveHeader("SCRAPER", Facts: $"{rows.Count} games · {lacking} missing media");
                // The systems are learnt from the whole library only: a section or a filter lists part of it.
                if (SelectedSection is null or "all" && SelectedFilter is null or "all") LearnSystems(rows);
            }
            LearnCollections();
            ShowOverview();
            UpdateActions();
            if (!_following) Follow();
        }

        private void LearnSystems(IEnumerable<ScrapeRow> rows)
        {
            var systems = rows.Select(r => r.System).Distinct().Order(StringComparer.OrdinalIgnoreCase).Select(s => new AdaptiveSection("system:" + s, s, "Systems")).ToList();
            if (systems.Select(s => s.Key).SequenceEqual(_systems.Select(s => s.Key))) return;
            _systems.Clear();
            _systems.AddRange(systems);
            FillSections();
        }

        private void LearnCollections()
        {
            if (_collections.Count > 0 || Asked(ScrapeCommand.Request(["collections"])) is not { } answer) return;
            using JsonDocument doc = JsonDocument.Parse(answer);
            if (doc.RootElement.ValueKind != JsonValueKind.Array) return;
            _collections = doc.RootElement.EnumerateArray().Select(c => ScrapeRow.Text(c, "name")).OfType<string>().Where(n => n.Length > 0)
                .Select(n => new AdaptiveSection("collection:" + n, n, "Collections")).ToList();
            if (_collections.Count > 0) FillSections();
        }

        // All, the criteria of ES-DE's "Scrape these games", each system, and each collection: one section is one choice of the module's.
        private void FillSections()
        {
            var sections = new List<AdaptiveSection> { new("all", "All", "Systems") };
            sections.AddRange(_systems);
            string selected = SelectedSection is { } s && sections.Any(x => x.Key == s) ? s : "all";
            SetSections(sections, selected);
            // The criteria and the collections choose among a system's games, so they are the list's filters, not sections.
            var filters = new List<AdaptiveSection>
            {
                new("all", "All Games"), new("favorites", "Favourites"), new("missing", "Missing Media"), new("noimage", "No Game Image"), new("nometadata", "No Description"),
            };
            filters.AddRange(_collections.Select(c => new AdaptiveSection(c.Key, c.Label)));
            string filter = SelectedFilter is { } f && filters.Any(x => x.Key == f) ? f : "all";
            SetFilters(filters, filter);
        }

        private void OptionsChanged(bool relist = true)
        {
            ShowOverview();
            if (relist) _ = Refresh();
        }

        private void WatchOption(LunaAction option, bool relist)
        {
            bool was = option.IsChecked;
            option.Changed += a =>
            {
                if (a.IsChecked == was) return;
                was = a.IsChecked;
                OptionsChanged(relist);
            };
        }

        private void MarkWhere(Func<ScrapeRow, bool> which)
        {
            if (IsRunning) return;
            foreach (ScrapeRow row in Shown.Where(which)) SetMark(row, ScraperPlan.Scrape.Key);
        }

        private void UpdateActions()
        {
            foreach (LunaAction a in new[] { _markAll, _theme, _videos, _overwrite }) a.IsEnabled = !IsRunning && !_following;
            _stop.IsEnabled = _following;
        }

        // The plan sheet's words: the scraper's ScrapePlan for the marked games, before anything is sent.
        private string PlanText(IReadOnlyList<ManagerMarked<ScrapeRow>> marked)
        {
            ScrapeOptions o = Options;
            string request = ScrapeCommand.Request(ScraperPlan.Line("plan", null, o, byHand: true), marked.Select(m => m.Row.Path));
            if (Asked(request) is not { } answer) return _notice ?? "The plan could not be made.";
            using JsonDocument doc = JsonDocument.Parse(answer);
            JsonElement e = doc.RootElement;
            var kinds = e.TryGetProperty("kinds", out JsonElement k) && k.ValueKind == JsonValueKind.Array ? k.EnumerateArray().Select(x => x.GetString() ?? "").ToList() : [];
            return ScraperPlan.Text(ScrapeRow.Text(e, "said") ?? "", kinds, o, ScrapeRow.Notes(e));
        }

        // A confirmed plan: one run of the marked games through `scrape run`, followed to its end.
        private void Run(IReadOnlyList<ManagerMarked<ScrapeRow>> marked)
        {
            _endSaid = null;
            Say(null);
            string request = ScrapeCommand.Request(ScraperPlan.Line("run", null, Options, byHand: true), marked.Select(m => m.Row.Path));
            if (Asked(request) is not { } answer)
            {
                Finish(clearMarks: false);
                UpdateActions();
                return;
            }
            using JsonDocument doc = JsonDocument.Parse(answer);
            _since = doc.RootElement.TryGetProperty("since", out JsonElement s) && s.TryGetInt64(out long since) ? since : -1;
            foreach (ManagerMarked<ScrapeRow> m in marked) _outcomes.Remove(m.Row.Path);
            SetProgress(0, "Scraping: starting…");
            _following = true;
            _timer.Start();
            UpdateActions();
        }

        // Follows a run going when the window opens, whoever started it.
        private void Follow()
        {
            if (Asked(ScrapeCommand.Request(["progress"], since: long.MaxValue)) is not { } answer) return;
            if (!ScraperPlan.Progress(answer).Running) return;
            using (JsonDocument doc = JsonDocument.Parse(answer))
                _since = doc.RootElement.GetProperty("progress").TryGetProperty("serial", out JsonElement s) && s.TryGetInt64(out long serial) ? serial : -1;
            _following = true;
            Take(answer);
            _timer.Start();
            UpdateActions();
        }

        // The run's progress read and shown; a test calls it in place of the timer.
        public void Poll()
        {
            if (!_following) return;
            if (Asked(ScrapeCommand.Request(["progress"], since: Math.Max(0, _since))) is not { } answer)
            {
                End(null);
                return;
            }
            Take(answer);
        }

        private void Take(string answer)
        {
            foreach ((string path, string outcome, long serial) in ScraperPlan.Outcomes(answer))
            {
                _outcomes[path] = outcome;
                _since = Math.Max(_since, serial);
            }
            (double? fraction, string line, bool running, string? said) = ScraperPlan.Progress(answer);
            if (running) SetProgress(fraction, line);
            else End(said);
        }

        private void End(string? said)
        {
            _timer.Stop();
            _following = false;
            Finish();
            _endSaid = said;
            if (said is not null) Say(said);
            UpdateActions();
            RunEnded?.Invoke();
            _ = Refresh();
        }

        private void StopRun()
        {
            if (!_following) return;
            Asked(ScrapeCommand.Request(["cancel"]));
            SetProgress(null, "Stopping: the games not reached are not asked…");
        }

        private Control Detail(ScrapeRow row)
        {
            var panel = new StackPanel { Spacing = 6 };
            panel.Children.Add(Line(row.Title, bold: true));
            panel.Children.Add(Line(row.Path));
            panel.Children.Add(Line($"System: {row.System}"));
            if (row.Package is { Length: > 0 } package) panel.Children.Add(Line($"Package: {package}"));
            foreach ((string kind, string? source) in row.Media) panel.Children.Add(Line($"{kind}: {ScraperPlan.Source(source)}"));
            if (OutcomeOf(row) is { } outcome) panel.Children.Add(Line($"Last run: {outcome}"));
            return new ScrollViewer { Content = panel, HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled };
        }

        private TextBlock Line(string text, bool bold = false) =>
            new() { Text = text, TextWrapping = Avalonia.Media.TextWrapping.Wrap, FontWeight = bold && Form == AdaptiveForm.Desktop ? Avalonia.Media.FontWeight.SemiBold : Avalonia.Media.FontWeight.Normal, Classes = { bold ? AdaptiveLayout.PaneTitleClass : "line" } };

        // What the detail shows with no game chosen: the games, the options, and the account's limits as the scraper holds them.
        public string OverviewText
        {
            get
            {
                ScrapeOptions o = Options;
                var lines = new List<string>
                {
                    $"{Rows.Count} games chosen, {Rows.Count(r => r.Missing.Count > 0)} of them missing one of the kinds asked.",
                    $"Kinds: {(o.Theme ? "what the active theme draws" : "the scraper's default")}; videos {(o.Videos ? "on" : "off")}; {(o.Overwrite ? "what is there is replaced" : "what is there is kept")}.",
                };
                if (CommandAvailable() && Status() is { } status) lines.Add(status);
                return string.Join("\n", lines);
            }
        }

        // The account's limits: threads, per minute and today's requests left; never a name or password.
        private string? Status()
        {
            string? answer;
            try { answer = Ask(ScrapeCommand.Request(["status"])); }
            catch (ScrapeCommandException) { return null; }
            using JsonDocument doc = JsonDocument.Parse(answer);
            JsonElement q = doc.RootElement.GetProperty("quota");
            JsonElement a = doc.RootElement.GetProperty("account");
            string left = Number(q, "max_per_day") is int m && Number(q, "requests") is int u
                ? $"{Math.Max(0, m - u)} of {m} requests left today" : "today's requests left not known until the first answer";
            string minute = Number(q, "per_minute") is int per ? $", {per} a minute" : "";
            string threads = Number(q, "threads") is int n ? $"{n} thread{(n == 1 ? "" : "s")}" : "1 thread";
            bool signedIn = a.TryGetProperty("signed_in", out JsonElement si) && si.ValueKind == JsonValueKind.True;
            string why = ScrapeRow.Text(q, "gate") == "stopped" && ScrapeRow.Text(q, "reason") is { } reason ? $" ScreenScraper is not asked now: {reason}." : "";
            return $"ScreenScraper: {threads}{minute}, {left}; {(signedIn ? "a member is signed in" : "no member is signed in")}.{why}";
        }

        private static int? Number(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.Number && v.TryGetInt32(out int n) ? n : null;

        private Control Overview()
        {
            _overview.Text = OverviewText;
            return _overview;
        }

        private void ShowOverview() => _overview.Text = OverviewText;
    }
}
