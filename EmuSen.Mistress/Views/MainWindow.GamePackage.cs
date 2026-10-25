using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // Make Package and Add Patch for a game of the library, from its menu, the game options sheet and the pad - see EmuSen_GamePackages.md §15.
    public partial class MainWindow
    {
        private GamePackageWindow? _gamePackage;

        public GamePackageWindow? GamePackageShown => _gamePackage;

        // The game's package window; a package made or changed scans the library again when it closes.
        private void ShowGamePackage(string path, string title)
        {
            if (_gamePackage is not null) return;
            string? package = EmuSen.Galaxia.Media.GamePackages.RootFor(path);
            // A claimed file its package will not open with plays as a plain game, and the window says the package is changed - see EmuSen_GamePackages.md §14.8 Q34.
            string? claimant = package is null && File.Exists(path) ? EmuSen.Galaxia.Media.GamePackages.Index(EmuSen.Galaxia.Media.PackagePlaces.Current()).Claimant(Path.GetFullPath(path))?.Folder : null;
            var window = _gamePackage = new GamePackageWindow(title, path, _appSettings.RomDirectory, package ?? claimant, browse: !_bigScreen, refused: claimant is not null);
            window.Closed += (_, _) =>
            {
                if (ReferenceEquals(_gamePackage, window)) _gamePackage = null;
                if (window.Changed && !_recordsClosed && LibraryView.IsVisible) RefreshLibrary();
            };
            _ = SheetLayer.Show(window, this);
        }
    }
}
