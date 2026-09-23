using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.Galaxia.Input;
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
        }

        public void Dispose()
        {
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

        // One ROM path for both, since the path is in the state (Mercury_Native.md §6.1, D3); C#'s save is moved aside before MercuryRT's frame 300.
        [Fact]
        public void A_battery_save_loads_and_writes_the_same_bytes()
        {
            CoreOptions.BatteryRamDisabled = false;
            string rom = Rom("battery", CountingRom(0x03)), srm = Path.ChangeExtension(rom, ".srm");
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
