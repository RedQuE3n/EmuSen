using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The debugger's claims on a Game Boy engine, each driven by a few instructions at the entry point; Mercury and MercuryRT are held to the same transcripts - see Mercury_Native.md §8.5.
    public abstract class MercuryDebugClaims : IDisposable
    {
        protected const int Entry = SyntheticGbRom.EntryPoint;

        // ld hl,$C000 / inc (hl) / jr the inc
        protected static readonly (int, byte[])[] CountForever = { (0, new byte[] { 0x21, 0x00, 0xC0, 0x34, 0x18, 0xFD }) };

        // call $0160 / jr self; at $0160 four nops and ret
        protected static readonly (int, byte[])[] CallOnce =
        {
            (0, new byte[] { 0xCD, 0x60, 0x01, 0x18, 0xFE }),
            (0x10, new byte[] { 0x00, 0x00, 0x00, 0x00, 0xC9 }),
        };

        // call $0150, forever
        protected static readonly (int, byte[])[] Recurse = { (0, new byte[] { 0xCD, 0x50, 0x01 }) };

        // The vblank interrupt into a handler that counts in WRAM and HRAM through a call and leaves by reti; the main loop halts.
        protected static readonly (int, byte[])[] Interrupts =
        {
            (0, new byte[] { 0x31, 0xFE, 0xFF, 0x3E, 0x01, 0xE0, 0xFF, 0xFB, 0x76, 0x00, 0x18, 0xFC }),
            (0x40 - Entry, new byte[] { 0xC3, 0x00, 0x02 }),
            (0x200 - Entry, new byte[] { 0xF5, 0xE5, 0x21, 0x00, 0xC0, 0x34, 0xCD, 0x20, 0x02, 0xE1, 0xF1, 0xD9 }),
            (0x220 - Entry, new byte[] { 0xF0, 0x90, 0x3C, 0xE0, 0x90, 0xC9 }),
        };

        private readonly List<string> _files = new();
        private readonly List<MercuryDebugRig> _rigs = new();

        public virtual void Dispose()
        {
            foreach (MercuryDebugRig rig in _rigs) rig.Dispose();
            foreach (string file in _files) try { File.Delete(file); } catch (IOException) { }
        }

        // The engine under the claims.
        protected abstract MercuryDebugRig Rig(string rom);

        protected MercuryDebugRig Load(params (int, byte[])[] program)
        {
            MercuryDebugRig rig = Rig(Temporary(program));
            _rigs.Add(rig);
            return rig;
        }

        protected string Temporary((int, byte[])[] program)
        {
            string rom = SyntheticGbRom.WriteTemp(SyntheticGbRom.Build(patches: program));
            _files.Add(rom);
            return rom;
        }

        // Runs to the breakpoint at <address>, then removes it: the machine halted in front of it.
        protected static void HaltAt(MercuryDebugRig core, int address)
        {
            int id = core.Breakpoints.AddBreakpoint(address);
            for (int f = 0; f < 3 && !core.IsHaltedAtBreakpoint; f++) core.RunFrame();
            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(address, core.HaltedAddress);
            core.Breakpoints.RemoveBreakpoint(id);
        }

        [Fact]
        public void A_breakpoint_stops_in_front_of_its_instruction_and_a_resume_runs_it()
        {
            MercuryDebugRig core = Load(CountForever);
            core.Breakpoints.AddBreakpoint(Entry + 3);

            core.RunFrame();
            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(Entry + 3, core.HaltedAddress);
            Assert.Equal(Entry + 3, core.Pc);
            Assert.Equal(0, core.ReadSpace("WRAM", 0));
            Assert.Equal(0, core.TotalFrames);

            core.Breakpoints.RemoveBreakpoint(core.Breakpoints.GetBreakpoints()[0].Id);
            core.RunFrame();
            Assert.False(core.IsHaltedAtBreakpoint);
            Assert.Equal(1, core.TotalFrames);
            Assert.NotEqual(0, core.ReadSpace("WRAM", 0));
        }

        // From a halt, as the step command arms it: the halted instruction runs, then the count - see `man step`.
        [Fact]
        public void A_step_runs_exactly_the_instructions_it_asked_for()
        {
            MercuryDebugRig core = Load(CountForever);
            HaltAt(core, Entry);

            core.Breakpoints.ArmStep(3);
            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(Entry + 3, core.HaltedAddress);
            Assert.Equal(1, core.ReadSpace("WRAM", 0));
        }

        // Armed between frames, the step is asked before the first instruction and nothing runs.
        [Fact]
        public void A_step_armed_between_frames_stops_before_the_next_instruction()
        {
            MercuryDebugRig core = Load(CountForever);
            core.RunFrame();
            int pc = core.Pc;

            core.Breakpoints.ArmStep(1);
            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(pc, core.HaltedAddress);
            Assert.Equal(1, core.TotalFrames);
        }

        // The registry says no at every pass, so the frame runs on and each pass is counted once.
        [Fact]
        public void A_logpoint_counts_every_pass_and_never_halts()
        {
            MercuryDebugRig core = Load(CountForever);
            core.Breakpoints.AddBreakpoint(Entry + 3, Entry + 3, null, "tick");

            core.RunFrame();
            core.RunFrame();

            Assert.False(core.IsHaltedAtBreakpoint);
            Assert.Equal(2, core.TotalFrames);
            long hits = core.Breakpoints.GetBreakpoints()[0].HitCount;
            Assert.Equal(hits, core.Breakpoints.LogEntriesRecorded);
            Assert.InRange(hits, 5800, 5860);
        }

        [Fact]
        public void A_disabled_breakpoint_and_a_forbidden_range_do_not_halt()
        {
            MercuryDebugRig core = Load(CountForever);
            int id = core.Breakpoints.AddBreakpoint(Entry + 3);
            core.Breakpoints.SetEnabled(id, false);
            core.RunFrame();
            Assert.False(core.IsHaltedAtBreakpoint);

            core.Breakpoints.SetEnabled(id, true);
            core.Breakpoints.AddForbidRange(Entry, Entry + 5);
            core.RunFrame();
            Assert.False(core.IsHaltedAtBreakpoint);
            Assert.Equal(2, core.TotalFrames);
        }

        // Each byte a store leaves, in the space it landed in, with the storing instruction as its context - see Mercury_Debug.md §2.
        [Fact]
        public void A_watch_records_each_store_with_the_storing_instruction()
        {
            MercuryDebugRig core = Load(CountForever);
            int id = core.Watches.AddWatch("WRAM", 0, 1);

            core.RunFrame();

            IReadOnlyList<DebugWatchEvent> events = core.Watches.GetEvents(id, 500);
            for (int i = 1; i < events.Count; i++) Assert.Equal((byte)(events[i - 1].Value + 1), events[i].Value);
            Assert.All(events, e => Assert.Equal($"PC=${Entry + 3:X4}", e.Context));
            (long total, int retained) = core.Watches.GetEventCounts(id);
            Assert.Equal(500, retained);
            Assert.InRange(total, 2900, 2930);
            Assert.Equal(core.ReadSpace("WRAM", 0), events[^1].Value);
        }

        // The stack's pushes are stores too, reported in HRAM with the instruction that made them.
        [Fact]
        public void A_watch_sees_a_call_push_its_return_address()
        {
            MercuryDebugRig core = Load(CallOnce);
            int id = core.Watches.AddWatch("HRAM", 0x7C, 2);

            core.RunFrame();

            var events = core.Watches.GetEvents(id);
            Assert.Equal(new[] { (0x7D, (byte)0x01), (0x7C, (byte)0x53) }, events.Select(e => (e.Address, e.Value)));
            Assert.All(events, e => Assert.Equal($"PC=${Entry:X4}", e.Context));
        }

        // A cheat's poke and a debugger write go through the bus's decode, so a watch hears them as it hears the processor.
        [Fact]
        public void A_watch_hears_a_cheat_poke_and_a_debugger_write_to_the_bus()
        {
            MercuryDebugRig core = Load(CountForever);
            int poke = core.Watches.AddWatch("WRAM", 0x10, 1);
            int typed = core.Watches.AddWatch("WRAM", 0x11, 1);
            core.Cheats.AddRamPoke(MercuryCore.SpaceCpuBus, 0xC010, 0x5A, "a poke");

            core.RunFrame();
            core.Target.GetMemorySpaces().Single(s => s.Name == "CPUBUS").Write(0xC011, 0x33);
            core.Target.GetMemorySpaces().Single(s => s.Name == "WRAM").Write(0x11, 0x44);

            Assert.Equal(0x5A, Assert.Single(core.Watches.GetEvents(poke)).Value);
            DebugWatchEvent seen = Assert.Single(core.Watches.GetEvents(typed));
            Assert.Equal(0x33, seen.Value);
            Assert.Equal($"PC=${Entry + 4:X4}", seen.Context);
        }

        [Fact]
        public void A_watch_survives_a_reload()
        {
            MercuryDebugRig core = Load(CountForever);
            int id = core.Watches.AddWatch("WRAM", 0, 1);

            core.Reload();
            core.RunFrame();

            Assert.NotEmpty(core.Watches.GetEvents(id));
        }

        [Fact]
        public void Coverage_records_what_the_processor_ran_only_while_armed()
        {
            MercuryDebugRig core = Load(CountForever);
            core.RunFrame();
            Assert.False(core.Coverage.WasExecuted(Entry + 3));

            core.Coverage.Arm();
            core.RunFrame();

            Assert.True(core.Coverage.WasExecuted(Entry + 3));
            Assert.True(core.Coverage.WasExecuted(Entry + 4));
            Assert.False(core.Coverage.WasExecuted(Entry));
            Assert.False(core.Coverage.WasExecuted(Entry + 5));
            Assert.InRange(core.Coverage.InstructionsRecorded, 5800, 5860);

            core.Coverage.Disarm();
            core.Coverage.Clear();
            core.RunFrame();
            Assert.False(core.Coverage.WasExecuted(Entry + 3));
        }

        // Disarmed in a frame something else observes, the recorder keeps nothing to hand over when it is armed again.
        [Fact]
        public void Coverage_disarmed_records_nothing_even_in_a_frame_that_is_observed()
        {
            MercuryDebugRig core = Load(CallOnce);
            core.Coverage.Arm();
            HaltAt(core, Entry);
            core.Coverage.Disarm();
            core.Coverage.Clear();
            core.Breakpoints.AddBreakpoint(0x7000);

            core.RunFrame();
            core.Coverage.Arm();
            core.RunFrame();

            Assert.True(core.Coverage.WasExecuted(Entry + 3));
            Assert.False(core.Coverage.WasExecuted(0x160));
            Assert.False(core.Coverage.WasExecuted(Entry));
        }

        // A halt is a frame's end for every counter: what ran before it is recorded when it is reported.
        [Fact]
        public void A_halt_leaves_the_coverage_and_the_profile_current()
        {
            MercuryDebugRig core = Load(CallOnce);
            core.Coverage.Arm();
            core.CallStack!.ArmProfiler();
            core.Breakpoints.AddBreakpoint(Entry + 3);

            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.True(core.Coverage.WasExecuted(0x160));
            Assert.False(core.Coverage.WasExecuted(Entry + 3));
            Assert.Equal(8, core.Coverage.InstructionsRecorded);
            Assert.Equal(8, core.CallStack.ProfiledInstructions);
            Assert.Equal(5, core.CallStack.Hottest(5).Single(h => h.Address == 0x160).Instructions);
        }

        // A write lands mid-instruction, so the halt waits for the boundary after it - see `man bp`.
        [Fact]
        public void A_data_breakpoint_halts_after_the_store_that_wrote_it()
        {
            MercuryDebugRig core = Load(CountForever);
            core.Breakpoints.AddDataBreakpoint("WRAM", 0, 3);

            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(Entry + 4, core.HaltedAddress);
            Assert.Equal(3, core.ReadSpace("WRAM", 0));
            Assert.Contains("data breakpoint", core.Breakpoints.LastBreakReason);
        }

        [Fact]
        public void A_data_breakpoint_whose_value_is_never_written_never_halts()
        {
            MercuryDebugRig core = Load(CallOnce);
            core.Breakpoints.AddDataBreakpoint("HRAM", 0x7C, 0x7D, 0x42, onRead: false, changedOnly: false, condition: null);

            core.RunFrame();

            Assert.False(core.IsHaltedAtBreakpoint);
            Assert.Equal(1, core.TotalFrames);
            Assert.Equal(0, core.Breakpoints.GetDataBreakpoints()[0].HitCount);
        }

        // A call is a frame on the stack from the moment it is made, and its return pops it - see Mercury_Debug.md §7.
        [Fact]
        public void Stepping_over_a_call_stops_after_it_in_the_caller()
        {
            MercuryDebugRig core = Load(CallOnce);
            HaltAt(core, Entry);

            Assert.True(core.Breakpoints.ArmStepToDepth(core.CallStack!.Depth));
            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(Entry + 3, core.HaltedAddress);
            Assert.Equal(0, core.CallStack.Depth);
        }

        [Fact]
        public void Inside_a_call_the_backtrace_names_it_and_stepping_out_returns_to_the_caller()
        {
            MercuryDebugRig core = Load(CallOnce);
            HaltAt(core, 0x162);

            CallFrame frame = Assert.Single(core.CallStack!.Backtrace());
            Assert.Equal((Entry, 0x160, CallFrameKind.Call), (frame.Source, frame.Target, frame.Kind));

            core.Breakpoints.ArmStepToDepth(core.CallStack.Depth - 1);
            core.RunFrame();

            Assert.Equal(Entry + 3, core.HaltedAddress);
            Assert.Equal(0, core.CallStack.UnmatchedReturns);
        }

        // An interrupt pushes a return address as a call does, so it is a frame, and reti pops it.
        [Fact]
        public void An_interrupt_is_a_frame_until_its_reti()
        {
            MercuryDebugRig core = Load(Interrupts);
            HaltAt(core, 0x222);

            Assert.Equal(new[] { (0x220, CallFrameKind.Call, 0x206), (0x40, CallFrameKind.Irq, Entry + 9) },
                core.CallStack!.Backtrace().Select(f => (f.Target, f.Kind, f.Source)));

            core.Breakpoints.ArmStepToDepth(0);
            core.RunFrame();

            Assert.Equal(Entry + 9, core.HaltedAddress);
            Assert.Equal(0, core.CallStack.UnmatchedReturns);
        }

        [Fact]
        public void The_depth_guard_halts_a_call_past_it()
        {
            MercuryDebugRig core = Load(Recurse);
            Assert.True(core.Breakpoints.ArmDepthGuard(3));

            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(4, core.CallStack!.Depth);
            Assert.Contains("exceeded the guard", core.Breakpoints.LastBreakReason);
        }

        [Fact]
        public void Run_to_interrupt_stops_at_the_vector()
        {
            MercuryDebugRig core = Load(Interrupts);
            core.Breakpoints.ArmRunToInterrupt(CallFrameKind.Irq);

            core.RunFrame();
            if (!core.IsHaltedAtBreakpoint) core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(0x40, core.HaltedAddress);
            Assert.Equal("IRQ taken", core.Breakpoints.LastBreakReason);
        }

        [Fact]
        public void Run_to_frame_stops_at_the_start_of_the_frame_after_it()
        {
            MercuryDebugRig core = Load(CountForever);
            core.Breakpoints.ArmRunToFrame(2);

            core.RunFrame();
            core.RunFrame();
            Assert.False(core.IsHaltedAtBreakpoint);

            int pc = core.Pc;
            core.RunFrame();
            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(pc, core.HaltedAddress);
            Assert.Equal(2, core.TotalFrames);
        }

        // Every step the frame ran is charged once the profiler is armed, and none before - see `man profile`.
        [Fact]
        public void The_profiler_counts_what_the_processor_ran_only_while_armed()
        {
            MercuryDebugRig core = Load(Interrupts);
            core.RunFrame();
            Assert.Equal(0, core.CallStack!.ProfiledInstructions);
            int before = core.ReadSpace("HRAM", 0x10);

            core.CallStack.ArmProfiler();
            core.Coverage.Arm();
            for (int i = 0; i < 3; i++) core.RunFrame();

            Assert.Equal(core.Coverage.InstructionsRecorded, core.CallStack.ProfiledInstructions);
            var hottest = core.CallStack.Hottest(10).ToDictionary(h => h.Address);
            int calls = core.ReadSpace("HRAM", 0x10) - before;
            Assert.InRange(calls, 2, 3);
            Assert.Equal(4L * calls, hottest[0x220].Instructions);
            Assert.Equal(9L * calls, hottest[0x40].Instructions);
            Assert.Equal(calls, hottest[0x220].Calls);
        }

        // A routine found by watching calls land, for `cov funcs` - see `man cov`.
        [Fact]
        public void Coverage_discovers_the_routines_calls_land_in()
        {
            MercuryDebugRig core = Load(Interrupts);
            core.Coverage.Arm();
            for (int i = 0; i < 3; i++) core.RunFrame();

            var found = core.Coverage.EntryPoints(10).ToDictionary(e => e.Address);
            Assert.Equal(CallFrameKind.Irq, found[0x40].Kind);
            Assert.Equal(CallFrameKind.Call, found[0x220].Kind);
            Assert.Equal(found[0x40].Entries, found[0x220].Entries);
        }

        [Fact]
        public void An_illegal_opcode_in_an_observed_frame_throws_the_cores_exception()
        {
            MercuryDebugRig core = Load((0, new byte[] { 0x00, 0xD3 }));
            core.Coverage.Arm();
            core.Watches.AddWatch("WRAM", 0, 16);

            var thrown = Assert.Throws<NotSupportedException>(() => core.RunFrame());
            Assert.Equal($"Opcode $D3 at ${Entry + 1:X4} is not a real SM83 instruction - see Mercury_Cpu.md §6.1.", thrown.Message);
        }

        // Halted at the vblank vector on every third frame and resumed, against the program run plain on the C# core.
        [Fact]
        public void A_frame_halted_and_resumed_is_the_frame_run_through()
        {
            string rom = Temporary(Interrupts);
            MercuryDebugRig core = Rig(rom);
            _rigs.Add(core);
            var plain = new MercuryCore();
            plain.LoadRom(rom);

            int halts = 0;
            for (int f = 0; f < 120; f++)
            {
                plain.RunFrame();
                int? id = f % 3 == 0 ? core.Breakpoints.AddBreakpoint(0x40) : null;
                core.RunFrame();
                if (id is { } bp)
                {
                    core.Breakpoints.RemoveBreakpoint(bp);
                    if (core.IsHaltedAtBreakpoint)
                    {
                        halts++;
                        core.RunFrame();
                    }
                }
                using var expected = new MemoryStream();
                plain.SaveState(expected);
                Assert.True(expected.ToArray().AsSpan().SequenceEqual(core.State()), $"frame {f}");
            }
            Assert.InRange(halts, 39, 40);
        }

        [Fact]
        public void A_frontend_bundle_wires_the_target_to_the_core()
        {
            string rom = Temporary(CountForever);
            MercuryDebugRig rig = Load(CountForever);
            CoreBundle bundle = CoreFactory.Load(rom, engine: rig.Engine);
            Assert.IsType(rig.CoreType, bundle.Core);
            Assert.IsType(rig.TargetType, bundle.DebugTarget);

            IDebugTarget target = bundle.DebugTarget;
            DebugCpu cpu = Assert.Single(target.DebugCpus);
            Assert.True(cpu.CanHalt);
            Assert.Same(target.Breakpoints, cpu.Breakpoints);
            Assert.Same(target.CallStack, cpu.CallStack);
            Assert.Same(target.Breakpoints.CallStack, target.CallStack);

            target.Breakpoints.AddBreakpoint(Entry + 3);
            bundle.Core.RunFrame();
            Assert.True(bundle.Core.IsHaltedAtBreakpoint);
            Assert.Equal(Entry + 3, cpu.ProgramCounter!());
            (bundle.Core as IDisposable)?.Dispose();
        }

        // The console's own commands drive the halt, the step and the step over - see `man step`.
        [Fact]
        public void The_consoles_bp_step_and_bt_commands_drive_the_engine()
        {
            MercuryDebugRig core = Load(CallOnce);
            IDebugTarget target = core.Target;

            Assert.Contains("added", new BreakCommand().Execute(target, new[] { "bp", "add", "162" }, null).Output);
            core.RunFrame();
            Assert.Equal(0x162, core.HaltedAddress);
            Assert.Contains("$000160", new BtCommand().Execute(target, new[] { "bt" }, null).Output);

            target.Breakpoints.RemoveBreakpoint(target.Breakpoints.GetBreakpoints()[0].Id);
            Assert.Equal(0, new StepCommand().Execute(target, new[] { "step" }, null).ExitCode);
            core.RunFrame();
            Assert.Equal(0x163, core.HaltedAddress);

            Assert.Equal(0, new StepCommand().Execute(target, new[] { "step", "out" }, null).ExitCode);
            core.RunFrame();
            Assert.Equal(Entry + 3, core.HaltedAddress);

            Assert.Equal(0, new StepCommand().Execute(target, new[] { "step", "over" }, null).ExitCode);
            core.RunFrame();
            Assert.Equal(Entry + 3, core.HaltedAddress);
        }
    }

    // The C# Mercury under the claims, and what only it does.
    public class MercuryDebugTests : MercuryDebugClaims
    {
        protected override MercuryDebugRig Rig(string rom) => MercuryDebugRig.Mercury(rom);

        // The C# core tracks the stack on every frame, armed or not; MercuryRT only in an observed one - see Mercury_Native.md §8.5.
        [Fact]
        public void A_call_made_in_a_frame_nothing_observes_is_on_the_stack()
        {
            MercuryDebugRig core = Load((0, new byte[] { 0xCD, 0x60, 0x01 }), (0x10, new byte[] { 0x18, 0xFE }));

            core.RunFrame();

            Assert.Equal(1, core.CallStack!.Depth);
        }
    }
}
