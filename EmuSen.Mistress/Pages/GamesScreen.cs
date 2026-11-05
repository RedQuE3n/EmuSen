using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;

namespace EmuSen.Mistress.Pages
{
    // A game as the grid sorts and filters it: its facts, its shelf, and what the sorts read.
    public sealed record GridGame(GameFacts Facts, string Shelf, int PlayCount, double? Rating);

    // A collection the grid can show: its id, its name and the files in it.
    public sealed record GridCollection(long Id, string Name, IReadOnlySet<string> Members);

    // One row of the List card: its key, its words, its value, and whether A opens a page, steps a value or acts.
    public sealed record ListRow(string Key, string Label, string? Value = null, bool Opens = false, bool Steps = false);

    // What the grid reads from the window and asks it to do; the tests give a fake.
    public interface IGamesHost
    {
        DateTime Now { get; }
        IReadOnlyList<GridGame> Games { get; }
        IReadOnlyList<(string Key, string Label)> Consoles { get; }
        // Every shelf of the systems table in its order, with or without games.
        IReadOnlyList<ConsoleShelf> Table { get; }
        IReadOnlyList<GridCollection> Collections { get; }
        IReadOnlyList<string> Recent(IReadOnlyList<string> paths, int limit);
        bool CanStartConsole { get; }
        int Random(int count);
        void Play(string path);
        void OpenPage(string path);
        void ToggleFavourite(string path);
        void Home();
        // B past everything on the grid: Choose a console, its focus on the section left.
        void ChooseConsole(string section);
    }

    // The grid of covers in EmuSen's own look as one plain description: its sections, sort, filter and search, the focused game's facts, the List card's rows and the help bar - see EmuSen_BigPicture.md §51.
    public sealed class GamesScreen(IGamesHost host)
    {
        public const string All = "all", Favourites = "favourites", Recent = "recent", ConsolePrefix = "console:", CollectionPrefix = "collection:";
        public const string SearchRow = "search", SortRow = "sort", FilterRow = "filter", JumpRow = "jump", RandomRow = "random", StartConsoleRow = "start-console", CollectionsRow = "collections";
        public const string EveryGame = "Every game", FavouritesFilter = "Favourites", ListTitle = "List", ConsolesTitle = "Consoles";

        public static readonly IReadOnlyList<string> Sorts = ["Name", "Last played", "Most played", "Year", "Rating"];

        // Recently played holds thirty, as the desktop's does.
        public const int RecentLimit = 30;

        private readonly IGamesHost _host = host;
        private string _section = All;
        private long? _collection;
        private string _search = string.Empty;
        private readonly HashSet<string> _favouritesIn = new(StringComparer.Ordinal);

        public IGamesHost Host => _host;

        public string Section => _section;

        public string Sort { get; private set; } = Sorts[0];

        public string SearchText => _search;

        // Whether the Filter page's Favourites narrows the section shown; each section keeps its own.
        public bool FavouritesOnly => _section != Favourites && _favouritesIn.Contains(_section);

        // The Filter row's value: Favourites, the collection shown, or nothing for every game.
        public string? FilterValue => FavouritesOnly ? FavouritesFilter : Collection?.Name;

        // The collection shown in the strip's place, or null.
        public GridCollection? Collection => _collection is long id ? _host.Collections.FirstOrDefault(c => c.Id == id) : null;

        // All, Favourites and Recent, then each console that has a game; a collection alone in the strip's place while it is shown.
        public IReadOnlyList<(string Key, string Label)> Sections =>
            Collection is { } c ? [(CollectionPrefix + c.Id, c.Name)]
            : [(All, "All"), (Favourites, "Favourites"), (Recent, "Recent"), .. _host.Consoles.Select(k => (ConsolePrefix + k.Key, k.Label))];

        public void ChooseSection(string key)
        {
            if (Sections.Any(s => s.Key == key)) _section = key;
        }

        public void ShowCollection(long? id)
        {
            if (_collection is long left) _favouritesIn.Remove(CollectionPrefix + left);
            _collection = id;
            _section = id is long i ? CollectionPrefix + i : All;
        }

        public void SetSearch(string text) => _search = text ?? string.Empty;

        public void StepSort(int by)
        {
            int at = Sorts.ToList().IndexOf(Sort);
            Sort = Sorts[((at + by) % Sorts.Count + Sorts.Count) % Sorts.Count];
        }

        // The section's games, narrowed by the search and sorted.
        public IReadOnlyList<GridGame> Shown
        {
            get
            {
                IReadOnlyList<GridGame> games = InSection();
                if (FavouritesOnly) games = games.Where(g => g.Facts.Favourite).ToList();
                if (_search.Trim() is { Length: > 0 } words) games = games.Where(g => g.Facts.Title.Contains(words, StringComparison.CurrentCultureIgnoreCase)).ToList();
                return Sorted(games);
            }
        }

        private IReadOnlyList<GridGame> InSection()
        {
            IReadOnlyList<GridGame> all = _host.Games;
            if (_section == Recent)
            {
                var byPath = all.ToDictionary(g => g.Facts.Path, StringComparer.Ordinal);
                return _host.Recent(all.Select(g => g.Facts.Path).ToList(), RecentLimit).Where(byPath.ContainsKey).Select(p => byPath[p]).ToList();
            }
            if (_section == Favourites) return all.Where(g => g.Facts.Favourite).ToList();
            if (_section.StartsWith(ConsolePrefix, StringComparison.Ordinal)) return all.Where(g => g.Shelf == _section[ConsolePrefix.Length..]).ToList();
            if (Collection is { } c) return all.Where(g => c.Members.Contains(g.Facts.Path)).ToList();
            return all;
        }

        // Recent keeps its own order under the name sort; every other sort breaks a tie by name.
        private IReadOnlyList<GridGame> Sorted(IReadOnlyList<GridGame> games)
        {
            if (_section == Recent && Sort == Sorts[0]) return games;
            IOrderedEnumerable<GridGame> by = Sort switch
            {
                "Last played" => games.OrderByDescending(g => g.Facts.LastPlayed ?? DateTime.MinValue),
                "Most played" => games.OrderByDescending(g => g.PlayCount),
                "Year" => games.OrderBy(g => g.Facts.Year ?? "9999", StringComparer.Ordinal),
                "Rating" => games.OrderByDescending(g => g.Rating ?? -1),
                _ => games.OrderBy(g => g.Facts.Title, StringComparer.CurrentCultureIgnoreCase),
            };
            return by.ThenBy(g => g.Facts.Title, StringComparer.CurrentCultureIgnoreCase).ToList();
        }

        // The count at the strip's right, and how many of them a search leaves.
        public string CountLine
        {
            get
            {
                int all = InSection().Count, shown = Shown.Count;
                string games = all == 1 ? "1 game" : $"{all.ToString("N0", CultureInfo.CurrentCulture)} games";
                return _search.Trim().Length > 0 || FavouritesOnly ? $"{shown.ToString("N0", CultureInfo.CurrentCulture)} of {games}" : games;
            }
        }

        // The letters Jump to steps through: each first letter of the shown games in order.
        public IReadOnlyList<string> Letters => Shown.Select(g => Letter(g.Facts.Title)).Distinct().ToList();

        private static string Letter(string title) => title.Length > 0 && char.IsLetter(title[0]) ? char.ToUpperInvariant(title[0]).ToString() : "#";

        // The first game under the letter a step of Jump to reaches from the focused game's letter.
        public string? JumpFrom(string? focused, int by)
        {
            IReadOnlyList<GridGame> shown = Shown;
            if (shown.Count == 0) return null;
            IReadOnlyList<string> letters = Letters;
            string here = Letter(shown.FirstOrDefault(g => g.Facts.Path == focused)?.Facts.Title ?? shown[0].Facts.Title);
            int at = letters.ToList().IndexOf(here);
            string to = letters[((at + by) % letters.Count + letters.Count) % letters.Count];
            return shown.First(g => Letter(g.Facts.Title) == to).Facts.Path;
        }

        public string? RandomGame()
        {
            IReadOnlyList<GridGame> shown = Shown;
            return shown.Count == 0 ? null : shown[Math.Clamp(_host.Random(shown.Count), 0, shown.Count - 1)].Facts.Path;
        }

        // The List card's rows (EmuSen_Interface.md §18.1), Start Console where a console starts empty and Collections after them.
        public IReadOnlyList<ListRow> ListRows(string? focused)
        {
            var rows = new List<ListRow>
            {
                new(SearchRow, "Search", Opens: true),
                new(SortRow, "Sort", Sort, Steps: true),
                new(FilterRow, "Filter", FilterValue, Opens: true),
                new(JumpRow, "Jump to", Letter(Shown.FirstOrDefault(g => g.Facts.Path == focused)?.Facts.Title ?? Shown.FirstOrDefault()?.Facts.Title ?? "#"), Steps: true),
                new(RandomRow, "Random game"),
            };
            if (_host.CanStartConsole) rows.Add(new(StartConsoleRow, "Start Console", Opens: true));
            rows.Add(new(CollectionsRow, "Collections", Opens: true));
            return rows;
        }

        // The card's pane for a row: what the row does, and what the list shows now.
        public IReadOnlyList<(string Line, string Muted)> ListPane(string row) => row switch
        {
            SortRow => [(SortSentence(), "Also: " + string.Join(" · ", Sorts.Where(s => s != Sort)))],
            SearchRow => [("Type part of a name; the covers narrow as you type.", "The keyboard docks at the foot of the screen.")],
            FilterRow => [(FavouritesOnly ? "Shows the section's favourites." : Collection is { } c ? $"Shows {c.Name}." : "Shows every game in the section.", "Choose Favourites to narrow it, or a collection to show it in the strip's place.")],
            JumpRow => [("Moves to the first game under the letter.", string.Join(" ", Letters))],
            RandomRow => [("Moves to a game chosen at random.", "A plays it.")],
            StartConsoleRow => [("Starts a console with no disc in it.", "Change Disc on the Pause Card puts a game in.")],
            _ => [("Your collections, and the automatic ones.", "A opens a collection as a section of the library.")],
        };

        private string SortSentence() => Sort switch
        {
            "Last played" => "Sorted by when each was last played, newest first.",
            "Most played" => "Sorted by how often each was played, most first.",
            "Year" => "Sorted by year of release, oldest first.",
            "Rating" => "Sorted by rating, highest first.",
            _ => "Sorted by name, A to Z.",
        };

        // The Filter page's rows: every game, Favourites except in the Favourites section, then each of the player's collections.
        public IReadOnlyList<(string Key, string Label)> FilterRows =>
            [("", EveryGame), .. _section == Favourites ? Array.Empty<(string, string)>() : [(Favourites, FavouritesFilter)],
             .. _host.Collections.Select(c => (c.Id.ToString(CultureInfo.InvariantCulture), c.Name))];

        // A Filter page row chosen: every game clears the section's favourites and leaves a collection, Favourites narrows the section, a collection takes the strip's place.
        public void ChooseFilter(string key)
        {
            if (key == Favourites) { if (_section != Favourites) _favouritesIn.Add(_section); return; }
            _favouritesIn.Remove(_section);
            if (long.TryParse(key, NumberStyles.Integer, CultureInfo.InvariantCulture, out long id)) ShowCollection(id);
            else if (_collection is not null) ShowCollection(null);
        }

        // The focused game's facts under the grid.
        public string FactsOf(GridGame game) => HomeScreen.GameOf(game.Facts, _host.Now).Facts;

        // The help bar: the keyboard's while it has the focus, else the grid's; B named only where it does something.
        public IReadOnlyList<ScreenHint> HintsFor(bool inKeyboard, bool searched) => inKeyboard
            ? [new("South", "Type"), new("East", "Delete"), new("North", "Space"), new("West", "Done"), new("Shoulders", "Move to grid"), new("Start", HomeScreen.Title, "home")]
            : [new("South", "Play"), new("West", "Game page", "page"), new("North", "Favourite", "favourite"), new("Shoulders", Collection is null ? "Console" : "Section"), new("Select", ListTitle, "list"),
               new("East", searched ? "Clear search" : ConsolesTitle, "back"), new("Start", HomeScreen.Title, "home")];

        public void Hint(string key, string? focused)
        {
            switch (key)
            {
                case "page" when focused is not null: _host.OpenPage(focused); break;
                case "favourite" when focused is not null: _host.ToggleFavourite(focused); break;
                case "home": _host.Home(); break;
            }
        }

        // B: a search is cleared first, then the section's favourites, then a collection left, then Choose a console.
        public bool Back()
        {
            if (_search.Length > 0) { _search = string.Empty; return true; }
            if (FavouritesOnly) { _favouritesIn.Remove(_section); return true; }
            if (_collection is not null) { ShowCollection(null); return true; }
            _host.ChooseConsole(_section);
            return true;
        }
    }
}
