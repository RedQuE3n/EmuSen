using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // What recording calls on every frame would cost the v1 cores, against the plain frame and a breakpoint armed; only with EMUSEN_CALLCOST_BENCH - see EmuSen_CoreAPI.md §31.4.
    [Collection(TestCollections.ProcessGlobals)]
    public class CallRecordingCostBench : IDisposable
    {
        public const string BenchVariable = "EMUSEN_CALLCOST_BENCH";

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCallCost_" + Guid.NewGuid().ToString("N"));
        private readonly ITestOutputHelper _output;

        public CallRecordingCostBench(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            FirmwareLibrary.Directory = Path.Combine(_root, "Firmware");
            Directory.CreateDirectory(FirmwareLibrary.Directory);
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(true);
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            FirmwareLibrary.ResetDirectory();
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(null);
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // plain: nothing armed; attached: today's bridge on every frame with empty tables; calls: debug_run_frame with only the calls flag; breakpoint: one that is never reached.
        public enum Mode { Plain, Attached, Calls, Breakpoint }

        // The adapter with its frame taken one of the four ways; nothing here is the product's.
        private sealed unsafe class BenchEngine(CoreLibrary library, Mode mode) : CoreEngine(library)
        {
            public long Pushes;

            public void Arm()
            {
                Debug.CallStack.EntryPointObserver = (_, _) => Pushes++;
                if (mode == Mode.Breakpoint) Breakpoints.AddBreakpoint(0x7FFFFFF0, 0x7FFFFFFF, null);
                if (mode == Mode.Calls) Machine.Library.Api.DebugSet(Machine.Handle, CoreDebugBridge.FlagCalls, int.MinValue, -1);
            }

            protected override bool AdvanceFrame(CoreMachine m)
            {
                if (mode == Mode.Attached) return Debug.RunFrame(false, out _, out _);
                if (mode != Mode.Calls) return base.AdvanceFrame(m);
                CoreInterface api = m.Library.Api;
                uint flags = 0, processor;
                ulong pc, detail;
                while (true)
                {
                    int reasons = api.DebugRunFrame(m.Handle, flags, &processor, &pc, &detail);
                    Debug.Drain(m);
                    if (reasons < 0) throw new CoreRefusedException(reasons, m.Words(reasons));
                    if (reasons == 0) return true;
                    flags = CoreDebugBridge.RunUnchecked | CoreDebugBridge.RunContinue;
                }
            }
        }

        private static readonly string[] Cores = { "moonrt", "mercuryrt", "venusrt", "nephrite" };

        // The four images that call in a loop, each written to <dir>: two nested calls, an eight-step count, back to the loop.
        private static IReadOnlyDictionary<string, string> Synthetic(string dir)
        {
            var nes = SyntheticNesRom.Build(patches: (0, new byte[]
            {
                0x78, 0xD8, 0xA2, 0xFF, 0x9A,       // $8000 SEI CLD LDX #$FF TXS
                0x20, 0x10, 0x80, 0x4C, 0x05, 0x80, // $8005 JSR $8010; JMP $8005
                0, 0, 0, 0, 0,
                0x20, 0x20, 0x80, 0xA2, 0x08, 0xCA, 0xD0, 0xFD, 0x60, // $8010 JSR $8020; LDX #8; DEX; BNE; RTS
                0, 0, 0, 0, 0, 0, 0,
                0xE6, 0x00, 0x60,                   // $8020 INC $00; RTS
            }));
            var gb = SyntheticGbRom.Build(patches: (0, new byte[]
            {
                0xF3, 0x31, 0xFE, 0xFF,             // $0150 DI; LD SP,$FFFE
                0xCD, 0x60, 0x01, 0x18, 0xFB,       // $0154 CALL $0160; JR $0154
                0, 0, 0, 0, 0, 0, 0,
                0xCD, 0x70, 0x01, 0x06, 0x08, 0x05, 0x20, 0xFD, 0xC9, // $0160 CALL $0170; LD B,8; DEC B; JR NZ; RET
                0, 0, 0, 0, 0, 0, 0,
                0x3C, 0xEA, 0x00, 0xC0, 0xC9,       // $0170 INC A; LD ($C000),A; RET
            }));
            var snes = SyntheticRom.Build((5, new byte[]
            {
                0x78, 0x20, 0x10, 0x80, 0x80, 0xFB, // $8005 SEI; JSR $8010; BRA $8006
                0, 0, 0, 0, 0,
                0x20, 0x20, 0x80, 0xA2, 0x08, 0xCA, 0xD0, 0xFD, 0x60, // $8010 JSR $8020; LDX #8; DEX; BNE; RTS
                0, 0, 0, 0, 0, 0, 0,
                0xE6, 0x00, 0x60,                   // $8020 INC $00; RTS
            }));
            // The zero gaps in each listing above are never run.
            var md = SyntheticMdRom.Cartridge();
            Words(md, 0, 0x00FF, 0xFE00, 0x0000, 0x0200);
            Words(md, 0x200, 0x4EB9, 0x0000, 0x0300, 0x60F8);                     // JSR $300; BRA $200
            Words(md, 0x300, 0x4EB9, 0x0000, 0x0380, 0x7007, 0x51C8, 0xFFFE, 0x4E75); // JSR $380; MOVEQ #7,D0; DBRA D0,*; RTS
            Words(md, 0x380, 0x5282, 0x4E75);                                     // ADDQ.L #1,D2; RTS
            var files = new Dictionary<string, string>();
            foreach (var (core, name, image) in new[] { ("moonrt", "calls.nes", nes), ("mercuryrt", "calls.gb", gb), ("venusrt", "calls.sfc", snes), ("nephrite", "calls.md", md) })
            {
                string path = Path.Combine(dir, name);
                File.WriteAllBytes(path, image);
                files[core] = path;
            }
            return files;
        }

        private static void Words(byte[] r, int at, params ushort[] words)
        {
            foreach (ushort w in words)
            {
                r[at++] = (byte)(w >> 8);
                r[at++] = (byte)w;
            }
        }

        private static int Env(string name, int fallback) => int.TryParse(Environment.GetEnvironmentVariable(name), out int v) ? v : fallback;

        // One round: every core, image and condition in an order rotated by EMUSEN_CALLCOST_ROUND; rows appended to EMUSEN_CALLCOST_OUT.
        [Fact]
        public void Bench_call_recording()
        {
            if (Environment.GetEnvironmentVariable(BenchVariable) is null) { _output.WriteLine($"{BenchVariable} unset, not run"); return; }
            int round = Env("EMUSEN_CALLCOST_ROUND", 0), boot = Env("EMUSEN_CALLCOST_BOOT", 600), timed = Env("EMUSEN_CALLCOST_FRAMES", 2000);
            string? probe = Environment.GetEnvironmentVariable("EMUSEN_CALLCOST_PROBE");
            string? only = Environment.GetEnvironmentVariable("EMUSEN_CALLCOST_CORES");
            string? output = Environment.GetEnvironmentVariable("EMUSEN_CALLCOST_OUT");

            var images = new List<(string Core, string Label, string Path)>();
            foreach (var (core, path) in Synthetic(_root)) images.Add((core, "synthetic", path));
            // EMUSEN_CALLCOST_GAMES: core=path pairs separated by ';', scratch copies of commercial images.
            foreach (string pair in (Environment.GetEnvironmentVariable("EMUSEN_CALLCOST_GAMES") ?? "").Split(';', StringSplitOptions.RemoveEmptyEntries))
            {
                string[] kv = pair.Split('=', 2);
                images.Add((kv[0], Path.GetFileNameWithoutExtension(kv[1]), kv[1]));
            }
            images = images.Where(i => only is null || only.Split(',').Contains(i.Core)).ToList();

            var runs = new List<(string Core, string Label, string Path, string Library, Mode Mode)>();
            foreach (var image in images)
            {
                string shipped = CoreDiscovery.Found.Single(c => c.Info.Id == image.Core).Open()!.Path;
                foreach (Mode mode in Enum.GetValues<Mode>()) runs.Add((image.Core, image.Label, image.Path, shipped, mode));
                if (probe is not null)
                {
                    string lib = Path.Combine(probe, $"lib{image.Core}.so");
                    runs.Add((image.Core, image.Label, image.Path, lib, Mode.Plain));
                    runs.Add((image.Core, image.Label, image.Path, lib, Mode.Calls));
                }
            }
            // Each image's conditions back to back, rotated a step a round, so no condition always runs first.
            var ordered = runs.GroupBy(r => (r.Core, r.Label)).SelectMany(g =>
            {
                var list = g.ToList();
                int k = round % list.Count;
                return list.Skip(k).Concat(list.Take(k));
            }).ToList();

            foreach (var run in ordered)
            {
                CoreLibrary library = CoreLibrary.Open(run.Library);
                Assert.True(library.Available, library.Report);
                using var engine = new BenchEngine(library, run.Mode) { Options = TestCores.NoBattery };
                engine.LoadRom(run.Path);
                void Input(int f)
                {
                    int phase = f % 90;
                    engine.SetButton(0, PadButton.Start, phase < 5);
                    engine.SetButton(0, PadButton.A, phase >= 45 && phase < 50);
                }
                for (int f = 0; f < boot; f++)
                {
                    Input(f);
                    engine.RunFrame();
                    engine.ReturnFrameBuffer(engine.GetFrameBufferRgba());
                    engine.DequeueAudioSamples(int.MaxValue);
                }
                engine.Arm();
                var times = new double[timed];
                var clock = new Stopwatch();
                for (int f = 0; f < timed; f++)
                {
                    Input(boot + f);
                    clock.Restart();
                    engine.RunFrame();
                    byte[] picture = engine.GetFrameBufferRgba();
                    engine.DequeueAudioSamples(int.MaxValue);
                    times[f] = clock.Elapsed.TotalMilliseconds;
                    engine.ReturnFrameBuffer(picture);
                }
                Assert.False(engine.IsHaltedAtBreakpoint);
                using var state = new MemoryStream();
                engine.SaveState(state);
                string hash = Convert.ToHexString(SHA256.HashData(state.ToArray()))[..16];
                var sorted = times.OrderBy(t => t).ToArray();
                string which = run.Library.StartsWith(probe ?? "\0", StringComparison.Ordinal) ? "probe" : "shipped";
                string row = $"round {round} {run.Core,-9} {run.Label,-15} {which,-7} {run.Mode,-10} mean {times.Average():F4} p50 {sorted[timed / 2]:F4} p90 {sorted[timed * 9 / 10]:F4} ms  pushes/frame {engine.Pushes / (double)timed:F1}  depth {engine.Debug.CallStack.Depth}  state {hash}  load {File.ReadAllText("/proc/loadavg").Split(' ')[0]}";
                _output.WriteLine(row);
                if (output is not null) File.AppendAllText(output, row + Environment.NewLine);
            }
        }
    }
}
