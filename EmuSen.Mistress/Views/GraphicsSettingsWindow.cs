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
            var shaderRow = new DockPanel { LastChildFill = true };
            shaders.Margin = new Avalonia.Thickness(0, 0, 12, 0);
            DockPanel.SetDock(shaders, Dock.Left);
            shaderRow.Children.Add(shaders);
            shaderRow.Children.Add(shaderName);
            panel.Children.Add(new FieldRow { Label = "Shader", Hint = "Drawn over the picture as it is shown, never in the game's own frame. Chosen, searched and adjusted in the Shaders window.", Content = shaderRow });

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
                panel.Children.Add(new FieldRow { Label = setting.Label, Hint = setting.Hint, Content = control });
            }

            _panels.Add((console, () => { foreach (Action refresh in refreshers) refresh(); }));
            return panel;
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
            string[] items = setting.Kind == CoreSettingKind.Count
                ? Enumerable.Range(setting.Min, setting.Max - setting.Min + 1).Select(n => n.ToString()).ToArray()
                : (setting.Choices ?? Array.Empty<string>()).ToArray();
            dropdown.Chose += chosen => { if (!_filling && chosen is string value) Changed(console, setting.Key, value); };
            void Fill() { _filling = true; string current = Current(); dropdown.Fill(items, items.Contains(current) ? current : setting.Default); _filling = false; }
            Fill();
            return (dropdown, Fill);
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
