using System;
using System.IO;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Common;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // A closed MainWindow, in a game or in the themed library, is collected with what it drew - see EmuSen_Debugging_Tools_Reference_v5.md §3.62.
    [Collection(TestCollections.ProcessGlobals)]
    public class WindowRetentionTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(WindowRetentionTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenWindowRetention", Guid.NewGuid().ToString("N"));

        public WindowRetentionTests()
        {
            Directory.CreateDirectory(_root);
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        [MethodImpl(MethodImplOptions.NoInlining)]
        private (WeakReference Window, WeakReference Core) InGame(bool bigScreen = false, string? sheet = null)
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            File.WriteAllBytes(Path.Combine(roms, "Game.sfc"), PadRewindReelTests.ChangingBackdrop());
            new AppSettings
            {
                RomDirectory = roms, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = bigScreen,
                StateDirectory = Path.Combine(_root, "States"), LogDirectory = Path.Combine(_root, "Logs"),
            }.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            typeof(MainWindow).GetMethod("LaunchSelectedLibraryEntry", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            for (int i = 0; i < 5; i++) { Dispatcher.UIThread.RunJobs(); UiTest.Capture(window); Thread.Sleep(20); }
            if (sheet is not null)
            {
                typeof(MainWindow).GetMethod(sheet, BindingFlags.Instance | BindingFlags.NonPublic | BindingFlags.Public)!.Invoke(window, null);
                Dispatcher.UIThread.RunJobs();
                UiTest.Capture(window);
                Assert.NotNull(window.GetControl<SheetLayer>("Sheets").Current);
            }
            object core = ((EmulatorSession)typeof(MainWindow).GetField("_session", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!).Core!;
            window.Close();
            return (new WeakReference(window), new WeakReference(core));
        }

        [MethodImpl(MethodImplOptions.NoInlining)]
        private static WeakReference Themed()
        {
            using var s = new ThemedSession();
            s.Run(200);
            s.Capture();
            return new WeakReference(s.Window);
        }

        private static void Collect()
        {
            for (int i = 0; i < 4; i++)
            {
                AvaloniaHeadlessPlatform.ForceRenderTimerTick();
                Dispatcher.UIThread.RunJobs();
                GC.Collect();
                GC.WaitForPendingFinalizers();
            }
        }

        [Fact]
        public Task A_closed_window_that_ran_a_game_is_collected() => Session.Dispatch(() =>
        {
            (WeakReference window, WeakReference core) = InGame();
            Collect();
            Assert.False(window.IsAlive, "the closed MainWindow is still reachable");
            Assert.False(core.IsAlive, "the game's core is still reachable");
        }, default);

        [Theory]
        [InlineData("ShowGraphicsSettings")]
        [InlineData("ShowControllerBindings")]
        public Task A_window_closed_under_a_sheet_is_collected(string sheet) => Session.Dispatch(() =>
        {
            (WeakReference window, WeakReference core) = InGame(bigScreen: true, sheet: sheet);
            Collect();
            Assert.False(window.IsAlive, "the closed MainWindow is still reachable");
            Assert.False(core.IsAlive, "the game's core is still reachable");
        }, default);

        [Fact]
        public Task A_closed_themed_window_is_collected() => Session.Dispatch(() =>
        {
            WeakReference w = Themed();
            Collect();
            Assert.False(w.IsAlive, "the closed MainWindow is still reachable");
        }, default);
    }
}
