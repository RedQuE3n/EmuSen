using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Layout;
using Avalonia.LogicalTree;
using Avalonia.Media;
using Avalonia.Threading;
using EmuSen.Cores;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;

namespace EmuSen.Mistress.Views
{
    // One tab per console: the built-in filters and RetroArch's presets in one searchable list, and the chosen one's parameters as sliders - see EmuSen_Settings_Reference.md §4.48.
    public class ShaderSettingsWindow : ToolWindow
    {
        private readonly GraphicsConfig _config;
        private readonly Action<string>? _changed;
        private readonly Func<HttpClient> _http;
        private readonly Tabs _tabs = new() { Name = "ConsoleTabs" };
        private readonly List<ShaderPanel> _panels = new();
        private readonly RecentCache<string, IReadOnlyList<SlangParameter>> _read = new(CacheLimit, StringComparer.Ordinal);

        internal string Pack { get; }
        internal IReadOnlyList<ShaderEntry> Presets { get; private set; } = Array.Empty<ShaderEntry>();
        internal string? Built { get; private set; }

        // Tells a test when the download it started has finished.
        public Task? Downloading { get; private set; }

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public ShaderSettingsWindow() : this(new GraphicsConfig(), null, null) { }

        public ShaderSettingsWindow(GraphicsConfig config, Action<string>? changed, string? selectedConsole,
            Func<HttpClient>? http = null, string? pack = null)
        {
            _config = config;
            _changed = changed;
            _http = http ?? (() => new HttpClient());
            Pack = pack ?? SlangPackDownload.DefaultDirectory;

            Title = "Shaders";
            Width = 980;
            Height = 660;
            MinWidth = 760;
            MinHeight = 480;
            CanResize = true;

            LoadPack();
            var consoles = CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console).ToList();
            foreach (string console in consoles)
            {
                var panel = new ShaderPanel(this, console);
                _panels.Add(panel);
                _tabs.Add(console, panel);
            }

            int selected = selectedConsole is null ? -1 : consoles.IndexOf(selectedConsole);
            if (selected >= 0) _tabs.SelectedIndex = selected;

            Control hint = Ui.Hint("Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use.");
            Control buttons = Ui.Buttons(Ui.Button("Close", Close));
            hint.Margin = new Thickness(0, 0, 0, 10);
            buttons.Margin = new Thickness(0, 10, 0, 0);
            DockPanel.SetDock(hint, Dock.Top);
            DockPanel.SetDock(buttons, Dock.Bottom);
            var dock = new DockPanel { Margin = new Thickness(16), LastChildFill = true };
            dock.Children.Add(hint);
            dock.Children.Add(buttons);
            dock.Children.Add(_tabs);
            Content = dock;
        }

        public ShaderPanel PanelFor(string console) => _panels.Single(p => p.Console == console);

        internal GraphicsConfig Config => _config;

        internal void Changed(string console) => _changed?.Invoke(console);

        // A label for a stored value, for the Graphics window's row and the status line.
        public static string Describe(string? stored)
        {
            ShaderEntry entry = ShaderCatalog.For(stored);
            return entry.IsPreset ? $"{entry.Name} (RetroArch, {entry.Group})" : entry.Name;
        }

        private void LoadPack()
        {
            Presets = ShaderCatalog.Presets(SlangPackDownload.Presets(Pack));
            Built = SlangPackDownload.Installed(Pack);
        }

        // How long the list's selection must hold still before the shown preset is read and its sliders built - see EmuSen_Settings_Reference.md §4.48.9.
        public TimeSpan Settle { get; set; } = TimeSpan.FromMilliseconds(120);

        // The clock the settle is timed by; a test gives its own, so no settle depends on how fast the machine runs - see EmuSen_Settings_Reference.md §4.48.10.
        public TimeProvider Time { get; set; } = TimeProvider.System;

        // Background reads of the rows beside a settled one, at most this many at once - see EmuSen_Settings_Reference.md §4.48.9.
        public const int PrefetchLimit = 2;

        // The presets' parameters kept, the most recently used; a read ahead takes a place like any other - see EmuSen_Settings_Reference.md §4.48.10.
        public const int CacheLimit = 8;

        // Counters a test reads: sources read from disk, the reads of them that were prefetches, and what the cache holds and dropped.
        public int ReadsStarted { get; private set; }
        public int PrefetchesStarted { get; private set; }
        public int PrefetchesRunning { get { lock (_read) return _prefetching; } }
        public int Cached { get { lock (_read) return _read.Count; } }
        public int Evicted { get { lock (_read) return _read.Evicted; } }
        public int EvictedUnused { get { lock (_read) return _read.EvictedUnused; } }

        private readonly Dictionary<string, (Task<IReadOnlyList<SlangParameter>> Task, bool Wanted)> _inFlight = new(StringComparer.Ordinal);
        private int _prefetching, _packGeneration;

        private string FullPath(string relative) => Path.GetFullPath(Path.Combine(Pack, relative));

        public bool IsRead(string relative)
        {
            lock (_read) return _read.Contains(FullPath(relative));
        }

        // Read off the UI thread once per preset, a read already running shared rather than repeated; a Mega Bezel preset declares hundreds.
        internal Task<IReadOnlyList<SlangParameter>> ReadAsync(string relative, bool ahead = false)
        {
            string path = FullPath(relative);
            lock (_read)
            {
                if (_read.TryGet(path, out var known)) return Task.FromResult(known);
                if (_inFlight.TryGetValue(path, out var running))
                {
                    if (!ahead) _inFlight[path] = running with { Wanted = true };
                    return running.Task;
                }
                int generation = _packGeneration;
                ReadsStarted++;
                // Whoever waits on the read finds it in the cache, since the task handed out ends only once it is stored.
                Task<IReadOnlyList<SlangParameter>> task = Task.Run(() => SlangParameters.Read(SlangPreset.Load(path))).ContinueWith(done =>
                {
                    lock (_read)
                    {
                        if (generation == _packGeneration && _inFlight.Remove(path, out var ended) && done.IsCompletedSuccessfully)
                            _read.Add(path, done.Result, used: ended.Wanted);
                    }
                    return done.Result;
                }, TaskScheduler.Default);
                _inFlight[path] = (task, !ahead);
                return task;
            }
        }

        // Reads a preset into the cache in the background unless it is there, being read, or the limit is reached - see EmuSen_Settings_Reference.md §4.48.9.
        internal void Prefetch(string relative)
        {
            string path = FullPath(relative);
            lock (_read)
            {
                if (_read.Contains(path) || _inFlight.ContainsKey(path) || _prefetching >= PrefetchLimit || !File.Exists(path)) return;
                _prefetching++;
                PrefetchesStarted++;
            }
            ReadAsync(relative, ahead: true).ContinueWith(_ => { lock (_read) _prefetching--; }, TaskScheduler.Default);
        }

        internal void Download() => Downloading = DownloadAsync();

        private async Task DownloadAsync()
        {
            foreach (ShaderPanel panel in _panels) panel.ShowPack("Downloading libretro's slang pack...", busy: true);
            var progress = new Progress<(long Read, long? Total)>(p =>
            {
                string text = p.Total is long total
                    ? $"Downloading... {p.Read / 1048576.0:F1} of {total / 1048576.0:F1} MB"
                    : $"Downloading... {p.Read / 1048576.0:F1} MB";
                foreach (ShaderPanel panel in _panels) panel.ShowPack(text, busy: true);
            });
            string? failure = null;
            try
            {
                // Its own client with a long timeout: 54 MB outlasts the thirty seconds a cover lookup is given.
                using HttpClient http = _http();
                http.Timeout = TimeSpan.FromMinutes(30);
                await SlangPackDownload.FetchAsync(http, Pack, progress);
                lock (_read)
                {
                    _read.Clear();
                    _inFlight.Clear();
                    _packGeneration++;
                }
            }
            catch (Exception ex) when (ex is HttpRequestException or IOException or InvalidDataException or TaskCanceledException or UnauthorizedAccessException)
            {
                failure = $"Could not download the pack: {ex.Message}";
            }
            LoadPack();
            foreach (ShaderPanel panel in _panels) panel.Reload(failure);
        }
    }

    // One console's tab: search and list on the left, the shader shown and its sliders on the right - see EmuSen_Settings_Reference.md §4.48.
    public sealed class ShaderPanel : Grid
    {
        private readonly ShaderSettingsWindow _owner;
        private readonly FilterBar _filter;
        private readonly GroupedList<ShaderEntry> _list;
        private readonly TextBlock _packStatus;
        private readonly TextBlock _packHeading = new() { Text = "RetroArch's shaders", FontWeight = FontWeight.SemiBold };
        private readonly Grid _left;
        private readonly Button _download;
        private readonly TextBlock _name = new() { FontSize = 20, FontWeight = FontWeight.SemiBold, TextWrapping = TextWrapping.Wrap };
        private readonly MonoText _where = new() { TextWrapping = TextWrapping.Wrap };
        private readonly Button _use;
        private readonly Button _resetAll;
        private readonly SliderList _parameters;
        private readonly HintText _parametersNote = new();
        private readonly FilterBar _parameterSearch;
        private readonly IReadOnlyList<ShaderEntry> _builtIns;
        private int _reading;
        private ShaderEntry? _builtFor;
        private (ShaderEntry Entry, int Reading, TaskCompletionSource Done)? _settling;
        private bool _keying;
        private long? _lastKeyMove;
        private System.Threading.ITimer? _settleTimer;

        public string Console { get; }

        // The shader whose details are shown, which Use applies whether or not the list still shows it.
        public ShaderEntry? Shown { get; private set; }

        // Completes once the shown shader's sliders are built, for a test.
        public Task Reading { get; private set; } = Task.CompletedTask;

        public GroupedList<ShaderEntry> List => _list;

        public FilterBar Filter => _filter;

        public FilterBar ParameterSearch => _parameterSearch;

        // The rows that exist now; the list is virtualised, so a long preset's are only those in view and a few beyond.
        public IEnumerable<SliderRow> Sliders => _parameters.Realized;

        // Every parameter of the shader shown, realised or not.
        public IReadOnlyList<SliderItem> Parameters => _parameters.Sliders.ToList();

        public SliderList ParameterList => _parameters;

        // Settled reads and builds, one per stop of the selection, for a test.
        public int Loads { get; private set; }

        internal ShaderPanel(ShaderSettingsWindow owner, string console)
        {
            _owner = owner;
            Console = console;
            _builtIns = ShaderCatalog.BuiltIns(console);

            _filter = new FilterBar { Name = $"{console}.ShaderSearch", Placeholder = "Search", FacetLabel = "Category", ShowFacet = true, SearchDelay = TimeSpan.Zero };
            _list = new GroupedList<ShaderEntry>
            {
                Name = $"{console}.ShaderList",
                Group = e => e.Group,
                Label = e => e.Name,
                Detail = e => e.Recent ? (e.IsPreset ? e.Relative : ShaderCatalog.BuiltIn) : null,
                Badge = e => e.Stored == Current ? "In use" : null,
                Key = e => (e.Recent ? "recent:" : "") + e.Stored,
            };
            _packStatus = new TextBlock { Name = $"{console}.PackStatus", TextWrapping = TextWrapping.Wrap, VerticalAlignment = VerticalAlignment.Center };
            _download = Ui.Button("Download Pack", owner.Download);
            _download.Name = $"{console}.DownloadPack";
            _use = Ui.Button("Use This Shader", () => Use());
            _use.Name = $"{console}.UseShader";
            _resetAll = Ui.Button("Reset All", ResetAll);
            _resetAll.Name = $"{console}.ResetShader";
            _name.Name = $"{console}.ShaderName";
            _where.Name = $"{console}.ShaderPath";

            _parameters = new SliderList { Name = $"{console}.ShaderParameters", Padding = new Thickness(0, 0, 14, 0) };
            _parameters.ValueChanged += Changed;
            // Kept across shaders, so a word looked for is looked for in the next preset too - see EmuSen_Settings_Reference.md §4.48.10.
            _parameterSearch = new FilterBar { Name = $"{console}.ParameterSearch", Placeholder = "Search parameters", ShowFacet = false, SearchDelay = TimeSpan.Zero, IsVisible = false, Margin = new Thickness(0, 2, 14, 0) };
            _parameterSearch.Changed += () =>
            {
                _parameters.Search = _parameterSearch.SearchText;
                ShowCount();
            };

            _filter.Changed += Refresh;
            // Only a key move that follows another inside the settle waits; a click, a jump, a first step and the focus leaving the list do not - see EmuSen_Settings_Reference.md §4.48.10.
            _list.Chose += entry => { if (entry is not null) Show(entry, now: !_keying || FirstStep()); };
            _list.AddHandler(KeyDownEvent, (_, _) => _keying = true, Avalonia.Interactivity.RoutingStrategies.Tunnel, handledEventsToo: true);
            _list.AddHandler(KeyDownEvent, (_, _) => _keying = false, Avalonia.Interactivity.RoutingStrategies.Bubble, handledEventsToo: true);
            _list.PropertyChanged += (_, e) => { if (e.Property == IsKeyboardFocusWithinProperty && e.NewValue is false) SettleNow(); };
            _list.DoubleTapped += (_, _) => Use();
            // Handled events too: the list claims Enter itself, and the pad's A is sent as Enter - see EmuSen_Settings_Reference.md §4.45.3.
            _list.AddHandler(KeyDownEvent, (_, e) => { if (e.Key == Key.Enter) Use(); }, handledEventsToo: true);

            var packRow = new DockPanel { LastChildFill = true, Margin = new Thickness(0, 10, 0, 0) };
            DockPanel.SetDock(_download, Dock.Right);
            _download.Margin = new Thickness(12, 0, 0, 0);
            _download.VerticalAlignment = VerticalAlignment.Top;
            packRow.Children.Add(_download);
            packRow.Children.Add(Ui.Stack(4, _packHeading, _packStatus));

            // Without a pack the list is only the built-ins, and the download sits right under them, where the presets would be.
            _left = new Grid { RowDefinitions = new RowDefinitions("Auto,*,Auto") };
            _filter.Margin = new Thickness(0, 0, 0, 8);
            SetRow(_list, 1);
            SetRow(packRow, 2);
            _left.Children.Add(_filter);
            _left.Children.Add(_list);
            _left.Children.Add(packRow);

            // Right-aligned, over the column of Reset buttons, so up from the first slider's Reset reaches Use - see EmuSen_Settings_Reference.md §4.48.5.
            var actions = Ui.Row(8, _resetAll, _use);
            actions.HorizontalAlignment = HorizontalAlignment.Right;
            actions.Margin = new Thickness(0, 0, 14, 0);
            var header = Ui.Stack(6, _name, _where, actions, _parametersNote, _parameterSearch);
            header.Margin = new Thickness(0, 0, 0, 8);
            var right = new DockPanel { LastChildFill = true };
            DockPanel.SetDock(header, Dock.Top);
            right.Children.Add(header);
            right.Children.Add(_parameters);

            ColumnDefinitions = new ColumnDefinitions("5*,20,6*");
            Margin = new Thickness(0, 8, 0, 0);
            SetColumn(_left, 0);
            SetColumn(right, 2);
            Children.Add(_left);
            Children.Add(right);

            Reload(null);
        }

        private GraphicsConfig Config => _owner.Config;

        public string Current => Config.Value(Console, GraphicsSettingsWindow.ScreenFilterKey) ?? ScreenFilters.None;

        // The pack changed, or the window opened: the categories, the list and the pack's line again.
        internal void Reload(string? failure)
        {
            object? facet = _filter.Facet;
            IReadOnlyList<string> categories = ShaderCatalog.Categories(_owner.Presets);
            _filter.SetFacets(categories, facet is string f && categories.Contains(f) ? f : ShaderCatalog.AllCategories);
            Refresh();
            ShowPack(failure ?? (_owner.Built is null
                ? "Not downloaded. libretro's slang pack, about 54 MB from buildbot.libretro.com, is the file RetroArch's updater fetches: some 2,600 presets. EmuSen ships none of it, and each shader keeps its authors' licence."
                : $"RetroArch: {_owner.Presets.Count:N0} presets, pack built {_owner.Built}."), busy: false);
            _download.Content = _owner.Built is null ? "Download Pack" : "Update Pack";
            bool missing = _owner.Built is null;
            _packHeading.IsVisible = missing;
            _left.RowDefinitions = new RowDefinitions(missing ? "Auto,Auto,*" : "Auto,*,Auto");

            ShaderEntry current = ShaderCatalog.For(Current);
            _list.Select(current);
            if (failure is not null) return;
            Shown = null;
            Show(current, now: true);
        }

        internal void ShowPack(string text, bool busy)
        {
            _packStatus.Text = text;
            _download.IsEnabled = !busy;
        }

        private void Refresh()
        {
            _list.Refresh(ShaderCatalog.Shown(_builtIns, _owner.Presets, Config.RecentFor(Console), _filter.SearchText, _filter.Facet as string ?? ShaderCatalog.AllCategories));
        }

        // Applies the shown shader to this console, marks it and remembers it among the recent ones; its sliders need not be shown yet.
        public void Use()
        {
            SettleNow();
            if (Shown is not { } entry || entry.Stored == Current) return;
            Config.SetValue(Console, GraphicsSettingsWindow.ScreenFilterKey, entry.Stored);
            if (entry.Stored != ScreenFilters.None) Config.NoteRecent(Console, entry.Stored, ShaderCatalog.RecentKept);
            Config.Save();
            _owner.Changed(Console);
            Refresh();
            ShowState();
        }

        private void ShowState()
        {
            bool inUse = Shown?.Stored == Current;
            _use.Content = inUse ? "In Use" : "Use This Shader";
            bool focused = _use.IsFocused || _resetAll.IsFocused;
            _use.IsEnabled = !inUse;
            _resetAll.IsEnabled = Shown is { } entry && Config.ParametersFor(Console, entry.Stored).Count > 0;
            if (focused && !_use.IsFocused && !_resetAll.IsFocused) FocusNearby();
        }

        // A button disabled under the focus leaves it nowhere, and a pad can then move nothing: the next thing along takes it - see EmuSen_Settings_Reference.md §4.48.5.
        private void FocusNearby()
        {
            if (_use.IsEnabled && _use.Focus(NavigationMethod.Directional)) return;
            if (_resetAll.IsEnabled && _resetAll.Focus(NavigationMethod.Directional)) return;
            if (Sliders.SelectMany(row => row.GetLogicalDescendants().OfType<Slider>()).FirstOrDefault(slider => slider.IsEffectivelyEnabled) is { } first && first.Focus(NavigationMethod.Directional)) return;
            if (_list.SelectedIndex >= 0 && _list.ContainerFromIndex(_list.SelectedIndex) is { } row && row.Focus(NavigationMethod.Directional)) return;
            _list.Focus(NavigationMethod.Directional);
        }

        // The name and path at once; the read and the sliders once the selection has held still, or at once when asked - see EmuSen_Settings_Reference.md §4.48.9.
        private void Show(ShaderEntry entry, bool now = false)
        {
            entry = entry with { Recent = false, Group = ShaderCatalog.For(entry.Stored).Group };
            if (Shown == entry)
            {
                if (now) SettleNow();
                return;
            }
            Shown = entry;
            _name.Text = entry.Name;
            _where.Text = entry.IsPreset
                ? $"{entry.Relative}\nin {Path.GetFullPath(_owner.Pack)}"
                : ScreenFilters.Find(entry.Stored).Filter?.Credit is { Length: > 0 } credit ? $"{ShaderCatalog.BuiltIn}. {credit}" : ShaderCatalog.BuiltIn;
            _builtFor = null;
            _parameters.ItemsSource = null;
            _parameterSearch.IsVisible = false;
            _parametersNote.Text = entry.IsPreset ? "Reading the preset's parameters..." : string.Empty;
            ShowState();

            int reading = ++_reading;
            _settling?.Done.TrySetResult();
            var done = new TaskCompletionSource();
            Reading = done.Task;
            _settling = (entry, reading, done);
            _settleTimer?.Dispose();
            if (now) { SettleNow(); return; }
            _settleTimer = _owner.Time.CreateTimer(_ => Dispatcher.UIThread.Post(() =>
            {
                if (_settling is { } settling && settling.Reading == reading) SettleNow();
            }), null, _owner.Settle, System.Threading.Timeout.InfiniteTimeSpan);
        }

        // A key move after the selection has held still for the settle reads at once; only the moves that follow inside it wait - see EmuSen_Settings_Reference.md §4.48.10.
        private bool FirstStep()
        {
            long now = _owner.Time.GetTimestamp();
            bool first = _lastKeyMove is not long last || _owner.Time.GetElapsedTime(last, now) >= _owner.Settle;
            _lastKeyMove = now;
            return first;
        }

        // Ends a settle still waiting: the shown shader is read and its sliders built now.
        private void SettleNow()
        {
            if (_settling is not { } settling) return;
            _settling = null;
            Load(settling.Entry, settling.Reading, settling.Done);
        }

        private void Load(ShaderEntry entry, int reading, TaskCompletionSource done)
        {
            Loads++;
            if (!entry.IsPreset)
            {
                BuildSliders(entry, ScreenFilters.Find(entry.Stored).Filter?.Parameters ?? Array.Empty<SlangParameter>());
                done.TrySetResult();
                PrefetchBeside(entry);
                return;
            }
            if (!File.Exists(Path.Combine(_owner.Pack, entry.Relative!)))
            {
                _parametersNote.Text = "This preset is not in the downloaded pack, so it is drawn plain until the pack has it again.";
                done.TrySetResult();
                return;
            }

            void Arrived(Task<IReadOnlyList<SlangParameter>> read)
            {
                if (reading == _reading)
                {
                    if (read.IsCompletedSuccessfully) BuildSliders(entry, read.Result);
                    else _parametersNote.Text = $"Its parameters could not be read: {read.Exception?.GetBaseException().Message}";
                }
                done.TrySetResult();
                if (reading == _reading) PrefetchBeside(entry);
            }

            // Parameters already in the cache are built now, in the same step, not a dispatcher turn later - see EmuSen_Settings_Reference.md §4.48.10.
            Task<IReadOnlyList<SlangParameter>> pending = _owner.ReadAsync(entry.Relative!);
            if (pending.IsCompleted) Arrived(pending);
            else pending.ContinueWith(read => Dispatcher.UIThread.Post(() => Arrived(read)), TaskScheduler.Default);
        }

        // The presets a row away from the settled one, so the usual next step finds its parameters read - see EmuSen_Settings_Reference.md §4.48.9.
        private void PrefetchBeside(ShaderEntry entry)
        {
            int at = _list.SelectedIndex;
            if (at < 0 || _list.Selected is not { } selected || selected.Stored != entry.Stored) return;
            foreach (int beside in new[] { at + 1, at - 1 })
                if (beside >= 0 && beside < _list.Models.Count && _list.Models[beside] is { IsPreset: true, Relative: { } relative })
                    _owner.Prefetch(relative);
        }

        // A slider per parameter over its declared range, the preset's own value as the default; a zero-width one is a heading; only the rows in view are built.
        private void BuildSliders(ShaderEntry entry, IReadOnlyList<SlangParameter> parameters)
        {
            IReadOnlyDictionary<string, string> stored = Config.ParametersFor(Console, entry.Stored);
            int count = parameters.Count(p => !SlangParameters.IsHeading(p));

            var items = new List<object>(parameters.Count);
            foreach (SlangParameter parameter in parameters)
            {
                if (SlangParameters.IsHeading(parameter))
                {
                    if (!string.IsNullOrWhiteSpace(parameter.Description.Trim(' ', '-', '=', '*', '#', '[', ']')))
                        items.Add(parameter.Description.Trim());
                    continue;
                }

                items.Add(new SliderItem(
                    string.IsNullOrWhiteSpace(parameter.Description) ? parameter.Id : parameter.Description.Trim(),
                    Exact(parameter.Minimum),
                    Exact(parameter.Maximum),
                    Exact(parameter.Step),
                    Exact(parameter.Initial),
                    Exact(stored.TryGetValue(parameter.Id, out string? text) && float.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out float value) ? value : parameter.Initial))
                {
                    Name = $"{Console}.Parameter.{parameter.Id}",
                    Tag = parameter.Id,
                    Keywords = parameter.Id,
                });
            }
            _builtFor = entry;
            _parameters.ItemsSource = items;
            _parameterSearch.IsVisible = _parameters.Sliders.Any();
            ShowCount();
        }

        // How many parameters there are, and while a search narrows them, how many match - see EmuSen_Settings_Reference.md §4.48.10.
        private void ShowCount()
        {
            if (_builtFor is null) return;
            int count = _parameters.Sliders.Count(), matching = _parameters.Matching.Count();
            string noun = count == 1 ? "parameter" : "parameters";
            _parametersNote.Text = count == 0 ? "This shader has nothing to adjust."
                : _parameters.Search.Length > 0 ? $"{matching} of {count} {noun} match “{_parameters.Search}”. Left and right move a slider; the button above one returns it to its default."
                : $"{count} {noun}. Left and right move a slider; the button above one returns it to its default.";
        }

        // A float as the decimal it was written as, so 0.041f is 0.041 and not 0.041000001, which the row would show to four places.
        private static double Exact(float value) => (double)(decimal)value;

        private void Changed(SliderItem item)
        {
            if (_builtFor is not { } entry || item.Tag is not string id) return;
            if (item.IsDefault) Config.ForgetParameter(Console, entry.Stored, id);
            else Config.SetParameter(Console, entry.Stored, id, ((float)Math.Round(item.Value, 6)).ToString(CultureInfo.InvariantCulture));
            Config.Save();
            ShowState();
            if (entry.Stored == Current) _owner.Changed(Console);
        }

        private void ResetAll()
        {
            if (Shown is not { } entry) return;
            Config.ForgetParameter(Console, entry.Stored);
            Config.Save();
            foreach (SliderItem item in _parameters.Sliders) item.Value = item.DefaultValue;
            ShowState();
            if (entry.Stored == Current) _owner.Changed(Console);
        }
    }
}
