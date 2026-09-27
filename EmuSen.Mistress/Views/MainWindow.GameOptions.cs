using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Views
{
    // A game's options and metadata editor, in the themed gamelist and the sidebar library, and the edits as every view shows them - see EmuSen_Settings_Reference.md §4.59 and §4.63.
    public partial class MainWindow : IGameEditorHost
    {
        private static readonly IReadOnlyDictionary<string, IReadOnlyDictionary<string, string>> NoEdits = new Dictionary<string, IReadOnlyDictionary<string, string>>();

        private IReadOnlyDictionary<string, IReadOnlyDictionary<string, string>> _editSnapshot = NoEdits;
        private GameOptionsWindow? _gameOptions;
        private MetadataEditorWindow? _metadataEditor;

        public GameOptionsWindow? GameOptionsShown => _gameOptions;

        public MetadataEditorWindow? MetadataEditorShown => _metadataEditor;

        // Hook for ES-DE's Jump to, Sort games by and Filter gamelist entries, which open the menu; the collections work fills it - see §4.59.
        partial void AddGamelistOptions(SceneGame game, List<GameOption> options);

        // Hook for ES-DE's add or remove this game from the custom collection being edited; the collections work fills it - see §4.59.
        partial void AddCollectionOptions(SceneGame game, List<GameOption> options);

        private GameMetadata MetadataFor(string path) =>
            GameMetadata.Resolve(path, _scrapedText.GetValueOrDefault(path), _editSnapshot.GetValueOrDefault(path));

        // The player's name for a game where they gave one, else the file's, in every list Mistress draws.
        private string DisplayTitle(RomEntry entry) =>
            _editSnapshot.TryGetValue(entry.FullPath, out IReadOnlyDictionary<string, string>? edits) && edits.TryGetValue(GameMetadata.Name, out string? name) && name.Length > 0
                ? name : entry.Title;

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
                NotCounted = m.IsNotCounted, Hidden = m.IsHidden,
            };
        }

        // ES-DE's gamelist options menu, as a sheet over the view; its entries are those Mistress has the features for.
        private void ShowGameOptions(SceneGame game)
        {
            if (_gameOptions is not null || _metadataEditor is not null || _folderEditor is not null) return;
            var options = new List<GameOption>();
            AddGamelistOptions(game, options);
            AddCollectionOptions(game, options);
            // A grouped collection's entry is no game: it has the list's rows and nothing of a game's (§4.58).
            if (game is { Folder: true, IsCollection: false }) AddFolderEntries(game, options);
            else if (!game.IsCollection) AddGameEntries(options, game.File, game.Name, () => ToggleThemedFavourite(game));
            PresentGameOptions(game.Name, options);
        }

        // The sidebar library's menu, on the desktop or its big screen: the game's entries only; the themed gamelist's rows are left out - see §4.63.
        private void ShowLibraryGameOptions(RomEntry entry)
        {
            if (_gameOptions is not null || _metadataEditor is not null) return;
            var options = new List<GameOption>();
            AddGameEntries(options, entry.FullPath, DisplayTitle(entry), () => ToggleFavourite(entry));
            PresentGameOptions(DisplayTitle(entry), options);
        }

        // The editor for the sidebar library's game, from its context menu, Ctrl+I or the options menu.
        private void EditLibraryGameMetadata(RomEntry entry) => ShowMetadataEditor(entry.FullPath, DisplayTitle(entry));

        // Finder's and OpenEmu's Get Info gesture; bound here rather than by the menu, which only draws it.
        internal static readonly Avalonia.Input.KeyGesture EditMetadataGesture = new(Avalonia.Input.Key.I, Avalonia.Input.KeyModifiers.Control);

        // Ctrl+I over either library opens the selected game's editor; true when it did, so the key goes no further.
        private bool EditMetadataFromTheKeyboard(Avalonia.Input.KeyEventArgs e)
        {
            if (!EditMetadataGesture.Matches(e) || !LibraryView.IsVisible || Sheets.IsPresenting || _gameOptions is not null || _coverPicker is not null) return false;
            if (ThemedLibraryShown)
            {
                if (_themed?.SelectedGame is not { IsCollection: false } game) return false;
                if (game.Folder) ShowFolderEditor(game);
                else ShowMetadataEditor(game.File, game.Name);
                return true;
            }
            if (SelectedLibraryEntry is not RomEntry entry) return false;
            EditLibraryGameMetadata(entry);
            return true;
        }

        private void AddGameEntries(List<GameOption> options, string path, string title, Action toggleFavourite)
        {
            options.Add(new GameOption(_records.IsFavourite(path) ? "Remove from Favourites" : "Add to Favourites", toggleFavourite));
            options.Add(new GameOption("Edit This Game's Metadata", () => ShowMetadataEditor(path, title)));
            AddCoverEntries(options, path, title);
            // Scraping only ever starts where the player asks for it - see EmuSen_BigPicture.md §17.14.
            if (!ScrapeRunning) options.Add(new GameOption("Scrape This Game...", () => _ = ConfirmAndScrapeAsync(ScrapeScope.ThisGame(path))));
        }

        // A sheet in a big-screen session, a LunaP window owned by Mistress's on the desktop: SheetLayer decides which.
        private void PresentGameOptions(string title, IReadOnlyList<GameOption> options)
        {
            var window = _gameOptions = new GameOptionsWindow(title, options);
            window.Closed += (_, _) => { if (ReferenceEquals(_gameOptions, window)) _gameOptions = null; };
            _ = SheetLayer.Show(window, this);
        }

        private void ToggleThemedFavourite(SceneGame game)
        {
            _themed?.Sound("favorite");
            _records.ToggleFavourite(game.File);
            IdentifyLater(game.File);
            StatusText.Text = _records.IsFavourite(game.File) ? $"Added {game.Name} to Favourites" : $"Removed {game.Name} from Favourites";
            ShowLibraryEntries();
        }

        private void ShowMetadataEditor(string path, string title)
        {
            if (_metadataEditor is not null)
            {
                SheetLayer.Activate(_metadataEditor);
                return;
            }
            var window = _metadataEditor = new MetadataEditorWindow(this, path, title);
            window.Closed += (_, _) => { if (ReferenceEquals(_metadataEditor, window)) _metadataEditor = null; };
            _ = SheetLayer.Show(window, this);
        }

        // A sheet is not an owned window, so nothing else closes these with Mistress - see EmuSen_BigPicture.md §15.14 and §20.4.
        private void CloseGameSheets()
        {
            _coverPicker?.Close();
            _coverPicker = null;
            _metadataEditor?.Close();
            _metadataEditor = null;
            _folderEditor?.Close();
            _folderEditor = null;
            _gameOptions?.Close();
            _gameOptions = null;
        }

        MetadataDraft IGameEditorHost.DraftFor(string path) =>
            new(path, ScrapedNow(path) ?? _scrapedText.GetValueOrDefault(path), _records.Edits(path), _records.Find(path));

        public ScrapedRecord? ScrapedNow(string path) => _mediaStore is { IsOpen: true } store ? store.FoundFor(path) : _scrapedText.GetValueOrDefault(path);

        Task<bool> IGameEditorHost.ScrapeGameAsync(string path) => ConfirmAndScrapeAsync(ScrapeScope.ThisGame(path));

        // Edits go to games.db beside the favourite and play records; media.db, which a scrape writes, is never where an edit lives.
        public void SaveMetadata(MetadataDraft draft)
        {
            IReadOnlyDictionary<string, string?> changes = draft.Changes();
            if (changes.Count > 0) _records.SaveEdits(draft.Path, changes, DateTime.Now);
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
            string? system = EmuSen.Cores.CoreCatalog.ShelfByName(EmuSen.Cores.CoreCatalog.ShelfFor(path) ?? "")?.EsdeSystem;
            IReadOnlyList<string> gone = _mediaStore is { IsOpen: true } store ? store.Forget(path, system, GameFolders.Of(_appSettings.RomDirectory, path)) : [];
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
