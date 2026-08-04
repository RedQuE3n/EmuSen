using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Interactivity;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;

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

        // Called when the library's console filter moves under an already-open window.
        public void SetConsole(string? console, (ICheatCodeCodec? AutoDetect, ICheatCodeCodec? Explicit) codecs)
        {
            _console = console;
            _pokeCodec = codecs.AutoDetect;
            _patchCodec = codecs.Explicit;
            Refresh();
        }

        public ActiveCheatsWindow() : this(new CheatRegistry()) { }

        public ActiveCheatsWindow(CheatRegistry registry, ICheatCodeCodec? pokeCodec = null, ICheatCodeCodec? patchCodec = null,
            Func<bool>? applyNow = null, Func<string?>? saveName = null, string? console = null)
        {
            InitializeComponent();
            _registry = registry;
            _pokeCodec = pokeCodec;
            _patchCodec = patchCodec;
            _applyNow = applyNow;
            _saveName = saveName;
            _console = console;
            Refresh();

            // The property, not IsCheckedChanged - only this reacts to a set
            // that didn't come from a click.
            MasterSwitch.PropertyChanged += (_, args) =>
            {
                if (args.Property == CheckBox.IsCheckedProperty) OnMasterSwitchChanged();
            };
        }

        // Called by the owner when something outside this window changed the
        // list - loading a game from the cheat database, or a ROM change.
        public void Refresh()
        {
            IReadOnlyList<CheatInfo> cheats = _registry.GetCheats();

            var selectedId = (CheatsList.SelectedItem as CheatRow)?.Id;
            CheatsList.ItemsSource = cheats.Select(c => new CheatRow(_registry, c, Detail(c))).ToList();
            if (selectedId is int id)
            {
                CheatsList.SelectedItem = CheatsList.ItemsSource.Cast<CheatRow>().FirstOrDefault(r => r.Id == id);
            }

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
                StatusText.Text = "Load a game's cheats from Settings > Cheat Database..., or add a code below.";
            }

            // A console whose core has no codec cannot parse a typed code at all - see EmuSen_Multicore.md §4.
            bool canAdd = _pokeCodec is not null || _patchCodec is not null;
            AddButton.IsEnabled = canAdd;
            if (!canAdd && _console is { } noCodec)
            {
                StatusText.Text = $"{noCodec} has no cheat-code format in this build, so codes cannot be added for it.";
            }

            ConsoleText.Text = _console is null ? "" : $"Console: {_console}";

            ClearButton.IsEnabled = cheats.Count > 0;
            ApplyButton.IsEnabled = cheats.Count > 0;
            SaveButton.IsEnabled = cheats.Count > 0;
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

        private void OnCheatSelected(object? sender, SelectionChangedEventArgs e) => UpdateRemoveButton();

        private void UpdateRemoveButton() => RemoveButton.IsEnabled = CheatsList.SelectedItem is CheatRow;

        private void OnRemoveClick(object? sender, RoutedEventArgs e)
        {
            if (CheatsList.SelectedItem is not CheatRow row) return;

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
            string code = (CodeBox.Text ?? "").Trim();
            if (code.Length == 0)
            {
                StatusText.Text = "Type a cheat code first.";
                return;
            }

            string description = (DescriptionBox.Text ?? "").Trim();
            if (description.Length == 0) description = code;

            bool gameGenie = CheatCommand.PrefersExplicitCodec(_patchCodec, _pokeCodec, code);
            ICheatCodeCodec? codec = gameGenie ? _patchCodec : _pokeCodec;
            if (codec is null)
            {
                StatusText.Text = "No cheat code decoder is available for this core.";
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

            CodeBox.Text = "";
            DescriptionBox.Text = "";
            Refresh();
            StatusText.Text = $"Added '{description}' as {codec.Name}, enabled.";
        }

        private void OnCloseClick(object? sender, RoutedEventArgs e) => Close();
    }
}
