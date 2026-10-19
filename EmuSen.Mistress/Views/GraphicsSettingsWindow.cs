using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.Mistress.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // One tab per console, each the settings its core offers, saved as they change and applied to a running session - see EmuSen_Settings_Reference.md §4.26.
    public class GraphicsSettingsWindow : ToolWindow
    {
        private readonly GraphicsConfig _config;
        private readonly Action<string>? _changed;
        private readonly Tabs _tabs = new() { Name = "ConsoleTabs" };
        private readonly List<(string Console, Action Refresh)> _panels = new();
        private readonly List<(string Console, Action Show)> _notes = new();
        private bool _filling;

        // The keys each console's schema names, the runtime's since SR12: read and written through the frontend API; every other key is Mistress's, in graphics.json - see EmuSen_Backend.md §14.12.
        private readonly HashSet<(string Console, string Key)> _core = new();

        private string? Value(string console, string key) =>
            _core.Contains((console, key)) ? CoreSettings.Value(console, key) : _config.Value(console, key);

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public GraphicsSettingsWindow() : this(new GraphicsConfig(), null, null) { }

        private readonly Func<System.Net.Http.HttpClient> _http;
        private readonly string _pack;

        // The Shaders window the button opened last, for a test to drive.
        public ShaderSettingsWindow? ShaderWindow { get; private set; }

        // A console's line under its Overscan row, or null: what replaces the setting for the game now running.
        private Func<string, string?>? OverscanNote { get; }

        // A console's rows from the session running it, null for any other console: a running machine words the notes.
        private Func<string, IReadOnlyList<SettingRow>?>? RunningSettings { get; }

        // What a shader change calls instead of the constructor's callback, so a slider does not re-apply the core's settings.
        public Action<string>? ShadersChanged { get; set; }

        public GraphicsSettingsWindow(GraphicsConfig config, Action<string>? changed, string? selectedConsole,
            Func<System.Net.Http.HttpClient>? http = null, string? pack = null, Func<string, string?>? overscanNote = null,
            Func<string, IReadOnlyList<SettingRow>?>? runningSettings = null)
        {
            OverscanNote = overscanNote;
            RunningSettings = runningSettings;
            _config = config;
            _changed = changed;
            _http = http ?? (() => new System.Net.Http.HttpClient());
            _pack = pack ?? Library.SlangPackDownload.DefaultDirectory;

            Title = "Graphics Settings";
            Width = 680;
            Height = 560;
            MinWidth = 560;
            MinHeight = 400;
            CanResize = true;

            var consoles = RuntimeCatalog.ConsoleNames.ToList();
            foreach (string console in consoles)
            {
                _tabs.Add(console, new ScrollViewer { Content = BuildConsolePanel(console) });
            }

            int selected = selectedConsole is null ? -1 : consoles.IndexOf(selectedConsole);
            if (selected >= 0) _tabs.SelectedIndex = selected;

            // A dock, not a stack: a stack offers its children unbounded height, and a tab's scroll viewer then never scrolls - see §4.26.
            Control hint = Ui.Hint("What each console's core does with its picture and its machine. A change is saved at once and reaches a running game between frames unless its hint says otherwise; a game loaded later reads these too.");
            Control buttons = Ui.Buttons(
                Ui.Button("Reset This Console", ResetSelected),
                Ui.Button("Close", Close));
            hint.Margin = new Avalonia.Thickness(0, 0, 0, 10);
            buttons.Margin = new Avalonia.Thickness(0, 10, 0, 0);
            DockPanel.SetDock(hint, Dock.Top);
            DockPanel.SetDock(buttons, Dock.Bottom);
            var dock = new DockPanel { Margin = new Avalonia.Thickness(16), LastChildFill = true };
            dock.Children.Add(hint);
            dock.Children.Add(buttons);
            dock.Children.Add(_tabs);
            Content = dock;
        }

        // The key the frontend's own row is stored under, beside the core's; no core declares it, so no core is handed it - see EmuSen_Settings_Reference.md §4.40.
        public const string ScreenFilterKey = "ScreenFilter";

        // The frontend's other row: the screen's shape as the core reports it, or the frame's square pixels - see EmuSen_Settings_Reference.md §4.97.
        public const string PictureShapeKey = "PictureShape", TvShape = "tv", SquarePixels = "square";

        public static readonly SettingRow PictureShape = new(PictureShapeKey, "Picture shape",
            "The screen the console was made for, or each of the frame's pixels square.",
            SettingRow.Choice, TvShape, 0, 0, new[] { TvShape, SquarePixels }, new[] { "TV shape", "Square pixels" }, null, null, null, null);

        // The frontend's third row: what is hidden at the picture's edges, on a console that had a television - see EmuSen_Settings_Reference.md §4.99.
        public const string OverscanKey = "Overscan", NoOverscan = "off", TvOverscan = "tv", CustomCrop = "custom";

        // The four edges of a custom crop, each a percentage of the picture's width or height, in the order they are shown.
        public static readonly (string Key, string Label)[] CropEdges = CropFields.Edges;

        public static readonly SettingRow Overscan = new(OverscanKey, "Overscan",
            "The whole picture, the part a television showed, or a crop of your own.",
            SettingRow.Choice, NoOverscan, 0, 0, new[] { NoOverscan, TvOverscan, CustomCrop }, new[] { "Whole picture", "TV overscan", "Custom crop" }, null, null, null, null);

        // The consoles that drove a television, an attachment by its host's screen; an LCD showed every pixel - see EmuSen_Serenity.md §3.12.
        public static bool HasOverscan(string console) => EmuSen.Serenity.Shaders.CrtFilter.TelevisionConsoles.Contains(RuntimeCatalog.HostOf(console));

        // What a console's stored choice hides at each edge of the picture - see EmuSen_Settings_Reference.md §4.99.
        public static EmuSen.Serenity.PictureCrop CropFor(GraphicsConfig config, string console)
        {
            if (!HasOverscan(console)) return EmuSen.Serenity.PictureCrop.None;
            double Edge(int i) => ParsePercent(config.Value(console, CropEdges[i].Key)) ?? 0;
            return config.Value(console, OverscanKey) switch
            {
                TvOverscan => EmuSen.Serenity.PictureCrop.Television,
                CustomCrop => EmuSen.Serenity.PictureCrop.Percent(Edge(0), Edge(2), Edge(1), Edge(3)),
                _ => EmuSen.Serenity.PictureCrop.None,
            };
        }

        // A percentage as typed, with a point or a comma, held to what one edge may hide; null when it is no number.
        public static double? ParsePercent(string? text) => CropFields.ParsePercent(text);

        // A stored value naming a preset in the downloaded pack, by its path inside it - see EmuSen_Settings_Reference.md §4.41.
        public const string SlangPrefix = "slang:";

        // The running game's rows for its own console, the console's schema for every other - see EmuSen_Backend.md §18.10.11.
        private IReadOnlyList<SettingRow> SettingsFor(string console) => RunningSettings?.Invoke(console) ?? RuntimeCatalog.SettingsFor(console);

        private Control BuildConsolePanel(string console)
        {
            // The schema's rows sorted as the tab shows them: the engine's, a medium's mode and the slots shown with it, then the core's own.
            IReadOnlyList<SettingRow> schema = SettingsFor(console);
            foreach (SettingRow s in schema) _core.Add((console, s.Key));
            var shownWith = schema.Select(r => r.ShownWithKey).OfType<string>().ToHashSet();
            SettingRow? engineRow = schema.FirstOrDefault(r => r.Key == RuntimeCatalog.EngineKey && r.Choices.Count > 1);
            IReadOnlyList<SettingRow> cardRows = schema.Where(r => r.ShownWithKey is not null || shownWith.Contains(r.Key)).ToArray();
            IReadOnlyList<SettingRow> settings = schema.Where(r => r.Key != RuntimeCatalog.EngineKey && !cardRows.Contains(r)).ToArray();
            var panel = new StackPanel { Spacing = 12, Margin = new Avalonia.Thickness(0, 8, 0, 0) };
            var refreshers = new List<Action>();

            // First on every tab: it belongs to the window the picture is drawn in, not to the core, so every console has it - see EmuSen_Settings_Reference.md §4.48.
            var shaderName = new TextBlock { Name = $"{console}.ShaderInUse", VerticalAlignment = VerticalAlignment.Center, TextWrapping = Avalonia.Media.TextWrapping.Wrap };
            var shaders = Ui.Button("Shaders...", () => OpenShaders(console));
            shaders.Name = $"{console}.Shaders";
            void ShowShader() => shaderName.Text = ShaderSettingsWindow.Describe(_config.Value(console, ScreenFilterKey));
            ShowShader();
            refreshers.Add(ShowShader);
            // The picture's shape beside its shader, in the one row, so no tab grows by a row - see EmuSen_Settings_Reference.md §4.97.
            (Control shapeControl, Action shapeRefresh) = BuildControl(console, PictureShape);
            refreshers.Add(shapeRefresh);
            var shaderRow = new DockPanel { LastChildFill = true };
            shaders.Margin = new Avalonia.Thickness(0, 0, 12, 0);
            shapeControl.Margin = new Avalonia.Thickness(12, 0, 0, 0);
            DockPanel.SetDock(shaders, Dock.Left);
            DockPanel.SetDock(shapeControl, Dock.Right);
            shaderRow.Children.Add(shaders);
            shaderRow.Children.Add(shapeControl);
            shaderRow.Children.Add(shaderName);
            panel.Children.Add(new FieldRow { Label = "Shader and shape", Hint = "The shader drawn over the picture, chosen in the Shaders window; and its shape, the console's screen's or each pixel square.", Content = shaderRow });

            // A row of its own: a dropdown takes left and right itself, so a second one beside the shape's could not be reached with a pad - see EmuSen_Settings_Reference.md §4.99.
            if (HasOverscan(console))
            {
                (Control overscanControl, Action overscanRefresh) = BuildControl(console, Overscan);
                (FieldRow cropRow, Action cropRefresh, Action cropShown) = BuildCropRow(console);
                refreshers.Add(overscanRefresh);
                refreshers.Add(cropRefresh);
                _notes.Add((console, cropShown));
                // Said under the row while the running game has a crop of its own, so the console's setting never seems ignored - see EmuSen_Settings_Reference.md §4.100.
                var replaced = Ui.Hint(OverscanNote?.Invoke(console) ?? "");
                replaced.Name = $"{console}.Overscan.Note";
                replaced.IsVisible = replaced.Text is { Length: > 0 };
                panel.Children.Add(new FieldRow { Label = Overscan.Label, Hint = Overscan.Hint, Content = new StackPanel { Spacing = 4, Children = { overscanControl, replaced } } });
                panel.Children.Add(cropRow);
            }

            // Before the core's own rows, since it decides which core reads them - see EmuSen_Settings_Reference.md §4.44.
            if (engineRow is { } engine)
            {
                (Control control, Action refresh) = BuildControl(console, engine);
                refreshers.Add(refresh);
                panel.Children.Add(new FieldRow { Label = engine.Label, Hint = engine.Hint, Content = control });
            }

            // The runtime's card mode, where the engine lists a card the player carries; the slots' rows only under Named cards - see EmuSen_Settings_Reference.md §4.114.
            foreach (FieldRow row in BuildCardRows(console, cardRows, refreshers)) panel.Children.Add(row);

            if (settings.Count == 0)
            {
                panel.Children.Add(new EmptyState { Message = "No core settings yet", Detail = $"The {console} core has nothing else to offer here; what it draws, it draws one way." });
                _panels.Add((console, () => { foreach (Action refresh in refreshers) refresh(); }));
                return panel;
            }

            string? group = null;
            foreach (SettingRow setting in settings)
            {
                // A heading before each group of rows the schema sets apart, enhancements among them - see EmuSen_Settings_Reference.md §4.110.
                if (setting.Group is { } g && g != group) panel.Children.Add(new SectionHeader { Name = $"{console}.Group.{g}", Text = g });
                group = setting.Group;
                (Control control, Action refresh) = BuildControl(console, setting);
                refreshers.Add(refresh);
                if (setting.HasNote)
                {
                    (Control noted, Action show) = WithNote(console, setting, control);
                    control = noted;
                    refreshers.Add(show);
                }
                panel.Children.Add(new FieldRow { Label = setting.Label, Hint = setting.Hint, Content = control });
            }

            _panels.Add((console, () => { foreach (Action refresh in refreshers) refresh(); }));
            return panel;
        }

        // Each medium's rows from the schema, a slot's shown while the row it is shown with holds its value, as the crop's row is while a custom crop is - see EmuSen_SaveMedia.md §3.3.
        private List<FieldRow> BuildCardRows(string console, IReadOnlyList<SettingRow> cardRows, List<Action> refreshers)
        {
            var rows = new List<FieldRow>();
            foreach (SettingRow setting in cardRows)
            {
                (Control control, Action refresh) = BuildControl(console, setting);
                refreshers.Add(refresh);
                if (setting.HasNote)
                {
                    (Control noted, Action show) = WithNote(console, setting, control);
                    control = noted;
                    refreshers.Add(show);
                }
                var row = new FieldRow { Name = $"{console}.{setting.Key}.Row", Label = setting.Label, Hint = setting.Hint, Content = control };
                if (setting.ShownWithKey is { } key)
                {
                    void Shown() => row.IsVisible = Value(console, key) == setting.ShownWithValue;
                    Shown();
                    refreshers.Add(Shown);
                    _notes.Add((console, Shown));
                }
                rows.Add(row);
            }
            return rows;
        }

        // Four numbers, shown while a custom crop is chosen; one that reads as a number is stored as it is typed, and shown as stored when the box is left.
        private (FieldRow, Action Refresh, Action Shown) BuildCropRow(string console)
        {
            double Stored(int edge) => ParsePercent(_config.Value(console, CropEdges[edge].Key)) ?? 0;
            var fields = new CropFields(console, Stored, (edge, percent) => Changed(console, CropEdges[edge].Key, CropFields.Text(percent)));
            var field = new FieldRow
            {
                Name = $"{console}.Crop", Label = "Crop, % of the picture", Content = fields.Row,
                Hint = $"{CropFields.Hint} The rest fills the screen, the same for every game of this console in every mode.",
            };
            // The boxes are filled only when the console's values are read afresh; a change elsewhere must not rewrite one being typed in.
            void Shown() => field.IsVisible = _config.Value(console, OverscanKey) == CustomCrop;
            void Refresh()
            {
                Shown();
                fields.Show();
            }
            Refresh();
            return (field, Refresh, Shown);
        }

        // A switch for on or off, a dropdown of the counts a range allows, or one of the names; each named so a test can find it.
        private (Control, Action) BuildControl(string console, SettingRow setting)
        {
            string name = $"{console}.{setting.Key}";
            string Current() => Value(console, setting.Key) ?? setting.Default;

            if (setting.Kind == SettingRow.Switch)
            {
                var toggle = new LunaSwitch { Name = name, HorizontalAlignment = HorizontalAlignment.Left };
                toggle.IsCheckedChanged += (_, _) => { if (!_filling) Changed(console, setting.Key, toggle.IsChecked == true ? "true" : "false"); };
                void Refresh() { _filling = true; toggle.IsChecked = string.Equals(Current(), "true", StringComparison.OrdinalIgnoreCase); _filling = false; }
                Refresh();
                return (toggle, Refresh);
            }

            var dropdown = new Dropdown { Name = name, HorizontalAlignment = HorizontalAlignment.Left, MinWidth = 120 };
            string[] values = setting.Kind == SettingRow.Count
                ? Enumerable.Range(setting.Min, setting.Max - setting.Min + 1).Select(n => n.ToString()).ToArray()
                : setting.Choices.ToArray();
            // A choice the core names in words is shown by them and stored by its value - see §4.90.
            string[] items = setting.ChoiceLabels.Count == values.Length ? setting.ChoiceLabels.ToArray() : values;
            string Shown(string value) => items[Math.Max(0, Array.IndexOf(values, value))];
            dropdown.Chose += chosen => { if (!_filling && chosen is string shown && Array.IndexOf(items, shown) is >= 0 and int i) Changed(console, setting.Key, values[i]); };
            void Fill() { _filling = true; string current = Current(); dropdown.Fill(items, Shown(values.Contains(current) ? current : setting.Default)); _filling = false; }
            Fill();
            return (dropdown, Fill);
        }

        // The core's sentence under its control when the value chosen is not the one it will use, the schema read again whenever the console's values change - see §4.26 and EmuSen_Multicore.md §13.1.
        private (Control, Action) WithNote(string console, SettingRow setting, Control control)
        {
            var text = Ui.Hint("");
            text.Name = $"{console}.{setting.Key}.Note";
            bool first = true;
            void Show()
            {
                string? said = first ? setting.Note : SettingsFor(console).FirstOrDefault(r => r.Key == setting.Key)?.Note;
                first = false;
                text.Text = said ?? "";
                text.IsVisible = said is not null;
            }
            Show();
            _notes.Add((console, Show));
            var stack = new StackPanel { Spacing = 4 };
            stack.Children.Add(control);
            stack.Children.Add(text);
            return (stack, Show);
        }

        // On a sheet over this one, on the same console; the row's label follows it when it closes.
        private void OpenShaders(string console)
        {
            ShaderWindow = new ShaderSettingsWindow(_config, ShadersChanged ?? _changed, console, _http, _pack);
            ShaderWindow.Closed += (_, _) => { foreach ((_, Action refresh) in _panels) refresh(); };
            _ = SheetLayer.Show(ShaderWindow, this);
        }

        // Saved at once, like the preferences, and the frontend told which console so a running one can take it - see §4.26.
        private void Changed(string console, string key, string value)
        {
            if (_core.Contains((console, key))) CoreSettings.Set(console, key, value);
            else
            {
                _config.SetValue(console, key, value);
                _config.Save();
            }
            foreach ((string noted, Action show) in _notes) if (noted == console) show();
            _changed?.Invoke(console);
        }

        private void ResetSelected()
        {
            int index = _tabs.SelectedIndex;
            if (index < 0 || index >= _panels.Count) return;

            (string console, Action refresh) = _panels[index];
            foreach ((string c, string key) in _core) if (c == console) CoreSettings.Set(console, key, null);
            _config.Forget(console);
            _config.Save();
            refresh();
            _changed?.Invoke(console);
        }
    }
}
