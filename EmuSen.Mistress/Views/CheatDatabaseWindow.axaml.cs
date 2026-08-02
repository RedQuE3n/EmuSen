using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Models;

namespace EmuSen.Mistress.Views
{
    // The GUI for `cheat db` - see `man cheat`. EmuSen redistributes no
    // cheat data: this either indexes a folder the user already has, or
    // downloads one to their machine on their explicit request.
    public partial class CheatDatabaseWindow : Window
    {
        private readonly AppSettings _settings;

        // Resolved per use, never cached - the frontend's registry outlives
        // this window and is replaced on a ROM change. See §4.14.
        private readonly Func<CheatRegistry?>? _activeCheats;

        private readonly ICheatCodeCodec? _codec;

        // Raised after a load, so an Active Cheats window already open shows
        // the new list without being reopened.
        private readonly Action? _changed;

        // Opens or focuses the Active Cheats window. Owned by MainWindow,
        // not by this one - both are its children and only one may exist.
        private readonly Action? _openActiveCheats;

        // One scan per folder, reused by both lists - see CheatDatabase.
        private CheatDatabase _db;

        private IReadOnlyList<CheatDatabaseEntry> _games = Array.Empty<CheatDatabaseEntry>();
        private string? _selectedSystem;

        public CheatDatabaseWindow() : this(AppSettings.Load()) { }

        public CheatDatabaseWindow(AppSettings settings, Func<CheatRegistry?>? activeCheats = null, ICheatCodeCodec? codec = null,
            Action? changed = null, Action? openActiveCheats = null)
        {
            InitializeComponent();
            _settings = settings;
            _activeCheats = activeCheats;
            _codec = codec;
            _changed = changed;
            _openActiveCheats = openActiveCheats;
            _db = new CheatDatabase(Directory);
            AttributionText.Text = CheatDatabaseInstaller.Attribution;
            DirectoryBox.Text = _settings.CheatDatabaseDirectory;
            ActiveCheatsButton.IsEnabled = _openActiveCheats is not null;
            Refresh();

            // The property, not the TextChanged event - only this reacts to a
            // Text set that didn't come from typing.
            GameFilterBox.PropertyChanged += (_, args) =>
            {
                if (args.Property == TextBox.TextProperty) ShowGames();
            };
        }

        // AppSettings when set, the sandbox's own Cheats folder otherwise -
        // the same resolution `cheat db` uses.
        private string Directory =>
            string.IsNullOrWhiteSpace(_settings.CheatDatabaseDirectory)
                ? DianaOSSandbox.CheatDatabaseDirectory
                : _settings.CheatDatabaseDirectory;

        private void Refresh()
        {
            _db = new CheatDatabase(Directory);
            IReadOnlyList<(string System, int Count)> systems = _db.Systems();
            int total = systems.Sum(s => s.Count);

            SystemsList.ItemsSource = systems.Select(s => $"{s.System}  ({s.Count})").ToList();
            StatusText.Text = total > 0
                ? $"{total:N0} cheat file(s) in {Directory}"
                : $"No cheat files in {Directory}";

            _selectedSystem = null;
            ShowGames();
        }

        // The list label carries its own count, so the name has to come back
        // off it rather than out of the ListBox's own string.
        private void OnSystemSelected(object? sender, SelectionChangedEventArgs e)
        {
            int index = SystemsList.SelectedIndex;
            IReadOnlyList<(string System, int Count)> systems = _db.Systems();

            _selectedSystem = index >= 0 && index < systems.Count ? systems[index].System : null;
            ShowGames();
        }

        private void ShowGames()
        {
            _games = _selectedSystem is null
                ? Array.Empty<CheatDatabaseEntry>()
                : _db.Games(_selectedSystem, GameFilterBox.Text);

            GamesList.ItemsSource = _games.Select(g => g.Game).ToList();

            GamesHeaderText.Text = _selectedSystem is null
                ? "Games"
                : $"Games in {_selectedSystem}  ({_games.Count:N0})";

            UpdateLoadButton();
        }

        private void OnGameSelected(object? sender, SelectionChangedEventArgs e) => UpdateLoadButton();

        private void UpdateLoadButton() =>
            LoadGameButton.IsEnabled = SelectedGame is not null && _activeCheats?.Invoke() is not null;

        private CheatDatabaseEntry? SelectedGame =>
            GamesList.SelectedIndex is int i && i >= 0 && i < _games.Count ? _games[i] : null;

        private void OnGameActivated(object? sender, RoutedEventArgs e) => LoadSelectedGame();

        private void OnLoadGameClick(object? sender, RoutedEventArgs e) => LoadSelectedGame();

        // Replaces the active list rather than merging, so picking a game
        // twice cannot double it up - see §4.14.
        private void LoadSelectedGame()
        {
            if (SelectedGame is not CheatDatabaseEntry game) return;

            if (_activeCheats?.Invoke() is not CheatRegistry registry)
            {
                StatusText.Text = "No cheat list to load into.";
                return;
            }

            CheatImportResult result;
            try { result = CheatImport.FromChtFile(registry, game.Path, _codec, replace: true); }
            catch (Exception ex)
            {
                StatusText.Text = $"Couldn't read {game.Game}: {ex.Message}";
                return;
            }

            _changed?.Invoke();

            string skipped = result.Skipped > 0 ? $", {result.Skipped} skipped" : "";
            StatusText.Text = result.Loaded > 0
                ? $"Loaded {result.Loaded} cheat(s) for {game.Game}{skipped} - all disabled, turn them on under Active Cheats..."
                : $"No usable cheats in {game.Game}{skipped}";
        }

        // Where a just-loaded list is actually turned on, so it is a button
        // here rather than a trip back to the menu bar - see §4.14.
        private void OnActiveCheatsClick(object? sender, RoutedEventArgs e) => _openActiveCheats?.Invoke();

        private async void OnBrowseClick(object? sender, RoutedEventArgs e)
        {
            IReadOnlyList<IStorageFolder> folders = await StorageProvider.OpenFolderPickerAsync(new FolderPickerOpenOptions
            {
                Title = "Choose a cheat folder (a RetroArch cheats folder works as-is)",
                AllowMultiple = false,
            });

            if (folders.FirstOrDefault()?.Path.LocalPath is not string picked) return;

            _settings.CheatDatabaseDirectory = picked;
            _settings.Save();
            DirectoryBox.Text = picked;
            Refresh();
        }

        private async void OnDownloadClick(object? sender, RoutedEventArgs e)
        {
            DownloadButton.IsEnabled = false;
            BrowseButton.IsEnabled = false;
            StatusText.Text = "Downloading from libretro...";

            string target = Directory;

            try
            {
                // Off the UI thread - this is a real network fetch of a
                // multi-megabyte archive.
                CheatDatabaseInstallResult result = await Task.Run(() =>
                {
                    using Stream zip = CheatDatabaseInstaller.Fetch(CheatDatabaseInstaller.LibretroCheatsUrl);
                    return CheatDatabaseInstaller.Install(zip, target);
                });

                Refresh();
                StatusText.Text = $"Installed {result.Installed:N0} cheat file(s) into {target}";
            }
            catch (Exception ex)
            {
                StatusText.Text = $"Download failed: {ex.Message}";
            }
            finally
            {
                DownloadButton.IsEnabled = true;
                BrowseButton.IsEnabled = true;
            }
        }

        private void OnCloseClick(object? sender, RoutedEventArgs e) => Close();
    }
}
