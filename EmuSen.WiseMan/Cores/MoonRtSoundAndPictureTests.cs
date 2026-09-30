using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Moon;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Stage 3: the mixer, the sample queue and the pixel writes, sample for sample and byte for byte against the C# Moon - see Moon_Native.md §8.2.
    public class MoonRtSoundAndPictureTests
    {
        public const string StatesVariable = "EMUSEN_MOONRT_STATES";

        private readonly ITestOutputHelper _output;

        public MoonRtSoundAndPictureTests(ITestOutputHelper output) => _output = output;

        private static MoonRtPair Busy(bool maskCycle = false) =>
            new(BoardPrograms.Build(4, 8, cycleIrq: false, maskCycle: maskCycle), skipRendering: false);

        // Undrained for up to seven frames under limits from 0 to past a frame's worth, so the drop-oldest rule runs on both.
        [Fact]
        public void A_low_audio_limit_drops_the_oldest_pairs_alike()
        {
            var pair = Busy();
            pair.DrainEvery = 7;
            foreach (int limit in new[] { 2000, 1001, 3, 0, 500, 128000 })
            {
                pair.Csharp.Apu!.Samples.Limit = limit;
                pair.Rust.SetAudioLimit(limit);
                pair.Run(49, MoonRtMachineTests.Pads);
                Assert.True(pair.Csharp.Apu.BufferedSamples <= Math.Max(limit, 0));
            }
            _output.WriteLine(pair.Summary);
        }

        // Every channel alone, all muted and none; the queue is drained every frame.
        [Theory]
        [InlineData(0x00)]
        [InlineData(0x1E)]
        [InlineData(0x1D)]
        [InlineData(0x1B)]
        [InlineData(0x17)]
        [InlineData(0x0F)]
        [InlineData(0x1F)]
        public void A_muted_channel_mixes_alike(int mask)
        {
            var pair = Busy();
            Mute(pair, mask);
            pair.Run(300, MoonRtMachineTests.Pads);
            _output.WriteLine($"mask {mask:X2}: {pair.Summary}");
        }

        private static void Mute(MoonRtPair pair, int mask)
        {
            for (int ch = 0; ch < 5; ch++) pair.Csharp.Apu!.SetChannelMuted(ch, (mask & (1 << ch)) != 0);
            pair.Rust.SetMutes((uint)mask);
        }

        // The bench games with each channel alone: the noise channel's new periods and the DMC's samples, sample for sample.
        [Fact]
        public void The_bench_games_mix_each_channel_alike()
        {
            string? folder = Environment.GetEnvironmentVariable(MoonRtStateTests.RomsVariable);
            if (folder is null || !Directory.Exists(folder))
            {
                _output.WriteLine($"{MoonRtStateTests.RomsVariable} unset, not run");
                return;
            }
            foreach (string path in Directory.GetFiles(folder, "*.nes").Order(StringComparer.Ordinal))
            {
                for (int solo = 0; solo < 5; solo++)
                {
                    var pair = new MoonRtPair(File.ReadAllBytes(path), skipRendering: true, stateEvery: 60);
                    Mute(pair, 0x1F & ~(1 << solo));
                    pair.Run(1200, Script);
                    _output.WriteLine($"{Path.GetFileName(path)}, channel {solo} alone: {pair.Summary}");
                }
            }
        }

        // SkipRendering turned on and off mid-run; every frame drawn after it is turned off again is compared.
        [Fact]
        public void Skip_rendering_switched_mid_run_draws_alike()
        {
            var pair = Busy();
            for (int i = 0; i < 8; i++)
            {
                pair.SetSkip(i % 2 == 0);
                pair.Run(37 + i, MoonRtMachineTests.Pads);
            }
            _output.WriteLine(pair.Summary);
        }

        // A palette entry rewritten each frame, with every combination of $2001's grayscale and emphasis bits.
        [Fact]
        public void Palette_writes_grayscale_and_emphasis_draw_alike()
        {
            var pair = Busy(maskCycle: true);
            pair.Run(512, MoonRtMachineTests.Pads);
            Assert.True(pair.Csharp.Bus!.Ram[1] > 0, "the NMI handler never ran");
            Assert.Equal(0x1E, pair.Csharp.Ppu!.Mask & 0x1E);
            _output.WriteLine(pair.Summary);
        }

        // The library's own saved states, with sound and picture compared every frame after the load.
        [Fact]
        public void Library_states_sound_and_draw_alike()
        {
            string? folder = Environment.GetEnvironmentVariable(StatesVariable);
            if (folder is null || !Directory.Exists(folder))
            {
                _output.WriteLine($"{StatesVariable} unset, not run");
                return;
            }
            int runs = 0;
            foreach (string rom in Directory.GetFiles(folder, "*.nes").Order(StringComparer.Ordinal))
            {
                byte[] image = File.ReadAllBytes(rom);
                foreach (string state in Directory.GetFiles(folder, Path.GetFileNameWithoutExtension(rom) + ".*state").Order(StringComparer.Ordinal))
                {
                    MoonCore from = MoonRtStateTests.Load(image);
                    using (var stream = File.OpenRead(state)) from.LoadState(stream);
                    MoonRtPair pair;
                    try
                    {
                        pair = new MoonRtPair(image, skipRendering: false, transferAt: from);
                    }
                    catch (MoonRtPair.BothRefused)
                    {
                        continue;
                    }
                    int frames = 0;
                    for (; frames < 900; frames++)
                        if (!pair.Frame(Script, frames)) break;
                    runs++;
                    _output.WriteLine($"{Path.GetFileName(state)}: {pair.Summary}{(frames < 900 ? $", both stopped at frame {frames}" : "")}");
                }
            }
            _output.WriteLine($"{runs} states");
        }

        private static (uint, uint)? Script(int frame) => (frame % 90) switch
        {
            0 => (1u << 3, 0u),
            5 => (0u, 0u),
            45 => (1u << 0, 0u),
            50 => (0u, 0u),
            _ => null,
        };
    }
}
