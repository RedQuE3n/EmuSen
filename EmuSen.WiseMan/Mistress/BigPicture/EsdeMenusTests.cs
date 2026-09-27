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

        private static Rect InWindow(Control c, Rect r, Window w) => new(c.TranslatePoint(r.Position, w)!.Value, r.Size);

        private static (byte R, byte G, byte B) At(RenderedFrame f, int x, int y)
        {
            int i = (y * f.Width + x) * 4;
            return (f.Rgba[i], f.Rgba[i + 1], f.Rgba[i + 2]);
        }

        // Pixels that differ between two frames inside the given boxes, and outside all of them.
        private static (int Inside, int Outside) Changed(RenderedFrame a, RenderedFrame b, params Rect[] boxes)
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

        // The bar across the row's top few pixels, above the text: every one the bar's colour from the panel's left edge to its right.
        private static void AssertBarSpans(RenderedFrame f, Rect bar, Rect panel, Color colour)
        {
            Assert.Equal(panel.Left, bar.Left, 0.5);
            Assert.Equal(panel.Width, bar.Width, 0.5);
            int y = (int)Math.Ceiling(bar.Top) + 2;
            int wrong = 0;
            for (int x = (int)Math.Ceiling(panel.Left); x < (int)Math.Floor(panel.Right); x++)
            {
                (byte r, byte g, byte b) = At(f, x, y);
                if (Math.Abs(r - colour.R) > 2 || Math.Abs(g - colour.G) > 2 || Math.Abs(b - colour.B) > 2) wrong++;
            }
            Assert.Equal(0, wrong);
        }

        private static void AssertCentred(Rect panel, Size window)
        {
            Assert.Equal(window.Width / 2, panel.Center.X, 1.0);
            Assert.True(panel.Top > 0 && panel.Bottom < window.Height, $"panel {panel} leaves the window");
        }

        [Theory]
        [MemberData(nameof(Sizes))]
        public Task The_start_menu_is_centred_its_bar_spans_the_panel_and_it_draws_nothing_outside_the_panel_but_the_backdrop_and_the_help(int width, int height) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(width, height);
            ThemedLibraryPadTests.Enter(s, "snes");
            RenderedFrame before = s.Capture();
            s.Pad.Start();
            s.Pad.Down(2);
            RenderedFrame open = s.Capture();

            MenuPanel menu = s.Window.GetControl<MenuPanel>("PadMenuBig");
            Assert.True(menu.IsVisible);
            Assert.False(s.Window.GetControl<Control>("PadMenuDesk").IsVisible);
            Assert.Equal("Main Menu", menu.Title);
            Assert.Equal("EmuSen 0.9.0", menu.Footer);
            Rect panel = InWindow(menu, menu.PanelBounds, s.Window);
            AssertCentred(panel, s.Window.GetControl<Control>("ScreenContent").Bounds.Size);

            ListBox list = s.Window.GetControl<ListBox>("PadMenuList");
            Assert.Equal(2, list.SelectedIndex);
            MenuRow row = ((Control)list.ContainerFromIndex(2)!).GetVisualDescendants().OfType<MenuRow>().Single();
            Assert.True(row.IsHighlighted);
            AssertBarSpans(open, InWindow(row, row.Layout(row.Bounds.Size).Bar, s.Window), panel, row.BarColor);

            // The screen under it is blurred, and the theme's own help bar is put away while the menu's is shown.
            BlurBackdrop backdrop = s.Window.GetControl<BlurBackdrop>("MenuBackdrop");
            Assert.True(backdrop.IsBlurring);
            Assert.IsType<BlurEffect>(s.Window.GetControl<Control>("ScreenContent").Effect);
            Assert.All(s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<HintBar>(), b => Assert.False(b.IsVisible));

            // The same frame with the panel alone taken away differs only inside the panel and the help bar.
            Rect help = InWindow(menu.HelpBar, new Rect(menu.HelpBar.Bounds.Size), s.Window);
            menu.IsVisible = false;
            RenderedFrame backdropOnly = s.Capture();
            menu.IsVisible = true;
            (int inside, int outside) = Changed(open, backdropOnly, panel.Inflate(1), help.Inflate(1));
            Assert.True(inside > 1000, $"only {inside} pixels of the panel were drawn");
            Assert.Equal(0, outside);
            // The synthetic theme is mostly black, which a shade leaves black; its list, cover and art must all have changed.
            int blurred = Changed(before, backdropOnly).Outside;
            Assert.True(blurred > before.Width * before.Height / 20, $"the backdrop changed only {blurred} pixels of the screen");

            s.Pad.B();
            s.Settle();
            Assert.False(backdrop.IsBlurring);
            Assert.Null(s.Window.GetControl<Control>("ScreenContent").Effect);
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
            AssertBarSpans(open, InWindow(row, row.Layout(row.Bounds.Size).Bar, s.Window), panel, row.BarColor);
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
            MenuPanel menu = s.Window.GetControl<MenuPanel>("PadMenuBig");
            Assert.True(menu.IsEffectivelyVisible);
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
            Assert.True(s.Window.GetControl<MenuPanel>("PadMenuBig").IsEffectivelyVisible);
            Assert.True(s.Window.GetControl<BlurBackdrop>("MenuBackdrop").IsBlurring);
            Press(s, Avalonia.Input.Key.Down);
            Assert.Equal(1, s.Window.GetControl<ListBox>("PadMenuList").SelectedIndex);
            Press(s, Avalonia.Input.Key.Back);
            Assert.False(s.Window.GetControl<Control>("PadMenuPanel").IsVisible);
            Assert.False(s.Window.GetControl<BlurBackdrop>("MenuBackdrop").IsBlurring);
        }, default);

        // The desktop keeps its own menu and sheet: the list in the bordered box, no backdrop, and a game's options on a sheet of their own.
        [Fact]
        public Task A_desktop_session_keeps_the_desktop_menus() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: a => { a.BigScreen = false; a.LibraryStyle = EmuSen.Galaxia.Models.AppSettings.LibraryStyleMistress; });
            s.Pad.Start();
            s.Settle();
            Assert.True(s.Window.GetControl<Control>("PadMenuDesk").IsVisible);
            Assert.False(s.Window.GetControl<MenuPanel>("PadMenuBig").IsVisible);
            Assert.False(s.Window.GetControl<BlurBackdrop>("MenuBackdrop").IsVisible);
            ListBox list = s.Window.GetControl<ListBox>("PadMenuList");
            Assert.IsType<DockPanel>(list.Parent);
            Assert.Empty(list.GetVisualDescendants().OfType<MenuRow>());
            Assert.Null(s.Window.GetControl<Control>("ScreenContent").Effect);
        }, default);

        private static void Press(ThemedSession s, Key key)
        {
            s.Window.KeyPress(key, RawInputModifiers.None, PhysicalKey.None, null);
            s.Window.KeyRelease(key, RawInputModifiers.None, PhysicalKey.None, null);
            s.Run(40);
            s.Settle();
        }
    }
}
