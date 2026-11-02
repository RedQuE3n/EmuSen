using System;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Media;
using Avalonia.VisualTree;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The big-screen menus in ES-DE's layout, read off the pixels: a centred panel, a bar the panel's width, and nothing drawn outside it but the backdrop and the help - see EmuSen_BigPicture.md §32.
    [Collection(TestCollections.ProcessGlobals)]
    public class EsdeMenusTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(EsdeMenusTests).GetTypeInfo().Assembly);

        public static TheoryData<int, int> Sizes() => new() { { 1280, 800 }, { 1920, 1200 } };

        internal static Rect InWindow(Control c, Rect r, Window w) => new(c.TranslatePoint(r.Position, w)!.Value, r.Size);

        internal static (byte R, byte G, byte B) At(RenderedFrame f, int x, int y)
        {
            int i = (y * f.Width + x) * 4;
            return (f.Rgba[i], f.Rgba[i + 1], f.Rgba[i + 2]);
        }

        // Pixels that differ between two frames inside the given boxes, and outside all of them.
        internal static (int Inside, int Outside) Changed(RenderedFrame a, RenderedFrame b, params Rect[] boxes)
        {
            int inside = 0, outside = 0;
            for (int y = 0; y < a.Height; y++)
                for (int x = 0; x < a.Width; x++)
                {
                    if (At(a, x, y) == At(b, x, y)) continue;
                    var p = new Point(x + 0.5, y + 0.5);
                    if (boxes.Any(r => r.Contains(p))) inside++; else outside++;
                }
            return (inside, outside);
        }

        // The bar across the row's top few pixels: the accent edge at its left, then the bar's colour to the panel's right edge (LunaP §206.4).
        internal static void AssertBarSpans(RenderedFrame f, Rect bar, Rect panel, Color colour, Color accent)
        {
            Assert.Equal(panel.Left, bar.Left, 0.5);
            Assert.Equal(panel.Width, bar.Width, 0.5);
            int y = (int)Math.Ceiling(bar.Top) + 2;
            static bool Near((byte R, byte G, byte B) p, Color c) => Math.Abs(p.R - c.R) <= 2 && Math.Abs(p.G - c.G) <= 2 && Math.Abs(p.B - c.B) <= 2;
            int x = (int)Math.Ceiling(panel.Left), end = (int)Math.Floor(panel.Right), edge = 0, wrong = 0;
            for (; x < end && Near(At(f, x, y), accent); x++) edge++;
            Assert.InRange(edge, 1, (int)Math.Ceiling(EmuSen.LunaP.Theme.LunaTokens.FocusMarks.BarWidth * 2) + 1);
            for (; x < end; x++) if (!Near(At(f, x, y), colour)) wrong++;
            Assert.Equal(0, wrong);
        }

        internal static void AssertCentred(Rect panel, Size window)
        {
            Assert.Equal(window.Width / 2, panel.Center.X, 1.0);
            Assert.True(panel.Top > 0 && panel.Bottom < window.Height, $"panel {panel} leaves the window");
        }

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_home_card_is_anchored_left_its_bar_spans_the_panel_and_the_theme_is_dimmed_not_blurred(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedLibraryPadTests.Enter(s, "snes");
            RenderedFrame before = s.Capture();
            s.Pad.Start();
            s.Pad.Down(2);
            RenderedFrame open = s.Capture();

            MenuPanel menu = PadMenu.Big(s.Window);
            Assert.True(PadMenu.Card(s.Window).IsVisible);
            Assert.False(s.Window.GetControl<Control>("PadMenuDesk").IsVisible);
            Assert.Equal(MainWindow.HomeTitle, menu.Title);
            Assert.Equal(MenuAnchor.Left, menu.Anchor);
            Rect panel = InWindow(menu, menu.PanelBounds, s.Window);
            Assert.True(panel.X < s.Window.Bounds.Width * 0.1 && panel.Right < s.Window.Bounds.Width * 0.5, $"the card {panel} is not at the left");

            ListBox list = s.Window.GetControl<ListBox>("PadMenuList");
            Assert.Equal(2, list.SelectedIndex);
            MenuRow row = ((Control)list.ContainerFromIndex(2)!).GetVisualDescendants().OfType<MenuRow>().Single();
            Assert.True(row.IsHighlighted);
            AssertBarSpans(open, InWindow(row, row.Layout(row.Bounds.Size).Bar, s.Window), panel, row.BarColor, row.AccentColor);

            // Dimmed by the card's scrim, never blurred; the theme's own help bar is put away while the card's is shown (EmuSen_Interface.md §7).
            Assert.False(s.Window.GetControl<BlurBackdrop>("MenuBackdrop").IsVisible);
            Assert.Null(s.Window.GetControl<Control>("ScreenContent").Effect);
            Assert.All(s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<HintBar>(), b => Assert.False(b.IsVisible));

            // The same frame with the panel alone taken away differs only inside the panel and the help bar; the scrim alone changed the theme.
            Rect help = InWindow(menu.HelpBar, new Rect(menu.HelpBar.Bounds.Size), s.Window);
            menu.IsVisible = false;
            RenderedFrame scrimOnly = s.Capture();
            menu.IsVisible = true;
            (int inside, int outside) = Changed(open, scrimOnly, panel.Inflate(1), help.Inflate(1));
            Assert.True(inside > 1000, $"only {inside} pixels of the panel were drawn");
            Assert.Equal(0, outside);
            int dimmed = Changed(before, scrimOnly).Outside;
            Assert.True(dimmed > before.Width * before.Height / 20, $"the scrim changed only {dimmed} pixels of the screen");

            s.Pad.B();
            s.Settle();
            Assert.False(PadMenu.IsOpen(s.Window));
            Assert.All(s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<HintBar>(), b => Assert.True(b.IsVisible));
        }, default);

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_game_options_are_a_centred_panel_their_focused_row_spans_it_and_nothing_else_is_drawn_outside_it(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedLibraryPadTests.Enter(s, "snes");
            ThemedCollectionsTests.OpenMenu(s);
            RenderedFrame open = s.Capture();

            Window window = ThemedCollectionsTests.Sheets(s).Current!;
            Assert.True(SheetLayer.GetChromeless(window));
            MenuPanel menu = ThemedCollectionsTests.Named<MenuPanel>(s, "GameOptionsMenu");
            Assert.Equal("Gamelist Options", menu.Title);
            Rect panel = InWindow(menu, menu.PanelBounds, s.Window);
            AssertCentred(panel, s.Window.GetControl<Control>("ScreenContent").Bounds.Size);

            // The first row, Jump To, has the focus and the bar; the Close button of the desktop's sheet is gone, Apply and Cancel stay.
            Dropdown jump = ThemedCollectionsTests.Named<Dropdown>(s, "GamelistJumpTo");
            Assert.True(jump.IsFocused);
            MenuRow row = jump.GetVisualDescendants().OfType<MenuRow>().Single();
            Assert.Equal(MenuRowKind.Option, row.Kind);
            AssertBarSpans(open, InWindow(row, row.Layout(row.Bounds.Size).Bar, s.Window), panel, row.BarColor, row.AccentColor);
            Assert.Equal("Apply", ThemedCollectionsTests.Named<Button>(s, "GameOptionsClose").Content);
            Assert.Equal(MenuRowKind.Submenu, MenuRows.GetKind(ThemedCollectionsTests.Named<Button>(s, "GameOption_EditThisGamesMetadata")));

            Rect help = InWindow(menu.HelpBar, new Rect(menu.HelpBar.Bounds.Size), s.Window);
            menu.IsVisible = false;
            RenderedFrame backdropOnly = s.Capture();
            menu.IsVisible = true;
            (int inside, int outside) = Changed(open, backdropOnly, panel.Inflate(1), help.Inflate(1));
            Assert.True(inside > 1000, $"only {inside} pixels of the panel were drawn");
            Assert.Equal(0, outside);
            Assert.True(s.Window.GetControl<BlurBackdrop>("MenuBackdrop").IsBlurring);

            s.Pad.Select();
            s.Settle();
            Assert.False(s.Window.GetControl<BlurBackdrop>("MenuBackdrop").IsVisible);
        }, default);

        // Over a running game the title is the game's name as the library shows it, with no extension (Q81).
        [Fact]
        public Task The_in_game_menu_is_titled_with_the_game_s_name_as_the_library_shows_it() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            string game = s.Game!;
            s.Pad.A();
            s.Settle();
            s.Pad.Chord(SDL3.SDL.GamepadButton.Back, SDL3.SDL.GamepadButton.Start);
            s.Settle();
            // Over a game the menu is the Pause Card, whose title row carries the name (EmuSen_BigPicture.md §48.2).
            MenuPanel menu = PadMenu.Big(s.Window);
            Assert.True(menu.IsEffectivelyVisible);
            Assert.Same(PadMenu.Card(s.Window).CardPanel, menu);
            Assert.Equal(game, menu.Title);
            Assert.DoesNotContain(".sfc", menu.Title!);
        }, default);

        // F4 and Backspace, the keyboard's Start and B in the themed view (settings reference §4.52a), open and close the new menu as they did the old.
        [Fact]
        public Task F4_opens_the_big_menu_and_Backspace_closes_it_with_no_pad() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            s.Pad.Unplug();
            s.Pad.Tick();
            Press(s, Avalonia.Input.Key.F4);
            Assert.True(PadMenu.Big(s.Window).IsEffectivelyVisible);
            Assert.False(s.Window.GetControl<BlurBackdrop>("MenuBackdrop").IsBlurring);
            Press(s, Avalonia.Input.Key.Down);
            Assert.Equal(1, s.Window.GetControl<ListBox>("PadMenuList").SelectedIndex);
            Press(s, Avalonia.Input.Key.Back);
            Assert.False(s.Window.GetControl<Control>("PadMenuPanel").IsVisible);
            Assert.False(s.Window.GetControl<BlurBackdrop>("MenuBackdrop").IsBlurring);
        }, default);

        // A pad opens the same Home card on the desktop, over the library dimmed and with no backdrop (EmuSen_Interface.md §2.3).
        [Fact]
        public Task A_pad_opens_the_home_card_on_the_desktop_too() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: a => { a.BigScreen = false; a.LibraryStyle = EmuSen.Galaxia.Models.AppSettings.LibraryStyleMistress; });
            s.Pad.Start();
            s.Settle();
            Assert.False(s.Window.GetControl<Control>("PadMenuDesk").IsVisible);
            Assert.True(PadMenu.Card(s.Window).IsVisible);
            Assert.Equal(MainWindow.HomeTitle, PadMenu.Big(s.Window).Title);
            Assert.False(s.Window.GetControl<BlurBackdrop>("MenuBackdrop").IsVisible);
            ListBox list = s.Window.GetControl<ListBox>("PadMenuList");
            Assert.Same(list, PadMenu.Card(s.Window).Content);
            Assert.NotEmpty(list.GetVisualDescendants().OfType<MenuRow>());
            Assert.Null(s.Window.GetControl<Control>("ScreenContent").Effect);
        }, default);

        internal static void Press(ThemedSession s, Key key)
        {
            s.Window.KeyPress(key, RawInputModifiers.None, PhysicalKey.None, null);
            s.Window.KeyRelease(key, RawInputModifiers.None, PhysicalKey.None, null);
            s.Run(40);
            s.Settle();
        }
    }
}
