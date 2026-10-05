using System;
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
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // A game started from the themed gamelist, the menu over it, and the way back to the same system and game - see EmuSen_BigPicture.md §15.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedLibraryFlowTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedLibraryFlowTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;

        public ThemedLibraryFlowTests(ITestOutputHelper output) => _out = output;

        private static string[] MenuLines(MainWindow w) => PadMenu.AllLines(w);

        // The in-game chord, then down to the entry and A, as a player would.
        internal static void Choose(ThemedSession s, string entry)
        {
            s.Pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            Assert.True(s.Window.GetControl<Control>("PadMenuPanel").IsVisible);
            PadMenu.Choose(s.Window, s.Pad, entry);
        }

        // Quit Game... from the in-game menu, and Yes to its question.
        internal static void QuitGame(ThemedSession s)
        {
            Choose(s, "Quit Game");
            PadMenu.Answer(s.Window, s.Pad, yes: true);
        }

        private static void Stop(MainWindow window) =>
            typeof(MainWindow).GetMethod("StopEmulationThread", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);

        private static string? Running(MainWindow w) => (string?)typeof(MainWindow).GetField("_currentRomPath", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(w);

        private static SheetLayer Sheets(MainWindow w) => w.GetControl<SheetLayer>("Sheets");

        [Fact]
        public Task A_pad_walks_systems_searches_favourites_starts_a_game_and_comes_back_to_the_same_system_and_game() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            s.Pad.Right();
            Assert.Equal("gb", s.System);
            s.Pad.Right();
            Assert.Equal("snes", s.System);
            s.Pad.A();
            Assert.Equal("gamelist", s.View);

            ThemedGameOptionsTests.Choose(s, "Search...");
            PadCheatsTests.TypeByPad(s.Pad, OnScreenKeyboard.OpenOver(s.Window)!, "dune");
            s.Pad.Start();
            Assert.Equal("Dune Relay (Synthetic)", s.Game);
            s.Pad.Y();
            Assert.True(s.Themed.SelectedGame!.Favorite);

            s.Sounds.Clear();
            s.Pad.A();
            Assert.Equal(new[] { "launch" }, s.Sounds);
            Assert.True(s.Window.GetControl<Control>("GameFrame").IsVisible);
            Assert.False(s.Window.GetControl<Control>("LibraryView").IsVisible);
            Assert.Equal(Path.Combine(s.RomDirectory, "Dune Relay (Synthetic).sfc"), Running(s.Window));
            Assert.Empty(s.Window.OwnedWindows);

            // While the game is on screen the pad is the game's; the view behind hears nothing.
            s.Pad.Right();
            Assert.Equal("snes", s.System);
            Assert.Null(s.WakeAt);

            Choose(s, "Back to Library");
            Assert.True(s.Shown);
            Assert.True(s.Window.GetControl<Control>("LibraryView").IsVisible);
            Assert.True(s.Window.IsPaused);
            Assert.Equal("gamelist", s.View);
            Assert.Equal("snes", s.System);
            Assert.Equal("Dune Relay (Synthetic)", s.Game);
            Assert.Equal("dune", s.Themed.Filter);

            // East from the system view goes back to the suspended game.
            s.Pad.B();
            s.Pad.B();
            Assert.Equal("system", s.View);
            s.Pad.B();
            Assert.True(s.Window.GetControl<Control>("GameFrame").IsVisible);
            Assert.False(s.Window.IsPaused);

            QuitGame(s);
            Assert.True(s.Shown);
            Assert.Null(Running(s.Window));
            Assert.Equal("system", s.View);
            Assert.Equal("snes", s.System);
            s.Pad.A();
            Assert.Equal("Dune Relay (Synthetic)", s.Game);
            Stop(s.Window);
        }, default);

        // P40: the frame after the return is the frame a fresh view of the same selection draws.
        [Fact]
        public Task The_first_frame_after_the_return_is_a_fresh_static_build_of_the_same_selection() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Down(2);
            s.Run(1000);
            s.Pad.A();
            Choose(s, "Back to Library");
            Assert.Equal("Cobalt Harbor (Synthetic)", s.Game);
            RenderedFrame back = s.Capture();

            SceneData data = s.Themed.Stage!.Current.Data;
            RenderedFrame fresh = SceneAssets.Render(SceneBuilder.Build(data.System.Theme.View("gamelist"), data));
            Assert.Equal((back.Width, back.Height), (fresh.Width, fresh.Height));
            Assert.Equal(0, SceneAssets.Differing(back, fresh));
            Stop(s.Window);
        }, default);

        [Fact]
        public Task The_resume_question_is_asked_on_a_sheet_over_the_themed_view_and_answered_by_pad() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: a =>
            {
                a.ResumeOnLaunch = AppSettings.ResumeAsk;
                a.StateDirectory = Path.Combine(Path.GetTempPath(), "EmuSenThemedStates", Guid.NewGuid().ToString("N"));
            });
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Down();
            s.Pad.A();
            Assert.True(s.Window.GetControl<Control>("GameFrame").IsVisible);
            QuitGame(s);
            Assert.True(s.Shown);
            Assert.Equal("Brass Lantern (Synthetic)", s.Game);

            s.Pad.A();
            Assert.IsType<ResumeWindow>(Sheets(s.Window).Current);
            Assert.Empty(s.Window.OwnedWindows);
            Assert.True(s.Shown);
            s.Pad.A();
            Assert.False(Sheets(s.Window).IsPresenting);
            Assert.True(s.Window.GetControl<Control>("GameFrame").IsVisible);
            Stop(s.Window);
        }, default);
    }
}
