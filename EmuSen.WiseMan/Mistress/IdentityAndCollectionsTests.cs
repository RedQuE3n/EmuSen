using System;
using System.Collections.Generic;
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
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // A game known by its contents, states that say who wrote them, and collections the player makes - see EmuSen_Settings_Reference.md §4.37 and §4.38.
    [Collection(TestCollections.ProcessGlobals)]
    public class IdentityAndCollectionsTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(IdentityAndCollectionsTests).GetTypeInfo().Assembly);

        private const int Zero = 0, A1 = 5, T0 = 8;

        private readonly string _root, _romDir, _states;

        public IdentityAndCollectionsTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenIdentityTests", Guid.NewGuid().ToString("N"));
            _romDir = Path.Combine(_root, "Roms");
            _states = Path.Combine(_root, "States");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            new AppSettings { RomDirectory = _romDir, StateDirectory = _states, LogDirectory = Path.Combine(_root, "Logs"), ResumeOnLaunch = AppSettings.ResumeNever }.Save();
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private string Snes(string name, byte fill = 0)
        {
            string path = Path.Combine(_romDir, name);
            byte[] rom = SyntheticRom.BuildBlank();
            rom[0x100] = fill;
            File.WriteAllBytes(path, rom);
            return path;
        }

        private string Counter()
        {
            string path = Path.Combine(_romDir, "Counter.z64");
            File.WriteAllBytes(path, SyntheticN64Rom.BuildRunningFromRdram(new MipsAssembler()
                .Lui(A1, 0x8000).Addiu(T0, T0, 1).Sw(T0, A1, 0x3F0).Beq(Zero, Zero, -3).Nop().ToArray()));
            return path;
        }

        private static MainWindow Open()
        {
            var window = new MainWindow { Width = 1024, Height = 768 };
            window.Show();
            Dispatcher.UIThread.RunJobs();
            return window;
        }

        [Fact]
        public Task A_renamed_rom_keeps_its_favourite_and_its_collection() => Session.Dispatch(() =>
        {
            string before = Snes("Kirby (USA).sfc", 1);
            Snes("Other (USA).sfc", 2);
            MainWindow window = Open();
            RomEntry kirby = Entry(window, "Kirby (USA).sfc");
            Invoke(window, "ToggleFavourite", kirby);
            long rpgs = Records(window).CreateCollection("Platformers", DateTime.Now)!.Value;
            Invoke(window, "AddToCollection", rpgs, kirby);
            WaitFor(() => Records(window).Orphans(new HashSet<string>()).Any(o => o.Path == before));
            window.Close();

            string after = Path.Combine(_romDir, "Kirby's Adventure, renamed.sfc");
            File.Move(before, after);
            window = Open();

            WaitFor(() => Records(window).IsFavourite(after));
            Assert.False(Records(window).IsFavourite(before));
            Assert.Contains(rpgs, Records(window).CollectionsOf(after));
            Assert.False(Records(window).IsFavourite(Path.Combine(_romDir, "Other (USA).sfc")));
            Assert.Contains("renamed or moved", window.GetControl<TextBlock>("StatusText").Text);
            window.Close();
        }, default);

        [Fact]
        public Task A_state_saved_from_the_window_names_its_console_core_version_build_and_rom() => Session.Dispatch(() =>
        {
            string rom = Counter();
            MainWindow window = Open();
            Invoke(window, "LoadRom", rom, "Counter.z64");
            WaitFor(() => Field(window, "_currentRomMd5") is string);

            SaveSlot(window);

            StateRecord record = StateRecord.Read(SaveLibrary.StatePathFor(rom, 1, _states))!;
            Assert.Equal("N64", record.Console);
            Assert.Equal(Game(window).CoreName, record.Core);
            Assert.Equal(((EmuSen.Cores.IStateFormat)Game(window).Core!).StateVersion, record.StateVersion);
            Assert.Equal(RomHash.Md5(rom), record.RomMd5);
            Assert.Equal(new FileInfo(rom).Length, record.RomBytes);
            Assert.Equal("Counter.z64", record.RomFile);
            Assert.False(string.IsNullOrWhiteSpace(record.Build));
            window.Close();
        }, default);

        [Theory]
        [InlineData("SNES", 0, "SNES core")]
        [InlineData(null, 1, "newer build")]
        public Task A_state_from_another_console_or_a_newer_build_is_refused_before_the_core_reads_it(string? console, int newer, string says) => Session.Dispatch(() =>
        {
            string rom = Counter();
            MainWindow window = Open();
            Invoke(window, "LoadRom", rom, "Counter.z64");
            WaitFor(() => Field(window, "_currentRomMd5") is string);
            SaveSlot(window);
            string state = SaveLibrary.StatePathFor(rom, 1, _states);
            StateRecord saved = StateRecord.Read(state)!;
            (saved with { Console = console ?? saved.Console, StateVersion = saved.StateVersion + newer }).Write(state);

            Status(window).Text = "";
            Invoke(window, "LoadState");
            Dispatcher.UIThread.RunJobs();

            Assert.StartsWith("Load State:", Status(window).Text);
            Assert.Contains(says, Status(window).Text);
            window.Close();
        }, default);

        [Fact]
        public Task A_state_from_a_different_copy_of_the_game_is_asked_about_and_not_loaded_if_declined() => Session.Dispatch(() =>
        {
            string rom = Counter();
            MainWindow window = Open();
            Invoke(window, "LoadRom", rom, "Counter.z64");
            WaitFor(() => Field(window, "_currentRomMd5") is string);
            SaveSlot(window);
            string state = SaveLibrary.StatePathFor(rom, 1, _states);
            (StateRecord.Read(state)! with { RomMd5 = new string('0', 32), RomFile = "Counter (Beta).z64" }).Write(state);

            Status(window).Text = "";
            Invoke(window, "LoadState");
            WaitFor(() => window.OwnedWindows.Count > 0);
            Window ask = window.OwnedWindows.Single();
            Assert.Contains(ask.GetVisualDescendants().OfType<TextBlock>(), t => t.Text?.Contains("Counter (Beta).z64") == true);
            ask.Close();
            Dispatcher.UIThread.RunJobs();
            Thread.Sleep(100);
            Dispatcher.UIThread.RunJobs();

            Assert.DoesNotContain("State loaded", Status(window).Text ?? "");
            window.Close();
        }, default);

        [Fact]
        public Task A_collection_made_from_the_sidebar_shows_its_games_and_deleting_it_leaves_them() => Session.Dispatch(() =>
        {
            Snes("Alpha.sfc", 1);
            Snes("Beta.sfc", 2);
            MainWindow window = Open();

            Invoke(window, "ChooseCollection", MainWindow.NewCollectionKey);
            WaitFor(() => window.OwnedWindows.Count > 0);
            Window prompt = window.OwnedWindows.Single();
            prompt.GetVisualDescendants().OfType<TextBox>().Single().Text = "Shooters";
            Click(prompt.GetVisualDescendants().OfType<Button>().Single(b => b.Name == "PART_Accept"));
            WaitFor(() => Records(window).Collections().Count == 1);

            GameCollection shooters = Records(window).Collections().Single();
            Invoke(window, "ToggleMembership", shooters, Entry(window, "Beta.sfc"));
            Invoke(window, "ChooseCollection", MainWindow.CollectionKeyPrefix + shooters.Id);

            Assert.Equal(new[] { "Beta.sfc" }, Shown(window));
            Assert.Equal(MainWindow.CollectionKeyPrefix + shooters.Id, window.GetControl<SourceList>("LibrarySidebar").SelectedKey);

            Invoke(window, "ToggleMembership", shooters, Entry(window, "Beta.sfc"));
            Assert.Empty(Shown(window));
            Assert.Contains("Nothing in Shooters yet", window.GetControl<TextBlock>("LibraryHeaderText").Text);

            Task deleting = (Task)Method(window, "DeleteCollectionAsync").Invoke(window, null)!;
            WaitFor(() => window.OwnedWindows.Count > 0);
            Click(window.OwnedWindows.Single().GetVisualDescendants().OfType<Button>().First(b => b.Content as string == "Delete"));
            WaitFor(() => deleting.IsCompleted);

            Assert.Empty(Records(window).Collections());
            Assert.Equal(2, Shown(window).Length);
            window.Close();
        }, default);

        private void SaveSlot(MainWindow window)
        {
            Status(window).Text = "";
            Invoke(window, "SaveState");
            WaitFor(() => Status(window).Text?.StartsWith("State saved") == true);
        }

        private static GameRecords Records(MainWindow w) => (GameRecords)Field(w, "_records")!;
        private static EmulatorSession Game(MainWindow w) => (EmulatorSession)Field(w, "_session")!;
        private static TextBlock Status(MainWindow w) => w.GetControl<TextBlock>("StatusText");
        private static RomEntry Entry(MainWindow w, string name) => ((IReadOnlyList<RomEntry>)Field(w, "_shownEntries")!).Single(e => e.FileName == name);
        private static string[] Shown(MainWindow w) => ((IReadOnlyList<RomEntry>)Field(w, "_shownEntries")!).Select(e => e.FileName).ToArray();

        private static void Click(Button button) => button.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

        private static object? Field(MainWindow w, string name) =>
            typeof(MainWindow).GetField(name, BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(w);

        private static MethodInfo Method(MainWindow w, string name) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic)!;

        private static void Invoke(MainWindow w, string name, params object[] args) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, null, Array.ConvertAll(args, a => a.GetType()), null)!.Invoke(w, args);

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
    }
}
