using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json.Nodes;
using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.VenusRT;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Fixtures.Snes;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // VenusRT on the core ABI v1 through discovery: the SNES row, the factory and its fallback, the generic adapter against the shim, and battery saves both ways with C# Venus - see VenusRT_Native.md §33.
    [Collection(TestCollections.ProcessGlobals)]
    public class VenusRtCoreAbiTests : IDisposable
    {
        public const string Engine = "VenusRT (Rust)";
        public const string BatteryGamesVariable = "EMUSEN_VENUSRT_BATTERY_GAMES";

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenVenusRtV1_" + Guid.NewGuid().ToString("N"));
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;
        private readonly ITestOutputHelper _output;

        public VenusRtCoreAbiTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            FirmwareLibrary.Directory = Path.Combine(_root, "Firmware");
            Directory.CreateDirectory(FirmwareLibrary.Directory);
            CoreOptions.BatteryRamDisabled = true;
            CoreDiscovery.UseDirectories(null);
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            DataStore.OverrideDirectory = null;
            FirmwareLibrary.ResetDirectory();
            CoreDiscovery.UseDirectories(null);
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static DiscoveredCore? Discovered => CoreDiscovery.Found.SingleOrDefault(c => c.Info.Id == "venusrt");

        // The firmware a run needs in the test's firmware folder: the stand-in or the real boot ROM, and the DSP images whole from the corpus's split pairs.
        private void InstallFirmware(byte[] ipl)
        {
            File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, "spc700.rom"), ipl);
            if (SnesTestRomCorpus.Root is not { } root || !Directory.Exists(Path.Combine(root, "firmware"))) return;
            foreach (string program in Directory.GetFiles(Path.Combine(root, "firmware"), "*.program.rom"))
            {
                string stem = program[..^".program.rom".Length], data = stem + ".data.rom";
                if (File.Exists(data)) File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, Path.GetFileName(stem) + ".rom"), File.ReadAllBytes(program).Concat(File.ReadAllBytes(data)).ToArray());
            }
        }

        // A LoROM image that brightens the screen, then copies pad 1's high byte into the backdrop's colour forever.
        public static byte[] PadToBackdropRom()
        {
            byte[] program = { 0xA9, 0x0F, 0x8D, 0x00, 0x21, 0xAD, 0x19, 0x42, 0x8D, 0x22, 0x21, 0x8D, 0x22, 0x21, 0xEE, 0x10, 0x00, 0x80, 0xF2 };
            return SyntheticRom.Build((5, program), (0x7FC0, "VENUSRT ADAPTER TEST "u8.ToArray()), (0x7FD5, new byte[] { 0x20 }));
        }

        [Fact]
        public void The_build_writes_venusrts_sidecar_and_discovery_lists_and_opens_it()
        {
            if (Discovered is not { } found)
            {
                Assert.False(VenusNative.Available, "VenusRT's library is beside the tests without its sidecar");
                return;
            }
            Assert.Equal(Engine, found.EngineName);
            Assert.Equal(new[] { ".smc", ".sfc" }, found.Info.Systems.Single().Extensions);
            var library = found.Open();
            Assert.True(library is { Available: true }, found.Report);
            Assert.Equal(CoreInterface.CapReset | CoreInterface.CapSnapshot | CoreInterface.CapBatteryDirty | CoreInterface.CapRomPatches | CoreInterface.CapCheatPokes, library!.Capabilities);
        }

        [Fact]
        public void The_snes_row_offers_venusrt_through_discovery_with_venus_the_default()
        {
            if (Discovered is null) return;
            Assert.False(CoreCatalog.IsRegisteredEngine(Engine));
            var row = CoreCatalog.EngineFor("SNES")!;
            Assert.Equal(new[] { CoreCatalog.VenusEngine, Engine }, row.Choices);
            Assert.Equal(CoreCatalog.VenusEngine, row.Default);
            Assert.IsType<VenusCore>(CoreFactory.Create("game.sfc"));
            using var engine = Assert.IsType<CoreEngine>(CoreFactory.Create("game.sfc", engine: Engine));
            Assert.Equal("venusrt", engine.Info.Id);
            Assert.Null(CoreFactory.EngineNotice("game.sfc", Engine, engine));
        }

        [Fact]
        public void Without_its_library_or_with_another_one_venusrt_falls_back_to_venus_with_a_notice()
        {
            if (Discovered is not { } found) return;
            CoreDiscovery.UseDirectories(new[] { Path.Combine(_root, "nothing") });
            Assert.Null(CoreCatalog.EngineFor("SNES"));
            ICore missing = CoreFactory.Create("game.sfc", engine: Engine);
            Assert.IsType<VenusCore>(missing);

            string dir = Path.Combine(_root, "swapped");
            Directory.CreateDirectory(dir);
            string library = Path.Combine(dir, Path.GetFileName(found.Sidecar.LibraryPath));
            File.WriteAllBytes(library, File.ReadAllBytes(found.Sidecar.LibraryPath).Concat(new byte[] { 0 }).ToArray());
            File.Copy(CoreSidecar.PathFor(found.Sidecar.LibraryPath), CoreSidecar.PathFor(library));
            CoreDiscovery.UseDirectories(new[] { dir });
            Assert.Equal(new[] { CoreCatalog.VenusEngine, Engine }, CoreCatalog.EngineFor("SNES")!.Choices);
            ICore swapped = CoreFactory.Create("game.sfc", engine: Engine);
            Assert.IsType<VenusCore>(swapped);
            string notice = CoreFactory.EngineNotice("game.sfc", Engine, swapped)!;
            Assert.StartsWith($"{Engine} is not available (", notice);
            Assert.Contains("its SHA-256 differs", notice);
            Assert.EndsWith("Venus (C#) is running.", notice);
            Assert.False(CoreLibrary.IsOpen(library));
        }

        // The generic engine over VenusRT's v1 exports and the shim over its pre-stable ones: the same picture, sound and state, frame for frame.
        [Fact]
        public void The_v1_adapter_runs_venusrt_exactly_as_the_shim()
        {
            if (Discovered is not { } found) return;
            byte[] ipl = VenusRtTestRomRunnerTests.IdleIpl();
            InstallFirmware(ipl);
            string rom = Path.Combine(_root, "adapter.sfc");
            File.WriteAllBytes(rom, PadToBackdropRom());
            using var adapter = new CoreEngine(found.Open()!);
            adapter.LoadRom(rom);
            using var shim = new VenusMachine(File.ReadAllBytes(rom), ipl);
            Assert.True(Compare(adapter, shim, 240) > 0);
            adapter.Cheats.AddRamPoke("WRAM", 0x20, 0x5A, "poke");
            adapter.RunFrame();
            Assert.Equal(0x5A, adapter.ReadSpace("WRAM", 0x20));
        }

        // Frame by frame with pad 1 pressing in turn: the picture and the sound each frame, the state at the end; how many frames showed a picture.
        private static int Compare(CoreEngine adapter, VenusMachine shim, int frames)
        {
            int lit = 0;
            Assert.Equal(shim.AudioSampleRate, adapter.AudioSampleRate);
            var picture = new byte[0];
            for (int f = 0; f < frames; f++)
            {
                uint mask = (uint)(1 << (f / 8 % 12));
                for (int b = 0; b < 12; b++) adapter.SetButton(0, (PadButton)b, (mask & (1u << b)) != 0);
                shim.SetButtons(0, mask, 0xFFF);
                adapter.RunFrame();
                shim.Advance();
                if (picture.Length != (int)shim.FrameInfo.Bytes) picture = new byte[(int)shim.FrameInfo.Bytes];
                shim.CopyFrame(picture);
                Assert.True(picture.AsSpan().SequenceEqual(adapter.GetFrameBufferRgba()), $"the picture differs at frame {f + 1}");
                if (picture.Where((_, i) => i % 4 != 3).Any(b => b != 0)) lit++;
                Assert.Equal(shim.DrainAudio(int.MaxValue), adapter.DequeueAudioSamples(int.MaxValue));
            }
            var state = new MemoryStream();
            adapter.SaveState(state);
            Assert.True(shim.Save().AsSpan().SequenceEqual(state.ToArray()), "the states differ");
            return lit;
        }

        // Opt-in: each game in EMUSEN_VENUSRT_GAMES through ICore on the adapter, as the shim runs it with the same firmware, for 600 frames.
        [Fact]
        public void Commercial_games_run_on_the_adapter_as_on_the_shim()
        {
            string? dir = Environment.GetEnvironmentVariable("EMUSEN_VENUSRT_GAMES");
            if (dir is null || Discovered is not { } found || VenusRtSnesEngine.Ipl() is not { } ipl) return;
            InstallFirmware(ipl);
            foreach (string source in Directory.GetFiles(dir).Order(StringComparer.Ordinal))
            {
                string rom = Path.Combine(_root, Path.GetFileName(source));
                File.Copy(source, rom, overwrite: true);
                using var adapter = new CoreEngine(found.Open()!);
                adapter.LoadRom(rom);
                using var shim = new VenusMachine(File.ReadAllBytes(rom), ipl, dspFirmware: VenusRtSnesEngine.DspFirmware(rom));
                int lit = Compare(adapter, shim, 600);
                Assert.True(lit > 0, $"{Path.GetFileName(rom)} shows nothing in 600 frames");
                _output.WriteLine($"{Path.GetFileName(rom)}: 600 frames equal, {lit} with a picture");
            }
        }

        // Opt-in: for each game in EMUSEN_VENUSRT_BATTERY_GAMES, the .srm Venus (C#) writes read whole by VenusRT, and the one VenusRT writes read whole by Venus.
        [Fact]
        public void A_battery_save_crosses_between_venus_and_venusrt_both_ways()
        {
            string? dir = Environment.GetEnvironmentVariable(BatteryGamesVariable);
            if (dir is null || Discovered is null || VenusRtSnesEngine.Ipl() is not { } ipl) return;
            InstallFirmware(ipl);
            CoreOptions.BatteryRamDisabled = false;
            foreach (string source in Directory.GetFiles(dir).Order(StringComparer.Ordinal))
            {
                string rom = Path.Combine(_root, Path.GetFileName(source));
                File.Copy(source, rom, overwrite: true);
                string save = BatterySave.Open(rom, BatterySave.Snes).Path!;
                Directory.CreateDirectory(Path.GetDirectoryName(save)!);
                File.Delete(save);

                byte[] venusFresh = VenusWrites(rom, save);
                byte[] pattern = Enumerable.Range(0, venusFresh.Length).Select(i => (byte)(i * 7 + 3)).ToArray();
                using (var fresh = (CoreEngine)CoreFactory.Load(rom, engine: Engine).Core) Assert.Equal(venusFresh.Length, fresh.Machine.Battery(0).Data.Length);

                File.WriteAllBytes(save, pattern);
                Assert.Equal(pattern, VenusWrites(rom, save));
                byte[] written;
                using (var venusRt = (CoreEngine)CoreFactory.Load(rom, engine: Engine).Core)
                {
                    Assert.Equal(pattern, venusRt.Machine.Battery(0).Data);
                    foreach (int at in new[] { 0, pattern.Length / 2, pattern.Length - 1 }) venusRt.WriteSpace("SRAM", at, (byte)~pattern[at]);
                    venusRt.RunFrame();
                    venusRt.SaveSram();
                    written = File.ReadAllBytes(save);
                    Assert.Equal(venusRt.Machine.Battery(0).Data, written);
                }
                Assert.NotEqual(pattern, written);
                Assert.Equal(written, VenusWrites(rom, save));
                _output.WriteLine($"{Path.GetFileName(rom)}: {pattern.Length} bytes both ways");
            }
        }

        // What Venus (C#) writes from the save it loaded: the file removed after the load, so the bytes are its own.
        private static byte[] VenusWrites(string rom, string save)
        {
            ICore venus = CoreFactory.Load(rom).Core;
            Assert.IsType<VenusCore>(venus);
            File.Delete(save);
            venus.SaveSram();
            (venus as IDisposable)?.Dispose();
            return File.ReadAllBytes(save);
        }
    }
}
