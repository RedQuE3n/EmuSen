using System.Collections.Generic;

namespace EmuSen.Galaxia.Models
{
    // Big picture's collection and gamelist settings, spelled as ES-DE's es_settings.xml values - see EmuSen_Settings_Reference.md §4.58.
    public sealed class BigPictureCollections
    {
        public const string AllGames = "all", Favorites = "favorites", LastPlayed = "recent";
        public const string GroupUnthemed = "unthemed", GroupAlways = "always", GroupNever = "never";
        public const string RandomGames = "games", RandomGamesAndSystems = "gamessystems", RandomDisabled = "disabled";

        // All three on, by the choice of 2026-09-26 (Q11), where ES-DE 3.4.1 writes CollectionSystemsAuto empty.
        public List<string> AutoCollections { get; set; } = new() { AllGames, Favorites, LastPlayed };

        // Mistress's own collections left out of big picture, by games.db id; every other one is shown.
        public List<long> HiddenCustomCollections { get; set; } = new();

        public string GroupCustomCollections { get; set; } = GroupUnthemed;

        public bool FavoritesFirst { get; set; } = true;

        public bool FavoritesFirstCustom { get; set; }

        public bool StarsCustom { get; set; }

        public string DefaultSortOrder { get; set; } = "name, ascending";

        public string RandomEntryButton { get; set; } = RandomGames;
    }
}
