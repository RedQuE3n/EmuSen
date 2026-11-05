using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture;
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
    }
}
