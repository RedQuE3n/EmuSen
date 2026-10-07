using System;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Serenity;

namespace EmuSen.Mistress.Views
{
    // A crop kept with one game, which replaces its console's while that game runs - see EmuSen_Settings_Reference.md §4.100.
    public partial class MainWindow
    {
        public const string CropThisGame = "Crop This Game...";

        private GameCropWindow? _gameCrop;

        public GameCropWindow? GameCropShown => _gameCrop;

        // Offered for a game of a console that has an overscan to replace.
        private static bool CropOffered(string path) => EmuSen.Cores.CoreCatalog.ConsoleForRom(path) is { } console && GraphicsSettingsWindow.HasOverscan(console);

        private bool IsRunning(string path) => _session is { IsRomLoaded: true } && string.Equals(path, _currentRomPath, StringComparison.Ordinal);

        // The running game's own crop where it has one and its console an overscan, else the console's.
        private PictureCrop RunningCrop(string console) =>
            GraphicsSettingsWindow.HasOverscan(console) && _currentRomPath is string rom && _records.Crop(rom) is { } own ? own.Picture : GraphicsSettingsWindow.CropFor(_graphics, console);

        // Graphics Settings' line under a console's Overscan row while the game running on it has a crop of its own.
        private string? RunningGameCropNote(string console) =>
            _session is { IsRomLoaded: true } && console == _activeConsole && GraphicsSettingsWindow.HasOverscan(console) && _currentRomPath is string rom && _records.Crop(rom) is not null
                ? $"{RunningGameTitle()} has a crop of its own, which replaces this while it runs. Change it with {CropThisGame.TrimEnd('.')} in the game's options."
                : null;

        private void ShowGameCrop(string path, string title)
        {
            if (_gameCrop is not null || (IsRunning(path) ? _activeConsole : EmuSen.Cores.CoreCatalog.ConsoleForRom(path)) is not { } console) return;
            var window = _gameCrop = new GameCropWindow(title, console, GraphicsSettingsWindow.CropFor(_graphics, console), _records.Crop(path), crop =>
            {
                if (crop is { } own) _records.SetCrop(path, own);
                else _records.ClearCrop(path);
                // A running game follows each number as it is typed, as it follows its console's.
                if (IsRunning(path)) ApplyScreenFilter(_activeConsole);
            });
            window.Closed += (_, _) => { if (ReferenceEquals(_gameCrop, window)) _gameCrop = null; };
            _ = SheetLayer.Show(window, this);
        }
    }
}
