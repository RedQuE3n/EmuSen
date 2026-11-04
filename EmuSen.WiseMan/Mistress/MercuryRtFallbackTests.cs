using EmuSen.Cores.Native;
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
    // A build whose MercuryRT library is missing: Mistress loads no Game Boy game and says plainly why, as for any console whose one engine is not found - see Mercury_Native.md §15.4, EmuSen_Backend.md §17.10.
    [Collection(TestCollections.ProcessGlobals)]
    public class MercuryRtFallbackTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MercuryRtFallbackTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMercuryRtFallbackTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;

        public MercuryRtFallbackTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _rom = Path.Combine(roms, "Loop.gb");
            File.WriteAllBytes(_rom, SyntheticGbRom.Build(patches: (0, new byte[] { 0x18, 0xFE })));
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(_root, "States") }.Save();
        }

        public void Dispose()
        {
            CoreDiscovery.UseDirectories(null);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static TextBlock Status(MainWindow window) => window.GetControl<TextBlock>("StatusText");

        private static void Invoke(MainWindow window, string name, params object[] args) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, null, Array.ConvertAll(args, a => a.GetType()), null)!.Invoke(window, args);
    }
}
