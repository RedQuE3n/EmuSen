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

        // The firmware a run needs in the test's firmware folder: the DSP images whole from the corpus's split pairs; nothing for the SPC700 (D-38).
        private static void InstallFirmware()
        {
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
            Assert.Equal(CoreInterface.CapReset | CoreInterface.CapSnapshot | CoreInterface.CapBatteryDirty | CoreInterface.CapRomPatches | CoreInterface.CapCheatPokes
                | CoreInterface.CapDebug | CoreInterface.CapDebugStack | CoreInterface.CapDebugRegisters | CoreInterface.CapDebugDisassemble, library!.Capabilities);
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

        // VenusRT_DspHle.md §7.3: a DSP-1 cartridge and an empty firmware folder; nothing is missing, the game is created on the replacement, its state carries the replacement's tag, machine info says so, and the notice names the cost.
        [Fact]
        public void A_dsp_cartridge_runs_on_the_replacement_with_no_firmware()
        {
            if (Discovered is not { } found) return;
            string rom = Path.Combine(_root, "kart.sfc");
            File.WriteAllBytes(rom, SyntheticRom.BuildNecDsp("SUPER MARIO KART"));
            Assert.Empty(Directory.GetFiles(FirmwareLibrary.Directory).Where(f => f.EndsWith(".rom")));
            Assert.Empty(EmuSen.Common.EmulatorSession.MissingFirmwareFor(rom, Engine));
            using var engine = new CoreEngine(found.Open()!);
            engine.LoadRom(rom);
            Assert.Equal(new[] { new CoreFirmwareSource(1, "replacement"), new CoreFirmwareSource(2, "replacement") }, engine.Machine.Info.Firmware);
            Assert.DoesNotContain(engine.Machine.Info.Processors, p => p.Name == "DSP");
            Assert.Contains("Coprocessor.DspHle", engine.Machine.Layout(0));
            Assert.StartsWith("VenusRT's open replacement for dsp1b.rom - Without the image", engine.FirmwareNotice);
            string gear = Path.Combine(_root, "gear.sfc");
            File.WriteAllBytes(gear, SyntheticRom.BuildNecDsp("TOP GEAR 3000"));
            Assert.Empty(EmuSen.Common.EmulatorSession.MissingFirmwareFor(gear, Engine));
            engine.LoadRom(gear);
            Assert.Equal(new[] { new CoreFirmwareSource(1, "replacement"), new CoreFirmwareSource(2, "replacement") }, engine.Machine.Info.Firmware);
            string gundam = Path.Combine(_root, "gundam.sfc");
            File.WriteAllBytes(gundam, SyntheticRom.BuildNecDsp("SD GUNDAM GX"));
            Assert.Empty(EmuSen.Common.EmulatorSession.MissingFirmwareFor(gundam, Engine));
            engine.LoadRom(gundam);
            Assert.Equal(new[] { new CoreFirmwareSource(1, "replacement"), new CoreFirmwareSource(2, "absent") }, engine.Machine.Info.Firmware);
            Assert.EndsWith("dsp3.rom would supply the chip.", engine.FirmwareNotice);
        }

        // EmuSen_Firmware.md §0: a player's spc700.rom in the folder is passed as file 1 and runs in place of VenusRT's own boot program; a synthetic image, never a dump.
        [Fact]
        public void A_players_boot_image_runs_in_place_of_the_open_program()
        {
            if (Discovered is not { } found) return;
            string rom = Path.Combine(_root, "plain.sfc");
            File.WriteAllBytes(rom, SyntheticRom.BuildNecDsp("PLAIN", cartType: 0x00));
            using var engine = new CoreEngine(found.Open()!);
            var request = engine.GetFirmwareRequirements(rom).Single();
            Assert.Equal(("spc700.rom", 64, false, "accuracy"), (request.FileName, request.Size, request.Required, request.ReplacementEffect));
            engine.LoadRom(rom);
            Assert.Equal(new[] { new CoreFirmwareSource(1, "replacement") }, engine.Machine.Info.Firmware);
            Assert.StartsWith("VenusRT's open replacement for spc700.rom", engine.FirmwareNotice);
            byte[] image = new byte[64];
            (image[0], image[1], image[62], image[63]) = (0x2F, 0xFE, 0xC0, 0xFF);
            File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, "spc700.rom"), image);
            engine.LoadRom(rom);
            Assert.Equal(new[] { new CoreFirmwareSource(1, "file") }, engine.Machine.Info.Firmware);
            Assert.Null(engine.FirmwareNotice);
            Assert.Empty(EmuSen.Common.EmulatorSession.MissingFirmwareFor(rom, Engine));
        }

        // VenusRT_Plan.md §4.5: a state C# Venus wrote is refused in words naming Venus (C#), and the machine is unchanged.
        [Fact]
        public void A_venus_state_is_refused_naming_the_engine_that_made_it()
        {
            if (Discovered is not { } found) return;
            string rom = Path.Combine(_root, "plain.sfc");
            File.WriteAllBytes(rom, SyntheticRom.Build());
            ICore venus = CoreFactory.Create(rom);
            venus.LoadRom(rom);
            for (int i = 0; i < 30; i++) venus.RunFrame();
            using var state = new MemoryStream();
            venus.SaveState(state);
            (venus as IDisposable)?.Dispose();
            using var engine = new CoreEngine(found.Open()!);
            engine.LoadRom(rom);
            for (int i = 0; i < 30; i++) engine.RunFrame();
            using var before = new MemoryStream();
            engine.SaveState(before);
            var refused = Assert.Throws<InvalidDataException>(() => engine.LoadState(new MemoryStream(state.ToArray())));
            Assert.Equal("VenusRT (Rust) refused the state: it was saved by Venus (C#), the C# SNES engine, whose states VenusRT cannot read.", refused.Message);
            using var after = new MemoryStream();
            engine.SaveState(after);
            Assert.Equal(before.ToArray(), after.ToArray());
            byte[] other = before.ToArray();
            other[0] ^= 1;
            Assert.Equal("VenusRT (Rust) refused the state: not this core's state.", Assert.Throws<InvalidDataException>(() => engine.LoadState(new MemoryStream(other))).Message);
        }

        // A DSP's firmware found whole or as its program and data pair, the pair passed as files 2 and 3, and nothing missing on VenusRT, which needs none; synthetic bytes, not a dump.
        [Fact]
        public void The_adapter_takes_a_dsp_firmware_whole_or_as_its_split_pair()
        {
            if (Discovered is not { } found) return;
            string rom = Path.Combine(_root, "pilot.sfc");
            File.WriteAllBytes(rom, SyntheticRom.BuildNecDsp("PILOTWINGS"));
            Assert.Empty(EmuSen.Common.EmulatorSession.MissingFirmwareFor(rom, Engine));
            Assert.DoesNotContain(EmuSen.Common.EmulatorSession.MissingFirmwareFor(rom), r => r.FileName == "spc700.rom");
            byte[] program = Enumerable.Range(0, 6144).Select(i => (byte)(i * 3)).ToArray(), data = Enumerable.Range(0, 2048).Select(i => (byte)(i * 5)).ToArray();
            string whole = Path.Combine(FirmwareLibrary.Directory, "dsp1.rom");
            int StateSize()
            {
                using var engine = new CoreEngine(found.Open()!);
                engine.LoadRom(rom);
                return engine.Machine.StateSize(0);
            }

            var request = new CoreEngine(found.Open()!).GetFirmwareRequirements(rom).Single(r => r.FileName == "dsp1.rom");
            Assert.Equal(8192, request.Size);
            Assert.False(request.Required);
            Assert.Equal("accuracy", request.ReplacementEffect);
            Assert.Contains(request.Parts, form => form.SequenceEqual(new[] { "dsp1.program.rom", "dsp1.data.rom" }));
            int none = StateSize();
            Assert.False(FirmwareLibrary.IsInstalled(request));

            File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, "dsp1.program.rom"), program);
            File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, "dsp1.data.rom"), data);
            Assert.True(FirmwareLibrary.IsInstalled(request));
            Assert.Empty(EmuSen.Common.EmulatorSession.MissingFirmwareFor(rom, Engine));
            Assert.Equal(new[] { program, data }, FirmwareLibrary.TryLoadParts(request));
            int pair = StateSize();
            Assert.True(pair > none, "the split pair did not reach the DSP");

            File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, "dsp1.data.rom"), data[..1000]);
            Assert.Null(FirmwareLibrary.TryLoadParts(request));
            Assert.Equal(none, StateSize());
            File.WriteAllBytes(whole, program.Concat(data).ToArray());
            Assert.Equal(pair, StateSize());
        }

        // The generic engine over VenusRT's v1 exports and the shim over its pre-stable ones: the same picture, sound and state, frame for frame.
        [Fact]
        public void The_v1_adapter_runs_venusrt_exactly_as_the_shim()
        {
            if (Discovered is not { } found) return;
            string rom = Path.Combine(_root, "adapter.sfc");
            File.WriteAllBytes(rom, PadToBackdropRom());
            using var adapter = new CoreEngine(found.Open()!);
            adapter.LoadRom(rom);
            using var shim = new VenusMachine(File.ReadAllBytes(rom));
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
            if (dir is null || Discovered is not { } found) return;
            InstallFirmware();
            foreach (string source in Directory.GetFiles(dir).Order(StringComparer.Ordinal))
            {
                string rom = Path.Combine(_root, Path.GetFileName(source));
                File.Copy(source, rom, overwrite: true);
                using var adapter = new CoreEngine(found.Open()!);
                adapter.LoadRom(rom);
                using var shim = new VenusMachine(File.ReadAllBytes(rom), dspFirmware: VenusRtSnesEngine.DspFirmware(rom));
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
            if (dir is null || Discovered is null) return;
            InstallFirmware();
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
