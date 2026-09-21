using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Closing a game keeps where it was, and starting it again offers that back - see EmuSen_Settings_Reference.md §4.31.
    [Collection(TestCollections.ProcessGlobals)]
    public class ResumeWhereYouLeftOffTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ResumeWhereYouLeftOffTests).GetTypeInfo().Assembly);

        private const int Zero = 0, A1 = 5, T0 = 8;
        private const uint Counter = 0x3F0;

        private readonly string _root;
        private readonly string _romPath;
        private readonly string _states;

        public ResumeWhereYouLeftOffTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenResumeTests", Guid.NewGuid().ToString("N"));
            string romDir = Path.Combine(_root, "Roms");
            _states = Path.Combine(_root, "States");
            Directory.CreateDirectory(romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Data");

            _romPath = Path.Combine(romDir, "Counter.z64");
            File.WriteAllBytes(_romPath, SyntheticN64Rom.BuildRunningFromRdram(new MipsAssembler()
                .Lui(A1, 0x8000)
                .Addiu(T0, T0, 1)
                .Sw(T0, A1, (short)Counter)
                .Beq(Zero, Zero, -3)
                .Nop()
                .ToArray()));
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private string ResumeState => SaveLibrary.ResumeStatePathFor(_romPath, _states);

        private void Settings(string resume) =>
            new AppSettings { RomDirectory = Path.GetDirectoryName(_romPath), LogDirectory = Path.Combine(_root, "Logs"), StateDirectory = _states, ResumeOnLaunch = resume }.Save();

        [Fact]
        public Task Closing_a_game_writes_where_it_was_and_the_picture_on_screen() => Session.Dispatch(() =>
        {
            Settings(AppSettings.ResumeAsk);
            MainWindow window = Open();
            Start(window);
            WaitFor(() => Game(window).TotalFrames > 30);

            Invoke(window, "ShowLibrary");

            Assert.True(File.Exists(ResumeState));
            Assert.True(File.Exists(SaveLibrary.PicturePathFor(ResumeState)));
            Assert.True(Saved().TotalFrames > 30);
            window.Close();
        }, default);

        [Fact]
        public Task A_game_resumed_carries_on_from_the_frame_it_was_closed_at() => Session.Dispatch(() =>
        {
            Settings(AppSettings.ResumeAlways);
            MainWindow window = Open();
            Start(window);
            WaitFor(() => Game(window).TotalFrames > 60);
            Invoke(window, "ShowLibrary");
            long closedAt = Saved().TotalFrames;

            Start(window);

            Assert.True(Game(window).TotalFrames >= closedAt, $"started at frame {Game(window).TotalFrames}, closed at {closedAt}");
            window.Close();
        }, default);

        [Fact]
        public Task A_game_restarted_ignores_the_state_it_left() => Session.Dispatch(() =>
        {
            Settings(AppSettings.ResumeNever);
            MainWindow window = Open();
            Start(window);
            WaitFor(() => Game(window).TotalFrames > 120);
            Invoke(window, "ShowLibrary");
            long closedAt = Saved().TotalFrames;

            window.PauseEmulation();
            Start(window);

            Assert.True(Game(window).TotalFrames < closedAt, $"started at frame {Game(window).TotalFrames}, closed at {closedAt}");
            window.Close();
        }, default);

        [Fact]
        public Task Asked_the_question_resume_loads_the_state_and_do_not_ask_again_remembers_it() => Session.Dispatch(() =>
        {
            Settings(AppSettings.ResumeAsk);
            MainWindow window = Open();
            Start(window);
            WaitFor(() => Game(window).TotalFrames > 60);
            Invoke(window, "ShowLibrary");
            long closedAt = Saved().TotalFrames;

            Task starting = Starting(window);
            ResumeWindow ask = WaitForDialog(window);
            Find<ToggleSwitch>(ask, "RememberResumeSwitch").IsChecked = true;
            Click(Find<Button>(ask, "ResumeButton"));
            WaitFor(() => starting.IsCompleted);

            Assert.True(Game(window).TotalFrames >= closedAt);
            Assert.Equal(AppSettings.ResumeAlways, AppSettings.Load().ResumeOnLaunch);
            window.Close();
        }, default);

        [Fact]
        public Task Closing_the_question_starts_nothing() => Session.Dispatch(() =>
        {
            Settings(AppSettings.ResumeAsk);
            MainWindow window = Open();
            Start(window);
            WaitFor(() => Game(window).TotalFrames > 30);
            Invoke(window, "ShowLibrary");

            Task starting = Starting(window);
            WaitForDialog(window).Close();
            WaitFor(() => starting.IsCompleted);

            Assert.Null(Live(window));
            Assert.Equal(AppSettings.ResumeAsk, AppSettings.Load().ResumeOnLaunch);
            window.Close();
        }, default);

        [Fact]
        public Task A_reset_is_not_leaving_the_game_and_writes_no_resume_state() => Session.Dispatch(() =>
        {
            Settings(AppSettings.ResumeAsk);
            MainWindow window = Open();
            Start(window);
            WaitFor(() => Game(window).TotalFrames > 30);

            Invoke(window, "ResetEmulation");

            Assert.False(File.Exists(ResumeState));
            using (GameRecords records = GameRecords.Load()) Assert.Equal(1, records.Find(_romPath)!.PlayCount);
            window.Close();
        }, default);

        [Fact]
        public Task Starting_and_closing_a_game_records_the_start_and_the_time_played() => Session.Dispatch(() =>
        {
            Settings(AppSettings.ResumeNever);
            MainWindow window = Open();
            Start(window);
            WaitFor(() => Game(window).TotalFrames > 30);
            Thread.Sleep(50);
            Invoke(window, "ShowLibrary");

            GameRecord record;
            using (GameRecords records = GameRecords.Load()) record = records.Find(_romPath)!;
            Assert.Equal(1, record.PlayCount);
            Assert.NotNull(record.LastPlayed);
            Assert.True(record.PlaySeconds > 0.04, $"{record.PlaySeconds} s");
            window.Close();
        }, default);

        [Fact]
        public Task A_slot_saved_by_the_hotkey_has_the_picture_beside_it() => Session.Dispatch(() =>
        {
            Settings(AppSettings.ResumeNever);
            MainWindow window = Open();
            Start(window);
            WaitFor(() => Game(window).TotalFrames > 30);

            window.GetControl<TextBlock>("StatusText").Text = "";
            Invoke(window, "SaveState");
            WaitFor(() => window.GetControl<TextBlock>("StatusText").Text?.StartsWith("State saved") == true);

            string picture = SaveLibrary.PicturePathFor(SaveLibrary.StatePathFor(_romPath, 1, _states));
            using var bitmap = new Avalonia.Media.Imaging.Bitmap(picture);
            Assert.Equal(Game(window).ScreenWidth, bitmap.PixelSize.Width);
            window.Close();
        }, default);

        private static MainWindow Open()
        {
            var window = new MainWindow();
            window.Show();
            return window;
        }

        private void Start(MainWindow window)
        {
            Task starting = Starting(window);
            WaitFor(() => starting.IsCompleted);
            Assert.NotNull(Live(window));
        }

        private Task Starting(MainWindow window) =>
            (Task)typeof(MainWindow).GetMethod("StartGameAsync", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, new object[] { _romPath, "Counter.z64" })!;

        private MarsCore Saved()
        {
            var core = new MarsCore();
            core.LoadRom(_romPath);
            core.LoadState(ResumeState);
            return core;
        }

        private static ResumeWindow WaitForDialog(MainWindow window)
        {
            WaitFor(() => window.OwnedWindows.OfType<ResumeWindow>().Any());
            return window.OwnedWindows.OfType<ResumeWindow>().Single();
        }

        private static T Find<T>(Window window, string name) where T : Control =>
            window.GetVisualDescendants().OfType<T>().Single(c => c.Name == name);

        private static void Click(Button button) => button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

        private static void WaitFor(Func<bool> condition)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!condition())
            {
                Dispatcher.UIThread.RunJobs();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail("timed out");
                Thread.Sleep(2);
            }
        }

        private static EmulatorSession? Live(MainWindow window) =>
            (EmulatorSession?)typeof(MainWindow).GetField("_session", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window);

        private static EmulatorSession Game(MainWindow window) => Live(window)!;

        private static void Invoke(MainWindow window, string name) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic | BindingFlags.Public, null, Type.EmptyTypes, null)!.Invoke(window, null);
    }
}
