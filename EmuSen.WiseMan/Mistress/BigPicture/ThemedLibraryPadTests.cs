using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
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
    // The themed view as a big-screen session's library, steered by a pad with no device: each rule of §4.9's table - see EmuSen_BigPicture.md §15.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedLibraryPadTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedLibraryPadTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;

        public ThemedLibraryPadTests(ITestOutputHelper output) => _out = output;

        internal static Task Run(Action<ThemedSession> test, Action<AppSettings>? settings = null, string? extraGamelist = null) => Session.Dispatch(() =>
        {
            using var session = new ThemedSession(settings: settings, extraGamelist: extraGamelist);
            test(session);
        }, default);

        internal static TextBox SearchBox(ThemedSession s) =>
            Avalonia.VisualTree.VisualExtensions.GetVisualDescendants(s.Window).OfType<TextBox>().Single(b => b.Name == "ThemedSearchBox");

        // From the system view: right until the named system, then into its gamelist.
        internal static void Enter(ThemedSession s, string system)
        {
            for (int guard = 0; guard < 6 && s.System != system; guard++) s.Pad.Right();
            Assert.Equal(system, s.System);
            s.Pad.A();
            Assert.Equal("gamelist", s.View);
        }

        [Fact]
        public Task The_themed_view_is_the_library_of_a_big_screen_session_with_a_theme() => Run(s =>
        {
            Assert.True(s.Shown, s.Themed.Error);
            Assert.False(s.Window.GetControl<Control>("LibraryContent").IsVisible);
            Assert.False(s.Window.GetControl<Control>("StatusBar").IsVisible);
            Assert.Equal("system", s.View);
            Assert.Equal(new[] { "nes", "gb", "snes" }, s.Themed.Stage!.Current.Data.Systems.Select(x => x.System.Name));
            Assert.Equal("nes", s.System);
            Assert.Equal(new Size(1280, 800), s.Themed.Stage.Current.Data.Screen);
        });

        [Theory]
        [InlineData("Mistress style")]
        [InlineData("no theme")]
        [InlineData("not big screen")]
        [InlineData("broken theme")]
        public Task Mistress_library_is_kept_where_the_themed_view_is_not_wanted_or_cannot_load(string why) => Session.Dispatch(() =>
        {
            Action<AppSettings> settings = why switch
            {
                "Mistress style" => a => a.LibraryStyle = AppSettings.LibraryStyleMistress,
                "no theme" => a => a.BigPictureTheme = null,
                "not big screen" => a => a.BigScreen = false,
                _ => a => a.BigPictureTheme = Path.Combine(Path.GetTempPath(), "EmuSenNoSuchTheme"),
            };
            using var s = new ThemedSession(settings: settings);
            Assert.False(s.Shown);
            Assert.True(s.Window.GetControl<Control>("LibraryContent").IsVisible);
            if (why == "broken theme") Assert.Contains("could not be read", s.Window.GetControl<TextBlock>("StatusText").Text);
        }, default);

        [Fact]
        public Task Left_and_right_move_the_system_carousel_with_its_sound_and_a_held_direction_repeats_at_the_measured_rate() => Run(s =>
        {
            s.Pad.Right();
            Assert.Equal("gb", s.System);
            Assert.Equal(new[] { "systembrowse" }, s.Sounds);
            s.Pad.Left();
            Assert.Equal("nes", s.System);

            // ES-DE's carousel without fastScrolling: the press, then 500 ms, then every 200 ms (§14.7).
            s.Sounds.Clear();
            s.Hold(SDL.GamepadButton.DPadRight, 480);
            Assert.Single(s.Sounds);
            s.Sounds.Clear();
            s.Hold(SDL.GamepadButton.DPadRight, 520);
            Assert.Equal(2, s.Sounds.Count);
            s.Sounds.Clear();
            s.Hold(SDL.GamepadButton.DPadRight, 910);
            Assert.Equal(4, s.Sounds.Count);
            Assert.All(s.Sounds, n => Assert.Equal("systembrowse", n));
            // One tap right, one left, then 1 + 2 + 4 held steps: seven right of nes round three systems is gb.
            Assert.Equal("gb", s.System);
            Assert.Equal("system", s.View);
        });

        [Fact]
        public Task South_enters_the_system_s_gamelist_and_east_goes_back_each_with_its_sound() => Run(s =>
        {
            s.Pad.Right(2);
            s.Sounds.Clear();
            s.Pad.A();
            Assert.Equal("gamelist", s.View);
            Assert.Equal("snes", s.System);
            Assert.Equal(ThemedSession.SnesGames[0], s.Game);
            s.Pad.B();
            Assert.Equal("system", s.View);
            Assert.Equal("snes", s.System);
            Assert.Equal(new[] { "select", "back" }, s.Sounds);

            // Each system's gamelist opens at the game last chosen there, and a system never entered at its first game.
            s.Pad.A();
            s.Pad.Down(3);
            s.Pad.B();
            s.Pad.Left(2);
            Assert.Equal("nes", s.System);
            s.Pad.A();
            Assert.Equal(ThemedSession.NesGames[0], s.Game);
            s.Pad.B();
            s.Pad.Right(2);
            s.Pad.A();
            Assert.Equal(ThemedSession.SnesGames[3], s.Game);
        });

        [Fact]
        public Task Up_and_down_move_the_list_with_its_repeat_and_left_and_right_change_the_system_at_the_game_last_chosen_there() => Run(s =>
        {
            Enter(s, "snes");
            s.Sounds.Clear();
            s.Pad.Down(2);
            Assert.Equal(ThemedSession.SnesGames[2], s.Game);
            Assert.Equal(new[] { "scroll", "scroll" }, s.Sounds);
            s.Pad.Up(3);
            Assert.Equal(ThemedSession.SnesGames[^1], s.Game);

            // ES-DE's list: the press, then 500 ms, then every 114 ms, and a held list stops at its end (§14.7).
            s.Pad.L2();
            s.Sounds.Clear();
            s.Hold(SDL.GamepadButton.DPadDown, 500 + 114 + 20);
            Assert.Equal(3, s.Sounds.Count);
            Assert.Equal(ThemedSession.SnesGames[3], s.Game);
            s.Hold(SDL.GamepadButton.DPadDown, 1500);
            Assert.Equal(ThemedSession.SnesGames[^1], s.Game);

            // Quick system select keeps the gamelist view and remembers each system's game.
            s.Pad.Up();
            s.Sounds.Clear();
            s.Pad.Right();
            Assert.Equal("gamelist", s.View);
            Assert.Equal("nes", s.System);
            Assert.Equal(new[] { "quicksysselect" }, s.Sounds);
            s.Pad.Down();
            Assert.Equal(ThemedSession.NesGames[1], s.Game);
            s.Pad.Left();
            Assert.Equal("snes", s.System);
            Assert.Equal(ThemedSession.SnesGames[3], s.Game);
            s.Pad.Right();
            Assert.Equal(ThemedSession.NesGames[1], s.Game);
        });

        [Fact]
        public Task Shoulders_page_by_the_rows_the_list_shows_and_triggers_jump_to_the_first_and_last() => Run(s =>
        {
            Enter(s, "snes");
            int page = ThemedLibrary.PageSize(s.Themed.Stage!.Current);
            Assert.InRange(page, 2, 30);
            s.Pad.R1();
            Assert.Equal(ThemedSession.SnesGames[Math.Min(page, ThemedSession.SnesGames.Length - 1)], s.Game);
            s.Pad.L1();
            Assert.Equal(ThemedSession.SnesGames[0], s.Game);
            s.Sounds.Clear();
            s.Pad.R2();
            Assert.Equal(ThemedSession.SnesGames[^1], s.Game);
            s.Pad.L2();
            Assert.Equal(ThemedSession.SnesGames[0], s.Game);
            Assert.Equal(new[] { "scroll", "scroll" }, s.Sounds);
            // At the first game a jump to it moves nothing and says nothing.
            s.Pad.L2();
            Assert.Equal(2, s.Sounds.Count);
        });

        [Fact]
        public Task Shoulders_page_a_list_longer_than_a_page_by_exactly_one_page() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            for (int i = 0; i < 40; i++) File.WriteAllBytes(Path.Combine(s.RomDirectory, $"Zeta {i:D2}.sfc"), SyntheticRom.BuildBlank());
            s.Window.GetType().GetMethod("RefreshLibrary", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(s.Window, null);
            s.Settle();
            Enter(s, "snes");
            var list = s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<TextRowList>().Single();
            int page = (int)Math.Floor(list.Bounds.Height / list.RowPitch + 1e-6);
            Assert.Equal(page, ThemedLibrary.PageSize(s.Themed.Stage.Current));
            s.Pad.R1();
            Assert.Equal(page, s.Themed.Stage.Current.Index);
            s.Pad.R1();
            Assert.Equal(2 * page, s.Themed.Stage.Current.Index);
            s.Pad.L1();
            Assert.Equal(page, s.Themed.Stage.Current.Index);
        }, default);

        [Fact]
        public Task North_searches_with_the_on_screen_keyboard_and_east_clears_the_search_before_it_goes_back() => Run(s =>
        {
            Enter(s, "snes");
            s.Pad.Y();
            OnScreenKeyboard keyboard = OnScreenKeyboard.OpenOver(s.Window) ?? throw new InvalidOperationException("no keyboard");
            PadCheatsTests.TypeByPad(s.Pad, keyboard, "cob");
            s.Pad.Start();
            Assert.Null(OnScreenKeyboard.OpenOver(s.Window));
            Assert.Equal("cob", s.Themed.Filter);
            Assert.Equal(new[] { ThemedSession.SnesGames[2] }, s.Themed.Stage!.Current.Data.System.Games.Select(g => g.Name));
            Assert.Equal(ThemedSession.SnesGames[2], s.Game);
            Assert.True(SearchBox(s).IsEffectivelyVisible);

            // The list still moves under a search; east first clears it, keeping the game, then goes back.
            s.Sounds.Clear();
            s.Pad.B();
            Assert.Equal("", s.Themed.Filter);
            Assert.Equal("gamelist", s.View);
            Assert.Equal(ThemedSession.SnesGames[2], s.Game);
            Assert.Equal(ThemedSession.SnesGames.Length, s.Themed.Stage.Current.Data.System.Games.Count);
            Assert.False(SearchBox(s).IsEffectivelyVisible);
            s.Pad.B();
            Assert.Equal("system", s.View);
            Assert.Equal(new[] { "back", "back" }, s.Sounds);
        });

        [Fact]
        public Task Select_opens_the_game_options_whose_favourite_entry_moves_the_game_first_and_keeps_it_selected() => Run(s =>
        {
            Enter(s, "snes");
            s.Pad.Down(3);
            string chosen = ThemedSession.SnesGames[3];
            s.Sounds.Clear();
            ThemedGameOptionsTests.Choose(s, "Add to Favourites");
            Assert.Equal(new[] { "favorite" }, s.Sounds);
            Assert.Equal(chosen, s.Game);
            Assert.True(s.Themed.SelectedGame!.Favorite);
            Assert.Equal(chosen, s.Themed.Stage!.Current.Data.System.Games[0].Name);
            Assert.Equal(0, s.Themed.Stage.Current.Index);
            ThemedGameOptionsTests.Choose(s, "Remove from Favourites");
            Assert.False(s.Themed.SelectedGame!.Favorite);
            Assert.Equal(chosen, s.Game);
            Assert.Equal(3, s.Themed.Stage.Current.Index);
        });

        [Fact]
        public Task Start_opens_the_pad_menu_over_the_view_and_no_window_is_opened() => Run(s =>
        {
            s.Pad.Start();
            Assert.True(s.Window.GetControl<Control>("PadMenuPanel").IsVisible);
            Assert.True(s.Shown);
            string[] lines = s.Window.GetControl<ListBox>("PadMenuList").ItemsSource!.Cast<object>().Select(o => o.ToString()!).ToArray();
            Assert.Contains("Preferences", lines);
            // The directions steer the menu while it is up, not the view behind it.
            s.Pad.Right();
            Assert.Equal("nes", s.System);
            s.Pad.B();
            Assert.False(s.Window.GetControl<Control>("PadMenuPanel").IsVisible);
            Assert.Empty(s.Window.OwnedWindows);
            s.Pad.Right();
            Assert.Equal("gb", s.System);
        });
    }
}
