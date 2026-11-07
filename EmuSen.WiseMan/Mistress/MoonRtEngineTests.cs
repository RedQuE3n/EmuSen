using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;
using EmuSen.Mistress.Pages;

namespace EmuSen.WiseMan.Mistress
{
    // MoonRT, the NES's one engine, played in Mistress: frames, rewind and states, a stored Moon (C#) running there quietly, and a build without the library - see Moon_Native.md §9.5.
    [Collection(TestCollections.ProcessGlobals)]
    public class MoonRtEngineTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MoonRtEngineTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMoonRtEngineTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;

        public MoonRtEngineTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _rom = Path.Combine(roms, "Program.nes");
            File.WriteAllBytes(_rom, SyntheticNesRom.Build());
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States") }.Save();
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static void Choose(string engine)
        {
            ConsolesConfig config = ConsolesConfig.Load();
            config.SetValue("NES", EngineNames.EngineKey, engine);
            config.Save();
        }

        // The NES has one engine, so its page in Settings offers it alone.
        [Fact]
        public Task The_NES_page_offers_one_engine() => Session.Dispatch(() =>
        {
            MainWindow window = SettingsDesk.Open(SettingsScreen.Consoles, "console:NES");
            Assert.True(!SettingsDesk.Keys(window).Contains("engine") || SettingsDesk.Row(window, "engine").Choices?.Count == 1, "the page offers a choice of engine");
            window.Close();
        }, default);

        private MainWindow Start()
        {
            var window = new MainWindow();
            window.Show();
            Invoke(window, "LoadRom", _rom, "Program.nes");
            WaitFor(() => Game(window).TotalFrames > 20);
            return window;
        }

        // Pumps the UI thread's queue, since the emulation thread reports back by posting to it.
        private static void WaitFor(Func<bool> condition)
        {
            var clock = Stopwatch.StartNew();
            while (!condition())
            {
                Dispatcher.UIThread.RunJobs();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail("timed out waiting for the emulation thread");
                Thread.Sleep(2);
            }
        }

        private static WindowGame Game(MainWindow window) => new(window);

        // A stored Moon (C#) runs the game on MoonRT with no notice; frames, rewind and a state saved and loaded all work there.
        [Fact]
        public Task A_stored_csharp_choice_plays_on_MoonRT_with_rewind_and_its_states() => Session.Dispatch(() =>
        {
            Assert.True(Fixtures.CoreSidecars.Beside("moonrt").Built, Fixtures.CoreSidecars.Beside("moonrt").Report);
            Choose("Moon (C#)");
            MainWindow window = Start();
            WindowGame game = Game(window);
            Assert.Equal(EngineNames.MoonRt, MistressGame.Engine(window));
            Assert.Null(game.EngineNotice);
            Assert.DoesNotContain("not available", Status(window).Text ?? "");
            WaitFor(() => game.TotalFrames > 90 && MistressGame.RewindDepth(window) > 0);

            Status(window).Text = "";
            Invoke(window, "SaveState");
            WaitFor(() => Status(window).Text?.StartsWith("State saved") == true);
            string state = StatePath(window);
            window.Close();
            using var reader = new EmuSen.Crystal.Client.CrystalRuntime();
            using var opened = reader.Open(new EmuSen.Crystal.Client.CrystalOpen { Path = _rom, NoBattery = true });
            opened.LoadState(state);
            using var info = System.Text.Json.JsonDocument.Parse(opened.Info());
            Assert.Equal(EngineNames.MoonRt, info.RootElement.GetProperty("engine").GetString());
            Assert.True(info.RootElement.GetProperty("total_frames").GetInt64() > 60);
        }, default);

        // A build without MoonRT's library, or its sidecar, opens no NES game and says why in the player's words; the window's own runtime always finds the build's cores, so a runtime of the test's names the missing one.
        [Fact]
        public Task Without_its_library_an_nes_game_does_not_load_and_the_status_says_why() => Session.Dispatch(() =>
        {
            using (var runtime = new EmuSen.Crystal.Client.CrystalRuntime(new EmuSen.Crystal.Client.CrystalRuntimeOptions { Cores = new[] { Path.Combine(_root, "nothing") } }))
            {
                var refused = Assert.Throws<EmuSen.Crystal.Client.CrystalException>(() => runtime.Open(new EmuSen.Crystal.Client.CrystalOpen { Path = _rom, NoBattery = true }));
                Assert.Equal($"This build has no NES core: {EngineNames.MoonRt} is missing (no sidecar for it was found beside the program).", refused.Message);
            }
            // The window takes a refusal of the runtime's into its status line and holds no session.
            string unknown = Path.Combine(_root, "Program.xyz");
            File.WriteAllBytes(unknown, new byte[16]);
            var window = new MainWindow();
            window.Show();
            Invoke(window, "LoadRom", unknown, "Program.xyz");
            Assert.Null(typeof(MainWindow).GetField("_session", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window));
            Assert.StartsWith("Failed to load Program.xyz: No core in this build handles '.xyz'", Status(window).Text ?? "");
            window.Close();
        }, default);

        private static string StatePath(MainWindow window) =>
            (string)typeof(MainWindow).GetProperty("CurrentStatePath", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;

        private static TextBlock Status(MainWindow window) => window.GetControl<TextBlock>("StatusText");

        private static void Invoke(MainWindow window, string name, params object[] args) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, null, Array.ConvertAll(args, a => a.GetType()), null)!.Invoke(window, args);
    }
}
