using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MarsRT's machine core against the C# Mars, interpreter against interpreter, compared as save states after every frame - see Mars_Native.md §5.2.
    [Collection("MarsStatics")]
    public class MarsRtTests : IDisposable
    {
        public const string CorpusVariable = "EMUSEN_MARSRT_CORPUS";
        public const string StatesVariable = "EMUSEN_MARSRT_STATES";
        public const string BenchVariable = "EMUSEN_MARSRT_BENCH";

        private const string Finished = "Base: Failed ";

        private readonly ITestOutputHelper _output;
        private readonly List<string> _temporary = new();
        private readonly bool _nativeWas = EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseNative;

        public MarsRtTests(ITestOutputHelper output)
        {
            _output = output;
            EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseNative = false;
        }

        public void Dispose()
        {
            EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseNative = _nativeWas;
            foreach (string path in _temporary)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        // The C# interpreter as the oracle: no compiled blocks, the display processor on this thread, the picture not scanned.
        public static MarsCore Oracle(bool expansionPak = false) => new(expansionPak, batteryRamDisabled: true)
        {
            UseBlocks = false,
            ThreadedRdp = false,
            DeferredPresentation = false,
            SkipRendering = true,
        };

        public static MarsRtCore Twin(bool expansionPak = false, bool framer = false) => new(expansionPak, batteryRamDisabled: true) { SkipRendering = true, FramesRdp = framer };

        [Fact]
        public void The_corpus_reports_line_for_line_what_the_csharp_core_reports()
        {
            string? path = Environment.GetEnvironmentVariable(CorpusVariable) ?? N64TestRomLibrary.FindSystemTest();
            if (path is null || !File.Exists(path))
            {
                _output.WriteLine("n64-systemtest absent, not run");
                return;
            }

            Assert.True(MarsRtCore.Available, MarsNative.Report);
            var clock = Stopwatch.StartNew();
            var rom = RomImage.Load(path);
            var bus = new MemoryBus(expansionPak: true);
            var cpu = new Cpu(bus);
            Boot.HandOff(bus, cpu, rom);
            for (int steps = 0; steps < 400_000_000; steps++)
            {
                cpu.Step();
                if ((steps & 0xFFFFF) == 0 && bus.IsViewer.Text.Contains(Finished, StringComparison.Ordinal)) break;
            }
            string csharp = bus.IsViewer.Text;
            TimeSpan csharpTime = clock.Elapsed;

            clock.Restart();
            using var twin = new MarsRtCore { FramesRdp = Environment.GetEnvironmentVariable("EMUSEN_MARSRT_CORPUS_FRAMER") == "1" };
            twin.Boot(File.ReadAllBytes(path));
            string rust = "";
            for (int i = 0; i < 400; i++)
            {
                twin.RunSteps(1 << 20);
                rust = twin.IsViewerTranscript;
                if (rust.Contains(Finished, StringComparison.Ordinal)) break;
            }
            _output.WriteLine($"C# {cpu.Instructions} instructions in {csharpTime.TotalSeconds:F1} s; Rust {twin.Instructions} in {clock.Elapsed.TotalSeconds:F1} s");
            if (Environment.GetEnvironmentVariable("EMUSEN_MARSRT_CORPUS_OUT") is { } folder)
            {
                File.WriteAllText(Path.Combine(folder, "csharp.txt"), csharp);
                File.WriteAllText(Path.Combine(folder, "rust.txt"), rust);
            }

            // Until the display processor is ported, the four tests that wait on its drawing fail here and only here - see Mars_Native.md §5.2.
            string[] want = Through(csharp, Finished), got = Through(rust, Finished);
            var drawing = new List<string>();
            int w = 0, g = 0;
            while (w < want.Length - 1 && g < got.Length - 1)
            {
                if (want[w] == got[g]) { w++; g++; continue; }
                Assert.True(got[g].StartsWith("Test 'RDP STATUS: ", StringComparison.Ordinal) && got[g + 1].Length == 0, $"line {g + 1}: C# \"{want[w]}\", Rust \"{got[g]}\"");
                drawing.Add(got[g]);
                g += 2;
            }
            Assert.Equal(want.Length - w, got.Length - g);
            Assert.Equal($"Failed {46 + drawing.Count} of 4637 tests", Summary(rust));
            Assert.Equal("Failed 46 of 4637 tests", Summary(csharp));
            Assert.InRange(drawing.Count, 0, 4);
            _output.WriteLine($"{got.Length} lines, {got.Length - 2 * drawing.Count} identical to C#'s; the rest are the display processor's drawing:\n{string.Join("\n", drawing)}");
        }

        // A machine built from the ROM in both, then compared after the load and after every frame; the scenarios reach every device.
        [Theory]
        [InlineData("system", 240)]
        [InlineData("system-without-rsp", 240)]
        [InlineData("count-forever", 60)]
        public void A_synthetic_rom_stays_byte_exact_frame_by_frame_from_boot(string scenario, int frames)
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = Temporary(Scenario(scenario));
            MarsCore oracle = Oracle();
            using MarsRtCore twin = Twin(framer: scenario == "system");
            oracle.LoadRom(rom);
            twin.LoadRom(rom);

            var compare = new StateComparer(ignoreRdp: scenario == "system");
            compare.Frame(0, Save(oracle), twin.Save(false));
            for (int frame = 1; frame <= frames; frame++)
            {
                Drive(oracle, twin, frame);
                oracle.RunFrame();
                twin.RunFrame();
                compare.Frame(frame, Save(oracle), twin.Save(false));
            }

            _output.WriteLine($"{scenario}: {frames} frames exact, {oracle.Bus!.Cycles} cycles, {oracle.Cpu!.Instructions} instructions; MarsRT passed {twin.IdleTurnsPassed} idle turns{compare.Note}");
            Assert.True(oracle.Cpu.Instructions > 100_000, "the program never ran");
            if (scenario.StartsWith("system")) Assert.True(oracle.Bus.Rdram[0x1003] > 100, "the handler saw too few fields");
        }

        // A state the C# core wrote mid-run, loaded by both, so MarsRT runs everything a C# load derives; then the two run on side by side.
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void A_state_written_by_the_csharp_core_runs_on_byte_exact_in_marsrt(bool snapshot)
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = Temporary(Scenario("system"));
            MarsCore oracle = Oracle(expansionPak: true);
            oracle.LoadRom(rom);
            for (int frame = 1; frame <= 37; frame++)
            {
                Drive(oracle, null, frame);
                oracle.RunFrame();
            }

            byte[] state = Save(oracle, snapshot);
            oracle.LoadState(new MemoryStream(state));
            using MarsRtCore twin = Twin(framer: true);
            twin.LoadRom(rom);
            twin.LoadState(state);
            var compare = new StateComparer(ignoreRdp: true);
            compare.Frame(37, Save(oracle), twin.Save(false));
            for (int frame = 38; frame <= 200; frame++)
            {
                Drive(oracle, twin, frame);
                oracle.RunFrame();
                twin.RunFrame();
                compare.Frame(frame, Save(oracle), twin.Save(false));
            }
            _output.WriteLine($"163 frames exact after a {(snapshot ? "snapshot" : "state")} of {state.Length} bytes{compare.Note}");
        }

        // A state MarsRT wrote, loaded by the C# core and by MarsRT itself, runs on alike in both.
        [Fact]
        public void A_state_written_by_marsrt_runs_on_byte_exact_in_the_csharp_core()
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = Temporary(Scenario("system-without-rsp"));
            using MarsRtCore twin = Twin();
            twin.LoadRom(rom);
            for (int frame = 1; frame <= 50; frame++)
            {
                Drive(null, twin, frame);
                twin.RunFrame();
            }

            byte[] state = twin.Save(false);
            MarsCore oracle = Oracle();
            oracle.LoadRom(rom);
            oracle.LoadState(new MemoryStream(state));
            twin.LoadState(state);
            var compare = new StateComparer(ignoreRdp: false);
            compare.Frame(50, Save(oracle), twin.Save(false));
            for (int frame = 51; frame <= 120; frame++)
            {
                Drive(oracle, twin, frame);
                oracle.RunFrame();
                twin.RunFrame();
                compare.Frame(frame, Save(oracle), twin.Save(false));
            }
        }

        // The idle loop run whole, stepped, and with the signal processor run to its events, leave the state the plain steps leave.
        [Fact]
        public void The_idle_skip_changes_nothing_marsrt_computes()
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = Temporary(Scenario("system"));
            byte[]? reference = null;
            long passed = 0;
            foreach (var (idle, whole) in new[] { (false, false), (true, false), (true, true) })
            {
                using MarsRtCore twin = Twin(framer: true);
                twin.IdleSkip = idle;
                twin.RspWhole = whole;
                twin.LoadRom(rom);
                for (int frame = 1; frame <= 120; frame++)
                {
                    Drive(null, twin, frame);
                    twin.RunFrame();
                }
                byte[] state = twin.Save(false);
                reference ??= state;
                Assert.True(reference.AsSpan().SequenceEqual(state), $"idle {idle}, whole {whole}: the state differs from the plain steps'");
                passed = Math.Max(passed, twin.IdleTurnsPassed);
            }
            Assert.True(passed > 1000, "the idle loop was never passed, so the test compared nothing");
        }

        // Real games from boot and from their states, until the stub display processor makes them part: first exactly, then with its own fields and drawing left out.
        [Theory]
        [InlineData("sm64.z64", null)]
        [InlineData("oot.z64", null)]
        [InlineData("ge.z64", null)]
        [InlineData("sm64.z64", "sm64.state")]
        [InlineData("oot.z64", "oot.state")]
        [InlineData("ge.z64", "ge-dam.state")]
        public void A_real_game_stays_exact_until_the_display_processor_draws(string romName, string? stateName)
        {
            string? folder = Environment.GetEnvironmentVariable(StatesVariable);
            if (folder is null || !File.Exists(Path.Combine(folder, romName)) || (stateName != null && !File.Exists(Path.Combine(folder, stateName))))
            {
                _output.WriteLine($"{romName} {stateName}: absent, not run");
                return;
            }

            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = Temporary(File.ReadAllBytes(Path.Combine(folder, romName)));
            int frames = int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_MARSRT_FRAMES"), out int n) ? n : 300;
            foreach (bool ignoreRdp in new[] { false, true })
            {
                MarsCore oracle = Oracle(expansionPak: true);
                using MarsRtCore twin = Twin(expansionPak: true, framer: ignoreRdp);
                oracle.LoadRom(rom);
                twin.LoadRom(rom);
                if (stateName != null)
                {
                    byte[] state = File.ReadAllBytes(Path.Combine(folder, stateName));
                    oracle.LoadState(new MemoryStream(state));
                    twin.LoadState(state);
                }

                var compare = new StateComparer(ignoreRdp) { Regions = ignoreRdp ? twin : null };
                string verdict = $"all {frames} frames exact";
                try
                {
                    compare.Frame(0, Save(oracle), twin.Save(false));
                    for (int frame = 1; frame <= frames; frame++)
                    {
                        oracle.RunFrame();
                        twin.RunFrame();
                        compare.Frame(frame, Save(oracle), twin.Save(false));
                    }
                }
                catch (DivergedException diverged)
                {
                    verdict = diverged.Message;
                }
                _output.WriteLine($"{romName} {stateName ?? "from boot"} {(ignoreRdp ? "without the RDP's fields and drawing" : "exactly")}: {verdict}{compare.Note}");
            }
        }

        // MarsRT's interpreter against the C# interpreter, blocks off in both, three interleaved rounds from the games' states.
        [Theory]
        [InlineData("sm64.z64", "sm64.state")]
        [InlineData("oot.z64", "oot.state")]
        [InlineData("ge.z64", "ge-dam.state")]
        public void Bench(string romName, string stateName)
        {
            string? folder = Environment.GetEnvironmentVariable(StatesVariable);
            if (Environment.GetEnvironmentVariable(BenchVariable) != "1" || folder is null || !File.Exists(Path.Combine(folder, stateName))) return;

            string rom = Temporary(File.ReadAllBytes(Path.Combine(folder, romName)));
            byte[] state = File.ReadAllBytes(Path.Combine(folder, stateName));
            int frames = int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_MARSRT_FRAMES"), out int n) ? n : 300;
            bool rspBlocks = EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks;
            EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks = false;
            try
            {
                for (int round = 1; round <= 3; round++)
                {
                    MarsCore oracle = Oracle(expansionPak: true);
                    oracle.LoadRom(rom);
                    oracle.LoadState(new MemoryStream(state));
                    double csharp = Time(frames, oracle.RunFrame);

                    var times = new List<string>();
                    foreach (var (idle, framer) in new[] { (false, false), (true, false), (true, true) })
                    {
                        using MarsRtCore twin = Twin(expansionPak: true, framer: framer);
                        twin.IdleSkip = idle;
                        twin.LoadRom(rom);
                        twin.LoadState(state);
                        times.Add($"{Time(frames, twin.RunFrame):F2} ms{(idle ? " idle skip" : " plain")}{(framer ? " + syncs" : "")}");
                    }
                    _output.WriteLine($"{romName} round {round}: C# {csharp:F2} ms; MarsRT {string.Join(", ", times)} (per frame, {frames} frames)");
                }
            }
            finally
            {
                EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks = rspBlocks;
            }
        }

        private static double Time(int frames, Action frame)
        {
            var clock = Stopwatch.StartNew();
            for (int i = 0; i < frames; i++) frame();
            return clock.Elapsed.TotalMilliseconds / frames;
        }

        // The same input to both cores, a button pattern and a stick sweep that change every frame.
        private static void Drive(MarsCore? oracle, MarsRtCore? twin, int frame)
        {
            PadButton[] buttons = { PadButton.A, PadButton.B, PadButton.Start, PadButton.Up, PadButton.L2, PadButton.R };
            PadButton button = buttons[frame % buttons.Length];
            bool pressed = (frame / buttons.Length) % 2 == 0;
            double x = Math.Sin(frame * 0.37), y = Math.Cos(frame * 0.23), c = (frame % 7 - 3) / 3.0;
            foreach (EmuSen.Cores.ICore core in new EmuSen.Cores.ICore?[] { oracle, twin }.OfType<EmuSen.Cores.ICore>())
            {
                core.SetButton(0, button, pressed);
                core.SetAxis(0, PadAxis.LeftX, x);
                core.SetAxis(0, PadAxis.LeftY, y);
                core.SetAxis(0, PadAxis.RightX, c);
                core.SetAxis(0, PadAxis.RightY, -c);
            }
        }

        private static byte[] Scenario(string name) => name switch
        {
            "system" => SyntheticN64System.Build(rsp: true),
            "system-without-rsp" => SyntheticN64System.Build(rsp: false),
            _ => SyntheticN64Rom.Build(patches: (0, new byte[]
            {
                0x3C, 0x08, 0xA4, 0x40, 0x24, 0x09, 0x02, 0x0D, 0xAD, 0x09, 0x00, 0x18, 0x24, 0x09, 0x00, 0x40, 0xAD, 0x09, 0x00, 0x1C,
                0x3C, 0x04, 0xA0, 0x10, 0x8C, 0x88, 0x00, 0x00, 0x25, 0x08, 0x00, 0x01, 0xAC, 0x88, 0x00, 0x00, 0x10, 0x00, 0xFF, 0xFC, 0x00, 0x00, 0x00, 0x00,
            })),
        };

        private string Temporary(byte[] image)
        {
            string path = SyntheticN64Rom.WriteTemp(image);
            _temporary.Add(path);
            return path;
        }

        private static byte[] Save(MarsCore core, bool snapshot = false)
        {
            using var stream = new MemoryStream();
            if (snapshot) core.SaveSnapshot(stream);
            else core.SaveState(stream);
            return stream.ToArray();
        }

        private static string[] Through(string text, string marker)
        {
            int at = text.IndexOf(marker, StringComparison.Ordinal);
            int end = at < 0 ? text.Length : text.IndexOf('\n', at) is int newline and >= 0 ? newline : text.Length;
            return text[..end].Split('\n');
        }

        private static string Summary(string text)
        {
            int start = text.IndexOf(Finished, StringComparison.Ordinal);
            if (start < 0) return "the run did not reach the end";
            int end = text.IndexOf(" tests", start, StringComparison.Ordinal);
            return end < 0 ? "the run did not reach the end" : text[(start + 6)..(end + 6)];
        }

        private sealed class DivergedException(string message) : Exception(message);

        // Two states compared by the C# serializer's field names; with the RDP left out, its fields, hidden bits and drawn RDRAM do not count.
        private sealed class StateComparer(bool ignoreRdp)
        {
            private readonly SortedSet<string> _ignoredFields = new(StringComparer.Ordinal);
            private readonly List<(uint Start, uint End)> _drawn = new();
            private long _ignoredBytes;

            public MarsRtCore? Regions { get; init; }

            public string Note => ignoreRdp ? $"; differing only in {string.Join(", ", _ignoredFields.Take(12))}{(_ignoredFields.Count > 12 ? ", ..." : "")} ({_ignoredBytes} bytes over the run)" : "";

            public void Frame(int frame, byte[] csharp, byte[] rust)
            {
                if (Regions is { } twin)
                {
                    foreach (var (start, end) in twin.TakeRdpRegions())
                    {
                        int at = _drawn.FindIndex(r => start <= r.End && end >= r.Start);
                        if (at < 0) _drawn.Add((start, end));
                        else _drawn[at] = (Math.Min(start, _drawn[at].Start), Math.Max(end, _drawn[at].End));
                    }
                }
                if (csharp.AsSpan().SequenceEqual(rust)) return;
                if (csharp.Length != rust.Length) throw new DivergedException($"frame {frame}: C# wrote {csharp.Length} bytes and Rust {rust.Length}");

                using var machine = new MarsMachine(BitConverter.ToInt32(csharp, 8));
                machine.Load(csharp);
                var fields = machine.Layout(snapshot: false).Split('\n', StringSplitOptions.RemoveEmptyEntries).Select(line => line.Split(' ')).Select(p => (Offset: int.Parse(p[0]), Length: int.Parse(p[1]), Path: p[3])).ToArray();
                var real = new List<string>();
                foreach (var (offset, length, path) in fields)
                {
                    if (csharp.AsSpan(offset, length).SequenceEqual(rust.AsSpan(offset, length))) continue;
                    bool rdp = path.StartsWith("Bus.Dp.Processor.", StringComparison.Ordinal) || path == "Bus.RdramHidden";
                    if (ignoreRdp && rdp)
                    {
                        _ignoredFields.Add(path);
                        _ignoredBytes += Differing(csharp, rust, offset, length);
                        continue;
                    }
                    if (ignoreRdp && path == "Bus.Rdram")
                    {
                        string? outside = Outside(csharp, rust, offset, length, frame);
                        if (outside is null) continue;
                        real.Add(outside);
                        continue;
                    }
                    real.Add(Describe(path, csharp, rust, offset, length));
                }
                if (real.Count > 0) throw new DivergedException($"frame {frame}: {string.Join("; ", real.Take(8))}{(real.Count > 8 ? $" and {real.Count - 8} more" : "")}");
            }

            private string? Outside(byte[] csharp, byte[] rust, int offset, int length, int frame)
            {
                var ranges = new List<(int, int)>();
                for (int i = 0; i < length; i++)
                {
                    if (csharp[offset + i] == rust[offset + i]) continue;
                    if (_drawn.Any(r => (uint)i >= r.Start && (uint)i < r.End)) { _ignoredFields.Add("Bus.Rdram (drawn)"); _ignoredBytes++; continue; }
                    int start = i;
                    while (i < length && csharp[offset + i] != rust[offset + i]) i++;
                    ranges.Add((start, i));
                }
                if (ranges.Count == 0) return null;
                return $"Bus.Rdram undrawn at {string.Join(", ", ranges.Take(6).Select(r => $"0x{r.Item1:X6}-0x{r.Item2:X6}"))}{(ranges.Count > 6 ? $" and {ranges.Count - 6} more ranges" : "")}";
            }

            private static long Differing(byte[] a, byte[] b, int offset, int length)
            {
                long n = 0;
                for (int i = 0; i < length; i++) if (a[offset + i] != b[offset + i]) n++;
                return n;
            }

            private static string Describe(string path, byte[] csharp, byte[] rust, int offset, int length)
            {
                if (length > 16)
                {
                    int first = 0;
                    while (csharp[offset + first] == rust[offset + first]) first++;
                    return $"{path}[+0x{first:X}] C# {csharp[offset + first]:X2}, Rust {rust[offset + first]:X2} ({Differing(csharp, rust, offset, length)} bytes)";
                }
                return $"{path} C# {Convert.ToHexString(csharp, offset, length)}, Rust {Convert.ToHexString(rust, offset, length)}";
            }
        }
    }
}
