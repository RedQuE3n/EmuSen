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
        private readonly Dictionary<string, IReadOnlyList<SlangParameter>> _read = new(StringComparer.Ordinal);

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

        // Read off the UI thread once per preset; a Mega Bezel preset declares hundreds.
        internal Task<IReadOnlyList<SlangParameter>> ReadAsync(string relative)
        {
            string path = Path.GetFullPath(Path.Combine(Pack, relative));
            lock (_read)
                if (_read.TryGetValue(path, out var known)) return Task.FromResult(known);
            return Task.Run(() =>
            {
                IReadOnlyList<SlangParameter> parameters = SlangParameters.Read(SlangPreset.Load(path));
                lock (_read) _read[path] = parameters;
                return parameters;
            });
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
                lock (_read) _read.Clear();
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
        private readonly StackPanel _parameters = new() { Spacing = 4 };
        private readonly HintText _parametersNote = new();
        private readonly IReadOnlyList<ShaderEntry> _builtIns;
        private int _reading;

        public string Console { get; }

        // The shader whose details are shown, which Use applies whether or not the list still shows it.
        public ShaderEntry? Shown { get; private set; }

        // Completes once the shown shader's sliders are built, for a test.
        public Task Reading { get; private set; } = Task.CompletedTask;

        public GroupedList<ShaderEntry> List => _list;

        public FilterBar Filter => _filter;

        public IEnumerable<SliderRow> Sliders => _parameters.Children.OfType<SliderRow>();

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

            _filter.Changed += Refresh;
            _list.Chose += entry => { if (entry is not null) Show(entry); };
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
            var header = Ui.Stack(6, _name, _where, actions, _parametersNote);
            header.Margin = new Thickness(0, 0, 0, 8);
            var scroll = new ScrollViewer { Name = $"{console}.ShaderParameters", Content = _parameters, HorizontalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Disabled, Padding = new Thickness(0, 0, 14, 0) };
            var right = new DockPanel { LastChildFill = true };
            DockPanel.SetDock(header, Dock.Top);
            right.Children.Add(header);
            right.Children.Add(scroll);

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
            Show(current);
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

        // Applies the shown shader to this console, marks it and remembers it among the recent ones.
        public void Use()
        {
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

        private void Show(ShaderEntry entry)
        {
            entry = entry with { Recent = false, Group = ShaderCatalog.For(entry.Stored).Group };
            if (Shown == entry) return;
            Shown = entry;
            _name.Text = entry.Name;
            _where.Text = entry.IsPreset
                ? $"{entry.Relative}\nin {Path.GetFullPath(_owner.Pack)}"
                : ScreenFilters.Find(entry.Stored).Filter?.Credit is { Length: > 0 } credit ? $"{ShaderCatalog.BuiltIn}. {credit}" : ShaderCatalog.BuiltIn;
            _parameters.Children.Clear();
            ShowState();

            int reading = ++_reading;
            if (!entry.IsPreset)
            {
                BuildSliders(entry, ScreenFilters.Find(entry.Stored).Filter?.Parameters ?? Array.Empty<SlangParameter>());
                Reading = Task.CompletedTask;
                return;
            }
            if (!File.Exists(Path.Combine(_owner.Pack, entry.Relative!)))
            {
                _parametersNote.Text = "This preset is not in the downloaded pack, so it is drawn plain until the pack has it again.";
                Reading = Task.CompletedTask;
                return;
            }

            _parametersNote.Text = "Reading the preset's parameters...";
            var done = new TaskCompletionSource();
            Reading = done.Task;
            _owner.ReadAsync(entry.Relative!).ContinueWith(read => Dispatcher.UIThread.Post(() =>
            {
                if (reading == _reading)
                {
                    if (read.IsCompletedSuccessfully) BuildSliders(entry, read.Result);
                    else _parametersNote.Text = $"Its parameters could not be read: {read.Exception?.GetBaseException().Message}";
                }
                done.TrySetResult();
            }), TaskScheduler.Default);
        }

        // A slider per parameter over its declared range, the preset's own value as the default; a zero-width one is a heading.
        private void BuildSliders(ShaderEntry entry, IReadOnlyList<SlangParameter> parameters)
        {
            IReadOnlyDictionary<string, string> stored = Config.ParametersFor(Console, entry.Stored);
            int count = parameters.Count(p => !SlangParameters.IsHeading(p));
            _parametersNote.Text = count == 0 ? "This shader has nothing to adjust." : $"{count} {(count == 1 ? "parameter" : "parameters")}. Left and right move a slider; the button above one returns it to its default.";

            foreach (SlangParameter parameter in parameters)
            {
                if (SlangParameters.IsHeading(parameter))
                {
                    if (!string.IsNullOrWhiteSpace(parameter.Description.Trim(' ', '-', '=', '*', '#', '[', ']')))
                        _parameters.Children.Add(new SectionHeader { Text = parameter.Description.Trim(), Margin = new Thickness(0, 10, 0, 0), TextWrapping = TextWrapping.Wrap });
                    continue;
                }

                var row = new SliderRow
                {
                    Name = $"{Console}.Parameter.{parameter.Id}",
                    Label = string.IsNullOrWhiteSpace(parameter.Description) ? parameter.Id : parameter.Description.Trim(),
                    Minimum = Exact(parameter.Minimum),
                    Maximum = Exact(parameter.Maximum),
                    Step = Exact(parameter.Step),
                    DefaultValue = Exact(parameter.Initial),
                    Value = Exact(stored.TryGetValue(parameter.Id, out string? text) && float.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out float value) ? value : parameter.Initial),
                };
                string id = parameter.Id;
                row.ValueChanged += _ => Changed(entry, id, row);
                _parameters.Children.Add(row);
            }
        }

        // A float as the decimal it was written as, so 0.041f is 0.041 and not 0.041000001, which the row would show to four places.
        private static double Exact(float value) => (double)(decimal)value;

        private void Changed(ShaderEntry entry, string id, SliderRow row)
        {
            if (row.IsDefault) Config.ForgetParameter(Console, entry.Stored, id);
            else Config.SetParameter(Console, entry.Stored, id, ((float)Math.Round(row.Value, 6)).ToString(CultureInfo.InvariantCulture));
            Config.Save();
            ShowState();
            if (entry.Stored == Current) _owner.Changed(Console);
        }

        private void ResetAll()
        {
            if (Shown is not { } entry) return;
            Config.ForgetParameter(Console, entry.Stored);
            Config.Save();
            foreach (SliderRow row in Sliders) row.Value = row.DefaultValue;
            ShowState();
            if (entry.Stored == Current) _owner.Changed(Console);
        }
    }
}
