using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The own look's navigation sounds: ES-DE's seven names from the player's presses, silent when switched off, at 0, over a game and on the desktop - see EmuSen_Interface.md §23.
    [Collection(TestCollections.ProcessGlobals)]
    public class OwnLookSoundsTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(OwnLookSoundsTests).GetTypeInfo().Assembly);

        private static ThemedSession Own(Action<AppSettings>? more = null) =>
            new(settings: a => { a.LibraryStyle = AppSettings.LibraryStyleMistress; more?.Invoke(a); });

        // The cues one press played, by their ES-DE names.
        private static string[] Heard(ThemedSession s, Action press)
        {
            s.Sounds.Clear();
            press();
            s.Settle();
            Assert.All(s.Sounds, n => Assert.StartsWith(NavigationSounds.Prefix, n));
            return s.Sounds.Select(n => n[NavigationSounds.Prefix.Length..]).ToArray();
        }

        private static bool GameShown(ThemedSession s) => s.Window.GetControl<Control>("GameFrame").IsVisible;

        [Fact]
        public Task Each_press_on_the_own_look_plays_its_ES_DE_cue() => Session.Dispatch(() =>
        {
            using var s = Own();
            HubLayout home = Assert.IsType<HubLayout>(s.Window.HomeLayout);
            Assert.Equal(["scroll"], Heard(s, () => s.Pad.Right()));

            home.FocusOn(HubRow.Places, "settings");
            Assert.Equal(["select"], Heard(s, () => s.Pad.A()));
            Assert.NotNull(s.Window.SettingsShown);
            Assert.Equal(["scroll"], Heard(s, () => s.Pad.Down()));
            Assert.Equal(["systembrowse"], Heard(s, () => s.Pad.R1()));
            while (s.Window.SettingsShown is not null) Assert.Equal(["back"], Heard(s, () => s.Pad.B()));

            home = Assert.IsType<HubLayout>(s.Window.HomeLayout);
            home.FocusOn(HubRow.Places, "games");
            Assert.Equal(["select"], Heard(s, () => s.Pad.A()));
            GalleryLayout grid = Assert.IsType<GalleryLayout>(s.Window.GamesLayout);
            Assert.Equal(["scroll"], Heard(s, () => s.Pad.Right()));
            Assert.Equal(["systembrowse"], Heard(s, () => s.Pad.R1()));
            Assert.Equal(["systembrowse"], Heard(s, () => s.Pad.L1()));
            Assert.Equal(["favorite"], Heard(s, () => s.Pad.Y()));

            Assert.Equal(["select"], Heard(s, () => s.Pad.X()));
            Assert.NotNull(s.Window.GamePageLayout);
            Assert.Equal(["scroll"], Heard(s, () => s.Pad.Right()));
            while (s.Window.GamePageLayout is not null) Assert.Equal(["back"], Heard(s, () => s.Pad.B()));

            Assert.Equal(["select"], Heard(s, () => s.Pad.Select()));
            Assert.Equal(["scroll"], Heard(s, () => s.Pad.Down()));
            Assert.Equal(["back"], Heard(s, () => s.Pad.B()));
            s.Run(1000);

            Assert.Same(grid, s.Window.GamesLayout);
            Assert.Equal(["back"], Heard(s, () => s.Pad.B()));
            Assert.NotNull(s.Window.HomeLayout);

            OwnLook.OpenGames(s.Window, s.Pad);
            Assert.Equal(["launch"], Heard(s, () => s.Pad.A()));
            Assert.True(GameShown(s));
            LaunchScreenTests.Stop(s.Window);
        }, default);

        [Theory]
        [InlineData("switch off")]
        [InlineData("volume 0")]
        public Task The_own_look_is_silent_with_the_switch_off_or_at_volume_0(string how) => Session.Dispatch(() =>
        {
            using var s = Own(a =>
            {
                if (how == "switch off") a.NavigationSounds = false;
                else a.BigPictureInterface.NavigationVolume = 0;
            });
            HubLayout home = Assert.IsType<HubLayout>(s.Window.HomeLayout);
            Assert.Empty(Heard(s, () => s.Pad.Right()));
            home.FocusOn(HubRow.Places, "games");
            Assert.Empty(Heard(s, () => s.Pad.A()));
            Assert.Empty(Heard(s, () => { s.Pad.Right(); s.Pad.R1(); s.Pad.Y(); s.Pad.B(); }));
        }, default);

        [Fact]
        public Task Nothing_plays_over_a_running_game_or_under_its_Pause_Card() => Session.Dispatch(() =>
        {
            using var s = Own();
            OwnLook.Play(s.Window, s.Pad);
            Assert.True(GameShown(s));
            Assert.Empty(Heard(s, () => s.Pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start)));
            Assert.Empty(Heard(s, () => { s.Pad.Down(); s.Pad.Up(); s.Pad.B(); s.Pad.A(); }));
            LaunchScreenTests.Stop(s.Window);
        }, default);

        [Fact]
        public Task The_desktop_form_is_silent() => Session.Dispatch(() =>
        {
            using var s = Own(a => a.BigScreen = false);
            Assert.Empty(Heard(s, () => { s.Pad.Down(); s.Pad.Start(); s.Pad.Down(); s.Pad.B(); s.Pad.Y(); }));
        }, default);

        // What one press played under a theme, as recorded: a theme's file by its name, Mistress's own by its key.
        private static string[] Played(ThemedSession s, Action press)
        {
            s.Sounds.Clear();
            press();
            s.Settle();
            return s.Sounds.ToArray();
        }

        // Under a theme the cards, Settings and the game page sound as under the own look, each cue resolved as the theme's view resolves it.
        private static void Walk(ThemedSession s, Func<string, string> heard)
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Start();
            Assert.True(PadMenu.IsOpen(s.Window));
            Assert.Equal([heard("scroll")], Played(s, () => s.Pad.Down()));
            Assert.Equal([heard("back")], Played(s, () => s.Pad.Start()));
            Assert.False(PadMenu.IsOpen(s.Window));

            s.Pad.Start();
            int[] path = PadMenu.PathTo(s.Window, MainWindow.SettingsMenu, exact: true)!;
            while (PadMenu.Selected(s.Window) != path[0]) s.Pad.Down();
            Assert.Equal([heard("select")], Played(s, () => s.Pad.A()));
            Assert.NotNull(s.Window.SettingsShown);
            Assert.Equal([heard("scroll")], Played(s, () => s.Pad.Down()));
            Assert.Equal([heard("systembrowse")], Played(s, () => s.Pad.R1()));
            while (s.Window.SettingsShown is not null) Assert.Equal([heard("back")], Played(s, () => s.Pad.B()));
            if (PadMenu.IsOpen(s.Window)) Assert.Equal([heard("back")], Played(s, () => s.Pad.Start()));

            s.Pad.X();
            Assert.NotNull(s.Window.GamePageLayout);
            Assert.Equal([heard("scroll")], Played(s, () => s.Pad.Right()));
            while (s.Window.GamePageLayout is not null) Assert.Equal([heard("back")], Played(s, () => s.Pad.B()));
            Assert.Equal("gamelist", s.View);
            Assert.Equal(["scroll"], Played(s, () => s.Pad.Down()).Select(n => n.Replace(NavigationSounds.Prefix, "")).ToArray());
        }

        [Fact]
        public Task Under_a_theme_the_cards_Settings_and_the_game_page_play_the_theme_s_own_cues() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            Walk(s, n => n);
        }, default);

        [Fact]
        public Task Under_a_theme_without_a_sound_the_cards_play_Mistress_s_own_for_it() => Session.Dispatch(() =>
        {
            using (var none = new ThemedSession(sounds: false)) Walk(none, NavigationSounds.Key);

            using var some = new ThemedSession();
            File.Delete(some.Theme.PathOf("sounds/scroll.wav"));
            some.Themed.Forget();
            some.Refresh();
            Walk(some, n => n == "scroll" ? NavigationSounds.Key(n) : n);
        }, default);

        [Fact]
        public Task Under_a_theme_the_cards_are_silent_with_the_switch_off_and_over_a_game() => Session.Dispatch(() =>
        {
            using (var off = new ThemedSession(settings: a => a.NavigationSounds = false))
            {
                ThemedLibraryPadTests.Enter(off, "snes");
                Assert.Empty(Played(off, () => { off.Pad.Start(); off.Pad.Down(); off.Pad.Up(); off.Pad.Start(); off.Pad.X(); off.Pad.Right(); off.Pad.B(); }));
            }

            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            Assert.True(GameShown(s));
            Assert.Empty(Played(s, () => s.Pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start)));
            Assert.True(PadMenu.IsOpen(s.Window));
            Assert.Empty(Played(s, () => { s.Pad.Down(); s.Pad.Up(); s.Pad.B(); s.Pad.A(); }));
            LaunchScreenTests.Stop(s.Window);
        }, default);
    }
}
