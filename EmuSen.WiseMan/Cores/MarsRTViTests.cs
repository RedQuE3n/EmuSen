using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.Mars.Vi;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;
using VideoInterface = EmuSen.Cores.Nintendo.Mars.Vi.Vi;

namespace EmuSen.WiseMan.Cores
{
    // MarsRT's scan-out against the C# VI it ports, raster, held lines and frame, byte for byte - see Mars_Native.md §5.4.
    public class MarsRTViTests
    {
        private const uint Framebuffer = 0x0020_0000;
        private const uint GammaOn = 1 << 3, DivotOn = 1 << 4, Serrated = 1 << 6, DitherFilter = 1 << 16;

        // Copies of real games and states, read from the folder MarsNativeStateTests reads; absent, those cases pass without running.
        public const string FramesVariable = "EMUSEN_MARSRT_VI_FRAMES";

        private readonly ITestOutputHelper _output;

        public MarsRTViTests(ITestOutputHelper output) => _output = output;

        // The register sets of MarsViTests and MarsDeferredPresentationTests, each scanned by both, then the same with every hidden byte at 3 and at 0.
        [Fact]
        public void The_reference_tests_scans_come_out_of_marsrt_as_out_of_the_csharp_vi()
        {
            Assert.True(MarsRTViScan.Available, MarsNative.Report);
            uint[] shorter = Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 60, 0, 0);
            uint[] blank = Registers(0, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0);
            uint[][] scans =
            {
                Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0),
                Registers(2, 2, 64, 0x2AB, 0x155, 108, 320, 34, 200, 0x80, 0x40),
                Registers(3, 2, 32, 0x400, 0x400, 108, 128, 34, 100, 0, 0),
                Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 110, 0, 0, serrate: true, currentLine: 1),
                Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 110, 0, 0, serrate: true),
                shorter, shorter, shorter,
                Registers(0, 3, 64, 0x400, 0x400, 148, 200, 34, 120, 0, 0),
                Registers(2, 3, 64, 0x400, 0x400, 40, 640, 34, 120, 0, 0),
                Registers(2, 3, 64, 0x400, 0x400, 128, 640, 44, 240, 0, 0, sync: 625),
                Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0, origin: 0),
                blank, blank,
                Registers(2, 3, 64, 0x400, 0x400, 108, 0, 34, 120, 0, 0),
                Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0),
                Registers(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0),
                Registers(3, 1, 32, 0x400, 0x400, 108, 128, 34, 100, 0, 0),
                Registers(2, 0, 64, 0x2AB, 0x155, 108, 320, 34, 200, 0x80, 0x40),
                Registers(2, 1, 64, 0x400, 0x200, 108, 256, 34, 110, 0, 0),
                Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0, control: DitherFilter),
                Registers(3, 3, 32, 0x400, 0x400, 108, 128, 34, 100, 0, 0, control: DitherFilter),
                Registers(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0, control: DivotOn),
                Registers(2, 3, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0, control: GammaOn),
                Registers(2, 1, 64, 0x2AB, 0x155, 108, 256, 34, 120, 0x80, 0x40, control: DitherFilter | DivotOn | GammaOn),
                Registers(2, 0, 320, 0x400, 0x400, 108, 320, 34, 240, 0, 0, control: DitherFilter | DivotOn | GammaOn),
                Registers(2, 0, 320, 0x200, 0x355, 108, 640, 44, 288, 0, 0, sync: 625, control: DitherFilter | DivotOn),
                Registers(3, 0, 160, 0x200, 0x355, 108, 640, 44, 288, 0, 0, sync: 625, control: DitherFilter | DivotOn | GammaOn),
                Registers(2, 1, 64, 0x400, 0x400, 108, 256, 34, 120, 0, 0, origin: 0x100),
            };

            MemoryBus bus = Noisy();
            using var rt = new MarsRTViScan();
            int compared = 0;
            foreach (int hidden in new[] { -1, 3, 0 })
            {
                if (hidden >= 0) Array.Fill(bus.RdramHidden, (byte)hidden, (int)Framebuffer / 2 - 0x4000, 0x18000);
                foreach (uint[] registers in scans)
                {
                    Program(bus, registers);
                    rt.Set(registers);
                    Assert.Equal(bus.Vi.Scan(), rt.Scan(bus.Rdram, bus.RdramHidden));
                    AssertSameScan(bus.Vi, rt, $"hidden {hidden}, scan {compared}");
                    compared++;
                }
            }

            _output.WriteLine($"{compared} scans, rasters, held lines and blank flags identical");
        }

        // The painter program through the core's own immediate presentation, rows sent once and repeated, progressive and interlaced.
        [Theory]
        [InlineData(false, false)]
        [InlineData(true, false)]
        [InlineData(false, true)]
        [InlineData(true, true)]
        public void A_painted_frame_buffer_presents_the_same_through_marsrt(bool repeatRows, bool interlaced)
        {
            Assert.True(MarsRTViScan.Available, MarsNative.Report);
            uint control = DitherFilter | DivotOn | GammaOn | (interlaced ? Serrated : 0);
            string path = SyntheticN64Rom.WriteTemp(MarsDeferredPresentationTests.Painter(control));
            try
            {
                var core = Immediate(expansionPak: false, repeatRows);
                core.LoadRom(path);
                using var rt = new MarsRTViScan { RepeatRows = repeatRows };
                Presenter present = new(core, rt);
                for (int frame = 1; frame <= 12; frame++) present.Frame($"frame {frame}");
                Assert.Equal(12, present.Walks);
            }
            finally
            {
                File.Delete(path);
            }
        }

        // Real games from power-on and from a gameplay state; every frame compared, and every tenth also scanned under other control words.
        [Theory]
        [InlineData("sm64", "sm64.state")]
        [InlineData("oot", "oot.state")]
        [InlineData("ge", "ge-dam.state")]
        public void Real_frames_scan_out_the_same_through_marsrt(string game, string stateName)
        {
            string? folder = Environment.GetEnvironmentVariable(MarsNativeStateTests.StatesVariable);
            string rom = Path.Combine(folder ?? "", game + ".z64"), state = Path.Combine(folder ?? "", stateName);
            if (folder is null || !File.Exists(rom) || !File.Exists(state))
            {
                _output.WriteLine($"{game}: absent, not run");
                return;
            }

            Assert.True(MarsRTViScan.Available, MarsNative.Report);
            int[] counts = (Environment.GetEnvironmentVariable(FramesVariable) ?? "900,600").Split(',').Select(int.Parse).ToArray();
            var modes = new SortedDictionary<string, int>();
            int total = 0, walks = 0, swept = 0;

            foreach ((string start, int frames) in new[] { ("power-on", counts[0]), (stateName, counts[1]) })
            {
                var core = Immediate(expansionPak: true, repeatRows: false);
                core.LoadRom(rom);
                if (start != "power-on") core.LoadState(state);

                using var rt = new MarsRTViScan { RepeatRows = false };
                using var sweep = new MarsRTViScan { RepeatRows = false };
                var sweepBus = new MemoryBus(expansionPak: true);
                var present = new Presenter(core, rt);

                for (int frame = 1; frame <= frames; frame++)
                {
                    uint[] registers = present.Frame($"{game} from {start}, frame {frame}");
                    string mode = Mode(registers);
                    modes[mode] = modes.GetValueOrDefault(mode) + 1;
                    total++;

                    if (frame % 10 != 0) continue;
                    MemoryBus bus = core.Bus!;
                    Array.Copy(bus.Rdram, sweepBus.Rdram, bus.Rdram.Length);
                    Array.Copy(bus.RdramHidden, sweepBus.RdramHidden, bus.RdramHidden.Length);
                    foreach (uint[] variant in Variants(registers))
                    {
                        Program(sweepBus, variant);
                        sweep.Set(variant);
                        Assert.Equal(sweepBus.Vi.Scan(), sweep.Scan(sweepBus.Rdram, sweepBus.RdramHidden));
                        AssertSameScan(sweepBus.Vi, sweep, $"{game} from {start}, frame {frame}, control {variant[0]:X}");
                        swept++;
                    }
                }

                walks += present.Walks;
            }

            _output.WriteLine($"{game}: {total} frames identical ({walks} walked), {swept} swept scans identical; modes as control/width/x scale/y scale/PAL: frames");
            foreach ((string mode, int count) in modes) _output.WriteLine($"  {mode}: {count}");
        }

        // Milliseconds a scan and composition, C# and Rust interleaved, three rounds; run by hand with EMUSEN_VI_SCAN_BENCH=1 - see Mars_Native.md §5.4.
        [Fact]
        public void Bench()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_VI_SCAN_BENCH") != "1") return;
            string folder = Environment.GetEnvironmentVariable(MarsNativeStateTests.StatesVariable)!;
            int frames = int.Parse(Environment.GetEnvironmentVariable(FramesVariable) ?? "300");

            foreach ((string game, string stateName) in new[] { ("sm64", "sm64.state"), ("oot", "oot.state"), ("ge", "ge-dam.state") })
            {
                for (int round = 1; round <= 3; round++)
                {
                    var core = Immediate(expansionPak: true, repeatRows: false);
                    core.LoadRom(Path.Combine(folder, game + ".z64"));
                    core.LoadState(Path.Combine(folder, stateName));
                    using var rt = new MarsRTViScan { RepeatRows = false };
                    VideoInterface vi = core.Bus!.Vi;
                    rt.Set((uint[])MarsRTViScan.RegistersOf(vi).Clone(), (int[])MarsRTViScan.HeldLinesOf(vi).Clone(), MarsRTViScan.WasBlankOf(vi));
                    var present = PresentOf(core);
                    var csharp = new List<double>();
                    var rust = new List<double>();

                    for (int frame = 1; frame <= frames; frame++)
                    {
                        core.RunFrame();
                        MemoryBus bus = core.Bus!;
                        rt.Set((uint[])MarsRTViScan.RegistersOf(bus.Vi).Clone());
                        long a, b, c;
                        if ((frame & 1) == 0)
                        {
                            a = Stopwatch.GetTimestamp(); present(bus.Vi);
                            b = Stopwatch.GetTimestamp(); rt.Scan(bus.Rdram, bus.RdramHidden);
                            c = Stopwatch.GetTimestamp();
                            csharp.Add(Stopwatch.GetElapsedTime(a, b).TotalMilliseconds);
                            rust.Add(Stopwatch.GetElapsedTime(b, c).TotalMilliseconds);
                        }
                        else
                        {
                            a = Stopwatch.GetTimestamp(); rt.Scan(bus.Rdram, bus.RdramHidden);
                            b = Stopwatch.GetTimestamp(); present(bus.Vi);
                            c = Stopwatch.GetTimestamp();
                            rust.Add(Stopwatch.GetElapsedTime(a, b).TotalMilliseconds);
                            csharp.Add(Stopwatch.GetElapsedTime(b, c).TotalMilliseconds);
                        }
                    }

                    Assert.Equal(core.GetFrameBufferRgba(), rt.Frame(out _, out _, out _));
                    _output.WriteLine($"{game} round {round}: C# mean {csharp.Average():F3} median {Median(csharp):F3} ms, Rust mean {rust.Average():F3} median {Median(rust):F3} ms, {frames} scans, {Environment.ProcessorCount} processors");
                }
            }
        }

        private static double Median(List<double> values)
        {
            var sorted = values.OrderBy(v => v).ToArray();
            return sorted[sorted.Length / 2];
        }

        // One frame of the core, presented by its own immediate path, and the same scan by MarsRT, compared in everything a scan leaves.
        private sealed class Presenter
        {
            private readonly MarsCore _core;
            private readonly MarsRTViScan _rt;
            private readonly Action<VideoInterface> _present;
            private readonly MemoryBus _prepared = new();

            public int Walks { get; private set; }

            public Presenter(MarsCore core, MarsRTViScan rt)
            {
                (_core, _rt, _present) = (core, rt, PresentOf(core));
                VideoInterface vi = core.Bus!.Vi;
                rt.Set((uint[])MarsRTViScan.RegistersOf(vi).Clone(), (int[])MarsRTViScan.HeldLinesOf(vi).Clone(), MarsRTViScan.WasBlankOf(vi));
            }

            public uint[] Frame(string what)
            {
                _core.RunFrame();
                VideoInterface vi = _core.Bus!.Vi;
                uint[] registers = (uint[])MarsRTViScan.RegistersOf(vi).Clone();
                bool wasBlank = MarsRTViScan.WasBlankOf(vi);

                (int[] held, bool blank) = _rt.Held();
                Assert.True(held.AsSpan().SequenceEqual(MarsRTViScan.HeldLinesOf(vi)) && blank == wasBlank, $"{what}: the held lines before the scan differ");

                // Scan() is true exactly when Prepare is at one, which reads the registers and the blank flag alone.
                Program(_prepared, registers);
                MarsRTViScan.SetWasBlank(_prepared.Vi, wasBlank);
                bool walks = _prepared.Vi.Prepare(new ScanJob());

                _rt.Set(registers);
                _present(vi);
                MemoryBus bus = _core.Bus!;
                Assert.Equal(walks, _rt.Scan(bus.Rdram, bus.RdramHidden));
                if (walks) Walks++;

                byte[] frame = _rt.Frame(out int width, out int height, out int rowRepeat);
                Assert.True((width, height, rowRepeat) == (_core.ScreenWidth, _core.ScreenHeight, _core.RowRepeat),
                    $"{what}: MarsRT {width}x{height}x{rowRepeat}, C# {_core.ScreenWidth}x{_core.ScreenHeight}x{_core.RowRepeat}");
                AssertSameBytes(_core.GetFrameBufferRgba(), frame, width, $"{what}: frame");
                AssertSameScan(vi, _rt, what);
                return registers;
            }
        }

        private static MarsCore Immediate(bool expansionPak, bool repeatRows) =>
            new(expansionPak, batteryRamDisabled: true) { DeferredPresentation = false, ThreadedRdp = false, RenderScale = 1, RepeatRows = repeatRows, SkipRendering = true };

        // The core's private Present, which the frame's end calls when rendering is not skipped.
        private static Action<VideoInterface> PresentOf(MarsCore core) =>
            typeof(MarsCore).GetMethod("Present", BindingFlags.Instance | BindingFlags.NonPublic)!.CreateDelegate<Action<VideoInterface>>(core);

        private static void AssertSameScan(VideoInterface vi, MarsRTViScan rt, string what)
        {
            AssertSameBytes(MarsRTViScan.RasterOf(vi), rt.Raster(), VideoInterface.RasterWidth, $"{what}: raster");
            (int[] held, bool blank) = rt.Held();
            Assert.True(held.AsSpan().SequenceEqual(MarsRTViScan.HeldLinesOf(vi)), $"{what}: held lines differ");
            Assert.True(blank == MarsRTViScan.WasBlankOf(vi), $"{what}: blank flag differs");
        }

        private static void AssertSameBytes(byte[] expected, byte[] actual, int width, string what)
        {
            Assert.True(expected.Length == actual.Length, $"{what}: C# {expected.Length} bytes, MarsRT {actual.Length}");
            int at = expected.AsSpan().CommonPrefixLength(actual);
            if (at == expected.Length) return;
            int pixel = at / 4;
            Assert.Fail($"{what}: first difference at ({pixel % width},{pixel / width}) byte {at % 4}: C# {expected[at]:X2}, MarsRT {actual[at]:X2}");
        }

        // The type, the anti-alias mode and the passes, read as a game set them, then the same frame under every other mode that decides a pass.
        private static IEnumerable<uint[]> Variants(uint[] registers)
        {
            uint keep = registers[0] & ~(3u | (3u << 8) | GammaOn | DivotOn | DitherFilter);
            (uint Type, uint AntiAlias, uint Passes)[] modes =
            {
                (2, 0, DitherFilter | DivotOn), (2, 1, GammaOn), (2, 2, DivotOn), (2, 3, DitherFilter | GammaOn),
                (2, 0, DitherFilter | DivotOn | GammaOn), (3, 1, DitherFilter), (3, 2, DivotOn | GammaOn), (3, 0, 0),
            };
            foreach ((uint type, uint antiAlias, uint passes) in modes)
            {
                var variant = (uint[])registers.Clone();
                variant[0] = keep | type | (antiAlias << 8) | passes;
                yield return variant;
            }
        }

        private static string Mode(uint[] r) =>
            $"{r[0]:X5}/{r[2] & 0xFFF}/{r[12] & 0xFFF:X3}/{r[13] & 0xFFF:X3}/{((r[6] & 0x3FF) > 550 ? "PAL" : "NTSC")}";

        private static void Program(MemoryBus bus, uint[] registers)
        {
            for (int i = 0; i < registers.Length; i++) bus.Write32(MemoryMap.ViBase + (uint)i * 4, registers[i]);
        }

        private static MemoryBus Noisy()
        {
            var bus = new MemoryBus();

            uint state = 0x2468_ACE0;
            for (uint i = 0; i < 0x30000; i++)
            {
                state = state * 1103515245 + 12345;
                bus.Write8(Framebuffer - 0x8000 + i, (byte)(state >> 16));
            }

            uint seed = 0x1B4E_81B4;
            for (int i = 0; i < 0x18000; i++)
            {
                seed = seed * 1664525 + 1013904223;
                bus.RdramHidden[Framebuffer / 2 - 0x4000 + i] = (byte)(seed >> 30);
            }

            return bus;
        }

        private static uint[] Registers(int type, int antialias, uint width, uint stepX, uint stepY,
            uint left, uint columns, uint top, uint rows, uint biasX, uint biasY,
            bool serrate = false, uint currentLine = 0, uint sync = 525, uint origin = Framebuffer, uint control = 0)
        {
            var registers = new uint[14];
            registers[0] = (uint)(type & 3) | ((uint)(antialias & 3) << 8) | (serrate ? 1u << 6 : 0) | control;
            registers[1] = origin;
            registers[2] = width;
            registers[4] = currentLine;
            registers[6] = sync;
            registers[9] = ((left & 0x3FF) << 16) | ((left + columns) & 0x3FF);
            registers[10] = ((top & 0x3FF) << 16) | ((top + rows * 2) & 0x3FF);
            registers[12] = ((biasX & 0xFFF) << 16) | (stepX & 0xFFF);
            registers[13] = ((biasY & 0xFFF) << 16) | (stepY & 0xFFF);
            return registers;
        }
    }
}
