using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Pages
{
    // A shelf of the systems table as Choose a console names it: its key, its name, its full name, maker, year and planet hue.
    public sealed record ConsoleShelf(string Key, string Label, string FullName, string Maker, int Year, double Hue);

    // One world on Choose a console: a wide place along the top or a console's planet, its words, the covers fanned beneath and the engine it runs on.
    public sealed record ConsoleChoice(string Key, string Name, double? Hue, string Status, string Title, string Facts, IReadOnlyList<GridGame> Covers, EngineShown? Engine = null);

    // Choose a console as one plain description: All games, Favourites and Recently played, then each console of the systems table that has a game - see EmuSen_Interface.md §24.
    public sealed class ConsolesScreen(IGamesHost host)
    {
        public const string Title = "Games", Subtitle = "Choose a console";
        public const string All = "all", Favourites = "favourites", Recent = "recent";
        public const int FanCount = 4;

        // The engine line's tags: one of EmuSen's own cores, or a hosted libretro core.
        public const string OwnTag = "EmuSen", LibretroTag = "libretro";

        // The wide places' icons, the mockup's paths in a 24-unit box.
        public static readonly IReadOnlyDictionary<string, string> Icons = new Dictionary<string, string>
        {
            [All] = "M4 5h7v6H4zM13 5h7v6h-7zM4 13h7v6H4zM13 13h7v6h-7z",
            [Favourites] = "M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z",
            [Recent] = "M12 7v5l3 2M21 12a9 9 0 1 1-3-6.7M21 4v5h-5",
        };

        private readonly IGamesHost _host = host;

        public IGamesHost Host => _host;

        private static string Count(int n) => n == 1 ? "1 game" : $"{n.ToString("N0", CultureInfo.InvariantCulture)} games";

        // The three wide places, as the mockup words them.
        public IReadOnlyList<ConsoleChoice> Places
        {
            get
            {
                IReadOnlyList<GridGame> all = _host.Games;
                List<GridGame> favourites = all.Where(g => g.Facts.Favourite).ToList(), recent = RecentOf(all, GamesScreen.RecentLimit);
                string last = recent.FirstOrDefault() is { } r ? "Last: " + r.Facts.Title : "Nothing played yet";
                return
                [
                    new(All, "All games", null, Count(all.Count), "All games", Count(all.Count), Fan(all)),
                    new(Favourites, "Favourites", null, "Your hearts, every console", "Favourites", Count(favourites.Count), Fan(favourites)),
                    new(Recent, "Recently played", null, last, "Recently played", last, recent.Take(FanCount).ToList()),
                ];
            }
        }

        // Each console of the systems table with at least one game, in the table's order.
        public IReadOnlyList<ConsoleChoice> Consoles
        {
            get
            {
                IReadOnlyList<GridGame> all = _host.Games;
                var choices = new List<ConsoleChoice>();
                foreach (ConsoleShelf shelf in _host.Table)
                {
                    List<GridGame> games = all.Where(g => g.Shelf == shelf.Key).ToList();
                    if (games.Count == 0) continue;
                    EngineShown? engine = _host.EngineOf(shelf.Key);
                    string facts = string.Join(" · ", new[] { shelf.Maker, shelf.Year > 0 ? shelf.Year.ToString(CultureInfo.InvariantCulture) : "", Count(games.Count), RunsOn(engine) }.Where(s => s.Length > 0));
                    choices.Add(new(shelf.Key, shelf.Label, shelf.Hue, Count(games.Count), shelf.FullName, facts, Fan(games), engine));
                }
                return choices;
            }
        }

        // The tag an engine line carries: libretro for a hosted libretro core, EmuSen for one of the project's own.
        public static string TagOf(EngineShown engine) => engine.Libretro ? LibretroTag : OwnTag;

        // The facts' last words: which engine runs the console now.
        public static string RunsOn(EngineShown? engine) => engine is null ? "" : $"runs on {engine.Name}, {(engine.Libretro ? "a libretro core" : "an EmuSen core")}";

        // The header's right: how many consoles and games.
        public string CountLine
        {
            get
            {
                int consoles = Consoles.Count;
                return $"{(consoles == 1 ? "1 console" : $"{consoles} consoles")} · {Count(_host.Games.Count)}";
            }
        }

        private List<GridGame> RecentOf(IReadOnlyList<GridGame> games, int limit)
        {
            var byPath = games.ToDictionary(g => g.Facts.Path, StringComparer.Ordinal);
            return _host.Recent(games.Select(g => g.Facts.Path).ToList(), limit).Where(byPath.ContainsKey).Select(p => byPath[p]).ToList();
        }

        // Up to four covers: the recently played first, then by name.
        private List<GridGame> Fan(IReadOnlyList<GridGame> games)
        {
            List<GridGame> fan = RecentOf(games, FanCount);
            fan.AddRange(games.Where(g => !fan.Contains(g)).OrderBy(g => g.Facts.Title, StringComparer.CurrentCultureIgnoreCase).Take(FanCount - fan.Count));
            return fan;
        }

        // The grid's section a choice opens.
        public static string SectionOf(string key) => key switch
        {
            All => GamesScreen.All,
            Favourites => GamesScreen.Favourites,
            Recent => GamesScreen.Recent,
            _ => GamesScreen.ConsolePrefix + key,
        };

        // The choice a grid's section returns to: its console, or the place it came from.
        public static string? KeyOf(string section) => section switch
        {
            GamesScreen.All => All,
            GamesScreen.Favourites => Favourites,
            GamesScreen.Recent => Recent,
            _ when section.StartsWith(GamesScreen.ConsolePrefix, StringComparison.Ordinal) => section[GamesScreen.ConsolePrefix.Length..],
            _ => null,
        };

        // X: a game chosen at random from the focused choice's games.
        public string? RandomGame(string key)
        {
            IReadOnlyList<GridGame> all = _host.Games;
            List<GridGame> games = key switch
            {
                All => all.ToList(),
                Favourites => all.Where(g => g.Facts.Favourite).ToList(),
                Recent => RecentOf(all, GamesScreen.RecentLimit),
                _ => all.Where(g => g.Shelf == key).ToList(),
            };
            return games.Count == 0 ? null : games[Math.Clamp(_host.Random(games.Count), 0, games.Count - 1)].Facts.Path;
        }

        // The help bar, as the mockup gives it.
        public static IReadOnlyList<ScreenHint> Hints =>
            [new("South", "Open"), new("East", "Back", "back"), new("North", "Favourites", "favourites"), new("West", "Random game", "random")];
    }
}
