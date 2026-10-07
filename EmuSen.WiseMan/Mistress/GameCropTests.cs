using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.Serenity;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;

namespace EmuSen.WiseMan.Mistress
{
    // A crop kept with one game: its window, and where the library offers it, on the desktop and in big picture - see EmuSen_Settings_Reference.md §4.100.
    [Collection(TestCollections.ProcessGlobals)]
    public class GameCropTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(GameCropTests).GetTypeInfo().Assembly);

        private static T Named<T>(Window window, string name) where T : Control =>
            Avalonia.LogicalTree.LogicalExtensions.GetLogicalDescendants(window).OfType<T>().First(c => c.Name == name);

        private static string[] Boxes(Window window) => CropFields.Edges.Select(e => Named<TextBox>(window, $"GameCrop.{e.Key}").Text!).ToArray();

        [Fact]
        public Task The_window_starts_from_the_console_s_crop_and_a_number_typed_makes_the_crop_the_game_s_own() => Session.Dispatch(() =>
        {
            var told = new List<GameCrop?>();
            var window = new GameCropWindow("Mario Kart 64 (Europe)", "N64", PictureCrop.Television, null, told.Add);
            window.Show();
            Assert.Equal("Crop This Game", window.Title);
            Assert.Equal("Mario Kart 64 (Europe)", Named<TextBlock>(window, "GameCropGame").Text);
            Assert.Equal(new[] { "3.3", "3.3", "3.3", "3.3" }, Boxes(window));
            Assert.Equal("This game is drawn with the N64's setting.", Named<HintText>(window, "GameCropStatus").Text);
            Assert.False(Named<Button>(window, "GameCropUseConsole").IsEnabled);
            Assert.Empty(told);

            Named<TextBox>(window, "GameCrop.CropTop").Text = "8,7";
            Named<TextBox>(window, "GameCrop.CropBottom").Text = "9";
            Named<TextBox>(window, "GameCrop.CropLeft").Text = "wide";
            Named<TextBox>(window, "GameCrop.CropRight").Text = "40";
            Dispatcher.UIThread.RunJobs();
            // Three numbers are told, one for each box that held one; "wide" is no number and tells nothing.
            Assert.Equal(3, told.Count);
            Assert.Equal(new GameCrop(3.3, 25, 8.7, 9), told[^1]);
            Assert.Equal(new GameCrop(3.3, 25, 8.7, 9), window.Own);
            Assert.Equal("This game has a crop of its own.", Named<HintText>(window, "GameCropStatus").Text);

            Named<Button>(window, "GameCropUseConsole").RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            Dispatcher.UIThread.RunJobs();
            Assert.Null(told[^1]);
            Assert.Equal(4, told.Count);
            Assert.Null(window.Own);
            Assert.Equal(new[] { "3.3", "3.3", "3.3", "3.3" }, Boxes(window));
            Assert.False(Named<Button>(window, "GameCropUseConsole").IsEnabled);
            window.Close();
        }, default);

        [Fact]
        public Task A_game_with_a_crop_of_its_own_opens_on_it() => Session.Dispatch(() =>
        {
            var own = new GameCropWindow("Kirby 64", "N64", PictureCrop.Television, new GameCrop(2.4, 2.5, 3.5, 3.5), _ => { });
            own.Show();
            Assert.Equal(new[] { "2.4", "2.5", "3.5", "3.5" }, Boxes(own));
            Assert.True(Named<Button>(own, "GameCropUseConsole").IsEnabled);
            Assert.Equal("This game has a crop of its own.", Named<HintText>(own, "GameCropStatus").Text);
            own.Close();
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

        // A window shown as a sheet has its content on the sheet, so the box is looked for wherever the content is.
        private static void Type(ThemedSession s, GameCropWindow window, string edge, string text)
        {
            Control? sheet = s.Window.GetControl<SheetLayer>("Sheets").SheetOf(window);
            TextBox box = sheet is null ? Named<TextBox>(window, $"GameCrop.{edge}") : sheet.GetVisualDescendants().OfType<TextBox>().First(b => b.Name == $"GameCrop.{edge}");
            box.Text = text;
            s.Settle();
        }

        // On the desktop the game's options open it as a window of Mistress's own, and what is typed is in the library for the game.
        [Fact]
        public Task On_the_desktop_the_game_s_options_open_it_as_a_window_and_the_crop_is_stored_for_that_game() => Session.Dispatch(() =>
        {
            using ThemedSession s = DesktopGameOptionsTests.Desktop();
            SheetLayer sheets = s.Window.GetControl<SheetLayer>("Sheets");
            var list = DesktopGameOptionsTests.List(s.Window);
            string game = DesktopGameOptionsTests.SnesPath(s, 1), other = DesktopGameOptionsTests.SnesPath(s, 2);
            list.Select(list.Models.Single(e => e.FullPath == game));
            s.Settle();
            BigPictureSwitchTests.ChooseFromPadMenu(s, "Game Options...");
            GameOptionsWindow options = Assert.IsType<GameOptionsWindow>(s.Window.GameOptionsShown);
            DesktopGameOptionsTests.Click(s, options.Entries.Single(b => (string?)b.Content == MainWindow.CropThisGame));

            GameCropWindow crop = Assert.IsType<GameCropWindow>(s.Window.GameCropShown);
            Assert.False(sheets.IsPresenting);
            Assert.Contains(crop, s.Window.OwnedWindows);
            Assert.Equal(ThemedSession.SnesGames[1], Named<TextBlock>(crop, "GameCropGame").Text);
            Type(s, crop, "CropTop", "8.7");
            Type(s, crop, "CropBottom", "9");
            Assert.Equal(new GameCrop(0, 0, 8.7, 9), Records(s.Window).Crop(game));
            Assert.Null(Records(s.Window).Crop(other));
            crop.Close();
            s.Settle();
            Assert.Null(s.Window.GameCropShown);
        }, default);

        // In big picture it is a sheet framed as a menu, every control of it reached by the pad, and B puts it away.
        [Fact]
        public Task In_big_picture_the_gamelist_s_options_open_it_as_a_sheet_the_pad_reaches_all_of() => ThemedLibraryPadTests.Run(s =>
        {
            SheetLayer sheets = s.Window.GetControl<SheetLayer>("Sheets");
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Select();
            s.Settle();
            GameOptionsWindow options = Assert.IsType<GameOptionsWindow>(sheets.Current);
            PadAudit.Reach(sheets.SheetOf(options)!, s.Pad, e => e is Button { Content: MainWindow.CropThisGame });
            s.Pad.A();
            s.Settle();

            GameCropWindow crop = Assert.IsType<GameCropWindow>(sheets.Current);
            Assert.Empty(s.Window.OwnedWindows);
            Assert.True(sheets.DrawsMenu(crop));
            Control sheet = sheets.SheetOf(crop)!;
            Type(s, crop, "CropLeft", "2.4");
            HashSet<Avalonia.Input.InputElement> reached = PadAudit.Reachable(sheet, s.Pad);
            Assert.Empty(PadAudit.Operable(sheet).Where(c => !reached.Contains(c)).Select(PadAudit.Describe));
            Assert.Equal(6, PadAudit.Operable(sheet).Count);
            Assert.Equal(new GameCrop(2.4, 0, 0, 0), Records(s.Window).Crop(System.IO.Path.Combine(s.RomDirectory, s.Game + ".sfc")));
            s.Pad.B();
            s.Settle();
            Assert.Null(s.Window.GameCropShown);
        });
    }
}
