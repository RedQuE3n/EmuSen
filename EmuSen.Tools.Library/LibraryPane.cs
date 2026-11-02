using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using static EmuSen.Galaxia.Native.ToolEvents;

namespace EmuSen.Tools.Library
{
    // Library's content, declared once on LunaP's manager frame: duplicates in two tiers, broken or suspect files and unrecognised ones, hidden and shown again through games.db; the library is only read - see EmuSen_LibraryTools.md §3 and §10.
    public sealed class LibraryPane : ManagerLayout<LibraryRow>
    {
        // How a job is started, whether the tool can run, whether a file is there, the claim index, and the file manager and save picker, which a test replaces.
        internal static Func<string, IToolJob> StartJob = LibraryToolJob.Start;
        internal static Func<bool> ToolAvailable = () => LibraryToolJob.Available;
        internal static Func<string, bool> Exists = File.Exists;
        internal static Func<PackageIndex?> Claims = ClaimIndex;
        internal static Func<TopLevel?, string, Task<bool>> Reveal = RevealFolder;
        internal static Func<TopLevel?, Task<string?>> PickReportFile = PickFile;

        public static readonly TimeSpan PollEvery = TimeSpan.FromMilliseconds(100);

        private readonly DispatcherTimer _timer;
        private readonly TextBlock _overview = new() { Name = "LibraryOverview", TextWrapping = TextWrapping.Wrap };
        private readonly LunaAction _hide, _reveal, _save, _again, _stop;
        private readonly List<IReadOnlyList<DumpMember>> _dumps = new();
        private readonly List<CheckedFile> _broken = new();
        private readonly Dictionary<string, string> _consoles = new(StringComparer.Ordinal);
        private IReadOnlyList<ScrapedFile> _scraped = [];
        private IReadOnlySet<string> _hidden = new HashSet<string>();
        private PackageIndex? _claims;
        private IToolJob? _job;
        private string? _phase;
        private bool _stopping, _checked, _compared;
        private string? _notice;

        public LibraryPane()
        {
            Title = "Library Manager";
            HasSearch = true;
            // The detail, what the file is and what it matches, stands beside the list; A hides or shows the chosen file (EmuSen_Interface.md §15.1).
            DetailPlacement = AdaptiveDetailPlacement.Beside;
            ContentShare = 0.6;
            Header = new AdaptiveHeader("LIBRARY MANAGER");
            SectionWord = "Finding";
            ActionFor = RowAction;
            PageFor = r => [new LunaAction(r.Hidden ? "Mark to show again" : "Mark to hide", () => CycleMark(r)) { IsEnabled = Records is not null && !IsRunning }, _reveal];
            PageTitle = r => r.Title;
            SearchHint = "Search files";
            RowKey = r => r.Key;
            Column(new LunaColumn<LibraryRow>("Game", r => r.Title) { Sort = (a, b) => string.Compare(a.Title, b.Title, StringComparison.CurrentCultureIgnoreCase) }, bigScreen: true)
                .Column(new LunaColumn<LibraryRow>("Console", r => r.Console) { Width = "Auto" })
                .Column(new LunaColumn<LibraryRow>("Finding", LibraryFindings.Words) { Tone = r => r.Kind == Finding.Broken ? "LunaAttention" : r.Hidden ? "LunaMuted" : null }, bigScreen: true)
                .Column(new LunaColumn<LibraryRow>("Group", LibraryFindings.GroupText) { Width = "Auto", Sort = (a, b) => (a.Kind, a.Group).CompareTo((b.Kind, b.Group)) })
                .Column(new LunaColumn<LibraryRow>("Size", r => r.Size > 0 ? ToolWords.Size(r.Size) : "") { Width = "Auto", Sort = (a, b) => a.Size.CompareTo(b.Size) })
                .Column(new LunaColumn<LibraryRow>("State", r => r.Hidden ? "hidden" : "") { Width = "Auto" });
            MarksFor = r => LibraryFindings.MarksFor(r, Records is not null);
            InSection = Belongs;
            SearchKey = r => $"{r.Source} {r.GameName}";
            DetailFor = r => r is null ? Overview() : Scrolled(FileDetail(r));
            Summarize = LibraryFindings.Summary;
            Plan = LibraryFindings.Plan;
            Applying += Run;

            _hide = new LunaAction("Hide", MarkSelected) { HelpText = "Marks the chosen file to be hidden from the library, or a hidden one to be shown again." };
            _reveal = new LunaAction("Show in File Manager", () => _ = RevealSelected()) { HelpText = "Opens the folder that holds the chosen file in the system's file manager." };
            _save = new LunaAction("Save List…", () => _ = SaveList()) { HelpText = "Saves every finding as a text file." };
            _again = new LunaAction("Check Again", Refresh) { HelpText = "Reads the library's files again." };
            _stop = new LunaAction("Stop", StopScan) { HelpText = "Stops reading the library; the lists keep what was read." };
            SetToolActions([_hide, _reveal, _save, LunaAction.Separator(), _again, _stop], bigScreenButtons: [_stop]);
            SetListActions([_save, _again]);
            MarksChanged += UpdateActions;

            _timer = new DispatcherTimer { Interval = PollEvery };
            _timer.Tick += (_, _) => Poll();
            DetachedFromVisualTree += (_, _) => EndJob();
            SetSections(Filters, "all");
            UpdateActions();
        }

        private static readonly IReadOnlyList<AdaptiveSection> Filters =
        [
            new("all", "All", "Findings"), new("same-dump", "Same Dump", "Findings"), new("same-game", "Same Game", "Findings"),
            new("broken", "Broken or Suspect", "Findings"), new("unrecognised", "Unrecognised", "Findings"), new("hidden", "Hidden", "Findings"),
        ];

        // The library's folder, AppSettings.RomDirectory, which the host gives.
        public string? Library { get; set; }

        // games.db and media.db as the host lends them; null where none is lent, so nothing can be hidden and ScreenScraper's findings are not shown.
        public ILibraryRecords? Records { get; set; }

        // Whether the library's files are being read.
        public bool Busy => _job is not null;

        // The last refusal or failure in a player's words, null when there was none.
        public string? Notice => _notice;

        // Whether both reads finished, so every finding is shown.
        public bool Complete => _checked && _compared;

        private void Say(string? words)
        {
            _notice = words;
            SetNotice(words);
        }

        private static bool Belongs(LibraryRow row, string section) => section switch
        {
            "all" => true,
            "same-dump" => row.Kind == Finding.SameDump,
            "same-game" => row.Kind == Finding.SameGame,
            "broken" => row.Kind == Finding.Broken,
            "unrecognised" => row.Kind == Finding.Unrecognised,
            "hidden" => row.Hidden,
            _ => false,
        };

        // Reads the host's records, then the library's files: `rom check -r` over the whole library, then `rom dups`.
        public void Refresh()
        {
            if (_job is not null || IsRunning) return;
            Say(null);
            _dumps.Clear();
            _broken.Clear();
            _consoles.Clear();
            _checked = _compared = _stopping = false;
            ReadRecords();
            if (string.IsNullOrEmpty(Library))
            {
                Say("No library folder is set, so there is nothing to check. Choose the library's folder in Preferences, then open this again.");
                Redraw();
                return;
            }
            if (!ToolAvailable())
            {
                Say($"The library's files are read by EmuSen's platform library, which is not in use here ({LibraryToolJob.Report}).");
                Redraw();
                return;
            }
            Redraw();
            StartPhase(LibraryFindings.CheckRequest);
        }

        private void ReadRecords()
        {
            _hidden = new HashSet<string>();
            _scraped = [];
            _claims = Claims();
            if (Records is not { } records) return;
            try
            {
                _hidden = records.Hidden();
                _scraped = records.Scraped();
            }
            catch (Exception e) when (e is IOException or InvalidOperationException or InvalidDataException)
            {
                Say($"games.db or media.db could not be read ({e.Message}), so nothing can be hidden and ScreenScraper's findings are not shown.");
            }
        }

        private void StartPhase(string request)
        {
            _phase = request;
            try { _job = StartJob(request); }
            catch (Exception e) when (e is FormatException or InvalidOperationException)
            {
                Say(ToolWords.Refusal(e.Message));
                _job = null;
                _phase = null;
                return;
            }
            SetProgress(null, request == LibraryFindings.CheckRequest ? "Checking each file…" : "Comparing dumps…");
            _timer.Start();
            UpdateActions();
        }

        private void EndJob()
        {
            _timer.Stop();
            _job?.Dispose();
            _job = null;
        }

        // Asks the read to stop; the files read so far keep their findings, and no dump group is given for a stopped comparison.
        public void StopScan()
        {
            if (_job is null) return;
            _stopping = true;
            _job.Stop();
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
                    Say(ToolWords.Refusal(e.Message));
                    break;
                }
                if (line is null) break;
                Take(line);
            }
            if (_job is not { Ended: true }) return;
            bool check = _phase == LibraryFindings.CheckRequest;
            EndJob();
            _phase = null;
            if (check) _checked = !_stopping;
            else _compared = !_stopping;
            if (check && !_stopping && _notice is null)
            {
                Redraw();
                StartPhase(LibraryFindings.DupsRequest);
                return;
            }
            Finish(clearMarks: false);
            if (_stopping) Say("The check was stopped, so the lists hold only what was read before it.");
            Redraw();
            UpdateActions();
        }

        private void Take(string line)
        {
            using JsonDocument doc = JsonDocument.Parse(line);
            JsonElement e = doc.RootElement;
            string? kind = Text(e, "event");
            if (kind == "end" && Text(e, "error") is { } error) Say(ToolWords.Refusal(error));
            if (kind == "file" && Text(e, "source") is { } source)
            {
                if (Text(e, "system") is { } system) _consoles[source] = system;
                // A comparison's unreadable file is the check's finding already, so only the check's events are taken as broken.
                if (_phase == LibraryFindings.CheckRequest && LibraryFindings.Failed(e) is { } failed) _broken.Add(failed);
                if (e.TryGetProperty("index", out JsonElement index) && e.TryGetProperty("of", out JsonElement of) && index.TryGetInt32(out int n) && of.TryGetInt32(out int total) && total > 0)
                {
                    bool check = _phase == LibraryFindings.CheckRequest;
                    SetProgress((check ? 0 : 0.5) + n / (double)total / 2, $"{(check ? "Checking each file" : "Comparing dumps")}: {n} of {total}, {Path.GetFileNameWithoutExtension(source)}");
                }
            }
            if (LibraryFindings.Group(e) is { } group) _dumps.Add(group);
        }

        // The rows rebuilt from what has been read and the records, the marks of rows still present kept.
        private void Redraw()
        {
            string library = Library ?? "";
            SetRows(library.Length == 0 ? [] : LibraryFindings.Rows(library, _dumps, _broken, _scraped, _hidden, _consoles, Exists, PackageOf));
            Header = new AdaptiveHeader("LIBRARY MANAGER", Facts: $"{library.Length} files · {_dumps.Count} same dumps · {_broken.Count} broken or suspect · {_hidden.Count} hidden");
            _overview.Text = OverviewText;
        }

        // The package that claims a file, by its folder under Packages as `rom dups` names it.
        private string? PackageOf(string full) =>
            _claims?.Claimant(full)?.Folder is { } folder ? Path.GetRelativePath(DataStore.Packages, folder).Replace(Path.DirectorySeparatorChar, '/') : null;

        private void MarkSelected()
        {
            if (Selected is { } row) CycleMark(row);
        }

        // The row's one action on the big screen: hiding it, or showing it again, asked in the plan's question before it is done.
        private LunaAction? RowAction(LibraryRow row) => Records is null || IsRunning ? null
            : new LunaAction(row.Hidden ? "Show Again…" : "Hide…", () => { if (SetMark(row, row.Hidden ? LibraryFindings.Unhide.Key : LibraryFindings.Hide.Key)) OpenPlan(); });

        // The actions stay enabled whichever row is chosen, so the pad's walk through them never changes with the row.
        private void UpdateActions()
        {
            _hide.IsEnabled = !IsRunning && Records is not null;
            _again.IsEnabled = _job is null && !IsRunning;
            _stop.IsEnabled = _job is not null;
        }

        // The marked games hidden or shown again in games.db, then the rows drawn with the new state; no file is touched.
        private void Run(IReadOnlyList<ManagerMarked<LibraryRow>> marked)
        {
            var failed = new List<string>();
            if (Records is { } records)
            {
                foreach ((string path, bool hide) in LibraryFindings.Changes(marked))
                {
                    try { records.SetHidden(path, hide); }
                    catch (Exception e) when (e is IOException or InvalidOperationException) { failed.Add($"{Path.GetFileName(path)} ({e.Message})"); }
                }
                try { _hidden = records.Hidden(); }
                catch (Exception e) when (e is IOException or InvalidOperationException) { failed.Add(e.Message); }
            }
            Finish();
            Say(failed.Count > 0 ? "games.db did not take every change: " + string.Join("; ", failed) : null);
            Redraw();
            UpdateActions();
        }

        // The findings as text, in the form Save List writes.
        public string ReportText => LibraryFindings.Report(Library ?? "", Rows);

        // Writes the list to a file outside the library; a path inside it is refused, since nothing here writes there.
        public bool SaveListTo(string path)
        {
            if (Library is { Length: > 0 } library && LibraryFindings.Relative(library, path) is not null)
            {
                Say("The list is not saved inside the library, which EmuSen never writes to. Choose a folder outside it.");
                return false;
            }
            try
            {
                File.WriteAllText(path, ReportText);
                Say($"The list was saved as {path}.");
                return true;
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException)
            {
                Say($"The list could not be saved as {path} ({e.Message}).");
                return false;
            }
        }

        // The desktop asks where; the big screen, which has no picker a pad drives, saves under EmuSen's Logs folder and says where.
        private async Task SaveList()
        {
            string? path = Form == AdaptiveForm.Desktop ? await PickReportFile(TopLevel.GetTopLevel(this)) : Path.Combine(DataStore.Logs, $"Library {DateTime.Now:yyyy-MM-dd HHmmss}.txt");
            if (path is null) return;
            if (Form != AdaptiveForm.Desktop) Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            SaveListTo(path);
        }

        private async Task RevealSelected()
        {
            if (Selected is not { } row) return;
            string? folder = Path.GetDirectoryName(row.FullPath);
            if (folder is null || !await Reveal(TopLevel.GetTopLevel(this), folder)) Say($"The file manager could not be opened at {folder}.");
        }

        private static async Task<bool> RevealFolder(TopLevel? top, string folder) =>
            Directory.Exists(folder) && top is not null && await top.Launcher.LaunchUriAsync(new Uri(folder));

        private static async Task<string?> PickFile(TopLevel? top)
        {
            if (top is null) return null;
            IStorageFile? file = await top.StorageProvider.SaveFilePickerAsync(new FilePickerSaveOptions { Title = "Save the library manager's list", SuggestedFileName = "EmuSen library.txt", DefaultExtension = "txt" });
            return file?.TryGetLocalPath();
        }

        private static PackageIndex? ClaimIndex()
        {
            try { return GamePackages.Index(PackagePlaces.Current()); }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException or InvalidOperationException) { return null; }
        }

        // What the detail shows with no file chosen: how many of each finding, and what the window never does.
        public string OverviewText
        {
            get
            {
                string counts = string.Join(", ", Enum.GetValues<Finding>().Select(k => $"{LibraryFindings.KindName(k).ToLowerInvariant()} {Rows.Count(r => r.Kind == k)}"));
                string state = Busy ? "The library's files are being read." : Complete ? "Every file of the library was read." : "The library's files have not all been read.";
                string records = Records is null ? " ScreenScraper's findings and hiding need EmuSen's games.db, which is not lent here." : "";
                return $"{state} Findings: {counts}.{records} Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db.";
            }
        }

        private Control Overview()
        {
            _overview.Text = OverviewText;
            return _overview;
        }

        // The chosen file whole: its path, finding, the rest of its group, each failed check, and the place the header checks will fill.
        private static Control FileDetail(LibraryRow row)
        {
            var panel = new StackPanel { Spacing = 6, Name = "LibraryDetail" };
            void Line(string text, bool strong = false) =>
                panel.Children.Add(new TextBlock { Text = text, TextWrapping = TextWrapping.Wrap, FontWeight = strong ? FontWeight.SemiBold : FontWeight.Normal });
            Line(row.Title, strong: true);
            Line(row.Source);
            Line($"{LibraryFindings.KindName(row.Kind)}: {LibraryFindings.Words(row)}");
            if (row.Console.Length > 0) Line($"System: {row.Console}");
            if (row.Package is { } package) Line($"Claimed by the package {package}, and counted with it as one game.");
            if (row.GameId is { } id) Line($"ScreenScraper game {id}" + (row.GameName is { Length: > 0 } n ? $": {n}" : ""));
            foreach (string other in row.Others) Line($"Also: {other}");
            foreach (FailedCheck f in row.Failed) Line($"{LibraryFindings.CheckName(f.Check)}: {f.Words}");
            if (row.Kind == Finding.Unrecognised) Line("ScreenScraper knew no game by this file's hashes. A hack, a translation or a rare dump is often unknown there too, so this is a hint, not a fault.");
            Line("Header checksum: not checked yet.");
            Line(row.Hidden ? "Hidden from the library." : "Shown in the library.");
            return panel;
        }

        private static ScrollViewer Scrolled(Control content) =>
            new() { Content = content, HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled };
    }
}
