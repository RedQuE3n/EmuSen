using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Big picture entered from the desktop and left again while the window runs, by every route, and what each switch must start and stop - see EmuSen_Settings_Reference.md §4.54.
    [Collection(TestCollections.ProcessGlobals)]
    public class BigPictureSwitchTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(BigPictureSwitchTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private const string Ticker = "<text name=\"ticker\"><pos>0.55 0.85</pos><size>0.2 0.05</size><fontSize>0.04</fontSize>" +
                                      "<text>A literal line much too long for the small box it has been given in this theme</text>" +
                                      "<container>true</container><containerType>horizontal</containerType><containerStartDelay>2</containerStartDelay></text>";

        private readonly ITestOutputHelper _out;

        public BigPictureSwitchTests(ITestOutputHelper output) => _out = output;

        internal static ThemedSession Desktop(bool theme = true, string? extraGamelist = null) =>
            new(settings: a => { a.BigScreen = false; if (!theme) a.BigPictureTheme = null; }, extraGamelist: extraGamelist);

        private static object? Get(MainWindow w, string property) => typeof(MainWindow).GetProperty(property, Hidden)!.GetValue(w);
        private static object? Field(MainWindow w, string name) => typeof(MainWindow).GetField(name, Hidden)!.GetValue(w);
        private static void Call(MainWindow w, string method, params object[] args) => typeof(MainWindow).GetMethod(method, Hidden)!.Invoke(w, args);

        internal static bool On(MainWindow w) => (bool)Get(w, "BigPictureOn")!;
        private static LunaList<RomEntry> List(MainWindow w) => (LunaList<RomEntry>)w.GetControl<ListBox>("LibraryList");
        private static SheetLayer Sheets(MainWindow w) => w.GetControl<SheetLayer>("Sheets");
        private static string SelectedCore() => AppSettings.Load().SelectedCore;

        private static string[] MenuLines(MainWindow w) =>
            w.GetControl<ListBox>("PadMenuList").ItemsSource!.Cast<object>().Select(o => o.ToString()!).ToArray();

        internal static IReadOnlyList<LunaAction> ViewMenu(MainWindow w) =>
            w.GetControl<MenuBar>("MenuStrip").Menus.Single(m => m.Title == "_View").Items.ToList();

        internal static LunaAction ViewItem(MainWindow w, string label) => MainWindowMenu.Find(w, "_View", label);

        // The View menu's entry chosen, as a click on it runs it.
        internal static void Choose(ThemedSession s, string label)
        {
            ViewItem(s.Window, label).Invoke();
            s.Settle();
        }

        internal static void Press(ThemedSession s, Key key)
        {
            s.Window.KeyPress(key, RawInputModifiers.None, PhysicalKey.None, null);
            s.Window.KeyRelease(key, RawInputModifiers.None, PhysicalKey.None, null);
            s.Settle();
        }

        // Start opens the pad menu over the library, then down to the entry and A, as a player would.
        internal static void ChooseFromPadMenu(ThemedSession s, string entry)
        {
            s.Pad.Start();
            Assert.True(s.Window.GetControl<Control>("PadMenuPanel").IsVisible);
            int at = Array.FindIndex(MenuLines(s.Window), l => l == entry);
            Assert.True(at >= 0, $"No '{entry}' in the pad menu: {string.Join(", ", MenuLines(s.Window))}");
            s.Pad.Down(at);
            s.Pad.A();
            s.Settle();
        }

        private static void Select(ThemedSession s, string title)
        {
            RomEntry entry = ((IReadOnlyList<RomEntry>)Field(s.Window, "_shownEntries")!).Single(e => e.Title == title);
            List(s.Window).Select(entry);
        }

        // Preferences' "Start in big screen mode", read off the window whichever tab is shown.
        private static LunaSwitch StartSwitch(Window preferences) =>
            (LunaSwitch)typeof(PreferencesWindow).GetField("_bigScreen", Hidden)!.GetValue(preferences)!;

        private static Popup FacetPopup(MainWindow w) =>
            w.GetControl<FilterBar>("LibraryFilter").GetVisualDescendants().OfType<ComboBox>().First().GetVisualDescendants().OfType<Popup>().First();

        private readonly record struct Look(double ListSize, double HeaderSize, double HintSize);

        private static Look LookOf(MainWindow w) =>
            new(List(w).FontSize, w.GetControl<TextBlock>("LibraryHeaderText").FontSize, w.GetControl<TextBlock>("LibraryHintText").FontSize);

        private static void AssertDesktop(ThemedSession s, Look look, string game, string console, bool fullScreen = false)
        {
            MainWindow w = s.Window;
            Assert.False(On(w));
            Assert.Equal(fullScreen, w.IsFullScreen);
            Assert.Equal(fullScreen, ViewItem(w, "_Fullscreen").IsChecked);
            Assert.True(w.GetControl<Control>("MenuStrip").IsVisible);
            Assert.True(w.GetControl<Control>("LibrarySidebarPane").IsVisible);
            Assert.False(w.GetControl<FilterBar>("LibraryFilter").ShowFacet);
            Assert.False(s.Shown);
            Assert.True(w.GetControl<Control>("LibraryContent").IsVisible);
            Assert.False(Sheets(w).PresentsWindows);
            Assert.DoesNotContain(EmbeddedPopups.StyleClass, w.Classes);
            Assert.Equal(look, LookOf(w));
            Assert.Equal(game, List(w).Selected?.Title);
            Assert.Equal("", w.GetControl<FilterBar>("LibraryFilter").SearchText);
            Assert.Equal(console, SelectedCore());
            Assert.False(AppSettings.Load().BigScreen);
        }

        private static void AssertBigPicture(ThemedSession s, bool themed)
        {
            MainWindow w = s.Window;
            Assert.True(On(w));
            Assert.True(w.IsFullScreen);
            Assert.False(w.GetControl<Control>("MenuStrip").IsVisible);
            Assert.False(w.GetControl<Control>("LibrarySidebarPane").IsVisible);
            Assert.True(w.GetControl<FilterBar>("LibraryFilter").ShowFacet);
            Assert.Equal(themed, s.Shown);
            Assert.Equal(!themed, w.GetControl<Control>("LibraryContent").IsVisible);
            Assert.True(Sheets(w).PresentsWindows);
            Assert.Contains(EmbeddedPopups.StyleClass, w.Classes);
            Assert.Equal(24, List(w).FontSize);
            Assert.False(AppSettings.Load().BigScreen);
        }

        // The pad steers what big picture shows: the carousel with its sound, or Mistress's own list and its console.
        private static void PadWorks(ThemedSession s, bool themed)
        {
            if (themed)
            {
                string? before = s.System;
                int sounds = s.Sounds.Count;
                s.Pad.Right();
                Assert.NotEqual(before, s.System);
                Assert.Equal("systembrowse", s.Sounds.Skip(sounds).First());
                return;
            }

            string? game = List(s.Window).Selected?.Title;
            s.Pad.Down();
            Assert.NotEqual(game, List(s.Window).Selected?.Title);
            FilterBar filter = s.Window.GetControl<FilterBar>("LibraryFilter");
            string console = filter.Facet as string ?? "";
            s.Pad.Right();
            Assert.NotEqual(console, filter.Facet as string);

            // A search typed in big picture's own box, which the desktop's shares.
            filter.SearchText = "Relay";
            Call(s.Window, "ShowLibraryEntries");
            s.Settle();
        }

        // Plain full screen and big picture side by side, with and without a theme: every way in and out of each, the desktop coming back as it was, window state and selection included.
        [Theory]
        [InlineData(true)]
        [InlineData(false)]
        public Task Fullscreen_and_big_picture_are_two_choices_and_every_way_out_returns_the_desktop_as_it_was(bool theme) => Session.Dispatch(() =>
        {
            using ThemedSession s = Desktop(theme);
            MainWindow w = s.Window;
            w.WindowState = WindowState.Maximized;
            Select(s, "Cobalt Harbor (Synthetic)");
            string console = SelectedCore();
            Look look = LookOf(w);
            object padTimer = Field(w, "_padTimer")!;
            const string game = "Cobalt Harbor (Synthetic)";
            AssertDesktop(s, look, game, console);
            // The View menu holds Fullscreen and, directly below it, Big Picture, each showing its key.
            IReadOnlyList<LunaAction> view = ViewMenu(w);
            int full = view.ToList().FindIndex(a => a.Text == "_Fullscreen");
            Assert.True(full >= 0);
            Assert.Equal("_Big Picture", view[full + 1].Text);
            Assert.Equal(Key.F11, view[full].Shortcut!.Key);
            Assert.Equal(Key.F10, view[full + 1].Shortcut!.Key);
            Assert.False(view[full + 1].IsCheckable);
            Assert.Null(Field(w, "_themed"));

            // The pad menu offers both on the desktop, and only the way out in big picture.
            s.Pad.Start();
            Assert.Contains("Full Screen", MenuLines(w));
            Assert.Contains("Big Picture", MenuLines(w));
            s.Pad.B();
            ChooseFromPadMenu(s, "Big Picture");
            AssertBigPicture(s, theme);
            s.Pad.Start();
            Assert.DoesNotContain(MenuLines(w), l => l.Contains("Full Screen"));
            s.Pad.B();
            Press(s, Key.Escape);
            AssertDesktop(s, look, game, console);
            Assert.Equal(WindowState.Maximized, w.WindowState);

            // Plain full screen, by the View menu, F11 and the window manager: never big picture.
            Choose(s, "_Fullscreen");
            AssertDesktop(s, look, game, console, fullScreen: true);
            Press(s, Key.F11);
            AssertDesktop(s, look, game, console);
            Assert.Equal(WindowState.Maximized, w.WindowState);
            Press(s, Key.F11);
            AssertDesktop(s, look, game, console, fullScreen: true);
            Choose(s, "_Fullscreen");
            Assert.Equal(WindowState.Maximized, w.WindowState);
            AssertDesktop(s, look, game, console);
            w.WindowState = WindowState.FullScreen;
            s.Settle();
            AssertDesktop(s, look, game, console, fullScreen: true);
            Press(s, Key.F11);
            Assert.Equal(WindowState.Maximized, w.WindowState);

            // Big picture by the View menu, out by the pad menu's entry.
            Choose(s, "_Big Picture");
            AssertBigPicture(s, theme);
            object? themed = Field(w, "_themed");
            Assert.NotNull(themed);
            PadWorks(s, theme);
            ChooseFromPadMenu(s, "Exit Big Picture");
            AssertDesktop(s, look, game, console);
            Assert.Equal(WindowState.Maximized, w.WindowState);

            // In by its key, out by Esc.
            Press(s, Key.F10);
            AssertBigPicture(s, theme);
            PadWorks(s, theme);
            Press(s, Key.Escape);
            AssertDesktop(s, look, game, console);
            Assert.Equal(WindowState.Maximized, w.WindowState);

            // In by the View menu again, out by its key.
            Choose(s, "_Big Picture");
            AssertBigPicture(s, theme);
            PadWorks(s, theme);
            Press(s, Key.F10);
            AssertDesktop(s, look, game, console);
            Assert.Equal(WindowState.Maximized, w.WindowState);

            // In from plain full screen; out by the pad menu, back to plain full screen.
            Press(s, Key.F11);
            Choose(s, "_Big Picture");
            AssertBigPicture(s, theme);
            PadWorks(s, theme);
            ChooseFromPadMenu(s, "Exit Big Picture");
            AssertDesktop(s, look, game, console, fullScreen: true);

            // Leaving full screen in big picture leaves both, to the state that route chose: F11, then the window manager.
            Press(s, Key.F10);
            AssertBigPicture(s, theme);
            Press(s, Key.F11);
            AssertDesktop(s, look, game, console);
            Assert.Equal(WindowState.Maximized, w.WindowState);
            Press(s, Key.F10);
            AssertBigPicture(s, theme);
            PadWorks(s, theme);
            w.WindowState = WindowState.Normal;
            s.Settle();
            AssertDesktop(s, look, game, console);
            Assert.Equal(WindowState.Normal, w.WindowState);

            // One pad timer and one themed library for the window's life, however often it switches.
            Assert.Same(padTimer, Field(w, "_padTimer"));
            Assert.Same(themed, Field(w, "_themed"));
        }, default);

        // Leaving with a text in its pause stops the wake: the hidden view draws nothing in real time after it, and draws again once big picture is back.
        [Fact]
        public Task Leaving_big_picture_stops_the_themed_view_s_wake_and_entering_again_starts_it() => Session.Dispatch(() =>
        {
            using ThemedSession s = Desktop(extraGamelist: Ticker);
            Press(s, Key.F10);
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Settle();
            Assert.Equal(s.Now + TimeSpan.FromSeconds(2), s.WakeAt);

            s.Now += TimeSpan.FromSeconds(1.9);
            Press(s, Key.F10);
            Assert.False(On(s.Window));
            Assert.Null(s.WakeAt);
            int before = s.FramesDrawn;
            var frame = new DispatcherFrame();
            DispatcherTimer.RunOnce(() => frame.Continue = false, TimeSpan.FromMilliseconds(2600));
            Dispatcher.UIThread.PushFrame(frame);
            Assert.Equal(before, s.FramesDrawn);

            Press(s, Key.F10);
            Assert.True(s.Shown);
            Assert.Equal("gamelist", s.View);
            Assert.Equal("snes", s.System);
            Assert.NotNull(s.WakeAt);
            Assert.InRange(s.Loop(2500), 1, 200);
        }, default);

        // The interface's sound stream is let go on the desktop and taken again in big picture.
        [Fact]
        public Task Leaving_big_picture_lets_go_of_the_interface_s_sound_stream_and_entering_takes_it_again() => Session.Dispatch(() =>
        {
            using ThemedSession s = Desktop();
            PropertyInfo sink = typeof(MainWindow).GetProperty("UiSoundSink", Hidden)!;
            object? recorder = sink.GetValue(s.Window);
            sink.SetValue(s.Window, null);
            try
            {
                Assert.False((bool)Get(s.Window, "UiSoundsHeld")!);
                Press(s, Key.F10);
                Assert.True(s.Shown);
                Assert.True((bool)Get(s.Window, "UiSoundsHeld")!);
                Press(s, Key.F10);
                Assert.False((bool)Get(s.Window, "UiSoundsHeld")!);
                Press(s, Key.F10);
                Assert.True((bool)Get(s.Window, "UiSoundsHeld")!);
            }
            finally
            {
                sink.SetValue(s.Window, recorder);
            }
        }, default);

        // A window opened in big picture is a sheet and its lists stay in the window; on the desktop it is a window again and the popups are the platform's.
        [Fact]
        public Task Windows_and_popups_follow_the_switch_sheets_in_big_picture_and_windows_on_the_desktop() => Session.Dispatch(() =>
        {
            using ThemedSession s = Desktop(theme: false);
            MainWindow w = s.Window;
            Assert.DoesNotContain(EmbeddedPopups.StyleClass, w.Classes);

            Press(s, Key.F10);
            Assert.True(FacetPopup(w).ShouldUseOverlayLayer);
            ChooseFromPadMenu(s, "Preferences");
            Assert.True(Sheets(w).IsPresenting);
            Assert.Empty(w.OwnedWindows);
            // A switch is not a choice of how to start: Preferences still says what it said.
            Assert.False(StartSwitch(Sheets(w).Current!).IsChecked == true);
            s.Pad.B();
            s.Settle();
            Assert.False(Sheets(w).IsPresenting);

            Press(s, Key.F10);
            Assert.False(FacetPopup(w).ShouldUseOverlayLayer);
            Call(w, "ShowPreferences");
            s.Settle();
            Assert.False(Sheets(w).IsPresenting);
            Window preferences = Assert.Single(w.OwnedWindows);
            Assert.False(StartSwitch(preferences).IsChecked == true);
            preferences.Close();
        }, default);

        // In a game Esc keeps its meaning, library and back; F11 leaves big picture with the game still on screen and running.
        [Fact]
        public Task In_a_game_Esc_still_goes_to_the_library_and_back_and_F11_leaves_with_the_game_running() => Session.Dispatch(() =>
        {
            using ThemedSession s = Desktop(theme: false);
            MainWindow w = s.Window;
            Press(s, Key.F10);
            Select(s, "Brass Lantern (Synthetic)");
            s.Pad.A();
            s.Settle();
            Assert.True(w.GetControl<Control>("GameFrame").IsVisible);

            Press(s, Key.Escape);
            Assert.True(w.GetControl<Control>("LibraryView").IsVisible);
            Assert.True(On(w));
            Press(s, Key.Escape);
            Assert.True(w.GetControl<Control>("GameFrame").IsVisible);
            Assert.True(On(w));

            Press(s, Key.F11);
            Assert.False(On(w));
            Assert.True(w.GetControl<Control>("GameFrame").IsVisible);
            Assert.False(w.IsPaused);
            Assert.True(w.GetControl<Control>("MenuStrip").IsVisible);
        }, default);

        // How the next start begins is Preferences' choice alone: a switch writes nothing, and a start in big picture that the player leaves keeps the choice.
        [Fact]
        public Task The_next_start_follows_preferences_and_not_the_last_switch() => Session.Dispatch(() =>
        {
            using ThemedSession s = Desktop();
            Press(s, Key.F10);
            Assert.False(AppSettings.Load().BigScreen);
            var next = new MainWindow { Width = 1280, Height = 800 };
            next.Show();
            Dispatcher.UIThread.RunJobs();
            Assert.False(On(next));
            Assert.False(next.IsFullScreen);
            next.Close();
            Press(s, Key.F10);

            AppSettings chosen = AppSettings.Load();
            chosen.BigScreen = true;
            chosen.Save();
            var started = new MainWindow { Width = 1280, Height = 800 };
            started.Show();
            Dispatcher.UIThread.RunJobs();
            Assert.True(On(started));
            Assert.True(started.IsFullScreen);
            Call(started, "SetBigPicture", false, true);
            Assert.False(On(started));
            Assert.False(started.IsFullScreen);
            Assert.True(AppSettings.Load().BigScreen);
            started.Close();
        }, default);

        // Game Mode is full screen and big picture for its whole life: no button, no menu entry, and neither F11, Esc nor the window's state takes it out.
        [Fact]
        public Task A_gamescope_session_stays_in_big_picture_and_offers_no_way_out() => Session.Dispatch(() =>
        {
            string? saved = Environment.GetEnvironmentVariable("XDG_CURRENT_DESKTOP");
            Environment.SetEnvironmentVariable("XDG_CURRENT_DESKTOP", "gamescope");
            ThemedSession s;
            try { s = Desktop(theme: false); }
            finally { Environment.SetEnvironmentVariable("XDG_CURRENT_DESKTOP", saved); }

            using (s)
            {
                MainWindow w = s.Window;
                Assert.True(On(w));
                Assert.True((bool)Get(w, "BigPictureForced")!);
                Assert.False(w.GetControl<Control>("MenuStrip").IsVisible);

                s.Pad.Start();
                Assert.DoesNotContain(MenuLines(w), l => l.Contains("Big Picture") || l.Contains("Full Screen"));
                s.Pad.B();

                Press(s, Key.F11);
                Assert.True(On(w));
                Press(s, Key.F10);
                Assert.True(On(w));
                Press(s, Key.Escape);
                Assert.True(On(w));
                w.WindowState = WindowState.Normal;
                s.Settle();
                Assert.True(On(w));
                Call(w, "SetBigPicture", false, true);
                Assert.True(On(w));
                Assert.False(AppSettings.Load().BigScreen);
            }
        }, default);

        // Popups for the whole process are a startup option, so only a session big-screen for its whole life asks for them; the setting no longer does.
        [Fact]
        public void Popups_are_embedded_process_wide_only_for_a_session_big_screen_for_its_whole_life()
        {
            static Func<string, string?> Env(string? desktop, string? deck = null) => k => k switch
            {
                "XDG_CURRENT_DESKTOP" => desktop,
                "SteamDeck" => deck,
                _ => null,
            };

            // The last mode used was big picture, and it must not decide.
            string root = Path.Combine(Path.GetTempPath(), "EmuSenPopupsAtStart", Guid.NewGuid().ToString("N"));
            EmuSen.Galaxia.ConfigStore.OverrideDirectory = root;
            try
            {
                new AppSettings { BigScreen = true }.Save();
                Assert.False(MainWindow.EmbedsPopupsAtStart([], Env("KDE")));
                Assert.True(MainWindow.EmbedsPopupsAtStart([], Env("gamescope")));
                Assert.True(MainWindow.EmbedsPopupsAtStart([], Env(null, "1")));
                Assert.True(MainWindow.EmbedsPopupsAtStart(["--bigscreen"], Env("KDE")));
                Assert.False(MainWindow.EmbedsPopupsAtStart(["--other"], Env("GNOME", "1")));
            }
            finally
            {
                EmuSen.Galaxia.ConfigStore.OverrideDirectory = null;
                try { Directory.Delete(root, recursive: true); } catch { }
            }
        }
    }
}
