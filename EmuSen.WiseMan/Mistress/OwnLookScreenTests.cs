using System;
using System.Collections.Generic;
using System.Linq;
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
            public int? MissingMedia { get; set; } = 41;
            public int? CoresWaiting { get; set; } = 1;
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
        }

        [Fact]
        public void The_desktop_has_eight_places_and_Game_Mode_quits_rather_than_exits()
        {
            var host = new FakeHome { Desktop = true, CoresWaiting = 0, Battery = "80%" };
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
        public void A_collection_takes_the_strip_s_place_and_B_leaves_it_then_goes_home()
        {
            var host = new FakeGames();
            var games = new GamesScreen(host);
            Assert.Equal([("", GamesScreen.EveryGame), ("7", "Weekend")], games.FilterRows);
            games.ShowCollection(7);
            Assert.Equal([(GamesScreen.CollectionPrefix + "7", "Weekend")], games.Sections);
            Assert.Equal(["Fable of Tiles", "Hollow Comet"], games.Shown.Select(g => g.Facts.Title));
            Assert.Equal("Section", games.HintsFor(false, false).Single(h => h.Button == "Shoulders").Text);
            games.Back();
            Assert.Null(games.Collection);
            games.Back();
            Assert.Equal(["home"], host.Said);
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
        public void The_grid_s_help_bar_follows_the_keyboard_and_names_B_only_when_it_clears_a_search()
        {
            var host = new FakeGames();
            var games = new GamesScreen(host);
            Assert.Equal(["Type", "Delete", "Space", "Done", "Move to grid", "Home"], games.HintsFor(true, true).Select(h => h.Text));
            Assert.Equal(["Play", "Game page", "Favourite", "Console", "List", "Home"], games.HintsFor(false, false).Select(h => h.Text));
            Assert.Contains("Clear search", games.HintsFor(false, true).Select(h => h.Text));
            games.Hint("page", "/g/fable.nes");
            games.Hint("favourite", "/g/fable.nes");
            games.Hint("home", null);
            Assert.Equal(["page /g/fable.nes", "favourite /g/fable.nes", "home"], host.Said);
        }
    }
}
