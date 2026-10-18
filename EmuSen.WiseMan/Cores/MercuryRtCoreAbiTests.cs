using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // MercuryRT on the core ABI v1 beside its pre-stable exports: its sidecar, the shim still its loader, and the v1 adapter running it exactly as the shim does - see EmuSen_CoreAPI.md §23.
    [Collection(TestCollections.ProcessGlobals)]
    public class MercuryRtCoreAbiTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMercuryRtV1_" + Guid.NewGuid().ToString("N"));

        public MercuryRtCoreAbiTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDirectories(null);
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            CoreDiscovery.UseDirectories(null);
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static DiscoveredCore Discovered => CoreDiscovery.Found.Single(c => c.Info.Id == "mercuryrt");

        private string Rom(string name, byte cgb, byte kind = 0x13)
        {
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, SyntheticGbRom.Build(romBanks: 4, cartridgeType: kind, ramSizeCode: 0x03, cgbFlag: cgb, patches: (0, MercuryPrograms.Busy)));
            return path;
        }

        [Fact]
        public void The_build_writes_mercuryrts_sidecar_and_discovery_lists_and_opens_it()
        {
            Assert.Equal("MercuryRT (Rust)", Discovered.EngineName);
            Assert.Equal(new[] { "gb", "gbc" }, Discovered.Info.Systems.Select(s => s.Id));
            var library = Discovered.Open();
            Assert.True(library is { Available: true }, Discovered.Report);
            Assert.Equal(CoreInterface.CapAudioPeek | CoreInterface.CapMutes | CoreInterface.CapRomPatches | CoreInterface.CapDebug | CoreInterface.CapDebugStack | CoreInterface.CapDebugRegisters | CoreInterface.CapDebugDisassemble, library!.Capabilities);
            Assert.Equal(new[] { "Model" }, library.Settings.Select(s => s.Key));
        }

        [Fact]
        public void The_shim_stays_mercuryrts_loader_and_the_game_boy_has_no_engine_row()
        {
            _ = Discovered;
            Assert.Null(CoreCatalog.EngineFor("GB"));
            Assert.True(CoreCatalog.IsRegisteredEngine(CoreCatalog.MercuryRtEngine));
            Assert.IsType<MercuryRtCore>(CoreFactory.Create("game.gb", options: TestCores.NoBattery));
            ICore core = CoreFactory.Create("game.gb", engine: CoreCatalog.MercuryRtEngine, options: TestCores.NoBattery);
            Assert.IsType<MercuryRtCore>(core);
            Assert.Null(CoreFactory.EngineNotice("game.gb", CoreCatalog.MercuryRtEngine, core));
        }

        // A player's stored choice of the C# core runs the game on MercuryRT, with no notice and at most one line in the log - see Mercury_Native.md §15.4.
        [Fact]
        public void A_stored_choice_of_the_retired_csharp_core_falls_back_quietly()
        {
            const string retired = "Mercury (C#)";
            Assert.False(CoreCatalog.IsRegisteredEngine(retired));
            string rom = Rom("stored.gb", 0x00);
            string? logsWere = ErrorLog.DirectoryOverride;
            string logs = Path.Combine(_root, "Logs");
            ErrorLog.DirectoryOverride = logs;
            try
            {
                string? engine = CoreCatalog.EngineChosen("GB", retired);
                Assert.Equal(CoreCatalog.MercuryRtEngine, engine);
                Assert.Equal(engine, CoreCatalog.EngineChosen("GB", retired));
                CoreBundle bundle = CoreFactory.Load(rom, engine: engine, options: TestCores.NoBattery);
                Assert.IsType<MercuryRtCore>(bundle.Core);
                Assert.Null(bundle.Notice);
                (bundle.Core as IDisposable)?.Dispose();
                string[] lines = Directory.Exists(logs) ? Directory.GetFiles(logs).SelectMany(File.ReadAllLines).Where(l => l.Contains(retired + " was retired")).ToArray() : Array.Empty<string>();
                Assert.True(lines.Length <= 1, string.Join("\n", lines));
            }
            finally
            {
                ErrorLog.DirectoryOverride = logsWere;
            }
        }

        // On a Game Boy and on a Game Boy Color, and with the Model setting forcing the colour console onto a Game Boy game.
        [Theory]
        [InlineData(0x00, "Auto")]
        [InlineData(0x80, "Auto")]
        [InlineData(0x00, "Game Boy Color")]
        public void The_v1_adapter_over_mercuryrt_runs_a_game_exactly_as_the_shim(byte cgb, string model)
        {
            string rom = Rom("busy.gb", cgb);
            using var shim = new MercuryRtCore() { Options = TestCores.NoBattery };
            using var adapter = new CoreEngine(Discovered.Open()!, new System.Collections.Generic.Dictionary<string, string> { ["Model"] = model }) { Options = TestCores.NoBattery };
            ((ICoreSettings)shim).Set("Model", model);
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
                    core.SetButton(0, PadButton.A, f % 30 < 2);
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
            Assert.Equal(model == "Auto" && cgb == 0 ? "gb" : "gbc", adapter.Machine.Info.System);
        }

        // A state made on the other console changes machine info's system, and the core says so with MACHINE_INFO - see EmuSen_CoreAPI.md §26.
        [Fact]
        public void A_state_from_the_other_console_tells_the_host_its_machine_info_changed()
        {
            string rom = Rom("busy.gb", 0x00);
            using var colour = new CoreEngine(Discovered.Open()!, new System.Collections.Generic.Dictionary<string, string> { ["Model"] = "Game Boy Color" }) { Options = TestCores.NoBattery };
            using var mono = new CoreEngine(Discovered.Open()!) { Options = TestCores.NoBattery };
            colour.LoadRom(rom);
            mono.LoadRom(rom);
            for (int f = 0; f < 30; f++)
            {
                colour.RunFrame();
                mono.RunFrame();
            }
            Assert.Equal(("gbc", "gb"), (colour.Machine.Info.System, mono.Machine.Info.System));
            var state = new MemoryStream();
            colour.SaveState(state);
            mono.LoadState(new MemoryStream(state.ToArray()));
            Assert.Equal("gbc", mono.Machine.Info.System);
            state.SetLength(0);
            using var other = new CoreEngine(Discovered.Open()!) { Options = TestCores.NoBattery };
            other.LoadRom(rom);
            other.SaveState(state);
            colour.LoadState(new MemoryStream(state.ToArray()));
            Assert.Equal("gb", colour.Machine.Info.System);
        }

        // A colour game's battery save is under GBC on the adapter as on the shim, the system's folder and not the shelf's.
        [Fact]
        public void A_colour_games_battery_save_goes_to_its_systems_folder()
        {
            string rom = Rom("colour.gbc", 0x80);
            using (var adapter = new CoreEngine(Discovered.Open()!))
            {
                adapter.LoadRom(rom);
                adapter.WriteSpace("CARTRAM", 0, 0x5A);
                adapter.SaveSram();
            }
            string save = Path.Combine(DataStore.Saves, "GBC", "colour.srm");
            Assert.True(File.Exists(save), save);
            Assert.Equal(0x5A, File.ReadAllBytes(save)[0]);
        }
    }
}
