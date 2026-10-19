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
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Crystal;
using EmuSen.WiseMan.Fixtures;

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
            File.WriteAllBytes(_rom, BoardPrograms.Build(1, 8, cycleIrq: false));
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
            config.SetValue("NES", CoreCatalog.EngineKey, engine);
            config.Save();
        }

        // The NES has one engine, so its tab offers no choice of engine.
        [Fact]
        public Task The_NES_tab_offers_no_engine_choice() => Session.Dispatch(() =>
        {
            var window = new GraphicsSettingsWindow(new GraphicsConfig(), _ => { }, "NES");
            window.Show();
            Assert.Empty(window.GetLogicalDescendants().OfType<Dropdown>().Where(d => d.Name == $"NES.{CoreCatalog.EngineKey}"));
            window.Close();
        }, default);

        // A stored Moon (C#) runs the game on MoonRT with no notice; frames, rewind and a state saved and loaded all work there.
        [CSharpImplementationFact("reads the C# engine's machine, rewind buffer or debug target behind the session")]
        public Task A_stored_csharp_choice_plays_on_MoonRT_with_rewind_and_its_states() => Session.Dispatch(() =>
        {
            Assert.True(MoonEngines.Available, MoonEngines.Report);
            Choose("Moon (C#)");
            MainWindow window = Start();
            WindowGame game = Game(window);
            Assert.IsType<EmuSen.Cores.Native.CoreEngine>(game.Core);
            Assert.Null(game.EngineNotice);
            Assert.DoesNotContain("not available", Status(window).Text ?? "");
            WaitFor(() => game.TotalFrames > 90 && Rewind(window).Depth > 0);

            Status(window).Text = "";
            Invoke(window, "SaveState");
            WaitFor(() => Status(window).Text?.StartsWith("State saved") == true);
            using var reader = MoonEngines.Create();
            reader.LoadRom(_rom);
            reader.LoadState(StatePath(window));
            Assert.True(reader.TotalFrames > 60);
            window.Close();
        }, default);

        // A build without MoonRT's library, or its sidecar, loads no NES game, and the status says why in the player's words; discovery is per process call, so no child process is needed.
        [CSharpImplementationFact("moves the C# discovery's directories (CoreDiscovery.UseDirectories), which the Rust takes from its config once a process")]
        public Task Without_its_library_an_nes_game_does_not_load_and_the_status_says_why() => Session.Dispatch(() =>
        {
            EmuSen.Cores.Native.CoreDiscovery.UseDirectories(new[] { Path.Combine(_root, "nothing") });
            try
            {
                var window = new MainWindow();
                window.Show();
                Invoke(window, "LoadRom", _rom, "Program.nes");
                string status = Status(window).Text ?? "";
                Assert.Null(typeof(MainWindow).GetField("_session", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window));
                Assert.StartsWith("Failed to load Program.nes: ", status);
                Assert.Contains($"This build has no NES core: {CoreCatalog.MoonRtEngine} is missing (no sidecar for it was found beside the program)", status);
                window.Close();
            }
            finally { EmuSen.Cores.Native.CoreDiscovery.UseDirectories(null); }
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

        private static EmuSen.Common.RewindBuffer Rewind(MainWindow window) => MistressGame.Rewind(window);

        private static string StatePath(MainWindow window) =>
            (string)typeof(MainWindow).GetProperty("CurrentStatePath", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;

        private static TextBlock Status(MainWindow window) => window.GetControl<TextBlock>("StatusText");

        private static void Invoke(MainWindow window, string name, params object[] args) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, null, Array.ConvertAll(args, a => a.GetType()), null)!.Invoke(window, args);
    }
}
