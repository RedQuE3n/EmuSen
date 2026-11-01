using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Threading;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;

namespace EmuSen.Tools.Packages
{
    // Packages' content, declared once on LunaP's manager frame: the library's games packaged and not, filters, marks and Apply, all through the library tool's jobs; the library is only read - see EmuSen_Packages.md §3 and §7.
    public sealed class PackagesPane : ManagerLayout<PackageRow>
    {
        // How a job is started, whether the tool can run, and the free and total bytes of a disk, which a test replaces.
        internal static Func<string, IToolJob> StartJob = LibraryToolJob.Start;
        internal static Func<bool> ToolAvailable = () => LibraryToolJob.Available;
        internal static Func<string, (long Free, long Total)?> Disk = DiskOf;

        public static readonly TimeSpan PollEvery = TimeSpan.FromMilliseconds(100);

        public const string ListingRequest = """{"op":"pkg-ls","paths":[],"unpackaged":true}""";

        private readonly DispatcherTimer _timer;
        private readonly TextBlock _overview = new() { Name = "PackagesOverview", TextWrapping = Avalonia.Media.TextWrapping.Wrap };
        private readonly Dictionary<string, string> _outcomes = new();
        private readonly LunaAction _make, _check, _zip, _convert, _refresh;
        private IToolJob? _job;
        private List<PackageRow> _listing = new();
        private Queue<PlanStep> _steps = new();
        private PlanStep? _step;
        private int _stepCount, _stepsDone;
        private string? _notice;

        public PackagesPane()
        {
            Title = "Packages";
            HasSearch = true;
            SearchHint = "Search games";
            RowKey = r => r.Key;
            // The widest two are capped until the frame's table holds a star column to its share (§7.4).
            Column(new LunaColumn<PackageRow>("Game", r => r.Title) { MaxWidth = 400, Sort = (a, b) => string.Compare(a.Title, b.Title, StringComparison.CurrentCultureIgnoreCase) }, bigScreen: true)
                .Column(new LunaColumn<PackageRow>("Console", r => r.Console) { Width = "Auto" })
                .Column(new LunaColumn<PackageRow>("Format", r => r.Format) { Width = "Auto" })
                .Column(new LunaColumn<PackageRow>("Mode", PackagesWords.Mode) { Width = "Auto" })
                .Column(new LunaColumn<PackageRow>("State", StateText) { MaxWidth = 240 }, bigScreen: true)
                .Column(new LunaColumn<PackageRow>("Size", r => r.Size > 0 ? ToolWords.Size(r.Size) : "") { Width = "Auto", Sort = (a, b) => a.Size.CompareTo(b.Size) });
            MarksFor = PackagesPlan.MarksFor;
            InSection = Belongs;
            SearchKey = r => $"{r.Title} {r.Source} {r.Package}";
            DetailFor = r => r is null ? Overview() : Scrolled(new PackageDetail(r, OutcomeOf(r)));
            Summarize = PackagesPlan.Summary;
            Plan = PackagesPlan.Text;
            Applying += Run;

            _make = new LunaAction("Make", () => MarkSelected(PackagesPlan.Make.Key)) { HelpText = "Marks the chosen game to be made into a linked package." };
            _check = new LunaAction("Check", () => MarkSelected(PackagesPlan.Check.Key)) { HelpText = "Marks the chosen package to be checked against its manifest." };
            _zip = new LunaAction("Zip", () => MarkSelected(PackagesPlan.Zip.Key, PackagesPlan.MakeZip.Key)) { HelpText = "Marks the chosen package to be written as one zip." };
            _convert = new LunaAction("Convert to CHD", () => MarkSelected(PackagesPlan.Convert.Key)) { HelpText = "Marks the chosen disc to be converted to CHD." };
            _refresh = new LunaAction("Refresh", Refresh) { PadButton = PadGlyphButton.Select, HelpText = "Lists the library and its packages again." };
            SetToolActions([_make, _check, _zip, _convert, LunaAction.Separator(), _refresh]);
            MarksChanged += UpdateActions;

            _timer = new DispatcherTimer { Interval = PollEvery };
            _timer.Tick += (_, _) => Poll();
            DetachedFromVisualTree += (_, _) => Stop();
            SetSections([new AdaptiveSection("all", "All", "Consoles")], "all");
            UpdateActions();
        }

        // Whether a job is running: the listing, or a step of a confirmed plan.
        public bool Busy => _job is not null;

        // The last refusal or failure the tool gave in a player's words, null when there was none.
        public string? Notice => _notice;

        // What the last run did to a row, by its key or its game's path.
        public string? OutcomeOf(PackageRow row) =>
            _outcomes.TryGetValue(row.Key, out string? said) ? said : row.Source is { } s && _outcomes.TryGetValue("library:" + s, out said) ? said : null;

        private string StateText(PackageRow row) => OutcomeOf(row) is { } said ? said : PackagesWords.State(row);

        private static bool Belongs(PackageRow row, string section) => section switch
        {
            "all" => true,
            "packaged" => row.Packaged,
            "unpackaged" => !row.Packaged,
            "problems" => row.Problem,
            _ => section.StartsWith("console:", StringComparison.Ordinal) && row.Console == section["console:".Length..],
        };

        // Lists the library and its packages, through `pkg ls --all`.
        public void Refresh()
        {
            if (_job is not null) return;
            if (!ToolAvailable())
            {
                _notice = $"Packages are listed and made by EmuSen's platform library, which is not in use here ({LibraryToolJob.Report}).";
                SetRows([]);
                ShowOverview();
                return;
            }
            _listing = new List<PackageRow>();
            if (Start(ListingRequest)) SetProgress(null, "Listing the library and its packages…");
            else ShowOverview();
        }

        private bool Start(string request)
        {
            try { _job = StartJob(request); }
            catch (Exception e) when (e is FormatException or InvalidOperationException)
            {
                _notice = ToolWords.Refusal(e.Message);
                return false;
            }
            _timer.Start();
            return true;
        }

        private void Stop()
        {
            _timer.Stop();
            _job?.Dispose();
            _job = null;
        }

        private void MarkSelected(params string[] keys)
        {
            if (Selected is not { } row) return;
            foreach (string key in keys)
                if (SetMark(row, key)) return;
        }

        // The marking actions stay enabled whichever row is chosen, so the pad's walk through them never changes with the row, and one the row is not offered marks nothing (§7.3).
        private void UpdateActions()
        {
            foreach (LunaAction a in new[] { _make, _check, _zip, _convert }) a.IsEnabled = !IsRunning;
            _refresh.IsEnabled = !IsRunning && _job is null;
        }

        // The steps of a confirmed plan, run one job at a time in order.
        private void Run(IReadOnlyList<ManagerMarked<PackageRow>> marked)
        {
            Stop();
            _outcomes.Clear();
            _notice = null;
            _steps = new Queue<PlanStep>(PackagesPlan.Steps(marked));
            _stepCount = _steps.Count;
            _stepsDone = 0;
            UpdateActions();
            NextStep();
        }

        private void NextStep()
        {
            while (_steps.TryDequeue(out PlanStep? step))
            {
                _step = step;
                if (Start(step.Request))
                {
                    SetProgress((double)_stepsDone / Math.Max(1, _stepCount), $"{step.Doing}…");
                    return;
                }
                foreach (PackageRow row in step.Rows) _outcomes[row.Key] = _notice!;
                _stepsDone++;
            }
            _step = null;
            Finish();
            Refresh();
        }

        // Every event the job has queued taken and shown; a test calls it in place of the timer.
        public void Poll()
        {
            if (_job is null) return;
            for (int i = 0; i < 256; i++)
            {
                string? line;
                try { line = _job.Next(0); }
                catch (InvalidOperationException e)
                {
                    _notice = ToolWords.Refusal(e.Message);
                    break;
                }
                if (line is null) break;
                Take(line);
            }
            if (_job is not { Ended: true }) return;
            Stop();
            if (_step is null)
            {
                Finish(clearMarks: false);
                SetRows(_listing);
                FillSections();
                ShowSpace();
                ShowOverview();
                UpdateActions();
                return;
            }
            _stepsDone++;
            NextStep();
        }

        private static string? Text(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;

        // One event: a row of the listing, or a step's progress and outcome.
        private void Take(string line)
        {
            using JsonDocument doc = JsonDocument.Parse(line);
            JsonElement e = doc.RootElement;
            string? kind = Text(e, "event");
            if (kind == "end" && Text(e, "error") is { } error)
            {
                _notice = ToolWords.Refusal(error);
                foreach (PackageRow row in _step?.Rows ?? []) _outcomes.TryAdd(row.Key, _notice);
            }
            if (kind == "start" && e.TryGetProperty("refused", out JsonElement refused) && refused.ValueKind == JsonValueKind.Array && refused.GetArrayLength() > 0)
                _notice = string.Join(" ", refused.EnumerateArray().Select(r => ToolWords.Refusal(r.GetString() ?? "")));
            if (_step is null)
            {
                if (PackageRow.Of(e) is { } row) _listing.Add(row);
                return;
            }
            if (PackagesWords.Outcome(e) is { } said)
            {
                if (Text(e, "source") is { } source) _outcomes["library:" + source] = said;
                if (Text(e, "package") is { } package) _outcomes["package:" + package] = said;
            }
            if (e.TryGetProperty("index", out JsonElement index) && e.TryGetProperty("of", out JsonElement of) && index.TryGetInt32(out int n) && of.TryGetInt32(out int total) && total > 0)
            {
                double within = (Text(e, "state") == "working" ? n - 1 : n) / (double)total;
                string what = Path.GetFileNameWithoutExtension(Text(e, "source") ?? Text(e, "package") ?? "");
                SetProgress((_stepsDone + within) / Math.Max(1, _stepCount), $"{_step.Doing}: {n} of {total}, {what}");
            }
        }

        /// <inheritdoc/>
        protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
        {
            base.OnPropertyChanged(change);
            if (change.Property == FormProperty && Rows.Count > 0) FillSections();
        }

        private void FillSections()
        {
            var sections = new List<AdaptiveSection> { new("all", "All", "Consoles") };
            // The big screen's section row holds the states alone until it scrolls a longer row (§7.4).
            if (Form == AdaptiveForm.Desktop) sections.AddRange(Rows.Select(r => r.Console).Where(c => c.Length > 0).Distinct().Order(StringComparer.CurrentCultureIgnoreCase).Select(c => new AdaptiveSection("console:" + c, c, "Consoles")));
            sections.AddRange([new AdaptiveSection("packaged", "Packaged", "State"), new AdaptiveSection("unpackaged", "Not yet", "State"), new AdaptiveSection("problems", "Problems", "State")]);
            string selected = SelectedSection is { } s && sections.Any(x => x.Key == s) ? s : "all";
            SetSections(sections, selected);
        }

        // The packages' total and the free space of the disk they are on (§3); the meter says how full the disk is until the frame's space line takes longer words (§7.4).
        private void ShowSpace()
        {
            if (Disk(DataStore.Packages) is not { Total: > 0 } disk) return;
            double used = 1 - (double)disk.Free / disk.Total;
            SetSpace((used * 100).ToString("0", System.Globalization.CultureInfo.InvariantCulture) + "%", used);
        }

        // What the detail shows with no game chosen: the tool's last refusal, else the library's and the packages' totals.
        public string OverviewText
        {
            get
            {
                if (_notice is not null) return _notice;
                long total = Rows.Sum(r => r.PackageSize);
                int packaged = Rows.Count(r => r.Packaged);
                string free = Disk(DataStore.Packages) is { Total: > 0 } disk ? $", on a disk with {ToolWords.Size(disk.Free)} free" : "";
                return $"{Rows.Count} games, {packaged} of them packaged. The packages' folders hold {ToolWords.Size(total)}{free}.";
            }
        }

        // A detail taller than its pane scrolls; in the big screen's look its edges fade as the menus' do.
        private static ScrollViewer Scrolled(Control content) =>
            new() { Content = content, HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled };

        private Control Overview()
        {
            _overview.Text = OverviewText;
            return _overview;
        }

        // The overview's words kept current, as a listing or a refusal changes them.
        private void ShowOverview() => _overview.Text = OverviewText;

        // The disk under the nearest folder of the path that exists; null where none can be read.
        private static (long Free, long Total)? DiskOf(string path)
        {
            try
            {
                string? at = Path.GetFullPath(path);
                while (at is not null && !Directory.Exists(at)) at = Path.GetDirectoryName(at);
                if (at is null) return null;
                var drive = new DriveInfo(at);
                return (drive.AvailableFreeSpace, drive.TotalSize);
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException or ArgumentException) { return null; }
        }
    }
}
