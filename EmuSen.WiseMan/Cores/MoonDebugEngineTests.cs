using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using EmuSen.Cores;
using EmuSen.Crystal.Systems.Nes;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // One NES engine behind its debug target, as the frontends reach it through CoreFactory - see Moon_Native.md §8.4.
    public sealed class NesDebugRig : IDisposable
    {
        public const string MoonRt = "MoonRT";
        public static readonly TheoryData<string> Engines = new() { MoonRt };

        public readonly ICore Core;
        public readonly IDebugTarget Target;

        private NesDebugRig(ICore core, IDebugTarget target)
        {
            Core = core;
            Target = target;
        }

        public static NesDebugRig Load(string engine, string path, CheatRegistry? cheats = null)
        {
            Assert.True(MoonRtCore.Available, MoonNative.Report);
            CoreBundle bundle = CoreFactory.Load(path, cheats: cheats, options: TestCores.NoBattery);
            Assert.IsType<MoonRtCore>(bundle.Core);
            return new NesDebugRig(bundle.Core, bundle.DebugTarget);
        }

        public BreakpointRegistry Breakpoints => Target.Breakpoints;
        public CallStackRegistry CallStack => Target.CallStack!;
        public CoverageRegistry Coverage => Target.Coverage!;
        public WatchRegistry Watches => Target.Watches;

        public bool Halted => ((MoonRtCore)Core).IsHaltedAtBreakpoint;
        public int HaltedAddress => ((MoonRtCore)Core).HaltedAddress;
        public int Pc => Target.DebugCpus[0].ProgramCounter!();

        public byte[] State()
        {
            using var stream = new MemoryStream();
            Core.SaveState(stream);
            return stream.ToArray();
        }

        public string StateHash() => Convert.ToHexString(SHA256.HashData(State()))[..16];

        public void Dispose() => (Core as IDisposable)?.Dispose();
    }

    // The debugger's behaviours on MoonRT, as the frontends reach it - see Moon_Native.md §8.4.
    public class MoonDebugEngineTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly List<string> _files = new();

        public MoonDebugEngineTests(ITestOutputHelper output)
        {
            _output = output;

        }

        public void Dispose()
        {
            foreach (string path in _files) try { File.Delete(path); } catch (IOException) { }
        }

        // NMI on, then JSR $8010 (which stores $0300) and INC $0042 in a loop; the NMI handler at $8020 counts in $0043.
        public static readonly byte[] Program =
        {
            0xA9, 0x80, 0x8D, 0x00, 0x20,
            0x20, 0x10, 0x80,
            0xEE, 0x42, 0x00,
            0x4C, 0x05, 0x80,
            0xEA, 0xEA,
            0xA9, 0x07, 0x8D, 0x00, 0x03, 0x60,
        };

        public string Rom()
        {
            byte[] image = SyntheticNesRom.Build(patches: new (int, byte[])[] { (0, Program), (0x20, new byte[] { 0xEE, 0x43, 0x00, 0x40 }) });
            image[16 + 0x3FFA] = 0x20;
            image[16 + 0x3FFB] = 0x80;
            string path = SyntheticNesRom.WriteTemp(image);
            _files.Add(path);
            return path;
        }

        private NesDebugRig Load(string engine) => NesDebugRig.Load(engine, Rom());

        [Theory]
        [MemberData(nameof(NesDebugRig.Engines), MemberType = typeof(NesDebugRig))]
        public void A_breakpoint_halts_in_front_of_its_instruction_and_a_resume_runs_it(string engine)
        {
            using var rig = Load(engine);
            int id = rig.Breakpoints.AddBreakpoint(0x8010);
            rig.Core.RunFrame();
            Assert.True(rig.Halted);
            Assert.Equal((0x8010, 0x8010, 0L), (rig.HaltedAddress, rig.Pc, rig.Core.TotalFrames));
            Assert.Equal(1, rig.CallStack.Depth);
            Assert.Equal((0x8005, 0x8010), (rig.CallStack.Frames[^1].Source, rig.CallStack.Frames[^1].Target));

            rig.Core.RunFrame();
            Assert.True(rig.Halted, "the loop comes round to the breakpoint again");
            Assert.Equal(0x8010, rig.HaltedAddress);
            rig.Breakpoints.RemoveBreakpoint(id);
            rig.Core.RunFrame();
            Assert.False(rig.Halted);
            Assert.Equal(1, rig.Core.TotalFrames);
        }

        [Theory]
        [MemberData(nameof(NesDebugRig.Engines), MemberType = typeof(NesDebugRig))]
        public void Stepping_in_over_and_out_lands_where_the_call_stack_says(string engine)
        {
            using var rig = Load(engine);
            rig.Breakpoints.AddBreakpoint(0x8005);
            rig.Core.RunFrame();
            rig.Breakpoints.RemoveBreakpoint(rig.Breakpoints.GetBreakpoints().Single().Id);

            rig.Breakpoints.ArmSingleStep();
            rig.Core.RunFrame();
            Assert.Equal((true, 0x8010), (rig.Halted, rig.Pc));
            rig.Breakpoints.ArmSingleStep();
            rig.Core.RunFrame();
            Assert.Equal(0x8012, rig.Pc);

            Assert.True(rig.Breakpoints.ArmStepToDepth(rig.CallStack.Depth - 1));
            rig.Core.RunFrame();
            Assert.Equal((true, 0x8008, 0), (rig.Halted, rig.Pc, rig.CallStack.Depth));

            rig.Breakpoints.ArmStep(2);
            rig.Core.RunFrame();
            Assert.Equal(0x8005, rig.Pc);
            Assert.True(rig.Breakpoints.ArmStepToDepth(rig.CallStack.Depth));
            rig.Core.RunFrame();
            Assert.Equal((true, 0x8008), (rig.Halted, rig.Pc));
        }

        [Theory]
        [MemberData(nameof(NesDebugRig.Engines), MemberType = typeof(NesDebugRig))]
        public void Run_to_the_NMI_and_to_a_frame_halt_where_the_registry_says(string engine)
        {
            using var rig = Load(engine);
            rig.Breakpoints.ArmRunToInterrupt(CallFrameKind.Nmi);
            rig.Core.RunFrame();
            Assert.Equal((true, 0x8020), (rig.Halted, rig.Pc));
            Assert.Equal(CallFrameKind.Nmi, rig.CallStack.Frames[^1].Kind);

            rig.Breakpoints.ArmRunToFrame(3);
            for (int i = 0; i < 4 && !(rig.Halted && rig.Core.TotalFrames == 3); i++) rig.Core.RunFrame();
            Assert.True(rig.Halted);
            Assert.Equal(3, rig.Core.TotalFrames);
            Assert.Equal("frame 3 reached", rig.Breakpoints.LastBreakReason);
        }

        [Theory]
        [MemberData(nameof(NesDebugRig.Engines), MemberType = typeof(NesDebugRig))]
        public void A_watch_logs_the_cpus_store_and_a_data_breakpoint_halts_after_it(string engine)
        {
            using var rig = Load(engine);
            int watch = rig.Watches.AddWatch("RAM", 0x0300, 1);
            rig.Breakpoints.AddDataBreakpoint("RAM", 0x0042);
            rig.Core.RunFrame();
            Assert.True(rig.Halted);
            Assert.Equal(0x800B, rig.Pc);
            DebugWatchEvent e = rig.Watches.GetEvents(watch).First();
            Assert.Equal((0x0300, (byte)0x07, "PC=$8012"), (e.Address, e.Value, e.Context));
        }

        [Theory]
        [MemberData(nameof(NesDebugRig.Engines), MemberType = typeof(NesDebugRig))]
        public void Coverage_and_the_profile_record_what_ran(string engine)
        {
            using var rig = Load(engine);
            rig.Coverage.Arm();
            rig.CallStack.ArmProfiler();
            rig.Core.RunFrame();
            rig.Core.RunFrame();
            Assert.True(rig.Coverage.WasExecuted(0x8010) && rig.Coverage.WasExecuted(0x8020) && !rig.Coverage.WasExecuted(0x800E));
            Assert.Contains(rig.CallStack.Hottest(5), h => h.Address == 0x8010);
        }

        // A store to a PPU register is logged under PPUREG with the register's number: the program's STA $2000 at $8002.
        [Theory]
        [MemberData(nameof(NesDebugRig.Engines), MemberType = typeof(NesDebugRig))]
        public void A_watch_on_the_ppu_registers_logs_the_store_under_its_register(string engine)
        {
            using var rig = Load(engine);
            int ppu = rig.Watches.AddWatch("PPUREG", 0, 8);
            int ram = rig.Watches.AddWatch("RAM", 0, 8);
            rig.Core.RunFrame();
            DebugWatchEvent e = rig.Watches.GetEvents(ppu).First();
            Assert.Equal((0, (byte)0x80, "PC=$8002"), (e.Address, e.Value, e.Context));
            Assert.Empty(rig.Watches.GetEvents(ram));
        }

        // A CPU-bus store from the debugger is reported as the processor's are.
        [Theory]
        [MemberData(nameof(NesDebugRig.Engines), MemberType = typeof(NesDebugRig))]
        public void A_store_through_the_bus_space_reaches_a_watch(string engine)
        {
            using var rig = Load(engine);
            int id = rig.Watches.AddWatch("RAM", 0x0042, 1);
            rig.Target.GetMemorySpaces().Single(s => s.Name == NesSystem.CpuBus).Write(0x0042, 0x7E);
            DebugWatchEvent e = Assert.Single(rig.Watches.GetEvents(id));
            Assert.Equal((0x42, (byte)0x7E), (e.Address, e.Value));
        }

        // A halt and a resume leave the machine where a run without the breakpoint leaves it: the halted scanline's cycles are earned once.
        [Theory]
        [MemberData(nameof(NesDebugRig.Engines), MemberType = typeof(NesDebugRig))]
        public void A_halt_and_its_resume_leave_the_machine_as_an_unhalted_run(string engine)
        {
            string rom = Rom();
            using var plain = NesDebugRig.Load(engine, rom);
            using var halted = NesDebugRig.Load(engine, rom);
            int id = halted.Breakpoints.AddBreakpoint(0x8010);
            halted.Core.RunFrame();
            Assert.True(halted.Halted);
            halted.Breakpoints.RemoveBreakpoint(id);
            for (int f = 0; f < 3; f++)
            {
                plain.Core.RunFrame();
                halted.Core.RunFrame();
                Assert.Equal(plain.StateHash(), halted.StateHash());
            }
        }
    }
}

namespace EmuSen.WiseMan.Cores
{
    // MoonRT with every table armed and nothing hit, against MoonRT with none: picture, sound and state after every frame - see Moon_Native.md §8.4.
    public class MoonRtArmedEqualsPlainTests
    {
        private readonly ITestOutputHelper _output;

        public MoonRtArmedEqualsPlainTests(ITestOutputHelper output) => _output = output;

        // A breakpoint no game here executes, a watch and a data breakpoint no store matches, coverage over all 64K and the profiler.
        public static void ArmEverything(MoonRtCore core)
        {
            core.CreateDebugTarget();
            core.Breakpoints.AddBreakpoint(0x5FF0, 0x5FFF, null);
            core.Watches.AddWatch(NesSystem.Ram, 0, 0x800);
            core.Watches.AddWatch(NesSystem.PrgRam, 0, 0x2000);
            core.Breakpoints.AddDataBreakpoint(NesSystem.PrgRam, 0x1FFF, 0x1FFF, 0x5A, false, false, "false");
            core.Coverage.Arm();
            core.CallStack.ArmProfiler();
        }

        public static void RunAlike(MoonRtCore plain, MoonRtCore armed, int frames, int from = 0)
        {
            byte[] a = new byte[MoonMachine.FrameBytes], b = new byte[MoonMachine.FrameBytes];
            for (int f = from; f < from + frames; f++)
            {
                int p = f % 90;
                foreach (MoonRtCore core in new[] { plain, armed })
                {
                    core.SetButton(0, EmuSen.Galaxia.Input.PadButton.Start, p < 5);
                    core.SetButton(0, EmuSen.Galaxia.Input.PadButton.A, p >= 45 && p < 50);
                    core.RunFrame();
                }
                Assert.False(armed.IsHaltedAtBreakpoint, $"frame {f}: the armed run halted at ${armed.HaltedAddress:X4}");
                Assert.True(plain.DequeueAudioSamples(int.MaxValue).AsSpan().SequenceEqual(armed.DequeueAudioSamples(int.MaxValue)), $"frame {f}: the sound differs");
                plain.Machine.CopyFrame(a);
                armed.Machine.CopyFrame(b);
                Assert.True(a.AsSpan().SequenceEqual(b), $"frame {f}: the pictures differ");
                Assert.True(plain.Machine.Save().AsSpan().SequenceEqual(armed.Machine.Save()), $"frame {f}: the states differ");
            }
        }

        [Fact]
        public void The_bench_games_armed_run_as_they_run_plain()
        {
            var runs = MoonGoldenTests.Games.Take(4).Select(g => MoonGoldenTests.Game(g.Sha)).OfType<byte[]>().ToList();
            if (runs.Count == 0)
            {
                _output.WriteLine($"{MoonGoldenTests.GamesVariable} unset or without the games, not run");
                return;
            }
            foreach (byte[] image in runs)
            {
                string rom = SyntheticNesRom.WriteTemp(image);
                using var plain = new MoonRtCore() { Options = TestCores.NoBattery };
                using var armed = new MoonRtCore() { Options = TestCores.NoBattery };
                plain.LoadRom(rom);
                armed.LoadRom(rom);
                File.Delete(rom);
                ArmEverything(armed);
                RunAlike(plain, armed, 900);
                Assert.True(armed.Coverage.InstructionsRecorded > 100_000, "the armed run did not take the observed frame");
                Assert.True(armed.Watches.GetWatches().Any(w => armed.Watches.GetEvents(w.Id).Count > 0), "no store reached the watches");
                _output.WriteLine($"{GoldenTrace.Hash(image)}: alike, {armed.Coverage.InstructionsRecorded} instructions covered");
            }
        }
    }
}
