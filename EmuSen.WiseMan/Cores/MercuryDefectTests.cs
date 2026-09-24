using System;
using System.IO;
using System.Linq;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Memory;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Mercury_Native.md §6.1's three defects, each shown on both engines through ICore - see Mercury_Native.md §9.
    [Collection(TestCollections.ProcessGlobals)]
    public class MercuryDefectTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _dir = Path.Combine(Path.GetTempPath(), "mercury_defects_" + Guid.NewGuid().ToString("N"));

        public MercuryDefectTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_dir);
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = true;
            try { Directory.Delete(_dir, recursive: true); } catch (IOException) { }
        }

        public static TheoryData<string> Engines => new() { "C#", "Rust" };

        private static ICore Make(string engine)
        {
            if (engine == "C#") return new MercuryCore();
            Assert.True(MercuryRtCore.Available, MercuryNative.Report);
            return new MercuryRtCore();
        }

        private static byte ReadSpace(ICore core, string space, int address) => core switch
        {
            MercuryCore cs => cs.ReadSpace(space, address),
            MercuryRtCore rt => rt.ReadSpace(space, address),
            _ => throw new ArgumentException(core.GetType().Name),
        };

        private string Rom(string name, byte[] image)
        {
            string sub = Path.Combine(_dir, name);
            Directory.CreateDirectory(sub);
            string path = Path.Combine(sub, "game.gb");
            File.WriteAllBytes(path, image);
            return path;
        }

        // D1: 600 frames of 70,224 cycles are 10.0456 s of console time, so 44,100 Hz is 443,012.0 stereo frames; the integer divide made 443,520.
        [Theory]
        [MemberData(nameof(Engines))]
        public void D1_the_mixer_makes_its_labelled_rate_of_samples_a_second(string engine)
        {
            CoreOptions.BatteryRamDisabled = true;
            ICore core = Make(engine);
            core.LoadRom(Rom("rate-" + engine, SyntheticGbRom.Build(patches: (0, new byte[] { 0x18, 0xFE }))));
            const int frames = 600;
            long stereo = 0;
            for (int f = 0; f < frames; f++)
            {
                core.RunFrame();
                stereo += core.DequeueAudioSamples(int.MaxValue).Length / 2;
            }
            double seconds = frames * (double)MercuryCore.CyclesPerFrame / MercuryCore.CpuClockHz;
            double expected = seconds * core.AudioSampleRate;
            _output.WriteLine($"{engine}: {stereo} stereo frames in {seconds:F4} s of console time, {stereo / seconds:F1} a second, labelled {core.AudioSampleRate}; expected {expected:F1}");
            Assert.True(Math.Abs(stereo - expected) <= 1.0, $"{stereo} stereo frames, {stereo / seconds:F1} a second, where {core.AudioSampleRate} Hz wants {expected:F1}");
        }

        // Enables the clock, selects its seconds register, optionally switches to double speed, then latches the clock and copies the seconds to $C000 forever.
        private static byte[] ClockRom(bool doubleSpeed)
        {
            byte[] program =
            [
                0x3E, 0x0A, 0xEA, 0x00, 0x00, 0x3E, 0x08, 0xEA, 0x00, 0x40,
                .. doubleSpeed ? new byte[] { 0x3E, 0x01, 0xE0, 0x4D, 0x10, 0x00 } : new byte[6],
                0xAF, 0xEA, 0x00, 0x60, 0x3C, 0xEA, 0x00, 0x60, 0xFA, 0x00, 0xA0, 0xEA, 0x00, 0xC0, 0x18, 0xF0,
            ];
            return SyntheticGbRom.Build(romBanks: 4, cartridgeType: 0x10, ramSizeCode: 0x03, cgbFlag: 0xC0, patches: (0, program));
        }

        // D2: the cartridge's clock has its own crystal, so 1,200 frames (20.09 s of console time) read 20 s at either CPU speed.
        [Theory]
        [InlineData("C#", false)]
        [InlineData("C#", true)]
        [InlineData("Rust", false)]
        [InlineData("Rust", true)]
        public void D2_the_mbc3_clock_keeps_console_time_in_double_speed(string engine, bool doubleSpeed)
        {
            CoreOptions.BatteryRamDisabled = true;
            ICore core = Make(engine);
            core.LoadRom(Rom($"clock-{engine}-{doubleSpeed}", ClockRom(doubleSpeed)));
            for (int f = 0; f < 1200; f++) core.RunFrame();
            bool fast = (ReadSpace(core, "CPUBUS", 0xFF4D) & 0x80) != 0;
            byte seconds = ReadSpace(core, "WRAM", 0);
            _output.WriteLine($"{engine}, double speed {fast}: the clock read {seconds} s after 1200 frames");
            Assert.Equal(doubleSpeed, fast);
            Assert.Equal(20, seconds);
        }

        // The same program on both engines in lock-step, the clock's sub-second count compared in every frame's state.
        [Fact]
        public void D2_the_clock_in_double_speed_runs_identically_on_both_engines()
        {
            var pair = new MercuryRtPair(ClockRom(doubleSpeed: true), skipRendering: false);
            pair.Run(1200, null);
            Assert.True(pair.Csharp.Bus!.DoubleSpeed, "the program never reached double speed");
            Assert.Equal(20, pair.Csharp.Bus!.Wram[0]);
            _output.WriteLine(pair.Summary);
        }

        // Counts in WRAM and copies the count to cart RAM, so a battery save always has something new to write.
        private static byte[] BatteryRom() => SyntheticGbRom.Build(cartridgeType: 0x03, ramSizeCode: 0x02, patches: (0, new byte[]
        {
            0x3E, 0x0A, 0xEA, 0x00, 0x00, 0x21, 0x00, 0xC0, 0x34, 0x7E, 0xEA, 0x00, 0xA0, 0x18, 0xF8,
        }));

        // D3: a state saved at path A and loaded while playing path B must leave the battery save B's, and a --nobattery loader must write nothing.
        [Theory]
        [InlineData("C#", false, false)]
        [InlineData("C#", true, false)]
        [InlineData("C#", false, true)]
        [InlineData("Rust", false, false)]
        [InlineData("Rust", true, false)]
        [InlineData("Rust", false, true)]
        public void D3_a_state_carries_no_host_path(string engine, bool saverWithoutBattery, bool loaderWithoutBattery)
        {
            byte[] image = BatteryRom();
            string a = Rom($"a-{engine}-{saverWithoutBattery}-{loaderWithoutBattery}", image), b = Rom($"b-{engine}-{saverWithoutBattery}-{loaderWithoutBattery}", image);
            string srmA = Path.ChangeExtension(a, ".srm"), srmB = Path.ChangeExtension(b, ".srm");
            string stray = Path.Combine(Environment.CurrentDirectory, ".tmp");
            bool strayBefore = File.Exists(stray);

            CoreOptions.BatteryRamDisabled = saverWithoutBattery;
            ICore saver = Make(engine);
            saver.LoadRom(a);
            for (int f = 0; f < 10; f++) saver.RunFrame();
            using var state = new MemoryStream();
            saver.SaveState(state);

            CoreOptions.BatteryRamDisabled = loaderWithoutBattery;
            ICore loader = Make(engine);
            loader.LoadRom(b);
            loader.LoadState(new MemoryStream(state.ToArray()));
            while (loader.TotalFrames < 300) loader.RunFrame();

            bool strayWritten = !strayBefore && File.Exists(stray);
            if (strayWritten) File.Delete(stray);
            _output.WriteLine($"{engine}: A's .srm {File.Exists(srmA)}, B's .srm {File.Exists(srmB)}, a .tmp in the working directory {strayWritten}");
            Assert.False(strayWritten, "the loader wrote its cart RAM to .tmp in the working directory");
            Assert.False(File.Exists(srmA), "the loader wrote the saver's battery save");
            Assert.Equal(!loaderWithoutBattery, File.Exists(srmB));
        }

        // MercuryRtStateTests' program: counts in WRAM, copies the count to cart RAM, keys a pulse note and scrolls.
        private static readonly byte[] Busy =
        {
            0x3E, 0x0A, 0xEA, 0x00, 0x00, 0x3E, 0xF0, 0xE0, 0x12, 0x3E, 0x87, 0xE0, 0x14,
            0x21, 0x00, 0xC0, 0x34, 0x7E, 0xEA, 0x00, 0xA0, 0xE0, 0x43, 0x18, 0xF4,
        };

        // SHA-256 prefixes of the version-5 states the unmodified build (f2c7f99) wrote for Busy at frame 300, with its save path set to /mercury-v5/A.srm.
        public static TheoryData<byte, byte, byte, string> Version5Boards => new()
        {
            { 0x00, 0x00, 0x00, "A8F5A6AE499B01D8" }, { 0x08, 0x02, 0x00, "4CE8AB33C05B5A53" }, { 0x03, 0x02, 0x00, "C904450EE75DBBEC" },
            { 0x03, 0x03, 0x80, "A4947B8697CBAEC0" }, { 0x06, 0x00, 0x00, "45C6CF989CA6BA94" }, { 0x06, 0x00, 0xC0, "77EEB044A60922C5" },
            { 0x10, 0x03, 0x00, "414D74B71818B3E4" }, { 0x13, 0x03, 0xC0, "8616742760CEF290" }, { 0x1B, 0x03, 0x80, "72F2BFB3C1FD3EAF" },
            { 0x1E, 0x04, 0x00, "99523C6CA29E10ED" }, { 0x19, 0x00, 0xC0, "0BB6C2ED37B31BE9" },
        };

        private static MercuryCore LoadCs(byte[] rom)
        {
            string path = SyntheticGbRom.WriteTemp(rom);
            try
            {
                var core = new MercuryCore();
                core.LoadRom(path);
                return core;
            }
            finally
            {
                File.Delete(path);
            }
        }

        private static byte[] Save(MercuryCore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        private static string Sha(byte[] bytes) => Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(bytes))[..16];

        private static readonly System.Reflection.FieldInfo SavePath = typeof(Cartridge).GetField("_savePath", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)!;

        // Version 5's bytes from a running C# core: MercuryCore.SaveState's header and walks, the retired fields included (EmuSen_Save_States.md §7).
        private static byte[] WriteVersion5(MercuryCore core)
        {
            using var stream = new MemoryStream();
            using (var w = new BinaryWriter(stream, System.Text.Encoding.UTF8, leaveOpen: true))
            {
                w.Write(0x4352454Du);
                w.Write(5);
                w.Write(core.TotalFrames);
                w.Write((long)typeof(MercuryCore).GetField("_cyclesIntoFrame", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)!.GetValue(core)!);
                foreach (object part in new object[] { core.Cart!, core.Cart!.Mapper, core.Cpu!, core.Bus! }) StateSerializer.Write(w, part, includeRetired: true);
            }
            return stream.ToArray();
        }

        // D3's format half: version 6 carries neither the path nor the two copies, and the unmodified build's version-5 states still load, on both engines, dropping the path.
        [Theory]
        [MemberData(nameof(Version5Boards))]
        public void D3_version_5_states_the_unmodified_build_wrote_load_on_both_engines(byte kind, byte ramCode, byte cgb, string sha)
        {
            CoreOptions.BatteryRamDisabled = true;
            byte[] rom = SyntheticGbRom.Build(romBanks: 4, cartridgeType: kind, ramSizeCode: ramCode, cgbFlag: cgb, patches: (0, Busy));
            MercuryCore source = LoadCs(rom);
            for (int f = 0; f < 300; f++) source.RunFrame();
            SavePath.SetValue(source.Cart, "/mercury-v5/A.srm");
            byte[] v5 = WriteVersion5(source), v6 = Save(source);
            Assert.True(Sha(v5) == sha, $"the version-5 writer's bytes hash {Sha(v5)}, not the unmodified build's {sha}");
            Assert.Equal(7, BitConverter.ToInt32(v6, 4));
            Assert.DoesNotContain("mercury-v5", System.Text.Encoding.UTF8.GetString(v6));

            MercuryCore cs = LoadCs(rom);
            cs.LoadState(new MemoryStream(v5));
            Assert.True(Save(cs).AsSpan().SequenceEqual(v6), "C# read version 5 into a different machine");
            Assert.Null(SavePath.GetValue(cs.Cart));

            using var rt = new MercuryMachine(rom);
            rt.Load(v5);
            Assert.True(rt.Save().AsSpan().SequenceEqual(v6), "MercuryRT read version 5 into a different machine");

            var pair = new MercuryRtPair(rom, skipRendering: false, state: v5);
            pair.Run(120, null);
            _output.WriteLine($"type ${kind:X2} cgb ${cgb:X2}: version 5 {v5.Length} bytes (hash {sha}), version 6 {v6.Length}; {pair.Summary}");
        }

        // The bench's input: Start for five frames of every ninety, then A for five.
        private static uint GameScript(int frame) => (frame % 90) switch
        {
            >= 0 and < 5 => 1u << 7,
            >= 45 and < 50 => 1u << 4,
            _ => 0u,
        };

        // Real games' version-5 states from the unmodified build, at frames 3000 and 3600, from a directory kept outside the repository; absent, not run.
        [Fact]
        public void D3_real_games_version_5_states_resume_where_the_unmodified_build_went()
        {
            string? roms = Environment.GetEnvironmentVariable(MercuryRtStateTests.RomsVariable);
            string? states = Environment.GetEnvironmentVariable("EMUSEN_MERCURY_V5_STATES");
            if (roms is null || states is null || !Directory.Exists(roms) || !Directory.Exists(states))
            {
                _output.WriteLine("EMUSEN_MERCURYRT_ROMS or EMUSEN_MERCURY_V5_STATES unset, not run");
                return;
            }
            CoreOptions.BatteryRamDisabled = true;
            foreach (string path in Directory.GetFiles(roms, "*.gb*").Order(StringComparer.Ordinal))
            {
                string name = Path.GetFileNameWithoutExtension(path);
                string early = Path.Combine(states, $"{name}-3000.state"), late = Path.Combine(states, $"{name}-3600.state");
                if (!File.Exists(early) || !File.Exists(late)) continue;
                byte[] rom = File.ReadAllBytes(path);
                var pair = new MercuryRtPair(rom, skipRendering: false, state: File.ReadAllBytes(early));
                pair.Run(600, f => GameScript(f + 3000));
                MercuryCore then = LoadCs(rom);
                then.LoadState(new MemoryStream(File.ReadAllBytes(late)));
                byte[] want = Save(then), got = Save(pair.Csharp);
                int same = want.AsSpan().CommonPrefixLength(got);
                Assert.True(same == want.Length && want.Length == got.Length, $"{name}: 600 frames on from the old build's frame-3000 state differ at byte {same} from its frame-3600 state");
                _output.WriteLine($"{name}: version 5 at frame 3000 ({new FileInfo(early).Length} bytes), run 600 frames on both engines, equals the old build's frame 3600 ({want.Length} bytes in version 6); {pair.Summary}");
            }
        }
    }
}
