using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Scene;

namespace EmuSen.Mistress.BigPicture
{
    // A gamelist's sort, filters, folders, quick selector and random entry, and the collection being edited - see EmuSen_BigPicture.md §22.
    public sealed partial class ThemedLibrary
    {
        private readonly Dictionary<string, IReadOnlyList<SceneGame>> _listed = new(StringComparer.Ordinal);
        private readonly Dictionary<string, GameSort> _sorts = new(StringComparer.Ordinal);
        private readonly Dictionary<string, GameFilter> _gameFilters = new(StringComparer.Ordinal);
        private readonly Dictionary<string, string> _openFolder = new(StringComparer.Ordinal);
        private EditedCollection? _editing;

        // ES-DE's "Game default sort order"; a gamelist's own choice outlasts it for the session.
        public GameSort DefaultSort { get; set; } = GameSort.Default;

        // ES-DE's "Random entry button": games, gamessystems or disabled.
        public string RandomEntryButton { get; set; } = BigPictureCollections.RandomGames;

        public Random Random { get; set; } = new();

        // The custom collection being edited: North adds and removes the game, and its members carry the tick and the collection badge.
        public EditedCollection? Editing
        {
            get => _editing;
            set
            {
                _editing = value;
                ForgetListed();
            }
        }

        private HelpContext HelpContext => new(Editing is not null, RandomEntryButton != BigPictureCollections.RandomDisabled,
            RandomEntryButton == BigPictureCollections.RandomGamesAndSystems, QuickSelect: QuickSelectFor("gamelist"), Screensaver: Interface.ScreensaverControls);

        private ThemedShelf? ShelfOf(string? system) => _shelves.FirstOrDefault(s => s.System.Name == system);

        // The folder open in a grouped system, or null at its top.
        private ThemedShelf? OpenFolder(string? system) =>
            system is not null && _openFolder.TryGetValue(system, out string? name) ? ShelfOf(system)?.Folders?.FirstOrDefault(f => f.System.Name == name) : null;

        // Each system, and each folder inside the grouped one, keeps its own sort, filters and selection.
        private string ListKey(string? system) => OpenFolder(system) is { } folder ? $"{system}/{folder.System.Name}" : system ?? "";

        public string GamelistKey => ListKey(_system);

        public bool InFolder => ViewName == "gamelist" && (OpenFolder(_system) is not null || FolderIn(_system).Length > 0);

        // The games the gamelist shows before its filters: a grouped system's folders at its top, a folder's games inside it.
        public IReadOnlyList<SceneGame> Unfiltered => Source(_system).Folders ? Listed(_system ?? "") : Source(_system).Games;

        private (IReadOnlyList<SceneGame> Games, bool FavoritesFirst, GameSort? Sort, bool Folders) Source(string? system)
        {
            ThemedShelf? shelf = ShelfOf(system);
            if (shelf is null) return ([], true, null, false);
            if (OpenFolder(system) is { } folder) return (folder.Games, folder.FavoritesFirst, folder.DefaultSort, false);
            if (shelf.Folders is not null) return ([], false, null, true);
            return (shelf.Games, shelf.FavoritesFirst, shelf.DefaultSort, false);
        }

        private bool FavoritesFirstIn(string system) => !Source(system).Folders && Source(system).FavoritesFirst;

        private bool StarsIn(string system) => OpenFolder(system)?.Stars ?? ShelfOf(system)?.Stars ?? true;

        // THEMES.md: a blank system name at the grouped collections' top, which a theme's defaultValue fills; inside, the collection's own.
        private string? HeadingIn(string system) => ShelfOf(system)?.Folders is null ? null : OpenFolder(system)?.System.Name ?? "";

        public GameSort CurrentSort => SortOf(_system);

        private GameSort SortOf(string? system) => _sorts.TryGetValue(ListKey(system), out GameSort chosen) ? chosen : Source(system).Sort ?? DefaultSort;

        public GameFilter CurrentFilter => _gameFilters.GetValueOrDefault(GamelistKey, GameFilter.None);

        // Whether the current gamelist can be sorted by system: an automatic or custom collection.
        public bool IsCollection => SelectedSystem?.System.Kind is Theme.ThemeSystemKind.AutoCollection or Theme.ThemeSystemKind.CustomCollection;

        // The custom collection the gamelist shows, when it shows one.
        public long? CurrentCollectionId => OpenFolder(_system)?.CollectionId ?? ShelfOf(_system)?.CollectionId;

        public bool FavoritesOnTop => !Source(_system).Folders && Source(_system).FavoritesFirst;

        // A system's games as the gamelist lists them: searched, filtered, sorted, marked for the collection being edited; kept until something changes.
        private IReadOnlyList<SceneGame> Listed(string system)
        {
            if (Foldered(system)) WalkUp(system);
            string key = ViewKey(system);
            if (_listed.TryGetValue(key, out IReadOnlyList<SceneGame>? kept)) return kept;
            (IReadOnlyList<SceneGame> games, bool favoritesFirst, _, bool folders) = Source(system);
            if (Foldered(system)) return _listed[key] = Marked(FolderListing(system, favoritesFirst));
            if (folders)
                games = ShelfOf(system)!.Folders!.OrderBy(f => f.System.Name, StringComparer.OrdinalIgnoreCase).Select(f => CollectionShelves.Folder(f, Random)).ToList();
            IEnumerable<SceneGame> shown = _filter.Length == 0 ? games : games.Where(g => g.Folder || FilterBar.Matches(_filter, g.Name));
            shown = GamelistOptions.Apply(shown, _gameFilters.GetValueOrDefault(key, GameFilter.None));
            return _listed[key] = Marked(folders ? shown.ToList() : GamelistOptions.Sort(shown, SortOf(system), favoritesFirst));
        }

        // The collection being edited ticks its members; a folder is never one.
        private IReadOnlyList<SceneGame> Marked(IReadOnlyList<SceneGame> sorted) =>
            _editing is { } editing ? sorted.Select(g => g.Folder ? g : g with { InCollection = editing.Members.Contains(g.File) }).ToList() : sorted;

        // The view rebuilt from the kept selection after something it lists changed.
        public void Refresh(TimeSpan now)
        {
            ForgetListed();
            if (Stage is null) return;
            Stage.Replace(Data(), now);
            Remember();
        }

        // ES-DE's random entry button: another game of the gamelist, or with "games and systems" another system in the system view.
        private void RandomEntry(TimeSpan now)
        {
            if (Stage is null || RandomEntryButton == BigPictureCollections.RandomDisabled) return;
            SceneView view = Stage.Current;
            if (view.ViewName == "system" && RandomEntryButton != BigPictureCollections.RandomGamesAndSystems) return;
            int target = GamelistOptions.RandomIndex(view.Count, view.Index, Random);
            if (target != view.Index) view.Jump(target - view.Index, now);
        }

        // The screensaver's jump to a game: its system's gamelist, in the folder it sits in, with it selected; false when the view has no such game.
        public bool ShowGame(string system, string file, TimeSpan now)
        {
            if (Stage is null || ShelfOf(system)?.Games.FirstOrDefault(g => g.File == file) is not { } game) return false;
            ForgetListed();
            _system = system;
            _openFolder.Remove(system);
            if (Foldered(system) && game.FolderPath.Length > 0) _path[system] = game.FolderPath;
            else _path.Remove(system);
            _cursor[ViewKey(system)] = file;
            if (Stage.Current.ViewName == "gamelist") Stage.Replace(Data(), now);
            else Stage.Switch(now, Data());
            Remember();
            return true;
        }

        // The quick selector's letters for the gamelist as it is listed now.
        public IReadOnlyList<(string Label, int Index)> Letters() =>
            Stage is { Current.ViewName: "gamelist" } s ? GamelistOptions.Letters(s.Current.Data.System.Games, FavoritesOnTop, FoldersOnTop) : [];

        // The gamelist options sheet's choices, applied as ES-DE applies them on closing it: the sort and filters, then the jump.
        public void ApplyOptions(GameSort sort, GameFilter filter, string? letter, TimeSpan now)
        {
            if (Stage is null) return;
            string key = GamelistKey;
            _sorts[key] = sort;
            if (filter.IsActive) _gameFilters[key] = filter;
            else _gameFilters.Remove(key);
            Refresh(now);
            if (letter is not null && Letters().FirstOrDefault(l => l.Label == letter) is { Label: not null } found && Stage.Current.ViewName == "gamelist")
            {
                SceneView view = Stage.Current;
                if (found.Index != view.Index) view.Jump(found.Index - view.Index, now);
                Remember();
            }
        }
    }
}
