using System;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // A crop kept with one game, as Settings' one description holds it for the Pause Card's Display and the game's page alike - see EmuSen_Interface.md §20 and EmuSen_Settings_Reference.md §4.100.
    [Collection(TestCollections.ProcessGlobals)]
    public class GameCropTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(GameCropTests).GetTypeInfo().Assembly);

        private const string Game = "/r/Mario Kart 64 (Europe).z64";

        private static (SettingsHostFake Host, SettingsScreen Screen) Open()
        {
            var host = new SettingsHostFake { RunningConsole = "N64", RunningGame = Game };
            var screen = new SettingsScreen(host);
            screen.OpenGameCrop(Game, "Mario Kart 64 (Europe)");
            return (host, screen);
        }

        private static string[] Edges(SettingsScreen s) => ConsoleCrop.Edges.Select(e => s.Rows.Single(r => r.Key == "gamecrop:" + e.Key).Status!).ToArray();

        // Until it has one of its own the edges show the console's; an edge stepped or typed makes the crop the game's own, a word typed changes nothing, and a number past the most is held to it.
        [Fact]
        public Task The_page_starts_from_the_console_s_crop_and_an_edge_changed_makes_the_crop_the_game_s_own() => Session.Dispatch(() =>
        {
            (SettingsHostFake host, SettingsScreen s) = Open();
            Assert.Equal("Mario Kart 64 (Europe)", s.Trail[^1]);
            Assert.Equal(["This game's crop", "Left", "Right", "Top", "Bottom"], s.Rows.Select(r => r.Label));
            Assert.Equal(SettingsScreen.Off, s.Rows[0].Status);
            Assert.Equal(["3.3%", "3.3%", "3.3%", "3.3%"], Edges(s));
            Assert.Equal("This game is drawn with its console's setting.", s.PaneOf("gamecrop")!.Lines[0].Text);
            Assert.Null(host.CropOf(Game));

            s.Step("gamecrop:CropTop", 1);
            Assert.Equal(new GameCrop(3.3, 3.3, 3.4, 3.3), host.CropOf(Game));
            host.Answer = "8,7";
            s.Choose("gamecrop:CropTop");
            host.Answer = "wide";
            s.Choose("gamecrop:CropLeft");
            host.Answer = "40";
            s.Choose("gamecrop:CropRight");
            Assert.Equal(new GameCrop(3.3, 25, 8.7, 3.3), host.CropOf(Game));
            Assert.Equal(SettingsScreen.On, s.Rows[0].Status);
            Assert.Equal("This game has a crop of its own.", s.PaneOf("gamecrop")!.Lines[0].Text);

            // Off goes back to the console's, and the edges show it again.
            s.Step("gamecrop", 1);
            Assert.Null(host.CropOf(Game));
            Assert.Equal(["3.3%", "3.3%", "3.3%", "3.3%"], Edges(s));
        }, default);

        // A game with a crop of its own opens on it, and On from Off starts from the console's.
        [Fact]
        public Task A_game_with_a_crop_of_its_own_opens_on_it_and_on_starts_from_the_console_s() => Session.Dispatch(() =>
        {
            (SettingsHostFake host, SettingsScreen s) = Open();
            host.SetCrop(Game, new GameCrop(2.4, 2.5, 3.5, 3.5));
            s.Read();
            Assert.Equal(["2.4%", "2.5%", "3.5%", "3.5%"], Edges(s));
            Assert.Equal(SettingsScreen.On, s.Rows[0].Status);
            s.Step("gamecrop", 1);
            s.Step("gamecrop", 1);
            Assert.Equal(new GameCrop(3.3, 3.3, 3.3, 3.3), host.CropOf(Game));
        }, default);

        // The Pause Card's Display offers it for a console with an overscan, and A on it opens the four edges as a page.
        [Fact]
        public Task Display_offers_it_for_a_game_whose_console_has_an_overscan() => Session.Dispatch(() =>
        {
            var host = new SettingsHostFake { RunningConsole = "N64", RunningGame = Game };
            var s = new SettingsScreen(host);
            s.OpenGame(SettingsScreen.GameDisplay);
            Assert.Equal(["Picture shape", "CRT shader", "Crop overscan", "This game's crop", "Full screen"], s.Rows.Select(r => r.Label));
            s.Choose("gamecrop");
            Assert.Equal("This game's crop", s.Trail[^1]);
            Assert.NotNull(host.CropOf(Game));

            var handheld = new SettingsScreen(new SettingsHostFake { RunningConsole = "Game Boy", RunningGame = "/r/a.gb" });
            handheld.OpenGame(SettingsScreen.GameDisplay);
            Assert.DoesNotContain("This game's crop", handheld.Rows.Select(r => r.Label));
        }, default);

        [Fact]
        public void It_is_offered_for_a_game_of_a_console_that_has_an_overscan()
        {
            MethodInfo offered = typeof(MainWindow).GetMethod("CropOffered", BindingFlags.Static | BindingFlags.NonPublic)!;
            bool Offered(string path) => (bool)offered.Invoke(null, new object[] { path })!;
            Assert.True(Offered("/r/a.sfc"));
            Assert.True(Offered("/r/a.z64"));
            Assert.True(Offered("/r/a.nes"));
            Assert.False(Offered("/r/a.gb"));
            Assert.False(Offered("/r/a.gbc"));
            Assert.False(Offered("/r/a.txt"));
        }

        private static GameRecords Records(MainWindow window) => (GameRecords)typeof(MainWindow).GetField("_records", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;

        // On the desktop the game's page opens Settings at the game's own crop, and what is changed is in the library for that game alone.
        [Fact]
        public Task On_the_desktop_the_game_s_page_opens_settings_at_its_crop_and_the_crop_is_stored_for_that_game() => Session.Dispatch(() =>
        {
            using ThemedSession s = DesktopLibrary.Desktop();
            var list = DesktopLibrary.List(s.Window);
            string game = DesktopLibrary.SnesPath(s, 1), other = DesktopLibrary.SnesPath(s, 2);
            list.Select(list.Models.Single(e => e.FullPath == game));
            s.Settle();
            s.Window.ShowGamePage(game, GamePage.Options);
            s.Window.GamePageShown!.ChooseRow("crop");
            s.Settle();

            SettingsScreen settings = Assert.IsType<SettingsScreen>(s.Window.SettingsShown);
            Assert.Equal(ThemedSession.SnesGames[1], settings.Trail[^1]);
            settings.Step("gamecrop", -1);
            settings.Step("gamecrop:CropTop", 1);
            GameCrop own = Assert.IsType<GameCrop>(Records(s.Window).Crop(game));
            Assert.True(own.Top > own.Bottom, $"{own}");
            Assert.Null(Records(s.Window).Crop(other));
            s.Window.CloseSettings();
            s.Settle();
        }, default);
    }
}
