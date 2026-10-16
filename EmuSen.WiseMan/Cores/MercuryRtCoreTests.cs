using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Systems.GameBoy;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MercuryRtCore through ICore alone: the engine setting, cheats, battery saves, states, the spaces and the debugger's view - see Mercury_Native.md §8.3, §15.
    [Collection(TestCollections.ProcessGlobals)]
    public class MercuryRtCoreTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _dir = Path.Combine(Path.GetTempPath(), "mercuryrt_core_" + Guid.NewGuid().ToString("N"));

        public MercuryRtCoreTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_dir);
            DataStore.OverrideDirectory = Path.Combine(_dir, "Home");
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            CoreOptions.BatteryRamDisabled = true;
            try { Directory.Delete(_dir, recursive: true); } catch (IOException) { }
        }

        private string Rom(string name, byte[] image)
        {
            string sub = Path.Combine(_dir, name);
            Directory.CreateDirectory(sub);
            string path = Path.Combine(sub, "game.gb");
            File.WriteAllBytes(path, image);
            return path;
        }

        // Enables cart RAM, then counts into it and into WRAM every frame from the vblank handler, reading a ROM byte a Game Genie code can change.
        private static byte[] CountingRom(byte kind) => SyntheticGbRom.Build(romBanks: 4, cartridgeType: kind, ramSizeCode: 0x02,
            patches: new (int, byte[])[]
            {
                (0x40 - 0x150, new byte[] { 0xC3, 0x00, 0x02 }),
                (0, new byte[] { 0x31, 0xFE, 0xFF, 0x3E, 0x0A, 0xEA, 0x00, 0x00, 0x3E, 0x01, 0xE0, 0xFF, 0xFB, 0x76, 0x00, 0x18, 0xFC }),
                (0x200 - 0x150, new byte[] { 0xF5, 0xE5, 0x21, 0x00, 0xC0, 0xFA, 0x00, 0x03, 0x86, 0x77, 0xEA, 0x00, 0xA0, 0xE1, 0xF1, 0xD9 }),
                (0x300 - 0x150, new byte[] { 0x01 }),
            });

        [Fact]
        public void The_default_engine_builds_mercuryrt()
        {
            Assert.True(MercuryRtCore.Available, MercuryNative.Report);
            string rom = Rom("engine", CountingRom(0x03));
            CoreBundle rt = CoreFactory.Load(rom, engine: CoreCatalog.EngineChosen("GB", null));
            Assert.IsType<MercuryRtCore>(rt.Core);
            Assert.Null(rt.Notice);
            Assert.Equal("GB", rt.Core.CoreName);
            Assert.Equal(4194304.0 / 70224, rt.Core.FrameRateHz);
            (rt.Core as IDisposable)?.Dispose();
        }

        // The count the vblank handler keeps after some frames, with the registry given.
        private byte CountAfter(string rom, int frames, CheatRegistry? cheats = null)
        {
            using var core = (MercuryRtCore)CoreFactory.Load(rom, cheats: cheats, engine: CoreCatalog.MercuryRtEngine).Core;
            for (int f = 0; f < frames; f++) core.RunFrame();
            return core.ReadSpace(MercuryRtCore.SpaceWram, 0);
        }

        // A RAM poke lands, and a compare-gated ROM patch makes the handler count by three; with the master switch off it counts by one.
        [Fact]
        public void Cheats_of_both_kinds_act()
        {
            CoreOptions.BatteryRamDisabled = true;
            string rom = Rom("cheats", CountingRom(0x03));
            byte plain = CountAfter(rom, 300);
            Assert.NotEqual(0, plain);

            var cheats = new CheatRegistry();
            cheats.AddRomPatch(0x0300, 0x03, compare: 0x01, "count by three");
            cheats.AddRamPoke(MercuryRtCore.SpaceCpuBus, 0xC010, 0x5A, "a poke");
            using var core = (MercuryRtCore)CoreFactory.Load(rom, cheats: cheats, engine: CoreCatalog.MercuryRtEngine).Core;
            for (int f = 0; f < 300; f++) core.RunFrame();
            Assert.Equal((byte)(plain * 3), core.ReadSpace(MercuryRtCore.SpaceWram, 0));
            Assert.Equal(0x5A, core.ReadSpace(MercuryRtCore.SpaceWram, 0x10));

            cheats.MasterEnabled = false;
            byte before = core.ReadSpace(MercuryRtCore.SpaceWram, 0);
            for (int f = 0; f < 300; f++) core.RunFrame();
            Assert.Equal((byte)(before + plain), core.ReadSpace(MercuryRtCore.SpaceWram, 0));
            _output.WriteLine($"WRAM count {plain} plain after 300 frames, {(byte)(plain * 3)} by three");
        }

        // The save in the Saves folder is read into cart RAM at the load and written at frame 300's flush.
        [Fact]
        public void A_battery_save_loads_and_is_written_back()
        {
            CoreOptions.BatteryRamDisabled = false;
            string rom = Rom("battery", CountingRom(0x03)), srm = SaveLibrary.SramPathFor(rom, BatterySave.GameBoy);
            Directory.CreateDirectory(Path.GetDirectoryName(srm)!);
            var saved = new byte[8192];
            new Random(64).NextBytes(saved);
            File.WriteAllBytes(srm, saved);

            using var rt = new MercuryRtCore();
            rt.LoadRom(rom);
            Assert.Equal(saved[17], rt.ReadSpace(MercuryRtCore.SpaceCartRam, 17));
            for (int f = 0; f < 300; f++) rt.RunFrame();
            byte[] written = File.ReadAllBytes(srm);
            Assert.False(written.SequenceEqual(saved), "frame 300's save wrote nothing new");
            Assert.Equal(rt.ReadSpace(MercuryRtCore.SpaceCartRam, 0), written[0]);
        }

        // A state saved and loaded again resumes into the same future, and a foreign or too old state is refused.
        [Fact]
        public void A_state_saved_resumes_into_the_same_future()
        {
            CoreOptions.BatteryRamDisabled = true;
            string rom = Rom("states", CountingRom(0x1B));
            using var rt = (MercuryRtCore)CoreFactory.Load(rom, engine: CoreCatalog.MercuryRtEngine).Core;
            for (int f = 0; f < 120; f++) rt.RunFrame();

            string path = Path.Combine(_dir, "rt.state");
            rt.SaveState(path);
            for (int i = 0; i < 50; i++) rt.RunFrame();
            byte[] later = State(rt);
            rt.LoadState(path);
            for (int i = 0; i < 50; i++) rt.RunFrame();
            Assert.True(later.AsSpan().SequenceEqual(State(rt)), "the state loaded again ran on differently");

            Assert.Throws<InvalidDataException>(() => rt.LoadState(new MemoryStream(new byte[] { 0x4D, 0x41, 0x52, 0x54, 5, 0, 0, 0 })));
            Assert.Throws<InvalidDataException>(() => rt.LoadState(new MemoryStream(new byte[] { 0x4D, 0x45, 0x52, 0x43, 4, 0, 0, 0 })));
            Assert.Throws<InvalidOperationException>(() => { using var unloaded = new MercuryRtCore(); unloaded.RunFrame(); });
        }

        private static byte[] State(ICore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        // The triples against TryPatchRom for every ROM address, over mixed compares, repeats, a Game Genie code and one past the range (EmuSen_NativeCores.md §3.12).
        [Fact]
        public void The_patch_triples_answer_as_TryPatchRom_everywhere()
        {
            CoreOptions.BatteryRamDisabled = true;
            var cheats = new CheatRegistry();
            cheats.AddRomPatch(0x1123, 0x42, 0x07, "compared");
            cheats.AddRomPatch(0x1123, 0x55, null, "later, unconditional");
            cheats.AddRomPatch(0x1124, 0x10, null, "unconditional");
            cheats.AddRomPatch(0x1124, 0x20, null, "shadowed, unconditional");
            cheats.AddRomPatch(0x1123, 0x66, 0x07, "shadowed");
            cheats.AddRomPatch(0x7FFF, 0x01, 0xFF, "last");
            cheats.AddRomPatch(0x0000, 0x02, null, "first");
            cheats.AddRomPatch(0x8000, 0x03, null, "above the range");
            const string GameGenie = "00A-17B-C49";
            var (gg, value) = GbGameGenieCodec.Decode(GameGenie);
            cheats.AddRomPatch(gg, value, GbGameGenieCodec.DecodeCompare(GameGenie), "nine");
            int disabled = cheats.AddRomPatch(0x2200, 0x77, null, "off");
            cheats.SetEnabled(disabled, false);

            var core = (MercuryRtCore)CoreFactory.Load(Rom("patches", CountingRom(0x03)), cheats: cheats, engine: CoreCatalog.MercuryRtEngine).Core;
            core.RunFrame();
            var touched = new HashSet<int> { 0x1123, 0x1124, 0x7FFF, 0x0000, gg, 0x2200 };
            for (int address = 0; address <= 0x7FFF; address++)
            {
                IEnumerable<int> values = touched.Contains(address) ? Enumerable.Range(0, 256) : new[] { 0, 0x07, 0xFF };
                foreach (int original in values)
                {
                    int want = cheats.TryPatchRom((uint)address, (byte)original, out byte patched) ? patched : -1;
                    int got = core.Machine.RomPatch(address, (byte)original);
                    Assert.True(want == got, $"${address:X4} over {original:X2}: TryPatchRom {want}, MercuryRT {got}");
                }
            }
            Assert.Equal(-1, core.Machine.RomPatch(0x8000, 0));
            core.Dispose();
        }

        // A second pad's buttons do not reach the joypad (§8.7.4).
        [Fact]
        public void A_second_ports_buttons_are_ignored()
        {
            CoreOptions.BatteryRamDisabled = true;
            string rom = Rom("ports", CountingRom(0x03));
            using var rt = (MercuryRtCore)CoreFactory.Load(rom, engine: CoreCatalog.MercuryRtEngine).Core;
            rt.SetButton(1, PadButton.A, true);
            rt.SetButton(0, PadButton.B, true);
            // A v1 engine hands a press to its machine as the next frame starts - see Diamond_Players.md §12.7.
            rt.RunFrame();
            rt.WriteSpace(MercuryRtCore.SpaceCpuBus, 0xFF00, 0x10);
            Assert.Equal(0b1101, rt.ReadSpace(MercuryRtCore.SpaceCpuBus, 0xFF00) & 0x0F);
        }

        // Every named space sized as the board and the console make it, and an unknown one empty - mutant M22 (§8.4).
        [Theory]
        [InlineData((byte)0x00, (byte)0x00, 0)]
        [InlineData((byte)0x03, (byte)0x02, 0x2000)]
        public void Every_space_has_its_size(byte kind, byte ramCode, int ram)
        {
            CoreOptions.BatteryRamDisabled = true;
            byte[] image = SyntheticGbRom.Build(cartridgeType: kind, ramSizeCode: ramCode, patches: (0, new byte[] { 0x18, 0xFE }));
            using var rt = (MercuryRtCore)CoreFactory.Load(Rom($"spaces-{kind}", image), engine: CoreCatalog.MercuryRtEngine).Core;
            for (int i = 0; i < 5; i++) rt.RunFrame();
            var sizes = new Dictionary<string, int> { ["ROM"] = image.Length, ["VRAM"] = 0x2000, ["CARTRAM"] = ram, ["WRAM"] = 0x2000, ["OAM"] = 0xA0, ["HRAM"] = 0x7F, ["CPUBUS"] = 0x10000, ["NOSUCH"] = 0 };
            foreach (var (space, size) in sizes) Assert.True(size == rt.SpaceSize(space), $"{space}: {rt.SpaceSize(space)}, not {size}");
            Assert.Equal(image[0x150], rt.ReadSpace("ROM", 0x150));
            Assert.Equal(0, rt.ReadSpace("NOSUCH", 0));
        }

        [Fact]
        public void The_debug_target_shows_mercuryrts_machine_and_writes_reach_it()
        {
            CoreOptions.BatteryRamDisabled = true;
            string rom = Rom("debug", CountingRom(0x03));
            CoreBundle rt = CoreFactory.Load(rom, engine: CoreCatalog.MercuryRtEngine);
            var core = (MercuryRtCore)rt.Core;
            for (int i = 0; i < 200; i++) core.RunFrame();
            rt.DebugTarget.RefreshProviders();

            var (state, layout) = core.Snapshot();
            var registers = rt.DebugTarget.CpuRegisters.Current.ToDictionary(r => r.Name, r => (long)r.Value);
            Assert.Equal(layout.Get(state, "Cpu.PC"), registers["PC"]);
            Assert.Equal(layout.Get(state, "Cpu.SP"), registers["SP"]);
            Assert.Equal(layout.Get(state, "Cpu.H") << 8 | layout.Get(state, "Cpu.L"), registers["HL"]);
            Assert.Equal(layout.Get(state, "Cpu.<Cycles>k__BackingField"), registers["Cycles"]);
            Assert.Equal(core.TotalFrames, rt.DebugTarget.FrameCount);
            IDebugMemorySpace wram = rt.DebugTarget.GetMemorySpaces().Single(s => s.Name == "WRAM");
            Assert.Equal(layout.Bytes(state, "Bus.Wram")[0], wram.Read(0));
            wram.Write(0x20, 0x77);
            Assert.Equal(0x77, core.ReadSpace(MercuryRtCore.SpaceWram, 0x20));
            _output.WriteLine(string.Join(",", registers.Select(r => $"{r.Key}={r.Value}")));
            core.Dispose();
        }
    }
}
