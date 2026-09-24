using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Debug;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MercuryRT under the debugger's claims, held to the C# core's transcripts, and the proofs that the hooks change nothing the machine computes - see Mercury_Native.md §8.5.
    [Collection(TestCollections.ProcessGlobals)]
    public class MercuryRtDebugTests : MercuryDebugClaims
    {
        private readonly ITestOutputHelper _output;

        public MercuryRtDebugTests(ITestOutputHelper output)
        {
            _output = output;
            Assert.True(MercuryRtCore.Available, MercuryNative.Report);
            CoreOptions.BatteryRamDisabled = true;
        }

        protected override MercuryDebugRig Rig(string rom) => MercuryDebugRig.MercuryRt(rom);

        // A call pushes in an observed frame; one is armed by a breakpoint nothing reaches.
        [Fact]
        public void A_call_made_in_an_observed_frame_is_on_the_stack()
        {
            MercuryDebugRig core = Load((0, new byte[] { 0xCD, 0x60, 0x01 }), (0x10, new byte[] { 0x18, 0xFE }));
            core.Breakpoints.AddBreakpoint(0x7000);

            core.RunFrame();

            Assert.False(core.IsHaltedAtBreakpoint);
            Assert.Equal(1, core.CallStack!.Depth);
        }

        // The plain frame keeps no stack: what the C# core's A_call_made_in_a_frame_nothing_observes_is_on_the_stack sees, MercuryRT sees only while observed.
        [Fact]
        public void A_plain_frame_leaves_the_stack_where_the_last_observed_frame_left_it()
        {
            MercuryDebugRig core = Load((0, new byte[] { 0xCD, 0x60, 0x01 }), (0x10, new byte[] { 0x18, 0xFE }));

            core.RunFrame();

            Assert.Equal(0, core.CallStack!.Depth);
        }

        // The registry's stack is pushed down with the tables, so a reset or a load of it is MercuryRT's depth too.
        [Fact]
        public void The_registrys_stack_is_the_depth_the_machine_stops_at()
        {
            MercuryDebugRig core = Load(CallOnce);
            HaltAt(core, 0x162);
            var rt = (MercuryRtCore)core.Core;
            Assert.Equal(1, rt.NativeDepth);

            core.CallStack!.Reset();
            core.Breakpoints.ArmStepToDepth(0);
            core.RunFrame();

            Assert.Equal(0x163, core.HaltedAddress);
            Assert.Equal(0, rt.NativeDepth);
        }

        // C#'s bus reports stores only to a target that observes it; MercuryRT's core logs none until one exists, and none that no range covers.
        [Fact]
        public void A_store_is_reported_only_while_a_target_listens()
        {
            string rom = Temporary(CountForever);
            var cs = new MercuryCore();
            cs.LoadRom(rom);
            using var rt = new MercuryRtCore();
            rt.LoadRom(rom);
            int a = cs.Watches.AddWatch("WRAM", 0, 1), b = rt.Watches.AddWatch("WRAM", 0, 1);
            cs.RunFrame();
            rt.RunFrame();
            Assert.Empty(cs.Watches.GetEvents(a));
            Assert.Empty(rt.Watches.GetEvents(b));
            Assert.False(rt.Listening);

            _ = new MercuryDebugTarget(cs);
            _ = rt.CreateDebugTarget();
            Assert.True(rt.Listening);
            cs.RunFrame();
            rt.RunFrame();
            Assert.Equal(cs.Watches.GetEventCounts(a), rt.Watches.GetEventCounts(b));
            Assert.Equal(Events(cs.Watches, a), Events(rt.Watches, b));

            rt.Watches.RemoveWatch(b);
            Assert.False(rt.Listening);
        }

        // Every table armed, and every stop one the registry refuses, against the C# core armed the same and the C# core plain.
        [Fact]
        public void Synthetic_programs_with_every_table_armed_are_the_programs_run_plain()
        {
            foreach (var (name, program) in new[] { ("interrupts", Interrupts), ("count", CountForever), ("call", CallOnce), ("vram", StoresAcross(0x80, 0x6C)), ("oam", StoresAcross(0xFE, 0x7D)) })
                Armed(Temporary(program), name, 300);
        }

        // ld hl,<page>00 / ld (hl+),a / inc a / bit <bit> / jr z,the store / jr the start: stores through the whole of a space, drawing or not, so some the bus refuses.
        private static (int, byte[])[] StoresAcross(byte page, byte bit)
        {
            return new[] { (0, new byte[] { 0x21, 0x00, page, 0x22, 0x3C, 0xCB, bit, 0x28, 0xFA, 0x18, 0xF5 }) };
        }

        [Fact]
        public void A_game_with_every_table_armed_is_the_game_run_plain()
        {
            foreach (string path in Games())
                Armed(path, Path.GetFileName(path), 600);
        }

        // A breakpoint at the vblank vector on every third frame, halted at and resumed, on both engines: the halted states compared, and each frame's end against the plain run.
        [Fact]
        public void A_game_halted_at_breakpoints_and_resumed_is_the_game_run_through()
        {
            foreach (string path in Games().Prepend(Temporary(Interrupts)))
            {
                var plain = new MercuryCore();
                plain.LoadRom(path);
                var cs = new MercuryCore();
                cs.LoadRom(path);
                var csTarget = new MercuryDebugTarget(cs);
                using var rt = new MercuryRtCore();
                rt.LoadRom(path);
                MercuryRtDebugTarget rtTarget = rt.CreateDebugTarget();

                int halts = 0;
                for (int f = 0; f < 900; f++)
                {
                    Press(f, plain, cs, rt);
                    plain.RunFrame();
                    int? a = f % 3 == 0 ? cs.Breakpoints.AddBreakpoint(0x40) : null, b = f % 3 == 0 ? rt.Breakpoints.AddBreakpoint(0x40) : null;
                    cs.RunFrame();
                    rt.RunFrame();
                    Assert.Equal(cs.IsHaltedAtBreakpoint, rt.IsHaltedAtBreakpoint);
                    if (a is { } ida && b is { } idb)
                    {
                        cs.Breakpoints.RemoveBreakpoint(ida);
                        rt.Breakpoints.RemoveBreakpoint(idb);
                    }
                    if (cs.IsHaltedAtBreakpoint)
                    {
                        halts++;
                        Assert.Equal(cs.HaltedAddress, rt.HaltedAddress);
                        AssertSame(State(cs), State(rt), $"{Path.GetFileName(path)}, halted in frame {f}");
                        cs.RunFrame();
                        rt.RunFrame();
                    }
                    AssertSame(State(plain), State(rt), $"{Path.GetFileName(path)}, frame {f}");
                    AssertSame(State(plain), State(cs), $"{Path.GetFileName(path)}, frame {f} (C#)");
                    Assert.True(plain.GetFrameBufferRgba().AsSpan().SequenceEqual(rt.GetFrameBufferRgba()), $"frame {f}: pictures differ");
                    Assert.True(plain.DequeueAudioSamples(int.MaxValue).AsSpan().SequenceEqual(rt.DequeueAudioSamples(int.MaxValue)), $"frame {f}: sound differs");
                    cs.DequeueAudioSamples(int.MaxValue);
                }
                _output.WriteLine($"{Path.GetFileName(path)}: {halts} halts at $0040 in 900 frames, every halted state and every frame identical");
                Assert.True(halts > 0, "no halts");
            }
        }

        // The armed run: C# armed and MercuryRT armed against C# plain, every frame, and the registries' transcripts compared at the end.
        private void Armed(string path, string name, int frames)
        {
            var plain = new MercuryCore();
            plain.LoadRom(path);
            var cs = new MercuryCore();
            cs.LoadRom(path);
            _ = new MercuryDebugTarget(cs);
            using var rt = new MercuryRtCore();
            rt.LoadRom(path);
            _ = rt.CreateDebugTarget();
            var watches = new (int Cs, int Rt)[5];
            foreach (var (bp, watch, cov, stack, i) in new[] { (cs.Breakpoints, cs.Watches, cs.Coverage, cs.CallStack, 0), (rt.Breakpoints, rt.Watches, rt.Coverage, rt.CallStack, 1) })
            {
                bp.AddBreakpoint(0xFEFF);
                bp.AddBreakpoint(0x40, 0x40, null, "vblank");
                bp.AddBreakpoint(0x48, 0x5F, null, null);
                bp.AddForbidRange(0x48, 0x5F);
                bp.AddDataBreakpoint("WRAM", 0, 0xFF, 0x100, onRead: false, changedOnly: false, condition: null);
                bp.AddDataBreakpoint("HRAM", 0, 0x7E, 0x100, onRead: false, changedOnly: false, condition: null);
                bp.ArmDepthGuard(512);
                cov.Arm();
                stack.ArmProfiler();
                int[] ids = { watch.AddWatch("WRAM", 0, 0x1000), watch.AddWatch("HRAM", 0, 0x7F), watch.AddWatch("OAM", 0, 0xA0), watch.AddWatch("VRAM", 0, 0x2000), watch.AddWatch("cartram", 0, 0x2000) };
                for (int k = 0; k < ids.Length; k++)
                {
                    if (i == 0) watches[k].Cs = ids[k];
                    else watches[k].Rt = ids[k];
                }
            }

            for (int f = 0; f < frames; f++)
            {
                Press(f, plain, cs, rt);
                plain.RunFrame();
                cs.RunFrame();
                rt.RunFrame();
                Assert.False(cs.IsHaltedAtBreakpoint || rt.IsHaltedAtBreakpoint, $"{name}, frame {f}: halted");
                byte[] want = State(plain);
                AssertSame(want, State(rt), $"{name}, frame {f}");
                AssertSame(want, State(cs), $"{name}, frame {f} (C# armed)");
                Assert.True(plain.GetFrameBufferRgba().AsSpan().SequenceEqual(rt.GetFrameBufferRgba()), $"{name}, frame {f}: pictures differ");
                short[] sound = plain.DequeueAudioSamples(int.MaxValue);
                Assert.True(sound.AsSpan().SequenceEqual(rt.DequeueAudioSamples(int.MaxValue)), $"{name}, frame {f}: sound differs");
                Assert.True(sound.AsSpan().SequenceEqual(cs.DequeueAudioSamples(int.MaxValue)), $"{name}, frame {f}: C# armed sound differs");
            }

            Assert.Equal(Bps(cs.Breakpoints), Bps(rt.Breakpoints));
            Assert.Equal(cs.Breakpoints.LogEntriesRecorded, rt.Breakpoints.LogEntriesRecorded);
            Assert.Equal(cs.Coverage.InstructionsRecorded, rt.Coverage.InstructionsRecorded);
            Assert.True(cs.Coverage.Export().AsSpan().SequenceEqual(rt.Coverage.Export()), $"{name}: coverage maps differ");
            Assert.Equal(cs.CallStack.ProfiledInstructions, rt.CallStack.ProfiledInstructions);
            // Hottest breaks ties in the order routines were first charged, which a drained run need not share, so the whole profile is compared by address.
            Assert.Equal(cs.CallStack.Hottest(int.MaxValue).OrderBy(h => h.Address), rt.CallStack.Hottest(int.MaxValue).OrderBy(h => h.Address));
            Assert.Equal(cs.CallStack.Backtrace(), rt.CallStack.Backtrace());
            Assert.Equal(cs.CallStack.UnmatchedReturns, rt.CallStack.UnmatchedReturns);
            Assert.Equal(cs.Coverage.EntryPoints(1000), rt.Coverage.EntryPoints(1000));
            long stored = 0;
            foreach (var (a, b) in watches)
            {
                Assert.Equal(cs.Watches.GetEventCounts(a), rt.Watches.GetEventCounts(b));
                Assert.Equal(cs.Watches.GetSiteHits(a), rt.Watches.GetSiteHits(b));
                Assert.Equal(Events(cs.Watches, a), Events(rt.Watches, b));
                stored += cs.Watches.GetEventCounts(a).Total;
            }
            _output.WriteLine($"{name}: {frames} frames armed, identical; {cs.Coverage.InstructionsRecorded} steps recorded and charged, {stored} stores watched, " +
                              $"{cs.Breakpoints.LogEntriesRecorded} logpoint passes, {cs.Coverage.EntryPointCount} entry points, depth {cs.CallStack.Depth}, {cs.CallStack.UnmatchedReturns} unmatched returns");
        }

        private static IEnumerable<string> Games()
        {
            string? folder = Environment.GetEnvironmentVariable(MercuryRtStateTests.RomsVariable);
            if (folder is null || !Directory.Exists(folder)) return Array.Empty<string>();
            return Directory.GetFiles(folder, "*.gb*").Order(StringComparer.Ordinal);
        }

        private static void Press(int f, params ICore[] cores)
        {
            int k = f % 90;
            foreach (ICore core in cores)
            {
                core.SetButton(0, PadButton.Start, k < 5);
                core.SetButton(0, PadButton.A, k is >= 45 and < 50);
            }
        }

        private static byte[] State(ICore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        private static void AssertSame(byte[] want, byte[] got, string what)
        {
            int same = want.AsSpan().CommonPrefixLength(got);
            Assert.True(same == want.Length && want.Length == got.Length, $"{what}: states differ at byte {same}");
        }

        private static string Bps(BreakpointRegistry registry) =>
            string.Join(";", registry.GetBreakpoints().Select(b => $"{b.Id}:{b.HitCount}:{b.Condition}")) + "|" +
            string.Join(";", registry.GetDataBreakpoints().Select(b => $"{b.Id}:{b.HitCount}"));

        private static IEnumerable<(long, int, byte, string)> Events(WatchRegistry watches, int id) =>
            watches.GetEvents(id, 500).Select(e => (e.Sequence, e.Address, e.Value, e.Context));
    }
}
