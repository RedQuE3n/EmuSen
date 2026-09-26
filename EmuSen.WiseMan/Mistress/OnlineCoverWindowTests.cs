using System;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // With no developer file the failover is the only source: it asks only when ticked, and shows what arrives - see EmuSen_Settings_Reference.md §4.39 and §4.60.
    [Collection(TestCollections.ProcessGlobals)]
    public class OnlineCoverWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(OnlineCoverWindowTests).GetTypeInfo().Assembly);

        private static readonly FieldInfo Factory = typeof(MainWindow).GetField("HttpFactory", BindingFlags.Static | BindingFlags.NonPublic)!;

        private readonly string _root, _romDir;
        private readonly object _realFactory = Factory.GetValue(null)!;
        private readonly OnlineCoverTests.FakeServer _server = new() { Answer = _ => OnlineCoverTests.FakeServer.Png() };

        public OnlineCoverWindowTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenOnlineCoverWindowTests", Guid.NewGuid().ToString("N"));
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            Factory.SetValue(null, (Func<HttpClient>)(() => new HttpClient(_server)));
            File.WriteAllBytes(Path.Combine(_romDir, "F-Zero (USA).sfc"), SyntheticRom.BuildBlank());
            OnlineCoverTests.BuildOpenVgdb(OpenVgdb.DefaultPath, ("SNES", "F-Zero (USA)", "00", null));
        }

        public void Dispose()
        {
            Factory.SetValue(null, _realFactory);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private MainWindow Open(bool online)
        {
            new AppSettings { RomDirectory = _romDir, OpenEmuFallback = online }.Save();
            var window = new MainWindow { Width = 1024, Height = 768 };
            window.Show();
            return window;
        }

        [Fact]
        public Task With_the_setting_off_no_server_is_ever_asked() => Session.Dispatch(() =>
        {
            MainWindow window = Open(online: false);
            for (int i = 0; i < 40; i++) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(10); }

            Assert.Empty(_server.Asked);
            Assert.Null(typeof(MainWindow).GetField("_fetcher", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window));
            window.Close();
        }, default);

        [Fact]
        public Task With_it_on_a_cover_shown_without_art_is_fetched_into_its_own_folder_and_its_outcome_kept() => Session.Dispatch(() =>
        {
            MainWindow window = Open(online: true);
            string cover = Path.Combine(DataStore.Media, "openemu", "SNES", "F-Zero (USA).png");
            WaitFor(() => File.Exists(cover));
            WaitFor(() => Records(window).CoverLookup(Path.Combine(_romDir, "F-Zero (USA).sfc")) == nameof(CoverOutcome.Found));

            Assert.Single(_server.Asked);
            Assert.Contains("thumbnails.libretro.com", _server.Asked.Single());
            window.Close();

            window = Open(online: true);
            for (int i = 0; i < 40; i++) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(10); }
            Assert.Single(_server.Asked);
            window.Close();
        }, default);

        // A game with no box anywhere is asked about once, not every time its tile is drawn again.
        [Fact]
        public Task A_game_with_no_art_anywhere_is_not_asked_about_again() => Session.Dispatch(() =>
        {
            _server.Answer = _ => new HttpResponseMessage(HttpStatusCode.NotFound);
            MainWindow window = Open(online: true);
            WaitFor(() => Records(window).CoverLookup(Path.Combine(_romDir, "F-Zero (USA).sfc")) == nameof(CoverOutcome.NoArt));
            int asked = _server.Asked.Count;
            window.Close();

            window = Open(online: true);
            for (int i = 0; i < 40; i++) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(10); }
            Assert.Equal(asked, _server.Asked.Count);
            window.Close();
        }, default);

        private static GameRecords Records(MainWindow w) =>
            (GameRecords)typeof(MainWindow).GetField("_records", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(w)!;

        private static void WaitFor(Func<bool> condition)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!condition())
            {
                Dispatcher.UIThread.RunJobs();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail("timed out");
                Thread.Sleep(5);
            }
        }
    }
}
