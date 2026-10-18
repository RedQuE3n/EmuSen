using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Systems.Nes;
using EmuSen.Crystal.Cheats;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MoonRT through ICore as the frontends reach it: the factory, the library, battery saves and cheats - see Moon_Native.md §8.3, §9.5.
    [Collection(TestCollections.ProcessGlobals)]
    public class MoonRtFrontendTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMoonRtFrontendTests", Guid.NewGuid().ToString("N"));

        public MoonRtFrontendTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDirectories(null);
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            CoreDiscovery.UseDirectories(null);
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private string Rom(byte[] image, string name = "Game.nes")
        {
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, image);
            return path;
        }

        private static byte[] Program() => BoardPrograms.Build(1, 8, cycleIrq: false);

        private static byte[] State(ICore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        // The NES has one engine and no Engine row; a stored Moon (C#) runs on MoonRT with no notice.
        [Fact]
        public void The_NES_runs_on_MoonRT_alone_and_a_stored_csharp_choice_runs_there_quietly()
        {
            Assert.Null(CoreCatalog.EngineFor("NES"));
            Assert.Null(CoreCatalog.ReferenceEngine("NES"));
            Assert.Equal(CoreCatalog.MoonRtEngine, CoreCatalog.EngineChosen("NES", "Moon (C#)"));

            string rom = Rom(Program());
            foreach (string? engine in new[] { null, CoreCatalog.MoonRtEngine, "Moon (C#)" })
            {
                CoreBundle bundle = CoreFactory.Load(rom, engine: engine, options: TestCores.NoBattery);
                var core = Assert.IsType<CoreEngine>(bundle.Core);
                Assert.Null(bundle.Notice);
                Assert.IsType<CoreDebugTarget>(bundle.DebugTarget);
                Assert.Equal(("NES address:value", NesSystem.CpuBus), (bundle.CheatAutoDetectCodec!.Name, bundle.CheatAutoDetectCodec.SpaceName));
                Assert.Equal("Game Genie", bundle.CheatExplicitCodec!.Name);
                Assert.True(EngineFeatures.Of(core).RewindCapture);
                Assert.Equal(CoreCatalog.MoonRtEngine, CoreFactory.Running(core));
                core.Dispose();
            }
        }

        // A build without the library, or without its sidecar, opens no NES game and says why in the player's words, as every console with one discovered engine does.
        [Fact]
        public void Without_its_library_an_nes_game_is_refused_in_words()
        {
            CoreDiscovery.UseDirectories(new[] { Path.Combine(_root, "nothing") });
            Assert.Equal($"This build has no NES core: {CoreCatalog.MoonRtEngine} is missing (no sidecar for it was found beside the program).",
                Assert.Throws<NotSupportedException>(() => CoreFactory.Create("game.nes", options: TestCores.NoBattery)).Message);
        }

        // Since its shim retired the library is on the core ABI alone: no pre-stable export and no extension of the pre-stable handle is left in it.
        [Fact]
        public void The_library_is_on_the_core_abi_alone()
        {
            Assert.True(MoonEngines.Available, MoonEngines.Report);
            nint handle = NativeLibrary.Load(MoonEngines.LibraryPath);
            foreach (string name in NativeInterface.Required) Assert.False(NativeLibrary.TryGetExport(handle, name, out _), name);
            foreach (var (_, name, exports) in NativeInterface.Optional)
                foreach (string export in exports) Assert.False(NativeLibrary.TryGetExport(handle, export, out _), $"{name}: {export}");
            foreach (string gone in new[] { "moonrt_step", "moonrt_rom_patch", "moon_machine_new" }) Assert.False(NativeLibrary.TryGetExport(handle, gone, out _), gone);
            foreach (string kept in new[] { "moonrt_core_step", "moonrt_core_rom_patch" }) Assert.True(NativeLibrary.TryGetExport(handle, kept, out _), kept);
        }

        // The program counts into $6000 every loop; a battery save starts the machine, is written back, and a board without one opens none.
        [Fact]
        public void A_battery_save_loads_is_written_back_and_is_absent_without_a_battery()
        {
            string rom = Rom(Program(), "Battery.nes");
            string save = Path.ChangeExtension(SaveLibrary.SramPathFor(rom, BatterySave.Nes), SaveLibrary.SramExtension);
            byte[] marked = new byte[NesHeader.PrgRamSize];
            "MOONRT!!"u8.CopyTo(marked.AsSpan(0x1000));
            Directory.CreateDirectory(Path.GetDirectoryName(save)!);
            File.WriteAllBytes(save, marked);

            var rust = (CoreEngine)CoreFactory.Load(rom).Core;
            Assert.Equal(NesHeader.PrgRamSize, rust.Machine.Battery(0).Data.Length);
            Assert.Equal(marked[0x1000..0x1008], Enumerable.Range(0x1000, 8).Select(i => rust.ReadSpace(NesSystem.PrgRam, i)).ToArray());
            for (int f = 0; f < 301; f++) rust.RunFrame();
            byte[] written = File.ReadAllBytes(save);
            Assert.Equal(marked[0x1000..0x1008], written[0x1000..0x1008]);
            Assert.NotEqual(marked[..4], written[..4]);
            rust.Dispose();

            var again = (CoreEngine)CoreFactory.Load(rom).Core;
            Assert.Equal(written, Enumerable.Range(0, written.Length).Select(i => again.ReadSpace(NesSystem.PrgRam, i)).ToArray());
            again.Dispose();

            byte[] image = BoardPrograms.Build(4, 8, cycleIrq: false);
            image[6] &= 0xFD;
            var none = (CoreEngine)CoreFactory.Load(Rom(image, "NoBattery.nes")).Core;
            Assert.Empty(none.Machine.Battery(0).Data);
            none.Dispose();
        }

        // Game Genie's SXIOPO on Super Mario Bros. changes the byte the CPU reads and the machine's course.
        [Fact]
        public void A_Game_Genie_code_takes_effect()
        {
            if (MoonGoldenTests.Game("F61548FDF1670CFF") is not { } image)
            {
                _output.WriteLine($"{MoonGoldenTests.GamesVariable} unset or without the game, not run");
                return;
            }
            string smb = Rom(image, "Smb.nes");
            var cheats = new CheatRegistry();
            var (address, value) = NesGameGenieCodec.Decode("SXIOPO");
            cheats.AddRomPatch(address, value, NesGameGenieCodec.DecodeCompare("SXIOPO"), "infinite lives");
            var coded = (CoreEngine)CoreFactory.Load(smb, cheats: cheats, options: TestCores.NoBattery).Core;
            var plain = (CoreEngine)CoreFactory.Load(smb, options: TestCores.NoBattery).Core;
            for (int f = 0; f < 900; f++)
                foreach (CoreEngine core in new[] { coded, plain })
                {
                    core.SetButton(0, PadButton.Start, f % 90 < 5);
                    core.SetButton(0, PadButton.A, f % 90 is >= 45 and < 50);
                    core.RunFrame();
                }
            Assert.Equal(value, coded.ReadSpace(NesSystem.CpuBus, address));
            Assert.NotEqual(plain.ReadSpace(NesSystem.CpuBus, address), coded.ReadSpace(NesSystem.CpuBus, address));
            Assert.False(State(plain).AsSpan().SequenceEqual(State(coded)), "the code changed nothing in 900 frames");
            coded.Dispose();
            plain.Dispose();
        }

        // The triples against TryPatchRom for every address and every byte, over mixed compares, repeats and an overlap (EmuSen_NativeCores.md §3.12).
        [Fact]
        public void The_patch_triples_answer_as_TryPatchRom_everywhere()
        {
            var cheats = new CheatRegistry();
            cheats.AddRomPatch(0x9123, 0x42, 0x07, "compared");
            cheats.AddRomPatch(0x9123, 0x55, null, "later, unconditional");
            cheats.AddRomPatch(0x9124, 0x10, null, "unconditional");
            cheats.AddRomPatch(0x9123, 0x66, 0x07, "shadowed");
            cheats.AddRomPatch(0xFFFF, 0x01, 0xFF, "last");
            cheats.AddRomPatch(0x4020, 0x02, null, "first");
            cheats.AddRomPatch(0x3FFF, 0x03, null, "below the range");
            var (gg, value) = NesGameGenieCodec.Decode("SXIOPO");
            cheats.AddRomPatch(gg, value, NesGameGenieCodec.DecodeCompare("SXIOPO"), "six");
            int disabled = cheats.AddRomPatch(0x9200, 0x77, null, "off");
            cheats.SetEnabled(disabled, false);

            var core = (CoreEngine)CoreFactory.Load(Rom(Program()), cheats: cheats, options: TestCores.NoBattery).Core;
            core.RunFrame();
            var touched = new HashSet<int> { 0x9123, 0x9124, 0xFFFF, 0x4020, 0x3FFF, gg, 0x9200 };
            for (int address = 0x4020; address <= 0xFFFF; address++)
            {
                IEnumerable<int> values = touched.Contains(address) ? Enumerable.Range(0, 256) : new[] { 0, 0x07, 0xFF };
                foreach (int original in values)
                {
                    int want = cheats.TryPatchRom((uint)address, (byte)original, out byte patched) ? patched : -1;
                    Assert.True(want == MoonEngines.RomPatch(core.Machine, address, (byte)original), $"${address:X4} over {original:X2}: TryPatchRom {want}, MoonRT {MoonEngines.RomPatch(core.Machine, address, (byte)original)}");
                }
            }
            Assert.Equal(-1, MoonEngines.RomPatch(core.Machine, 0x3FFF, 0));
            core.Dispose();
        }

        // A cheat added between frames is on the CPU's bus at once, before the next frame (§9 Q11).
        [Fact]
        public void A_bus_read_after_a_new_cheat_sees_it()
        {
            var cheats = new CheatRegistry();
            var rust = (CoreEngine)CoreFactory.Load(Rom(Program()), cheats: cheats, options: TestCores.NoBattery).Core;
            rust.RunFrame();
            byte before = rust.ReadSpace(NesSystem.CpuBus, 0xC001);
            cheats.AddRomPatch(0xC001, (byte)(before ^ 0x5A), null, "new");
            Assert.Equal((byte)(before ^ 0x5A), rust.ReadSpace(NesSystem.CpuBus, 0xC001));
            rust.Dispose();
        }

        // An image with no PRG is refused at the load as a fault, in the core's words with its status, the oracle's exception types retired - see EmuSen_Backend.md §20.
        [Fact]
        public void An_image_without_prg_is_refused()
        {
            byte[] image = SyntheticNesRom.Build(mapper: 1);
            byte[] empty = image.AsSpan(0, 16).ToArray().Concat(image.AsSpan(16 + SyntheticNesRom.PrgBankSize).ToArray()).ToArray();
            empty[4] = 0;
            using var core = MoonEngines.Create(TestCores.NoBattery);
            var refused = Assert.Throws<CoreRefusedException>(() => core.LoadRom(Rom(empty, "NoPrg.nes")));
            _output.WriteLine(refused.Status + ": " + refused.Message);
            Assert.InRange(refused.Status, NativeInterface.FaultBase - 63, NativeInterface.FaultBase);
            Assert.StartsWith($"{CoreCatalog.MoonRtEngine} refused the game: ", refused.Message);
            Assert.False(core.IsRomLoaded);
        }
    }
}
