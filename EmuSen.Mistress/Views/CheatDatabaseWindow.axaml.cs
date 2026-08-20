using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using EmuSen.LunaP.Windowing;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Models;

namespace EmuSen.Mistress.Views
{
    // A named model so the count is a field, not something parsed back out - §4.11a.
    public sealed record CheatSystemRow(string System, int Count);

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

        // Which cheat-database folders this build's cores can use. Injected
        // rather than known here - see §4.16.
        private readonly Func<IReadOnlyCollection<string>>? _supportedSystems;

        // The plan the Prune button is currently offering to carry out, set
        // by its first click and consumed by its second. Null means the
        // button is back to being an offer rather than a confirmation.
        private CheatPrunePlan? _armedPrune;

        // One scan per folder, reused by both lists - see CheatDatabase.
        private CheatDatabase _db;

        private IReadOnlyList<CheatDatabaseEntry> _games = Array.Empty<CheatDatabaseEntry>();
        private string? _selectedSystem;

        // Narrows the systems list to one console's folders - see EmuSen_Multicore.md §10.
        private string? _console;

        // Called when the library's console filter moves under an already-open window.
        public void SetConsole(string? console)
        {
            _console = console;
            Refresh();
        }

        // The libretro folder names the selected console claims, or null for no narrowing.
        private IReadOnlyList<string>? ConsoleSystems =>
            EmuSen.Cores.CoreCatalog.ByDisplayName(_console) is { } core && core.CheatSystemNames.Count > 0
                ? core.CheatSystemNames
                : null;

        public CheatDatabaseWindow() : this(AppSettings.Load()) { }

        public CheatDatabaseWindow(AppSettings settings, Func<CheatRegistry?>? activeCheats = null, ICheatCodeCodec? codec = null,
            Action? changed = null, Action? openActiveCheats = null,
            Func<IReadOnlyCollection<string>>? supportedSystems = null, string? console = null)
        {
            InitializeComponent();
            SystemsList.Label = r => $"{r.System}  ({r.Count})";
            SystemsList.Key = r => r.System;
            _settings = settings;
            _activeCheats = activeCheats;
            _codec = codec;
            _changed = changed;
            _openActiveCheats = openActiveCheats;
            _supportedSystems = supportedSystems;
            _console = console;
            _db = new CheatDatabase(Directory);
            AttributionText.Text = CheatDatabaseInstaller.Attribution;
            DirectoryPicker.Path = _settings.CheatDatabaseDirectory ?? "";
            DirectoryPicker.PathPicked += OnDirectoryPicked;
            ActiveCheatsButton.IsEnabled = _openActiveCheats is not null;
            PruneButton.IsEnabled = _supportedSystems is not null;

            // Before Refresh, so the first ShowGames already has it - see EmuSen_Settings_Reference.md §4.23.
            GameFilter.SearchText = _settings.CheatSearch;
            Refresh();

            // FilterBar owns the "a Text set from code counts too" detail this used to spell out - see EmuSen_LunaP.md §14.2.
            GameFilter.Changed += ShowGames;

            // Once, rather than the per-keystroke write saving on Changed would be.
            Closing += (_, _) =>
            {
                _settings.CheatSearch = GameFilter.SearchText;
                _settings.Save();
            };
        }

        // AppSettings when set, the sandbox's own Cheats folder otherwise -
        // the same resolution `cheat db` uses.
        private string Directory =>
            string.IsNullOrWhiteSpace(_settings.CheatDatabaseDirectory)
                ? DianaOSSandbox.CheatDatabaseDirectory
                : _settings.CheatDatabaseDirectory;

        // Every system on disk, or only the selected console's when one is chosen.
        private IReadOnlyList<(string System, int Count)> VisibleSystems()
        {
            IReadOnlyList<(string System, int Count)> all = _db.Systems();
            IReadOnlyList<string>? only = ConsoleSystems;
            if (only is null) return all;

            return all.Where(s => only.Contains(s.System, StringComparer.OrdinalIgnoreCase)).ToList();
        }

        private void Refresh()
        {
            _db = new CheatDatabase(Directory);
            IReadOnlyList<(string System, int Count)> systems = VisibleSystems();
            int total = systems.Sum(s => s.Count);

            SystemsList.Refresh(systems.Select(s => new CheatSystemRow(s.System, s.Count)));
            SystemsList.SelectedIndex = -1; // Refresh restores by Key; this window starts with none.

            string scope = ConsoleSystems is null ? "" : $" for {_console}";
            StatusText.Text = total > 0
                ? $"{total:N0} cheat file(s){scope} in {Directory}"
                : $"No cheat files{scope} in {Directory}";

            _selectedSystem = null;
            ShowGames();
        }

        private void OnSystemSelected(object? sender, SelectionChangedEventArgs e)
        {
            _selectedSystem = SystemsList.Selected?.System;
            ShowGames();
        }

        private void ShowGames()
        {
            _games = _selectedSystem is null
                ? Array.Empty<CheatDatabaseEntry>()
                : _db.Games(_selectedSystem, GameFilter.SearchText);

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

        // Deleting is irreversible and the only way back is a 250MB
        // download, so the first click only ever reports - see §4.16.
        private void OnPruneClick(object? sender, RoutedEventArgs e)
        {
            if (_armedPrune is CheatPrunePlan armed)
            {
                (int removed, IReadOnlyList<string> failed) = CheatDatabasePruner.Apply(_db, armed);
                DisarmPrune();
                Refresh();

                string trouble = failed.Count > 0 ? $"  Couldn't remove: {string.Join("; ", failed)}" : "";
                StatusText.Text = $"Deleted {removed} system(s), {armed.Files:N0} file(s), {CheatPrunePlan.Human(armed.Bytes)}.{trouble}";
                return;
            }

            CheatPrunePlan plan = CheatDatabasePruner.Plan(_db, _supportedSystems?.Invoke() ?? Array.Empty<string>());
            if (!plan.CanApply)
            {
                StatusText.Text = $"Nothing to prune - {plan.Reason}";
                return;
            }

            _armedPrune = plan;
            PruneButton.Content = $"Delete {plan.Removing.Count} system(s)?";
            StatusText.Text = $"{plan.Removing.Count} system(s) have no core in this build - {plan.Files:N0} file(s), " +
                              $"{CheatPrunePlan.Human(plan.Bytes)}. Keeping {string.Join(", ", plan.Keeping)}. " +
                              "Click again to delete them for good, or Close to leave them alone.";
        }

        private void DisarmPrune()
        {
            _armedPrune = null;
            PruneButton.Content = "Prune Unsupported";
        }

        // Offered where the download lands, since that is the moment the
        // 250MB actually arrives - see §4.16.
        private void OfferPrune()
        {
            CheatPrunePlan plan = CheatDatabasePruner.Plan(_db, _supportedSystems?.Invoke() ?? Array.Empty<string>());
            if (!plan.CanApply) return;

            _armedPrune = plan;
            PruneButton.Content = $"Delete {plan.Removing.Count} system(s)?";
            StatusText.Text = $"{StatusText.Text}  -  {plan.Removing.Count} of {plan.Removing.Count + plan.Keeping.Count} " +
                              $"system(s) have no core in this build ({plan.Files:N0} file(s), {CheatPrunePlan.Human(plan.Bytes)}). " +
                              "Prune Unsupported deletes them.";
        }

        // PathPickerRow raises this only for a real pick, never for a cancel, so there is no
        // null to check and no "did they actually choose something" branch. The folder dialog,
        // the read-only box and the Browse button are all the control's now.
        private void OnDirectoryPicked(string picked)
        {
            _settings.CheatDatabaseDirectory = picked;
            _settings.Save();
            // The armed plan was measured against the old folder.
            DisarmPrune();
            Refresh();
        }

        private async void OnDownloadClick(object? sender, RoutedEventArgs e)
        {
            DownloadButton.IsEnabled = false;
            DirectoryPicker.IsEnabled = false;
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
                OfferPrune();
            }
            catch (Exception ex)
            {
                StatusText.Text = $"Download failed: {ex.Message}";
            }
            finally
            {
                DownloadButton.IsEnabled = true;
                DirectoryPicker.IsEnabled = true;
            }
        }

        private void OnCloseClick(object? sender, RoutedEventArgs e) => Close();
    }
}
