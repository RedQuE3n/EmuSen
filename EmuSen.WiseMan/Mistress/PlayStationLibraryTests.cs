using System;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // A zipped disc and a lone track from the library's forms, opened from Mistress's library with the pad as a player opens them, headless, in a player's build - see EmuSen_Settings_Reference.md §4.105.
    [Collection(TestCollections.ProcessGlobals)]
    public class PlayStationLibraryTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PlayStationLibraryTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPlayStationLibrary", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;
        private readonly ITestOutputHelper _output;

        public PlayStationLibraryTests(ITestOutputHelper output)
        {
            _output = output;
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private string[] Listing() => Directory.EnumerateFiles(_romDir, "*", SearchOption.AllDirectories).Order().Select(f => $"{f} {new FileInfo(f).Length} {File.GetLastWriteTimeUtc(f).Ticks}").ToArray();

        // The library's forms: a zip of a sheet and its track, a lone track, a lone "(Track 1)" file, a zip of a sheet at its root with a dozen tracks - see Diamond_Players.md §12.
        [Theory]
        [InlineData("Coral Gate (USA).zip", 1)]
        [InlineData("Ruby Lantern (USA).bin", 0)]
        [InlineData("Topaz Run (USA) (Track 1).bin", 0)]
        [InlineData("Jade Circuit (USA).zip", 11)]
        public Task A_zipped_disc_and_a_lone_track_open_from_the_library_and_play(string file, int audioTracks) => Session.Dispatch(() =>
        {
            var disc = SyntheticPsxDisc.Build();
            string path = Path.Combine(_romDir, file);
            if (file.EndsWith(".zip", StringComparison.Ordinal))
            {
                string cue = "FILE \"Inner (Track 01).bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n"
                    + string.Concat(Enumerable.Range(2, audioTracks).Select(t => $"FILE \"Inner (Track {t:00}).bin\" BINARY\n  TRACK {t:00} AUDIO\n    INDEX 00 00:00:00\n    INDEX 01 00:02:00\n"));
                using var zip = ZipFile.Open(path, ZipArchiveMode.Create);
                var entries = new[] { ("Inner.cue", Encoding.ASCII.GetBytes(cue)), ("Inner (Track 01).bin", disc.DataTrack) }
                    .Concat(Enumerable.Range(2, audioTracks).Select(t => ($"Inner (Track {t:00}).bin", disc.AudioTrack)));
                foreach (var (name, data) in entries)
                    using (var s = zip.CreateEntry(name).Open()) s.Write(data);
            }
            else File.WriteAllBytes(path, disc.DataTrack);
            File.WriteAllBytes(Path.Combine(_romDir, "Granite Choir (Synthetic).bin"), SyntheticMdRom.Cartridge());
            string[] before = Listing();
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();

            var shelves = RomLibrary.Scan(_romDir).Entries.ToDictionary(e => e.FileName, e => e.Shelf);
            Assert.Equal(("PlayStation (Diamond)", "Genesis (Nephrite)"), (shelves[file], shelves["Granite Choir (Synthetic).bin"]));

            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            try
            {
                var pad = new PadDriver(window);
                var list = window.GetControl<ListBox>("LibraryList");
                string title = Path.GetFileNameWithoutExtension(file);
                bool Listed() => list.Items.Cast<object>().Any(i => i?.ToString()?.StartsWith(title, StringComparison.Ordinal) == true);
                for (int i = 0; i < 200 && !Listed(); i++)
                {
                    Avalonia.Threading.Dispatcher.UIThread.RunJobs();
                    Thread.Sleep(25);
                }
                list.SelectedIndex = list.Items.Cast<object>().ToList().FindIndex(i => i?.ToString()?.StartsWith(title, StringComparison.Ordinal) == true);
                Assert.True(list.SelectedIndex >= 0, $"the list holds {string.Join(", ", list.Items.Cast<object>())}");
                pad.A();
                Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
                var core = Assert.IsType<CoreEngine>(MistressGame.Core(window));
                Assert.Equal(("diamond", "psx", path), (core.Info.Id, core.Machine.Info.System, core.RomPath));
                Assert.Equal("PlayStation", (string)typeof(MainWindow).GetField("_activeConsole", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!);
                Assert.True(SpinWait.SpinUntil(() => core.TotalFrames > 5, TimeSpan.FromSeconds(20)), "the game runs");
            }
            finally
            {
                typeof(MainWindow).GetMethod("StopEmulationThread", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
                window.Close();
            }
            Assert.Equal(before, Listing());
        }, default);
    
        // A file no console's content or folder decides is opened from the list as any game is, and the window says why it opens nowhere rather than closing - see Diamond_Players.md §12.
        [Fact]
        public Task A_file_that_opens_nowhere_says_why_from_the_library() => Session.Dispatch(() =>
        {
            byte[] sync = { 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00 };
            File.WriteAllBytes(Path.Combine(_romDir, "Static Hum.bin"), Enumerable.Repeat(sync, 2000).SelectMany(b => b).ToArray());
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            try
            {
                var pad = new PadDriver(window);
                var list = window.GetControl<ListBox>("LibraryList");
                for (int i = 0; i < 200 && list.Items.Count == 0; i++)
                {
                    Avalonia.Threading.Dispatcher.UIThread.RunJobs();
                    Thread.Sleep(25);
                }
                list.SelectedIndex = 0;
                pad.A();
                Avalonia.Threading.Dispatcher.UIThread.RunJobs();
                Assert.False(window.GetControl<Control>("GameFrame").IsVisible);
                Assert.Contains("could be", window.GetControl<TextBlock>("StatusText").Text);
            }
            finally
            {
                window.Close();
            }
        }, default);

        // A library folder of the tester's own discs, copied, as EMUSEN_DIAMOND_LAUNCH_LIBRARY names it; its unpackings go under EMUSEN_DIAMOND_LAUNCH_HOME where set.
        private static string? LaunchLibrary => Environment.GetEnvironmentVariable("EMUSEN_DIAMOND_LAUNCH_LIBRARY") is { } l && Directory.Exists(l) ? l : null;

        // Opens a listed game from the window's list with the pad and runs it to its frame; the status line where it does not open.
        private string OpenAndRun(string title, string path, int frames)
        {
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            try
            {
                var pad = new PadDriver(window);
                var list = window.GetControl<ListBox>("LibraryList");
                bool Is(object? i) => (i as RomEntry)?.FullPath == path || i?.ToString()?.StartsWith(title, StringComparison.Ordinal) == true;
                bool Listed() => list.Items.Cast<object>().Any(Is);
                for (int i = 0; i < 400 && !Listed(); i++)
                {
                    Avalonia.Threading.Dispatcher.UIThread.RunJobs();
                    Thread.Sleep(25);
                }
                list.SelectedIndex = list.Items.Cast<object>().ToList().FindIndex(Is);
                if (list.SelectedIndex < 0) return $"not in the window's list ({list.Items.Count} items)";
                pad.A();
                string status = window.GetControl<TextBlock>("StatusText").Text ?? "";
                if (!window.GetControl<Control>("GameFrame").IsVisible) return $"not opened: {status}";
                EmuSen.Cores.ICore? running = MistressGame.Live(window) is null ? null : MistressGame.Core(window);
                if (running is not CoreEngine core) return $"opened on {running?.GetType().Name ?? "nothing"}: {status}";
                bool ran = SpinWait.SpinUntil(() => core.TotalFrames >= frames, TimeSpan.FromSeconds(120));
                return $"opened on {core.Info.Id} as {core.Machine.Info.System}, {core.TotalFrames} frames{(ran ? "" : " (stopped short)")}, path {(core.RomPath == path ? "the library's" : core.RomPath)}: {status}";
            }
            catch (Exception e) { return $"threw {e.GetType().Name}: {e.Message}"; }
            finally
            {
                typeof(MainWindow).GetMethod("StopEmulationThread", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
                window.Close();
            }
        }

        // The launch path on the tester's discs: each item of the folder listed, routed and opened from the window as a player opens it, 120 frames run; opt-in - see Diamond_Players.md §12.
        [Fact]
        public Task The_testers_discs_list_route_and_open_from_the_library() => Session.Dispatch(() =>
        {
            if (LaunchLibrary is not { } library)
            {
                _output.WriteLine("skipped: EMUSEN_DIAMOND_LAUNCH_LIBRARY names a folder of the tester's discs, copied");
                return;
            }
            if (Environment.GetEnvironmentVariable("EMUSEN_DIAMOND_LAUNCH_HOME") is { Length: > 0 } home) DataStore.OverrideDirectory = home;
            if (Environment.GetEnvironmentVariable("EMUSEN_DIAMOND_LAUNCH_DEVELOPMENT") == "1") CoreDiscovery.UseDevelopment(true);
            string[] Library() => Directory.EnumerateFiles(library, "*", SearchOption.AllDirectories).Order().Select(f => $"{f} {new FileInfo(f).Length} {File.GetLastWriteTimeUtc(f).Ticks}").ToArray();
            string[] before = Library();
            new AppSettings { RomDirectory = library, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
            var entries = RomLibrary.Scan(library).Entries;
            foreach (string item in Directory.EnumerateFileSystemEntries(library).Order())
            {
                var mine = entries.Where(e => e.FullPath == item || e.FullPath.StartsWith(item + Path.DirectorySeparatorChar, StringComparison.Ordinal)).ToArray();
                if (mine.Length == 0) _output.WriteLine($"{Path.GetFileName(item)} | not listed");
                foreach (var e in mine)
                {
                    var (core, routed) = EmuSen.Cores.CoreCatalog.Route(e.FullPath);
                    _output.WriteLine($"{Path.GetFileName(item)} | listed as {e.FileName} on {e.Shelf} | routed {core?.Console ?? "nowhere"} {routed?.Reason}");
                }
            }
            foreach (var e in entries.Where(e => e.Shelf == "PlayStation (Diamond)"))
            {
                var started = System.Diagnostics.Stopwatch.StartNew();
                string outcome = OpenAndRun(e.Title, e.FullPath, 120);
                _output.WriteLine($"{e.FileName} | {outcome} | {started.Elapsed.TotalSeconds:F1} s");
            }
            Assert.Equal(before, Library());
        }, default);

        // The frame loop's own line each second on a disc the tester names, opened from the window as a player opens it: what the loop spends in the core and outside it, opt-in - see Diamond_Speed.md §16.
        [Fact]
        public Task The_frame_loop_reports_its_pace_on_a_named_disc() => Session.Dispatch(() =>
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_DIAMOND_PACE_DISC") is not { Length: > 0 } disc || !File.Exists(disc))
            {
                _output.WriteLine("skipped: EMUSEN_DIAMOND_PACE_DISC names a disc of the tester's, copied");
                return;
            }
            int seconds = int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_DIAMOND_PACE_SECONDS"), out int given) ? given : 20;
            // The disc's files linked into a library of their own, so that nothing else is listed and nothing of the tester's is written.
            string stem = Path.GetFileNameWithoutExtension(disc);
            foreach (string file in Directory.EnumerateFiles(Path.GetDirectoryName(disc)!).Where(f => Path.GetFileName(f).StartsWith(stem, StringComparison.Ordinal)))
                File.CreateSymbolicLink(Path.Combine(_romDir, Path.GetFileName(file)), file);
            string logs = Path.Combine(_root, "Logs");
            CoreDiscovery.UseDevelopment(true);
            new AppSettings { RomDirectory = _romDir, LogDirectory = logs, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            try
            {
                var pad = new PadDriver(window);
                var list = window.GetControl<ListBox>("LibraryList");
                for (int i = 0; i < 400 && list.Items.Count == 0; i++)
                {
                    Avalonia.Threading.Dispatcher.UIThread.RunJobs();
                    Thread.Sleep(25);
                }
                list.SelectedIndex = 0;
                pad.A();
                Assert.True(window.GetControl<Control>("GameFrame").IsVisible, window.GetControl<TextBlock>("StatusText").Text);
                var clock = System.Diagnostics.Stopwatch.StartNew();
                while (clock.Elapsed < TimeSpan.FromSeconds(seconds))
                {
                    Avalonia.Threading.Dispatcher.UIThread.RunJobs();
                    Thread.Sleep(5);
                }
            }
            finally
            {
                typeof(MainWindow).GetMethod("StopEmulationThread", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
                window.Close();
            }
            string[] lines = Directory.EnumerateFiles(logs, "general.log", SearchOption.AllDirectories).SelectMany(File.ReadAllLines).Where(l => l.StartsWith("[fps]", StringComparison.Ordinal)).ToArray();
            foreach (string line in lines) _output.WriteLine(line);
            Assert.NotEmpty(lines);
        }, default);
    }
}
