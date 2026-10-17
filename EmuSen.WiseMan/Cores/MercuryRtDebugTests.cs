using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Crystal.Debugger;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MercuryRT under the debugger's claims, and the proofs that the hooks change nothing the machine computes - see Mercury_Native.md §8.5, §15.
    [Collection(TestCollections.ProcessGlobals)]
    public class MercuryRtDebugTests : MercuryDebugClaims
    {
        private readonly ITestOutputHelper _output;

        public MercuryRtDebugTests(ITestOutputHelper output)
        {
            _output = output;
            Assert.True(MercuryRtCore.Available, MercuryNative.Report);

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

        // The plain frame keeps no stack: the stack is tracked only while a frame is observed - see Mercury_Native.md §8.5.
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

        // The core logs no store until a target listens, and none that no range covers.
        [Fact]
        public void A_store_is_reported_only_while_a_target_listens()
        {
            string rom = Temporary(CountForever);
            using var rt = new MercuryRtCore();
            rt.LoadRom(rom);
            int b = rt.Watches.AddWatch("WRAM", 0, 1);
            rt.RunFrame();
            Assert.Empty(rt.Watches.GetEvents(b));
            Assert.False(rt.Listening);

            _ = rt.CreateDebugTarget();
            Assert.True(rt.Listening);
            rt.RunFrame();
            Assert.NotEmpty(rt.Watches.GetEvents(b));
            Assert.All(Events(rt.Watches, b), e => Assert.Equal(0, e.Item2));

            rt.Watches.RemoveWatch(b);
            Assert.False(rt.Listening);
        }

        // Every table armed, and every stop one the registry refuses, against the same program run plain.
        [Fact]
        public void Synthetic_programs_with_every_table_armed_are_the_programs_run_plain()
        {
            foreach (var (name, program) in new[] { ("interrupts", Interrupts), ("count", CountForever), ("call", CallOnce), ("vram", StoresAcross(0x80, 0x6C)), ("oam", StoresAcross(0xFE, 0x7D)) })
                Armed(Temporary(program), name, 300);
            // HDMA in double speed, and an hblank transfer held back by HALT, which the observed step passes on to the bus - see Mercury_Native.md §14.2.
            string colour = SyntheticGbRom.WriteTemp(MercuryPrograms.ColourRom());
            try { Armed(colour, "colour", 300); }
            finally { File.Delete(colour); }
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

        // A breakpoint at the vblank vector on every third frame, halted at and resumed: each frame's end against the plain run.
        [Fact]
        public void A_game_halted_at_breakpoints_and_resumed_is_the_game_run_through()
        {
            foreach (string path in Games().Prepend(Temporary(Interrupts)))
            {
                using var plain = new MercuryRtCore();
                plain.LoadRom(path);
                using var rt = new MercuryRtCore();
                rt.LoadRom(path);
                MercuryRtDebugTarget rtTarget = rt.CreateDebugTarget();

                int halts = 0;
                for (int f = 0; f < 900; f++)
                {
                    Press(f, plain, rt);
                    plain.RunFrame();
                    int? b = f % 3 == 0 ? rt.Breakpoints.AddBreakpoint(0x40) : null;
                    rt.RunFrame();
                    if (b is { } idb) rt.Breakpoints.RemoveBreakpoint(idb);
                    if (rt.IsHaltedAtBreakpoint)
                    {
                        halts++;
                        Assert.Equal(0x40, rt.HaltedAddress);
                        rt.RunFrame();
                    }
                    AssertSame(State(plain), State(rt), $"{Path.GetFileName(path)}, frame {f}");
                    Assert.True(plain.GetFrameBufferRgba().AsSpan().SequenceEqual(rt.GetFrameBufferRgba()), $"frame {f}: pictures differ");
                    Assert.True(plain.DequeueAudioSamples(int.MaxValue).AsSpan().SequenceEqual(rt.DequeueAudioSamples(int.MaxValue)), $"frame {f}: sound differs");
                }
                _output.WriteLine($"{Path.GetFileName(path)}: {halts} halts at $0040 in 900 frames, every halted state and every frame identical");
                Assert.True(halts > 0, "no halts");
            }
        }

        // The armed run: MercuryRT with every table armed against MercuryRT plain, every frame, and the registries shown to have recorded.
        private void Armed(string path, string name, int frames)
        {
            using var plain = new MercuryRtCore();
            plain.LoadRom(path);
            using var rt = new MercuryRtCore();
            rt.LoadRom(path);
            _ = rt.CreateDebugTarget();
            BreakpointRegistry bp = rt.Breakpoints;
            bp.AddBreakpoint(0xFEFF);
            bp.AddBreakpoint(0x40, 0x40, null, "vblank");
            bp.AddBreakpoint(0x48, 0x5F, null, null);
            bp.AddForbidRange(0x48, 0x5F);
            bp.AddDataBreakpoint("WRAM", 0, 0xFF, 0x100, onRead: false, changedOnly: false, condition: null);
            bp.AddDataBreakpoint("HRAM", 0, 0x7E, 0x100, onRead: false, changedOnly: false, condition: null);
            bp.ArmDepthGuard(512);
            rt.Coverage.Arm();
            rt.CallStack.ArmProfiler();
            int[] watches = { rt.Watches.AddWatch("WRAM", 0, 0x1000), rt.Watches.AddWatch("HRAM", 0, 0x7F), rt.Watches.AddWatch("OAM", 0, 0xA0), rt.Watches.AddWatch("VRAM", 0, 0x2000), rt.Watches.AddWatch("cartram", 0, 0x2000) };

            for (int f = 0; f < frames; f++)
            {
                Press(f, plain, rt);
                plain.RunFrame();
                rt.RunFrame();
                Assert.False(rt.IsHaltedAtBreakpoint, $"{name}, frame {f}: halted");
                AssertSame(State(plain), State(rt), $"{name}, frame {f}");
                Assert.True(plain.GetFrameBufferRgba().AsSpan().SequenceEqual(rt.GetFrameBufferRgba()), $"{name}, frame {f}: pictures differ");
                Assert.True(plain.DequeueAudioSamples(int.MaxValue).AsSpan().SequenceEqual(rt.DequeueAudioSamples(int.MaxValue)), $"{name}, frame {f}: sound differs");
            }

            Assert.True(rt.Coverage.InstructionsRecorded > 0, $"{name}: nothing recorded");
            Assert.Equal(rt.Coverage.InstructionsRecorded, rt.CallStack.ProfiledInstructions);
            long stored = watches.Sum(w => rt.Watches.GetEventCounts(w).Total);
            _output.WriteLine($"{name}: {frames} frames armed, identical to plain; {rt.Coverage.InstructionsRecorded} steps recorded and charged, {stored} stores watched, " +
                              $"{rt.Breakpoints.LogEntriesRecorded} logpoint passes, {rt.Coverage.EntryPointCount} entry points, depth {rt.CallStack.Depth}, {rt.CallStack.UnmatchedReturns} unmatched returns");
        }

        private static IEnumerable<string> Games()
        {
            string? folder = Environment.GetEnvironmentVariable(MercuryGoldenTests.GamesVariable);
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

        private static IEnumerable<(long, int, byte, string)> Events(WatchRegistry watches, int id) =>
            watches.GetEvents(id, 500).Select(e => (e.Sequence, e.Address, e.Value, e.Context));
    }
}
