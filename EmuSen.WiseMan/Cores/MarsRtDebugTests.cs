using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MarsRT under the debugger's claims, held to the C# core's transcripts, and the proofs that the hooks change nothing the machine computes - see Mars_Native.md §6.5.
    public class MarsRtDebugTests : MarsDebugClaims
    {
        private readonly ITestOutputHelper _output;
        private readonly List<string> _temporary = new();

        public MarsRtDebugTests(ITestOutputHelper output)
        {
            _output = output;
            Assert.True(MarsRtCore.Available, MarsNative.Report);
        }

        protected override MarsDebugRig Rig(string rom) => MarsDebugRig.MarsRt(rom);

        // A jr through any register but ra is a jump, not a return; MarsRT tracks the stack only in an observed frame, so one is armed by a breakpoint nothing reaches.
        [Fact]
        public void A_jump_through_another_register_does_not_return_in_an_observed_frame()
        {
            MarsDebugRig core = Load(CallThenLeave);
            core.Breakpoints.AddBreakpoint(Entry + 0x100);

            core.RunFrame();

            Assert.False(core.IsHaltedAtBreakpoint);
            Assert.Equal(1, core.CallStack.Depth);
            Assert.Equal(0, core.CallStack.UnmatchedReturns);
        }

        // The plain frame keeps no stack: what the C# core's A_jump_through_another_register_does_not_return sees on every frame, MarsRT sees only while observed.
        [Fact]
        public void A_plain_frame_leaves_the_stack_where_the_last_observed_frame_left_it()
        {
            MarsDebugRig core = Load(CallThenLeave);

            core.RunFrame();

            Assert.Equal(0, core.CallStack.Depth);
        }

        // MarsRT's counterpart of the C# bus's rule: stores are logged only while a watch or a data breakpoint exists, and none is lost or invented.
        [Fact]
        public void A_store_is_reported_only_while_the_target_listens()
        {
            MarsDebugRig core = Load(CountForever);
            core.RunFrame();
            uint before = core.ReadRdram32(0x0010_0000);

            int watch = core.Watches.AddWatch("RDRAM", 0x0010_0000, 4);
            core.RunFrame();
            uint after = core.ReadRdram32(0x0010_0000);
            (long total, _) = core.Watches.GetEventCounts(watch);
            Assert.Equal((after - before) * 4, total);

            core.Watches.RemoveWatch(watch);
            core.RunFrame();
            Assert.Empty(core.Watches.GetEvents(watch));
        }

        // A state saved at a halt is the state of the unstopped run brought to the same instruction, and the resumed frame ends as the run's does.
        [Fact]
        public void A_state_saved_at_a_halt_is_the_unstopped_run_s_state_at_the_same_instruction()
        {
            MarsDebugRig halted = Load(CountForever);
            var through = (MarsRtCore)Load(CountForever).Core;
            halted.Breakpoints.AddBreakpoint(Entry + 12);
            var haltedCore = (MarsRtCore)halted.Core;

            for (int frame = 1; frame <= 3; frame++)
            {
                int halts = 0;
                while (true)
                {
                    halted.RunFrame();
                    if (!halted.IsHaltedAtBreakpoint) break;
                    halts++;
                    through.RunSteps((ulong)(Taken(haltedCore) - Taken(through)));
                    if (halts % 50 == 1) Assert.Equal(through.Save(false), haltedCore.Save(false));
                }
                Assert.True(halts > 10);
                through.RunFrame();
                Assert.Equal(through.TotalFrames, haltedCore.TotalFrames);
                Assert.Equal(through.Save(false), haltedCore.Save(false));
            }
        }

        private static long Taken(MarsRtCore core) => core.Instructions + core.DebugCounterValues()[2];

        // Every table armed and nothing that halts, against the same game run plain, frame by frame in state, picture and sound: unthreaded, then four workers deferred.
        [Theory]
        [InlineData("sm64.z64", "sm64.state")]
        [InlineData("oot.z64", "oot.state")]
        [InlineData("ge.z64", "ge-dam.state")]
        public void A_game_with_every_table_armed_is_the_game_run_plain(string romName, string stateName)
        {
            if (Game(romName, stateName) is not { } game) return;
            int frames = Frames(300);
            foreach (bool threaded in new[] { false, true })
            {
                using MarsRtCore plain = Production(threaded);
                using MarsRtCore armed = Production(threaded);
                foreach (MarsRtCore core in new[] { plain, armed })
                {
                    core.LoadRom(game.Rom);
                    core.LoadState(game.State);
                }
                // The page of RDRAM the game's state writes most, so the watch logs and the data breakpoint stops every frame; the registry says no to each stop, since no byte is 256.
                int page = romName switch { "sm64.z64" => 0x0020_1000, "oot.z64" => 0x0011_C000, _ => 0x003A_9000 };
                var target = new MarsRtDebugTarget(armed);
                armed.Breakpoints.AddBreakpoint(0);
                armed.Breakpoints.AddDataBreakpoint("RDRAM", page, page + 0xFF, value: 256, onRead: false, changedOnly: false, condition: null);
                armed.Breakpoints.AddForbidRange(0x0040_0000, 0x0040_0FFF);
                armed.Breakpoints.ArmRunToInterrupt(CallFrameKind.Nmi);
                armed.Breakpoints.ArmDepthGuard(100_000);
                int watch = armed.Watches.AddWatch("RDRAM", page, 0x1000);
                armed.Coverage.Arm();
                armed.RspCoverage.Arm();
                armed.CallStack.ArmProfiler();

                for (int frame = 1; frame <= frames; frame++)
                {
                    Drive(plain, frame);
                    Drive(armed, frame);
                    plain.RunFrame();
                    armed.RunFrame();
                    Assert.False(armed.IsHaltedAtBreakpoint, $"{romName}: frame {frame} halted at {armed.HaltedAddress:X8}: {armed.Breakpoints.LastBreakReason}");
                    Assert.True(plain.Save(false).AsSpan().SequenceEqual(armed.Save(false)), $"{romName}: the states part at frame {frame}");
                    Assert.True(plain.GetFrameBufferRgba().AsSpan().SequenceEqual(armed.GetFrameBufferRgba()), $"{romName}: the pictures part at frame {frame}");
                    Assert.True(plain.DequeueAudioSamples(1 << 20).AsSpan().SequenceEqual(armed.DequeueAudioSamples(1 << 20)), $"{romName}: the sound parts at frame {frame}");
                }

                Assert.True(armed.Coverage.InstructionsRecorded > 0 && armed.CallStack.ProfiledInstructions > 0);
                Assert.Equal(armed.CallStack.ProfiledInstructions, armed.Coverage.InstructionsRecorded);
                Assert.True(armed.Watches.GetEventCounts(watch).Total > 0, $"{romName}: the watch saw no store");
                Assert.True(armed.Breakpoints.GetDataBreakpoints()[0].HitCount == 0, $"{romName}: the data breakpoint counted a hit");
                _output.WriteLine($"{romName} {(threaded ? "four workers deferred" : "unthreaded")}: {frames} frames identical; {armed.Coverage.InstructionsRecorded} instructions recorded, {armed.RspCoverage.InstructionsRecorded} on the RSP, {armed.Watches.GetEventCounts(watch).Total} stores watched, depth {armed.CallStack.Depth}, {armed.CallStack.UnmatchedReturns} unmatched returns, {armed.Coverage.EntryPointCount} entry points; {string.Join(' ', target.DebugCpus.Select(c => c.Name))}");
            }
        }

        // A breakpoint at the general exception vector on every third frame, halted at and resumed through the registry, the other frames plain through the blocks:
        // against the same game run through, the state is one at the halts' instructions and state, picture and sound are one at every frame's end.
        [Theory]
        [InlineData("sm64.z64", "sm64.state")]
        [InlineData("oot.z64", "oot.state")]
        [InlineData("ge.z64", "ge-dam.state")]
        public void A_game_halted_at_breakpoints_and_resumed_is_the_game_run_through(string romName, string stateName)
        {
            if (Game(romName, stateName) is not { } game) return;
            int frames = Frames(300);
            const int Vector = unchecked((int)0x8000_0180);
            foreach (bool threaded in new[] { false, true })
            {
                using MarsRtCore through = Production(threaded);
                using MarsRtCore halted = Production(threaded);
                foreach (MarsRtCore core in new[] { through, halted })
                {
                    core.LoadRom(game.Rom);
                    core.LoadState(game.State);
                }
                long halts = 0, compared = 0;
                int? breakpoint = null;
                for (int frame = 1; frame <= frames; frame++)
                {
                    Drive(through, frame);
                    Drive(halted, frame);
                    bool armed = frame % 3 == 0;
                    if (armed && breakpoint is null) breakpoint = halted.Breakpoints.AddBreakpoint(Vector);
                    if (!armed && breakpoint is { } id) { halted.Breakpoints.RemoveBreakpoint(id); breakpoint = null; }

                    while (true)
                    {
                        halted.RunFrame();
                        if (!halted.IsHaltedAtBreakpoint) break;
                        halts++;
                        Assert.Equal(Vector, halted.HaltedAddress);
                        through.RunSteps((ulong)(Taken(halted) - Taken(through)));
                        if (frame % 10 != 0) continue;
                        Assert.True(through.Save(false).AsSpan().SequenceEqual(halted.Save(false)), $"{romName}: the states part at frame {frame}, halt {halts}");
                        compared++;
                    }
                    through.RunFrame();
                    Assert.Equal(through.TotalFrames, halted.TotalFrames);
                    Assert.True(through.Save(false).AsSpan().SequenceEqual(halted.Save(false)), $"{romName}: the states part at the end of frame {frame}");
                    Assert.True(through.GetFrameBufferRgba().AsSpan().SequenceEqual(halted.GetFrameBufferRgba()), $"{romName}: the pictures part at frame {frame}");
                    Assert.True(through.DequeueAudioSamples(1 << 20).AsSpan().SequenceEqual(halted.DequeueAudioSamples(1 << 20)), $"{romName}: the sound parts at frame {frame}");
                }
                Assert.True(halts >= frames / 3, $"{romName}: the vector was reached {halts} times in {frames} frames");
                _output.WriteLine($"{romName} {(threaded ? "four workers deferred" : "unthreaded")}: {frames} frames, {halts} halts, {compared} mid-frame states compared, all identical");
            }
        }

        // Production's configuration, as MarsRtThreadsTests.Bench builds it, or the same on one thread.
        private static MarsRtCore Production(bool threaded) => new(expansionPak: true, batteryRamDisabled: true)
        {
            SkipRendering = false,
            UseBlocks = true,
            ThreadedRdp = threaded,
            DeferredPresentation = threaded,
            RdpWorkers = threaded ? 4 : 1,
        };

        private static int Frames(int otherwise) => int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_MARSRT_FRAMES"), out int n) ? n : otherwise;

        // MarsRtTests.Drive: a button pattern and a stick sweep that change every frame.
        private static void Drive(MarsRtCore core, int frame)
        {
            EmuSen.Galaxia.Input.PadButton[] buttons = { EmuSen.Galaxia.Input.PadButton.A, EmuSen.Galaxia.Input.PadButton.B, EmuSen.Galaxia.Input.PadButton.Start, EmuSen.Galaxia.Input.PadButton.Up, EmuSen.Galaxia.Input.PadButton.L2, EmuSen.Galaxia.Input.PadButton.R };
            var button = buttons[frame % 6];
            bool pressed = (frame / 6) % 2 == 0;
            foreach (var b in buttons) core.SetButton(0, b, b == button && pressed);
            core.SetAxis(0, EmuSen.Galaxia.Input.PadAxis.LeftX, Math.Sin(frame * 0.37));
            core.SetAxis(0, EmuSen.Galaxia.Input.PadAxis.LeftY, Math.Cos(frame * 0.23));
            core.SetAxis(0, EmuSen.Galaxia.Input.PadAxis.RightX, (frame % 7 - 3) / 3.0);
            core.SetAxis(0, EmuSen.Galaxia.Input.PadAxis.RightY, -(frame % 7 - 3) / 3.0);
        }

        // The game's ROM copied to a scratch file, and its state; null when the folder does not hold them.
        private (string Rom, byte[] State)? Game(string romName, string stateName)
        {
            string? folder = Environment.GetEnvironmentVariable(MarsRtTests.StatesVariable);
            if (folder is null || !File.Exists(Path.Combine(folder, romName)) || !File.Exists(Path.Combine(folder, stateName)))
            {
                _output.WriteLine($"{romName} {stateName}: absent, not run");
                return null;
            }

            string rom = SyntheticN64Rom.WriteTemp(File.ReadAllBytes(Path.Combine(folder, romName)));
            _temporary.Add(rom);
            return (rom, File.ReadAllBytes(Path.Combine(folder, stateName)));
        }

        public override void Dispose()
        {
            base.Dispose();
            foreach (string path in _temporary)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }
    }
}
