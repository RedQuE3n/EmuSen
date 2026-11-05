using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Views
{
    // The themed gamelist's options sheet, the game's metadata and the edits as every view shows them; the game's own options are its page's - see EmuSen_Settings_Reference.md §4.59 and EmuSen_BigPicture.md §50.
    public partial class MainWindow
    {
        private static readonly IReadOnlyDictionary<string, IReadOnlyDictionary<string, string>> NoEdits = new Dictionary<string, IReadOnlyDictionary<string, string>>();

        private IReadOnlyDictionary<string, IReadOnlyDictionary<string, string>> _editSnapshot = NoEdits;
        private GamelistOptionsWindow? _gameOptions;

        public GamelistOptionsWindow? GameOptionsShown => _gameOptions;

        // Hook for ES-DE's Jump to, Sort games by and Filter gamelist entries, which open the menu; the collections work fills it - see §4.59.
        partial void AddGamelistOptions(SceneGame game, List<GameOption> options);

        // Hook for ES-DE's add or remove this game from the custom collection being edited; the collections work fills it - see §4.59.
        partial void AddCollectionOptions(SceneGame game, List<GameOption> options);

        private GameMetadata MetadataFor(string path) =>
            GameMetadata.Resolve(path, _scrapedText.GetValueOrDefault(path), _editSnapshot.GetValueOrDefault(path), _appSettings.ScrapeGameNames);

        // ES-DE's per-game alternative emulator, the game's own file's "Engine" since SR10, which the runtime keeps; one its console does not run is not stored (§4.66, EmuSen_Backend.md §14.12).
        private static void SetGameEngine(string path, string? chosen)
        {
            string? engine = GameMetadata.EngineFor(path, chosen);
            try { CoreSettings.Runtime.SetGameSetting(path, EmuSen.Mistress.Library.RuntimeCatalog.EngineKey, engine); }
            catch (EmuSen.Crystal.Client.CrystalException e) { EmuSen.Galaxia.Library.ErrorLog.Warning("settings", $"{System.IO.Path.GetFileName(path)}'s engine could not be stored: {e.Message}"); }
        }

        // The player's name for a game where they gave one, else ScreenScraper's with "Game names" on (Q30), else the file's, in every list Mistress draws.
        private string DisplayTitle(RomEntry entry) =>
            _editSnapshot.TryGetValue(entry.FullPath, out IReadOnlyDictionary<string, string>? edits) && edits.TryGetValue(GameMetadata.Name, out string? name) && name.Length > 0
                ? name
                : _appSettings.ScrapeGameNames && _scrapedText.TryGetValue(entry.FullPath, out ScrapedRecord? scraped) && scraped.Name is { Length: > 0 } theirs ? theirs : entry.Title;

        private bool IsHiddenGame(string path) =>
            _editSnapshot.TryGetValue(path, out IReadOnlyDictionary<string, string>? edits) && edits.GetValueOrDefault(GameMetadata.Hidden) == GameMetadata.Yes;

        // A hidden game is left out of every library view until Preferences lists hidden games again.
        private bool Listed(RomEntry entry) => _appSettings.ShowHiddenGames || !IsHiddenGame(entry.FullPath);

        private IReadOnlyList<RomEntry> Listed(IReadOnlyList<RomEntry> entries) =>
            _appSettings.ShowHiddenGames || _editSnapshot.Count == 0 ? entries : entries.Where(Listed).ToList();

        // A wide run leaves out games marked "Exclude from multi-scraper"; Scrape This Game still asks for them.
        private bool ExcludedFromMultiScrape(string path) =>
            _editSnapshot.TryGetValue(path, out IReadOnlyDictionary<string, string>? edits) && edits.GetValueOrDefault(GameMetadata.NoMultiScrape) == GameMetadata.Yes;

        // The themed view's game: the player's edit over ScreenScraper's text over the file's name.
        private SceneGame WithMetadata(SceneGame game)
        {
            GameMetadata m = MetadataFor(game.File);
            return game with
            {
                Name = m.Title, SortName = m.Sort, Description = m.DescriptionText, Developer = m.DeveloperText, Publisher = m.PublisherText, Genre = m.GenreText,
                Players = m.PlayersText, ReleaseDate = m.Released, Rating = m.RatingValue, Completed = m.IsCompleted, KidGame = m.IsKidGame, Broken = m.IsBroken,
                NotCounted = m.IsNotCounted, Hidden = m.IsHidden, Controller = m.ControllerType, AltEmulator = m.AltEmulatorEngine is not null,
                CustomSortName = m.CustomSort, HideMetadata = m.IsMetadataHidden,
            };
        }

        // ES-DE's gamelist options menu, as a sheet over the view; its entries are those Mistress has the features for.
        private void ShowGameOptions(SceneGame game)
        {
            if (_gameOptions is not null || _folderEditor is not null) return;
            var options = new List<GameOption>();
            AddGamelistOptions(game, options);
            AddStartConsoleOption(options);
            AddCollectionOptions(game, options);
            // A grouped collection's entry is no game: it has the list's rows and nothing of a game's (§4.58).
            if (game is { Folder: true, IsCollection: false }) AddFolderEntries(game, options);
            else if (!game.IsCollection) AddGamePageEntry(options, game.File);
            PresentGameOptions(game.Name, options);
        }

        // Space opens the game's page from the library, as EmuSen_Interface.md §17.3 has it.
        internal static readonly Avalonia.Input.KeyGesture GamePageGesture = new(Avalonia.Input.Key.Space);

        // Finder's and OpenEmu's Get Info gesture; bound here rather than by the menu, which only draws it.
        internal static readonly Avalonia.Input.KeyGesture EditMetadataGesture = new(Avalonia.Input.Key.I, Avalonia.Input.KeyModifiers.Control);

        // Ctrl+I over either library opens the selected game's editor; true when it did, so the key goes no further.
        private bool EditMetadataFromTheKeyboard(Avalonia.Input.KeyEventArgs e)
        {
            if (!EditMetadataGesture.Matches(e) || !LibraryView.IsVisible || Sheets.IsPresenting || _gameOptions is not null || _gamePage is not null) return false;
            if (ThemedLibraryShown)
            {
                if (_themed?.SelectedGame is not { IsCollection: false } game) return false;
                if (game.Folder) ShowFolderEditor(game);
                else ShowGamePage(game.File, GamePage.Details, GamePage.EditPage);
                return true;
            }
            if (SelectedLibraryEntry is not RomEntry entry) return false;
            ShowGamePage(entry.FullPath, GamePage.Details, GamePage.EditPage);
            return true;
        }

        // The game's own options are its page's: the sheet offers the way there (EmuSen_Interface.md §18.0).
        private void AddGamePageEntry(List<GameOption> options, string path) =>
            options.Add(new GameOption(GamePageRow, () => ShowGamePage(path)) { Opens = true });

        public const string GamePageRow = "Game Page";

        // A sheet in a big-screen session, a LunaP window owned by Mistress's on the desktop: SheetLayer decides which.
        private void PresentGameOptions(string title, IReadOnlyList<GameOption> options)
        {
            var window = _gameOptions = new GamelistOptionsWindow(title, options, _bigScreen ? HelpFamily : (EmuSen.LunaP.Controls.PadFamily?)null);
            window.Closed += (_, _) => { if (ReferenceEquals(_gameOptions, window)) _gameOptions = null; };
            _ = SheetLayer.Show(window, this);
        }

        private void ToggleThemedFavourite(SceneGame game)
        {
            _themed?.Sound("favorite");
            _themed?.KeepPlace();
            _records.ToggleFavourite(game.File);
            IdentifyLater(game.File);
            StatusText.Text = _records.IsFavourite(game.File) ? $"Added {game.Name} to Favourites" : $"Removed {game.Name} from Favourites";
            ShowLibraryEntries();
        }

        // A sheet is not an owned window, so nothing else closes these with Mistress - see EmuSen_BigPicture.md §15.14 and §20.4.
        private void CloseGameSheets()
        {
            _folderEditor?.Close();
            _folderEditor = null;
            _gameOptions?.Close();
            _gameOptions = null;
        }

        private MetadataDraft DraftFor(string path) =>
            new(path, ScrapedNow(path) ?? _scrapedText.GetValueOrDefault(path), _records.Edits(path), _records.Find(path), _appSettings.ScrapeGameNames);

        public ScrapedRecord? ScrapedNow(string path) => _native is { } n ? n.FoundFor(path) : _scrapedText.GetValueOrDefault(path);

        // Edits go to games.db beside the favourite and play records; media.db, which a scrape writes, is never where an edit lives.
        public void SaveMetadata(MetadataDraft draft)
        {
            IReadOnlyDictionary<string, string?> changes = draft.Changes();
            if (changes.Count > 0) _records.SaveEdits(draft.Path, changes, DateTime.Now);
            if (changes.TryGetValue(GameMetadata.AltEmulator, out string? engine)) SetGameEngine(draft.Path, engine);
            if (draft.Favourite != draft.StoredFavourite) _records.SetFavourite(draft.Path, draft.Favourite);
            if (draft.PlayCount != draft.StoredPlayCount || Math.Abs(draft.PlaySeconds - draft.StoredPlaySeconds) > 0.5)
                _records.SetPlayStats(draft.Path, draft.PlayCount, draft.PlaySeconds);
            IdentifyLater(draft.Path);
            StatusText.Text = $"Saved the metadata of {Path.GetFileNameWithoutExtension(draft.Path)}";
            ShowLibraryEntries();
        }

        // ES-DE's Clear: the edits, and ScreenScraper's text and pictures in Mistress's own store; the ROM, the player's covers and an ES-DE folder are left.
        public void ClearMetadata(string path)
        {
            _records.ClearEdits(path);
            string? system = EmuSen.Mistress.Library.RuntimeCatalog.ShelfByName(EmuSen.Mistress.Library.RuntimeCatalog.ShelfFor(path) ?? "")?.EsdeSystem;
            IReadOnlyList<string> gone = _native is { } n ? n.Forget(path, system, GameFolders.Of(_appSettings.RomDirectory, path)) : [];
            _scrapeGeneration++;
            ReadScrapeSnapshot();
            StatusText.Text = $"Cleared the metadata of {Path.GetFileNameWithoutExtension(path)}" + (gone.Count > 0 ? $" and {gone.Count} scraped picture(s)" : "");
            ShowLibraryEntries();
        }

        // Where ES-DE deletes the file, Mistress sets the Hidden field; nothing on disk but games.db changes.
        public void HideGame(string path)
        {
            _records.SaveEdits(path, new Dictionary<string, string?> { [GameMetadata.Hidden] = GameMetadata.Yes }, DateTime.Now);
            _gameOptions?.Close();
            StatusText.Text = $"Hid {Path.GetFileNameWithoutExtension(path)} from the library; its file was not touched";
            ShowLibraryEntries();
        }
    }
}
