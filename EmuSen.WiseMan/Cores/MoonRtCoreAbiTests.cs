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
                ICore core = CoreFactory.Create("game.nes", engine: engine, options: TestCores.NoBattery);
                Assert.IsType<MoonRtCore>(core);
                Assert.Null(CoreFactory.EngineNotice("game.nes", engine, core));
                ((IDisposable)core).Dispose();
            }
        }

        // A save beside a game whose header claims no battery reaches no machine on the adapter, as the shim never opened one - see EmuSen_Backend.md §19.6, C1.
        [Fact]
        public void A_battery_file_beside_a_game_without_one_is_not_loaded_on_the_adapter()
        {
            string rom = Path.Combine(_root, "plain.nes");
            File.WriteAllBytes(rom, SyntheticNesRom.Build(prgBanks: 2, chrBanks: 1, battery: false));
            string save = SaveLibrary.SramPathFor(rom, BatterySave.Nes);
            Directory.CreateDirectory(Path.GetDirectoryName(save)!);
            File.WriteAllBytes(save, Enumerable.Repeat((byte)0xA5, 0x2000).ToArray());
            using var shim = new MoonRtCore();
            using var adapter = new CoreEngine(Discovered.Open()!);
            shim.LoadRom(rom);
            adapter.LoadRom(rom);
            Assert.Equal(0, shim.ReadSpace("PRGRAM", 0x10));
            Assert.Equal(0, adapter.ReadSpace("PRGRAM", 0x10));
        }

        // The 2A03's registers through DEBUG_REGISTERS and its disassembly through DEBUG_DISASSEMBLE, on the generic target, are what the shim's target shows - see EmuSen_Backend.md §21.
        [Fact]
        public void The_cores_registers_and_disassembly_are_what_the_shims_target_shows()
        {
            string rom = Path.Combine(_root, "busy.nes");
            File.WriteAllBytes(rom, MoonPrograms.Rom(4, 8, (0, MoonPrograms.Busy)));
            using var shim = new MoonRtCore() { Options = TestCores.NoBattery };
            using var adapter = new CoreEngine(Discovered.Open()!) { Options = TestCores.NoBattery };
            shim.LoadRom(rom);
            adapter.LoadRom(rom);
            var theirs = shim.CreateDebugTarget();
            var ours = adapter.CreateDebugTarget();
            for (int f = 0; f < 90; f++)
            {
                shim.RunFrame();
                adapter.RunFrame();
                if (f % 30 != 29) continue;
                theirs.RefreshProviders();
                ours.RefreshProviders();
                Assert.Equal(theirs.CpuRegisters.Current.Select(r => (r.Name, r.Value, r.BitWidth)), ours.CpuRegisters.Current.Select(r => (r.Name, r.Value, r.BitWidth)));
            }
            foreach (int start in new[] { 0x8000, 0x8015, 0xC000, 0xFFF0, 0x0000, 0x6000 })
            {
                var a = theirs.Disassemble("CPUBUS", start, 64);
                var b = ours.Disassemble("CPUBUS", start, 64);
                Assert.Equal(a.Select(i => (i.Address, string.Join(" ", i.Bytes), i.Mnemonic, i.OperandText, theirs.ClassifyStaticReference(i))),
                    b.Select(i => (i.Address, string.Join(" ", i.Bytes), i.Mnemonic, i.OperandText, ours.ClassifyStaticReference(i))));
            }
        }

        // Every board's views on the generic target over MoonRT's units and the NES pack are the shim target's, but for the board's mapper row and the summary's head - see EmuSen_Backend.md §23.
        [Theory]
        [MemberData(nameof(MoonPrograms.Boards), MemberType = typeof(MoonPrograms))]
        public void The_units_and_the_pack_show_every_board_as_the_shims_target(int mapper, int chr)
        {
            string rom = Path.Combine(_root, $"board-{mapper}-{chr}.nes");
            File.WriteAllBytes(rom, BoardPrograms.Build(mapper, chr, cycleIrq: mapper is 64 or 65 or 67 or 69));
            using var shim = new MoonRtCore() { Options = TestCores.NoBattery };
            using var adapter = new CoreEngine(Discovered.Open()!) { Options = TestCores.NoBattery };
            shim.LoadRom(rom);
            adapter.LoadRom(rom);
            var theirs = shim.CreateDebugTarget();
            var ours = adapter.CreateDebugTarget();
            static string Rows(IReadOnlyList<EmuSen.Crystal.Telemetry.DebugRegisterValue> r) => string.Join("\n", r.Where(v => v.Name != "Mapper").Select(v => $"{v.Name} {v.Value:X} {v.BitWidth}"));
            Assert.Equal(theirs.MaxSprites, ours.MaxSprites);
            for (int f = 1; f <= 120; f++)
            {
                shim.SetButton(0, PadButton.Start, f % 40 < 3);
                adapter.SetButton(0, PadButton.Start, f % 40 < 3);
                shim.RunFrame();
                adapter.RunFrame();
                if (f % 30 != 0) continue;
                theirs.RefreshProviders();
                ours.RefreshProviders();
                Assert.Equal(Rows(theirs.CpuRegisters.Current), Rows(ours.CpuRegisters.Current));
                Assert.Equal(Rows(theirs.VideoRegisters.Current), Rows(ours.VideoRegisters.Current));
                Assert.Equal(Rows(theirs.ApuRegisters.Current), Rows(ours.ApuRegisters.Current));
                Assert.Equal(theirs.Sprites.Current.Select(s => (s.Index, s.X, s.Y, s.Width, s.Height, s.TileIndex, s.PaletteIndex, s.Priority, s.FlipX, s.FlipY)),
                    ours.Sprites.Current.Select(s => (s.Index, s.X, s.Y, s.Width, s.Height, s.TileIndex, s.PaletteIndex, s.Priority, s.FlipX, s.FlipY)));
                Assert.Equal(theirs.AudioChannels.Current.Select(c => (c.Index, c.Name, c.Active, c.Level, c.Muted, c.Info)), ours.AudioChannels.Current.Select(c => (c.Index, c.Name, c.Active, c.Level, c.Muted, c.Info)));
                string[] a = theirs.GetSummaryText().Split('\n'), b = ours.GetSummaryText().Split('\n');
                string scanline = a[0][(a[0].LastIndexOf("scanline ", StringComparison.Ordinal) + "scanline ".Length)..];
                Assert.Equal(a[1..5], b[1..5].Select((l, i) => i == 3 ? l.Replace($" scanline={scanline}", "") : l));
            }
        }

        // The NES pack's fixed memory map, for any engine's target once the views carry it, is the shim target's for every address - see EmuSen_Backend.md §21.
        [Fact]
        public void The_nes_packs_fixed_map_is_the_shim_targets()
        {
            string rom = Path.Combine(_root, "plain.nes");
            File.WriteAllBytes(rom, SyntheticNesRom.Build());
            using var shim = new MoonRtCore() { Options = TestCores.NoBattery };
            shim.LoadRom(rom);
            var target = shim.CreateDebugTarget();
            for (int a = -1; a <= 0x10000; a++)
            {
                var (theirs, ours) = (target.ResolvePhysical(a), EmuSen.Crystal.Systems.Nes.NesDebugViews.Instance.ResolvePhysical(a));
                Assert.Equal((theirs?.Space, theirs?.Offset, theirs?.IsAddressable), (ours?.Space, ours?.Offset, ours?.IsAddressable));
            }
        }

        [Fact]
        public void The_v1_adapter_over_moonrt_runs_a_game_exactly_as_the_shim()
        {
            string rom = Path.Combine(_root, "busy.nes");
            File.WriteAllBytes(rom, MoonPrograms.Rom(4, 8, (0, MoonPrograms.Busy)));
            using var shim = new MoonRtCore() { Options = TestCores.NoBattery };
            using var adapter = new CoreEngine(Discovered.Open()!) { Options = TestCores.NoBattery };
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
