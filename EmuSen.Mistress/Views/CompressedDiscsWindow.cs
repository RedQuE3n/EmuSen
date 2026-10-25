using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Threading;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // One disc of the library as the CHD tool lists it, and what became of it in the last batch.
    public sealed class CompressedDisc
    {
        public required string Source { get; init; }
        public required string Console { get; init; }
        public string Folder => Source.Contains('/') ? Source[..Source.IndexOf('/')] : "";
        public string Name => Source.Contains('/') ? Source[(Source.IndexOf('/') + 1)..] : Source;
        public long Size { get; set; }
        public string State { get; set; } = "not converted";
        public string? Words { get; set; }
        public string? Checks { get; set; }
        public long Original { get; set; }
        public long Chd { get; set; }
        public double? Seconds { get; set; }
        public bool Chosen { get; set; }
    }

    // The library's PlayStation and Sega CD discs converted to CHD, checked again and listed, through the platform library's library tool; the library is only read - see EmuSen_GamePackages.md §13.
    public sealed class CompressedDiscsWindow : ToolWindow
    {
        // How a job is started, which a test replaces with a scripted one.
        internal static Func<string, IToolJob> StartJob = LibraryToolJob.Start;
        internal static Func<bool> ToolAvailable = () => LibraryToolJob.Available;
        internal static Func<int> Processors = () => Environment.ProcessorCount;

        public static readonly TimeSpan PollEvery = TimeSpan.FromMilliseconds(100);
        public const string EveryFolder = "Every folder";

        private readonly Dropdown _folder = new() { Name = "CompressedDiscsFolder", MinWidth = 260, HorizontalAlignment = HorizontalAlignment.Left };
        private readonly Dropdown _threads = new() { Name = "CompressedDiscsThreads", MinWidth = 160, HorizontalAlignment = HorizontalAlignment.Left };
        private readonly Button _convert, _check, _stop, _resume;
        private readonly ProgressBar _bar = new() { Name = "CompressedDiscsProgress", Minimum = 0, Maximum = 1, HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly TextBlock _progress = new() { Name = "CompressedDiscsProgressText", TextWrapping = TextWrapping.Wrap };
        private readonly HintText _log = new() { Name = "CompressedDiscsLog" };
        private readonly HintText _status = new() { Name = "CompressedDiscsStatus" };
        private readonly StackPanel _list = Ui.Stack(10);
        private readonly EmptyState _empty = new() { Name = "CompressedDiscsEmpty" };
        private readonly ContentControl _listHost = new();
        private readonly DispatcherTimer _timer;
        private readonly List<CompressedDisc> _discs = new();
        private readonly Dictionary<string, (CheckBox Box, HintText State, HintText Checks)> _rows = new(StringComparer.Ordinal);
        private readonly Dictionary<string, long> _counts = new(StringComparer.Ordinal);
        private Dictionary<string, string> _folders = new(StringComparer.Ordinal);
        private IToolJob? _job;
        private string _op = "";
        private string? _lastConvert;
        private string? _working;
        private int _of, _done;
        private bool _stopping, _filling;

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public CompressedDiscsWindow() : this(null) { }

        public CompressedDiscsWindow(string? library)
        {
            Title = "Compressed Discs";
            Width = 760;
            Height = 640;
            MinWidth = 560;
            MinHeight = 420;
            CanResize = true;

            _list.Name = "CompressedDiscsList";
            _convert = Act("Convert", "CompressedDiscsConvert", Convert);
            _check = Act("Check Converted", "CompressedDiscsCheck", Check);
            _stop = Act("Stop", "CompressedDiscsStop", Stop);
            _resume = Act("Resume", "CompressedDiscsResume", Resume);
            int total = Math.Max(1, Processors());
            string[] threads = Enumerable.Range(1, total).Select(n => $"{n} of {total}").ToArray();
            _threads.Fill(threads, threads[Math.Max(1, total / 2) - 1]);
            _folder.Chose += _ => { if (!_filling) ShowFolder(); };

            string where = string.IsNullOrWhiteSpace(library) ? "the library" : library;
            Control about = Ui.Hint($"Makes a CHD file of each PlayStation and Sega CD disc in the library, about half its size, and keeps it only when three checks pass: chdman's own, every track read back and compared with the original, and the table of contents. The library is left untouched: nothing in {where} is changed, moved or removed, and the originals stay where they are. The CHDs go to {DataStore.Compressed}, each with its record in {DataStore.Packages}. Closing this window stops a batch; converting again later does only what is left.");
            about.Name = "CompressedDiscsAbout";
            var actions = new WrapPanel { Orientation = Orientation.Horizontal, ItemSpacing = 8, LineSpacing = 8 };
            foreach (Button b in new[] { _convert, _check, _stop, _resume }) actions.Children.Add(b);
            _listHost.Content = _list;
            var body = Ui.Stack(16,
                about,
                new FieldRow { Label = "Discs from", Hint = "A folder of the library, or every folder.", Content = _folder },
                new FieldRow { Label = "Threads", Hint = "How many of this machine's threads chdman uses; half, the default, leaves the rest for everything else.", Content = _threads },
                new FieldRow { Label = "Convert or check", Hint = "The discs chosen below, or with none chosen every disc shown. Check Converted runs the three checks again on discs already converted.", Content = actions },
                new FieldRow { Label = "Progress", Content = Ui.Stack(6, _bar, _progress, _log) },
                new FieldRow { Label = "Discs", Hint = "Each disc's size and state; choose discs to convert or check only those.", Content = _listHost });
            body.Margin = new Thickness(0, 0, 4, 0);

            Button close = Ui.Button("Close", Close);
            close.Name = "CompressedDiscsClose";
            Control foot = Ui.Stack(8, _status, Ui.Buttons(close));
            foot.Margin = new Thickness(0, 10, 0, 0);
            DockPanel.SetDock(foot, Dock.Bottom);
            var dock = new DockPanel { Margin = new Thickness(16), LastChildFill = true };
            dock.Children.Add(foot);
            dock.Children.Add(new ScrollViewer { Content = body, HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled });
            Content = dock;

            _timer = new DispatcherTimer { Interval = PollEvery };
            _timer.Tick += (_, _) => Poll();
            Closed += (_, _) =>
            {
                _timer.Stop();
                _job?.Dispose();
                _job = null;
            };

            if (ToolAvailable()) Run("ls", JsonSerializer.Serialize(new { op = "ls" }));
            else
            {
                _status.Text = $"Converting discs needs EmuSen's platform library, which is not in use here ({LibraryToolJob.Report}).";
                Empty("No discs listed", "The CHD tool runs in EmuSen's platform library, and without it the discs cannot be listed or converted.");
            }
            Render();
        }

        // The discs listed, in the tool's order.
        public IReadOnlyList<CompressedDisc> Discs => _discs;

        // Whether a job is running.
        public bool Busy => _job is not null;

        // What the last batch or refusal said.
        public string? Status => _status.Text;

        // The log the last batch wrote to, as the tool named it.
        public string? LogPath { get; private set; }

        private static Button Act(string text, string name, Action act)
        {
            Button button = Ui.Button(text, act);
            button.Name = name;
            return button;
        }

        // The discs the folder chooser shows: every disc, or one folder's.
        private IEnumerable<CompressedDisc> Shown() => _folder.SelectedItem is string label && _folders.TryGetValue(label, out string? folder) ? _discs.Where(d => d.Folder == folder) : _discs;

        // What a batch names: the chosen discs shown, else the folder, else the whole library.
        private string[] Paths()
        {
            string[] chosen = Shown().Where(d => d.Chosen).Select(d => d.Source).ToArray();
            if (chosen.Length > 0) return chosen;
            return _folder.SelectedItem is string label && _folders.TryGetValue(label, out string? folder) ? [folder] : [];
        }

        private int Threads() => _threads.SelectedIndex + 1;

        private void Convert()
        {
            _lastConvert = JsonSerializer.Serialize(new { op = "convert", paths = Paths(), threads = Threads() });
            Run("convert", _lastConvert);
        }

        private void Check() => Run("check", JsonSerializer.Serialize(new { op = "check", paths = Paths() }));

        private void Resume()
        {
            if (_lastConvert is not null) Run("convert", _lastConvert);
        }

        private void Stop()
        {
            if (_job is null) return;
            _job.Stop();
            _stopping = true;
            _status.Text = "Stopping: chdman is ended and its unfinished file removed.";
            Render();
        }

        private void Run(string op, string request)
        {
            if (_job is not null) return;
            try { _job = StartJob(request); }
            catch (Exception e) when (e is FormatException or InvalidOperationException)
            {
                _status.Text = CompressedDiscsWords.Refusal(e.Message);
                Render();
                return;
            }
            _op = op;
            _of = _done = 0;
            _working = null;
            _stopping = false;
            _counts.Clear();
            if (op != "ls")
            {
                using JsonDocument asked = JsonDocument.Parse(request);
                string[] paths = asked.RootElement.GetProperty("paths").EnumerateArray().Select(p => p.GetString() ?? "").ToArray();
                foreach (CompressedDisc d in _discs.Where(d => paths.Length == 0 || paths.Any(p => d.Source == p || d.Source.StartsWith(p + "/", StringComparison.Ordinal))))
                {
                    d.State = "waiting";
                    d.Words = d.Checks = null;
                    d.Seconds = null;
                    RefreshRow(d);
                }
                _status.Text = op == "check" ? "Checking the converted discs again." : "Converting. The library is only read.";
            }
            else _status.Text = "Reading the library.";
            _timer.Start();
            Render();
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
                    _status.Text = CompressedDiscsWords.Refusal(e.Message);
                    break;
                }
                if (line is null) break;
                Apply(line);
            }
            if (_job is { Ended: true })
            {
                _job.Dispose();
                _job = null;
                _timer.Stop();
            }
            Render();
        }

        private static string? Text(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;

        private static long Number(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.Number && v.TryGetInt64(out long n) ? n : 0;

        // One event of §12.3 applied to the list.
        public void Apply(string line)
        {
            using JsonDocument doc = JsonDocument.Parse(line);
            JsonElement e = doc.RootElement;
            switch (Text(e, "event"))
            {
                case "start":
                    _of = (int)Number(e, "discs");
                    if (Text(e, "log") is { } log) LogPath = log;
                    if (_op == "ls") _discs.Clear();
                    if (e.TryGetProperty("refused", out JsonElement refused) && refused.ValueKind == JsonValueKind.Array && refused.GetArrayLength() > 0)
                        _status.Text = string.Join(" ", refused.EnumerateArray().Select(r => CompressedDiscsWords.Refusal(r.GetString() ?? "")));
                    break;
                case "disc":
                    string source = Text(e, "source") ?? "";
                    CompressedDisc? disc = _discs.FirstOrDefault(d => d.Source == source);
                    if (disc is null)
                    {
                        disc = new CompressedDisc { Source = source, Console = Text(e, "system") ?? "" };
                        _discs.Add(disc);
                    }
                    disc.State = Text(e, "state") ?? "";
                    disc.Words = Text(e, "words");
                    if (_op == "ls") disc.Size = Number(e, "size");
                    if (Number(e, "original") is > 0 and var original) disc.Original = original;
                    if (e.TryGetProperty("chd", out JsonElement chd) && chd.ValueKind == JsonValueKind.Number) disc.Chd = chd.GetInt64();
                    disc.Seconds = e.TryGetProperty("seconds", out JsonElement s) && s.ValueKind == JsonValueKind.Number ? s.GetDouble() : null;
                    disc.Checks = e.TryGetProperty("checks", out JsonElement checks) ? CompressedDiscsWords.Checks(checks) : null;
                    if (disc.State == "working") _working = disc.Source;
                    else if (_op != "ls")
                    {
                        _done++;
                        if (_working == disc.Source) _working = null;
                    }
                    if (_op != "ls") RefreshRow(disc);
                    break;
                case "end":
                    if (e.TryGetProperty("counts", out JsonElement counts) && counts.ValueKind == JsonValueKind.Object)
                        foreach (JsonProperty c in counts.EnumerateObject()) _counts[c.Name] = c.Value.TryGetInt64(out long n) ? n : 0;
                    _working = null;
                    if (Text(e, "error") is { } error)
                    {
                        _status.Text = CompressedDiscsWords.Refusal(error);
                        if (_op == "ls") Empty("No discs listed", _status.Text);
                    }
                    else if (_op == "ls") Listed();
                    else
                    {
                        foreach (CompressedDisc d in _discs.Where(d => d.State == "waiting"))
                        {
                            d.State = "stopped";
                            RefreshRow(d);
                        }
                        _status.Text = CompressedDiscsWords.Summary(_op, _counts, e.TryGetProperty("seconds", out JsonElement took) ? took.ToString() : "0");
                    }
                    break;
            }
        }

        // The listing ended: the folders offered and the rows built.
        private void Listed()
        {
            _status.Text = _discs.Count == 0 ? "" : $"{_discs.Count} disc{(_discs.Count == 1 ? "" : "s")} in the library.";
            var folders = new Dictionary<string, string>(StringComparer.Ordinal);
            foreach (IGrouping<string, CompressedDisc> g in _discs.GroupBy(d => d.Folder))
            {
                string consoles = string.Join(" and ", g.Select(d => d.Console).Distinct());
                string name = g.Key.Length == 0 ? "The library's own folder" : g.Key;
                folders[$"{name}: {g.Count()} {consoles} disc{(g.Count() == 1 ? "" : "s")}"] = g.Key;
            }
            _folders = folders;
            _filling = true;
            string[] labels = new[] { EveryFolder }.Concat(folders.Keys).ToArray();
            _folder.Fill(labels, EveryFolder);
            _filling = false;
            if (_discs.Count == 0) Empty("No discs to convert", "The library holds no PlayStation or Sega CD disc this tool converts: a cue sheet with its tracks, a lone data track, an ISO image, or a zip of one of them.");
            else ShowFolder();
        }

        private void Empty(string message, string detail)
        {
            _empty.Message = message;
            _empty.Detail = detail;
            _listHost.Content = _empty;
        }

        // The rows of the folder chosen, built again.
        private void ShowFolder()
        {
            _list.Children.Clear();
            _rows.Clear();
            int i = 0;
            foreach (CompressedDisc d in Shown())
            {
                var name = new TextBlock { Text = d.Name, TextWrapping = TextWrapping.Wrap };
                var box = new CheckBox { Name = $"CompressedDiscs.Disc.{i}", Content = name, IsChecked = d.Chosen, HorizontalAlignment = HorizontalAlignment.Stretch };
                box.IsCheckedChanged += (_, _) => { d.Chosen = box.IsChecked == true; Render(); };
                var state = new HintText { Name = $"CompressedDiscs.Disc.{i}.State" };
                var checks = new HintText { Name = $"CompressedDiscs.Disc.{i}.Checks" };
                _list.Children.Add(Ui.Stack(2, box, state, checks));
                _rows[d.Source] = (box, state, checks);
                RefreshRow(d);
                i++;
            }
            _listHost.Content = _list;
            Render();
        }

        private void RefreshRow(CompressedDisc d)
        {
            if (!_rows.TryGetValue(d.Source, out var row)) return;
            string size = d.Size > 0 ? CompressedDiscsWords.Size(d.Size) + "  ·  " : "";
            row.State.Text = size + CompressedDiscsWords.State(d, _op);
            row.Checks.Text = d.Checks ?? "";
            row.Checks.IsVisible = d.Checks is not null;
        }

        // The buttons, the progress and the log line as the job and the choice allow.
        private void Render()
        {
            bool running = _job is not null;
            bool batch = running && _op != "ls";
            bool any = _discs.Count > 0 && ToolAvailable();
            int chosen = Shown().Count(d => d.Chosen);
            int shown = Shown().Count();
            _convert.Content = chosen > 0 ? $"Convert {chosen} Chosen" : shown == _discs.Count ? "Convert Every Disc" : $"Convert All {shown}";
            _check.Content = chosen > 0 ? $"Check {chosen} Chosen" : "Check Converted";
            _convert.IsEnabled = _check.IsEnabled = any && !running;
            _folder.IsEnabled = _threads.IsEnabled = !running && any;
            foreach (var row in _rows.Values) row.Box.IsEnabled = !running;
            _stop.IsEnabled = batch && !_stopping;
            _resume.IsEnabled = !running && _lastConvert is not null && _counts.TryGetValue("stopped", out long stopped) && stopped > 0;

            _bar.Value = _of == 0 ? 0 : (double)_done / _of;
            _bar.IsVisible = _op is "convert" or "check";
            _progress.Text = ProgressText();
            _log.Text = LogPath is null ? "" : $"Each disc's result is added to the log, {LogPath}.";
            _log.IsVisible = LogPath is not null;
        }

        private string ProgressText()
        {
            if (_op is not ("convert" or "check")) return _job is not null ? "Reading the library..." : "No batch has run since this window opened.";
            string verb = _op == "check" ? "checked" : "done";
            if (_job is null) return $"{_done} of {_of} disc{(_of == 1 ? "" : "s")} {verb}.";
            string now = _working is { } w ? $" Now: {w}." : "";
            return $"{_done} of {_of} disc{(_of == 1 ? "" : "s")} {verb}.{now}";
        }
    }
}
