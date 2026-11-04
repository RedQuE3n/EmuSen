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


        private static string StatePath(MainWindow window) =>
            (string)typeof(MainWindow).GetProperty("CurrentStatePath", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;

        private static TextBlock Status(MainWindow window) => window.GetControl<TextBlock>("StatusText");

        private static void Invoke(MainWindow window, string name, params object[] args) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, null, Array.ConvertAll(args, a => a.GetType()), null)!.Invoke(window, args);
    }
}
