using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // MoonRT on the core ABI v1 beside its pre-stable exports: its sidecar from the build, the shim its loader, and the v1 adapter running it exactly as the shim does - see EmuSen_CoreAPI.md §22.
    [Collection(TestCollections.ProcessGlobals)]
    public class MoonRtCoreAbiTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMoonRtV1_" + Guid.NewGuid().ToString("N"));
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public MoonRtCoreAbiTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreOptions.BatteryRamDisabled = true;
            CoreDiscovery.UseDirectories(null);
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            DataStore.OverrideDirectory = null;
            CoreDiscovery.UseDirectories(null);
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static DiscoveredCore Discovered => CoreDiscovery.Found.Single(c => c.Info.Id == "moonrt");

        [Fact]
        public void The_build_writes_moonrts_sidecar_and_discovery_lists_and_opens_it()
        {
            Assert.Equal("MoonRT (Rust)", Discovered.EngineName);
            Assert.Equal(new[] { ".nes" }, Discovered.Info.Systems.Single().Extensions);
            var library = Discovered.Open();
            Assert.True(library is { Available: true }, Discovered.Report);
            Assert.Equal(CoreInterface.CapReset | CoreInterface.CapMutes | CoreInterface.CapRomPatches | CoreInterface.CapDebug | CoreInterface.CapDebugStack | CoreInterface.CapRawPicture, library!.Capabilities);
            Assert.True(MoonNative.Available, MoonNative.Report);
        }

        // Discovery lists the library, yet the shim is its loader and the NES has no Engine row, MoonRT being its only engine.
        [Fact]
        public void The_shim_stays_moonrts_loader_and_the_nes_has_no_engine_row()
        {
            _ = Discovered;
            Assert.Null(CoreCatalog.EngineFor("NES"));
            Assert.True(CoreCatalog.IsRegisteredEngine(CoreCatalog.MoonRtEngine));
            foreach (string? engine in new[] { null, CoreCatalog.MoonRtEngine })
            {
                ICore core = CoreFactory.Create("game.nes", engine: engine);
                Assert.IsType<MoonRtCore>(core);
                Assert.Null(CoreFactory.EngineNotice("game.nes", engine, core));
                ((IDisposable)core).Dispose();
            }
        }

        [Fact]
        public void The_v1_adapter_over_moonrt_runs_a_game_exactly_as_the_shim()
        {
            string rom = Path.Combine(_root, "busy.nes");
            File.WriteAllBytes(rom, MoonPrograms.Rom(4, 8, (0, MoonPrograms.Busy)));
            using var shim = new MoonRtCore();
            using var adapter = new CoreEngine(Discovered.Open()!);
            shim.LoadRom(rom);
            adapter.LoadRom(rom);
            Assert.Equal((shim.ScreenWidth, shim.ScreenHeight, shim.StateVersion), (adapter.ScreenWidth, adapter.ScreenHeight, adapter.StateVersion));
            Assert.Equal(shim.FrameRateHz, adapter.FrameRateHz, 9);
            Assert.Equal(shim.SupportedButtons, adapter.SupportedButtons);
            for (int f = 0; f < 240; f++)
            {
                foreach (ICore core in new ICore[] { shim, adapter })
                {
                    core.SetButton(0, PadButton.Start, f % 60 < 4);
                    core.SetButton(1, PadButton.A, f % 30 < 2);
                    core.RunFrame();
                }
                Assert.Equal(shim.GetFrameBufferRgba(), adapter.GetFrameBufferRgba());
                Assert.Equal(shim.DequeueAudioSamples(int.MaxValue), adapter.DequeueAudioSamples(int.MaxValue));
            }
            var a = new MemoryStream();
            var b = new MemoryStream();
            shim.SaveState(a);
            adapter.SaveState(b);
            Assert.Equal(a.ToArray(), b.ToArray());
            adapter.Cheats.AddRamPoke("RAM", 0x10, 0x5A, "poke");
            adapter.RunFrame();
            Assert.Equal(0x5A, adapter.ReadSpace("RAM", 0x10));
        }
    }
}
