using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture
{
    // One of Mistress's own collections from games.db, as big picture lists it.
    public sealed record CustomCollection(long Id, string Name, IReadOnlySet<string> Members);

    // ES-DE's automatic collections and the player's custom ones as systems of the carousel, built from Mistress's own records - see EmuSen_BigPicture.md §22.
    public static class CollectionShelves
    {
        public static readonly ThemeSystem AllGames = new(BigPictureCollections.AllGames, "all games", "auto-allgames", ThemeSystemKind.AutoCollection);
        public static readonly ThemeSystem Favorites = new(BigPictureCollections.Favorites, "favorites", "auto-favorites", ThemeSystemKind.AutoCollection);
        public static readonly ThemeSystem LastPlayed = new(BigPictureCollections.LastPlayed, "last played", "auto-lastplayed", ThemeSystemKind.AutoCollection);
        public static readonly ThemeSystem Grouped = new("collections", "collections", "custom-collections", ThemeSystemKind.CustomCollection);

        // USERGUIDE: "a list of the 50 last games you have launched".
        public const int LastPlayedLimit = 50;

        public const string FolderPrefix = "collection:";

        // USERGUIDE's characters a collection name cannot hold.
        public const string ForbiddenInName = "*\",./:;<>\\|";

        public static ThemeSystem Custom(string name) => new(name, name, name, ThemeSystemKind.CustomCollection);

        public static string Clean(string name) => new string(name.Where(c => !ForbiddenInName.Contains(c)).ToArray()).Trim();

        // ES-DE's rule for a name already taken: a number in brackets, the first one free.
        public static string Unique(string name, IEnumerable<string> taken)
        {
            var names = new HashSet<string>(taken, StringComparer.OrdinalIgnoreCase);
            if (!names.Contains(name)) return name;
            for (int n = 1; ; n++)
                if (!names.Contains($"{name} ({n})")) return $"{name} ({n})";
        }

        // As ES-DE 3.4.1 was measured to order them (§22.2): the regular systems, the grouped custom collections, then all games, favorites and last played; discrete custom collections sit beside the grouped one.
        public static IReadOnlyList<ThemedShelf> Build(IReadOnlyList<ThemedShelf> systems, IReadOnlyList<CustomCollection> custom, BigPictureCollections settings, Func<string, bool> themedByTheme)
        {
            var all = systems.SelectMany(s => s.Games.Select(g => g.Source is null ? g with { Source = s.System } : g)).ToList();
            var shelves = new List<ThemedShelf>(systems);

            var byFile = all.GroupBy(g => g.File, StringComparer.Ordinal).ToDictionary(g => g.Key, g => g.First(), StringComparer.Ordinal);
            var shown = custom.Where(c => !settings.HiddenCustomCollections.Contains(c.Id)).OrderBy(c => c.Name, StringComparer.OrdinalIgnoreCase).ToList();
            var folders = new List<ThemedShelf>();
            var discrete = new List<ThemedShelf>();
            foreach (CustomCollection c in shown)
            {
                var shelf = new ThemedShelf(Custom(c.Name), c.Members.Where(byFile.ContainsKey).Select(f => byFile[f]).ToList())
                {
                    FavoritesFirst = settings.FavoritesFirstCustom, Stars = settings.StarsCustom, CollectionId = c.Id,
                };
                bool alone = settings.GroupCustomCollections switch
                {
                    BigPictureCollections.GroupNever => true,
                    BigPictureCollections.GroupAlways => false,
                    _ => themedByTheme(c.Name),
                };
                (alone ? discrete : folders).Add(shelf);
            }

            if (folders.Count > 0) shelves.Add(new ThemedShelf(Grouped, folders.SelectMany(f => f.Games).DistinctBy(g => g.File).ToList()) { Folders = folders });
            shelves.AddRange(discrete);

            bool favFirst = settings.FavoritesFirst;
            foreach (string auto in new[] { BigPictureCollections.AllGames, BigPictureCollections.Favorites, BigPictureCollections.LastPlayed })
            {
                if (!settings.AutoCollections.Contains(auto)) continue;
                shelves.Add(auto switch
                {
                    BigPictureCollections.AllGames => new ThemedShelf(AllGames, all) { FavoritesFirst = favFirst },
                    BigPictureCollections.Favorites => new ThemedShelf(Favorites, all.Where(g => g.Favorite).ToList()) { FavoritesFirst = favFirst, Stars = false },
                    _ => new ThemedShelf(LastPlayed, all.Where(g => g.LastPlayed is not null || g.PlayCount > 0).OrderByDescending(g => g.LastPlayed ?? DateTime.MinValue).Take(LastPlayedLimit).ToList())
                    {
                        FavoritesFirst = false, DefaultSort = new GameSort(GameSortKey.LastPlayed, true),
                    },
                });
            }

            return shelves;
        }

        // A grouped collection as a folder row: its name, and ES-DE's description naming its games in a random order, the first of them the one shown (§22.2).
        public static SceneGame Folder(ThemedShelf collection, Random random)
        {
            var games = collection.Games.OrderBy(_ => random.Next()).ToList();
            IEnumerable<string> names = games.Select(g => $"'{g.Name} [{(g.Source ?? collection.System).Name.ToUpperInvariant()}]'");
            string list = games.Count switch
            {
                0 => "",
                1 => ": " + names.First(),
                _ => ": " + string.Join(", ", names.SkipLast(1)) + " and " + names.Last(),
            };
            return new SceneGame(collection.System.Name, FolderPrefix + collection.System.Name)
            {
                Folder = true, IsCollection = true, HideMetadata = true,
                Description = $"This collection contains {games.Count} {(games.Count == 1 ? "game" : "games")}{list}",
                Face = games.FirstOrDefault(),
            };
        }
    }
}
