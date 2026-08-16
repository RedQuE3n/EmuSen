using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Theme;
using EmuSen.LunaP.Windowing;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;

namespace EmuSen.Mistress.Views
{
    // The GUI half of `cheat list`/`enable`/`disable`/`remove`/`master` -
    // see `man cheat` and EmuSen_Settings_Reference.md §4.14.
    public partial class ActiveCheatsWindow : Window
    {
        private readonly CheatRegistry _registry;
        private ICheatCodeCodec? _pokeCodec;
        private ICheatCodeCodec? _patchCodec;

        // Queues one immediate apply on the thread that owns the core and
        // says whether there was a core to queue it on - see §4.15.
        private readonly Func<bool>? _applyNow;

        // What this list would be saved as, or null when no game is running
        // to name it after - see §4.15.
        private readonly Func<string?>? _saveName;

        // Set while Refresh() writes MasterSwitch, so its own change handler
        // does not write the value straight back into the registry.
        private bool _syncing;

        // Which console a typed code is parsed for - see EmuSen_Multicore.md §10.
        private string? _console;

        // One per console tab, in the order BuildConsoleTabs added them - see §4.14.
        private readonly List<ConsoleTab> _consoleTabs = new();

        // The Add box is the only console-specific part of this window - see §4.14.
        private sealed class ConsoleTab
        {
            public required string Console { get; init; }
            public required string DisplayName { get; init; }
            public required TextBox CodeBox { get; init; }
            public required TextBox DescriptionBox { get; init; }
            public required Button AddButton { get; init; }
            public required TextBlock FormatsText { get; init; }
            public ICheatCodeCodec? Poke { get; set; }
            public ICheatCodeCodec? Patch { get; set; }
        }

        // Called when the library's console filter moves under an already-open window.
        public void SetConsole(string? console, (ICheatCodeCodec? AutoDetect, ICheatCodeCodec? Explicit) codecs)
        {
            _console = console;
            _pokeCodec = codecs.AutoDetect;
            _patchCodec = codecs.Explicit;
            ApplyLiveCodecsToTab();
            SelectConsoleTab(console);
            Refresh();
        }

        // Four columns matching the hand-laid Grid this replaced - see §4.14a.
        private void BuildCheatColumns()
        {
            CheatsList.Key = r => r.Id;
            CheatsList.Chose += _ => UpdateRemoveButton();

            // Until LunaP > 0.8.0 forwards it - LunaP.md §78.4, fixed there, unreleased here.
            CheatsList.TemplateApplied += (_, e) =>
            {
                if (e.NameScope.Find<ListBox>("PART_Rows") is { } rows)
                {
                    Avalonia.Automation.AutomationProperties.SetName(rows, "Cheats for this console");
                }
            };

            CheatsList.Column(new LunaColumn<CheatRow>("On", r => r.Enabled, (r, on) => r.Enabled = on, r => r.Description)
            {
                Width = "40",
            });
            CheatsList.Column(new LunaColumn<CheatRow>("Kind", r => Muted(r.Kind), r => r.Kind) { Width = "44" });
            CheatsList.Column(new LunaColumn<CheatRow>("Cheat", r => r.Description) { Width = "*" });
            CheatsList.Column(new LunaColumn<CheatRow>("Code", r => Mono(r.Detail), r => r.Detail) { Width = "Auto" });
        }

        private static Control Muted(string text) => new TextBlock
        {
            Text = text,
            Foreground = Brush("LunaMuted"),
            FontSize = 11,
            VerticalAlignment = Avalonia.Layout.VerticalAlignment.Center,
        };

        private static Control Mono(string text) => new TextBlock
        {
            Text = text,
            Foreground = Brush("LunaMuted"),
            FontFamily = new Avalonia.Media.FontFamily("monospace"),
            FontSize = 11,
            VerticalAlignment = Avalonia.Layout.VerticalAlignment.Center,
        };

        // DynamicResource in code: the theme can change under a live window - see EmuSen_LunaP.md §12.3.
        private static Avalonia.Media.IBrush? Brush(string key) =>
            Avalonia.Application.Current?.FindResource(key) as Avalonia.Media.IBrush;

        public ActiveCheatsWindow() : this(new CheatRegistry()) { }

        public ActiveCheatsWindow(CheatRegistry registry, ICheatCodeCodec? pokeCodec = null, ICheatCodeCodec? patchCodec = null,
            Func<bool>? applyNow = null, Func<string?>? saveName = null, string? console = null)
        {
            InitializeComponent();
            BuildCheatColumns();
            _registry = registry;
            _pokeCodec = pokeCodec;
            _patchCodec = patchCodec;
            _applyNow = applyNow;
            _saveName = saveName;
            _console = console;
            BuildConsoleTabs();
            ApplyLiveCodecsToTab();
            SelectConsoleTab(console);
            _built = true;
            Refresh();

            // The property, not IsCheckedChanged - only this reacts to a set
            // that didn't come from a click.
            MasterSwitch.PropertyChanged += (_, args) =>
            {
                if (args.Property == LunaSwitch.IsCheckedProperty) OnMasterSwitchChanged();
            };
        }

        private void BuildConsoleTabs()
        {
            foreach (var console in EmuSen.Cores.CoreCatalog.ConsolesInReleaseOrder)
            {
                var codecs = EmuSen.Cores.CoreFactory.CheatCodecsFor(console.DisplayName);

                var codeBox = new TextBox { Name = console.Console + "CodeBox", PlaceholderText = "Code" };
                TextBox descriptionBox = new TextBox { Name = console.Console + "DescriptionBox", PlaceholderText = "Description" }.Margin(8, 0, 0, 0);
                Button addButton = new Button { Name = console.Console + "AddButton", Content = "Add" }.Margin(8, 0, 0, 0);
                HintText formats = Ui.Hint("");

                addButton.Click += OnAddClick;

                StackPanel panel = Ui.Stack(6,
                    new TextBlock { Text = $"Add a {console.Console} cheat", FontWeight = Avalonia.Media.FontWeight.Bold },
                    formats,
                    Ui.Cols("*,*,Auto", codeBox, descriptionBox, addButton)).Margin(0, 8, 0, 0);

                var tab = new ConsoleTab
                {
                    Console = console.Console,
                    DisplayName = console.DisplayName,
                    CodeBox = codeBox,
                    DescriptionBox = descriptionBox,
                    AddButton = addButton,
                    FormatsText = formats,
                    Poke = codecs.AutoDetect,
                    Patch = codecs.Explicit,
                };
                _consoleTabs.Add(tab);
                DescribeFormats(tab);

                Tabs.Add(console.Console, panel);
            }
        }

        // A running game's codecs beat the catalog's, since those are the ones that can actually apply.
        private void ApplyLiveCodecsToTab()
        {
            if (_console is null || (_pokeCodec is null && _patchCodec is null)) return;

            foreach (ConsoleTab tab in _consoleTabs)
            {
                if (!Names(tab, _console)) continue;
                tab.Poke = _pokeCodec;
                tab.Patch = _patchCodec;
                DescribeFormats(tab);
            }
        }

        // Named from the codecs themselves, so a console with no decoder says so instead of offering a dead box.
        private static void DescribeFormats(ConsoleTab tab)
        {
            var names = new List<string>();
            if (tab.Patch is { } patch) names.Add(patch.Name);
            if (tab.Poke is { } poke) names.Add(poke.Name);

            tab.FormatsText.Text = names.Count == 0
                ? $"{tab.Console} has no cheat-code format in this build, so codes cannot be added for it."
                : $"Accepts: {string.Join(", ", names)}.";
        }

        // Callers pass "SNES" or "SNES (Venus)" depending on where they got it - see EmuSen_Multicore.md §10.
        private static bool Names(ConsoleTab tab, string? name) =>
            string.Equals(tab.Console, name, StringComparison.OrdinalIgnoreCase)
            || string.Equals(tab.DisplayName, name, StringComparison.OrdinalIgnoreCase);

        // General is index 0, so console i sits at i + 1 - matches InputSettingsWindow.
        private void SelectConsoleTab(string? console)
        {
            if (console is null) return;

            for (int i = 0; i < _consoleTabs.Count; i++)
            {
                if (!Names(_consoleTabs[i], console)) continue;
                Tabs.SelectedIndex = i + 1;
                return;
            }
        }

        // Null on General, which has no code box of its own.
        private ConsoleTab? SelectedTab()
        {
            int index = Tabs.SelectedIndex - 1;
            return index >= 0 && index < _consoleTabs.Count ? _consoleTabs[index] : null;
        }

        // Adding the tabs raises SelectionChanged before the constructor has finished wiring this window up.
        private bool _built;

        private void OnTabChanged(object? sender, SelectionChangedEventArgs e)
        {
            if (_built) Refresh();
        }

        // Called by the owner when something outside this window changed the
        // list - loading a game from the cheat database, or a ROM change.
        public void Refresh()
        {
            IReadOnlyList<CheatInfo> cheats = _registry.GetCheats();

            // Key is the cheat id, so Refresh puts the selection back itself - see §4.14a.
            CheatsList.Refresh(cheats.Select(c => new CheatRow(_registry, c, Detail(c))));
            UpdateRemoveButton();

            _syncing = true;
            MasterSwitch.IsChecked = _registry.MasterEnabled;
            _syncing = false;

            int enabled = cheats.Count(c => c.Enabled);
            ListHeaderText.Text = cheats.Count == 0
                ? "No cheats loaded"
                : $"{cheats.Count} cheat(s), {enabled} on";

            MasterHintText.Text = _registry.MasterEnabled
                ? "Off switches every cheat below off at once, without changing any of their own boxes."
                : "Every cheat below is off while this is unchecked, whatever its own box says.";

            if (cheats.Count == 0)
            {
                StatusText.Text = "Load a game's cheats from Settings > Cheat Database..., or add a code on a console tab.";
            }

            // A console whose core has no codec cannot parse a typed code at all - see EmuSen_Multicore.md §4.
            foreach (ConsoleTab tab in _consoleTabs)
            {
                tab.AddButton.IsEnabled = tab.Poke is not null || tab.Patch is not null;
            }

            string? saveName = _saveName?.Invoke();
            SaveTargetText.Text = saveName is null
                ? "Save names the list after the running game, so start one first. Save As writes anywhere and is never loaded automatically."
                : $"Save writes '{saveName}', which comes back automatically next time you start it. Save As writes anywhere and is never loaded automatically.";

            ClearButton.IsEnabled = cheats.Count > 0;
            ApplyButton.IsEnabled = cheats.Count > 0;
            SaveButton.IsEnabled = cheats.Count > 0 && saveName is not null;
            SaveAsButton.IsEnabled = cheats.Count > 0;
            LoadButton.IsEnabled = saveName is not null && CheatFile.For(saveName).Exists;
            UpdateRemoveButton();
        }

        // Ticking a box already takes effect on the next frame - this is for
        // a paused game, and for wanting to be told it worked. See §4.15.
        private void OnApplyClick(object? sender, RoutedEventArgs e)
        {
            IReadOnlyList<CheatInfo> cheats = _registry.GetCheats();
            int enabled = cheats.Count(c => c.Enabled);

            if (enabled == 0)
            {
                StatusText.Text = "No cheat is ticked, so there is nothing to apply.";
                return;
            }

            // Otherwise "Applied 12 cheat(s)" would be a lie for as long as
            // the master switch is off.
            if (!_registry.MasterEnabled)
            {
                _registry.MasterEnabled = true;
                Refresh();
            }

            bool running = _applyNow?.Invoke() ?? false;
            StatusText.Text = running
                ? $"Applied {enabled} cheat(s), and they stay applied every frame."
                : $"{enabled} cheat(s) armed - they apply as soon as a game is running.";
        }

        // Named after the running game so LoadRom can find it again without
        // being told - see §4.15.
        private void OnSaveClick(object? sender, RoutedEventArgs e)
        {
            if (_saveName?.Invoke() is not string name)
            {
                StatusText.Text = "Start a game first - a saved cheat list is named after the game it belongs to.";
                return;
            }

            ConfigFile<CheatFile> file = CheatFile.For(name);
            if (!file.Save(_registry.ToCheatFile()))
            {
                StatusText.Text = $"Couldn't write {file.Path}";
                return;
            }

            StatusText.Text = $"Saved {_registry.GetCheats().Count} cheat(s) as '{name}' - they come back automatically next time you start it.";
            Refresh();
        }

        // The counterpart to Save: pulls the running game's saved list back, discarding what is live.
        private void OnLoadClick(object? sender, RoutedEventArgs e)
        {
            if (_saveName?.Invoke() is not string name)
            {
                StatusText.Text = "Start a game first - a saved cheat list is named after the game it belongs to.";
                return;
            }

            ConfigFile<CheatFile> file = CheatFile.For(name);
            if (file.Load() is not CheatFile loaded)
            {
                StatusText.Text = file.Exists ? $"Couldn't read {file.Path}" : $"No saved cheat list named '{name}' yet.";
                return;
            }

            ReplaceWith(loaded, $"'{name}'");
        }

        private async void OnSaveAsClick(object? sender, RoutedEventArgs e)
        {
            string? path = await Dialogs.SaveFileAsync(this, "Save cheat list as",
                (_saveName?.Invoke() ?? "cheats") + ".json", new[] { CheatFileType }, defaultExtension: "json");

            if (path is null) return;

            StatusText.Text = CheatFile.SaveTo(path, _registry.ToCheatFile())
                ? $"Saved {_registry.GetCheats().Count} cheat(s) to {path}. This copy is not loaded automatically."
                : $"Couldn't write {path}";
        }

        private async void OnLoadFromClick(object? sender, RoutedEventArgs e)
        {
            if (await Dialogs.PickFileAsync(this, "Load cheat list", new[] { CheatFileType }) is not { } picked) return;

            if (CheatFile.LoadFrom(picked.Path) is not CheatFile loaded)
            {
                StatusText.Text = $"Couldn't read a cheat list from {picked.Path}";
                return;
            }

            ReplaceWith(loaded, picked.Path);
        }

        private static FilePickerFileType CheatFileType => new("Cheat list") { Patterns = new[] { "*.json" } };

        // Replaces rather than merges, so what is on screen is what the file says - see §4.15.
        private void ReplaceWith(CheatFile file, string source)
        {
            _registry.Clear();
            (int loaded, int skipped) = _registry.LoadFrom(file);
            Refresh();
            StatusText.Text = skipped == 0
                ? $"Loaded {loaded} cheat(s) from {source}."
                : $"Loaded {loaded} cheat(s) from {source}; {skipped} entr(y/ies) could not be read.";
        }

        // The write model, formatted the same way `cheat list` prints it.
        private static string Detail(CheatInfo cheat)
        {
            if (cheat.Writes.Count != 1) return $"{cheat.Writes.Count} writes";
            return CheatCommand.FormatWrite(cheat.Writes[0], cheat.Kind);
        }

        private void OnMasterSwitchChanged()
        {
            if (_syncing) return;

            _registry.MasterEnabled = MasterSwitch.IsChecked == true;
            StatusText.Text = _registry.MasterEnabled ? "Cheats are on." : "Cheats are off.";
            Refresh();
        }

        private void UpdateRemoveButton() => RemoveButton.IsEnabled = CheatsList.Selected is not null;

        private void OnRemoveClick(object? sender, RoutedEventArgs e)
        {
            if (CheatsList.Selected is not CheatRow row) return;

            _registry.RemoveCheat(row.Id);
            Refresh();
            StatusText.Text = $"Removed {row.Description}.";
        }

        private void OnClearClick(object? sender, RoutedEventArgs e)
        {
            int removed = _registry.GetCheats().Count;
            _registry.Clear();
            Refresh();
            StatusText.Text = $"Removed {removed} cheat(s).";
        }

        // Same best-effort format guess `cheat add` makes - see `man cheat`.
        private void OnAddClick(object? sender, RoutedEventArgs e)
        {
            // The tab the button lives on, so a code is always parsed as the console it was typed under.
            ConsoleTab? tab = _consoleTabs.FirstOrDefault(t => ReferenceEquals(t.AddButton, sender)) ?? SelectedTab();
            if (tab is null) return;

            string code = (tab.CodeBox.Text ?? "").Trim();
            if (code.Length == 0)
            {
                StatusText.Text = "Type a cheat code first.";
                return;
            }

            string description = (tab.DescriptionBox.Text ?? "").Trim();
            if (description.Length == 0) description = code;

            bool gameGenie = CheatCommand.PrefersExplicitCodec(tab.Patch, tab.Poke, code);
            ICheatCodeCodec? codec = gameGenie ? tab.Patch : tab.Poke;
            if (codec is null)
            {
                StatusText.Text = $"{tab.Console} has no cheat-code decoder in this build.";
                return;
            }

            try
            {
                (int address, byte value) = codec.Decode(code);
                // Dropping the compare would not be a lesser cheat but a wrong one - see EmuSen_Cheats.md §2.
                if (gameGenie) _registry.AddRomPatch(address, value, codec.DecodeCompare(code), description);
                else _registry.AddRamPoke(codec.SpaceName ?? CheatImport.DefaultSpaceName, address, value, description);
            }
            catch (Exception ex)
            {
                StatusText.Text = $"Couldn't decode '{code}' as {codec.Name}: {ex.Message}";
                return;
            }

            tab.CodeBox.Text = "";
            tab.DescriptionBox.Text = "";
            Refresh();
            StatusText.Text = $"Added '{description}' as {codec.Name}, enabled.";
        }

        private void OnCloseClick(object? sender, RoutedEventArgs e) => Close();
    }
}
