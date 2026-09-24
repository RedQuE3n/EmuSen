using System;
using System.IO;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
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
    }
}
