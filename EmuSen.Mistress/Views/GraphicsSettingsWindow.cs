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

        // The picker the "RetroArch Preset..." entry opened last, for a test to drive.
        public SlangPresetWindow? PresetPicker { get; private set; }

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

        // The dropdown's last entry, which opens the picker rather than being a filter itself.
        public const string ChooseRetroArch = "RetroArch Preset...";

        public static string SlangLabel(string relative) => "RetroArch: " + System.IO.Path.GetFileNameWithoutExtension(relative);

        private Control BuildConsolePanel(string console)
        {
            IReadOnlyList<CoreSetting> settings = CoreCatalog.SettingsFor(console);
            var panel = new StackPanel { Spacing = 12, Margin = new Avalonia.Thickness(0, 8, 0, 0) };
            var refreshers = new List<Action>();

            // First on every tab: it belongs to the window the picture is drawn in, not to the core, so every console has it.
            var filter = new Dropdown { Name = $"{console}.{ScreenFilterKey}", HorizontalAlignment = HorizontalAlignment.Left, MinWidth = 160 };
            void FillFilter()
            {
                _filling = true;
                string current = _config.Value(console, ScreenFilterKey) ?? EmuSen.Serenity.Shaders.ScreenFilters.None;
                var names = EmuSen.Serenity.Shaders.ScreenFilters.NamesFor(console).ToList();
                string selected = names.Contains(current) ? current : EmuSen.Serenity.Shaders.ScreenFilters.None;
                if (current.StartsWith(SlangPrefix, StringComparison.Ordinal))
                {
                    selected = SlangLabel(current[SlangPrefix.Length..]);
                    names.Add(selected);
                }
                names.Add(ChooseRetroArch);
                filter.Fill(names, selected);
                _filling = false;
            }
            filter.Chose += chosen =>
            {
                if (_filling || chosen is not string value) return;
                if (value == ChooseRetroArch)
                {
                    string current = _config.Value(console, ScreenFilterKey) ?? "";
                    PresetPicker = new SlangPresetWindow(_http, _pack,
                        current.StartsWith(SlangPrefix, StringComparison.Ordinal) ? current[SlangPrefix.Length..] : null,
                        preset => Changed(console, ScreenFilterKey, SlangPrefix + preset), console);
                    PresetPicker.Closed += (_, _) => FillFilter();
                    _ = SheetLayer.Show(PresetPicker, this);
                }
                else if (!value.StartsWith("RetroArch: ", StringComparison.Ordinal)) Changed(console, ScreenFilterKey, value);
            };
            FillFilter();
            refreshers.Add(FillFilter);
            panel.Children.Add(new FieldRow { Label = "Screen Filter", Hint = "Drawn over the picture as it is shown, never in the game's own frame; changes apply at once.", Content = filter });

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
