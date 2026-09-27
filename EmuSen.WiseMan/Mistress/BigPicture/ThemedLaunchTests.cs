using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // A game started from a theme's gamelist by the pad's A - see EmuSen_Settings_Reference.md §4.52.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedLaunchTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedLaunchTests).GetTypeInfo().Assembly);

        private static bool Loaded(ThemedSession s) =>
            typeof(MainWindow).GetField("_session", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(s.Window) is EmuSen.Common.EmulatorSession { IsRomLoaded: true };

        private static void LaunchFromTheGamelist(ThemedSession s)
        {
            while (s.System != "snes") s.Hold(SDL.GamepadButton.DPadRight, 40);
            s.Hold(SDL.GamepadButton.South, 40);
            Assert.Equal("gamelist", s.View);
            s.Hold(SDL.GamepadButton.South, 40);
            s.Settle();
        }

        [Fact]
        public Task A_on_a_game_in_the_synthetic_theme_starts_it() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            LaunchFromTheGamelist(s);
            Assert.True(Loaded(s));
        }, default);

        [ArtBookNextFact]
        public Task A_on_a_game_in_Art_Book_Next_starts_it() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(themeDirectory: ArtBookNextFactAttribute.Folder);
            LaunchFromTheGamelist(s);
            Assert.True(Loaded(s));
        }, default);

        [ArtBookNextFact]
        public Task A_on_a_game_in_Art_Book_Next_starts_it_after_F10_on_the_desktop() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: a => a.BigScreen = false, themeDirectory: ArtBookNextFactAttribute.Folder);
            BigPictureSwitchTests.Press(s, Avalonia.Input.Key.F10);
            Assert.True(s.Shown);
            LaunchFromTheGamelist(s);
            Assert.True(Loaded(s));
        }, default);

        // No controller plugged in: ES-DE's default keys (USERGUIDE.md, "Default keyboard mappings"), less Escape, steer the view and start the game, which then has the keys.
        [ArtBookNextFact]
        public Task Arrows_and_Enter_on_the_keyboard_start_a_game_after_F10_on_the_desktop() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: a => a.BigScreen = false, themeDirectory: ArtBookNextFactAttribute.Folder);
            s.Pad.Unplug();
            s.Pad.Tick();
            BigPictureSwitchTests.Press(s, Avalonia.Input.Key.F10);
            Assert.True(s.Shown);
            string before = s.System!;
            Key(s, Avalonia.Input.Key.Right);
            Assert.NotEqual(before, s.System);
            while (s.System != "snes") Key(s, Avalonia.Input.Key.Right);
            Key(s, Avalonia.Input.Key.Enter);
            Assert.Equal("gamelist", s.View);
            string first = s.Game!;
            Key(s, Avalonia.Input.Key.Down);
            Assert.NotEqual(first, s.Game);
            Key(s, Avalonia.Input.Key.Back);
            Assert.Equal("system", s.View);
            Key(s, Avalonia.Input.Key.Enter);
            Key(s, Avalonia.Input.Key.Enter);
            Assert.True(Loaded(s));

            s.Window.KeyPress(Avalonia.Input.Key.Enter, RawInputModifiers.None, PhysicalKey.None, null);
            Assert.Empty((System.Collections.Generic.HashSet<EmuSen.Mistress.Input.UiButton>)typeof(MainWindow).GetField("_themedKeys", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(s.Window)!);
            s.Window.KeyRelease(Avalonia.Input.Key.Enter, RawInputModifiers.None, PhysicalKey.None, null);
        }, default);

        // F4 stands in for ES-DE's Escape (Start), which leaves big picture here (§4.54); Alt+F4 stays the window manager's.
        [Fact]
        public Task F4_opens_the_menu_over_the_themed_view_Backspace_closes_it_and_Alt_F4_is_not_taken() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            Key(s, Avalonia.Input.Key.F4);
            Assert.True(s.Window.GetControl<Control>("PadMenuPanel").IsVisible);
            Key(s, Avalonia.Input.Key.Back);
            Assert.False(s.Window.GetControl<Control>("PadMenuPanel").IsVisible);
            Assert.Equal("system", s.View);

            s.Window.KeyPress(Avalonia.Input.Key.F4, RawInputModifiers.Alt, PhysicalKey.None, null);
            s.Run(40);
            s.Window.KeyRelease(Avalonia.Input.Key.F4, RawInputModifiers.Alt, PhysicalKey.None, null);
            s.Settle();
            Assert.False(s.Window.GetControl<Control>("PadMenuPanel").IsVisible);
        }, default);

        private static void Key(ThemedSession s, Key key)
        {
            s.Window.KeyPress(key, RawInputModifiers.None, PhysicalKey.None, null);
            s.Window.KeyRelease(key, RawInputModifiers.None, PhysicalKey.None, null);
            s.Run(40);
            s.Settle();
        }
    }
}
