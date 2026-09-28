using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using EmuSen.Mistress.BigPicture.Scene;

namespace EmuSen.Mistress.BigPicture
{
    // ES-DE's documented sort keys, in USERGUIDE's order; System applies to collections only - see EmuSen_BigPicture.md §22.
    public enum GameSortKey { Name, Rating, ReleaseDate, Developer, Publisher, Genre, Players, LastPlayed, TimesPlayed, PlayTime, System }

    // One of ES-DE's sort orders: a key, ascending or descending, the name ascending after it (§22).
    public readonly record struct GameSort(GameSortKey Key, bool Descending)
    {
        public static GameSort Default { get; } = new(GameSortKey.Name, false);

        public static IReadOnlyList<GameSort> All(bool collection) =>
            Enum.GetValues<GameSortKey>().Where(k => collection || k != GameSortKey.System)
                .SelectMany(k => new[] { new GameSort(k, false), new GameSort(k, true) }).ToList();

        // As ES-DE's DefaultSortOrder setting spells it, e.g. "name, ascending".
        public string Stored => $"{KeyWord(Key)}, {(Descending ? "descending" : "ascending")}";

        public string Label => char.ToUpperInvariant(Stored[0]) + Stored[1..];

        public static GameSort Parse(string? stored) => All(true).FirstOrDefault(s => s.Stored == stored?.Trim().ToLowerInvariant(), Default);

        private static string KeyWord(GameSortKey key) => key switch
        {
            GameSortKey.ReleaseDate => "release date",
            GameSortKey.LastPlayed => "last played",
            GameSortKey.TimesPlayed => "times played",
            GameSortKey.PlayTime => "play time",
            _ => key.ToString().ToLowerInvariant(),
        };
    }

    // The filters USERGUIDE lists that Mistress has data for; the name is a text, the rest sets of values (§22).
    public enum FilterField { Rating, Developer, Publisher, Genre, Players, Favorite, Completed, KidGame, Broken }

    public sealed record GameFilter
    {
        public static GameFilter None { get; } = new();

        public string Name { get; init; } = "";

        public IReadOnlyDictionary<FilterField, IReadOnlySet<string>> Chosen { get; init; } = new Dictionary<FilterField, IReadOnlySet<string>>();

        public bool IsActive => Name.Trim().Length > 0 || Chosen.Values.Any(v => v.Count > 0);

        public GameFilter With(FilterField field, IReadOnlySet<string> values)
        {
            var chosen = Chosen.ToDictionary(p => p.Key, p => p.Value);
            if (values.Count == 0) chosen.Remove(field);
            else chosen[field] = values;
            return this with { Chosen = chosen };
        }
    }

    // Sorting, filtering, the quick selector and the random entry for one gamelist, with no view behind them - see EmuSen_BigPicture.md §22.
    public static class GamelistOptions
    {
        public const string Unknown = "Unknown", Star = "★", NothingToFilter = "Nothing to filter";

        public static readonly IReadOnlyList<FilterField> Fields = Enum.GetValues<FilterField>();

        public static string Label(FilterField field) => field switch
        {
            FilterField.KidGame => "Kidgame",
            _ => field.ToString(),
        };

        // Favourites first when asked, each part in the order's key, games without the key's value after those with it, then the name ascending, then the order given (a stable sort).
        public static IReadOnlyList<SceneGame> Sort(IEnumerable<SceneGame> games, GameSort order, bool favoritesFirst)
        {
            var list = games.ToList();
            var byKey = Comparer<SceneGame>.Create((a, b) => Compare(a, b, order));
            IOrderedEnumerable<SceneGame> sorted = favoritesFirst ? list.OrderByDescending(g => g.Favorite).ThenBy(g => g, byKey) : list.OrderBy(g => g, byKey);
            return sorted.ThenBy(SortKey, StringComparer.OrdinalIgnoreCase).ToList();
        }

        // ES-DE's sortname where the player set one (§4.59), else the name; the quick selector indexes by it too (USERGUIDE, "sortname").
        public static string SortKey(SceneGame g) => string.IsNullOrWhiteSpace(g.SortName) ? g.Name : g.SortName;

        private static int Compare(SceneGame a, SceneGame b, GameSort order)
        {
            if (order.Key == GameSortKey.Name)
            {
                int byName = StringComparer.OrdinalIgnoreCase.Compare(SortKey(a), SortKey(b));
                return order.Descending ? -byName : byName;
            }

            IComparable? x = KeyOf(a, order.Key), y = KeyOf(b, order.Key);
            if (x is null || y is null) return x is null ? (y is null ? 0 : 1) : -1;
            int c = x is string sx && y is string sy ? StringComparer.OrdinalIgnoreCase.Compare(sx, sy) : x.CompareTo(y);
            return order.Descending ? -c : c;
        }

        private static IComparable? KeyOf(SceneGame g, GameSortKey key) => key switch
        {
            GameSortKey.Rating => g.Rating,
            GameSortKey.ReleaseDate => g.ReleaseDate,
            GameSortKey.Developer => Text(g.Developer),
            GameSortKey.Publisher => Text(g.Publisher),
            GameSortKey.Genre => Text(g.Genre),
            GameSortKey.Players => PlayerCount(g.Players),
            GameSortKey.LastPlayed => g.LastPlayed,
            GameSortKey.TimesPlayed => g.PlayCount,
            GameSortKey.PlayTime => g.PlayTime ?? TimeSpan.Zero,
            GameSortKey.System => g.Source?.FullName,
            _ => g.Name,
        };

        private static string? Text(string? value) => string.IsNullOrWhiteSpace(value) ? null : value.Trim();

        // "1-4" sorts as four players: the highest count the text names.
        private static int? PlayerCount(string? players)
        {
            if (Text(players) is not { } text) return null;
            int? best = null;
            foreach (string part in text.Split(['-', ',', '+', ' ', '/'], StringSplitOptions.RemoveEmptyEntries))
                if (int.TryParse(part, NumberStyles.Integer, CultureInfo.InvariantCulture, out int n)) best = Math.Max(best ?? n, n);
            return best;
        }

        // A game's value for a filter, as the filter screen lists it; null for a game without one.
        public static string? ValueOf(SceneGame g, FilterField field) => field switch
        {
            FilterField.Rating => g.Rating is { } r ? Stars(r) : null,
            FilterField.Developer => Text(g.Developer),
            FilterField.Publisher => Text(g.Publisher),
            FilterField.Genre => Text(g.Genre),
            FilterField.Players => Text(g.Players),
            FilterField.Favorite => YesNo(g.Favorite),
            FilterField.Completed => YesNo(g.Completed),
            FilterField.KidGame => YesNo(g.KidGame),
            FilterField.Broken => YesNo(g.Broken),
            _ => null,
        };

        private static string YesNo(bool value) => value ? "Yes" : "No";

        // Half-star steps, as ES-DE's rating is drawn.
        public static string Stars(float rating)
        {
            double stars = Math.Round(Math.Clamp(rating, 0, 1) * 10, MidpointRounding.AwayFromZero) / 2;
            return stars.ToString("0.#", CultureInfo.InvariantCulture) + (stars == 1 ? " star" : " stars");
        }

        // The values a filter offers, from the games themselves; empty when there is nothing to tell them apart by (USERGUIDE's "Nothing to filter").
        public static IReadOnlyList<string> Values(IEnumerable<SceneGame> games, FilterField field)
        {
            var files = games.Where(g => !g.Folder).ToList();
            var known = files.Select(g => ValueOf(g, field)).ToList();
            if (known.All(v => v is null)) return [];
            var values = known.Select(v => v ?? Unknown).Distinct(StringComparer.OrdinalIgnoreCase).ToList();
            if (values.Count < 2) return [];
            return field == FilterField.Rating
                ? values.OrderBy(v => v == Unknown ? double.MaxValue : double.Parse(v.Split(' ')[0], CultureInfo.InvariantCulture)).ToList()
                : values.OrderBy(v => v == Unknown ? 1 : 0).ThenBy(v => v, StringComparer.OrdinalIgnoreCase).ToList();
        }

        // Every chosen field must hold, a field held when the game's value is one of those chosen; the name matches anywhere, ignoring case; folders are never filtered.
        public static bool Passes(SceneGame g, GameFilter filter)
        {
            if (g.Folder) return true;
            if (filter.Name.Trim() is { Length: > 0 } name && !g.Name.Contains(name, StringComparison.OrdinalIgnoreCase)) return false;
            foreach ((FilterField field, IReadOnlySet<string> chosen) in filter.Chosen)
                if (chosen.Count > 0 && !chosen.Contains(ValueOf(g, field) ?? Unknown)) return false;
            return true;
        }

        public static IReadOnlyList<SceneGame> Apply(IEnumerable<SceneGame> games, GameFilter filter) =>
            filter.IsActive ? games.Where(g => Passes(g, filter)).ToList() : games.ToList();

        // The quick selector's entry for the folders sorted on top, where ES-DE draws a folder icon (§30).
        public const string FolderEntry = "Folders";

        // The quick selector's entries: the folders and a star for the favourites when each is sorted on top among other entries, then each first character in list order.
        public static IReadOnlyList<(string Label, int Index)> Letters(IReadOnlyList<SceneGame> games, bool favoritesOnTop, bool foldersOnTop = false)
        {
            var entries = new List<(string, int)>();
            bool folders = foldersOnTop && games.Any(g => g.Folder && !g.IsCollection) && games.Any(g => !g.Folder);
            if (folders) entries.Add((FolderEntry, 0));
            var files = games.Where(g => !g.Folder).ToList();
            bool star = favoritesOnTop && files.Any(g => g.Favorite) && files.Any(g => !g.Favorite);
            if (star) entries.Add((Star, games.ToList().FindIndex(g => !g.Folder)));
            for (int i = 0; i < games.Count; i++)
            {
                if (folders && games[i].Folder) continue;
                if (star && games[i].Favorite && !games[i].Folder) continue;
                string first = FirstLetter(SortKey(games[i]));
                if (!entries.Any(e => e.Item1 == first)) entries.Add((first, i));
            }
            return entries;
        }

        public static string FirstLetter(string name) => name.TrimStart() is { Length: > 0 } t ? char.ToUpperInvariant(t[0]).ToString() : "#";

        // Another entry than the one selected, when there is another.
        public static int RandomIndex(int count, int current, Random random)
        {
            if (count <= 1) return Math.Max(0, Math.Min(current, count - 1));
            int pick = random.Next(count - 1);
            return pick >= current ? pick + 1 : pick;
        }
    }
}
