using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Threading;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Views
{
    // A ScreenScraper run while it happens and after: progress, the current game, tallies, the quota and the recent games; it reads and never asks a server - see EmuSen_Settings_Reference.md §4.57.
    public sealed class ScrapeStatusWindow : ToolWindow
    {
        // The most often the window redraws, however fast the run's events come.
        public static readonly TimeSpan RefreshEvery = TimeSpan.FromMilliseconds(250);

        private readonly IScrapeHost _host;
        private readonly DispatcherTimer _timer;
        private bool _dirty = true;
        private bool _closed;
        private ScrapeProgress? _seenRun;
        private int _seenVersion = -1;
        private string? _pictureShown;

        private readonly TextBlock _heading = new() { Name = "ScrapeStatusHeading", FontSize = 18, FontWeight = FontWeight.SemiBold, TextWrapping = TextWrapping.Wrap };
        private readonly ProgressBar _bar = new() { Name = "ScrapeStatusProgress", Minimum = 0, Maximum = 1, HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly HintText _timing = new() { Name = "ScrapeStatusTiming" };
        private readonly Image _picture = new() { Name = "ScrapeStatusPicture", Width = 96, Height = 96, Stretch = Stretch.Uniform, VerticalAlignment = VerticalAlignment.Top };
        private readonly TextBlock _game = new() { Name = "ScrapeStatusGame", FontWeight = FontWeight.SemiBold, TextWrapping = TextWrapping.Wrap };
        private readonly HintText _console = new() { Name = "ScrapeStatusConsole" };
        private readonly TextBlock _step = new() { Name = "ScrapeStatusStep", TextWrapping = TextWrapping.Wrap };
        private readonly TextBlock _tallies = new() { Name = "ScrapeStatusTallies", TextWrapping = TextWrapping.Wrap };
        private readonly HintText _failure = new() { Name = "ScrapeStatusFailure" };
        private readonly TextBlock _member = new() { Name = "ScrapeStatusMember", TextWrapping = TextWrapping.Wrap };
        private readonly MeterRow _quota = new() { Name = "ScrapeStatusQuota", Label = "Requests today" };
        private readonly TextBlock _requests = new() { Name = "ScrapeStatusRequests", TextWrapping = TextWrapping.Wrap };
        private readonly HintText _limits = new() { Name = "ScrapeStatusLimits" };
        private readonly TextBlock _why = new() { Name = "ScrapeStatusWhy", TextWrapping = TextWrapping.Wrap, FontWeight = FontWeight.SemiBold };
        private readonly TextBlock _summary = new() { Name = "ScrapeStatusSummary", TextWrapping = TextWrapping.Wrap };
        private readonly LunaList<ScrapeRecent> _recent = new() { Name = "ScrapeStatusRecent", MinHeight = 120 };
        private readonly Button _pause;
        private readonly Button _cancel;
        private readonly Button _hide;
        private readonly Control _current;
        private readonly Control _tallySection;
        private readonly Control _recentSection;

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public ScrapeStatusWindow() : this(null!) { }

        public ScrapeStatusWindow(IScrapeHost host)
        {
            _host = host;
            Title = "Scraping";
            Width = 720;
            Height = 680;
            WindowStartupLocation = WindowStartupLocation.CenterOwner;
            ClosesOnEscape = true;

            _pause = Ui.Button("Pause", () => { _host.SetScrapePaused(!_host.ScrapePaused); Refresh(); });
            _pause.Name = "ScrapeStatusPauseButton";
            _cancel = Ui.Button("Cancel...", async () => { await _host.ConfirmCancelAsync(this); Refresh(); });
            _cancel.Name = "ScrapeStatusCancelButton";
            _hide = Ui.Button("Hide", Close);
            _hide.Name = "ScrapeStatusHideButton";
            _hide.IsDefault = true;
            _recent.Label = Describe;
            _recent.Key = r => r.Path;

            _current = Ui.Cols("Auto,*", _picture, Ui.Stack(4, _game, _console, _step).Margin(12, 0, 0, 0));
            _tallySection = Ui.Section("Results", Ui.Stack(4, _tallies, _failure));
            // A fixed height, and everything above the buttons scrolls, so a short sheet can never lay the list over its heading - see EmuSen_Settings_Reference.md §4.57.
            _recentSection = Ui.Rows("Auto,*", Ui.Header("Recent games"), _recent);
            _recentSection.Height = RecentHeight;
            var body = Ui.Stack(12,
                Ui.Stack(6, _heading, _bar, _timing),
                _current,
                _tallySection,
                Ui.Section("Quota", Ui.Stack(4, _member, _quota, _requests, _limits, _why, _summary)),
                _recentSection);
            Content = Ui.Rows("*,Auto",
                new ScrollViewer { Content = body, HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled },
                new ButtonBar { ItemsSource = new[] { _pause, _cancel, _hide }, HorizontalAlignment = HorizontalAlignment.Right }).Margin(16);
            if (Content is Grid grid) grid.RowSpacing = 12;

            _timer = new DispatcherTimer { Interval = RefreshEvery };
            _timer.Tick += OnTick;
            if (_host is null) return;
            _host.ScrapeChanged += OnChanged;
            Closed += (_, _) => Stop();
            _timer.Start();
            Refresh();
        }

        private const double RecentHeight = 200;

        // How many times the window has drawn the run; a test counts it.
        public int Refreshes { get; private set; }

        public bool Polling => _timer.IsEnabled;

        private void OnChanged() => _dirty = true;

        // At most once a tick: when the host said something changed, when the run moved, or each tick while it runs, for its clock.
        private void OnTick(object? sender, EventArgs e)
        {
            ScrapeProgress? run = _host.Progress;
            if (!_dirty && ReferenceEquals(run, _seenRun) && run?.Version == _seenVersion && run?.State != ScrapeRunState.Running) return;
            Refresh();
        }

        // Unsubscribed from the host and the timer stopped, so a closed window draws nothing more - see EmuSen_BigPicture.md §15.14.
        private void Stop()
        {
            _closed = true;
            _host.ScrapeChanged -= OnChanged;
            _timer.Stop();
            _timer.Tick -= OnTick;
        }

        public void Refresh()
        {
            if (_closed) return;
            Refreshes++;
            _dirty = false;
            ScrapeProgress? run = _host.Progress;
            DateTimeOffset now = _host.ScrapeNow;
            bool running = run is { State: ScrapeRunState.Running };

            ShowQuota(run);
            _pause.IsVisible = running;
            _pause.IsEnabled = _host.CanPauseScrape;
            _pause.Content = _host.ScrapePaused ? "Resume" : "Pause";
            _cancel.IsVisible = running;
            _hide.Content = running ? "Hide" : "Close";

            if (run is null)
            {
                _heading.Text = "No scraping in progress.";
                _bar.IsVisible = false;
                _timing.Text = "Nothing is sent from this window. A run starts from Scrape This Game... or Preferences ▸ Scraping.";
                _current.IsVisible = _tallySection.IsVisible = _recentSection.IsVisible = false;
                _why.IsVisible = _summary.IsVisible = false;
                return;
            }

            _bar.IsVisible = true;
            _bar.Value = run.Total == 0 ? 0 : (double)run.Done / run.Total;
            _heading.Text = run.State switch
            {
                ScrapeRunState.Running when run.IsPaused => $"Paused at {run.Done:N0} of {run.Total:N0} games",
                ScrapeRunState.Running => $"Scraping {run.Done:N0} of {run.Total:N0} games",
                ScrapeRunState.Done => $"Finished: {run.Done:N0} of {run.Total:N0} games",
                ScrapeRunState.Stopped => $"Stopped after {run.Done:N0} of {run.Total:N0} games",
                _ => $"Cancelled after {run.Done:N0} of {run.Total:N0} games",
            };
            TimeSpan elapsed = run.Elapsed(now);
            _timing.Text = running
                ? $"Elapsed {Clock(elapsed)} · " + (run.EstimateLeft(now) is { } left ? $"about {Clock(left)} left at this run's pace" : "the time left is estimated after the first game ScreenScraper answers")
                  + (run.IsPaused ? " · paused: nothing new is asked until you resume" : "")
                : $"Took {Clock(elapsed)}";

            _current.IsVisible = running;
            if (running) ShowCurrent(run);

            _tallySection.IsVisible = true;
            _tallies.Text = $"Found {run.Found:N0} · Not found {run.Unknown:N0} · Failed {run.Failed:N0} · Skipped {run.Skipped:N0} · Filled by OpenEmu {run.FailoverFound:N0}"
                + (run.Retrying > 0 ? $" · {run.Retrying:N0} to retry later" : "");
            _failure.IsVisible = run.LastFailure is not null;
            _failure.Text = ScrapeRedactor.Redact(run.LastFailure is { } f ? $"Last failure: {f}" : "");

            _why.IsVisible = run.State is ScrapeRunState.Stopped or ScrapeRunState.Cancelled;
            _why.Text = ScrapeRedactor.Redact(run.State == ScrapeRunState.Cancelled ? "Why it stopped: you cancelled it." : $"Why it stopped: {run.Why ?? "the quota"}.");
            _summary.IsVisible = !running;
            if (!running) _summary.Text = ScrapeRedactor.Redact(Summary(run));

            _recentSection.IsVisible = true;
            if (!ReferenceEquals(run, _seenRun) || run.Version != _seenVersion) _recent.Refresh(run.Recent);
            _seenRun = run;
            _seenVersion = run.Version;
        }

        private void ShowCurrent(ScrapeProgress run)
        {
            ScrapeActivity? now = run.Current;
            _game.Text = now is null ? "" : ScrapeRedactor.Redact(Path.GetFileNameWithoutExtension(now.Path));
            _console.Text = now is null ? "" : ConsoleOf(now.System);
            _step.Text = run.IsPaused ? "Paused: a request already sent finishes; nothing new is asked." : now?.Step switch
            {
                ScrapeStep.LookingUp => "Looking it up",
                ScrapeStep.Downloading => $"Downloading its {KindLabel(now.Kind)}",
                ScrapeStep.Arrived => $"Its {KindLabel(now.Kind)} arrived",
                _ => _host.CanPauseScrape ? "Waiting for the next game" : "Asking OpenEmu's sources for covers",
            };
            string? picture = run.LastPicture;
            if (picture == _pictureShown) return;
            _pictureShown = picture;
            _picture.Source = Thumbnail(picture);
        }

        // Decoded small, once per picture that arrives.
        private static Bitmap? Thumbnail(string? path)
        {
            if (path is null || !File.Exists(path)) return null;
            try
            {
                using FileStream stream = File.OpenRead(path);
                return Bitmap.DecodeToWidth(stream, 192);
            }
            catch (Exception)
            {
                return null;
            }
        }

        private void ShowQuota(ScrapeProgress? run)
        {
            MemberAccount member = _host.Member;
            string name = member.IsSet ? member.User : "";
            // The member's own name is shown to them; every other text passes the redactor.
            _member.Text = member.IsSet
                ? $"Member: {name}" + (member.Verified is null && _host.MemberChecked is null ? " (not checked; Check in Preferences ▸ Scraping)" : "")
                : "No member account: EmuSen's developer credentials alone, with their limits.";

            QuotaSnapshot? q = _host.Quota;
            _quota.IsVisible = q?.MaxPerDay is > 0;
            _requests.IsVisible = q is not null;
            if (q is not null)
            {
                _quota.Percent = q.MaxPerDay is int max && max > 0 ? Math.Min(100, 100.0 * q.RequestsToday / max) : 0;
                _quota.ValueText = $"{_quota.Percent:0}%";
                _requests.Text = q.MaxPerDay is int m
                    ? $"{q.RequestsToday:N0} of {m:N0} requests used today · {Math.Max(0, m - q.RequestsToday):N0} left"
                    : $"{q.RequestsToday:N0} requests used today; the day's limit is not known yet";
            }
            ScrapeQuota? limits = _host.ScrapeLimits;
            _limits.Text = limits is null
                ? "The limits are read from ScreenScraper's first answer: until then one thread, 30 requests a minute."
                : $"{Plural(limits.MaxThreads ?? 1, "thread")} · download limit {(limits.MaxDownloadKBps is int kb ? $"{kb:N0} KB/s" : "not given")}"
                  + (limits.MaxRequestsPerDay is int day ? $" · {day:N0} requests a day" : "")
                  + (limits.MaxRequestsPerMinute is int minute ? $" · {minute:N0} a minute" : "");
        }

        private string Summary(ScrapeProgress run)
        {
            string text = $"{run.Found:N0} found, {run.Unknown:N0} not found, {run.Failed:N0} failed, {run.Skipped:N0} skipped, {run.FailoverFound:N0} covers from OpenEmu's sources.";
            text += $" {Plural(run.Requests, "request")} sent to ScreenScraper";
            text += _host.Quota is { MaxPerDay: int max } q ? $"; it counts {q.RequestsToday:N0} of {max:N0} today." : ".";
            int left = _host.Interrupted;
            if (left > 0) text += $" {Plural(left, "game")} left queued: Resume, in Preferences ▸ Scraping, goes on with them. Nothing resumes by itself.";
            return text;
        }

        // One row of the recent list: the game, its outcome, and the pictures that arrived.
        public static string Describe(ScrapeRecent r)
        {
            string name = Path.GetFileNameWithoutExtension(r.Path);
            string outcome = r switch
            {
                { Skipped: true } => $"skipped, {r.Detail}",
                { Outcome: ScrapeOutcome.Found } => "found",
                { Outcome: ScrapeOutcome.Unknown } => "not found",
                _ => $"failed: {r.Detail}",
            };
            string pictures = r.Kinds.Count > 0 ? " · " + string.Join(", ", r.Kinds.Select(KindLabel)) : r.Outcome == ScrapeOutcome.Found && !r.Skipped ? " · no pictures" : "";
            string failover = r.FromFailover ? " · cover from OpenEmu's sources" : "";
            return ScrapeRedactor.Redact($"{name} ({ConsoleOf(r.System)}): {outcome}{pictures}{failover}");
        }

        public static string KindLabel(string? kind) => kind switch
        {
            "cover" => "cover",
            "screenshot" => "screenshot",
            "marquee" => "marquee",
            "titlescreen" => "title screen",
            "miximage" => "mix image",
            _ => kind ?? "picture",
        };

        private static string ConsoleOf(string system) =>
            EmuSen.Cores.CoreCatalog.ShelvesInReleaseOrder.FirstOrDefault(s => s.EsdeSystem == system)?.Name ?? system;

        private static string Plural(int n, string what) => n == 1 ? $"1 {what}" : $"{n:N0} {what}s";

        private static string Clock(TimeSpan t) => t.TotalHours >= 1 ? $"{(int)t.TotalHours}:{t.Minutes:00}:{t.Seconds:00}" : $"{t.Minutes}:{t.Seconds:00}";
    }
}
