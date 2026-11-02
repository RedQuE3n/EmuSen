using System.Text.Json;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Threading;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using static EmuSen.Tools.Cores.CoreRow;

namespace EmuSen.Tools.Cores
{
    // Cores' content on LunaP's manager frame: the project's cores and libretro's, filtered by console and state, marked and applied through the core tool's jobs, approved and made default through the runtime - see EmuSen_Cores.md §2.
    public sealed class CoresPane : ManagerLayout<CoreRow>
    {
        // How a job is started, whether the tool can run, and the free and total bytes of a disk, which a test replaces.
        internal static Func<string, IToolJob> StartJob = LibraryToolJob.Start;
        internal static Func<bool> ToolAvailable = () => LibraryToolJob.Available;
        internal static Func<string, (long Free, long Total)?> Disk = DiskOf;
        // The runtime a pane is given, which a test replaces with one of its own.
        internal static Func<CoresRuntime?, CoresRuntime?> RuntimeFor = r => r;

        public static readonly TimeSpan PollEvery = TimeSpan.FromMilliseconds(100);
        public const string ListingRequest = """{"op":"core-ls","show":"all","detail":true}""";
        public const string FetchRequest = """{"op":"core-fetch"}""";
        public const string VerifyRequest = """{"op":"core-verify","cores":[]}""";
        public const string EngineKey = "Engine";

        private readonly CoresRuntime? _runtime;
        private readonly CoresHost _host;
        private readonly DispatcherTimer _timer;
        private readonly TextBlock _overview = new() { Name = "CoresOverview", TextWrapping = TextWrapping.Wrap };
        private readonly Dictionary<string, JsonElement> _info = new();
        private readonly Dictionary<string, string> _outcomes = new();
        private readonly LunaAction _install, _update, _remove, _approve, _default, _options, _verify, _check;
        private Dictionary<string, string> _consoleOf = new();
        private Dictionary<string, IReadOnlyList<string>> _enginesOf = new();
        private List<CoreRow> _listing = new();
        private Queue<CoresStep> _steps = new();
        private CoresStep? _step;
        private int _stepCount, _stepsDone;
        private IToolJob? _job;
        private string _jobKind = "";
        private string? _notice;
        private string? _catalogue;
        private readonly List<string> _installed = new();

        public CoresPane(CoresRuntime? runtime, CoresHost? host = null)
        {
            _runtime = RuntimeFor(runtime);
            _host = host ?? new CoresHost();
            Title = "Cores";
            HasSearch = true;
            SearchHint = "Search cores";
            RowKey = r => r.Key;
            // The screen of EmuSen_Interface.md §14: the pane beside the list, the consoles on the shoulders, the states as the List card's filter, one action per row on A and the rest on X.
            DetailPlacement = AdaptiveDetailPlacement.Beside;
            ContentShare = 0.6;
            SectionWord = "Console";
            Header = new AdaptiveHeader("CORES");
            Column(new LunaColumn<CoreRow>("Core", r => r.Name) { Sort = (a, b) => string.Compare(a.Name, b.Name, StringComparison.CurrentCultureIgnoreCase) }, bigScreen: true)
                .Column(new LunaColumn<CoreRow>("Console", CoresWords.Consoles) { Width = "Auto" }, bigScreen: true)
                .Column(new LunaColumn<CoreRow>("Source", r => r.Source) { Width = "Auto" })
                .Column(new LunaColumn<CoreRow>("Version", r => r.Version))
                .Column(new LunaColumn<CoreRow>("Status", StateText) { Tone = StateTone, Width = "Auto", Alignment = HorizontalAlignment.Right }, bigScreen: true)
                .Column(new LunaColumn<CoreRow>("Size", r => r.Size is long n and > 0 ? CoresWords.Size(n) : "") { Width = "Auto", Sort = (a, b) => (a.Size ?? 0).CompareTo(b.Size ?? 0) });
            MarksFor = CoresPlan.MarksFor;
            InSection = Belongs;
            SetFilters([new AdaptiveSection("all", "All"), new AdaptiveSection("installed", "Installed"), new AdaptiveSection("available", "Available"), new AdaptiveSection("updates", "Updates"), new AdaptiveSection("approval", "To approve"), new AdaptiveSection("unsupported", "Not supported")]);
            InFilter = Belongs;
            SearchKey = r => $"{r.Name} {r.ListedAs} {r.Stem} {r.Engine} {string.Join(" ", r.Consoles)} {r.Licence}";
            DetailFor = r => r is null ? Overview() : Scrolled(DetailOf(r));
            Summarize = CoresPlan.Summary;
            Plan = CoresPlan.Text;
            PlanTitle = CoresPlan.Title;
            ActionFor = RowAction;
            PageFor = RowPage;
            PageTitle = r => r.Name;
            Applying += Run;

            _install = new LunaAction("Install", () => MarkSelected(CoresPlan.Install.Key)) { HelpText = "Marks the chosen libretro core to be downloaded and installed." };
            _update = new LunaAction("Update", () => MarkSelected(CoresPlan.Update.Key)) { HelpText = "Marks the chosen core to be updated to the build the index offers." };
            _remove = new LunaAction("Remove", () => MarkSelected(CoresPlan.Remove.Key)) { HelpText = "Marks the chosen downloaded core to be removed; saves and settings are kept." };
            _approve = new LunaAction("Approve…", () => ApproveSelectedAsync().Logged("cores", "Approve")) { HelpText = "Asks whether to trust the chosen core, showing what it says it is and its SHA-256." };
            _default = new LunaAction("Make Default", MakeDefault) { HelpText = "Makes the chosen core the engine of its console." };
            _options = new LunaAction("Options…", OpenOptions) { HelpText = "Opens the chosen core's options in its console's settings." };
            _verify = new LunaAction("Verify", Verify) { HelpText = "Checks each downloaded core against its record, and refreshes EmuSen's copies beside it." };
            _check = new LunaAction("Check for Updates", CheckForUpdates) { HelpText = "Reads libretro's catalogue and this platform's index from buildbot.libretro.com." };
            // The desktop's toolbar; on the big screen A, X and the List card carry them and no button is drawn.
            SetToolActions([_install, _update, _remove, _approve, _default, _options, LunaAction.Separator(), _verify, _check], bigScreenButtons: []);
            SetListActions([_check, _verify]);
            MarksChanged += UpdateActions;
            foreach (LunaTable<CoreRow> table in new[] { Table, BigScreenTable })
                table.AddHandler(Avalonia.Controls.Primitives.SelectingItemsControl.SelectionChangedEvent, (_, _) => UpdateActions());

            _timer = new DispatcherTimer { Interval = PollEvery };
            _timer.Tick += (_, _) => Poll();
            DetachedFromVisualTree += (_, _) => Stop();
            SetSections([new AdaptiveSection("all", "All", "Consoles")], "all");
            UpdateActions();
        }

        // Whether a job is running.
        public bool Busy => _job is not null;

        // The last refusal, failure or outcome in a player's words, null when there is none.
        public string? Notice => _notice;

        // The date the catalogue was fetched, null when it never was.
        public string? Catalogue => _catalogue;

        public string? OutcomeOf(CoreRow row) => _outcomes.TryGetValue(row.Key, out string? said) ? said : null;

        // The status column: a run's outcome for the row while one is shown, else the words of EmuSen_Interface.md §14.3, each with its colour.
        private string StateText(CoreRow row) => OutcomeOf(row) is { Length: > 0 } said ? char.ToUpperInvariant(said[0]) + said[1..] : CoresWords.Status(row);

        private string? StateTone(CoreRow row) => OutcomeOf(row) is { } said ? (said.StartsWith("refused", StringComparison.Ordinal) ? "LunaError" : "LunaMuted") : CoresWords.Tone(row);

        // The row's one action on the big screen, named in the help bar; none while a job runs (EmuSen_Interface.md §14.5).
        private LunaAction? RowAction(CoreRow row)
        {
            if (IsRunning || _job is not null) return null;
            if (row.AwaitsApproval && HashOf(row) is not null && _runtime is not null) return new LunaAction("Approve…", () => ApproveSelectedAsync().Logged("cores", "Approve"));
            if (row.IsLibretro && row.State == "available") return new LunaAction("Install…", () => MarkAndAsk(row, CoresPlan.Install.Key));
            if (row.IsLibretro && row.State == "update") return new LunaAction("Update…", () => MarkAndAsk(row, CoresPlan.Update.Key));
            if (row is { Engine: not null, Consoles.Count: > 0 } && !row.IsDefault && _runtime is not null) return new LunaAction("Make Default", MakeDefault);
            if (row is { Engine: not null, Consoles.Count: > 0 } && _host.Options is not null) return new LunaAction("Options", OpenOptions);
            return null;
        }

        private void MarkAndAsk(CoreRow row, string key)
        {
            if (SetMark(row, key)) OpenPlan();
        }

        // The row's page under X: everything else it offers, each where it applies (EmuSen_Interface.md §14.5).
        private IReadOnlyList<LunaAction> RowPage(CoreRow row)
        {
            var rows = new List<LunaAction>();
            foreach (ManagerMark mark in CoresPlan.MarksFor(row))
                rows.Add(new LunaAction(MarkOf(row)?.Key == mark.Key ? $"Unmark ({mark.Label})" : $"Mark to {mark.Label}", () => SetMark(row, MarkOf(row)?.Key == mark.Key ? null : mark.Key)));
            if (row is { Engine: not null, Consoles.Count: > 0 } && _host.Options is not null) rows.Add(new LunaAction("Options", OpenOptions));
            if (Firmware(row).Count > 0 && _host.Firmware is { } firmware) rows.Add(new LunaAction("Firmware", firmware));
            if (HashOf(row) is not null && (row.AwaitsApproval || row.IsLibretro && row.Installed) && _runtime is not null) rows.Add(new LunaAction("Approve…", () => ApproveSelectedAsync().Logged("cores", "Approve")));
            if (row.IsLibretro && row.Installed) rows.Add(new LunaAction("Verify", Verify));
            if (row.IsLibretro && row.Installed) rows.Add(new LunaAction("Remove…", () => MarkAndAsk(row, CoresPlan.Remove.Key)));
            if (row.Installed && _host.OpenFolder is { } open && FolderOf(row) is { } folder) rows.Add(new LunaAction("Open Folder", () => open(folder)));
            // The page closes itself once a row is chosen, so the details open after that.
            rows.Add(new LunaAction("Details", () => Dispatcher.UIThread.Post(() => OpenPage(Scrolled(DetailsOf(row))), DispatcherPriority.Loaded)));
            return rows;
        }

        private IReadOnlyList<JsonElement> Firmware(CoreRow row) =>
            row.Stem is { } stem && _info.TryGetValue(stem, out JsonElement i) && i.TryGetProperty("firmware", out JsonElement fw) && fw.ValueKind == JsonValueKind.Array ? fw.EnumerateArray().ToList() : [];

        private static bool Belongs(CoreRow row, string section) => section switch
        {
            "all" => true,
            "installed" => row.Installed,
            "available" => row.State == "available",
            "updates" => row.State == "update",
            "approval" => row.AwaitsApproval,
            "unsupported" => row.State is "unsupported" or "not_read",
            _ => section.StartsWith("console:", StringComparison.Ordinal) && row.Consoles.Contains(section["console:".Length..]),
        };

        // The runtime's cores read, then the core tool's listing started; the rows are shown when it ends.
        public void Refresh() => Refresh(keepNotice: false);

        // After a run, an approval or a fetch the notice saying what happened is kept; a listing asked for afresh clears it.
        private void Refresh(bool keepNotice)
        {
            if (_job is not null) return;
            if (!keepNotice) _notice = null;
            Rescan();
            ReadRuntime();
            _listing = RuntimeRows();
            if (!ToolAvailable())
            {
                _notice = $"Downloading cores needs EmuSen's platform library, which is not in use here ({LibraryToolJob.Report}).";
                Show(_listing);
                return;
            }
            if (Start(ListingRequest, "ls")) SetProgress(null, "Listing the cores…");
            else Show(_listing);
        }

        private void CheckForUpdates()
        {
            if (_job is not null || !ToolAvailable()) return;
            _notice = null;
            if (Start(FetchRequest, "fetch")) SetProgress(null, "Reading libretro's catalogue from buildbot.libretro.com…");
        }

        private void Verify()
        {
            if (_job is not null || !ToolAvailable()) return;
            _notice = null;
            _outcomes.Clear();
            if (Start(VerifyRequest, "verify")) SetProgress(null, "Checking the downloaded cores…");
        }

        private void ReadRuntime()
        {
            _consoleOf = new();
            _enginesOf = new();
            if (_runtime is null) return;
            try
            {
                using JsonDocument systems = JsonDocument.Parse(_runtime.Systems());
                foreach (JsonElement s in systems.RootElement.EnumerateArray())
                {
                    if (Text(s, "id") is not { } id || Text(s, "console") is not { } console) continue;
                    _consoleOf[id] = console;
                    if (!_enginesOf.ContainsKey(console)) _enginesOf[console] = Strings(s, "engines");
                }
            }
            catch (Exception e) when (e is JsonException or InvalidOperationException) { _notice = "EmuSen's runtime did not list its systems: " + e.Message; }
        }

        // The console's engine: its stored setting, else the first engine discovery lists for it.
        private bool IsDefault(string console, string engine)
        {
            string? stored = null;
            try { stored = _runtime?.Setting(console, EngineKey); } catch (InvalidOperationException) { }
            string? first = _enginesOf.TryGetValue(console, out IReadOnlyList<string>? engines) && engines.Count > 0 ? engines[0] : null;
            return (string.IsNullOrEmpty(stored) ? first : stored) == engine;
        }

        private List<CoreRow> RuntimeRows()
        {
            var rows = new List<CoreRow>();
            if (_runtime is null) return rows;
            try
            {
                using JsonDocument cores = JsonDocument.Parse(_runtime.Cores());
                foreach (JsonElement c in cores.RootElement.EnumerateArray())
                    if (OfRuntime(c, _consoleOf, IsDefault) is { } row) rows.Add(row);
            }
            catch (Exception e) when (e is JsonException or InvalidOperationException) { _notice = "EmuSen's runtime did not list its cores: " + e.Message; }
            return rows;
        }

        // A libretro core discovery lists under its folder lends the libretro row its engine and trust (EmuSen_Cores.md §7.3).
        private static List<CoreRow> Joined(List<CoreRow> runtime, List<CoreRow> tool)
        {
            string Folder(CoreRow r) => r.Library is { } l ? Path.GetFileName(Path.GetDirectoryName(l) ?? "") ?? "" : "";
            bool InLibretro(CoreRow r) => r.Library is { } l && Path.GetFileName(Path.GetDirectoryName(Path.GetDirectoryName(l) ?? "") ?? "") == "libretro";
            var rows = runtime.Where(r => !InLibretro(r)).ToList();
            foreach (CoreRow t in tool)
            {
                CoreRow? found = runtime.FirstOrDefault(r => InLibretro(r) && Folder(r) == t.Stem);
                rows.Add(found is null ? t : t with { Engine = found.Engine, Trust = found.Trust, Library = found.Library, IsDefault = found.IsDefault, Sha256 = found.Sha256 });
            }
            return rows;
        }

        private void Show(List<CoreRow> rows)
        {
            SetRows(rows);
            FillSections();
            ShowSpace();
            SetNotice(_notice);
            ShowOverview();
            UpdateActions();
        }

        private bool Start(string request, string kind)
        {
            try { _job = StartJob(request); }
            catch (Exception e) when (e is FormatException or InvalidOperationException)
            {
                _notice = CoresWords.Refusal(e.Message);
                SetNotice(_notice);
                return false;
            }
            _jobKind = kind;
            _timer.Start();
            UpdateActions();
            return true;
        }

        private void Stop()
        {
            _timer.Stop();
            _job?.Dispose();
            _job = null;
        }

        private void MarkSelected(string key)
        {
            if (Selected is { } row) SetMark(row, key);
        }

        private void UpdateActions()
        {
            CoreRow? row = Selected;
            bool idle = !IsRunning && _job is null;
            foreach (LunaAction a in new[] { _install, _update, _remove }) a.IsEnabled = !IsRunning;
            _approve.IsEnabled = idle && row is { Sha256: not null } && (row.AwaitsApproval || row.IsLibretro && row.Installed);
            _default.IsEnabled = idle && row is { Engine: not null, Consoles.Count: > 0 } && !row.IsDefault && _runtime is not null;
            _options.IsEnabled = row is { Engine: not null, Consoles.Count: > 0 } && _host.Options is not null;
            _verify.IsEnabled = idle && ToolAvailable();
            _check.IsEnabled = idle && ToolAvailable();
            RefreshHints();
        }

        private void Run(IReadOnlyList<ManagerMarked<CoreRow>> marked)
        {
            _outcomes.Clear();
            _installed.Clear();
            _notice = null;
            _steps = new Queue<CoresStep>(CoresPlan.Steps(marked));
            _stepCount = _steps.Count;
            _stepsDone = 0;
            NextStep();
        }

        private void NextStep()
        {
            while (_steps.TryDequeue(out CoresStep? step))
            {
                _step = step;
                if (Start(step.Request, "step"))
                {
                    SetProgress((double)_stepsDone / Math.Max(1, _stepCount), $"{step.Doing} {string.Join(", ", step.Rows.Select(r => r.Name))}…");
                    return;
                }
                foreach (CoreRow row in step.Rows) _outcomes[row.Key] = _notice!;
                _stepsDone++;
            }
            _step = null;
            Finish();
            if (_installed.Count > 0)
                _notice = $"Installed {string.Join(", ", _installed)}. Nothing downloaded runs until you approve it." + (_notice is null ? "" : " " + _notice);
            Refresh(keepNotice: true);
        }

        // Every event the jobs have queued taken and shown; a test calls it in place of the timer.
        public void Poll()
        {
            if (_job is null) return;
            for (int i = 0; i < 256; i++)
            {
                string? line;
                try { line = _job.Next(0); }
                catch (InvalidOperationException e)
                {
                    _notice = CoresWords.Refusal(e.Message);
                    break;
                }
                if (line is null) break;
                Take(line);
            }
            if (_job is not { Ended: true }) return;
            _timer.Stop();
            _job.Dispose();
            _job = null;
            switch (_jobKind)
            {
                case "ls":
                    Finish(clearMarks: false);
                    Show(Joined(_listing, _toolRows));
                    _toolRows.Clear();
                    break;
                case "fetch":
                case "verify":
                    Finish(clearMarks: false);
                    Refresh(keepNotice: true);
                    break;
                default:
                    _stepsDone++;
                    NextStep();
                    break;
            }
        }

        private readonly List<CoreRow> _toolRows = new();

        private void Take(string line)
        {
            using JsonDocument doc = JsonDocument.Parse(line);
            JsonElement e = doc.RootElement;
            string? stem = Text(e, "stem");
            string key = "libretro:" + stem;
            switch (Text(e, "event"))
            {
                case "core" when _jobKind == "ls":
                    if (OfTool(e, _consoleOf) is { } row) _toolRows.Add(row);
                    if (stem is not null) _info[stem] = e.Clone();
                    break;
                case "done":
                    if (e.TryGetProperty("catalogue", out JsonElement c)) _catalogue = c.ValueKind == JsonValueKind.String ? c.GetString() : null;
                    if (_jobKind == "ls" && _catalogue is null && _notice is null) _notice = "No catalogue of libretro's cores yet: Check for Updates reads it from buildbot.libretro.com.";
                    break;
                case "refused" when stem is null:
                    _notice = CoresWords.Refusal(Text(e, "reason") ?? "");
                    break;
                case "refused":
                    _outcomes[key] = "refused: " + (Text(e, "reason") ?? "");
                    _notice = $"{stem}: {Text(e, "reason")}";
                    break;
                case "fetched":
                    _notice = $"Read libretro's catalogue on {Text(e, "date")}: {Number(e, "cores")} cores for this machine.";
                    break;
                case "progress" when _step is not null:
                    long bytes = Number(e, "bytes"), total = Number(e, "total");
                    double within = total > 0 ? Math.Min(1, (double)bytes / total) * 0.9 : 0;
                    SetProgress((_stepsDone + within) / Math.Max(1, _stepCount), $"{_step.Doing} {stem}: {CoresWords.Size(bytes)}{(total > 0 ? " of " + CoresWords.Size(total) : "")}");
                    break;
                case "checked" when _step is not null:
                    SetProgress((_stepsDone + 0.95) / Math.Max(1, _stepCount), $"{_step.Doing} {stem}: checking its {Text(e, "check")}");
                    break;
                case "installed":
                    _outcomes[key] = "installed, to approve";
                    _installed.Add(stem ?? "");
                    RevokeAll(e);
                    break;
                case "removed":
                    _outcomes[key] = "removed";
                    RevokeAll(e);
                    break;
                case "current":
                    _outcomes[key] = "already current";
                    break;
                case "verified":
                    string result = Text(e, "result") ?? "";
                    _outcomes[key] = result switch
                    {
                        "refreshed" => "refreshed",
                        "changed" => "changed: needs approval",
                        _ => result,
                    };
                    if (result == "changed") _notice = $"{stem}: {Text(e, "reason")}";
                    break;
            }
        }

        // The hashes a job dropped from disk revoked through the runtime, since approvals are its calls, never a job's (EmuSen_Cores.md §6).
        private void RevokeAll(JsonElement e)
        {
            if (_runtime is null) return;
            foreach (string sha in Strings(e, "revoke"))
            {
                try { _runtime.Revoke(sha); }
                catch (InvalidOperationException x) { _notice = $"The approval of {sha} could not be revoked: {x.Message}"; }
            }
        }

        /// <summary>The chosen core's approval asked, and recorded through the runtime when the player approves.</summary>
        public async Task<bool> ApproveSelectedAsync()
        {
            if (_runtime is null || Selected is not { } row) return false;
            if (HashOf(row) is not { } sha)
            {
                await RefuseAsync($"{row.Name} cannot be approved yet: its SHA-256 is known once it is downloaded.");
                return false;
            }
            string file = row.IsLibretro ? $"{row.Stem}_libretro" : Path.GetFileName(row.Library ?? row.Name);
            string message = CoreApproval.Message(row.Name, file, row.Authors, row.Licence, row.Consoles, sha, downloaded: row.IsLibretro);
            // On the big screen the question is the frame's card (EmuSen_Interface.md §14.6); on the desktop the host's dialog.
            bool yes = Form == AdaptiveForm.BigScreen ? await Ask($"Approve {row.Name}?", message, "Approve", "Not Now") : await _host.Confirm(CoreApproval.Title, message);
            if (!yes) return false;
            try { _runtime.Approve(sha); }
            catch (InvalidOperationException e)
            {
                await RefuseAsync($"{row.Name} could not be approved: {e.Message}");
                return false;
            }
            _notice = $"Approved {row.Name} (SHA-256 {sha}).";
            Refresh(keepNotice: true);
            return true;
        }

        // A refusal on the notice line and, where the host can, in a message the player must dismiss, since the big screen's bar is easily missed.
        private async Task RefuseAsync(string words)
        {
            _notice = words;
            SetNotice(words);
            if (_host.Tell is { } tell) await tell(CoreApproval.Title, words);
        }

        // The runtime's directories listed again, so a core a job just installed or removed is the runtime's to approve.
        private void Rescan()
        {
            try { _runtime?.Rescan?.Invoke(); }
            catch (InvalidOperationException e) { _notice = $"The cores could not be listed again: {e.Message}"; }
        }

        // The hash a row is approved by: the runtime's, else the one its downloaded entry records; the page shows the same one.
        private string? HashOf(CoreRow row) => row.Sha256 ?? (row.Stem is { } stem && _info.TryGetValue(stem, out JsonElement i) ? Text(i, "sha256") : null);

        // The chosen core made its console's engine: the section's console, else each console it runs (EmuSen_Cores.md §4.3).
        private void MakeDefault()
        {
            if (_runtime is null || Selected is not { Engine: { } engine } row) return;
            IEnumerable<string> consoles = SelectedSection is { } s && s.StartsWith("console:", StringComparison.Ordinal) && row.Consoles.Contains(s["console:".Length..]) ? [s["console:".Length..]] : row.Consoles;
            foreach (string console in consoles)
            {
                try { _runtime.SetSetting(console, EngineKey, engine); }
                catch (InvalidOperationException e) { _notice = $"{row.Name} could not be made {console}'s engine: {e.Message}"; }
            }
            _listing = RuntimeRows();
            SetRows(Rows.Select(r => r with { IsDefault = r.Engine is { } en && r.Consoles.Any(c => IsDefault(c, en)) }).ToList());
            SetNotice(_notice);
            UpdateActions();
        }

        private void OpenOptions()
        {
            if (Selected is { Engine: { } engine, Consoles.Count: > 0 } row) _host.Options?.Invoke(row.Consoles[0], engine);
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
            sections.AddRange(Rows.SelectMany(r => r.Consoles).Distinct().Order(StringComparer.CurrentCultureIgnoreCase).Select(c => new AdaptiveSection("console:" + c, c, "Consoles")));
            string selected = SelectedSection is { } s && sections.Any(x => x.Key == s) ? s : "all";
            SetSections(sections, selected);
            ShowHeader();
        }

        // The header's facts: the counts that need the player first, then the catalogue's date (EmuSen_Interface.md §14.2).
        private void ShowHeader()
        {
            var facts = new List<string> { $"{Rows.Count} cores", $"{Rows.Count(r => r.Installed)} installed" };
            int approve = Rows.Count(r => r.AwaitsApproval), updates = Rows.Count(r => r.State == "update");
            if (approve > 0) facts.Add($"⚠ {approve} to approve");
            if (updates > 0) facts.Add($"↑ {updates} {(updates == 1 ? "update" : "updates")}");
            facts.Add(_catalogue is null ? "catalogue not read yet" : $"catalogue read {CoresWords.Day(_catalogue)}");
            Header = new AdaptiveHeader("CORES", Facts: string.Join(" · ", facts));
        }

        // /Cores' total and its disk's free space (EmuSen_Cores.md §2.1).
        private void ShowSpace()
        {
            if (Disk(DataStore.Cores) is not { Total: > 0 } disk) return;
            long cores = Rows.Where(r => r.Installed && r.Source != Project).Sum(r => r.Size ?? 0);
            SetSpace($"/Cores {CoresWords.Size(cores)} · {CoresWords.Size(disk.Free)} free", 1 - (double)disk.Free / disk.Total);
        }

        public string OverviewText
        {
            get
            {
                if (_notice is not null) return _notice;
                int built = Rows.Count(r => r.Source == Project), libretro = Rows.Count(r => r.IsLibretro), installed = Rows.Count(r => r.IsLibretro && r.Installed);
                string fetched = _catalogue is null ? "libretro's catalogue has not been read yet" : $"libretro's catalogue was read on {_catalogue}";
                return $"{built} cores ship with EmuSen. {libretro} libretro cores are listed, {installed} of them installed; {fetched}.";
            }
        }

        private Control Overview()
        {
            _overview.Text = OverviewText;
            return _overview;
        }

        private void ShowOverview() => _overview.Text = OverviewText;

        private static TextBlock Line(string text, string? name = null, string? tone = null)
        {
            var line = new TextBlock { Text = text, TextWrapping = TextWrapping.Wrap, Name = name };
            if (tone is not null) line.Bind(TextBlock.ForegroundProperty, line.GetResourceObservable(tone));
            return line;
        }

        // The pane: the core in sentences, nothing raw, each fact once (EmuSen_Interface.md §14.4); the raw facts are the Details page.
        private Control DetailOf(CoreRow row)
        {
            var panel = new StackPanel { Spacing = 8, Name = "CoresDetail" };
            panel.Children.Add(new TextBlock { Text = row.Name, TextWrapping = TextWrapping.Wrap, FontWeight = Form == AdaptiveForm.Desktop ? FontWeight.SemiBold : FontWeight.Normal, Classes = { AdaptiveLayout.PaneTitleClass } });
            JsonElement? info = row.Stem is { } stem && _info.TryGetValue(stem, out JsonElement i) ? i : null;
            string source = row.Source == Project ? "Ships with EmuSen" : row.Source == Added ? "Added by you" : "From libretro's buildbot";
            panel.Children.Add(Line($"{source} · {CoresWords.Consoles(row)}{(row.ListedAs is { } listed ? $" · listed there as “{listed}”" : "")}", tone: "LunaMuted"));
            string runs = row.Consoles.Count > 0 ? string.Join(" and ", row.Consoles) + " games" : "games of no console EmuSen knows";
            panel.Children.Add(Line(row.IsLibretro ? $"Runs {runs} through EmuSen's libretro host, in a sandbox of its own." : row.Source == Project ? $"Runs {runs}. Built into EmuSen and updated with it, so there is nothing to install or approve." : $"Runs {runs}, from a file of your own."));
            string by = row.Authors.Count > 0 ? $"By {string.Join(", ", row.Authors)}" : "By authors not said";
            string licence = row.Licence.Length > 0 ? $", under the {row.Licence}" : ", licence not said";
            string build = row.Version.Length > 0 ? (row.IsLibretro ? $" Build {row.Version}" : $" Version {row.Version}") : "";
            string size = row.Size is long n and > 0 ? $", {CoresWords.Size(n)}" : "";
            string fetched = info is { } d1 && Text(d1, "fetched") is { } when ? $", downloaded {when}" : "";
            panel.Children.Add(Line($"{by}{licence}.{build}{size}{fetched}{(build.Length + size.Length + fetched.Length > 0 ? "." : "")}"));
            foreach (JsonElement f in Firmware(row))
            {
                bool optional = f.TryGetProperty("optional", out JsonElement o) && o.ValueKind == JsonValueKind.True;
                panel.Children.Add(Line($"Needs {Article(Text(f, "desc") ?? "firmware")} of your own{(optional ? " (optional)" : "")}, which EmuSen never downloads."));
            }
            if (row.AwaitsApproval) panel.Children.Add(Line("⚠ Waiting for your approval. Nothing downloaded runs until you approve it.", tone: "LunaAttention"));
            else if (row.Trust == "needs_refresh") panel.Children.Add(Line("⚠ EmuSen's copies beside it are older than this build: Verify refreshes them.", tone: "LunaAttention"));
            else if (row.State is "unsupported" or "not_read") panel.Children.Add(Line($"Not supported{(row.Reason is { } why ? $": {why}" : "")}.", tone: "LunaMuted"));
            else if (row.IsLibretro && row.Installed && HashOf(row) is not null) panel.Children.Add(Line("✓ Approved, held to its hash.", tone: "LunaSuccess"));
            else if (row.IsLibretro && !row.Installed) panel.Children.Add(Line("Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it.", tone: "LunaMuted"));
            if (info is { } d2 && d2.TryGetProperty("tested", out JsonElement t))
                panel.Children.Add(Line(Text(t, "match") switch
                {
                    "exact" => $"This exact build was tested by EmuSen on {Text(t, "date")}.",
                    "other" => $"An earlier build was tested by EmuSen on {Text(t, "date")}.",
                    _ => "Not tested by EmuSen.",
                }, tone: "LunaMuted"));
            if (row.IsDefault && row.Consoles.Count > 0) panel.Children.Add(Line($"● The {row.Consoles[0]}'s engine: every {row.Consoles[0]} game opens on it unless the game's own options choose another."));
            else if (row.State == "update") panel.Children.Add(Line("A newer build is offered; the build installed now is kept for Roll Back."));
            if (info is { } d3 && d3.TryGetProperty("previous", out JsonElement p)) panel.Children.Add(Line($"The build of {Text(p, "build")} is kept for Roll Back."));
            if (row.IsLibretro && row.Engine is null && row.Installed && !row.AwaitsApproval) panel.Children.Add(Line("It becomes a choice for its consoles once EmuSen's runtime lists it.", tone: "LunaMuted"));
            if (OutcomeOf(row) is { } outcome) panel.Children.Add(Line(char.ToUpperInvariant(outcome[0]) + outcome[1..] + ".", tone: outcome.StartsWith("refused", StringComparison.Ordinal) ? "LunaError" : "LunaMuted"));
            return panel;
        }

        // "a PlayStation 2 BIOS", "an Atari BIOS": the firmware's description with its article.
        private static string Article(string words) => (words.Length > 0 && "aeiouAEIOU".Contains(words[0]) ? "an " : "a ") + words;

        // The Details page: every fact as the tool holds it (EmuSen_Interface.md §14.5).
        private Control DetailsOf(CoreRow row)
        {
            var panel = new StackPanel { Spacing = 6, Name = "CoresDetails" };
            panel.Children.Add(new TextBlock { Text = $"{row.Name} · details", TextWrapping = TextWrapping.Wrap, Classes = { AdaptiveLayout.PaneTitleClass } });
            if (row.ListedAs is { } listed) panel.Children.Add(Line($"Listed by libretro as {listed}"));
            if (row.Stem is { } stem) panel.Children.Add(Line($"File stem {stem}"));
            if (row.Engine is { } engine) panel.Children.Add(Line($"Engine {engine}"));
            if (row.Library is { } library) panel.Children.Add(Line($"Library {library}"));
            JsonElement? info = row.Stem is { } st && _info.TryGetValue(st, out JsonElement i) ? i : null;
            if (info is { } d)
            {
                if (Text(d, "from") is { } from) panel.Children.Add(Line($"From {from}, index CRC-32 {Text(d, "index_crc32")}"));
                if (Text(d, "fetched") is { } fetched) panel.Children.Add(Line($"Downloaded {fetched}"));
                foreach (JsonElement f in Firmware(row))
                    panel.Children.Add(Line($"Firmware {Text(f, "desc")} at {Text(f, "path")}, {(f.TryGetProperty("optional", out JsonElement o) && o.ValueKind == JsonValueKind.True ? "optional" : "required")}"));
                if (Strings(d, "notes") is { Count: > 0 } notes) panel.Children.Add(Line("Notes: " + string.Join(" · ", notes)));
                if (Strings(d, "permissions") is { Count: > 0 } perms) panel.Children.Add(Line("Asks for: " + string.Join(", ", perms)));
            }
            if (HashOf(row) is { } sha) panel.Children.Add(Line($"SHA-256 {string.Join(" ", Enumerable.Range(0, sha.Length / 8).Select(k => sha.Substring(k * 8, 8)))}{(row.AwaitsApproval ? " · not yet approved" : "")}", "CoresDetailHash"));
            if (row.Trust is { } trust) panel.Children.Add(Line($"Trust {trust}"));
            return panel;
        }

        private static string? FolderOf(CoreRow row) => row.IsLibretro && row.Stem is { } s ? Path.Combine(DataStore.Cores, "libretro", s) : row.Library is { } l ? Path.GetDirectoryName(l) : null;

        private static ScrollViewer Scrolled(Control content) => new() { Content = content, HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled };

        private static long Number(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.Number && v.TryGetInt64(out long n) ? n : 0;

        private static (long Free, long Total)? DiskOf(string path) => DiskSpace.Under(path);
    }
}
