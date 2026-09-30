using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Cheats;
using EmuSen.Cores.Nintendo.Moon.Memory;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Stage 4: MoonRT through ICore on the common native host, against the C# Moon through the same factory - see Moon_Native.md §8.3.
    [Collection(TestCollections.ProcessGlobals)]
    public class MoonRtFrontendTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMoonRtFrontendTests", Guid.NewGuid().ToString("N"));
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public MoonRtFrontendTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreOptions.BatteryRamDisabled = true;
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private string Rom(byte[] image, string name = "Game.nes")
        {
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, image);
            return path;
        }

        private static byte Peek(ICore core, string space, int address) => core is MoonCore moon ? moon.ReadSpace(space, address) : ((MoonRtCore)core).ReadSpace(space, address);

        private static byte[] Program() => BoardPrograms.Build(1, 8, cycleIrq: false);

        private static byte[] State(ICore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        // Picture, sound and state after each frame, compared; the bench's Start and A presses, the same on both.
        private static void RunAlike(ICore csharp, ICore rust, int frames, int from = 0)
        {
            for (int f = from; f < from + frames; f++)
            {
                int p = f % 90;
                foreach (ICore core in new[] { csharp, rust })
                {
                    core.SetButton(0, PadButton.Start, p < 5);
                    core.SetButton(0, PadButton.A, p >= 45 && p < 50);
                    core.RunFrame();
                }
                short[] a = csharp.DequeueAudioSamples(int.MaxValue), b = rust.DequeueAudioSamples(int.MaxValue);
                Assert.True(a.AsSpan().SequenceEqual(b), $"frame {f}: the sound differs ({a.Length} and {b.Length} samples)");
                byte[] pa = csharp.GetFrameBufferRgba(), pb = rust.GetFrameBufferRgba();
                Assert.True(pa.AsSpan().SequenceEqual(pb), $"frame {f}: the pictures differ");
                if (rust is IFrameBufferPool pool) pool.ReturnFrameBuffer(pb);
                Assert.True(State(csharp).AsSpan().SequenceEqual(State(rust)), $"frame {f}: the states differ");
            }
        }

        [Fact]
        public void The_NES_row_offers_MoonRT_with_Moon_the_default_and_the_factory_honours_it()
        {
            CoreSetting engine = CoreCatalog.EngineFor("NES")!;
            Assert.Equal(CoreCatalog.MoonEngine, engine.Default);
            Assert.Equal(new[] { CoreCatalog.MoonEngine, CoreCatalog.MoonRtEngine }, engine.Choices);
            Assert.Equal(CoreCatalog.MoonEngine, CoreCatalog.EngineChosen("NES", null));

            string rom = Rom(Program());
            CoreBundle reference = CoreFactory.Load(rom);
            Assert.IsType<MoonCore>(reference.Core);
            CoreBundle rt = CoreFactory.Load(rom, engine: CoreCatalog.MoonRtEngine);
            var core = Assert.IsType<MoonRtCore>(rt.Core);
            Assert.Null(rt.Notice);
            Assert.NotNull(rt.DebugTarget);
            Assert.Equal(reference.CheatExplicitCodec!.GetType(), rt.CheatExplicitCodec!.GetType());
            Assert.Equal(reference.CheatAutoDetectCodec!.GetType(), rt.CheatAutoDetectCodec!.GetType());
            Assert.True(EngineFeatures.Of(core).RewindCapture);
            Assert.Contains("\"Nope\"", CoreFactory.EngineNotice(rom, "Nope", core));
            Assert.Contains("MoonRT (Rust) is running", CoreFactory.EngineNotice(rom, "Nope", core));
            core.Dispose();
        }

        // The loader refuses a library that is not there with the words the notice carries; the factory's fallback is the same Available.
        [Fact]
        public void A_missing_library_is_reported_and_not_used()
        {
            var missing = NativeCoreLibrary.Common("moonrt_missing", "EMUSEN_MOONRT_MISSING_TEST", MoonNative.CoreVersion, MoonNative.RequiredCapabilities);
            Assert.False(missing.Available);
            Assert.Contains("not found beside the assemblies", missing.Report);
            Assert.False(new NativeInterface(missing).Complete);
        }

        // Every capability the library claims has its exports, every one it does not claim has none, and every required export is there.
        [Fact]
        public void The_capabilities_and_the_exports_agree()
        {
            Assert.True(MoonNative.Available, MoonNative.Report);
            Assert.Equal(MoonNative.RequiredCapabilities, MoonNative.Library.Capabilities);
            string path = Path.Combine(AppContext.BaseDirectory, MoonNative.Library.FileName);
            nint handle = NativeLibrary.Load(path);
            foreach (string name in NativeInterface.Required) Assert.True(NativeLibrary.TryGetExport(handle, name, out _), name);
            foreach (var (bit, name, exports) in NativeInterface.Optional)
                foreach (string export in exports)
                    Assert.True(NativeLibrary.TryGetExport(handle, export, out _) == ((MoonNative.Library.Capabilities & bit) != 0), $"{name}: {export}");
            Assert.False(NativeLibrary.TryGetExport(handle, "moon_machine_new", out _), "the old moon_* exports are retired");
        }

        [Fact]
        public void The_bench_games_and_the_library_states_run_alike_through_ICore()
        {
            string? folder = Environment.GetEnvironmentVariable(MoonRtStateTests.RomsVariable);
            if (folder is null || !Directory.Exists(folder))
            {
                _output.WriteLine($"{MoonRtStateTests.RomsVariable} unset, not run");
                return;
            }
            foreach (string path in Directory.GetFiles(folder, "*.nes").Order(StringComparer.Ordinal))
            {
                ICore csharp = CoreFactory.Load(path).Core, rust = CoreFactory.Load(path, engine: CoreCatalog.MoonRtEngine).Core;
                Assert.IsType<MoonRtCore>(rust);
                RunAlike(csharp, rust, 1500);
                int size = ((MoonRtCore)rust).Machine.StateSize;
                RunAlike(csharp, rust, 300, from: 1500);
                Assert.Equal(size, ((MoonRtCore)rust).Machine.StateSize);
                ((IDisposable)rust).Dispose();
                _output.WriteLine($"{Path.GetFileName(path)}: 1,800 frames alike");
            }

            string? states = Environment.GetEnvironmentVariable(MoonRtSoundAndPictureTests.StatesVariable);
            if (states is null || !Directory.Exists(states)) return;
            int runs = 0;
            foreach (string path in Directory.GetFiles(states, "*.nes").Order(StringComparer.Ordinal))
                foreach (string state in Directory.GetFiles(states, Path.GetFileNameWithoutExtension(path) + ".*state").Order(StringComparer.Ordinal))
                {
                    ICore csharp = CoreFactory.Load(path).Core, rust = CoreFactory.Load(path, engine: CoreCatalog.MoonRtEngine).Core;
                    csharp.LoadState(state);
                    rust.LoadState(state);
                    Assert.True(State(csharp).AsSpan().SequenceEqual(State(rust)), $"{Path.GetFileName(state)}: the states differ after the load");
                    RunAlike(csharp, rust, 600);
                    ((IDisposable)rust).Dispose();
                    runs++;
                }
            _output.WriteLine($"{runs} library states, 600 frames each, alike");
        }

        [Fact]
        public void A_state_crosses_from_Moon_to_MoonRT_and_back()
        {
            string rom = Rom(Program());
            ICore csharp = CoreFactory.Load(rom).Core;
            for (int f = 0; f < 200; f++) csharp.RunFrame();
            byte[] fromCsharp = State(csharp);

            // The mixer is in no state, so each engine takes the state onto a fresh instance and the two are compared from there.
            ICore rust = CoreFactory.Load(rom, engine: CoreCatalog.MoonRtEngine).Core, again = CoreFactory.Load(rom).Core;
            rust.LoadState(new MemoryStream(fromCsharp));
            again.LoadState(new MemoryStream(fromCsharp));
            Assert.Equal(fromCsharp, State(rust));
            RunAlike(again, rust, 200, from: 200);
            byte[] fromRust = State(rust);
            ((IDisposable)rust).Dispose();

            ICore laterCsharp = CoreFactory.Load(rom).Core, laterRust = CoreFactory.Load(rom, engine: CoreCatalog.MoonRtEngine).Core;
            laterCsharp.LoadState(new MemoryStream(fromRust));
            laterRust.LoadState(new MemoryStream(fromRust));
            Assert.Equal(fromRust, State(laterCsharp));
            RunAlike(laterCsharp, laterRust, 200, from: 400);
            ((IDisposable)laterRust).Dispose();
        }

        // The program counts into $6000 every loop; each engine's battery save starts the other, and the machines agree straight after the load.
        [Fact]
        public void A_battery_save_made_on_one_engine_loads_on_the_other()
        {
            CoreOptions.BatteryRamDisabled = false;
            string rom = Rom(Program(), "Battery.nes");
            string save = Path.ChangeExtension(SaveLibrary.SramPathFor(rom, BatterySave.Nes), SaveLibrary.SramExtension);
            byte[] marked = new byte[0x2000];
            "MOONRT!!"u8.CopyTo(marked.AsSpan(0x1000));
            Directory.CreateDirectory(Path.GetDirectoryName(save)!);
            File.WriteAllBytes(save, marked);

            var rust = (MoonRtCore)CoreFactory.Load(rom, engine: CoreCatalog.MoonRtEngine).Core;
            Assert.Equal(new MoonCore().Let(c => c.LoadRom(rom)).Cart!.PrgRam.Length, rust.Machine.Battery(0).Data.Length);
            for (int f = 0; f < 301; f++) rust.RunFrame();
            byte[] written = File.ReadAllBytes(save);
            Assert.Equal(marked[0x1000..0x1008], written[0x1000..0x1008]);
            Assert.NotEqual(marked[..4], written[..4]);
            rust.Dispose();

            ICore csharp = CoreFactory.Load(rom).Core, again = CoreFactory.Load(rom, engine: CoreCatalog.MoonRtEngine).Core;
            Assert.True(State(csharp).AsSpan().SequenceEqual(State(again)), "the machines differ straight after a load with a battery save");
            Assert.Equal(written, Enumerable.Range(0, written.Length).Select(i => Peek(csharp, MoonCore.SpacePrgRam, i)).ToArray());
            RunAlike(csharp, again, 100);
            ((IDisposable)again).Dispose();
            for (int f = 100; f < 301; f++) csharp.RunFrame();
            byte[] fromCsharp = File.ReadAllBytes(save);
            Assert.Equal(marked[0x1000..0x1008], fromCsharp[0x1000..0x1008]);

            var third = (MoonRtCore)CoreFactory.Load(rom, engine: CoreCatalog.MoonRtEngine).Core;
            Assert.Equal(fromCsharp, Enumerable.Range(0, fromCsharp.Length).Select(i => third.ReadSpace(MoonCore.SpacePrgRam, i)).ToArray());
            third.Dispose();

            var none = (MoonRtCore)CoreFactory.Load(Rom(BoardPrograms.Build(4, 8, cycleIrq: false).Let(i => i[6] &= 0xFD), "NoBattery.nes"), engine: CoreCatalog.MoonRtEngine).Core;
            Assert.Empty(none.Machine.Battery(0).Data);
            none.Dispose();
        }

        // Game Genie's SXIOPO on Super Mario Bros.: the same effect on both engines, and an effect.
        [Fact]
        public void A_Game_Genie_code_does_the_same_on_both_engines()
        {
            string? folder = Environment.GetEnvironmentVariable(MoonRtStateTests.RomsVariable);
            string? smb = folder is null ? null : Path.Combine(folder, "smb.nes");
            if (smb is null || !File.Exists(smb))
            {
                _output.WriteLine($"{MoonRtStateTests.RomsVariable} unset, not run");
                return;
            }
            CheatRegistry Code()
            {
                var cheats = new CheatRegistry();
                var (address, value) = NesGameGenieCodec.Decode("SXIOPO");
                cheats.AddRomPatch(address, value, NesGameGenieCodec.DecodeCompare("SXIOPO"), "infinite lives");
                return cheats;
            }
            ICore csharp = CoreFactory.Load(smb, cheats: Code()).Core, rust = CoreFactory.Load(smb, cheats: Code(), engine: CoreCatalog.MoonRtEngine).Core;
            ICore plain = CoreFactory.Load(smb).Core;
            RunAlike(csharp, rust, 900);
            for (int f = 0; f < 900; f++) plain.RunFrame();
            int address = NesGameGenieCodec.Decode("SXIOPO").Address;
            Assert.Equal(Peek(csharp, MoonCore.SpaceCpuBus, address), Peek(rust, MoonCore.SpaceCpuBus, address));
            Assert.NotEqual(Peek(plain, MoonCore.SpaceCpuBus, address), Peek(rust, MoonCore.SpaceCpuBus, address));
            Assert.False(State(plain).AsSpan().SequenceEqual(State(rust)), "the code changed nothing in 900 frames");
            ((IDisposable)rust).Dispose();
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

            var core = (MoonRtCore)CoreFactory.Load(Rom(Program()), cheats: cheats, engine: CoreCatalog.MoonRtEngine).Core;
            core.RunFrame();
            var touched = new HashSet<int> { 0x9123, 0x9124, 0xFFFF, 0x4020, 0x3FFF, gg, 0x9200 };
            for (int address = 0x4020; address <= 0xFFFF; address++)
            {
                IEnumerable<int> values = touched.Contains(address) ? Enumerable.Range(0, 256) : new[] { 0, 0x07, 0xFF };
                foreach (int original in values)
                {
                    int want = cheats.TryPatchRom((uint)address, (byte)original, out byte patched) ? patched : -1;
                    Assert.True(want == core.Machine.RomPatch(address, (byte)original), $"${address:X4} over {original:X2}: TryPatchRom {want}, MoonRT {core.Machine.RomPatch(address, (byte)original)}");
                }
            }
            Assert.Equal(-1, core.Machine.RomPatch(0x3FFF, 0));
            core.Dispose();
        }

        // A cheat added between frames is on the CPU's bus at once, as the C# core's reads consult the registry (§9 Q11).
        [Fact]
        public void A_bus_read_after_a_new_cheat_answers_as_the_csharp_core()
        {
            string rom = Rom(Program());
            var cheats = new CheatRegistry();
            ICore csharp = CoreFactory.Load(rom, cheats: cheats).Core;
            var rust = (MoonRtCore)CoreFactory.Load(rom, cheats: cheats, engine: CoreCatalog.MoonRtEngine).Core;
            csharp.RunFrame();
            rust.RunFrame();
            byte before = Peek(rust, MoonCore.SpaceCpuBus, 0xC001);
            cheats.AddRomPatch(0xC001, (byte)(before ^ 0x5A), null, "new");
            Assert.Equal(Peek(csharp, MoonCore.SpaceCpuBus, 0xC001), Peek(rust, MoonCore.SpaceCpuBus, 0xC001));
            Assert.Equal((byte)(before ^ 0x5A), Peek(rust, MoonCore.SpaceCpuBus, 0xC001));
            rust.Dispose();
        }

        // A reproduced C# exception keeps its type through the host: D4's image without PRG, as MoonCore.LoadRom throws.
        [Fact]
        public void A_reproduced_exception_is_the_csharp_type()
        {
            byte[] image = SyntheticNesRom.Build(mapper: 1);
            byte[] empty = image.AsSpan(0, 16).ToArray().Concat(image.AsSpan(16 + SyntheticNesRom.PrgBankSize).ToArray()).ToArray();
            empty[4] = 0;
            string rom = Rom(empty, "NoPrg.nes");
            Exception csharp = Assert.ThrowsAny<Exception>(() => new MoonCore().LoadRom(rom));
            Exception rust = Assert.ThrowsAny<Exception>(() => new MoonRtCore().LoadRom(rom));
            Assert.Equal(csharp.GetType(), rust.GetType());
            Assert.Equal(typeof(DivideByZeroException), MoonMachine.Refusal(NativeInterface.FaultBase - 2).GetType());
            Assert.Equal(typeof(ArgumentOutOfRangeException), MoonMachine.Refusal(NativeInterface.FaultBase - 3).GetType());
        }
    }

    internal static class LetExtension
    {
        public static T Let<T>(this T value, Action<T> action)
        {
            action(value);
            return value;
        }
    }
}
