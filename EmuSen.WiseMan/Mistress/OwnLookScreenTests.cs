using EmuSen.Galaxia.Native;
using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using Xunit;

namespace EmuSen.WiseMan.Mistress
{
    // Home's and the grid's plain descriptions on fake hosts, with no window: the words, the places, the shelf, the sorts, the search and the List card - see EmuSen_BigPicture.md §51.
    public class OwnLookScreenTests
    {
        private static readonly DateTime Clock = new(2026, 11, 1, 21, 42, 0);

        private static GameFacts Facts(string path, string title, string console = "Super Nintendo", string? year = "1994", bool favourite = false, DateTime? last = null, double hours = 0) =>
            new(path, title, console, "SNES", year, "Studio A", null, TimeSpan.FromHours(hours), last, null, favourite, 1.4);

        private sealed class FakeHome : IHomeHost
        {
            public List<string> Said { get; } = new();
            public DateTime Now => Clock;
            public bool Desktop { get; set; }
            public HomeSuspended? Suspended { get; set; }
            public IReadOnlyList<string> Recent { get; set; } = ["/g/a.sfc", "/g/b.sfc"];
            public GameFacts Facts(string path) => OwnLookScreenTests.Facts(path, path == "/g/a.sfc" ? "Granite Choir" : "Basalt Run", last: Clock.AddDays(-1), hours: 14.33);
            public int GameCount { get; set; } = 1284;
            public Dictionary<string, ToolSummary> Summaries { get; set; } = new()
            {
                [HomeScreen.Scraper] = new("41 without art", "41 no art"),
                [HomeScreen.Library] = new("2 suspect dumps", "2 suspect", NeedsAttention: true),
                [HomeScreen.Cores] = new("1 to approve", NeedsAttention: true),
            };
            public ToolSummary? Summary(string place) => Summaries.GetValueOrDefault(place);
            public int ControllerCount => 2;
            public string? Battery { get; set; }
            public string SettingsLine => "Dark · 60% sound";
            public bool BigPictureForced { get; set; }
            public void Resume() => Said.Add("resume");
            public void Play(string path) => Said.Add("play " + path);
            public void OpenPage(string path) => Said.Add("page " + path);
            public void ToggleFavourite(string path) => Said.Add("favourite " + path);
            public void Open(string place) => Said.Add("open " + place);
        }

        [Fact]
        public void Home_reads_as_the_design_draws_it_and_its_places_carry_their_lines()
        {
            var host = new FakeHome { Suspended = new HomeSuspended("/g/a.sfc", "Granite Choir", "Super Nintendo", Clock.AddMinutes(-12)) };
            var home = new HomeScreen(host);
            Assert.Equal("2 controllers · 21:42", home.StatusLine);
            Assert.Equal(("Granite Choir", "Super Nintendo · Paused 12 minutes ago"), home.Continue);
            Assert.Equal(["/g/b.sfc"], home.Shelf.Select(g => g.Path));
            Assert.Equal("Super Nintendo · 1994 · Studio A · 14 h 19 min played · Yesterday", home.Shelf[0].Facts);
            Assert.Equal(["Games", "Scraper", "Packages", "Library", "Cores", "Settings", "Power"], home.Places.Select(p => p.Label));
            Assert.Equal(["1,284 games", "41 no art"], home.Places.Take(2).Select(p => p.Status));
            Assert.True(home.Places.Single(p => p.Key == HomeScreen.Cores).NeedsAttention);
            Assert.Equal("1 to approve", home.Places.Single(p => p.Key == HomeScreen.Cores).Status);
            Assert.Equal("Exit", home.Places.Last().Status);
            Assert.Equal("Game packages", home.Places.Single(p => p.Key == HomeScreen.Packages).Status);
            Assert.Equal(("2 suspect", true), home.Places.Single(p => p.Key == HomeScreen.Library) is var l ? (l.Status, l.NeedsAttention) : default);
        }

        [Fact]
        public void The_desktop_s_places_take_each_tool_s_whole_line_and_a_tool_that_has_said_nothing_its_description()
        {
            var host = new FakeHome { Desktop = true };
            host.Summaries[HomeScreen.Cheats] = new("12 on · 3 games");
            var home = new HomeScreen(host);
            Assert.Equal(["1,284 games", "41 without art", "Game packages", "2 suspect dumps", "1 to approve", "12 on · 3 games"], home.Places.Take(6).Select(p => p.Status));
            Assert.Equal([false, false, false, true, true, false], home.Places.Take(6).Select(p => p.NeedsAttention));
            host.Summaries.Clear();
            Assert.Equal(["Art and facts", "Game packages", "Checks dumps", "The engines", "Codes for games"], new HomeScreen(host).Places.Skip(1).Take(5).Select(p => p.Status));
        }

        [Fact]
        public void The_desktop_has_eight_places_and_Game_Mode_quits_rather_than_exits()
        {
            var host = new FakeHome { Desktop = true, Battery = "80%" };
            host.Summaries[HomeScreen.Cores] = new("17 cores · 5 installed");
            var home = new HomeScreen(host);
            Assert.Equal(["Games", "Scraper", "Packages", "Library Manager", "Cores", "Cheats", "Settings", "Big Picture"], home.Places.Select(p => p.Label));
            Assert.False(home.Places.Single(p => p.Key == HomeScreen.Cores).NeedsAttention);
            Assert.Equal("2 controllers · 80% · 21:42", home.StatusLine);
            Assert.Null(home.Continue);
            Assert.Equal("Quit", new HomeScreen(new FakeHome { BigPictureForced = true }).Places.Last().Status);
        }

        [Fact]
        public void Each_row_names_its_buttons_and_X_and_Y_act_on_the_game_in_hand()
        {
            var host = new FakeHome { Suspended = new HomeSuspended("/g/a.sfc", "Granite Choir", "Super Nintendo", Clock) };
            var home = new HomeScreen(host);
            Assert.Equal(["Resume", "Game page", "Favourite", "Search", "Home"], home.HintsFor("continue").Select(h => h.Text));
            Assert.Equal(["Play", "Game page", "Favourite", "Search", "Home"], home.HintsFor("shelf").Select(h => h.Text));
            Assert.Equal(["Open", "Search", "Home"], home.HintsFor("places").Select(h => h.Text));
            home.Hint("page", "continue", null);
            home.Hint("favourite", "shelf", "/g/b.sfc");
            home.Hint(HomeScreen.Search, "places", null);
            home.ChooseContinue();
            home.ChooseGame("/g/b.sfc");
            home.ChoosePlace(HomeScreen.Cores);
            Assert.Equal(["page /g/a.sfc", "favourite /g/b.sfc", "open search", "resume", "play /g/b.sfc", "open cores"], host.Said);
        }

        [Theory]
        [InlineData(0.5, "Paused just now")]
        [InlineData(12, "Paused 12 minutes ago")]
        [InlineData(90, "Paused 1 hour ago")]
        [InlineData(300, "Paused 5 hours ago")]
        [InlineData(1500, "Paused yesterday")]
        [InlineData(4400, "Paused 3 days ago")]
        public void The_pause_is_said_in_words(double minutes, string said) => Assert.Equal(said, HomeScreen.PausedAgo(TimeSpan.FromMinutes(minutes)));

        private sealed class FakeGames : IGamesHost
        {
            public List<string> Said { get; } = new();
            public DateTime Now => Clock;
            public IReadOnlyList<GridGame> Games { get; } =
            [
                new(Facts("/g/granite.sfc", "Granite Choir", year: "1994", favourite: true, last: Clock.AddDays(-1)), "SNES", 9, 0.8),
                new(Facts("/g/aurora.sfc", "Aurora Drift", year: "1993", last: Clock.AddDays(-3)), "SNES", 2, 0.9),
                new(Facts("/g/fable.nes", "Fable of Tiles", "NES", "1990"), "NES", 0, null),
                new(Facts("/g/hollow.gb", "Hollow Comet", "Game Boy", "1991", last: Clock.AddHours(-2)), "GB", 4, 0.5),
            ];
            public IReadOnlyList<(string Key, string Label)> Consoles => [("SNES", "Super Nintendo"), ("NES", "NES"), ("GB", "Game Boy")];
            public IReadOnlyList<GridCollection> Collections => [new(7, "Weekend", new HashSet<string> { "/g/fable.nes", "/g/hollow.gb" })];
            public IReadOnlyList<string> Recent(IReadOnlyList<string> paths, int limit) => ["/g/hollow.gb", "/g/granite.sfc", "/g/aurora.sfc"];
            public bool CanStartConsole { get; set; } = true;
            public int Random(int count) => count - 1;
            public void Play(string path) => Said.Add("play " + path);
            public void OpenPage(string path) => Said.Add("page " + path);
            public void ToggleFavourite(string path) => Said.Add("favourite " + path);
            public void Home() => Said.Add("home");
            // The systems table's order, which is not the games' or the alphabet's; Master System has no game.
            public IReadOnlyList<ConsoleShelf> Table { get; set; } =
            [
                new("NES", "NES", "Nintendo Entertainment System", "Nintendo", 1985, 8), new("SMS", "Master System", "Master System", "Sega", 1986, 220),
                new("GB", "Game Boy", "Game Boy", "Nintendo", 1989, 95), new("SNES", "Super Nintendo", "Super Nintendo", "Nintendo", 1991, 280),
            ];
            public void ChooseConsole(string section) => Said.Add("consoles " + section);
            // The runtime's cores listing as it gives one, and the engine the player's settings choose for each console; Game Boy's chooses none.
            public string Listing { get; set; } = """
                [{"engine":"VenusRT (Rust)","name":"VenusRT","libretro":null},{"engine":"MoonRT (Rust)","name":"MoonRT","libretro":null},
                 {"engine":"Nintendo - SNES / SFC (Snes9x)","name":"Snes9x","libretro":{"corename":"Snes9x"}}]
                """;
            public Dictionary<string, string> Chosen { get; } = new() { ["SNES"] = "VenusRT (Rust)", ["NES"] = "MoonRT (Rust)" };
            public EngineShown? EngineOf(string shelf) => Chosen.TryGetValue(shelf, out string? chosen) ? RuntimeCatalog.Shown(chosen, RuntimeCatalog.EnginesIn(Listing)) : null;
        }

        [Fact]
        public void The_grid_steps_its_sections_sorts_and_searches()
        {
            var games = new GamesScreen(new FakeGames());
            Assert.Equal(["All", "Favourites", "Recent", "Super Nintendo", "NES", "Game Boy"], games.Sections.Select(s => s.Label));
            Assert.Equal(["Aurora Drift", "Fable of Tiles", "Granite Choir", "Hollow Comet"], games.Shown.Select(g => g.Facts.Title));
            games.StepSort(1);
            Assert.Equal("Last played", games.Sort);
            Assert.Equal(["Hollow Comet", "Granite Choir", "Aurora Drift", "Fable of Tiles"], games.Shown.Select(g => g.Facts.Title));
            games.StepSort(1);
            Assert.Equal(["Granite Choir", "Hollow Comet", "Aurora Drift", "Fable of Tiles"], games.Shown.Select(g => g.Facts.Title));
            games.StepSort(2);
            Assert.Equal("Rating", games.Sort);
            Assert.Equal("Aurora Drift", games.Shown[0].Facts.Title);
            games.StepSort(1);
            games.ChooseSection(GamesScreen.Recent);
            Assert.Equal(["Hollow Comet", "Granite Choir", "Aurora Drift"], games.Shown.Select(g => g.Facts.Title));
            games.ChooseSection(GamesScreen.ConsolePrefix + "SNES");
            Assert.Equal("2 games", games.CountLine);
            games.ChooseSection(GamesScreen.All);
            games.SetSearch("gran");
            Assert.Equal("1 of 4 games", games.CountLine);
            Assert.True(games.Back());
            Assert.Equal("4 games", games.CountLine);
        }

        [Fact]
        public void A_collection_takes_the_strip_s_place_and_B_leaves_it_then_goes_to_Choose_a_console()
        {
            var host = new FakeGames();
            var games = new GamesScreen(host);
            Assert.Equal([("", GamesScreen.EveryGame), (GamesScreen.Favourites, "Favourites"), ("7", "Weekend")], games.FilterRows);
            games.ShowCollection(7);
            Assert.Equal([(GamesScreen.CollectionPrefix + "7", "Weekend")], games.Sections);
            Assert.Equal(["Fable of Tiles", "Hollow Comet"], games.Shown.Select(g => g.Facts.Title));
            Assert.Equal("Section", games.HintsFor(false, false).Single(h => h.Button == "Shoulders").Text);
            games.Back();
            Assert.Null(games.Collection);
            games.Back();
            Assert.Equal(["consoles " + GamesScreen.All], host.Said);
        }

        [Fact]
        public void Filter_Favourites_narrows_each_section_to_its_own_and_is_cleared_as_a_collection_is()
        {
            var host = new FakeGames();
            var games = new GamesScreen(host);
            games.ChooseSection(GamesScreen.ConsolePrefix + "SNES");
            games.ChooseFilter(GamesScreen.Favourites);
            Assert.Equal(["Granite Choir"], games.Shown.Select(g => g.Facts.Title));
            Assert.Equal("1 of 2 games", games.CountLine);
            Assert.Equal("Favourites", games.ListRows(null).Single(r => r.Key == GamesScreen.FilterRow).Value);
            games.ChooseSection(GamesScreen.ConsolePrefix + "NES");
            Assert.Equal("1 game", games.CountLine);
            Assert.Null(games.FilterValue);
            games.ChooseSection(GamesScreen.ConsolePrefix + "SNES");
            Assert.Equal("1 of 2 games", games.CountLine);
            games.ChooseFilter("");
            Assert.Equal("2 games", games.CountLine);
            games.ChooseFilter(GamesScreen.Favourites);
            Assert.True(games.Back());
            Assert.Equal("2 games", games.CountLine);
            Assert.Empty(host.Said);
            games.ChooseSection(GamesScreen.Favourites);
            Assert.DoesNotContain(GamesScreen.Favourites, games.FilterRows.Select(f => f.Key));
        }

        [Fact]
        public void The_List_card_holds_the_design_s_rows_and_its_pane_says_what_each_does()
        {
            var host = new FakeGames();
            var games = new GamesScreen(host);
            Assert.Equal(["Search", "Sort", "Filter", "Jump to", "Random game", "Start Console", "Collections"], games.ListRows("/g/granite.sfc").Select(r => r.Label));
            Assert.Equal("G", games.ListRows("/g/granite.sfc").Single(r => r.Key == GamesScreen.JumpRow).Value);
            Assert.Equal(("Sorted by name, A to Z.", "Also: Last played · Most played · Year · Rating"), games.ListPane(GamesScreen.SortRow)[0]);
            Assert.Equal("/g/hollow.gb", games.JumpFrom("/g/granite.sfc", 1));
            Assert.Equal("/g/aurora.sfc", games.JumpFrom("/g/hollow.gb", 1));
            Assert.Equal("/g/hollow.gb", games.RandomGame());
            host.CanStartConsole = false;
            Assert.DoesNotContain("Start Console", games.ListRows(null).Select(r => r.Label));
        }

        [Fact]
        public void The_grid_s_help_bar_follows_the_keyboard_and_names_B_for_the_consoles_or_a_search()
        {
            var host = new FakeGames();
            var games = new GamesScreen(host);
            Assert.Equal(["Type", "Delete", "Space", "Done", "Move to grid", "Home"], games.HintsFor(true, true).Select(h => h.Text));
            Assert.Equal(["Play", "Game page", "Favourite", "Console", "List", "Consoles", "Home"], games.HintsFor(false, false).Select(h => h.Text));
            Assert.Contains("Clear search", games.HintsFor(false, true).Select(h => h.Text));
            games.Hint("page", "/g/fable.nes");
            games.Hint("favourite", "/g/fable.nes");
            games.Hint("home", null);
            Assert.Equal(["page /g/fable.nes", "favourite /g/fable.nes", "home"], host.Said);
        }

        // Choose a console (EmuSen_Interface.md §24): the three places, then each console of the table with a game, in the table's order.
        [Fact]
        public void Choose_a_console_lists_the_table_s_consoles_that_have_games_in_its_order()
        {
            var host = new FakeGames();
            var consoles = new ConsolesScreen(host);
            Assert.Equal([ConsolesScreen.All, ConsolesScreen.Favourites, ConsolesScreen.Recent], consoles.Places.Select(p => p.Key));
            Assert.Equal(["4 games", "Your hearts, every console", "Last: Hollow Comet"], consoles.Places.Select(p => p.Status));
            Assert.Equal(["NES", "GB", "SNES"], consoles.Consoles.Select(c => c.Key));
            Assert.Equal([8.0, 95.0, 280.0], consoles.Consoles.Select(c => c.Hue!.Value));
            ConsoleChoice snes = consoles.Consoles.Last();
            Assert.Equal(("Super Nintendo", "2 games", "Nintendo · 1991 · 2 games · runs on Venus, an EmuSen core"), (snes.Title, snes.Status, snes.Facts));
            Assert.Equal(["/g/granite.sfc", "/g/aurora.sfc"], snes.Covers.Select(g => g.Facts.Path));
            Assert.Equal("Nintendo Entertainment System", consoles.Consoles[0].Title);
            Assert.Equal("3 consoles · 4 games", consoles.CountLine);
            host.Table = [.. host.Table.Reverse()];
            Assert.Equal(["SNES", "GB", "NES"], consoles.Consoles.Select(c => c.Key));
        }

        // Each console's engine line and facts read the player's choice afresh: an EmuSen core, a libretro core, and none (EmuSen_Interface.md §24.6).
        [Fact]
        public void Each_console_says_the_engine_it_runs_on_and_follows_a_change()
        {
            var host = new FakeGames();
            var consoles = new ConsolesScreen(host);
            ConsoleChoice Of(string key) => consoles.Consoles.Single(c => c.Key == key);
            Assert.Equal(("Venus", false), (Of("SNES").Engine!.Name, Of("SNES").Engine!.Libretro));
            Assert.NotNull(Of("SNES").Engine!.Mark!.Stroke);
            Assert.EndsWith(" · runs on Moon, an EmuSen core", Of("NES").Facts);
            Assert.Null(Of("GB").Engine);
            Assert.Equal("Nintendo · 1989 · 1 game", Of("GB").Facts);
            host.Chosen["SNES"] = "Nintendo - SNES / SFC (Snes9x)";
            Assert.Equal(("Snes9x", true), (Of("SNES").Engine!.Name, Of("SNES").Engine!.Libretro));
            Assert.Same(EngineMarks.Libretro, Of("SNES").Engine!.Mark);
            Assert.Equal("Nintendo · 1991 · 2 games · runs on Snes9x, a libretro core", Of("SNES").Facts);
            host.Chosen["SNES"] = "VenusRT (Rust)";
            Assert.EndsWith("runs on Venus, an EmuSen core", Of("SNES").Facts);
            Assert.Equal("", ConsolesScreen.RunsOn(null));
            Assert.Equal((ConsolesScreen.LibretroTag, ConsolesScreen.OwnTag), (ConsolesScreen.TagOf(new("Snes9x", true)), ConsolesScreen.TagOf(new("Venus", false))));
        }

        [Fact]
        public void A_choice_opens_its_section_and_X_picks_from_the_focused_console()
        {
            var host = new FakeGames();
            var consoles = new ConsolesScreen(host);
            Assert.Equal(GamesScreen.ConsolePrefix + "SNES", ConsolesScreen.SectionOf("SNES"));
            Assert.Equal(GamesScreen.All, ConsolesScreen.SectionOf(ConsolesScreen.All));
            Assert.Equal("SNES", ConsolesScreen.KeyOf(GamesScreen.ConsolePrefix + "SNES"));
            Assert.Equal(ConsolesScreen.Recent, ConsolesScreen.KeyOf(GamesScreen.Recent));
            Assert.Null(ConsolesScreen.KeyOf(GamesScreen.CollectionPrefix + "7"));
            Assert.Equal("/g/aurora.sfc", consoles.RandomGame("SNES"));
            Assert.Equal("/g/fable.nes", consoles.RandomGame("NES"));
            Assert.Equal("/g/hollow.gb", consoles.RandomGame(ConsolesScreen.All));
            Assert.Null(consoles.RandomGame("SMS"));
            Assert.Equal(["Open", "Back", "Favourites", "Random game"], ConsolesScreen.Hints.Select(h => h.Text));
        }
    }
}
