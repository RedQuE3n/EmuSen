using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Pass 11: ES-DE's launch screen between the resume question and the game - see EmuSen_BigPicture.md §33.
    [Collection(TestCollections.ProcessGlobals)]
    public class LaunchScreenTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(LaunchScreenTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly ITestOutputHelper _out;

        public LaunchScreenTests(ITestOutputHelper output) => _out = output;

        internal const string Aurora = "Aurora Drift (Synthetic)", Brass = "Brass Lantern (Synthetic)", Cobalt = "Cobalt Harbor (Synthetic)";

        internal static void Stop(MainWindow w) => typeof(MainWindow).GetMethod("StopEmulationThread", Hidden)!.Invoke(w, null);

        internal static string? Running(MainWindow w) => (string?)typeof(MainWindow).GetField("_currentRomPath", Hidden)!.GetValue(w);

        internal static LaunchScreen? Screen(ThemedSession s) => (LaunchScreen?)typeof(MainWindow).GetProperty("LaunchScreenShown", Hidden)!.GetValue(s.Window);

        internal static LaunchScreen Host(ThemedSession s) => Avalonia.VisualTree.VisualExtensions.GetVisualDescendants(s.Window).OfType<LaunchScreen>().Single();

        private static bool GameShown(ThemedSession s) => s.Window.GetControl<Control>("GameFrame").IsVisible;

        private static string Rom(ThemedSession s, string game) => Path.Combine(s.RomDirectory, game + ".sfc");

        // An ES-DE media folder with a marquee for Aurora, a cover for Aurora and Brass, and nothing for Cobalt.
        internal static string Media(string root)
        {
            string media = Path.Combine(root, "downloaded_media");
            void Put(string kind, string game, int w, int h, Color left, Color right)
            {
                string path = Path.Combine(media, "snes", kind, game + ".png");
                Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                File.Copy(SceneAssets.Halves($"launch-{kind}-{w}x{h}", w, h, left, right), path, overwrite: true);
            }
            Put("marquees", Aurora, 800, 300, Colors.MediumPurple, Colors.Gold);
            Put("covers", Aurora, 300, 400, Colors.Crimson, Colors.Orange);
            Put("covers", Brass, 300, 400, Colors.SeaGreen, Colors.Khaki);
            return media;
        }

        internal static ThemedSession Open(string duration, double width = 1280, double height = 800, Action<AppSettings>? more = null)
        {
            string? media = null;
            return new ThemedSession(width, height, settings: a =>
            {
                a.BigPictureInterface.LaunchScreenDuration = duration;
                a.EsdeMediaDirectory = media;
                more?.Invoke(a);
            }, roms: r => media = Media(Path.GetDirectoryName(r)!));
        }

        // Each duration holds the game back to within one 16 ms poll of its end, and not a poll longer.
        [Theory]
        [InlineData(BigPictureInterface.LaunchNormal, 3000)]
        [InlineData(BigPictureInterface.LaunchBrief, 1700)]
        [InlineData(BigPictureInterface.LaunchLong, 4500)]
        [InlineData(BigPictureInterface.LaunchPopup, 1700)]
        public Task Each_duration_starts_the_game_at_its_end(string duration, int ms) => Session.Dispatch(() =>
        {
            using var s = Open(duration);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Sounds.Clear();
            TimeSpan pressed = s.Now;
            s.Pad.A();
            Assert.Equal(["launch"], s.Sounds);
            LaunchScreen screen = Assert.IsType<LaunchScreen>(Screen(s));
            Assert.Equal(duration == BigPictureInterface.LaunchPopup, screen.Popup);
            Assert.Equal(pressed + TimeSpan.FromMilliseconds(ms), screen.Due);
            Assert.Null(Running(s.Window));
            Assert.False(GameShown(s));

            s.Run(ms - 16);
            Assert.Null(Running(s.Window));
            Assert.NotNull(Screen(s));
            Assert.True(s.Shown);

            s.Run(16);
            Assert.Equal(Rom(s, Aurora), Running(s.Window));
            Assert.True(GameShown(s));
            Assert.Null(Screen(s));
            Assert.False(Host(s).IsVisible);
            Stop(s.Window);
        }, default);

        [Fact]
        public Task Disabled_starts_the_game_at_once_with_no_screen() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchDisabled);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            Assert.Equal(Rom(s, Aurora), Running(s.Window));
            Assert.True(GameShown(s));
            Assert.Null(Screen(s));
            Assert.Empty(s.Window.GetVisualDescendantsOf<LaunchScreen>());
            Stop(s.Window);
        }, default);

        // The setting is stored as ES-DE spells it, and Normal is the default a new appsettings.json gets.
        [Fact]
        public void Normal_is_the_default_and_the_values_are_ES_DE_s()
        {
            Assert.Equal("normal", new BigPictureInterface().LaunchScreenDuration);
            Assert.Equal(["normal", "brief", "long", "popup", "disabled"], InterfaceSettingsPane.LaunchScreenChoices.Select(c => c.Value));
            Assert.Equal(3000, LaunchScreen.DurationOf("normal").TotalMilliseconds);
            Assert.Equal(3000, LaunchScreen.DurationOf("anything else").TotalMilliseconds);
            Assert.Equal(0, LaunchScreen.DurationOf("disabled").TotalMilliseconds);
        }

        [Fact]
        public Task The_screen_shows_the_game_s_name_its_system_and_its_marquee_else_its_cover_else_no_picture() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal);
            ThemedLibraryPadTests.Enter(s, "snes");
            string media = s.Settings.EsdeMediaDirectory!;
            var expected = new (string Game, string? Art)[]
            {
                (Aurora, Path.Combine(media, "snes", "marquees", Aurora + ".png")),
                (Brass, Path.Combine(media, "snes", "covers", Brass + ".png")),
                (Cobalt, null),
            };
            foreach ((string game, string? art) in expected)
            {
                for (int guard = 0; guard < 10 && s.Game != game; guard++) s.Pad.Down();
                Assert.Equal((game, "gamelist"), (s.Game, s.View));
                s.Pad.A();
                LaunchScreen screen = Assert.IsType<LaunchScreen>(Screen(s));
                Assert.Equal(game, screen.GameName);
                Assert.Equal("Super Nintendo", screen.SystemName);
                Assert.Equal(art, screen.ArtPath);
                double h = 800;
                Assert.Equal(art is null ? 256.0 : 437.0, screen.CardBounds.Height, 3);
                s.Run(3000);
                Assert.Equal(Rom(s, game), Running(s.Window));
                ThemedLibraryFlowTests.Choose(s, "Close Game");
                Assert.Null(Running(s.Window));
                if (s.View != "gamelist") s.Pad.A();
                Assert.Equal("gamelist", s.View);
            }
        }, default);

        // The metadata editor's name is the one shown, as the list shows it.
        [Fact]
        public Task An_edited_name_is_the_name_shown() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal);
            ThemedCollectionsTests.Records(s).SaveEdits(Rom(s, Aurora), new Dictionary<string, string?> { [EmuSen.Mistress.Library.GameMetadata.Name] = "Aurora Drift Deluxe" }, DateTime.Now);
            ThemedCollectionsTests.Refresh(s);
            ThemedLibraryPadTests.Enter(s, "snes");
            Assert.Equal("Aurora Drift Deluxe", s.Game);
            s.Pad.A();
            Assert.Equal("Aurora Drift Deluxe", Screen(s)!.GameName);
            s.Run(3000);
            Stop(s.Window);
        }, default);

        // ES-DE takes no input while its screen is up (§33): a direction, B and the menu chord change nothing and the game still starts on time.
        [Fact]
        public Task The_pad_does_nothing_while_the_screen_is_up() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            s.Run(500);
            s.Pad.Down();
            s.Pad.B();
            s.Pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            s.Pad.Start();
            Assert.False(s.Window.GetControl<Control>("PadMenuPanel").IsVisible);
            Assert.Equal(Aurora, s.Game);
            Assert.Equal("gamelist", s.View);
            Assert.Null(Running(s.Window));
            s.Run(2500);
            Assert.Equal(Rom(s, Aurora), Running(s.Window));
            Assert.False(s.Window.GetControl<Control>("PadMenuPanel").IsVisible);
            Stop(s.Window);
        }, default);

        // §21.3's risk: the pad menu's way back to a suspended game, and East from the system view, go straight back to it.
        [Fact]
        public Task A_resume_from_the_pad_menu_or_East_is_not_delayed() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchLong);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            s.Run(4500);
            Assert.True(GameShown(s));

            ThemedLibraryFlowTests.Choose(s, "Game Library");
            Assert.True(s.Shown);
            ThemedCollectionsTests.Choose(s, "Back to");
            Assert.True(GameShown(s));
            Assert.False(s.Window.IsPaused);
            Assert.Null(Screen(s));

            ThemedLibraryFlowTests.Choose(s, "Game Library");
            s.Pad.B();
            Assert.Equal("system", s.View);
            s.Pad.B();
            Assert.True(GameShown(s));
            Assert.Null(Screen(s));
            Stop(s.Window);
        }, default);

        private static SheetLayer Sheets(ThemedSession s) => s.Window.GetControl<SheetLayer>("Sheets");

        // Q33: the question first, then the screen after either answer; a question cancelled shows nothing and starts nothing.
        [Fact]
        public Task The_screen_follows_the_resume_question_and_a_cancelled_question_shows_nothing() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal, more: a =>
            {
                a.ResumeOnLaunch = AppSettings.ResumeAsk;
                a.StateDirectory = Path.Combine(Path.GetTempPath(), "EmuSenLaunchStates", Guid.NewGuid().ToString("N"));
            });
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            Assert.NotNull(Screen(s));
            s.Run(3000);
            ThemedLibraryFlowTests.Choose(s, "Close Game");

            s.Pad.A();
            Assert.IsType<ResumeWindow>(Sheets(s).Current);
            Assert.Null(Screen(s));
            s.Run(3500);
            Assert.Null(Running(s.Window));
            s.Pad.B();
            Assert.False(Sheets(s).IsPresenting);
            Assert.Null(Screen(s));
            s.Run(3500);
            Assert.Null(Running(s.Window));
            Assert.True(s.Shown);

            s.Pad.A();
            Assert.IsType<ResumeWindow>(Sheets(s).Current);
            TimeSpan answered = s.Now;
            s.Pad.A();
            Assert.False(Sheets(s).IsPresenting);
            LaunchScreen screen = Assert.IsType<LaunchScreen>(Screen(s));
            Assert.Equal(answered + TimeSpan.FromMilliseconds(3000), screen.Due);
            s.Run(3000);
            Assert.Equal(Rom(s, Aurora), Running(s.Window));
            Stop(s.Window);
        }, default);

        // A game that cannot load: the screen goes at its end, the library stays, the reason shows in the notice and is in the error log.
        [Fact]
        public Task A_game_that_fails_to_load_closes_the_screen_shows_the_error_and_logs_it() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchBrief);
            ThemedLibraryPadTests.Enter(s, "nes");
            string game = s.Game!;
            s.Pad.A();
            Assert.NotNull(Screen(s));
            s.Run(1700);
            Assert.Null(Screen(s));
            Assert.Null(Running(s.Window));
            Assert.False(GameShown(s));
            Assert.True(s.Shown);
            string? notice = s.Window.GetControl<NoticeLayer>("PadNotice").Current;
            Assert.NotNull(notice);
            Assert.StartsWith("Failed to load", notice);
            string log = File.ReadAllText(ErrorLog.PathFor(DateTime.Now));
            Assert.Contains($"[launch] Failed to load {game}", log);
            s.Pad.Down();
            Assert.NotEqual(game, s.Game);
        }, default);

        // Every route big picture starts a game by: a collection, the random entry and a folder link.
        [Fact]
        public Task A_collection_the_random_entry_and_a_folder_link_all_show_it() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: a =>
            {
                a.BigPictureInterface.LaunchScreenDuration = BigPictureInterface.LaunchBrief;
                ThemedCollectionsTests.AllAuto(a);
            }, roms: ThemedFoldersTests.Library);

            ThemedCollectionsTests.Enter(s, "all");
            string first = s.Game!;
            s.Pad.A();
            Assert.Equal(first, Screen(s)?.GameName);
            s.Run(1700);
            Assert.NotNull(Running(s.Window));
            ThemedLibraryFlowTests.Choose(s, "Close Game");

            s.Pad.A();
            s.Pad.Press(EmuSen.Mistress.Input.UiButton.Random);
            s.Settle();
            string random = s.Game!;
            s.Pad.A();
            Assert.Equal(random, Screen(s)?.GameName);
            s.Run(1700);
            ThemedLibraryFlowTests.Choose(s, "Close Game");

            string racing = Path.Combine(s.RomDirectory, "SNES", "Racing");
            ThemedCollectionsTests.Records(s).SaveEdits(racing, new Dictionary<string, string?> { [EmuSen.Mistress.Library.GameMetadata.FolderLink] = ThemedFoldersTests.VelvetRally + ".sfc" }, DateTime.Now);
            ThemedCollectionsTests.Refresh(s);
            for (int guard = 0; guard < 6 && s.View != "system"; guard++) s.Pad.B();
            ThemedCollectionsTests.Enter(s, "snes");
            Assert.Equal("Racing", s.Game);
            s.Pad.A();
            Assert.Equal(ThemedFoldersTests.VelvetRally, Screen(s)?.GameName);
            s.Run(1700);
            Assert.Equal(Path.Combine(racing, ThemedFoldersTests.VelvetRally + ".sfc"), Running(s.Window));
            Stop(s.Window);
        }, default);

        private static (byte R, byte G, byte B) At(RenderedFrame f, double x, double y)
        {
            int i = ((int)y * f.Width + (int)x) * 4;
            return (f.Rgba[i], f.Rgba[i + 1], f.Rgba[i + 2]);
        }

        private static Rect InWindow(ThemedSession s, Rect r)
        {
            Point o = Host(s).TranslatePoint(default, s.Window)!.Value;
            return r.Translate(new Vector(o.X, o.Y));
        }

        private static int DifferingOutside(RenderedFrame a, RenderedFrame b, Rect keep)
        {
            int n = 0;
            for (int y = 0; y < a.Height; y++)
                for (int x = 0; x < a.Width; x++)
                {
                    if (keep.Contains(new Point(x + 0.5, y + 0.5))) continue;
                    int i = (y * a.Width + x) * 4;
                    if (a.Rgba[i] != b.Rgba[i] || a.Rgba[i + 1] != b.Rgba[i + 1] || a.Rgba[i + 2] != b.Rgba[i + 2]) n++;
                }
            return n;
        }

        // Outside a box: how many pixels were not black before, and how many of those the shade and blur changed.
        private static (int Lit, int Changed) LitAndChanged(RenderedFrame before, RenderedFrame after, Rect keep)
        {
            int lit = 0, changed = 0;
            for (int y = 0; y < before.Height; y++)
                for (int x = 0; x < before.Width; x++)
                {
                    if (keep.Contains(new Point(x + 0.5, y + 0.5))) continue;
                    int i = (y * before.Width + x) * 4;
                    if (before.Rgba[i] + before.Rgba[i + 1] + before.Rgba[i + 2] == 0) continue;
                    lit++;
                    if (before.Rgba[i] != after.Rgba[i] || before.Rgba[i + 1] != after.Rgba[i + 1] || before.Rgba[i + 2] != after.Rgba[i + 2]) changed++;
                }
            return (lit, changed);
        }

        private static readonly Color Panel = MenuPanel.PanelColorProperty.GetDefaultValue(typeof(MenuPanel));

        private static (byte, byte, byte) Rgb(Color c) => (c.R, c.G, c.B);

        // §15's pixel style: the card where ES-DE draws it, in the menus' colour, over the view blurred and shaded everywhere else.
        [Theory]
        [InlineData(1280, 800)]
        [InlineData(1920, 1200)]
        public Task The_card_is_drawn_where_ES_DE_draws_it_over_the_view_blurred(int w, int h) => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal, w, h);
            ThemedLibraryPadTests.Enter(s, "snes");
            RenderedFrame before = s.Capture();
            s.Pad.A();
            s.Run(200);
            RenderedFrame during = s.Capture();
            LaunchScreen screen = Screen(s)!;
            double hh = Host(s).Bounds.Height;
            Rect card = InWindow(s, screen.CardBounds);
            Assert.Equal(1.0, screen.CardScale);
            // ES-DE at 1280x800: a card 709 by 437 px centred 356 px down, scaled with the height (§33.2); the width holds in the menus' Barlow Condensed (P190).
            Assert.Equal(437.0 / 800 * hh, card.Height, 3);
            Assert.InRange(card.Width, 709.0 / 800 * hh - 1.5, 709.0 / 800 * hh + 1.5);
            Assert.Equal(Host(s).Bounds.Width / 2, screen.CardBounds.Center.X, 3);
            Assert.Equal(356.0 / 800 * hh, screen.CardBounds.Center.Y, 3);

            double inset = LaunchScreen.CornerRadius * hh + 2;
            foreach (Point p in new[] { card.TopLeft + new Vector(inset, 3), card.TopRight + new Vector(-inset, 3), card.BottomLeft + new Vector(inset, -3), card.BottomRight + new Vector(-inset, -3) })
                Assert.Equal(Rgb(Panel), At(during, p.X, p.Y));
            foreach (Point p in new[] { card.TopLeft + new Vector(-3, -3), card.BottomRight + new Vector(3, 3), card.TopLeft + new Vector(1, 1) })
                Assert.NotEqual(Rgb(Panel), At(during, p.X, p.Y));

            Rect art = InWindow(s, new Rect(Host(s).Bounds.Width / 2 - LaunchScreen.ArtWidth * hh / 2, screen.CardBounds.Y + LaunchScreen.ArtCentre * hh - LaunchScreen.ArtHeight * hh / 2,
                LaunchScreen.ArtWidth * hh, LaunchScreen.ArtHeight * hh));
            Assert.Equal(Rgb(Colors.MediumPurple), At(during, art.X + art.Width * 0.25, art.Center.Y + art.Height * 0.2));
            Assert.Equal(Rgb(Colors.Gold), At(during, art.X + art.Width * 0.75, art.Center.Y + art.Height * 0.2));

            (int lit, int changed) = LitAndChanged(before, during, card);
            _out.WriteLine($"{w}x{h}: card {card}; outside it {changed} of the {lit} pixels that were not black changed");
            Assert.True(changed >= lit * 0.98, $"only {changed} of {lit} lit pixels outside the card changed");
            s.Run(3000);
            Assert.True(GameShown(s));
            Stop(s.Window);
        }, default);

        // The scale-up: half size at the press, whole 117 ms later, about the card's centre.
        [Fact]
        public Task The_card_scales_up_from_half_its_size_about_its_centre() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            LaunchScreen screen = Screen(s)!;
            RenderedFrame first = s.Capture();
            Rect card = InWindow(s, screen.CardBounds);
            Assert.Equal(0.5, screen.CardScale, 3);
            Rect half = new(card.Center.X - card.Width / 4, card.Center.Y - card.Height / 4, card.Width / 2, card.Height / 2);
            Assert.Equal(Rgb(Panel), At(first, half.X + 12, half.Y + 3));
            Assert.NotEqual(Rgb(Panel), At(first, half.X - 12, half.Y + 3));
            Assert.NotEqual(Rgb(Panel), At(first, card.X + 24, card.Y + 3));
            s.Run(58);
            Assert.InRange(screen.CardScale, 0.7, 0.8);
            s.Run(60);
            Assert.Equal(1.0, screen.CardScale);
            RenderedFrame whole = s.Capture();
            Assert.Equal(Rgb(Panel), At(whole, card.X + 24, card.Y + 3));
            s.Run(3000);
            Stop(s.Window);
        }, default);

        // The popup blurs nothing: outside its pill the frame is the view's own, and it fades in over half a second.
        [Fact]
        public Task The_popup_is_a_pill_at_the_top_that_fades_in_and_leaves_the_view_as_it_was() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchPopup);
            ThemedLibraryPadTests.Enter(s, "snes");
            RenderedFrame before = s.Capture();
            s.Pad.A();
            LaunchScreen screen = Screen(s)!;
            Assert.Equal(0, screen.PopupOpacity);
            s.Run(250);
            Assert.InRange(screen.PopupOpacity, 0.45, 0.55);
            s.Run(300);
            Assert.Equal(1, screen.PopupOpacity);
            RenderedFrame during = s.Capture();
            Rect pill = InWindow(s, screen.PopupBounds);
            Assert.Equal(16, pill.Y, 3);
            Assert.InRange(pill.Center.X, 639.5, 640.5);
            Assert.Equal(Rgb(Panel), At(during, pill.X + pill.Width * 0.5, pill.Y + 2));
            Assert.Equal(0, DifferingOutside(during, before, pill));
            Assert.True(DifferingOutside(during, before, default) > 500);
            s.Run(1200);
            Stop(s.Window);
        }, default);

        // A window closed while the screen shows starts nothing afterwards, when its timer would have come due (§15.14's lesson).
        [Fact]
        public Task Closing_the_window_while_the_screen_shows_starts_nothing() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            Assert.NotNull(Screen(s));
            s.Window.Close();
            s.Now += TimeSpan.FromSeconds(4);
            typeof(MainWindow).GetMethod("AdvanceLaunchScreen", Hidden)!.Invoke(s.Window, null);
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            Assert.Null(Screen(s));
            Assert.Null(Running(s.Window));
        }, default);

        // Desktop big picture (F10) shows it; the desktop's own list starts a game at once, as it did.
        [Fact]
        public Task F10_big_picture_shows_it_and_the_desktop_list_does_not() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal, more: a => a.BigScreen = false);
            Assert.False(s.Shown);
            s.Window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            typeof(MainWindow).GetMethod("LaunchSelectedLibraryEntry", Hidden)!.Invoke(s.Window, null);
            s.Settle();
            Assert.NotNull(Running(s.Window));
            Assert.True(GameShown(s));
            Assert.Null(Screen(s));
            Stop(s.Window);
            typeof(MainWindow).GetMethod("ShowLibrary", Hidden)!.Invoke(s.Window, null);
            s.Settle();

            BigPictureSwitchTests.Press(s, Avalonia.Input.Key.F10);
            Assert.True(s.Shown);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            Assert.Equal(Aurora, Screen(s)?.GameName);
            Assert.Null(Running(s.Window));
            s.Run(3000);
            Assert.Equal(Rom(s, Aurora), Running(s.Window));
            Stop(s.Window);
        }, default);

        // The row on Theme Settings' Interface tab writes appsettings.json, and the next start reads it.
        [Fact]
        public Task The_Interface_tab_s_row_stores_the_setting() => Session.Dispatch(() =>
        {
            using var s = Open(BigPictureInterface.LaunchNormal);
            ThemedSwitchesTests.Choose(s, "LaunchScreenDuration", "Popup");
            Assert.Equal(BigPictureInterface.LaunchPopup, s.Settings.BigPictureInterface.LaunchScreenDuration);
            Assert.Equal(BigPictureInterface.LaunchPopup, AppSettings.Load().BigPictureInterface.LaunchScreenDuration);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            Assert.True(Screen(s)!.Popup);
            s.Run(1700);
            Stop(s.Window);
        }, default);
    }

    internal static class VisualQueries
    {
        public static IEnumerable<T> GetVisualDescendantsOf<T>(this Visual v) => Avalonia.VisualTree.VisualExtensions.GetVisualDescendants(v).OfType<T>().Where(c => c is Visual { IsVisible: true });
    }
}
