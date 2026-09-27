using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Media;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The windows a big-screen session shows as sheets, framed as ES-DE's menus in its look: the cheats and the cheat database here, the scraping status in ScrapeStatusLookTests - see EmuSen_Settings_Reference.md §4.80.
    [Collection(TestCollections.ProcessGlobals)]
    public class SheetLookTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(SheetLookTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;
        internal static readonly Color Bar = Color.Parse("#050507");

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSheetLookTests", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;

        public SheetLookTests(ITestOutputHelper output)
        {
            _out = output;
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        public static TheoryData<int, int> Sizes() => EsdeMenusTests.Sizes();

        private (MainWindow Window, PadDriver Pad) InGame(int width = 1280, int height = 800, bool bigScreen = true)
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Game.sfc"), SyntheticRom.BuildBlank());
            var app = new AppSettings
            {
                RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = bigScreen,
                CheatDatabaseDirectory = Path.Combine(_root, "Cheats"),
            };
            app.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone;
            app.Save();
            var window = new MainWindow { Width = width, Height = height };
            window.Show();
            var pad = new PadDriver(window);
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            pad.A();
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            var cheats = (CheatRegistry)typeof(MainWindow).GetField("_cheats", Hidden)!.GetValue(window)!;
            cheats.AddRamPoke("CpuBus", 0x7E0100, 0x42, "Infinite lives");
            cheats.AddRamPoke("CpuBus", 0x7E0101, 0x09, "Nine continues");
            return (window, pad);
        }

        private void Database(int games = 3)
        {
            string db = Path.Combine(_root, "Cheats", "Nintendo - Super Nintendo Entertainment System");
            Directory.CreateDirectory(db);
            for (int g = 0; g < games; g++) File.WriteAllText(Path.Combine(db, $"Game {g:D3} (USA).cht"), "cheats = 1\ncheat0_desc = \"Lives\"\ncheat0_code = \"7E010042\"\ncheat0_enable = false\n");
        }

        private static void Stop(MainWindow w) => typeof(MainWindow).GetMethod("StopEmulationThread", Hidden)!.Invoke(w, null);

        internal static SheetLayer Sheets(MainWindow w) => w.GetControl<SheetLayer>("Sheets");
        internal static Control Sheet(MainWindow w) => Sheets(w).SheetOf(Sheets(w).Current!)!;
        internal static MenuPanel Frame(MainWindow w) => Sheet(w).GetVisualDescendants().OfType<MenuPanel>().Single(m => m.Name == "SheetMenu");

        internal static T Named<T>(MainWindow w, string name) where T : Control =>
            Sheet(w).GetVisualDescendants().OfType<T>().First(c => c.Name == name);

        private static void Choose(MainWindow window, PadDriver pad, string entry)
        {
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            string[] lines = window.GetControl<ListBox>("PadMenuList").ItemsSource!.Cast<object>().Select(o => o.ToString()!).ToArray();
            int at = Array.FindIndex(lines, l => l.StartsWith(entry, StringComparison.Ordinal));
            Assert.True(at >= 0, $"No '{entry}' in the pad menu: {string.Join(", ", lines)}");
            pad.Down(at);
            pad.A();
        }

        private static RenderedFrame Settled(Window w)
        {
            Dispatcher.UIThread.RunJobs();
            UiTest.Capture(w);
            Dispatcher.UIThread.RunJobs();
            return UiTest.Capture(w);
        }

        // Framed as a menu: centred, titled by the window, over the screen blurred, its help bar's words as given, and nothing drawn outside the panel and the help bar.
        internal static void AssertFramed(MainWindow w, string title, params string[] help)
        {
            MenuPanel frame = Frame(w);
            Assert.Equal(title, frame.Title);
            EsdeMenusTests.AssertCentred(frame.PanelBounds, w.Bounds.Size);
            Assert.True(w.GetControl<BlurBackdrop>("MenuBackdrop").IsVisible);
            Assert.True(MenuLook.Covers((Control)((ContentControl)frame.GetVisualDescendants().OfType<LayoutTransformControl>().First().Child!).Content!));
            Assert.Equal(help, frame.Hints!.Select(h => h.Label).Where(l => l.Length > 0));

            RenderedFrame shown = Settled(w);
            Rect panel = EsdeMenusTests.InWindow(frame, frame.PanelBounds, w);
            Rect helpBar = EsdeMenusTests.InWindow(frame.HelpBar, new Rect(frame.HelpBar.Bounds.Size), w);
            // Transparent rather than hidden, so the focus stays where it was and nothing behind the sheet takes it and draws its ring.
            frame.Opacity = 0;
            RenderedFrame without = Settled(w);
            frame.Opacity = 1;
            Settled(w);
            (int inside, int outside) = EsdeMenusTests.Changed(shown, without, panel.Inflate(1), helpBar.Inflate(1));
            Assert.True(inside > 1000, $"only {inside} pixels changed inside");
            Assert.True(outside == 0, $"{outside} pixels changed outside the panel and the help bar, within {Outside(shown, without, panel.Inflate(1), helpBar.Inflate(1))}");
        }

        private static Rect Outside(RenderedFrame a, RenderedFrame b, params Rect[] boxes)
        {
            double l = double.MaxValue, t = double.MaxValue, r = 0, d = 0;
            for (int y = 0; y < a.Height; y++)
                for (int x = 0; x < a.Width; x++)
                {
                    if (EsdeMenusTests.At(a, x, y) == EsdeMenusTests.At(b, x, y) || boxes.Any(k => k.Contains(new Point(x + 0.5, y + 0.5)))) continue;
                    l = Math.Min(l, x); t = Math.Min(t, y); r = Math.Max(r, x + 1); d = Math.Max(d, y + 1);
                }
            return r == 0 ? default : new Rect(l, t, r - l, d - t);
        }

        // The width a framed sheet's panel takes: its share of the layer's width, no more than the share times 1.75 of the layer's height.
        internal static double PanelWidth(MainWindow w, double fraction) =>
            Math.Min(fraction * Sheets(w).Bounds.Width, fraction * 1.75 * Sheets(w).Bounds.Height);

        private static T InScope<T>(Window window, string name) where T : Control => NameScope.GetNameScope(window)!.Find<T>(name)!;

        // Every control the pad can reach is at least 40 pixels tall at 800 lines, scaled with the window.
        internal static void AssertTargets(MainWindow w, Control root)
        {
            double least = 40 * w.Bounds.Height / 800;
            List<string> small = PadAudit.Operable(root)
                .Where(e => e.IsEffectivelyVisible && e.TranslatePoint(new Point(0, e.Bounds.Height), w) is { } bottom && e.TranslatePoint(default, w) is { } top && bottom.Y - top.Y < least - 0.5)
                .Select(e => $"{PadAudit.Describe(e)} {e.TranslatePoint(new Point(0, e.Bounds.Height), w)!.Value.Y - e.TranslatePoint(default, w)!.Value.Y:F1}")
                .ToList();
            Assert.Empty(small);
        }

        // A focused push button is filled with the menu row's bar.
        internal static void AssertFilledWhenFocused(MainWindow w, Button button)
        {
            RenderedFrame f = Settled(w);
            Point at = button.TranslatePoint(new Point(6, button.Bounds.Height / 2), w)!.Value;
            (byte r, byte g, byte b) = EsdeMenusTests.At(f, (int)at.X, (int)at.Y);
            Assert.True(Math.Abs(r - Bar.R) <= 3 && Math.Abs(g - Bar.G) <= 3 && Math.Abs(b - Bar.B) <= 3, $"{button.Name} at {at}: {r},{g},{b}");
        }

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_cheats_are_framed_as_a_menu_in_the_look_with_no_file_dialogs(int width, int height) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = InGame(width, height);
            Choose(window, pad, "Cheats");
            Assert.IsType<ActiveCheatsWindow>(Sheets(window).Current);
            AssertFramed(window, "Active Cheats", "Select", "Back", "Tab", "Choose");
            Assert.False(InScope<Button>(Sheets(window).Current!, "SaveAsButton").IsVisible);
            Assert.False(InScope<Button>(Sheets(window).Current!, "LoadFromButton").IsVisible);
            Assert.True(InScope<Button>(Sheets(window).Current!, "SaveButton").IsVisible);
            AssertTargets(window, Sheet(window));
            PadAudit.Reach(Sheet(window), pad, e => e is Button { Name: "ApplyButton" });
            AssertFilledWhenFocused(window, Named<Button>(window, "ApplyButton"));

            // The table's chosen row is the bar across the table.
            PadAudit.Reach(Sheet(window), pad, e => e is ListBoxItem);
            var row = (ListBoxItem)window.FocusManager!.GetFocusedElement()!;
            RenderedFrame f = Settled(window);
            Point left = row.TranslatePoint(new Point(2, 3), window)!.Value, right = row.TranslatePoint(new Point(row.Bounds.Width - 3, 3), window)!.Value;
            foreach (Point p in new[] { left, right })
            {
                (byte r, byte g, byte b) = EsdeMenusTests.At(f, (int)p.X, (int)p.Y);
                Assert.True(r <= 8 && g <= 8 && b <= 10, $"the row at {p} is {r},{g},{b}");
            }
            Stop(window);
            window.Close();
        }, default);

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_cheat_database_is_framed_wider_its_attribution_the_footer_and_its_folder_typed_not_browsed(int width, int height) => Session.Dispatch(() =>
        {
            Database();
            (MainWindow window, PadDriver pad) = InGame(width, height);
            Choose(window, pad, "Cheats");
            PadAudit.Reach(Sheet(window), pad, e => e is Button { Name: "DatabaseButton" });
            pad.A();
            Assert.IsType<CheatDatabaseWindow>(Sheets(window).Current);
            AssertFramed(window, "Cheat Database", "Select", "Back", "Choose");
            MenuPanel frame = Frame(window);
            Assert.Equal(PanelWidth(window, 0.8), frame.PanelBounds.Width, 1.0);
            Assert.Equal(CheatDatabaseInstaller.Attribution.ReplaceLineEndings(" "), frame.Footer);
            Assert.False(Named<TextBlock>(window, "AttributionText").IsEffectivelyVisible);
            Assert.DoesNotContain(Sheet(window).GetVisualDescendants().OfType<Control>(), c => c.Name == "PART_Browse" && c.IsEffectivelyVisible);
            TextBlock status = Named<TextBlock>(window, "StatusText");
            Button close = Sheet(window).GetVisualDescendants().OfType<Button>().First(b => b.Content as string == "Close");
            Assert.True(status.TranslatePoint(new Point(0, status.Bounds.Height), window)!.Value.Y <= close.TranslatePoint(default, window)!.Value.Y + 0.5);
            AssertTargets(window, Sheet(window));
            Stop(window);
            window.Close();
        }, default);

        [Fact]
        public Task Every_control_of_the_two_cheat_sheets_is_reached_by_the_pad_and_B_backs_out_of_each() => Session.Dispatch(() =>
        {
            Database();
            (MainWindow window, PadDriver pad) = InGame();
            Choose(window, pad, "Cheats");
            Window cheats = Sheets(window).Current!;
            TabControl tabs = Sheet(window).GetVisualDescendants().OfType<TabControl>().First();
            var missing = new List<string>();
            for (int page = 0; page < tabs.ItemCount; page++)
            {
                if (page > 0) pad.R1();
                window.UpdateLayout();
                HashSet<InputElement> reached = PadAudit.Reachable(Sheet(window), pad);
                missing.AddRange(PadAudit.Operable(Sheet(window)).Where(c => !reached.Contains(c)).Select(c => $"[{(tabs.SelectedItem as TabItem)?.Header}] {PadAudit.Describe(c)}"));
            }
            PadAudit.Reach(Sheet(window), pad, e => e is Button { Name: "DatabaseButton" });
            pad.A();
            window.UpdateLayout();
            HashSet<InputElement> inDatabase = PadAudit.Reachable(Sheet(window), pad);
            missing.AddRange(PadAudit.Operable(Sheet(window)).Where(c => !inDatabase.Contains(c)).Select(c => $"[database] {PadAudit.Describe(c)}"));
            foreach (string m in missing) _out.WriteLine("unreachable: " + m);
            Assert.Empty(missing);

            pad.B();
            Assert.Same(cheats, Sheets(window).Current);
            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            Stop(window);
            window.Close();
        }, default);

        [Fact]
        public Task The_shoulders_page_a_long_list_where_the_sheet_has_no_tabs() => Session.Dispatch(() =>
        {
            Database(games: 300);
            (MainWindow window, PadDriver pad) = InGame();
            typeof(MainWindow).GetMethod("ShowCheatDatabase", Hidden)!.Invoke(window, null);
            Settled(window);
            PadAudit.Reach(Sheet(window), pad, e => e is ListBoxItem item && item.FindAncestorOfType<ListBox>()?.Name == "SystemsList");
            pad.A();
            Settled(window);
            ListBox games = Named<ListBox>(window, "GamesList");
            Assert.Equal(300, games.ItemCount);
            PadAudit.Reach(Sheet(window), pad, e => e is ListBoxItem item && item.FindAncestorOfType<ListBox>() == games);
            int before = games.IndexFromContainer((Control)window.FocusManager!.GetFocusedElement()!);
            pad.R1();
            Settled(window);
            int after = games.SelectedIndex;
            Assert.True(after >= before + 2, $"R1 moved from {before} to {after}");
            Assert.Same(games.ContainerFromIndex(after), window.FocusManager!.GetFocusedElement());
            pad.L1();
            Assert.Equal(before, games.SelectedIndex);
            Stop(window);
            window.Close();
        }, default);

        private static readonly MethodInfo PadTick = typeof(MainWindow).GetMethod("PadTick", Hidden)!;

        private static void Key(MainWindow w, Key key)
        {
            w.KeyPress(key, RawInputModifiers.None, PhysicalKey.None, null);
            w.KeyRelease(key, RawInputModifiers.None, PhysicalKey.None, null);
            PadTick.Invoke(w, null);
            Dispatcher.UIThread.RunJobs();
        }

        [Fact]
        public Task ES_DE_s_keys_drive_the_framed_cheats_and_a_text_box_keeps_the_keys_it_types_with() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = InGame();
            pad.Unplug();
            typeof(MainWindow).GetMethod("ShowActiveCheats", Hidden)!.Invoke(window, null);
            Settled(window);
            var focus = window.FocusManager!;
            Assert.IsType<TextBox>(focus.GetFocusedElement());
            var box = (TextBox)focus.GetFocusedElement()!;
            window.KeyTextInput("7E01");
            Key(window, Avalonia.Input.Key.Back);
            Assert.Equal("7E0", box.Text);
            Assert.IsType<ActiveCheatsWindow>(Sheets(window).Current);
            Key(window, Avalonia.Input.Key.Right);
            Assert.Same(box, focus.GetFocusedElement());

            Key(window, Avalonia.Input.Key.Down);
            Assert.NotSame(box, focus.GetFocusedElement());
            TabControl tabs = Sheet(window).GetVisualDescendants().OfType<TabControl>().First();
            int tab = tabs.SelectedIndex;
            Key(window, Avalonia.Input.Key.PageDown);
            Assert.Equal((tab + 1) % tabs.ItemCount, tabs.SelectedIndex);
            if (focus.GetFocusedElement() is TextBox) Key(window, Avalonia.Input.Key.Down);
            Assert.IsNotType<TextBox>(focus.GetFocusedElement());
            Key(window, Avalonia.Input.Key.Back);
            Assert.False(Sheets(window).IsPresenting);
            Stop(window);
            window.Close();
        }, default);

        [Fact]
        public Task The_desktop_keeps_its_own_windows_and_the_controller_bindings_keep_the_plain_sheet() => Session.Dispatch(() =>
        {
            (MainWindow desk, _) = InGame(bigScreen: false);
            typeof(MainWindow).GetMethod("ShowActiveCheats", Hidden)!.Invoke(desk, null);
            Dispatcher.UIThread.RunJobs();
            ActiveCheatsWindow owned = desk.OwnedWindows.OfType<ActiveCheatsWindow>().Single();
            Assert.False(MenuLook.Covers((Control)owned.Content!));
            Assert.Empty(owned.GetVisualDescendants().OfType<MenuPanel>());
            Assert.True(InScope<Button>(owned, "SaveAsButton").IsVisible);
            owned.Close();
            Stop(desk);
            desk.Close();

            (MainWindow big, PadDriver pad) = InGame();
            Choose(big, pad, "Controller Bindings");
            Assert.IsType<InputSettingsWindow>(Sheets(big).Current);
            Assert.False(Sheets(big).DrawsMenu(Sheets(big).Current!));
            Assert.Empty(Sheet(big).GetVisualDescendants().OfType<MenuPanel>());
            Stop(big);
            big.Close();
        }, default);
    }

    // The scraping status framed as a menu, the run at the left and the recent games at the right - see EmuSen_Settings_Reference.md §4.80.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScrapeStatusLookTests : EmuSen.WiseMan.Mistress.Scraping.ScrapeWindowFixture
    {
        public ScrapeStatusLookTests(ITestOutputHelper output) : base(output) { }

        public static TheoryData<int, int> Sizes() => EsdeMenusTests.Sizes();

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_status_is_framed_with_the_recent_games_beside_the_run_every_row_reached_and_B_closes_it(int width, int height) => OnUi(() =>
        {
            string[] names = ["Aurora Drift (USA)", "Brass Lantern (USA)", "Cobalt Harbor (USA)", "Dune Relay (USA)", "Zzz Unknown Dump"];
            for (int i = 0; i < names.Length; i++)
            {
                string path = Game(names[i], (byte)(10 + i * 7));
                if (i < 4) Server.Games.Add(new EmuSen.WiseMan.Mistress.Scraping.FakeGame(100 + i, names[i], Md5(path)));
            }
            MainWindow window = Open(settings: a => { a.OpenEmuFallback = false; a.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone; }, bigScreen: true);
            window.Width = width;
            window.Height = height;
            var pad = new PadDriver(window);
            Pump(200);
            Scrape(window, new EmuSen.Mistress.Scraping.ScrapeScope());
            RunEnds(window);
            Pump(1500);
            Assert.IsType<ScrapeStatusWindow>(Sheets(window).Current);
            SheetLookTests.AssertFramed(window, "Scraping", "Select", "Back", "Choose");
            Assert.Equal(SheetLookTests.PanelWidth(window, 0.9), SheetLookTests.Frame(window).PanelBounds.Width, 1.0);

            Control sheet = RootOf(window.ScrapeStatusShown!);
            ListBox recent = sheet.GetVisualDescendants().OfType<ListBox>().First(l => l.Name == "ScrapeStatusRecent");
            TextBlock heading = sheet.GetVisualDescendants().OfType<TextBlock>().First(t => t.Name == "ScrapeStatusHeading");
            double recentLeft = recent.TranslatePoint(default, window)!.Value.X, headingRight = heading.TranslatePoint(new Point(heading.Bounds.Width, 0), window)!.Value.X;
            Assert.True(recentLeft > headingRight, $"the recent games at {recentLeft} are not right of the run, which ends at {headingRight}");
            Assert.Equal(names.Length, recent.GetRealizedContainers().Count());

            window.UpdateLayout();
            var reached = PadAudit.Reachable(sheet, pad);
            List<string> missing = PadAudit.Operable(sheet).Where(c => !reached.Contains(c)).Select(PadAudit.Describe).ToList();
            foreach (string m in missing) Out.WriteLine("unreachable: " + m);
            Assert.Empty(missing);
            Assert.Equal(names.Length, reached.OfType<ListBoxItem>().Count());
            SheetLookTests.AssertTargets(window, sheet);

            pad.B();
            Assert.False(Sheets(window).IsPresenting);
        });
    }
}
