using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Interactivity;
using System.Text.Json;
using EmuSen.LunaP.Windowing;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Library;

namespace EmuSen.Mistress.Views
{
    // A named model so the count is a field, not something parsed back out - §4.11a.
    public sealed record CheatSystemRow(string System, int Count);

    // The GUI for `cheat db` - see `man cheat`. EmuSen redistributes no
    // cheat data: this either indexes a folder the player already has, or
    // downloads one to their machine on their explicit request.
    public partial class CheatDatabaseWindow : ToolWindow
    {
        private readonly AppSettings _settings;

        // The list a game's cheats load into: the running session's, or the one held for the next game - see §4.14.
        private readonly CheatList? _cheats;

        // The running session, whose codecs read a game's codes; null with no game, when the database's own ops answer on session 0.
        private readonly Func<CrystalSession?> _session;

        // Raised after a load, so an Active Cheats window already open shows
        // the new list without being reopened.
        private readonly Action? _changed;

        // Opens or focuses the Active Cheats window. Owned by MainWindow,
        // not by this one - both are its children and only one may exist.
        private readonly Action? _openActiveCheats;

        // The plan the Prune button offers, set by its first click and confirmed by its second.
        private bool _armedPrune;

        private IReadOnlyList<(string Game, string Path)> _games = Array.Empty<(string, string)>();
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
            EmuSen.Mistress.Library.RuntimeCatalog.ByDisplayName(_console) is { } core && core.CheatSystems.Count > 0
                ? core.CheatSystems
                : null;

        public CheatDatabaseWindow() : this(AppSettings.Load()) { }

        public CheatDatabaseWindow(AppSettings settings, CheatList? cheats = null, Func<CrystalSession?>? session = null,
            Action? changed = null, Action? openActiveCheats = null, string? console = null)
        {
            InitializeComponent();
            if (Content is DockPanel body) EmuSen.LunaP.Controls.MenuLook.WhenApplied(body, () => InLook(body));
            EmuSen.LunaP.Controls.MenuLook.SetWidthFraction(this, 0.8);
            SystemsList.Label = r => r.Count > 0 ? $"{r.System}  ({r.Count})" : $"{r.System}  (not downloaded)";
            SystemsList.Key = r => r.System;
            _settings = settings;
            _cheats = cheats;
            _session = session ?? (() => null);
            _changed = changed;
            _openActiveCheats = openActiveCheats;
            _console = console;
            AttributionText.Text = Attribution;
            DirectoryPicker.Path = _settings.CheatDatabaseDirectory ?? "";
            DirectoryPicker.PathPicked += OnDirectoryPicked;
            // A folder typed, and a game loaded with Enter, for a pad in Game Mode - see EmuSen_Settings_Reference.md §4.45.5.
            DirectoryPicker.IsEditable = true;
            GamesList.AddHandler(KeyDownEvent, (_, e) => { if (e.Key == Avalonia.Input.Key.Enter) { e.Handled = true; LoadSelectedGame(); } }, handledEventsToo: true);
            ActiveCheatsButton.IsEnabled = _openActiveCheats is not null;

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

        // A database op through the runtime: on the running session where one is, else on session 0, which answers those that need no game - see EmuSen_Backend.md §14.12.
        private string Op(uint op, object? args = null)
        {
            string json = args is null ? "{}" : JsonSerializer.Serialize(args);
            return _session() is { Closed: false } s ? s.CheatOp(op, json) : CoreSettings.Runtime.CheatOp(op, json);
        }

        // The licence's words the database's status carries.
        private string Attribution
        {
            get
            {
                try
                {
                    using JsonDocument status = JsonDocument.Parse(Op(CrystalSessionAbi.CheatDbStatus));
                    return status.RootElement.TryGetProperty("attribution", out JsonElement a) ? a.GetString() ?? "" : "";
                }
                catch (CrystalException) { return ""; }
            }
        }

        private static string Human(long bytes) =>
            bytes >= 1 << 30 ? $"{bytes / (double)(1 << 30):F1} GB" : bytes >= 1 << 20 ? $"{bytes / (double)(1 << 20):F1} MB" : bytes >= 1 << 10 ? $"{bytes / 1024.0:F1} KB" : $"{bytes} bytes";

        // On a big-screen sheet in ES-DE's look: the status over centred buttons, and the folder typed rather than browsed, as a pad cannot drive the file dialog (§4.80).
        private void InLook(DockPanel dock)
        {
            {
                dock.Margin = new Avalonia.Thickness(0, 0, 0, 16);
                // The folder's label beside its box, as a menu row's is, and the two lists given equal room.
                if (dock.Children.OfType<TextBlock>().FirstOrDefault(t => t.Text == "Cheat Folder") is { } label)
                {
                    int at = dock.Children.IndexOf(label);
                    dock.Children.Remove(label);
                    dock.Children.Remove(DirectoryPicker);
                    label.VerticalAlignment = Avalonia.Layout.VerticalAlignment.Center;
                    label.Margin = new Avalonia.Thickness(0, 0, 16, 0);
                    Grid.SetColumn(DirectoryPicker, 1);
                    var row = new Grid { ColumnDefinitions = new ColumnDefinitions("Auto,*"), Margin = DirectoryPicker.Margin, Children = { label, DirectoryPicker } };
                    DirectoryPicker.Margin = default;
                    DockPanel.SetDock(row, Dock.Top);
                    dock.Children.Insert(at, row);
                }
                if (dock.Children.OfType<Grid>().LastOrDefault() is { ColumnDefinitions.Count: 3 } lists) lists.ColumnDefinitions = new ColumnDefinitions("*,16,*");
            }
            SheetLook.StatusOverButtons(StatusText);
            // The introduction and the licence's attribution stay in view as the panel's footer, leaving the lists the room (Q170).
            if (AttributionText.Parent is Border attribution) attribution.IsVisible = false;
            string intro = dock.Children.OfType<TextBlock>().FirstOrDefault(t => t.Text?.StartsWith("EmuSen ships no cheat data", StringComparison.Ordinal) == true) is { } first ? first.Text! : "";
            if (dock.Children.OfType<TextBlock>().FirstOrDefault(t => t.Text == intro) is { } shownIntro) shownIntro.IsVisible = false;
            EmuSen.LunaP.Controls.MenuLook.SetFooterLines(this, 4);
            EmuSen.LunaP.Controls.MenuLook.SetFooter(this, (intro + " " + Attribution).Trim());
            GamesHeaderText.TextWrapping = Avalonia.Media.TextWrapping.Wrap;
            _inLook = true;
            ShowGames();
            SheetLook.SelectOnFocus(GamesList);
            SheetLook.SelectOnFocus(SystemsList);
            // The filter and Load side by side above the games, so the list keeps the room (§4.83).
            if (GameFilter.Parent is DockPanel games)
            {
                Avalonia.Thickness filterMargin = GameFilter.Margin;
                games.Children.Remove(GameFilter);
                games.Children.Remove(LoadGameButton);
                GameFilter.Margin = default;
                LoadGameButton.Margin = default;
                LoadGameButton.HorizontalAlignment = Avalonia.Layout.HorizontalAlignment.Stretch;
                Grid.SetColumn(LoadGameButton, 2);
                var tools = new Grid { ColumnDefinitions = new ColumnDefinitions("*,8,Auto"), Margin = filterMargin, Children = { GameFilter, LoadGameButton } };
                DockPanel.SetDock(tools, Dock.Top);
                games.Children.Insert(games.Children.IndexOf(GamesHeaderText) + 1, tools);
            }
            DirectoryPicker.TemplateApplied += (_, e) => { if (e.NameScope.Find<Control>("PART_Browse") is { } browse) browse.IsVisible = false; };
            if (Avalonia.VisualTree.VisualExtensions.GetVisualDescendants(DirectoryPicker).OfType<Control>().FirstOrDefault(c => c.Name == "PART_Browse") is { } shown) shown.IsVisible = false;
        }

        private bool _inLook;

        // The same resolution `cheat db` uses - see EmuSen_Config_Reference.md §7.
        private string Directory => SettingFolders.Cheats(_settings);

        // Every system on disk, or only the selected console's when one is chosen.
        private IReadOnlyList<(string System, int Count)> VisibleSystems()
        {
            IReadOnlyList<(string System, int Count)> all;
            try
            {
                using JsonDocument systems = JsonDocument.Parse(Op(CrystalSessionAbi.CheatDbSystems));
                all = systems.RootElement.EnumerateArray().Select(e => (e.GetProperty("system").GetString() ?? "", e.GetProperty("count").GetInt32())).ToArray();
            }
            catch (CrystalException) { all = Array.Empty<(string, int)>(); }
            IReadOnlyList<string>? only = ConsoleSystems;
            if (only is null) return all;

            return all.Where(s => only.Contains(s.System, StringComparer.OrdinalIgnoreCase)).ToList();
        }

        // The folders a playable console claims that this folder lacks, as a pruned database does for a console added since - see EmuSen_Settings_Reference.md §4.95.
        private IReadOnlyList<string> MissingSystems(IReadOnlyList<(string System, int Count)> shown)
        {
            IReadOnlyCollection<string> supported = EmuSen.Mistress.Library.RuntimeCatalog.SupportedCheatSystems;
            IReadOnlyList<string>? only = ConsoleSystems;

            return supported
                .Where(s => only is null || only.Contains(s, StringComparer.OrdinalIgnoreCase))
                .Where(s => !shown.Any(have => have.System.Equals(s, StringComparison.OrdinalIgnoreCase)))
                .OrderBy(s => s, StringComparer.OrdinalIgnoreCase)
                .ToList();
        }

        private void Refresh()
        {
            IReadOnlyList<(string System, int Count)> systems = VisibleSystems();
            int total = systems.Sum(s => s.Count);

            IReadOnlyList<string> missing = MissingSystems(systems);
            SystemsList.Refresh(systems.Select(s => new CheatSystemRow(s.System, s.Count)).Concat(missing.Select(s => new CheatSystemRow(s, 0))));
            SystemsList.SelectedIndex = -1; // Refresh restores by Key; this window starts with none.

            string scope = ConsoleSystems is null ? "" : $" for {_console}";
            StatusText.Text = total > 0
                ? $"{total:N0} cheat file(s){scope} in {Directory}"
                : $"No cheat files{scope} in {Directory}";
            if (missing.Count > 0) StatusText.Text += $"  -  {string.Join(", ", missing)} not downloaded: Download fetches {(missing.Count == 1 ? "it" : "them")}.";

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
            _games = Array.Empty<(string, string)>();
            if (_selectedSystem is not null)
            {
                try
                {
                    using JsonDocument games = JsonDocument.Parse(Op(CrystalSessionAbi.CheatDbGames, new { system = _selectedSystem, filter = GameFilter.SearchText }));
                    _games = games.RootElement.EnumerateArray().Select(g => (g.GetProperty("game").GetString() ?? "", g.GetProperty("path").GetString() ?? "")).ToArray();
                }
                catch (CrystalException) { }
            }

            GamesList.ItemsSource = _games.Select(g => g.Game).ToList();

            // On a big-screen sheet the chosen system is the row beside it, so the heading need not name it again (§4.83).
            GamesHeaderText.Text = _selectedSystem is null
                ? "Games"
                : _inLook ? $"Games  ({_games.Count:N0})" : $"Games in {_selectedSystem}  ({_games.Count:N0})";

            UpdateLoadButton();
        }

        private void OnGameSelected(object? sender, SelectionChangedEventArgs e) => UpdateLoadButton();

        private void UpdateLoadButton() =>
            LoadGameButton.IsEnabled = SelectedGame is not null && _cheats is not null;

        private (string Game, string Path)? SelectedGame =>
            GamesList.SelectedIndex is int i && i >= 0 && i < _games.Count ? _games[i] : null;

        private void OnGameActivated(object? sender, RoutedEventArgs e) => LoadSelectedGame();

        private void OnLoadGameClick(object? sender, RoutedEventArgs e) => LoadSelectedGame();

        // Replaces the active list rather than merging, so picking a game twice cannot double it up; the codes are read by the running game's codecs where one runs - see §4.14.
        private void LoadSelectedGame()
        {
            if (SelectedGame is not { } game) return;

            if (_cheats is null)
            {
                StatusText.Text = "No cheat list to load into.";
                return;
            }

            CheatFile parsed;
            int skipped;
            try
            {
                using JsonDocument cht = JsonDocument.Parse(Op(CrystalSessionAbi.CheatChtParse, new { text = File.ReadAllText(game.Path) }));
                (parsed, skipped) = CheatFileOf(cht.RootElement);
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or CrystalException or JsonException)
            {
                ErrorLog.Error("cheats", $"Couldn't read {game.Game}", ex);
                StatusText.Text = $"Couldn't read {game.Game}: {ex.Message}";
                return;
            }

            (int loaded, int refused) = _cheats.Replace(parsed);
            skipped += refused;
            _changed?.Invoke();

            string skippedText = skipped > 0 ? $", {skipped} skipped" : "";
            StatusText.Text = loaded > 0
                ? $"Loaded {loaded} cheat(s) for {game.Game}{skippedText} - all disabled, turn them on under Active Cheats..."
                : $"No usable cheats in {game.Game}{skippedText}";
        }

        // CHT_PARSE's record as the cheat file a list imports.
        private static (CheatFile File, int Skipped) CheatFileOf(JsonElement parsed)
        {
            var file = new CheatFile();
            foreach (JsonElement c in parsed.GetProperty("cheats").EnumerateArray())
            {
                bool rom = c.GetProperty("kind").GetString() == "rom";
                file.Cheats.Add(new CheatFileEntry
                {
                    Kind = rom ? "RomPatch" : "RamPoke",
                    Description = c.GetProperty("description").GetString() ?? "",
                    Enabled = c.GetProperty("enabled").GetBoolean(),
                    Writes = c.GetProperty("writes").EnumerateArray().Select(w => new CheatFileWrite
                    {
                        Space = w.GetProperty("space").GetString() ?? "",
                        Address = CheatFileWrite.Hex((uint)w.GetProperty("address").GetInt64(), rom ? 6 : 1),
                        Value = CheatFileWrite.Hex(w.GetProperty("value").GetUInt32(), Math.Max(1, w.GetProperty("width").GetInt32()) * 2),
                        Width = Math.Max(1, w.GetProperty("width").GetInt32()),
                        Type = w.GetProperty("type").GetString() ?? "Set",
                        BitPosition = w.GetProperty("bit").ValueKind == JsonValueKind.Number ? w.GetProperty("bit").GetInt32() : null,
                        BigEndian = w.GetProperty("big_endian").GetBoolean(),
                        RepeatCount = Math.Max(1, w.GetProperty("repeat").GetInt32()),
                        RepeatAddAddress = CheatFileWrite.Hex((uint)w.GetProperty("repeat_address").GetInt64(), 1),
                        RepeatAddValue = CheatFileWrite.Hex(w.GetProperty("repeat_value").GetUInt32(), 1),
                    }).ToList(),
                });
            }
            return (file, parsed.GetProperty("skipped").GetInt32());
        }

        // Where a just-loaded list is actually turned on, so it is a button
        // here rather than a trip back to the menu bar - see §4.14.
        private void OnActiveCheatsClick(object? sender, RoutedEventArgs e) => _openActiveCheats?.Invoke();

        // The prune's plan or its outcome, as DB_PRUNE answers.
        private JsonDocument? Prune(bool apply)
        {
            try { return JsonDocument.Parse(Op(CrystalSessionAbi.CheatDbPrune, new { apply })); }
            catch (CrystalException ex)
            {
                StatusText.Text = $"Prune failed: {ex.Message}";
                return null;
            }
        }

        private static (int Systems, long Files, long Bytes, string Keeping) Sum(JsonElement plan)
        {
            JsonElement[] removing = plan.GetProperty("removing").EnumerateArray().ToArray();
            return (removing.Length, removing.Sum(r => r.GetProperty("files").GetInt64()), removing.Sum(r => r.GetProperty("bytes").GetInt64()),
                string.Join(", ", plan.GetProperty("keeping").EnumerateArray().Select(k => k.GetString())));
        }

        // Deleting is irreversible and the only way back is a 250MB
        // download, so the first click only ever reports - see §4.16.
        private void OnPruneClick(object? sender, RoutedEventArgs e)
        {
            if (_armedPrune)
            {
                using JsonDocument? done = Prune(apply: true);
                DisarmPrune();
                Refresh();
                if (done is null) return;
                var (systems, files, bytes, _) = Sum(done.RootElement);
                string[] failed = done.RootElement.GetProperty("failed").EnumerateArray().Select(f => f.GetString() ?? "").ToArray();
                string trouble = failed.Length > 0 ? $"  Couldn't remove: {string.Join("; ", failed)}" : "";
                StatusText.Text = $"Deleted {done.RootElement.GetProperty("pruned").GetInt32()} system(s), {files:N0} file(s), {Human(bytes)}.{trouble}";
                return;
            }

            using JsonDocument? plan = Prune(apply: false);
            if (plan is null) return;
            if (!plan.RootElement.GetProperty("can_apply").GetBoolean())
            {
                StatusText.Text = $"Nothing to prune - {plan.RootElement.GetProperty("reason").GetString()}";
                return;
            }

            var (count, fileCount, size, keeping) = Sum(plan.RootElement);
            _armedPrune = true;
            PruneButton.Content = $"Delete {count} system(s)?";
            StatusText.Text = $"{count} system(s) have no core in this build - {fileCount:N0} file(s), " +
                              $"{Human(size)}. Keeping {keeping}. " +
                              "Click again to delete them for good, or Close to leave them alone.";
        }

        private void DisarmPrune()
        {
            _armedPrune = false;
            PruneButton.Content = "Prune Unsupported";
        }

        // Offered where the download lands, since that is the moment the
        // 250MB actually arrives - see §4.16.
        private void OfferPrune()
        {
            using JsonDocument? plan = Prune(apply: false);
            if (plan is null || !plan.RootElement.GetProperty("can_apply").GetBoolean()) return;

            var (count, files, bytes, _) = Sum(plan.RootElement);
            int keeping = plan.RootElement.GetProperty("keeping").GetArrayLength();
            _armedPrune = true;
            PruneButton.Content = $"Delete {count} system(s)?";
            StatusText.Text = $"{StatusText.Text}  -  {count} of {count + keeping} " +
                              $"system(s) have no core in this build ({files:N0} file(s), {Human(bytes)}). " +
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
                // Off the UI thread - this is a real network fetch of a multi-megabyte archive, made by the runtime.
                string answer = await Task.Run(() => Op(CrystalSessionAbi.CheatDbUpdate));
                using JsonDocument installed = JsonDocument.Parse(answer);

                Refresh();
                StatusText.Text = $"Installed {installed.RootElement.GetProperty("installed").GetInt64():N0} cheat file(s) into {target}";
                OfferPrune();
            }
            catch (Exception ex) when (ex is CrystalException or JsonException)
            {
                ErrorLog.Error("cheats", "The cheat database download failed", ex);
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
