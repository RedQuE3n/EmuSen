using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Styling;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Theme;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Choose a console between Home and the grid, and the night sky under the three: the player's presses on the real window - see EmuSen_Interface.md §24.
    [Collection(TestCollections.ProcessGlobals)]
    public class ChooseConsoleTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ChooseConsoleTests).GetTypeInfo().Assembly);

        private static ThemedSession Own(Action<AppSettings>? more = null) =>
            new(settings: a => { a.LibraryStyle = AppSettings.LibraryStyleMistress; more?.Invoke(a); });

        // Home's Games place, as the player reaches it.
        private static PlanetLayout Open(ThemedSession s)
        {
            Assert.IsType<HubLayout>(s.Window.HomeLayout).FocusOn(HubRow.Places, HomeScreen.Games);
            s.Pad.A();
            s.Settle();
            return Assert.IsType<PlanetLayout>(s.Window.ConsolesLayout);
        }

        private static string ShelfOf(ThemedSession s, string extension) =>
            s.Window.ConsolesShown!.Host.Games.First(g => g.Facts.Path.EndsWith(extension, StringComparison.Ordinal)).Shelf;

        [Fact]
        public Task Home_s_Games_place_opens_Choose_a_console_with_the_table_s_consoles_that_have_games() => Session.Dispatch(() =>
        {
            using var s = Own();
            PlanetLayout consoles = Open(s);
            Assert.Null(s.Window.GamesLayout);
            Assert.Equal([ConsolesScreen.All, ConsolesScreen.Favourites, ConsolesScreen.Recent], consoles.Tiles.Select(t => t.Key));
            string[] withGames = [".sfc", ".nes", ".gb"];
            string[] expected = RuntimeCatalog.Shelves.Select(sh => sh.Name).Where(n => withGames.Any(x => ShelfOf(s, x) == n)).ToArray();
            Assert.Equal(3, expected.Length);
            Assert.Equal(expected, consoles.Planets.Select(p => p.Key));
            Assert.True(RuntimeCatalog.Shelves.Count > consoles.Planets.Count, "the table holds consoles with no game, which are not listed");
            Assert.All(consoles.Planets, p => Assert.Equal(StartupSplash.HueOf(RuntimeCatalog.ShelfByName(p.Key)!.System), p.Hue));
            Assert.Equal(["Open", "Back", "Favourites", "Random game"], consoles.Hints.Select(h => h.Label));
            s.Pad.B();
            s.Settle();
            Assert.NotNull(s.Window.HomeLayout);
        }, default);

        // Each console drawn by its maker, and the engine the player's settings choose for it on its tile and in its facts (EmuSen_Interface.md §24.6).
        [Fact]
        public Task Each_console_is_drawn_by_its_maker_and_says_the_engine_it_runs_on() => Session.Dispatch(() =>
        {
            using var s = Own();
            PlanetLayout consoles = Open(s);
            Assert.Equal(3, consoles.Planets.Select(p => p.Engine?.Name).Distinct().Count());
            foreach (string extension in new[] { ".sfc", ".nes", ".gb" })
            {
                string shelf = ShelfOf(s, extension);
                PlanetItem item = consoles.Planets.Single(p => p.Key == shelf);
                LibraryShelf table = RuntimeCatalog.ShelfByName(shelf)!;
                Assert.Equal(StartupSplash.BodyOf(table.System), item.Body);
                Assert.NotNull(item.Body!.Palette);
                EngineShown shown = Assert.IsType<EngineShown>(RuntimeCatalog.EngineOf(table.Core.Console));
                Assert.False(shown.Libretro);
                string engine = shown.Name;
                Assert.True(shown.Mark is { Stroke.Length: > 0 } && shown.Mark != EngineMarks.Planet, $"{engine} has its own symbol");
                Assert.Equal(new PlanetEngine(engine, ConsolesScreen.OwnTag, true) { Mark = shown.Mark }, item.Engine);
                Assert.EndsWith($" · runs on {engine}, an EmuSen core", item.Facts);
                Control cell = s.Window.GetVisualDescendants().OfType<Control>().First(c => c.Name == "PlanetCell." + shelf);
                Assert.Contains(cell.GetVisualDescendants().OfType<TextBlock>(), t => t.Text == engine.ToUpperInvariant());
                Assert.Contains(cell.GetVisualDescendants().OfType<TextBlock>(), t => t.Text == "EMUSEN");
            }
            // The splash draws each of these consoles as its tile's body.
            var splash = StartupSplash.Bodies(RuntimeCatalog.Systems);
            foreach (PlanetItem item in consoles.Planets)
            {
                SystemFacts system = RuntimeCatalog.Systems.First(f => f.Id == RuntimeCatalog.ShelfByName(item.Key)!.System);
                Assert.Equal(item.Body, splash.Single(b => b.Name == (system.DisplayName is { Length: > 0 } shown ? shown : system.Console)).Body);
            }
            consoles.FocusOn(ShelfOf(s, ".sfc"));
            s.Settle();
            Assert.EndsWith(ConsolesScreen.RunsOn(RuntimeCatalog.EngineOf(RuntimeCatalog.ShelfByName(ShelfOf(s, ".sfc"))!.Core.Console)), ((TextBlock)s.Window.GetVisualDescendants().OfType<Control>().First(c => c.Name == "PlanetFocusFacts")).Text);
        }, default);

        [Fact]
        public Task A_opens_the_console_s_grid_the_shoulders_step_and_name_the_neighbours_and_B_returns_to_it() => Session.Dispatch(() =>
        {
            using var s = Own();
            PlanetLayout consoles = Open(s);
            string snes = ShelfOf(s, ".sfc");
            consoles.FocusOn(snes);
            s.Pad.A();
            s.Settle();
            GalleryLayout grid = Assert.IsType<GalleryLayout>(s.Window.GamesLayout);
            Assert.Equal(GamesScreen.ConsolePrefix + snes, s.Window.GamesShown!.Section);
            Assert.True(grid.Night);
            Assert.Equal(StartupSplash.HueOf(RuntimeCatalog.ShelfByName(snes)!.System), grid.Hue);
            var sections = grid.Sections.ToList();
            int at = sections.FindIndex(x => x.Key == GamesScreen.ConsolePrefix + snes);
            Assert.Equal((sections[at - 1].Label, sections[(at + 1) % sections.Count].Label), grid.Neighbours);
            string next = sections[(at + 1) % sections.Count].Key;
            s.Pad.R1();
            s.Settle();
            Assert.Equal(next, s.Window.GamesShown!.Section);
            Assert.Equal(sections[at].Label, grid.Neighbours!.Value.Previous);
            // B returns with the focus on the console the grid shows now, not the one it was opened on.
            s.Pad.B();
            s.Settle();
            PlanetLayout back = Assert.IsType<PlanetLayout>(s.Window.ConsolesLayout);
            Assert.Equal(ConsolesScreen.KeyOf(next), back.Focused);
            back.FocusOn(snes);
            s.Pad.A();
            s.Settle();
            s.Pad.B();
            s.Settle();
            Assert.Equal(snes, s.Window.ConsolesLayout!.Focused);
            Assert.Null(s.Window.GamesLayout);
        }, default);

        [Fact]
        public Task X_picks_a_game_from_the_focused_console_and_on_All_games_from_every_game() => Session.Dispatch(() =>
        {
            using var s = Own();
            PlanetLayout consoles = Open(s);
            string nes = ShelfOf(s, ".nes");
            consoles.FocusOn(nes);
            s.Pad.X();
            s.Settle();
            GalleryLayout grid = Assert.IsType<GalleryLayout>(s.Window.GamesLayout);
            Assert.Equal(GamesScreen.ConsolePrefix + nes, s.Window.GamesShown!.Section);
            Assert.EndsWith(".nes", grid.Focused.Key);
            s.Pad.B();
            s.Settle();
            consoles = Assert.IsType<PlanetLayout>(s.Window.ConsolesLayout);
            Assert.Equal(nes, consoles.Focused);
            consoles.FocusOn(ConsolesScreen.All);
            s.Pad.X();
            s.Settle();
            Assert.Equal(GamesScreen.All, s.Window.GamesShown!.Section);
            Assert.NotNull(s.Window.GamesLayout!.Focused.Key);
        }, default);

        [Fact]
        public Task The_sky_s_glow_follows_the_focus_and_holds_still_under_reduced_motion() => Session.Dispatch(() =>
        {
            using var s = Own();
            PlanetLayout consoles = Open(s);
            Assert.True(consoles.Night);
            foreach (PlanetItem planet in consoles.Planets)
            {
                consoles.FocusOn(planet.Key);
                Assert.Equal(planet.Hue, consoles.Sky.GlowHue);
            }
            LunaMotion.Reduced = true;
            try
            {
                PlanetItem first = consoles.Planets[0];
                s.Pad.Up();
                s.Pad.Down();
                consoles.FocusOn(first.Key);
                Assert.True(consoles.Sky.Settled);
                Assert.Equal(first.Hue!.Value, consoles.Sky.ShownHue!.Value, 3);
                consoles.FocusOn(consoles.Planets[^1].Key);
                Assert.True(consoles.Sky.Settled);
                Assert.Equal(consoles.Planets[^1].Hue!.Value, consoles.Sky.ShownHue!.Value, 3);
            }
            finally { LunaMotion.Reduced = false; }
            s.Pad.B();
            s.Settle();
            HubLayout home = Assert.IsType<HubLayout>(s.Window.HomeLayout);
            Assert.True(home.Night);
            Assert.Equal(StartupSplash.Title, home.Mark);
            if (home.Shelf.Count > 0)
            {
                home.FocusOn(HubRow.Shelf, home.Shelf[0].Key);
                Assert.Equal(home.Shelf[0].Hue, home.Sky.GlowHue);
            }
        }, default);

        [Fact]
        public Task The_light_variant_keeps_the_day_s_surfaces() => Session.Dispatch(() =>
        {
            using var s = Own();
            s.Window.RequestedThemeVariant = ThemeVariant.Light;
            s.Settle();
            PlanetLayout consoles = Open(s);
            Assert.False(consoles.Night);
            Assert.DoesNotContain(consoles.GetVisualDescendants(), c => c is LunaSky);
        }, default);
    }
}
