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

        public static MarsRtCore Twin(bool expansionPak = false) => new(expansionPak, batteryRamDisabled: true) { SkipRendering = true };

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
            using var twin = new MarsRtCore();
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

            string[] want = Through(csharp, Finished), got = Through(rust, Finished);
            for (int i = 0; i < Math.Min(want.Length, got.Length); i++)
            {
                Assert.True(want[i] == got[i], $"line {i + 1}: C# \"{want[i]}\", Rust \"{got[i]}\"");
            }
            Assert.Equal(want.Length, got.Length);
            Assert.Equal("Failed 46 of 4637 tests", Summary(rust));
            _output.WriteLine($"{got.Length} lines identical, ending \"{got[^1]}\"");
        }

        // A machine built from the ROM in both, then compared after the load and after every frame; the scenarios reach every device.
        [Theory]
        [InlineData("system", 240, false)]
        [InlineData("system", 240, true)]
        [InlineData("system-without-rsp", 240, false)]
        [InlineData("system-without-rsp", 240, true)]
        [InlineData("count-forever", 60, false)]
        public void A_synthetic_rom_stays_byte_exact_frame_by_frame_from_boot(string scenario, int frames, bool render)
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = Temporary(Scenario(scenario));
            MarsCore oracle = Oracle();
            using MarsRtCore twin = Twin();
            oracle.SkipRendering = twin.SkipRendering = !render;
            oracle.LoadRom(rom);
            twin.LoadRom(rom);

            var compare = new StateComparer();
            compare.Frame(0, Save(oracle), twin.Save(false));
            for (int frame = 1; frame <= frames; frame++)
            {
                Drive(oracle, twin, frame);
                oracle.RunFrame();
                twin.RunFrame();
                compare.Frame(frame, Save(oracle), twin.Save(false));
                if (render) SamePicture(frame, oracle, twin);
                SameSound(frame, oracle, twin);
            }

            _output.WriteLine($"{scenario}{(render ? " with the picture" : "")}: {frames} frames exact, {oracle.Bus!.Cycles} cycles, {oracle.Cpu!.Instructions} instructions; MarsRT passed {twin.IdleTurnsPassed} idle turns");
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
            using MarsRtCore twin = Twin();
            twin.LoadRom(rom);
            twin.LoadState(state);
            var compare = new StateComparer();
            compare.Frame(37, Save(oracle), twin.Save(false));
            for (int frame = 38; frame <= 200; frame++)
            {
                Drive(oracle, twin, frame);
                oracle.RunFrame();
                twin.RunFrame();
                compare.Frame(frame, Save(oracle), twin.Save(false));
            }
            _output.WriteLine($"163 frames exact after a {(snapshot ? "snapshot" : "state")} of {state.Length} bytes");
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
            var compare = new StateComparer();
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
                using MarsRtCore twin = Twin();
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

        // A signal processor that sets its own single-step bit while the CPU idles: the tick's step halts it at once, and C#'s idle loop in blocks does not.
        [Fact]
        public void A_processor_that_single_steps_itself_under_the_idle_loop_halts_where_the_interpreter_halts_it()
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = Temporary(SyntheticN64Rom.BuildRunningFromRdram(new uint[] { 0x1000_FFFF, 0x0000_0000 }));
            // Set the single-step bit, then count in $2 until a break the halt should prevent.
            uint[] program = { 0x2401_0040, 0x4081_2000, 0x2442_0001, 0x2442_0001, 0x2442_0001, 0x2442_0001, 0x0000_000D };

            byte[] Run(bool blocks, out long counted)
            {
                bool skip = Cpu.SkipIdle, rsp = EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks, background = EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.CompileBlocksInBackground;
                try
                {
                    Cpu.SkipIdle = true;
                    EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks = blocks;
                    EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.CompileBlocksInBackground = false;
                    MarsCore core = Oracle();
                    core.UseBlocks = blocks;
                    core.LoadRom(rom);
                    core.Cpu!.CompileInBackground = false;
                    for (int i = 0; i < 3; i++) core.RunFrame();
                    Start(core, program);
                    for (int i = 0; i < 3; i++) core.RunFrame();
                    counted = core.Bus!.Sp.Processor.Gpr[2];
                    return Save(core);
                }
                finally
                {
                    Cpu.SkipIdle = skip;
                    EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks = rsp;
                    EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.CompileBlocksInBackground = background;
                }
            }

            byte[] interpreted = Run(blocks: false, out long stepped);
            byte[] compiled = Run(blocks: true, out long inBlocks);

            MarsCore setup = Oracle();
            setup.LoadRom(rom);
            for (int i = 0; i < 3; i++) setup.RunFrame();
            Start(setup, program);
            using MarsRtCore twin = Twin();
            twin.LoadRom(rom);
            byte[] started = Save(setup);
            setup.LoadState(new MemoryStream(started));
            twin.LoadState(started);
            for (int i = 0; i < 3; i++)
            {
                setup.RunFrame();
                twin.RunFrame();
            }

            _output.WriteLine($"$2 counted {stepped} under the interpreter and {inBlocks} under C#'s blocks; MarsRT passed {twin.IdleTurnsPassed} idle turns");
            new StateComparer().Frame(3, Save(setup), twin.Save(false));
            Assert.Equal(0, stepped);
            Assert.True(inBlocks > 0 || !interpreted.AsSpan().SequenceEqual(compiled), "C#'s blocks halted where the interpreter halts, so there is no gap to record");
        }

        // The program into IMEM and the processor started, while the CPU sits in its idle loop.
        private static void Start(MarsCore core, uint[] program)
        {
            for (int i = 0; i < program.Length; i++) core.Bus!.Write32(0x0400_1000 + (uint)i * 4, program[i]);
            core.Bus!.Write32(0x0408_0000, 0);
            core.Bus.Write32(0x0404_0010, 0x0005);
        }

        // Real games from boot and from states: the whole state, the picture and the sound, after every frame.
        [Theory]
        [InlineData("sm64.z64", null)]
        [InlineData("oot.z64", null)]
        [InlineData("ge.z64", null)]
        [InlineData("sm64.z64", "sm64.state")]
        [InlineData("oot.z64", "oot.state")]
        [InlineData("ge.z64", "ge-dam.state")]
        public void A_real_game_stays_byte_exact_with_its_picture_and_sound_frame_by_frame(string romName, string? stateName)
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
            MarsCore oracle = Oracle(expansionPak: true);
            using MarsRtCore twin = Twin(expansionPak: true);
            oracle.SkipRendering = twin.SkipRendering = false;
            oracle.LoadRom(rom);
            twin.LoadRom(rom);
            if (stateName != null)
            {
                byte[] state = File.ReadAllBytes(Path.Combine(folder, stateName));
                oracle.LoadState(new MemoryStream(state));
                twin.LoadState(state);
            }

            var compare = new StateComparer();
            compare.Frame(0, Save(oracle), twin.Save(false));
            SamePicture(0, oracle, twin);
            for (int frame = 1; frame <= frames; frame++)
            {
                Drive(oracle, twin, frame);
                oracle.RunFrame();
                twin.RunFrame();
                compare.Frame(frame, Save(oracle), twin.Save(false));
                SamePicture(frame, oracle, twin);
                SameSound(frame, oracle, twin);
            }
            _output.WriteLine($"{romName} {stateName ?? "from boot"}: {frames} frames, state, picture and sound exact; {oracle.Bus!.Cycles} cycles, {oracle.Cpu!.Instructions} instructions");
        }

        // MarsRT's interpreter against the C# interpreter, blocks off in both, three interleaved rounds from the games' states; C# with its blocks is a reference.
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
            try
            {
                for (int round = 1; round <= 3; round++)
                {
                    var times = new List<string>();
                    foreach (bool blocks in new[] { false, true })
                    {
                        EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks = blocks;
                        MarsCore oracle = Oracle(expansionPak: true);
                        oracle.UseBlocks = blocks;
                        oracle.LoadRom(rom);
                        oracle.LoadState(new MemoryStream(state));
                        times.Add($"C# {(blocks ? "with blocks" : "interpreter")} {Time(frames, oracle.RunFrame):F2} ms");
                    }
                    EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks = false;
                    foreach (bool idle in new[] { false, true })
                    {
                        using MarsRtCore twin = Twin(expansionPak: true);
                        twin.IdleSkip = idle;
                        twin.LoadRom(rom);
                        twin.LoadState(state);
                        times.Add($"MarsRT {(idle ? "idle skip" : "interpreter")} {Time(frames, twin.RunFrame):F2} ms");
                    }
                    _output.WriteLine($"{romName} round {round}, {frames} frames: {string.Join(", ", times)}");
                }
            }
            finally
            {
                EmuSen.Cores.Nintendo.Mars.Rsp.Rsp.UseBlocks = rspBlocks;
            }
        }

        // Random programs of every instruction class over edge-valued registers, stepped by both interpreters from one state and compared whole.
        [Theory]
        [MemberData(nameof(Seeds))]
        public void A_random_program_leaves_the_state_the_csharp_interpreter_leaves(int seed)
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            const int Steps = 24_000, Stride = 16;
            string rom = Temporary(SyntheticN64Rom.Build());
            MarsCore oracle = Oracle();
            oracle.LoadRom(rom);
            Randomise(oracle, new Random(seed));
            byte[] start = Save(oracle);
            oracle.LoadState(new MemoryStream(start));

            using MarsRtCore twin = Twin();
            twin.LoadRom(rom);
            twin.LoadState(start);
            var seen = new HashSet<ulong>();
            long before = oracle.Cpu!.Instructions;
            for (int i = 0; i < Steps; i += Stride)
            {
                for (int j = 0; j < Stride; j++)
                {
                    seen.Add(oracle.Cpu!.Pc);
                    oracle.Cpu!.Step();
                }
                twin.RunSteps(Stride);
                byte[] rust = twin.SaveProcessor();
                using var csharp = new MemoryStream();
                using (var w = new BinaryWriter(csharp, System.Text.Encoding.UTF8, leaveOpen: true)) EmuSen.Common.StateSerializer.Write(w, oracle.Cpu!);
                Assert.True(csharp.ToArray().AsSpan().SequenceEqual(rust) && oracle.Bus!.Cycles == twin.Cycles, $"seed {seed}: the processors part after {i + Stride} steps (C# pc {oracle.Cpu!.CurrentPc:X}, cycles {oracle.Bus!.Cycles} and {twin.Cycles})");
            }
            _output.WriteLine($"seed {seed}: {oracle.Cpu!.Instructions - before} instructions, {seen.Count} addresses");

            new StateComparer().Frame(seed, Save(oracle), twin.Save(false));
            Assert.True(oracle.Cpu!.Instructions > Steps / 20, "almost every step raised, so the test compared little");
        }

        public static TheoryData<int> Seeds()
        {
            var data = new TheoryData<int>();
            for (int seed = 1; seed <= 160; seed++) data.Add(seed);
            return data;
        }

        private static readonly ulong[] Edges =
        {
            0, 1, 2, 3, 0x7F, 0x80, 0xFF, 0x7FFF, 0x8000, 0xFFFF, 0x7FFF_FFFF, 0x8000_0000, 0xFFFF_FFFF, 0x1_0000_0000,
            0x7FFF_FFFF_FFFF_FFFF, 0x8000_0000_0000_0000, 0xFFFF_FFFF_FFFF_FFFF, 0xFFFF_FFFF_8000_0000, 0xFFFF_FFFF_7FFF_FFFF, 0xFFFF_FFFF_FFFF_FFFE,
        };

        // Floats from every class: zeros, normals, the extremes, infinities, both NaNs, and subnormals, in either width.
        private static readonly ulong[] FloatEdges =
        {
            0x0000_0000, 0x8000_0000, 0x3F80_0000, 0xBF80_0000, 0x4040_0000, 0x3EAA_AAAB, 0x7F7F_FFFF, 0x0080_0000, 0x7F80_0000, 0xFF80_0000,
            0x7FC0_0000, 0x7FBF_FFFF, 0x0000_0001, 0x4F00_0000, 0xCF00_0000, 0x3F00_0000,
            0x3FF0_0000_0000_0000, 0xBFF0_0000_0000_0000, 0x4000_0000_0000_0000, 0x7FEF_FFFF_FFFF_FFFF, 0x0010_0000_0000_0000, 0x7FF0_0000_0000_0000,
            0x7FF8_0000_0000_0000, 0x7FF7_FFFF_FFFF_FFFF, 0x0000_0000_0000_0001, 0x43E0_0000_0000_0000, 0xC3E0_0000_0000_0000, 0x4330_0000_0000_0000,
        };

        private static ulong StatusValue(Random r) =>
            0x1000_0000UL | (r.Next(4) == 0 ? 0 : 1UL << 29) | (r.Next(2) == 0 ? 1UL << 26 : 0) | (r.Next(3) == 0 ? 1UL << 25 : 0) | (ulong)r.Next(8) << 5
            | (r.Next(6) == 0 ? (ulong)r.Next(1, 3) << 3 : 0) | (r.Next(2) == 0 ? 0x8401UL : 0) | (r.Next(8) == 0 ? 1UL << 30 : 0);

        private static ulong Value(Random r) => r.Next(3) switch
        {
            0 => Edges[r.Next(Edges.Length)],
            1 => (ulong)(long)(short)r.Next(-40, 40),
            _ => (ulong)r.NextInt64() ^ ((ulong)r.Next(2) << 63),
        };

        private const uint ProgramAt = 0x0400, ProgramWords = 0x2000, DataAt = 0x0010_0000;

        // Registers, FPU, COP0's timer, the TLB and memory set at random; s0-s7 point into the data, t8 and t9 into the program.
        private static void Randomise(MarsCore core, Random r)
        {
            Cpu cpu = core.Cpu!;
            MemoryBus bus = core.Bus!;
            for (int i = 1; i < 32; i++) cpu.Gpr[i] = r.Next(4) == 0 && i > 1 ? cpu.Gpr[r.Next(1, i)] : Value(r);
            for (int i = 16; i < 24; i++) cpu.Gpr[i] = 0xFFFF_FFFF_8000_0000UL | (DataAt + (uint)(i - 16) * 0x1000 + (uint)r.Next(0x100) * 8);
            // v0-a3 hold Status values a move may write: the FPU's half and full modes, 64-bit addressing, reverse endian, the three modes, the interrupt masks.
            for (int i = 2; i < 8; i++) cpu.Gpr[i] = StatusValue(r);
            cpu.Gpr[24] = 0xFFFF_FFFF_8000_0000UL | (ProgramAt + (uint)r.Next((int)ProgramWords) * 4);
            cpu.Gpr[25] = 0xFFFF_FFFF_8000_0000UL | (ProgramAt + (uint)r.Next((int)ProgramWords) * 4);
            for (int i = 0; i < 32; i++) cpu.Fpr[i] = r.Next(3) == 0 ? (ulong)r.NextInt64() : FloatEdges[r.Next(FloatEdges.Length)] | (r.Next(4) == 0 ? (ulong)r.NextInt64() << 32 : 0);
            cpu.Fcsr = (uint)r.Next(4) | (r.Next(3) == 0 ? Cpu.FcsrFlushToZero : 0) | (r.Next(6) == 0 ? (uint)r.Next(0x20) << 7 : 0);
            cpu.Hi = Value(r);
            cpu.Lo = Value(r);
            cpu.Cop0[Cpu.StatusRegister] = 0x3400_0000UL | (r.Next(2) == 0 ? 0x8001UL : 0) | (r.Next(4) == 0 ? 0xE0UL : 0);
            cpu.Cop0[Cpu.CompareRegister] = bus.Count + (uint)r.Next(1, 20_000);
            for (int i = 0; i < 32; i++)
            {
                ref TlbEntry entry = ref cpu.Tlb.Entries[i];
                entry.PageMask = Tlb.PairedPageMask((ulong)r.Next() & Cpu.PageMaskWritable);
                entry.EntryHi = ((ulong)r.Next(0x40) << 13) | (ulong)r.Next(4);
                entry.EntryLo0 = (((ulong)(0x100 + r.Next(0x100)) << 6) | (ulong)r.Next(8) << 3 | (ulong)r.Next(8)) & Tlb.EntryLoKept | (ulong)(i & 1);
                entry.EntryLo1 = (((ulong)(0x100 + r.Next(0x100)) << 6) | (ulong)r.Next(8) << 3 | (ulong)r.Next(8)) & Tlb.EntryLoKept | (ulong)(i & 1);
            }

            // Each vector steps over the faulting instruction and returns; the program and its data are random.
            uint[] handler = { 0x401A_7000, 0x275A_0004, 0x409A_7000, 0x4200_0018 };
            foreach (uint vector in new uint[] { 0x000, 0x080, 0x180 })
                for (int i = 0; i < handler.Length; i++) Put(bus.Rdram, vector + (uint)i * 4, handler[i]);
            for (uint i = 0; i < ProgramWords; i++) Put(bus.Rdram, ProgramAt + i * 4, Instruction(r));
            for (uint i = 0; i < 0x8000; i += 4) Put(bus.Rdram, DataAt + i, (uint)Value(r));
            cpu.Pc = 0xFFFF_FFFF_8000_0000UL | ProgramAt;
            cpu.NextPc = cpu.Pc + 4;
            cpu.Cop0Written();
        }

        private static void Put(byte[] rdram, uint at, uint word) => System.Buffers.Binary.BinaryPrimitives.WriteUInt32BigEndian(rdram.AsSpan((int)at), word);

        private static readonly uint[] SpecialFunctions =
        {
            0x00, 0x02, 0x03, 0x04, 0x06, 0x07, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F,
            0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x36, 0x38, 0x3A, 0x3B, 0x3C, 0x3E, 0x3F,
        };

        private static readonly uint[] MemoryOps =
        {
            0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x1A, 0x1B, 0x37, 0x3F, 0x30, 0x34, 0x38, 0x3C, 0x31, 0x35, 0x39, 0x3D, 0x2F,
        };

        private static readonly int[] Cop0Registers = { 0, 1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 13, 14, 16, 17, 20, 26, 27, 28, 29, 30, 7, 31 };

        private static uint Instruction(Random r)
        {
            uint R(int bits) => (uint)r.Next(1 << bits);
            uint Imm() => r.Next(2) == 0 ? R(16) : (ushort)(short)r.Next(-9, 9);
            // A third of the two-register forms name one register twice, so equal operands are common.
            uint Pair() { uint rs = R(5); return rs << 21 | (r.Next(3) == 0 ? rs : R(5)) << 16; }
            // Branches mostly forward, so a loop a taken branch closes does not hold the program in a few words.
            uint Offset() => (ushort)(short)(r.Next(32) == 0 ? r.Next(-8, 0) : r.Next(1, 9));
            int roll = r.Next(100);
            return roll switch
            {
                < 22 => Pair() | R(5) << 11 | R(5) << 6 | SpecialFunctions[r.Next(SpecialFunctions.Length)],
                < 34 => (uint)new[] { 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x18, 0x19 }[r.Next(10)] << 26 | R(5) << 21 | R(5) << 16 | Imm(),
                < 48 => MemoryOps[r.Next(MemoryOps.Length)] << 26 | (uint)r.Next(16, 24) << 21 | R(5) << 16 | (ushort)(short)(r.Next(-16, 16) * (r.Next(3) == 0 ? 1 : 8)),
                < 55 => (uint)new[] { 0x04, 0x05, 0x06, 0x07, 0x14, 0x15, 0x16, 0x17 }[r.Next(8)] << 26 | Pair() | Offset(),
                < 58 => 0x0400_0000u | R(5) << 21 | (uint)new[] { 0, 1, 2, 3, 0x10, 0x11, 0x12, 0x13, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0E }[r.Next(14)] << 16 | Offset(),
                < 60 => (r.Next(2) == 0 ? 0x0800_0000u : 0x0C00_0000u) | ((0x8000_0000u | ProgramAt + (uint)r.Next((int)ProgramWords) * 4) >> 2 & 0x03FF_FFFF),
                < 61 => (uint)r.Next(24, 26) << 21 | R(5) << 11 | (r.Next(2) == 0 ? 0x08u : 0x09u),
                < 76 => 0x4400_0000u | (uint)new[] { 0x10, 0x11, 0x14, 0x15, 0x10, 0x11 }[r.Next(6)] << 21 | R(5) << 16 | R(5) << 11 | R(5) << 6 | R(6),
                < 81 => 0x4400_0000u | (uint)new[] { 0, 1, 2, 4, 5, 6, 8, 3, 7 }[r.Next(9)] << 21 | R(5) << 16 | (r.Next(3) == 0 ? 31u : R(5)) << 11 | Offset(),
                < 86 => 0x4000_0000u | (uint)new[] { 0, 1, 4, 5, 2, 8 }[r.Next(6)] << 21 | R(5) << 16 | (uint)Cop0Registers[r.Next(Cop0Registers.Length)] << 11,
                < 87 => 0x4080_0000u | (uint)r.Next(2, 8) << 16 | 12u << 11,
                < 88 => new uint[] { 0x4200_0001, 0x4200_0002, 0x4200_0006, 0x4200_0008, 0x4200_0018, 0x4200_0010, 0x4200_0020 }[r.Next(7)],
                < 90 => new uint[] { 0x0000_000C, 0x0000_000D, 0x0000_000F }[r.Next(3)],
                < 93 => 0x4800_0000u | (uint)new[] { 0, 1, 2, 4, 5, 6, 3 }[r.Next(7)] << 21 | R(5) << 16 | R(5) << 11,
                < 95 => 0xBC00_0000u | (uint)r.Next(16, 24) << 21 | R(5) << 16 | (ushort)(short)(r.Next(-8, 8) * 4),
                _ => (uint)r.Next() ^ ((uint)r.Next(2) << 31),
            };
        }

        // The samples each core played since the last frame, drained whole, and the rate it plays them at.
        private static void SameSound(int frame, MarsCore oracle, MarsRtCore twin)
        {
            short[] want = oracle.DequeueAudioSamples(1 << 20), got = twin.DequeueAudioSamples(1 << 20);
            if (oracle.AudioSampleRate != twin.AudioSampleRate || !want.AsSpan().SequenceEqual(got))
                throw new DivergedException($"frame {frame}: C# played {want.Length} samples at {oracle.AudioSampleRate} Hz, Rust {got.Length} at {twin.AudioSampleRate} Hz, the first difference at {want.AsSpan().CommonPrefixLength(got)}");
        }

        // The picture each core shows: its size, its rows' repeat and every byte.
        private static void SamePicture(int frame, MarsCore oracle, MarsRtCore twin)
        {
            byte[] want = oracle.GetFrameBufferRgba(), got = twin.GetFrameBufferRgba();
            if (oracle.ScreenWidth != twin.ScreenWidth || oracle.ScreenHeight != twin.ScreenHeight || oracle.RowRepeat != twin.RowRepeat)
                throw new DivergedException($"frame {frame}: C# shows {oracle.ScreenWidth}x{oracle.ScreenHeight} rows x{oracle.RowRepeat}, Rust {twin.ScreenWidth}x{twin.ScreenHeight} x{twin.RowRepeat}");
            if (!want.AsSpan().SequenceEqual(got))
                throw new DivergedException($"frame {frame}: the pictures differ from byte {want.AsSpan().CommonPrefixLength(got)} of {want.Length}");
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

        // Two states compared by the C# serializer's field names, so a difference names the fields it is in.
        private sealed class StateComparer
        {
            public void Frame(int frame, byte[] csharp, byte[] rust)
            {
                if (csharp.AsSpan().SequenceEqual(rust)) return;
                if (csharp.Length != rust.Length) throw new DivergedException($"frame {frame}: C# wrote {csharp.Length} bytes and Rust {rust.Length}");

                using var machine = new MarsMachine(BitConverter.ToInt32(csharp, 8));
                machine.Load(csharp);
                var fields = machine.Layout(snapshot: false).Split('\n', StringSplitOptions.RemoveEmptyEntries).Select(line => line.Split(' ')).Select(p => (Offset: int.Parse(p[0]), Length: int.Parse(p[1]), Path: p[3])).ToArray();
                var differing = new List<string>();
                foreach (var (offset, length, path) in fields)
                {
                    if (!csharp.AsSpan(offset, length).SequenceEqual(rust.AsSpan(offset, length))) differing.Add(Describe(path, csharp, rust, offset, length));
                }
                throw new DivergedException($"frame {frame}: {string.Join("; ", differing.Take(8))}{(differing.Count > 8 ? $" and {differing.Count - 8} more" : "")}");
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
