using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Crystal.Systems.Nes;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MoonRT's picture before the palette through the core ABI: each frame's nes.ppu is its RGB frame through the NES palette, and keeping it changes no golden - see EmuSen_CoreAPI.md §6.30, Moon_Native.md §9.7.
    [Collection(TestCollections.ProcessGlobals)]
    public class MoonRtRawPictureTests
    {
        private readonly ITestOutputHelper _output;

        public MoonRtRawPictureTests(ITestOutputHelper output)
        {
            _output = output;
            CoreOptions.BatteryRamDisabled = true;
        }

        public static TheoryData<string> DrawnRuns
        {
            get
            {
                var data = new TheoryData<string>();
                foreach (var run in MoonGoldenTests.FrameRuns().Where(r => !r.SkipRendering)) data.Add(run.Name);
                return data;
            }
        }

        // Every pixel's nes.ppu value through the NES system pack's palette is the frame's pixel; the emphasis bits seen are added to the set.
        private static void Held(MoonRtCore core, int frame, HashSet<int> emphasis)
        {
            RawPicture raw = core.RawPicture ?? throw new Xunit.Sdk.XunitException($"frame {frame}: no picture before the palette");
            Assert.Equal((RawEncodings.NesPpu, 256, 240, 2, 512), (raw.Encoding, raw.Width, raw.Height, raw.Element, raw.Stride));
            byte[] rgba = core.GetFrameBufferRgba();
            for (int i = 0; i < 256 * 240; i++)
            {
                int v = raw.Values[2 * i] | raw.Values[2 * i + 1] << 8, c = (v & 0x3F) * 3;
                if (v >> 9 != 0 || rgba[4 * i] != NesDebugViews.MasterPalette[c] || rgba[4 * i + 1] != NesDebugViews.MasterPalette[c + 1] || rgba[4 * i + 2] != NesDebugViews.MasterPalette[c + 2])
                    Assert.Fail($"frame {frame}, pixel ({i % 256}, {i / 256}): value {v:X3} against the frame's {rgba[4 * i]:X2}{rgba[4 * i + 1]:X2}{rgba[4 * i + 2]:X2}");
                emphasis.Add(v >> 6);
            }
            core.ReturnFrameBuffer(rgba);
            core.ReturnFrameBuffer(raw.Values);
        }

        private string Grade(MoonGoldenRun run)
        {
            var emphasis = new HashSet<int>();
            int frames = 0;
            GoldenTrace trace = MoonGolden.Run(run, setup: c => c.RawEncoding = RawEncodings.NesPpu, frame: (f, c) => { Held(c, f, emphasis); frames++; });
            var golden = GoldenTrace.Parse(File.ReadAllText(MoonGolden.TracePath(run.Name)));
            if (golden.FirstDifference(trace) is { } difference) Assert.Fail($"{run.Name}, keeping nes.ppu: {difference}");
            return $"{run.Name}: {frames} frames, each its frame through the palette, emphasis values {string.Join(" ", emphasis.Order())}; the golden as recorded";
        }

        [Theory]
        [MemberData(nameof(DrawnRuns))]
        public void Beside_a_golden_run_the_raw_picture_is_its_frame_through_the_palette_and_the_golden_stands(string name) =>
            _output.WriteLine(Grade(MoonGoldenTests.FrameRuns().Single(r => r.Name == name)));

        [Fact]
        public void Beside_the_games_goldens_the_raw_picture_is_each_frame_through_the_palette()
        {
            var runs = MoonGoldenTests.GameRuns().ToList();
            if (runs.Count == 0) _output.WriteLine($"{MoonGoldenTests.GamesVariable} unset or holding none of the games, not run");
            foreach (MoonGoldenRun run in runs) _output.WriteLine(Grade(run));
        }

        // The host asks for the encoding before a frame: none until asked, none for a name the core does not list, and none for a frame drawn after it was let go.
        [Fact]
        public void The_raw_picture_is_handed_only_for_frames_drawn_while_its_encoding_is_asked_for()
        {
            byte[] rom = BoardPrograms.Build(4, 8, cycleIrq: false, maskCycle: true);
            string path = Path.Combine(Path.GetTempPath(), $"emusen-raw-{Environment.ProcessId}.nes");
            File.WriteAllBytes(path, rom);
            try
            {
                using var core = new MoonRtCore();
                core.LoadRom(path);
                Assert.Equal(new[] { RawEncodings.NesPpu }, ((EmuSen.Cores.Native.CoreEngine)core).Machine.Info.Raw.Select(r => r.Encoding));
                // The palette MoonRT declares is the NES system pack's, all 64 colours, whatever the emphasis bits (Moon_Native.md §9.6).
                IReadOnlyList<int> palette = ((EmuSen.Cores.Native.CoreEngine)core).Machine.Info.Raw[0].Palette;
                Assert.Equal(512, palette.Count);
                for (int v = 0; v < 512; v++)
                {
                    int c = (v & 0x3F) * 3;
                    Assert.Equal(NesDebugViews.MasterPalette[c] << 16 | NesDebugViews.MasterPalette[c + 1] << 8 | NesDebugViews.MasterPalette[c + 2], palette[v]);
                }
                core.RunFrame();
                Assert.Null(core.RawPicture);
                core.RawEncoding = "snes.ppu";
                core.RunFrame();
                Assert.Null(core.RawPicture);
                core.RawEncoding = RawEncodings.NesPpu;
                Assert.Null(core.RawPicture);
                core.RunFrame();
                Held(core, 3, new HashSet<int>());
                core.SkipRendering = true;
                core.RunFrame();
                Held(core, 4, new HashSet<int>());
                core.SkipRendering = false;
                core.RawEncoding = null;
                core.RunFrame();
                Assert.Null(core.RawPicture);
            }
            finally { File.Delete(path); }
        }
    }
}
