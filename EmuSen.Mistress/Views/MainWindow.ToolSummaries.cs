using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Threading;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Scraping;
using EmuSen.Tools.Cores;
using EmuSen.Tools.Packages;
using EmuSen.Tools.Scraper;

namespace EmuSen.Mistress.Views
{
    // The tools' one-line states for Home, the Home card and the desktop's Home, each in its own library's words, kept as last heard - see EmuSen_Interface.md §19.
    public partial class MainWindow
    {
        private readonly Dictionary<string, ToolSummary> _toolSummaries = new();
        private DateTime _toolSummariesAsked = DateTime.MinValue;
        private CancellationTokenSource? _toolSummariesCancel;
        private static readonly TimeSpan ToolSummariesKept = TimeSpan.FromMinutes(1);

        // The reads made without a window, and where they run; a test replaces them.
        internal static Func<IReadOnlyCollection<string>, CancellationToken, IReadOnlyDictionary<string, ToolSummary?>> ToolSummariesQuery = ReadToolSummaries;

        // The reads that cost seconds on a large library, made once and again only when their data changes; the cheap ones each minute (EmuSen_Interface.md §19.3).
        private static readonly string[] CostlyReads = [HomeScreen.Scraper, HomeScreen.Packages];
        private readonly HashSet<string> _staleSummaries = new();
        internal static Func<Action, Task> ToolSummariesBackground = OnLowestThread;

        // The Library Manager reads every file to check it, so it has no read of its own here: its line comes from its last whole check.
        private static IReadOnlyDictionary<string, ToolSummary?> ReadToolSummaries(IReadOnlyCollection<string> places, CancellationToken cancel)
        {
            var read = new Dictionary<string, ToolSummary?>();
            foreach (string place in places)
                read[place] = place switch
                {
                    HomeScreen.Scraper => ScraperPane.ReadSummary(cancel),
                    HomeScreen.Packages => PackagesPane.ReadSummary(cancel),
                    HomeScreen.Cores => CoresPane.ReadSummary(CoresRuntimeOf(Runtime), cancel),
                    HomeScreen.Cheats => GameCheats.Summary(cancel),
                    _ => null,
                };
            return read;
        }

        // The places to read now: the cheap ones, and a costly one not yet said or since changed.
        private List<string> PlacesToRead() =>
            [HomeScreen.Cores, HomeScreen.Cheats, .. CostlyReads.Where(p => !_toolSummaries.ContainsKey(p) || _staleSummaries.Contains(p))];

        private static Task OnLowestThread(Action work)
        {
            var done = new TaskCompletionSource();
            var thread = new Thread(() =>
            {
                try { work(); done.SetResult(); }
                catch (Exception e) { done.SetException(e); }
            }) { IsBackground = true, Priority = ThreadPriority.Lowest, Name = "Tool summaries" };
            thread.Start();
            return done.Task;
        }

        // The tool's line as last heard; the Scraper's is the run's progress while Mistress's own run goes (EmuSen_Interface.md §19.7).
        internal ToolSummary? SummaryOf(string place) =>
            place == HomeScreen.Scraper && _scrapeRun is { State: ScrapeRunState.Running } run ? ScraperPlan.Running(run.Done, run.Total) : _toolSummaries.GetValueOrDefault(place);

        private bool _scrapeWasRunning;

        // A run's every step redraws the lines; its end makes the Scraper's count stale, read again at the next Home.
        private void SetUpToolSummaries() => ScrapeChanged += () =>
        {
            bool running = _scrapeRun is { State: ScrapeRunState.Running };
            if (_scrapeWasRunning && !running) ToolDataChanged(HomeScreen.Scraper);
            _scrapeWasRunning = running;
            ToolSummariesChanged();
        };

        // Asked once a minute at most, only while Home, a card over it or the desktop's library shows; never while a game runs (EmuSen_Interface.md §19).
        private void AskToolSummaries()
        {
            if (GameOnScreen || DateTime.UtcNow - _toolSummariesAsked < ToolSummariesKept) return;
            _toolSummariesAsked = DateTime.UtcNow;
            _toolSummariesCancel?.Cancel();
            var cancel = new CancellationTokenSource();
            _toolSummariesCancel = cancel;
            Func<IReadOnlyCollection<string>, CancellationToken, IReadOnlyDictionary<string, ToolSummary?>> query = ToolSummariesQuery;
            List<string> places = PlacesToRead();
            IReadOnlyDictionary<string, ToolSummary?>? read = null;
            Task asked = ToolSummariesBackground(() => read = query(places, cancel.Token));
            if (asked.IsCompletedSuccessfully) ToolSummariesAnswered(read!, cancel);
            else asked.ContinueWith(t => { if (t.IsCompletedSuccessfully) Dispatcher.UIThread.Post(() => ToolSummariesAnswered(read!, cancel)); }, TaskScheduler.Default);
        }

        // A game starting stops a read still running; the next Home asks again.
        private void CancelToolSummaries()
        {
            if (_toolSummariesCancel is not { } cancel) return;
            cancel.Cancel();
            _toolSummariesCancel = null;
            _toolSummariesAsked = DateTime.MinValue;
        }

        // A tool's data changed outside its window, so the next Home reads it again rather than wait out the minute.
        private void ToolDataChanged(string place)
        {
            _staleSummaries.Add(place);
            _toolSummariesAsked = DateTime.MinValue;
        }

        private void ToolSummariesAnswered(IReadOnlyDictionary<string, ToolSummary?> read, CancellationTokenSource cancel)
        {
            if (cancel.IsCancellationRequested) return;
            foreach ((string place, ToolSummary? summary) in read)
            {
                _staleSummaries.Remove(place);
                if (summary is not null) _toolSummaries[place] = summary;
            }
            ToolSummariesChanged();
        }

        // A tool window's own listing heard: what it shows is the newest the player has seen.
        private void ToolSummaryHeard(string place, ToolSummary? summary)
        {
            if (summary is null) return;
            _staleSummaries.Remove(place);
            _toolSummaries[place] = summary;
            ToolSummariesChanged();
        }

        private void ToolSummariesChanged()
        {
            if (_homeLayout is { IsVisible: true } home) home.SetPlaces(HomePlaces());
            if (!_padMenuOpen) return;
            RefreshPadMenuText(Math.Max(PadMenuList.SelectedIndex, 0));
            UpdateCardPane();
        }
    }
}
