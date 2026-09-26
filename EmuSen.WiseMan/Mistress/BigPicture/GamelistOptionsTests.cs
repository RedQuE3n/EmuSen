using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The sorts, filters, quick selector, random entry and collection shelves with no view behind them - see EmuSen_BigPicture.md §22.
    public class GamelistOptionsTests
    {
        private static readonly ThemeSystem Nes = new("nes", "Nintendo Entertainment System", "nes"), Snes = new("snes", "Super Nintendo", "snes");

        private static SceneGame G(string name, float? rating = null, bool fav = false, string? genre = null, string? players = null, ThemeSystem? source = null,
            DateTime? played = null, int count = 0) =>
            new(name, "/roms/" + name + (source == Nes ? ".nes" : ".sfc")) { Rating = rating, Favorite = fav, Genre = genre, Players = players, Source = source, LastPlayed = played, PlayCount = count };

        private static string[] Names(IEnumerable<SceneGame> games) => games.Select(g => g.Name).ToArray();

        [Fact]
        public void Every_documented_sort_key_orders_with_missing_values_last_and_the_name_ascending_after()
        {
            SceneGame[] games = [G("Delta", 0.6f), G("alpha", 0.8f), G("Charlie"), G("Bravo", 0.6f), G("Echo", 0.2f)];
            Assert.Equal(["alpha", "Bravo", "Charlie", "Delta", "Echo"], Names(GamelistOptions.Sort(games, GameSort.Default, false)));
            Assert.Equal(["Echo", "Delta", "Charlie", "Bravo", "alpha"], Names(GamelistOptions.Sort(games, new GameSort(GameSortKey.Name, true), false)));
            Assert.Equal(["Echo", "Bravo", "Delta", "alpha", "Charlie"], Names(GamelistOptions.Sort(games, new GameSort(GameSortKey.Rating, false), false)));
            Assert.Equal(["alpha", "Bravo", "Delta", "Echo", "Charlie"], Names(GamelistOptions.Sort(games, new GameSort(GameSortKey.Rating, true), false)));

            // "1-4" is four players; a text without a number goes last with the games that have none.
            SceneGame[] party = [G("One", players: "1"), G("Four", players: "1-4"), G("Two", players: "2"), G("None"), G("Many", players: "lots")];
            Assert.Equal(["Four", "Two", "One", "Many", "None"], Names(GamelistOptions.Sort(party, new GameSort(GameSortKey.Players, true), false)));

            // Favourites on top when asked, each part in the order.
            SceneGame[] favs = [G("Bravo"), G("Alpha"), G("Zulu", fav: true), G("Mike", fav: true)];
            Assert.Equal(["Mike", "Zulu", "Alpha", "Bravo"], Names(GamelistOptions.Sort(favs, GameSort.Default, true)));
            Assert.Equal(["Alpha", "Bravo", "Mike", "Zulu"], Names(GamelistOptions.Sort(favs, GameSort.Default, false)));

            // System, for collections, by the source system's full name; same names keep the order they came in.
            SceneGame[] mixed = [G("Same", source: Snes), G("Same", source: Nes), G("Other", source: Snes)];
            Assert.Equal(["Same", "Other", "Same"], Names(GamelistOptions.Sort(mixed, new GameSort(GameSortKey.System, false), false)));
            Assert.Equal(new[] { "nes", "snes", "snes" }, GamelistOptions.Sort(mixed, new GameSort(GameSortKey.System, false), false).Select(g => g.Source!.Name));
            Assert.Equal(new[] { "snes", "nes" }, GamelistOptions.Sort(mixed.Take(2), GameSort.Default, false).Select(g => g.Source!.Name));

            // The stored spelling is ES-DE's DefaultSortOrder value, and System is offered for collections only.
            Assert.Equal("release date, descending", new GameSort(GameSortKey.ReleaseDate, true).Stored);
            Assert.Equal(new GameSort(GameSortKey.TimesPlayed, true), GameSort.Parse("times played, descending"));
            Assert.Equal(GameSort.Default, GameSort.Parse("nonsense"));
            Assert.DoesNotContain(GameSort.All(false), s => s.Key == GameSortKey.System);
            Assert.Equal(22, GameSort.All(true).Count);
        }

        [Fact]
        public void Filter_values_come_from_the_list_and_a_field_with_nothing_to_tell_apart_offers_none()
        {
            SceneGame[] games = [G("A", genre: "Racing", rating: 0.8f), G("B", genre: "puzzle", rating: 0.8f), G("C", genre: "Racing"), G("D", fav: true)];
            Assert.Equal(["puzzle", "Racing", GamelistOptions.Unknown], GamelistOptions.Values(games, FilterField.Genre));
            Assert.Equal(["4 stars", GamelistOptions.Unknown], GamelistOptions.Values(games, FilterField.Rating));
            Assert.Equal(["No", "Yes"], GamelistOptions.Values(games, FilterField.Favorite));
            Assert.Empty(GamelistOptions.Values(games, FilterField.Publisher));
            Assert.Empty(GamelistOptions.Values(games, FilterField.Completed));
            Assert.Empty(GamelistOptions.Values(games.Take(1), FilterField.Genre));
            Assert.Equal("2.5 stars", GamelistOptions.Stars(0.5f));
            Assert.Equal("1 star", GamelistOptions.Stars(0.2f));

            // Any chosen value within a field, every field, the name anywhere in any case; a folder always passes.
            var filter = GameFilter.None.With(FilterField.Genre, new HashSet<string> { "Racing", GamelistOptions.Unknown });
            Assert.Equal(["A", "C", "D"], Names(GamelistOptions.Apply(games, filter)));
            filter = filter.With(FilterField.Favorite, new HashSet<string> { "No" });
            Assert.Equal(["A", "C"], Names(GamelistOptions.Apply(games, filter)));
            Assert.Equal(["C"], Names(GamelistOptions.Apply(games, filter with { Name = "c" })));
            Assert.True(GamelistOptions.Passes(new SceneGame("X", "folder") { Folder = true }, filter with { Name = "zzz" }));
            Assert.False(GameFilter.None.IsActive);
            Assert.False(filter.With(FilterField.Genre, new HashSet<string>()).With(FilterField.Favorite, new HashSet<string>()).IsActive);
        }

        [Fact]
        public void The_quick_selector_offers_only_the_letters_present_and_a_star_for_favourites_on_top()
        {
            IReadOnlyList<SceneGame> list = GamelistOptions.Sort([G("Kestrel", fav: true), G("aurora"), G("Brass"), G("Bolt"), G("9 Lives")], GameSort.Default, true);
            Assert.Equal([("★", 0), ("9", 1), ("A", 2), ("B", 3)], GamelistOptions.Letters(list, favoritesOnTop: true));
            Assert.Equal([("9", 0), ("A", 1), ("B", 2), ("K", 4)], GamelistOptions.Letters(GamelistOptions.Sort(list, GameSort.Default, false), favoritesOnTop: false));
            // Only favourites: indexed by their letters, no star.
            Assert.Equal([("K", 0)], GamelistOptions.Letters([G("Kestrel", fav: true)], favoritesOnTop: true));
        }

        [Fact]
        public void The_random_entry_never_picks_the_game_already_selected_when_there_is_another()
        {
            var random = new Random(7);
            var seen = new HashSet<int>();
            for (int i = 0; i < 400; i++)
            {
                int pick = GamelistOptions.RandomIndex(5, 2, random);
                Assert.NotEqual(2, pick);
                Assert.InRange(pick, 0, 4);
                seen.Add(pick);
            }
            Assert.Equal(4, seen.Count);
            Assert.Equal(0, GamelistOptions.RandomIndex(1, 0, random));
        }

        [Fact]
        public void Collection_names_lose_ES_DE_s_forbidden_characters_and_a_taken_name_is_numbered()
        {
            Assert.Equal("Beat em Up", CollectionShelves.Clean("Beat: em/ Up*"));
            Assert.Equal("ab", CollectionShelves.Clean("a*\",./:;<>\\|b"));
            Assert.Equal("Platform (1)", CollectionShelves.Unique("Platform", ["platform"]));
            Assert.Equal("Platform (2)", CollectionShelves.Unique("Platform", ["Platform", "Platform (1)"]));
            Assert.Equal("New", CollectionShelves.Unique("New", ["Old"]));
        }

        private static ThemedShelf Shelf(ThemeSystem system, params SceneGame[] games) => new(system, games);

        [Fact]
        public void Collection_shelves_follow_ES_DE_s_measured_order_and_group_what_the_theme_does_not_style()
        {
            ThemedShelf nes = Shelf(Nes, G("Fable", source: null) with { File = "/n/fable.nes" }), snes = Shelf(Snes, G("Aurora") with { File = "/s/a.sfc", Favorite = true }, G("Brass") with { File = "/s/b.sfc" });
            var custom = new[] { new CustomCollection(1, "Platform", new HashSet<string> { "/s/a.sfc", "/n/fable.nes", "/gone.sfc" }), new CustomCollection(2, "Beat", new HashSet<string> { "/s/b.sfc" }) };
            var settings = new BigPictureCollections { AutoCollections = [BigPictureCollections.LastPlayed, BigPictureCollections.AllGames, BigPictureCollections.Favorites] };

            IReadOnlyList<ThemedShelf> shelves = CollectionShelves.Build([nes, snes], custom, settings, _ => false);
            Assert.Equal(["nes", "snes", "collections", "all", "favorites", "recent"], shelves.Select(s => s.System.Name));
            ThemedShelf grouped = shelves[2];
            Assert.Equal(["Beat", "Platform"], grouped.Folders!.Select(f => f.System.Name));
            Assert.Equal(["Aurora", "Fable"], grouped.Folders![1].Games.Select(g => g.Name).Order());
            Assert.Equal("nes", grouped.Folders![1].Games.Single(g => g.Name == "Fable").Source!.Name);
            Assert.All(grouped.Folders!, f => Assert.False(f.Stars));
            Assert.Equal(3, shelves[3].Games.Count);
            Assert.Equal(["Aurora"], shelves[4].Games.Select(g => g.Name));
            Assert.False(shelves[4].Stars);
            Assert.True(shelves[3].Stars);

            // A theme folder of the collection's name makes it a system of its own, beside the grouped one; Always groups it anyway, Never groups nothing.
            Assert.Equal(["nes", "snes", "collections", "Platform", "all", "favorites", "recent"], CollectionShelves.Build([nes, snes], custom, settings, n => n == "Platform").Select(s => s.System.Name));
            settings.GroupCustomCollections = BigPictureCollections.GroupAlways;
            Assert.Equal(["nes", "snes", "collections", "all", "favorites", "recent"], CollectionShelves.Build([nes, snes], custom, settings, n => n == "Platform").Select(s => s.System.Name));
            settings.GroupCustomCollections = BigPictureCollections.GroupNever;
            Assert.Equal(["nes", "snes", "Beat", "Platform", "all", "favorites", "recent"], CollectionShelves.Build([nes, snes], custom, settings, _ => false).Select(s => s.System.Name));
            Assert.Equal(ThemeSystemKind.CustomCollection, CollectionShelves.Build([nes, snes], custom, settings, _ => false)[2].System.Kind);

            // A hidden collection is left out; with nothing on, the systems alone.
            settings.HiddenCustomCollections = [1, 2];
            settings.AutoCollections = [];
            Assert.Equal(["nes", "snes"], CollectionShelves.Build([nes, snes], custom, settings, _ => false).Select(s => s.System.Name));
        }

        [Fact]
        public void Last_played_is_newest_first_across_systems_fifty_at_most_with_a_counted_game_without_a_date_last()
        {
            var start = new DateTime(2026, 9, 1);
            var played = Enumerable.Range(0, 60).Select(i => G($"G{i:D2}", played: start.AddHours(i), count: 1) with { File = $"/s/{i}.sfc" }).ToList();
            played.Add(G("Counted", count: 9) with { File = "/s/c.sfc" });
            played.Add(G("Never") with { File = "/s/n.sfc" });
            var settings = new BigPictureCollections { AutoCollections = [BigPictureCollections.LastPlayed] };
            ThemedShelf recent = CollectionShelves.Build([new ThemedShelf(Snes, played)], [], settings, _ => false).Last();
            Assert.Equal(CollectionShelves.LastPlayed, recent.System);
            Assert.Equal(CollectionShelves.LastPlayedLimit, recent.Games.Count);
            Assert.Equal("G59", recent.Games[0].Name);
            Assert.Equal(new GameSort(GameSortKey.LastPlayed, true), recent.DefaultSort);
            Assert.False(recent.FavoritesFirst);

            var few = played.Skip(57).ToList();
            ThemedShelf short_ = CollectionShelves.Build([new ThemedShelf(Snes, few)], [], settings, _ => false).Last();
            Assert.Equal(["G59", "G58", "G57", "Counted"], GamelistOptions.Sort(short_.Games, short_.DefaultSort!.Value, false).Select(g => g.Name));
        }

        [Fact]
        public void A_grouped_collection_s_folder_names_its_games_in_a_random_order_and_shows_the_first()
        {
            ThemedShelf platform = new(CollectionShelves.Custom("Platform"), [G("Hollow Comet", source: Snes), G("Ember Circuit", source: Snes), G("Fable", source: Nes)]);
            SceneGame folder = CollectionShelves.Folder(platform, new Random(1));
            Assert.True(folder.Folder);
            Assert.Equal("Platform", folder.Name);
            Assert.StartsWith("This collection contains 3 games: '", folder.Description);
            Assert.Contains("'Fable [NES]'", folder.Description);
            Assert.Contains("' and '", folder.Description);
            string first = folder.Description!.Split(": '")[1].Split(" [")[0];
            Assert.Equal(first, folder.Face!.Name);
            Assert.Equal("This collection contains 1 game: 'Solo [SNES]'", CollectionShelves.Folder(new(CollectionShelves.Custom("One"), [G("Solo", source: Snes)]), new Random(1)).Description);
            Assert.Equal("This collection contains 0 games", CollectionShelves.Folder(new(CollectionShelves.Custom("None"), []), new Random(1)).Description);

            // Some seed puts another game first; the description's order is not fixed.
            Assert.Contains(Enumerable.Range(0, 20), seed => CollectionShelves.Folder(platform, new Random(seed)).Face!.Name != folder.Face!.Name);
        }
    }
}
