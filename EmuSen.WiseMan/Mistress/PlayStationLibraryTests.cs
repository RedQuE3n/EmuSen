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

namespace EmuSen.WiseMan.Mistress
{
    // A zipped disc and a lone track from the library's forms, opened from Mistress's library with the pad as a player opens them, headless; Diamond's development mark shown - see EmuSen_Settings_Reference.md §4.105.
    [Collection(TestCollections.ProcessGlobals)]
    public class PlayStationLibraryTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PlayStationLibraryTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPlayStationLibrary", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;

        public PlayStationLibraryTests()
        {
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(true);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private string[] Listing() => Directory.EnumerateFiles(_romDir, "*", SearchOption.AllDirectories).Order().Select(f => $"{f} {new FileInfo(f).Length} {File.GetLastWriteTimeUtc(f).Ticks}").ToArray();

        [Theory]
        [InlineData("Coral Gate (USA).zip")]
        [InlineData("Ruby Lantern (USA).bin")]
        public Task A_zipped_disc_and_a_lone_track_open_from_the_library_and_play(string file) => Session.Dispatch(() =>
        {
            var disc = SyntheticPsxDisc.Build();
            string path = Path.Combine(_romDir, file);
            if (file.EndsWith(".zip", StringComparison.Ordinal))
            {
                using var zip = ZipFile.Open(path, ZipArchiveMode.Create);
                foreach (var (name, data) in new[] { ("Inner.cue", Encoding.ASCII.GetBytes(disc.Cue.Replace("{name}", "Inner"))), ("Inner (Track 1).bin", disc.DataTrack), ("Inner (Track 2).bin", disc.AudioTrack) })
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
                var session = (EmuSen.Common.EmulatorSession)typeof(MainWindow).GetField("_session", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;
                var core = Assert.IsType<CoreEngine>(session.Core);
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
    }
}
