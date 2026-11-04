using System;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Library;
using EmuSen.Serenity;

namespace EmuSen.Mistress.Views
{
    // A crop kept with one game, which replaces its console's while that game runs - see EmuSen_Settings_Reference.md §4.100.
    public partial class MainWindow
    {
        public const string CropThisGame = "This game's crop";

        // Offered for a game of a console that has an overscan to replace.
        private static bool CropOffered(string path) => EmuSen.Mistress.Library.RuntimeCatalog.ConsoleForRom(path) is { } console && ConsoleCrop.HasOverscan(console);

        private bool IsRunning(string path) => _session is not null && string.Equals(path, _currentRomPath, StringComparison.Ordinal);

        // The running game's own crop where it has one and its console an overscan, else the console's.
        private PictureCrop RunningCrop(string console) =>
            ConsoleCrop.HasOverscan(console) && _currentRomPath is string rom && _records.Crop(rom) is { } own ? own.Picture : ConsoleCrop.CropFor(_graphics, console);

        // Graphics Settings' line under a console's Overscan row while the game running on it has a crop of its own.
        private string? RunningGameCropNote(string console) =>
            _session is not null && console == _activeConsole && ConsoleCrop.HasOverscan(console) && _currentRomPath is string rom && _records.Crop(rom) is not null
                ? $"{RunningGameTitle()} has a crop of its own, which replaces this while it runs. Change it with {CropThisGame} in the Pause Card's Display."
                : null;

        // The game's page's door: Settings at the game's own crop, the switch and the four edges of Settings' one description (EmuSen_Interface.md §20).
        private void ShowGameCrop(string path, string title)
        {
            if ((IsRunning(path) ? _activeConsole : EmuSen.Mistress.Library.RuntimeCatalog.ConsoleForRom(path)) is null) return;
            ShowSettings(SettingsScreen.Display);
            if (_settings is not { } view) return;
            view.Model.OpenGameCrop(path, title);
            RefreshSettings();
        }
    }
}
