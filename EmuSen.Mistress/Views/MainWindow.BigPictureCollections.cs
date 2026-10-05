using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // Big picture's collections and a gamelist's options, over Mistress's own games.db records - see EmuSen_Settings_Reference.md §4.58.
    public partial class MainWindow : ICollectionSettingsHost
    {
        // The custom collection being edited, for this session only, as ES-DE's edit mode is.
        private (long Id, string Name)? _editedCollection;
        private CollectionSettingsWindow? _collectionSettings;

        internal CollectionSettingsWindow? CollectionSettingsSheet => _collectionSettings;

        // The library's systems followed by ES-DE's collections, from the same records the sidebar shows.
        private IReadOnlyList<ThemedShelf> WithCollections(IReadOnlyList<ThemedShelf> systems) =>
            CollectionShelves.Build(systems, CustomCollections(), _appSettings.BigPictureCollections, ThemeHasFolderFor);

        private IReadOnlyList<CustomCollection> CustomCollections() =>
            _collections.Select(c => new CustomCollection(c.Id, c.Name, _records.Members(c.Id))).ToList();

        // A theme supports a collection when it has a folder of that name with a theme.xml, as ES-DE's per-system folders are (THEMES.md, "How it works").
        private bool ThemeHasFolderFor(string name) =>
            _appSettings.BigPictureTheme is { Length: > 0 } dir && name.Length > 0 && name.IndexOfAny(Path.GetInvalidFileNameChars()) < 0 && File.Exists(Path.Combine(dir, name, "theme.xml"));

        // The settings the themed view reads before each showing.
        private void ApplyCollectionSettings()
        {
            if (_themed is null) return;
            BigPictureCollections s = _appSettings.BigPictureCollections;
            _themed.DefaultSort = GameSort.Parse(s.DefaultSortOrder);
            _themed.RandomEntryButton = s.RandomEntryButton;
            _themed.FoldersOnTop = s.FoldersOnTop;
            if (_editedCollection is { } e && _collections.All(c => c.Id != e.Id)) _editedCollection = null;
            _themed.Editing = _editedCollection is { } edited ? new EditedCollection(edited.Id, edited.Name, _records.Members(edited.Id)) : null;
        }

        // North in a gamelist while a collection is edited: the game goes in, or comes out.
        private void ToggleEditedMembership(SceneGame game)
        {
            if (_editedCollection is not { } edited || _collections.FirstOrDefault(c => c.Id == edited.Id) is not { } collection) return;
            if (_records.CollectionsOf(game.File).Contains(collection.Id))
            {
                _records.RemoveFromCollection(collection.Id, game.File);
                StatusText.Text = $"Removed {game.Name} from {collection.Name}";
            }
            else
            {
                _records.AddToCollection(collection.Id, game.File);
                IdentifyLater(game.File);
                StatusText.Text = $"Added {game.Name} to {collection.Name}";
            }
            ShowLibraryEntries();
        }

        // ES-DE's Game Collection Settings, as the pad menu's Library ▸ Collections over the themed view; the gamelist options are Select's menu (§4.58, §4.69.8).
        private void AddCollectionMenuEntries(List<Input.PadMenuEntry> entries)
        {
            if (!ThemedLibraryShown || !LibraryView.IsVisible || _themed is null) return;
            entries.Add(new Input.PadMenuEntry(() => "Collections", ShowCollectionSettings) { Opens = true });
        }

        // Jump To, Sort Games By and Filter Gamelist first in Select's menu, then Search, which ES-DE has not (§4.58).
        partial void AddGamelistOptions(SceneGame game, List<GameOption> options)
        {
            if (_themed is not { ViewName: "gamelist" } themed) return;
            IReadOnlyList<(string Label, int Index)> letters = themed.Letters();
            string letter = game.Folder && letters.Any(l => l.Label == GamelistOptions.FolderEntry) ? GamelistOptions.FolderEntry
                : themed.FavoritesOnTop && game.Favorite && letters.Any(l => l.Label == GamelistOptions.Star)
                ? GamelistOptions.Star : GamelistOptions.FirstLetter(GamelistOptions.SortKey(game));
            var model = new GamelistOptionsModel(letters, letter, GameSort.All(themed.IsCollection), themed.CurrentSort, themed.CurrentFilter, themed.Unfiltered);
            options.AddRange(new GamelistOptionRows(model, this).Options(ApplyGamelistOptions));
            options.Add(new GameOption("Search...", SearchThemedGamelist));
        }

        // The collection being edited finished from any list, or the list's own custom collection edited, as ES-DE's menu offers them.
        partial void AddCollectionOptions(SceneGame game, List<GameOption> options)
        {
            if (_themed?.CurrentCollectionId is { } id && _collections.FirstOrDefault(c => c.Id == id) is { } shown && _editedCollection?.Id != id)
                options.Add(new GameOption("Add/Remove Games to This Collection", () => SetEditedCollection((shown.Id, shown.Name))));
            if (_editedCollection is { } editing)
                options.Add(new GameOption($"Finish Editing '{editing.Name}' Collection", () => SetEditedCollection(null)));
        }

        private void SetEditedCollection((long Id, string Name)? collection)
        {
            _editedCollection = collection;
            ApplyCollectionSettings();
            _themed?.Refresh(UiClock());
            ScheduleThemedFrame();
        }

        private void ApplyGamelistOptions(GamelistOptionsResult result)
        {
            if (_themed is null) return;
            _themed.ApplyOptions(result.Sort, result.Filter, result.Letter, UiClock());
            ScheduleThemedFrame();
        }

        // The search box, which North opened before North became the favourite (§4.58).
        private void SearchThemedGamelist()
        {
            if (_themedSearch is null) return;
            ShowThemedSearchBar(open: true);
            PadKeyboard.Open(_themedSearch);
        }

        internal void ShowCollectionSettings()
        {
            if (_collectionSettings is not null) return;
            var window = _collectionSettings = new CollectionSettingsWindow(this, _bigScreen ? HelpFamily : null);
            window.Closed += (_, _) => { if (_collectionSettings == window) _collectionSettings = null; };
            _ = SheetLayer.Show(window, this);
        }

        BigPictureCollections ICollectionSettingsHost.Settings => _appSettings.BigPictureCollections;

        IReadOnlyList<GameCollection> ICollectionSettingsHost.Collections => _collections;

        string? ICollectionSettingsHost.EditingName => _editedCollection?.Name;

        // ES-DE's rules for a new name: its forbidden characters dropped, a taken name numbered; then the edit mode starts on it.
        async Task ICollectionSettingsHost.CreateCollectionAsync(Window owner)
        {
            if (await Dialogs.PromptAsync(owner, "Create New Custom Collection", "Name the new collection", "", "Create") is not string typed) return;
            string name = CollectionShelves.Clean(typed);
            if (name.Length == 0) return;
            name = CollectionShelves.Unique(name, _collections.Select(c => c.Name));
            if (_records.CreateCollection(name, DateTime.Now) is not long id) return;
            _editedCollection = (id, name);
            StatusText.Text = $"Created {name}; add games to it with {PadHints.Glyph(EmuSen.LunaP.Controls.PadGlyphButton.North)}";
            ShowLibraryEntries();
        }

        async Task ICollectionSettingsHost.DeleteCollectionAsync(Window owner, GameCollection collection)
        {
            if (!await Dialogs.ConfirmAsync(owner, "Delete Custom Collection", $"Delete the collection {collection.Name}? The games in it are not touched.", "Delete", "Cancel")) return;
            _records.DeleteCollection(collection.Id);
            if (_editedCollection?.Id == collection.Id) _editedCollection = null;
            _appSettings.BigPictureCollections.HiddenCustomCollections.Remove(collection.Id);
            if (_appSettings.LibraryCollection == KeyOf(collection)) _appSettings.LibraryCollection = AllGamesKey;
            _appSettings.Save();
            StatusText.Text = $"Deleted the collection {collection.Name}";
            ShowLibraryEntries();
        }

        void ICollectionSettingsHost.FinishEditing()
        {
            _editedCollection = null;
            ShowLibraryEntries();
        }

        void ICollectionSettingsHost.CollectionSettingsChanged()
        {
            _appSettings.Save();
            if (LibraryView.IsVisible) ShowThemedLibrary();
        }
    }
}
