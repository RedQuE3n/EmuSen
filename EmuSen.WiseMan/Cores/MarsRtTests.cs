using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MarsRT's machine core against its own oracles: the test ROM corpus and its own runs compared with each other - see Mars_Native.md §5.2, §7.3.
    [Collection("MarsStatics")]
    public class MarsRtTests : IDisposable
    {
        public const string CorpusVariable = "EMUSEN_MARSRT_CORPUS";
        public const string StatesVariable = "EMUSEN_MARSRT_STATES";
        public const string BenchVariable = "EMUSEN_MARSRT_BENCH";

        private const string Finished = "Base: Failed ";

        private readonly ITestOutputHelper _output;
        private readonly List<string> _temporary = new();

        public MarsRtTests(ITestOutputHelper output) => _output = output;

        public void Dispose()
        {
            foreach (string path in _temporary)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        // MarsRT's interpreter on one thread, now that the shim's defaults are production's; the tier and thread runs turn theirs on themselves.
        public static MarsRtCore Twin(bool expansionPak = false) => new(expansionPak) { Options = TestCores.NoBattery, SkipRendering = true, UseBlocks = false, ThreadedRdp = false, DeferredPresentation = false, RdpWorkers = 1 };

        // n64-systemtest through the IS-Viewer: its own summary, and through the recompiler line for line what the interpreter reports.
        [Fact]
        public void The_corpus_reports_its_own_summary_and_the_recompiler_reports_what_the_interpreter_reports()
        {
            string? path = Environment.GetEnvironmentVariable(CorpusVariable) ?? N64TestRomLibrary.FindSystemTest();
            if (path is null || !File.Exists(path))
            {
                _output.WriteLine("n64-systemtest absent, not run");
                return;
            }

            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string Run(bool blocks)
            {
                using var core = new MarsRtCore { UseBlocks = blocks };
                core.Boot(path);
                string text = "";
                for (int i = 0; i < 400; i++)
                {
                    core.RunSteps(1 << 20);
                    text = core.IsViewerTranscript;
                    if (text.Contains(Finished, StringComparison.Ordinal)) break;
                }
                return text;
            }

            string[] interpreted = Through(Run(blocks: false), Finished), compiled = Through(Run(blocks: true), Finished);
            for (int i = 0; i < Math.Min(interpreted.Length, compiled.Length); i++)
                Assert.True(interpreted[i] == compiled[i], $"line {i + 1}: interpreter \"{interpreted[i]}\", blocks \"{compiled[i]}\"");
            Assert.Equal(interpreted.Length, compiled.Length);
            Assert.Equal("Failed 46 of 4637 tests", Summary(interpreted[^1]));
            _output.WriteLine($"{interpreted.Length} lines identical, ending \"{interpreted[^1]}\"");
        }

        [Fact]
        public void The_idle_skip_changes_nothing_marsrt_computes()
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = Temporary(Scenario("system"));
            byte[]? reference = null;
            long passed = 0;
            foreach (var (idle, whole) in new[] { (false, false), (true, false), (true, true) })
            {
                using MarsRtCore twin = Twin();
                twin.IdleSkip = idle;
                twin.RspWhole = whole;
                twin.LoadRom(rom);
                for (int frame = 1; frame <= 120; frame++)
                {
                    Drive(twin, frame);
                    twin.RunFrame();
                }
                byte[] state = twin.Save(false);
                reference ??= state;
                Assert.True(reference.AsSpan().SequenceEqual(state), $"idle {idle}, whole {whole}: the state differs from the plain steps'");
                passed = Math.Max(passed, twin.IdleTurnsPassed);
            }
            Assert.True(passed > 1000, "the idle loop was never passed, so the test compared nothing");
        }

        // The program into IMEM and the processor started by the status write given, while the CPU sits in its idle loop.
        private static void Drive(MarsRtCore twin, int frame)
        {
            PadButton[] buttons = { PadButton.A, PadButton.B, PadButton.Start, PadButton.Up, PadButton.L2, PadButton.R };
            PadButton button = buttons[frame % buttons.Length];
            bool pressed = (frame / buttons.Length) % 2 == 0;
            double x = Math.Sin(frame * 0.37), y = Math.Cos(frame * 0.23), c = (frame % 7 - 3) / 3.0;
            foreach (EmuSen.Cores.ICore core in new EmuSen.Cores.ICore[] { twin })
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
    }
}
