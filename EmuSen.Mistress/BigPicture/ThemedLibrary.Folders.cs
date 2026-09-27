using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.BigPicture
{
    // A console's folders as ES-DE shows them: entries South enters and East leaves, on top, each system keeping the folder it is in - see EmuSen_BigPicture.md §30.
    public sealed partial class ThemedLibrary
    {
        private readonly Dictionary<string, string> _path = new(StringComparer.Ordinal);
        private readonly HashSet<string> _foldered = new(StringComparer.Ordinal);

        // ES-DE's "Sort folders on top of gamelists", on by default.
        public bool FoldersOnTop { get; set; } = true;

        // A regular system shows folders unless the player flattened it, once a game of it sits in one.
        private void FindFoldered(IEnumerable<ThemedShelf> shelves)
        {
            _foldered.Clear();
            foreach (ThemedShelf s in shelves)
                if (s is { Flatten: false, Folders: null, System.Kind: ThemeSystemKind.Regular } && s.Games.Any(g => g.FolderPath.Length > 0)) _foldered.Add(s.System.Name);
        }

        private bool Foldered(string? system) => system is not null && _foldered.Contains(system);

        // The folder a system's gamelist is in, "" at its top.
        private string FolderIn(string? system) => Foldered(system) ? _path.GetValueOrDefault(system!, "") : "";

        // The folder the current gamelist shows, '/'-separated; "" at the top or when the system shows no folders.
        public string CurrentFolder => FolderIn(_system);

        // A list's own place: the sort and filters are the system's, the selection each folder's own.
        private string ViewKey(string? system) => FolderIn(system) is { Length: > 0 } path ? $"{ListKey(system)}\u001f{path}" : ListKey(system);

        // The games a filtered and searched system keeps, over every folder, as ES-DE's filters apply to "the complete game system, including all folder content".
        private IReadOnlyList<SceneGame> Kept(string system)
        {
            if (_kept.TryGetValue(system, out IReadOnlyList<SceneGame>? kept)) return kept;
            IReadOnlyList<SceneGame> games = ShelfOf(system)?.Games ?? [];
            IEnumerable<SceneGame> shown = _filter.Length == 0 ? games : games.Where(g => FilterBar.Matches(_filter, g.Name));
            return _kept[system] = GamelistOptions.Apply(shown, _gameFilters.GetValueOrDefault(ListKey(system), GameFilter.None));
        }

        private readonly Dictionary<string, IReadOnlyList<SceneGame>> _kept = new(StringComparer.Ordinal);

        // The listed games and what they were kept from are forgotten together, whenever something they depend on changes.
        private void ForgetListed()
        {
            _listed.Clear();
            _kept.Clear();
        }

        // A folder a search, a filter or a change on disk has emptied gives way to the nearest one above it that still holds a game.
        private void WalkUp(string system)
        {
            string path = FolderIn(system);
            if (path.Length == 0) return;
            IReadOnlyList<SceneGame> kept = Kept(system);
            while (path.Length > 0 && !kept.Any(g => GameFolders.ChildWithin(g.FolderPath, path) is not null)) path = GameFolders.Parent(path);
            if (path.Length > 0) _path[system] = path;
            else _path.Remove(system);
        }

        // One folder's entries: its folders holding a kept game, then its games, or all in one order when folders are not on top.
        private IReadOnlyList<SceneGame> FolderListing(string system, bool favoritesFirst)
        {
            IReadOnlyList<SceneGame> kept = Kept(system);
            string path = FolderIn(system);
            var here = new List<SceneGame>();
            var folders = new Dictionary<string, SceneGame>(StringComparer.Ordinal);
            IReadOnlyDictionary<string, string>? links = ShelfOf(system)?.FolderLinks;
            foreach (SceneGame g in kept)
            {
                switch (GameFolders.ChildWithin(g.FolderPath, path))
                {
                    case "":
                        here.Add(g);
                        break;
                    case { } child when !folders.ContainsKey(child):
                        string folder = path.Length == 0 ? child : $"{path}/{child}";
                        string disk = GameFolders.OnDisk(g.File, g.FolderPath, folder);
                        folders[child] = new SceneGame(child, disk) { Folder = true, FolderPath = folder, FolderLink = links?.GetValueOrDefault(disk) };
                        break;
                }
            }

            GameSort sort = SortOf(system);
            if (!FoldersOnTop) return GamelistOptions.Sort(here.Concat(folders.Values), sort, favoritesFirst);
            return [.. GamelistOptions.Sort(folders.Values, sort, false), .. GamelistOptions.Sort(here, sort, favoritesFirst)];
        }

        // Every kept game of a foldered system, which its system view counts.
        private IReadOnlyList<SceneGame>? CountedIn(string system) => Foldered(system) ? Kept(system) : null;

        // A folder with a link launches the linked game, unless a collection is being edited (USERGUIDE, "Custom collections").
        private SceneGame? LinkedGame(SceneGame folder) =>
            folder is { Folder: true, IsCollection: false, FolderLink: { } link } && Editing is null ? ShelfOf(_system)?.Games.FirstOrDefault(g => g.File == link) : null;

        // Into a folder of the system's own, or a grouped collection; the selection inside is the one left there.
        public void EnterFolder(SceneGame folder, TimeSpan now)
        {
            if (_system is null) return;
            if (folder.IsCollection)
            {
                if (ShelfOf(_system)?.Folders is null) return;
                _openFolder[_system] = folder.Name;
            }
            else if (Foldered(_system) && folder.Folder) _path[_system] = folder.FolderPath;
            else return;
            Refresh(now);
            Sound("select");
        }

        private void LeaveFolder(TimeSpan now)
        {
            if (_system is null) return;
            // The level above keeps its own selection, the folder left, since each list keeps its own (§22.8, C13).
            if (FolderIn(_system) is { Length: > 0 } path)
            {
                if (GameFolders.Parent(path) is { Length: > 0 } up) _path[_system] = up;
                else _path.Remove(_system);
            }
            else if (OpenFolder(_system) is not null) _openFolder.Remove(_system);
            else return;
            Refresh(now);
            Sound("back");
        }
    }
}
