using System;
using System.IO;
using System.Linq;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The debugger's claims on an N64 engine, each driven by a few instructions at the boot entry; the C# core and MarsRT are held to the same transcripts - see Mars_Debug.md and Mars_Native.md §6.5.
    public abstract class MarsDebugClaims : IDisposable
    {
        protected const int Entry = unchecked((int)0xA400_0040);

        // lui a0,0xA010 / lw t0,0(a0) / addiu t0,t0,1 / sw t0,0(a0) / b the lw / nop
        protected static readonly byte[] CountForever =
        {
            0x3C, 0x04, 0xA0, 0x10, 0x8C, 0x88, 0x00, 0x00, 0x25, 0x08, 0x00, 0x01,
            0xAC, 0x88, 0x00, 0x00, 0x10, 0x00, 0xFF, 0xFC, 0x00, 0x00, 0x00, 0x00,
        };

        // jal 0xA4000060 / nop / b self / nop / four nops / jr ra / nop
        protected static readonly byte[] CallOnce =
        {
            0x0D, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x03, 0xE0, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
        };

        // jal 0xA4000060 / nop / b self / nop / four nops / lui t0,0xA400 / ori t0,t0,0x70 / jr t0 / nop / b self / nop
        protected static readonly byte[] CallThenLeave =
        {
            0x0D, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x3C, 0x08, 0xA4, 0x00, 0x35, 0x08, 0x00, 0x70, 0x01, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
        };

        private readonly System.Collections.Generic.List<string> _files = new();
        private readonly System.Collections.Generic.List<MarsDebugRig> _rigs = new();

        public virtual void Dispose()
        {
            foreach (MarsDebugRig rig in _rigs) rig.Dispose();
            foreach (string file in _files) try { File.Delete(file); } catch (IOException) { }
        }

        // The engine under the claims.
        protected abstract MarsDebugRig Rig(string rom);

        protected MarsDebugRig Load(byte[] program)
        {
            MarsDebugRig rig = Rig(Temporary(program));
            _rigs.Add(rig);
            return rig;
        }

        protected string Temporary(byte[] program)
        {
            string rom = SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build(patches: (0, program)));
            _files.Add(rom);
            return rom;
        }

        [Fact]
        public void A_breakpoint_stops_in_front_of_its_instruction_and_a_resume_runs_it()
        {
            MarsDebugRig core = Load(CountForever);
            core.Breakpoints.AddBreakpoint(Entry + 12);

            core.RunFrame();
            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(Entry + 12, core.HaltedAddress);
            Assert.Equal(0u, core.ReadRdram32(0x0010_0000));

            core.Breakpoints.RemoveBreakpoint(core.Breakpoints.GetBreakpoints()[0].Id);
            core.RunFrame();
            Assert.False(core.IsHaltedAtBreakpoint);
            Assert.True(core.ReadRdram32(0x0010_0000) > 0);
        }

        // From a halt, as the step command arms it: the halted instruction runs, then the count - see `man step`.
        [Fact]
        public void A_step_runs_exactly_the_instructions_it_asked_for()
        {
            MarsDebugRig core = Load(CountForever);
            core.Breakpoints.AddBreakpoint(Entry);
            core.RunFrame();
            core.Breakpoints.RemoveBreakpoint(core.Breakpoints.GetBreakpoints()[0].Id);

            core.Breakpoints.ArmStep(3);
            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(Entry + 12, core.HaltedAddress);
        }

        // A call pushes as it is made, a return pops once its delay slot has run - see Mars_Debug.md §2.
        [Fact]
        public void Stepping_over_a_call_stops_after_it_in_the_caller()
        {
            MarsDebugRig core = Load(CallOnce);
            core.Breakpoints.AddBreakpoint(Entry);
            core.RunFrame();
            core.Breakpoints.RemoveBreakpoint(core.Breakpoints.GetBreakpoints()[0].Id);

            core.Breakpoints.ArmStepToDepth(core.CallStack.Depth);
            core.RunFrame();

            Assert.Equal(Entry + 8, core.HaltedAddress);
            Assert.Equal(0, core.CallStack.Depth);
        }

        [Fact]
        public void Inside_a_call_the_backtrace_names_it_and_stepping_out_returns_to_the_caller()
        {
            MarsDebugRig core = Load(CallOnce);
            core.Breakpoints.AddBreakpoint(Entry + 0x20);
            core.RunFrame();

            Assert.Equal(Entry + 0x20, core.HaltedAddress);
            CallFrame frame = Assert.Single(core.CallStack.Backtrace());
            Assert.Equal(Entry, frame.Source);
            Assert.Equal(Entry + 0x20, frame.Target);

            core.Breakpoints.RemoveBreakpoint(core.Breakpoints.GetBreakpoints()[0].Id);
            core.Breakpoints.ArmStepToDepth(core.CallStack.Depth - 1);
            core.RunFrame();

            Assert.Equal(Entry + 8, core.HaltedAddress);
        }

        [Fact]
        public void A_watch_survives_a_reload()
        {
            MarsDebugRig core = Load(CountForever);
            int id = core.Target.Watches.AddWatch("RDRAM", 0x0010_0000, 4);

            core.Reload();
            core.RunFrame();

            Assert.NotEmpty(core.Target.Watches.GetEvents(id));
        }

        [Fact]
        public void Run_to_frame_stops_at_the_start_of_the_frame_after_it()
        {
            MarsDebugRig core = Load(CountForever);
            core.Breakpoints.ArmRunToFrame(2);

            core.RunFrame();
            core.RunFrame();
            Assert.False(core.IsHaltedAtBreakpoint);

            core.RunFrame();
            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(2, core.TotalFrames);
        }

        [Fact]
        public void Run_to_interrupt_stops_at_the_handler()
        {
            MarsDebugRig core = Load(CountForever);
            core.SetStatus(0x3400_0401);
            core.RaiseVideoInterrupt();
            core.Breakpoints.ArmRunToInterrupt(CallFrameKind.Irq);

            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(unchecked((int)0x8000_0180), core.HaltedAddress);
        }

        [Fact]
        public void Coverage_records_what_the_processor_ran_only_while_armed()
        {
            MarsDebugRig core = Load(CountForever);
            core.RunFrame();
            Assert.False(core.Coverage.WasExecuted(Entry + 4));

            core.Coverage.Arm();
            core.RunFrame();

            Assert.True(core.Coverage.WasExecuted(Entry + 4));
            Assert.True(core.Coverage.WasExecuted(Entry + 20));
            Assert.False(core.Coverage.WasExecuted(Entry + 24));
        }

        // Every instruction the frame ran is charged once the profiler is armed, and none before - see Mars_Performance.md §13.
        [Fact]
        public void The_profiler_counts_what_the_processor_ran_only_while_armed()
        {
            MarsDebugRig core = Load(CountForever);
            core.RunFrame();
            Assert.Equal(0, core.CallStack.ProfiledInstructions);

            core.CallStack.ArmProfiler();
            long start = core.Instructions;
            core.RunFrame();

            Assert.True(core.CallStack.ProfiledInstructions > 0);
            Assert.Equal(core.Instructions - start, core.CallStack.ProfiledInstructions);
        }

        // A halt is a frame's end for every counter: what ran up to it is charged and recorded before the halt is reported.
        [Fact]
        public void A_halt_leaves_the_coverage_and_the_profile_current()
        {
            MarsDebugRig core = Load(CountForever);
            core.Coverage.Arm();
            core.CallStack.ArmProfiler();
            core.Breakpoints.AddBreakpoint(Entry + 12);
            long start = core.Instructions;

            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(3, core.Instructions - start);
            Assert.Equal(3, core.CallStack.ProfiledInstructions);
            Assert.Equal(3, core.Coverage.InstructionsRecorded);
            Assert.True(core.Coverage.WasExecuted(Entry + 8));
            Assert.False(core.Coverage.WasExecuted(Entry + 12));
        }

        [Fact]
        public void The_rsp_records_its_own_coverage()
        {
            MarsDebugRig core = Load(CountForever);
            core.RspCoverage.Arm();

            core.StartRsp(0x010);
            core.StepRsp(1);
            core.StepRsp(1);

            Assert.True(core.RspCoverage.WasExecuted(0x010));
            Assert.True(core.RspCoverage.WasExecuted(0x014));
            Assert.False(core.Coverage.WasExecuted(0x010));
        }

        [Fact]
        public void A_watch_sees_a_store_in_the_space_it_landed_in()
        {
            MarsDebugRig core = Load(CountForever);
            int id = core.Target.Watches.AddWatch("RDRAM", 0x0010_0000, 4);

            core.RunFrame();

            var events = core.Target.Watches.GetEvents(id);
            Assert.NotEmpty(events);
            Assert.All(events, e => Assert.InRange(e.Address, 0x0010_0000, 0x0010_0003));
            Assert.Contains("PC=A400004C", events[0].Context);
        }

        // Nothing watched, no data breakpoint, no uninitialised-read check: the target says so, and says otherwise as soon as one exists.
        [Fact]
        public void The_target_listens_exactly_while_a_watch_or_a_data_breakpoint_exists()
        {
            MarsDebugRig core = Load(CountForever);
            Assert.False(core.Listening);

            int watch = core.Target.Watches.AddWatch("RDRAM", 0x0010_0000, 4);
            Assert.True(core.Listening);
            core.Target.Watches.RemoveWatch(watch);
            Assert.False(core.Listening);

            int breakpoint = core.Target.Breakpoints.AddDataBreakpoint("RDRAM", 0x0010_0000);
            Assert.True(core.Listening);
            core.Target.Breakpoints.RemoveBreakpoint(breakpoint);
            Assert.False(core.Listening);

            core.Target.Breakpoints.ArmUninitializedReadBreak("RDRAM", 0x100);
            Assert.True(core.Listening);
            core.Target.Breakpoints.DisarmUninitializedReadBreak();
            Assert.False(core.Listening);
        }

        [Fact]
        public void A_data_breakpoint_halts_after_the_store_that_wrote_it()
        {
            MarsDebugRig core = Load(CountForever);
            core.Target.Breakpoints.AddDataBreakpoint("RDRAM", 0x0010_0003, 1);

            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(1u, core.ReadRdram32(0x0010_0000));
            Assert.Equal(Entry + 16, core.HaltedAddress);
        }

        [Fact]
        public void The_frame_log_reads_big_endian_at_each_frame_end()
        {
            MarsDebugRig core = Load(CountForever);
            int id = core.FrameLog.AddEntry("RDRAM", 0x0010_0000, 4);

            core.RunFrame();

            FrameLogSample sample = Assert.Single(core.FrameLog.GetSamples(id));
            Assert.Equal(core.ReadRdram32(0x0010_0000), (uint)sample.Value);
        }

        // The processor's addresses, into memories only - a register can change by being read - see Mars_Debug.md §4.
        [Fact]
        public void The_cpu_space_reads_memories_through_the_segments_and_leaves_registers_alone()
        {
            MarsDebugRig core = Load(CountForever);
            IDebugMemorySpace cpu = core.Target.GetMemorySpaces().Single(s => s.Name == "CPU");
            core.WriteRdram8(0x0010_0002, 0x5A);

            Assert.Equal(0x5A, cpu.Read(unchecked((int)0x8010_0002)));
            Assert.Equal(0x5A, cpu.Read(unchecked((int)0xA010_0002)));
            Assert.Equal(0x3C, cpu.Read(Entry));

            core.WriteBus32(MemoryMap.SpRegistersBase + 0x1C, 0);
            Assert.Equal(0, cpu.Read(unchecked((int)0xA404_001F)));
            Assert.Equal(0u, core.ReadBus32(MemoryMap.SpRegistersBase + 0x1C));

            cpu.Write(unchecked((int)0x8010_0010), 0x77);
            Assert.Equal(0x77, core.ReadRdram8(0x0010_0010));

            core.WriteImem(0x10, 0x99);
            Assert.Equal(0x99, cpu.Read(unchecked((int)0xA400_1010)));
        }

        [Fact]
        public void An_address_resolves_to_the_memory_it_names()
        {
            MarsDebugRig core = Load(CountForever);

            Assert.Equal(("RDRAM", 0x0010_0000, true), Resolved(core, 0x8010_0000));
            Assert.Equal(("IMEM", 0x10, true), Resolved(core, 0xA400_1010));
            Assert.Equal(("ROM", 0x40, true), Resolved(core, 0xB000_0040));
            Assert.False(Resolved(core, 0xA404_0000).Addressable);
        }

        [Fact]
        public void Both_processors_are_listed_and_only_the_cpu_can_be_halted()
        {
            MarsDebugRig core = Load(CountForever);

            Assert.Equal(new[] { "cpu", "rsp" }, core.Target.DebugCpus.Select(c => c.Name));
            Assert.True(core.Target.DebugCpus[0].CanHalt);
            Assert.False(core.Target.DebugCpus[1].CanHalt);
            Assert.Equal(Entry, core.Target.DebugCpus[0].ProgramCounter!());
        }

        // The phase's done-when, through the command itself - see Mars_Gameplan.md §4.6.
        [Fact]
        public void Disasm_cpu_lists_the_code_the_processor_is_about_to_run()
        {
            MarsDebugRig core = Load(CountForever);

            string listing = new DisasmCommand().Execute(core.Target, new[] { "disasm", "cpu", "A4000040", "6" }, null).Output;

            Assert.Contains("A4000040:", listing);
            Assert.Contains("lui", listing);
            Assert.Contains("sw", listing);
            Assert.Contains("0xA4000044", listing);
        }

        // RDRAM's code is seen where KSEG0 puts it, and IMEM's is the RSP's - see Mars_Debug.md §6.
        [Fact]
        public void Each_space_is_decoded_by_the_processor_that_runs_it_at_its_own_address()
        {
            MarsDebugRig core = Load(CountForever);
            core.WriteBus32(0x100, 0x1000_0002);
            core.WriteImem(0, 0x4A);
            core.WriteImem(3, 0x10);

            DisassembledInstruction branch = core.Target.Disassemble("RDRAM", 0x100, 1)[0];
            DisassembledInstruction vector = core.Target.Disassemble("IMEM", 0, 1)[0];

            Assert.Contains("0x8000010C", branch.OperandText);
            Assert.Equal("vadd", vector.Mnemonic);
        }

        [Fact]
        public void A_jump_and_link_is_classified_as_a_call_to_its_target()
        {
            MarsDebugRig core = Load(CallOnce);

            DisassembledInstruction jal = core.Target.Disassemble("CPU", Entry, 1)[0];

            Assert.Equal((StaticReferenceKind.Call, Entry + 0x20), core.Target.ClassifyStaticReference(jal));
        }

        [Fact]
        public void A_frontend_bundle_wires_the_target_to_the_core()
        {
            MarsDebugRig rig = Load(CountForever);
            string rom = Temporary(CountForever);

            CoreBundle bundle = CoreFactory.Load(rom, headless: true, engine: rig.Engine);
            Assert.IsType(rig.CoreType, bundle.Core);
            var (breakpoints, coverage, watches) = rig.RegistriesOf(bundle.Core);

            Assert.Same(breakpoints, bundle.DebugTarget.Breakpoints);
            Assert.Same(coverage, bundle.DebugTarget.Coverage);
            Assert.Same(watches, bundle.DebugTarget.Watches);
            (bundle.Core as IDisposable)?.Dispose();
        }

        // The two frontends' own routes to a core, with the commands the plan names run on each - see Mars_Gameplan.md §4.6.
        [Fact]
        public void Both_frontends_routes_open_a_z64_and_disassemble_save_and_load_it()
        {
            MarsDebugRig rig = Load(CountForever);
            string rom = Temporary(CountForever);
            string state = Path.Combine(Path.GetTempPath(), $"wiseman_{Guid.NewGuid():N}.state");
            _files.Add(state);

            // Hotaru: Create with a window, LoadRom, Bundle, and its state command on the core's own save and load.
            ICore hotaru = CoreFactory.Create(rom, headless: false, engine: rig.Engine);
            Assert.IsType(rig.CoreType, hotaru);
            hotaru.LoadRom(rom);
            IDebugTarget target = CoreFactory.Bundle(hotaru).DebugTarget;
            var stateCommand = new StateCommand(hotaru.SaveState, hotaru.LoadState, () => state);

            Assert.Contains("lui", new DisasmCommand().Execute(target, new[] { "disasm", "cpu" }, null).Output);
            Assert.Equal(0, stateCommand.Execute(target, new[] { "state", "save" }, null).ExitCode);
            hotaru.RunFrame();
            Assert.Equal(0, stateCommand.Execute(target, new[] { "state", "load" }, null).ExitCode);
            Assert.Equal(0, target.FrameCount);
            (hotaru as IDisposable)?.Dispose();

            // Mistress: the session a window drives.
            var session = new EmulatorSession { Engine = rig.Engine };
            session.LoadRom(rom);
            session.RunFrame();

            Assert.IsType(rig.CoreType, session.Core);
            Assert.Contains("lui", new DisasmCommand().Execute(session.DebugTarget, new[] { "disasm", "cpu", "A4000040" }, null).Output);
            (session.Core as IDisposable)?.Dispose();
        }

        protected static (string Space, int Offset, bool Addressable) Resolved(MarsDebugRig core, uint address)
        {
            PhysicalAddress where = core.ResolvePhysical(unchecked((int)address))!.Value;
            return (where.Space, where.Offset, where.IsAddressable);
        }
    }

    // The C# core under the claims, with the two that are the C# core's own - see Mars_Debug.md.
    public class MarsDebugTests : MarsDebugClaims
    {
        protected override MarsDebugRig Rig(string rom) => MarsDebugRig.Mars(rom);

        // A jr through any register but ra is a jump, not a return, so the call it left is still open; the C# core tracks the stack on every frame.
        [Fact]
        public void A_jump_through_another_register_does_not_return()
        {
            MarsDebugRig core = Load(CallThenLeave);

            core.RunFrame();

            Assert.Equal(1, core.CallStack.Depth);
        }

        // The report costs about what the store does, so the bus asks the observer first - see Mars_Performance.md §16.
        [Fact]
        public void A_store_is_reported_only_while_the_observer_listens()
        {
            var bus = new MemoryBus();
            var observer = new CountingObserver();
            bus.WriteObserver = observer;

            bus.Store(0x1000, 0x1122_3344, 4);
            Assert.Equal(0, observer.Bytes);

            observer.Listening = true;
            bus.Store(0x1000, 0x1122_3344, 4);
            Assert.Equal(4, observer.Bytes);
        }

        private sealed class CountingObserver : IWriteObserver
        {
            public int Bytes;
            public bool Listening { get; set; }
            public void OnWrite(string spaceName, int address, byte value) => Bytes++;
        }
    }
}
