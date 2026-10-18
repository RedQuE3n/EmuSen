using EmuSen.Cores.Native;
using System;
using System.IO;
using System.Linq;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Mercury_Native.md §6.1's three defects, each held on MercuryRT through ICore - see Mercury_Native.md §9.
    [Collection(TestCollections.ProcessGlobals)]
    public class MercuryDefectTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _dir = Path.Combine(Path.GetTempPath(), "mercury_defects_" + Guid.NewGuid().ToString("N"));

        public MercuryDefectTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_dir);
            DataStore.OverrideDirectory = Path.Combine(_dir, "Home");
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_dir, recursive: true); } catch (IOException) { }
        }

        private static CoreEngine Make(bool battery = false)
        {
            Assert.True(MercuryEngines.Available, MercuryEngines.Report);
            return MercuryEngines.Create(battery ? CoreOptions.Default : TestCores.NoBattery);
        }

        private string Rom(string name, byte[] image)
        {
            string sub = Path.Combine(_dir, name);
            Directory.CreateDirectory(sub);
            string path = Path.Combine(sub, name + ".gb");
            File.WriteAllBytes(path, image);
            return path;
        }

        // D1: 600 frames of 70,224 cycles are 10.0456 s of console time, so 44,100 Hz is 443,012.0 stereo frames; the integer divide made 443,520.
        [Fact]
        public void D1_the_mixer_makes_its_labelled_rate_of_samples_a_second()
        {
            using CoreEngine core = Make();
            core.LoadRom(Rom("rate", SyntheticGbRom.Build(patches: (0, new byte[] { 0x18, 0xFE }))));
            const int frames = 600;
            long stereo = 0;
            for (int f = 0; f < frames; f++)
            {
                core.RunFrame();
                stereo += core.DequeueAudioSamples(int.MaxValue).Length / 2;
            }
            double seconds = frames * (double)MercuryEngines.CyclesPerFrame / MercuryEngines.CpuClockHz;
            double expected = seconds * core.AudioSampleRate;
            _output.WriteLine($"{stereo} stereo frames in {seconds:F4} s of console time, {stereo / seconds:F1} a second, labelled {core.AudioSampleRate}; expected {expected:F1}");
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
        [InlineData(false)]
        [InlineData(true)]
        public void D2_the_mbc3_clock_keeps_console_time_in_double_speed(bool doubleSpeed)
        {
            using CoreEngine core = Make();
            core.LoadRom(Rom($"clock-{doubleSpeed}", ClockRom(doubleSpeed)));
            for (int f = 0; f < 1200; f++) core.RunFrame();
            bool fast = (core.ReadSpace("CPUBUS", 0xFF4D) & 0x80) != 0;
            byte seconds = core.ReadSpace("WRAM", 0);
            _output.WriteLine($"double speed {fast}: the clock read {seconds} s after 1200 frames");
            Assert.Equal(doubleSpeed, fast);
            Assert.Equal(20, seconds);
        }

        // Counts in WRAM and copies the count to cart RAM, so a battery save always has something new to write.
        private static byte[] BatteryRom() => SyntheticGbRom.Build(cartridgeType: 0x03, ramSizeCode: 0x02, patches: (0, new byte[]
        {
            0x3E, 0x0A, 0xEA, 0x00, 0x00, 0x21, 0x00, 0xC0, 0x34, 0x7E, 0xEA, 0x00, 0xA0, 0x18, 0xF8,
        }));

        // D3: a state saved at path A and loaded while playing path B must leave the battery save B's, and a --nobattery loader must write nothing.
        [Theory]
        [InlineData(false, false)]
        [InlineData(true, false)]
        [InlineData(false, true)]
        public void D3_a_state_carries_no_host_path(bool saverWithoutBattery, bool loaderWithoutBattery)
        {
            byte[] image = BatteryRom();
            string a = Rom($"a-{saverWithoutBattery}-{loaderWithoutBattery}", image), b = Rom($"b-{saverWithoutBattery}-{loaderWithoutBattery}", image);
            string srmA = SaveLibrary.SramPathFor(a, BatterySave.GameBoy), srmB = SaveLibrary.SramPathFor(b, BatterySave.GameBoy);
            string stray = Path.Combine(Environment.CurrentDirectory, ".tmp");
            bool strayBefore = File.Exists(stray);

            using CoreEngine saver = Make(battery: !saverWithoutBattery);
            saver.LoadRom(a);
            for (int f = 0; f < 10; f++) saver.RunFrame();
            using var state = new MemoryStream();
            saver.SaveState(state);

            using CoreEngine loader = Make(battery: !loaderWithoutBattery);
            loader.LoadRom(b);
            loader.LoadState(new MemoryStream(state.ToArray()));
            while (loader.TotalFrames < 300) loader.RunFrame();

            bool strayWritten = !strayBefore && File.Exists(stray);
            if (strayWritten) File.Delete(stray);
            _output.WriteLine($"A's .srm {File.Exists(srmA)}, B's .srm {File.Exists(srmB)}, a .tmp in the working directory {strayWritten}");
            Assert.False(strayWritten, "the loader wrote its cart RAM to .tmp in the working directory");
            Assert.False(File.Exists(srmA), "the loader wrote the saver's battery save");
            Assert.Equal(!loaderWithoutBattery, File.Exists(srmB));
        }

        // The bench's input: Start for five frames of every ninety, then A for five.
        private static uint GameScript(int frame) => (frame % 90) switch
        {
            >= 0 and < 5 => 1u << 7,
            >= 45 and < 50 => 1u << 4,
            _ => 0u,
        };

        // What a version-11 state carries after the walks, which version 5 does not: the mixer's three doubles, OAM DMA's start delay, the sweep's flag, the frozen LY=LYC bit - see Mercury_Native.md §10, §13.
        private const int MixerStateBytes = 3 * sizeof(double) + sizeof(int) + sizeof(bool) + sizeof(bool);

        // Real games' version-5 states from the unmodified build, at frames 3000 and 3600, from a directory kept outside the repository; absent, not run.
        [Fact]
        public void D3_real_games_version_5_states_resume_where_the_unmodified_build_went()
        {
            string? roms = Environment.GetEnvironmentVariable(MercuryGoldenTests.GamesVariable);
            string? states = Environment.GetEnvironmentVariable("EMUSEN_MERCURY_V5_STATES");
            if (roms is null || states is null || !Directory.Exists(roms) || !Directory.Exists(states))
            {
                _output.WriteLine("EMUSEN_MERCURYRT_ROMS or EMUSEN_MERCURY_V5_STATES unset, not run");
                return;
            }
            foreach (string path in Directory.GetFiles(roms, "*.gb*").Order(StringComparer.Ordinal))
            {
                string name = Path.GetFileNameWithoutExtension(path);
                string early = Path.Combine(states, $"{name}-3000.state"), late = Path.Combine(states, $"{name}-3600.state");
                if (!File.Exists(early) || !File.Exists(late)) continue;
                using MercuryRig run = MercuryRig.Open(path), then = MercuryRig.Open(path);
                run.LoadState(new MemoryStream(File.ReadAllBytes(early)));
                for (int f = 3000; f < 3600; f++)
                {
                    uint mask = GameScript(f);
                    for (int b = 0; b < MercuryGolden.MaskOrder.Length; b++) run.SetButton(0, MercuryGolden.MaskOrder[b], (mask & (1u << b)) != 0);
                    run.RunFrame();
                }
                then.LoadState(new MemoryStream(File.ReadAllBytes(late)));
                // The joypad's request (§13.7) is one the unmodified build never made; none of the four games enables it, so it is masked on both sides.
                then.SetField("Bus.InterruptFlags", then.Field("Bus.InterruptFlags") & 0x0F);
                run.SetField("Bus.InterruptFlags", run.Field("Bus.InterruptFlags") & 0x0F);
                byte[] want = then.State(), got = run.State();
                // Version 5 carries no mixer, so the loaded frame-3600 state's is not the game's: the comparison ends where version 8's begins - see Mercury_Native.md §13.1, §13.4.
                int body = want.Length - MixerStateBytes;
                int same = want.AsSpan(0, body).CommonPrefixLength(got);
                Assert.True(same == body && want.Length == got.Length, $"{name}: 600 frames on from the old build's frame-3000 state differ at byte {same} from its frame-3600 state");
                _output.WriteLine($"{name}: version 5 at frame 3000 ({new FileInfo(early).Length} bytes), run 600 frames, equals the old build's frame 3600 ({want.Length} bytes in version 11)");
            }
        }
    }
}
