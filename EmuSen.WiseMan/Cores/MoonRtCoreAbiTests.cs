using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // MoonRT on the core ABI v1 alone: its sidecar from the build, the factory's one engine for the NES, and the v1 adapter running it exactly as its bare machine does - see EmuSen_CoreAPI.md §22, EmuSen_Backend.md §23.
    [Collection(TestCollections.ProcessGlobals)]
    public class MoonRtCoreAbiTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMoonRtV1_" + Guid.NewGuid().ToString("N"));

        public MoonRtCoreAbiTests()
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

        private static DiscoveredCore Discovered => CoreDiscovery.Found.Single(c => c.Info.Id == "moonrt");

        [Fact]
        public void The_build_writes_moonrts_sidecar_and_discovery_lists_and_opens_it()
        {
            Assert.Equal("MoonRT (Rust)", Discovered.EngineName);
            Assert.Equal(new[] { ".nes" }, Discovered.Info.Systems.Single().Extensions);
            var library = Discovered.Open();
            Assert.True(library is { Available: true }, Discovered.Report);
            Assert.Equal(CoreInterface.CapReset | CoreInterface.CapMutes | CoreInterface.CapRomPatches | CoreInterface.CapDebug | CoreInterface.CapDebugStack | CoreInterface.CapRawPicture
                | CoreInterface.CapDebugRegisters | CoreInterface.CapDebugDisassemble, library!.Capabilities);
        }

        // Since its shim retired the NES has one discovered engine and no row; any name asked for, the retired Moon (C#) among them, builds MoonRT on the generic engine.
        [Fact]
        public void The_nes_has_one_engine_and_the_factory_builds_moonrt_whatever_name_is_asked_for()
        {
            _ = Discovered;
            Assert.False(CoreCatalog.IsRegisteredEngine(CoreCatalog.MoonRtEngine));
            Assert.Null(CoreCatalog.EngineFor("NES"));
            Assert.Equal(CoreCatalog.MoonRtEngine, CoreCatalog.EngineChosen("NES", "Moon (C#)"));
            foreach (string? engine in new[] { null, CoreCatalog.MoonRtEngine, "Moon (C#)", "Nope" })
            {
                using var core = Assert.IsType<CoreEngine>(CoreFactory.Create("game.nes", engine: engine, options: TestCores.NoBattery));
                Assert.Equal("moonrt", core.Info.Id);
                Assert.Null(CoreFactory.EngineNotice("game.nes", engine, core));
            }
        }

        // A save beside a game whose header claims no battery reaches no machine, as the shim never opened one - see EmuSen_Backend.md §19.6, C1.
        [Fact]
        public void A_battery_file_beside_a_game_without_one_is_not_loaded()
        {
            string rom = Path.Combine(_root, "plain.nes");
            File.WriteAllBytes(rom, SyntheticNesRom.Build(prgBanks: 2, chrBanks: 1, battery: false));
            string save = SaveLibrary.SramPathFor(rom, BatterySave.Nes);
            Directory.CreateDirectory(Path.GetDirectoryName(save)!);
            File.WriteAllBytes(save, Enumerable.Repeat((byte)0xA5, 0x2000).ToArray());
            using var core = (CoreEngine)CoreFactory.Loaded(rom);
            Assert.Equal(0, core.ReadSpace("PRGRAM", 0x10));
        }

        // The debugger's fixed map of an address is the NES pack's for every address, through the generic target - see EmuSen_Backend.md §21, §23.
        [Fact]
        public void The_targets_fixed_map_is_the_nes_packs()
        {
            string rom = Path.Combine(_root, "plain.nes");
            File.WriteAllBytes(rom, SyntheticNesRom.Build());
            CoreBundle bundle = CoreFactory.Load(rom, options: TestCores.NoBattery);
            using var core = (CoreEngine)bundle.Core;
            for (int a = -1; a <= 0x10000; a++)
            {
                var (target, pack) = (bundle.DebugTarget.ResolvePhysical(a), EmuSen.Crystal.Systems.Nes.NesDebugViews.Instance.ResolvePhysical(a));
                Assert.Equal((pack?.Space, pack?.Offset, pack?.IsAddressable), (target?.Space, target?.Offset, target?.IsAddressable));
            }
        }

        // The generic engine and the bare v1 machine under it: the same picture, sound and state, frame for frame, so the adapter adds nothing to what the core makes.
        [Fact]
        public void The_v1_adapter_runs_moonrt_exactly_as_its_bare_machine()
        {
            string rom = Path.Combine(_root, "busy.nes");
            byte[] image = MoonPrograms.Rom(4, 8, (0, MoonPrograms.Busy));
            File.WriteAllBytes(rom, image);
            using var adapter = new CoreEngine(Discovered.Open()!) { Options = TestCores.NoBattery };
            adapter.LoadRom(rom);
            using var bare = MoonEngines.Machine(image);
            var picture = new byte[bare.FrameInfo.Bytes];
            for (int f = 0; f < 240; f++)
            {
                uint mask = f % 60 < 4 ? 1u << 3 : 0;
                adapter.SetButton(0, PadButton.Start, mask != 0);
                bare.SetButtons(0, mask, 0xFF);
                adapter.RunFrame();
                bare.Advance();
                bare.CopyFrame(picture);
                Assert.True(picture.AsSpan().SequenceEqual(adapter.GetFrameBufferRgba()), $"the picture differs at frame {f + 1}");
                Assert.Equal(bare.DrainAudio(int.MaxValue).Samples, adapter.DequeueAudioSamples(int.MaxValue));
            }
            var state = new MemoryStream();
            adapter.SaveState(state);
            Assert.True(MoonEngines.Save(bare).AsSpan().SequenceEqual(state.ToArray()), "the states differ");
            adapter.Cheats.AddRamPoke("RAM", 0x10, 0x5A, "poke");
            adapter.RunFrame();
            Assert.Equal(0x5A, adapter.ReadSpace("RAM", 0x10));
        }
    }
}
