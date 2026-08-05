using System.Linq;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // Log/state/ROM directories and core selection - see EmuSen_Config_Reference.md for the persisted shape.
    public class PreferencesWindow : ToolWindow
    {
        // From the catalog - see AppSettings.SelectedCore for why it still drives nothing.
        private static readonly string[] AvailableCores =
            EmuSen.Cores.CoreCatalog.Cores.Select(c => c.DisplayName).ToArray();

        private readonly AppSettings _settings;
        private readonly ComboBox _core = new() { Name = "CoreComboBox", HorizontalAlignment = HorizontalAlignment.Stretch };
        private bool _initializing;

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public PreferencesWindow() : this(new AppSettings()) { }

        public PreferencesWindow(AppSettings settings)
        {
            _settings = settings;

            Title = "Preferences";
            Width = 520;
            CanResize = false;

            // Sized to its content rather than a fixed 330px: at that height the Close button sat below the
            // bottom edge of a window that cannot be resized or scrolled. See EmuSen_LunaP.md §11.1.
            SizeToContent = SizeToContent.Height;

            PathPickerRow logDirectory = Picker("LogDirectoryBox", "(not set)", "Choose Log Directory",
                _settings.LogDirectory, p => _settings.LogDirectory = p);
            PathPickerRow stateDirectory = Picker("StateDirectoryBox", "(default)", "Choose Save State Directory",
                _settings.StateDirectory, p => _settings.StateDirectory = p);
            PathPickerRow romDirectory = Picker("RomDirectoryBox", "(not set)", "Choose ROM Directory",
                _settings.RomDirectory, p => _settings.RomDirectory = p);

            Content = Ui.Stack(12,
                new FieldRow
                {
                    Label = "Log Directory",
                    Hint = "Where per-session log files are written. Leave blank to disable file logging entirely.",
                    Content = logDirectory,
                },
                new FieldRow
                {
                    Label = "Save State Directory",
                    Hint = "Where Save State/Load State write and read .state files. Leave blank to use the default (home/Saves/Save States/).",
                    Content = stateDirectory,
                },
                new FieldRow
                {
                    Label = "ROM Directory",
                    Hint = "Default folder for Open ROM... and the ROM list in File > Browse ROMs....",
                    Content = romDirectory,
                },
                new FieldRow
                {
                    Label = "Emulator Core",
                    Hint = "Only one core exists today - this is scaffolding for when a second one does.",
                    Content = _core,
                },
                Ui.Buttons(Ui.Button("Close", Close)).Margin(0, 12, 0, 0)).Margin(16);

            _initializing = true;
            _core.ItemsSource = AvailableCores;
            _core.SelectedIndex = System.Math.Max(0, System.Array.IndexOf(AvailableCores, _settings.SelectedCore));
            _core.SelectionChanged += OnCoreSelectionChanged;
            _initializing = false;
        }

        // Saves immediately on every change rather than needing a Save button - there is nothing here worth staging and discarding.
        private PathPickerRow Picker(string name, string placeholder, string title, string? current, System.Action<string> apply)
        {
            var picker = new PathPickerRow
            {
                Name = name,
                Placeholder = placeholder,
                BrowseTitle = title,
                Path = current ?? "",
            };

            picker.PathPicked += picked =>
            {
                apply(picked);
                _settings.Save();
            };

            return picker;
        }

        private void OnCoreSelectionChanged(object? sender, SelectionChangedEventArgs e)
        {
            // Fires once as a side effect of setting ItemsSource/SelectedIndex above; skip that spurious event.
            if (_initializing) return;
            if (_core.SelectedItem is not string selected) return;

            _settings.SelectedCore = selected;
            _settings.Save();
        }
    }
}
