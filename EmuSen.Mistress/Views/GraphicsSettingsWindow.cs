using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.Cores;
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

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public GraphicsSettingsWindow() : this(new GraphicsConfig(), null, null) { }

        private readonly Func<System.Net.Http.HttpClient> _http;
        private readonly string _pack;

        // The Shaders window the button opened last, for a test to drive.
        public ShaderSettingsWindow? ShaderWindow { get; private set; }

        // What a shader change calls instead of the constructor's callback, so a slider does not re-apply the core's settings.
        public Action<string>? ShadersChanged { get; set; }

        public GraphicsSettingsWindow(GraphicsConfig config, Action<string>? changed, string? selectedConsole,
            Func<System.Net.Http.HttpClient>? http = null, string? pack = null)
        {
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

            var consoles = CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console).ToList();
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

        public static readonly CoreSetting PictureShape = new(PictureShapeKey, "Picture shape",
            "The screen the console was made for, or each of the frame's pixels square.",
            CoreSettingKind.Choice, TvShape, Choices: new[] { TvShape, SquarePixels }, ChoiceLabels: new[] { "TV shape", "Square pixels" });

        // The frontend's third row: what is hidden at the picture's edges, on a console that had a television - see EmuSen_Settings_Reference.md §4.99.
        public const string OverscanKey = "Overscan", NoOverscan = "off", TvOverscan = "tv", CustomCrop = "custom";

        // The four edges of a custom crop, each a percentage of the picture's width or height, in the order they are shown.
        public static readonly (string Key, string Label)[] CropEdges = { ("CropLeft", "Left"), ("CropRight", "Right"), ("CropTop", "Top"), ("CropBottom", "Bottom") };

        public static readonly CoreSetting Overscan = new(OverscanKey, "Overscan",
            "The whole picture, the part a television showed, or a crop of your own.",
            CoreSettingKind.Choice, NoOverscan, Choices: new[] { NoOverscan, TvOverscan, CustomCrop }, ChoiceLabels: new[] { "Whole picture", "TV overscan", "Custom crop" });

        // The consoles that drove a television, which are the ones the tube's filter is offered for; an LCD showed every pixel.
        public static bool HasOverscan(string console) => EmuSen.Serenity.Shaders.CrtFilter.Filter.Consoles!.Contains(console);

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
        public static double? ParsePercent(string? text) =>
            double.TryParse(text?.Trim().Replace(',', '.'), System.Globalization.NumberStyles.Float, System.Globalization.CultureInfo.InvariantCulture, out double value) && double.IsFinite(value)
                ? Math.Clamp(Math.Round(value, 1), 0, EmuSen.Serenity.PictureCrop.MostAtAnEdge * 100) : null;

        // A stored value naming a preset in the downloaded pack, by its path inside it - see EmuSen_Settings_Reference.md §4.41.
        public const string SlangPrefix = "slang:";

        private Control BuildConsolePanel(string console)
        {
            IReadOnlyList<CoreSetting> settings = CoreCatalog.SettingsFor(console);
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
                panel.Children.Add(new FieldRow { Label = Overscan.Label, Hint = Overscan.Hint, Content = overscanControl });
                panel.Children.Add(cropRow);
            }

            // Before the core's own rows, since it decides which core reads them - see EmuSen_Settings_Reference.md §4.44.
            if (CoreCatalog.EngineFor(console) is { } engine)
            {
                (Control control, Action refresh) = BuildControl(console, engine);
                refreshers.Add(refresh);
                panel.Children.Add(new FieldRow { Label = engine.Label, Hint = engine.Hint, Content = control });
            }

            if (settings.Count == 0)
            {
                panel.Children.Add(new EmptyState { Message = "No core settings yet", Detail = $"The {console} core has nothing else to offer here; what it draws, it draws one way." });
                _panels.Add((console, () => { foreach (Action refresh in refreshers) refresh(); }));
                return panel;
            }

            foreach (CoreSetting setting in settings)
            {
                (Control control, Action refresh) = BuildControl(console, setting);
                refreshers.Add(refresh);
                if (setting.Note is { } note)
                {
                    (Control noted, Action show) = WithNote(console, setting, note, control, settings);
                    control = noted;
                    refreshers.Add(show);
                }
                panel.Children.Add(new FieldRow { Label = setting.Label, Hint = setting.Hint, Content = control });
            }

            _panels.Add((console, () => { foreach (Action refresh in refreshers) refresh(); }));
            return panel;
        }

        // Four numbers, shown while a custom crop is chosen; one that reads as a number is stored as it is typed, and shown as stored when the box is left.
        private (FieldRow, Action Refresh, Action Shown) BuildCropRow(string console)
        {
            var boxes = new List<TextBox>();
            var row = new WrapPanel { Orientation = Orientation.Horizontal };
            foreach ((string key, string label) in CropEdges)
            {
                var box = new TextBox { Name = $"{console}.{key}", Width = 64, VerticalAlignment = VerticalAlignment.Center };
                Avalonia.Automation.AutomationProperties.SetName(box, $"{label}, percent");
                // The event comes after the text is set, so a box filled from the stored value is told from a typed one by its value, not by a flag.
                box.TextChanged += (_, _) => { if (ParsePercent(box.Text) is { } percent && percent != (ParsePercent(_config.Value(console, key)) ?? 0)) Changed(console, key, Stored(percent)); };
                box.LostFocus += (_, _) => Show(box, key);
                boxes.Add(box);
                row.Children.Add(new TextBlock { Text = label, VerticalAlignment = VerticalAlignment.Center, Margin = new Avalonia.Thickness(0, 0, 6, 0) });
                box.Margin = new Avalonia.Thickness(0, 0, 14, 0);
                row.Children.Add(box);
            }
            var field = new FieldRow
            {
                Name = $"{console}.Crop", Label = "Crop, % of the picture", Content = row,
                Hint = "How much of the picture's width or height to hide at each edge, up to 25. The rest fills the screen, the same for every game of this console in every mode.",
            };
            string Stored(double percent) => percent.ToString("0.#", System.Globalization.CultureInfo.InvariantCulture);
            void Show(TextBox box, string key) => box.Text = Stored(ParsePercent(_config.Value(console, key)) ?? 0);
            // The boxes are filled only when the console's values are read afresh; a change elsewhere must not rewrite one being typed in.
            void Shown() => field.IsVisible = _config.Value(console, OverscanKey) == CustomCrop;
            void Refresh()
            {
                Shown();
                for (int i = 0; i < boxes.Count; i++) Show(boxes[i], CropEdges[i].Key);
            }
            Refresh();
            return (field, Refresh, Shown);
        }

        // A switch for on or off, a dropdown of the counts a range allows, or one of the names; each named so a test can find it.
        private (Control, Action) BuildControl(string console, CoreSetting setting)
        {
            string name = $"{console}.{setting.Key}";
            string Current() => _config.Value(console, setting.Key) ?? setting.Default;

            if (setting.Kind == CoreSettingKind.Switch)
            {
                var toggle = new LunaSwitch { Name = name, HorizontalAlignment = HorizontalAlignment.Left };
                toggle.IsCheckedChanged += (_, _) => { if (!_filling) Changed(console, setting.Key, toggle.IsChecked == true ? "true" : "false"); };
                void Refresh() { _filling = true; toggle.IsChecked = string.Equals(Current(), "true", StringComparison.OrdinalIgnoreCase); _filling = false; }
                Refresh();
                return (toggle, Refresh);
            }

            var dropdown = new Dropdown { Name = name, HorizontalAlignment = HorizontalAlignment.Left, MinWidth = 120 };
            string[] values = setting.Kind == CoreSettingKind.Count
                ? Enumerable.Range(setting.Min, setting.Max - setting.Min + 1).Select(n => n.ToString()).ToArray()
                : (setting.Choices ?? Array.Empty<string>()).ToArray();
            // A choice the core names in words is shown by them and stored by its value - see §4.90.
            string[] items = setting.ChoiceLabels is { } labels && labels.Count == values.Length ? labels.ToArray() : values;
            string Shown(string value) => items[Math.Max(0, Array.IndexOf(values, value))];
            dropdown.Chose += chosen => { if (!_filling && chosen is string shown && Array.IndexOf(items, shown) is >= 0 and int i) Changed(console, setting.Key, values[i]); };
            void Fill() { _filling = true; string current = Current(); dropdown.Fill(items, Shown(values.Contains(current) ? current : setting.Default)); _filling = false; }
            Fill();
            return (dropdown, Fill);
        }

        // The core's sentence under its control when the value chosen is not the one it will use, shown again whenever the console's values change - see §4.26 and EmuSen_Multicore.md §13.1.
        private (Control, Action) WithNote(string console, CoreSetting setting, Func<Func<string, string>, string?> note, Control control, IReadOnlyList<CoreSetting> settings)
        {
            var text = Ui.Hint("");
            text.Name = $"{console}.{setting.Key}.Note";
            string Value(string key) => _config.Value(console, key) ?? settings.FirstOrDefault(s => s.Key == key)?.Default ?? "";
            void Show()
            {
                string? said = note(Value);
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
            _config.SetValue(console, key, value);
            _config.Save();
            foreach ((string noted, Action show) in _notes) if (noted == console) show();
            _changed?.Invoke(console);
        }

        private void ResetSelected()
        {
            int index = _tabs.SelectedIndex;
            if (index < 0 || index >= _panels.Count) return;

            (string console, Action refresh) = _panels[index];
            _config.Forget(console);
            _config.Save();
            refresh();
            _changed?.Invoke(console);
        }
    }
}
