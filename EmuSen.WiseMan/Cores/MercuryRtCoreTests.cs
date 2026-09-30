using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Cheats;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MercuryRtCore against MercuryCore through ICore alone: the engine setting, cheats, battery saves, states across engines, the debugger's mirror - see Mercury_Native.md §8.3.
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
        public void The_engine_setting_builds_mercuryrt_and_the_default_builds_mercury()
        {
            Assert.True(MercuryRtCore.Available, MercuryNative.Report);
            string rom = Rom("engine", CountingRom(0x03));
            CoreBundle rt = CoreFactory.Load(rom, engine: CoreCatalog.MercuryRtEngine);
            CoreBundle cs = CoreFactory.Load(rom);
            Assert.IsType<MercuryRtCore>(rt.Core);
            Assert.IsType<MercuryCore>(cs.Core);
            Assert.Null(rt.Notice);
            Assert.Equal(CoreCatalog.MercuryEngine, CoreCatalog.EngineFor("GB")!.Default);
            Assert.Equal("GB", rt.Core.CoreName);
            Assert.Equal(cs.Core.FrameRateHz, rt.Core.FrameRateHz);
        }

        // A RAM poke and a compare-gated ROM patch on both engines, the states, pictures and sound compared every frame.
        [Fact]
        public void Cheats_of_both_kinds_act_identically()
        {
            CoreOptions.BatteryRamDisabled = true;
            string rom = Rom("cheats", CountingRom(0x03));
            var cs = (MercuryCore)CoreFactory.Load(rom).Core;
            var rt = (MercuryRtCore)CoreFactory.Load(rom, engine: CoreCatalog.MercuryRtEngine).Core;
            foreach (var registry in new[] { cs.Cheats, rt.Cheats })
            {
                registry.AddRomPatch(0x0300, 0x03, compare: 0x01, "count by three");
                registry.AddRamPoke(MercuryCore.SpaceCpuBus, 0xC010, 0x5A, "a poke");
            }
            Compare(cs, rt, 300, "cheats on");
            Assert.Equal(0x5A, rt.ReadSpace(MercuryCore.SpaceWram, 0x10));
            foreach (var registry in new[] { cs.Cheats, rt.Cheats }) registry.MasterEnabled = false;
            Compare(cs, rt, 60, "master off");
            _output.WriteLine($"WRAM count {cs.Bus!.Wram[0]} on both after 360 frames");
        }

        // One ROM path for both, and so one save in the Saves folder; C#'s save is moved aside before MercuryRT's frame 300.
        [Fact]
        public void A_battery_save_loads_and_writes_the_same_bytes()
        {
            CoreOptions.BatteryRamDisabled = false;
            string rom = Rom("battery", CountingRom(0x03)), srm = SaveLibrary.SramPathFor(rom, BatterySave.GameBoy);
            Directory.CreateDirectory(Path.GetDirectoryName(srm)!);
            var saved = new byte[8192];
            new Random(64).NextBytes(saved);
            File.WriteAllBytes(srm, saved);

            var cs = new MercuryCore();
            cs.LoadRom(rom);
            using var rt = new MercuryRtCore();
            rt.LoadRom(rom);
            Assert.Equal(cs.ReadSpace(MercuryCore.SpaceCartRam, 17), rt.ReadSpace(MercuryCore.SpaceCartRam, 17));
            Compare(cs, rt, 299, "battery");
            cs.RunFrame();
            byte[] fromCs = File.ReadAllBytes(srm);
            File.Delete(srm);
            rt.RunFrame();
            byte[] fromRt = File.ReadAllBytes(srm);
            Assert.False(fromCs.SequenceEqual(saved), "frame 300's save wrote nothing new");
            Assert.True(fromCs.SequenceEqual(fromRt), "the two engines wrote different battery saves");
        }

        [Fact]
        public void A_state_saved_on_either_engine_loads_on_the_other()
        {
            CoreOptions.BatteryRamDisabled = true;
            string rom = Rom("states", CountingRom(0x1B));
            var cs = (MercuryCore)CoreFactory.Load(rom).Core;
            var rt = (MercuryRtCore)CoreFactory.Load(rom, engine: CoreCatalog.MercuryRtEngine).Core;
            Compare(cs, rt, 120, "before");

            string fromCs = Path.Combine(_dir, "cs.state"), fromRt = Path.Combine(_dir, "rt.state");
            cs.SaveState(fromCs);
            rt.SaveState(fromRt);
            Assert.True(File.ReadAllBytes(fromCs).SequenceEqual(File.ReadAllBytes(fromRt)), "the two engines saved different states");
            for (int i = 0; i < 50; i++) { cs.RunFrame(); rt.RunFrame(); }
            cs.LoadState(fromRt);
            rt.LoadState(fromCs);
            Compare(cs, rt, 120, "after crossing");

            Assert.Throws<InvalidDataException>(() => rt.LoadState(new MemoryStream(new byte[] { 0x4D, 0x41, 0x52, 0x54, 5, 0, 0, 0 })));
            Assert.Throws<InvalidDataException>(() => rt.LoadState(new MemoryStream(new byte[] { 0x4D, 0x45, 0x52, 0x43, 4, 0, 0, 0 })));
            Assert.Throws<InvalidOperationException>(() => new MercuryRtCore().RunFrame());
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

        // A second pad's buttons reach neither engine's joypad, as MercuryCore.SetButton ignores them (§8.7.4).
        [Fact]
        public void A_second_ports_buttons_are_ignored_on_both_engines()
        {
            CoreOptions.BatteryRamDisabled = true;
            string rom = Rom("ports", CountingRom(0x03));
            var cs = (MercuryCore)CoreFactory.Load(rom).Core;
            using var rt = (MercuryRtCore)CoreFactory.Load(rom, engine: CoreCatalog.MercuryRtEngine).Core;
            foreach (ICore core in new ICore[] { cs, rt })
            {
                core.SetButton(1, PadButton.A, true);
                core.SetButton(0, PadButton.B, true);
            }
            cs.WriteSpace(MercuryCore.SpaceCpuBus, 0xFF00, 0x10);
            rt.WriteSpace(MercuryCore.SpaceCpuBus, 0xFF00, 0x10);
            Assert.Equal(0b1101, cs.ReadSpace(MercuryCore.SpaceCpuBus, 0xFF00) & 0x0F);
            Assert.Equal(0b1101, rt.ReadSpace(MercuryCore.SpaceCpuBus, 0xFF00) & 0x0F);
        }

        // Every named space read and sized as MercuryCore reads it, wrapping, past the ROM's end and on a cart with no RAM - mutant M22 (§8.4).
        [Theory]
        [InlineData((byte)0x00, (byte)0x00)]
        [InlineData((byte)0x03, (byte)0x02)]
        public void Every_space_reads_and_sizes_as_mercurys(byte kind, byte ramCode)
        {
            CoreOptions.BatteryRamDisabled = true;
            string rom = Rom($"spaces-{kind}", SyntheticGbRom.Build(cartridgeType: kind, ramSizeCode: ramCode, patches: (0, new byte[] { 0x18, 0xFE })));
            var cs = (MercuryCore)CoreFactory.Load(rom).Core;
            var rt = (MercuryRtCore)CoreFactory.Load(rom, engine: CoreCatalog.MercuryRtEngine).Core;
            for (int i = 0; i < 5; i++) { cs.RunFrame(); rt.RunFrame(); }
            foreach (string space in MercuryMachine.SpaceNames.Append("NOSUCH"))
            {
                Assert.Equal(cs.SpaceSize(space), rt.SpaceSize(space));
                foreach (int address in new[] { 0, 1, 0x7F, 0xA0, 0x1FFF, 0x2000, 0x7FFF, 0x8000, 0xFF44, 0xFFFF, 0x10000, -1, -0x2001 })
                {
                    if (space == "ROM" && address < 0) continue;
                    Assert.True(cs.ReadSpace(space, address) == rt.ReadSpace(space, address), $"{space} at {address}: C# {cs.ReadSpace(space, address)}, Rust {rt.ReadSpace(space, address)}");
                }
            }
        }

        [Fact]
        public void The_debug_target_shows_mercuryrts_machine_and_writes_reach_it()
        {
            CoreOptions.BatteryRamDisabled = true;
            string rom = Rom("debug", CountingRom(0x03));
            CoreBundle cs = CoreFactory.Load(rom), rt = CoreFactory.Load(rom, engine: CoreCatalog.MercuryRtEngine);
            for (int i = 0; i < 200; i++) { cs.Core.RunFrame(); rt.Core.RunFrame(); }
            cs.DebugTarget.RefreshProviders();
            rt.DebugTarget.RefreshProviders();

            string Registers(IDebugTarget t) => string.Join(",", t.CpuRegisters.Current.Select(r => $"{r.Name}={r.Value}"));
            Assert.Equal(Registers(cs.DebugTarget), Registers(rt.DebugTarget));
            Assert.Equal(cs.DebugTarget.FrameCount, rt.DebugTarget.FrameCount);
            IDebugMemorySpace wram = rt.DebugTarget.GetMemorySpaces().Single(s => s.Name == "WRAM");
            Assert.Equal(((MercuryCore)cs.Core).Bus!.Wram[0], wram.Read(0));
            wram.Write(0x20, 0x77);
            Assert.Equal(0x77, ((MercuryRtCore)rt.Core).ReadSpace(MercuryCore.SpaceWram, 0x20));
            _output.WriteLine(Registers(rt.DebugTarget));
        }

        [Fact]
        public void Real_games_run_identically_through_icore_with_input()
        {
            string? folder = Environment.GetEnvironmentVariable(MercuryRtStateTests.RomsVariable);
            if (folder is null || !Directory.Exists(folder))
            {
                _output.WriteLine($"{MercuryRtStateTests.RomsVariable} unset, not run");
                return;
            }
            CoreOptions.BatteryRamDisabled = true;
            foreach (string path in Directory.GetFiles(folder, "*.gb*").Order(StringComparer.Ordinal))
            {
                var cs = (MercuryCore)CoreFactory.Load(path).Core;
                var rt = (MercuryRtCore)CoreFactory.Load(path, engine: CoreCatalog.MercuryRtEngine).Core;
                Compare(cs, rt, 1500, Path.GetFileName(path), press: true);
                _output.WriteLine($"{Path.GetFileName(path)}: 1500 frames identical through ICore");
            }
        }

        private static void Compare(ICore cs, ICore rt, int frames, string what, bool press = false)
        {
            for (int f = 0; f < frames; f++)
            {
                if (press)
                {
                    int k = f % 90;
                    foreach (var core in new[] { cs, rt })
                    {
                        core.SetButton(0, PadButton.Start, k < 5);
                        core.SetButton(0, PadButton.A, k is >= 45 and < 50);
                    }
                }
                cs.RunFrame();
                rt.RunFrame();
                Assert.Equal(cs.TotalFrames, rt.TotalFrames);
                using var a = new MemoryStream();
                using var b = new MemoryStream();
                cs.SaveState(a);
                rt.SaveState(b);
                int same = a.ToArray().AsSpan().CommonPrefixLength(b.ToArray());
                Assert.True(same == a.Length && a.Length == b.Length, $"{what}, frame {f}: states differ at byte {same}");
                Assert.True(cs.GetFrameBufferRgba().AsSpan().SequenceEqual(rt.GetFrameBufferRgba()), $"{what}, frame {f}: pictures differ");
                Assert.True(cs.DequeueAudioSamples(int.MaxValue).AsSpan().SequenceEqual(rt.DequeueAudioSamples(int.MaxValue)), $"{what}, frame {f}: sound differs");
            }
        }
    }
}
