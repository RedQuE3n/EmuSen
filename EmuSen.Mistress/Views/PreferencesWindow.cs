using System.IO;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.Common.Firmware;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Theme;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // OpenEmu's preference panes, as tabs: Library, Gameplay, Appearance, System Files - see EmuSen_Settings_Reference.md §4.36.
    public class PreferencesWindow : ToolWindow
    {
        private static readonly (string Value, string Text)[] ResumeChoices =
        {
            (AppSettings.ResumeAsk, "Ask each time"),
            (AppSettings.ResumeAlways, "Always resume"),
            (AppSettings.ResumeNever, "Always start again"),
        };

        private readonly AppSettings _settings;
        private readonly LunaSwitch _bigScreen = new() { Name = "BigScreenSwitch", Label = "Start in big screen mode" };
        private readonly LunaSwitch _pauseInBackground = new() { Name = "PauseInBackgroundSwitch", Label = "Pause the game when another window is in front" };
        private readonly Dropdown _theme = new() { Name = "ThemeDropdown", HorizontalAlignment = HorizontalAlignment.Stretch };
        private readonly Dropdown _resume = new() { Name = "ResumeDropdown", HorizontalAlignment = HorizontalAlignment.Stretch };

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public PreferencesWindow() : this(new AppSettings()) { }

        public PreferencesWindow(AppSettings settings)
        {
            _settings = settings;

            Title = "Preferences";
            Width = 660;
            CanResize = false;

            // Sized to its content rather than a fixed height: a fixed one put the Close button below the edge. See EmuSen_LunaP.md §11.1.
            SizeToContent = SizeToContent.Height;

            var tabs = new Tabs { Name = "PreferenceTabs" };
            tabs.Add("Library", Pane(
                new FieldRow
                {
                    Label = "ROM Directory",
                    Hint = "The folder the library lists, and where Open ROM... starts. It is only ever read.",
                    Content = Picker("RomDirectoryBox", "(not set)", "Choose ROM Directory", _settings.RomDirectory, p => _settings.RomDirectory = p),
                },
                new FieldRow
                {
                    Label = "Cover Art Directory",
                    Hint = "Box art, named like the game's file (libretro-thumbnails' names work as they are). Leave blank for home/Artwork.",
                    Content = Picker("ArtworkDirectoryBox", "(default)", "Choose Cover Art Directory", _settings.ArtworkDirectory, p => _settings.ArtworkDirectory = p),
                },
                new FieldRow
                {
                    Label = "Save State Directory",
                    Hint = "Where save states, the state a game was left in, and their pictures go. Leave blank for home/Saves/Save States.",
                    Content = Picker("StateDirectoryBox", "(default)", "Choose Save State Directory", _settings.StateDirectory, p => _settings.StateDirectory = p),
                },
                new FieldRow
                {
                    Label = "Log Directory",
                    Hint = "Where per-session log files are written. Leave blank to disable file logging entirely.",
                    Content = Picker("LogDirectoryBox", "(not set)", "Choose Log Directory", _settings.LogDirectory, p => _settings.LogDirectory = p),
                }));
            tabs.Add("Gameplay", Pane(
                new FieldRow
                {
                    Label = "Continue Where You Left Off",
                    Hint = "Every game is saved when it is closed. This is what happens the next time it starts.",
                    Content = _resume,
                },
                new FieldRow
                {
                    Label = "In the Background",
                    Hint = "A game this window paused starts again when the window comes back to the front.",
                    Content = _pauseInBackground,
                },
                new FieldRow
                {
                    Label = "Big Screen",
                    Hint = "Start full screen with no menu bar and larger text, for a handheld or a television; a controller opens the menu with Start. Takes effect at the next start.",
                    Content = _bigScreen,
                }));
            tabs.Add("Appearance", Pane(
                new FieldRow
                {
                    Label = "Theme",
                    Hint = "Colours and fonts for every window. Drop a theme file in the themes folder and it appears here; a change applies without a restart.",
                    Content = _theme,
                }));
            tabs.Add("System Files", Pane(SystemFiles()));

            Content = Ui.Stack(12, tabs, Ui.Buttons(Ui.Button("Close", Close))).Margin(16);

            _bigScreen.IsChecked = _settings.BigScreen;
            _bigScreen.IsCheckedChanged += (_, _) => { _settings.BigScreen = _bigScreen.IsChecked == true; _settings.Save(); };
            _pauseInBackground.IsChecked = _settings.PauseInBackground;
            _pauseInBackground.IsCheckedChanged += (_, _) => { _settings.PauseInBackground = _pauseInBackground.IsChecked == true; _settings.Save(); };

            string[] resumeTexts = ResumeChoices.Select(c => c.Text).ToArray();
            _resume.Fill(resumeTexts, ResumeChoices.FirstOrDefault(c => c.Value == _settings.ResumeOnLaunch).Text ?? resumeTexts[0]);
            _resume.Chose += chosen =>
            {
                if (ResumeChoices.FirstOrDefault(c => c.Text == chosen as string).Value is not string value) return;
                _settings.ResumeOnLaunch = value;
                _settings.Save();
            };

            _theme.Fill(LunaTheme.Available(), LunaTheme.Current);
            _theme.Chose += ChoseTheme;
        }

        private static Control Pane(params Control[] rows) => Ui.Stack(12, rows).Margin(4, 12, 4, 4);

        // Nothing to install here: a game that needs a chip's dump asks for it the first time it starts - see EmuSen_Firmware.md §3.
        private static Control SystemFiles()
        {
            string directory = FirmwareLibrary.Directory;
            string[] files = Directory.Exists(directory) ? Directory.GetFiles(directory).Select(Path.GetFileName).OfType<string>().OrderBy(f => f).ToArray() : System.Array.Empty<string>();
            Control list = files.Length == 0
                ? new EmptyState { Message = "No system files installed.", Detail = "A game that needs a coprocessor's firmware asks for it the first time it starts." }
                : Ui.Stack(4, files.Select(f => (Control)Ui.Mono(f)).ToArray());
            return new FieldRow { Label = "Firmware", Hint = $"Installed in {directory}", Content = list };
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

        // A theme that will not load leaves the old one applied, so the row has to go back to it - see EmuSen_Settings_Reference.md §4.25.
        private void ChoseTheme(object? chosen)
        {
            if (chosen is not string name) return;
            if (!LunaTheme.Apply(name)) _theme.Fill(LunaTheme.Available(), LunaTheme.Current);
        }
    }
}
