using System;
using System.IO;
using System.Linq;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Debug;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;
using VideoInterface = EmuSen.Cores.Nintendo.Mars.Vi.Vi;

namespace EmuSen.WiseMan.Cores
{
    // The debugger's hooks on Mars, each driven by a few instructions at the boot entry - see Mars_Debug.md.
    public class MarsDebugTests : IDisposable
    {
        private const int Entry = unchecked((int)0xA400_0040);

        // lui a0,0xA010 / lw t0,0(a0) / addiu t0,t0,1 / sw t0,0(a0) / b the lw / nop
        private static readonly byte[] CountForever =
        {
            0x3C, 0x04, 0xA0, 0x10, 0x8C, 0x88, 0x00, 0x00, 0x25, 0x08, 0x00, 0x01,
            0xAC, 0x88, 0x00, 0x00, 0x10, 0x00, 0xFF, 0xFC, 0x00, 0x00, 0x00, 0x00,
        };

        // jal 0xA4000060 / nop / b self / nop / four nops / jr ra / nop
        private static readonly byte[] CallOnce =
        {
            0x0D, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x03, 0xE0, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
        };

        // jal 0xA4000060 / nop / b self / nop / four nops / lui t0,0xA400 / ori t0,t0,0x70 / jr t0 / nop / b self / nop
        private static readonly byte[] CallThenLeave =
        {
            0x0D, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x3C, 0x08, 0xA4, 0x00, 0x35, 0x08, 0x00, 0x70, 0x01, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00,
            0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
        };

        private readonly System.Collections.Generic.List<string> _roms = new();

        public void Dispose()
        {
            foreach (string rom in _roms) try { File.Delete(rom); } catch (IOException) { }
        }

        private (MarsCore Core, MarsDebugTarget Target) Load(byte[] program)
        {
            string rom = SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build(patches: (0, program)));
            _roms.Add(rom);

            var core = new MarsCore(batteryRamDisabled: true) { SkipRendering = true };
            core.LoadRom(rom);
            core.Bus!.Write32(MemoryMap.ViBase + VideoInterface.VerticalSync, 0x20);
            core.Bus.Write32(MemoryMap.ViBase + VideoInterface.HorizontalSync, 0x40);
            return (core, new MarsDebugTarget(core));
        }

        [Fact]
        public void A_breakpoint_stops_in_front_of_its_instruction_and_a_resume_runs_it()
        {
            var (core, _) = Load(CountForever);
            core.Breakpoints.AddBreakpoint(Entry + 12);

            core.RunFrame();
            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(Entry + 12, core.HaltedAddress);
            Assert.Equal(0u, core.Bus!.Read32(0x0010_0000));

            core.Breakpoints.RemoveBreakpoint(core.Breakpoints.GetBreakpoints()[0].Id);
            core.RunFrame();
            Assert.False(core.IsHaltedAtBreakpoint);
            Assert.True(core.Bus.Read32(0x0010_0000) > 0);
        }

        // From a halt, as the step command arms it: the halted instruction runs, then the count - see `man step`.
        [Fact]
        public void A_step_runs_exactly_the_instructions_it_asked_for()
        {
            var (core, _) = Load(CountForever);
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
            var (core, _) = Load(CallOnce);
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
            var (core, _) = Load(CallOnce);
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

        // A jr through any register but ra is a jump, not a return, so the call it left is still open.
        [Fact]
        public void A_jump_through_another_register_does_not_return()
        {
            var (core, _) = Load(CallThenLeave);

            core.RunFrame();

            Assert.Equal(1, core.CallStack.Depth);
        }

        [Fact]
        public void A_watch_survives_a_reload()
        {
            var (core, target) = Load(CountForever);
            int id = target.Watches.AddWatch("RDRAM", 0x0010_0000, 4);

            core.LoadRom(_roms[0]);
            core.Bus!.Write32(MemoryMap.ViBase + VideoInterface.VerticalSync, 0x20);
            core.Bus.Write32(MemoryMap.ViBase + VideoInterface.HorizontalSync, 0x40);
            core.RunFrame();

            Assert.NotEmpty(target.Watches.GetEvents(id));
        }

        [Fact]
        public void Run_to_frame_stops_at_the_start_of_the_frame_after_it()
        {
            var (core, _) = Load(CountForever);
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
            var (core, _) = Load(CountForever);
            core.Cpu!.Cop0[EmuSen.Cores.Nintendo.Mars.Cpu.Core.Cpu.StatusRegister] = 0x3400_0401;
            core.Bus!.Mi.Mask = MiInterrupt.VideoInterface;
            core.Bus.Mi.Raise(MiInterrupt.VideoInterface);
            core.Breakpoints.ArmRunToInterrupt(CallFrameKind.Irq);

            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(unchecked((int)0x8000_0180), core.HaltedAddress);
        }

        [Fact]
        public void Coverage_records_what_the_processor_ran_only_while_armed()
        {
            var (core, _) = Load(CountForever);
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
            var (core, _) = Load(CountForever);
            core.RunFrame();
            Assert.Equal(0, core.CallStack.ProfiledInstructions);

            core.CallStack.ArmProfiler();
            long start = core.Cpu!.Instructions;
            core.RunFrame();

            Assert.True(core.CallStack.ProfiledInstructions > 0);
            Assert.Equal(core.Cpu.Instructions - start, core.CallStack.ProfiledInstructions);
        }

        [Fact]
        public void The_rsp_records_its_own_coverage()
        {
            var (core, _) = Load(CountForever);
            core.RspCoverage.Arm();
            var rsp = core.Bus!.Sp.Processor;

            rsp.Start(0x010);
            rsp.Step();
            rsp.Step();

            Assert.True(core.RspCoverage.WasExecuted(0x010));
            Assert.True(core.RspCoverage.WasExecuted(0x014));
            Assert.False(core.Coverage.WasExecuted(0x010));
        }

        [Fact]
        public void A_watch_sees_a_store_in_the_space_it_landed_in()
        {
            var (core, target) = Load(CountForever);
            int id = target.Watches.AddWatch("RDRAM", 0x0010_0000, 4);

            core.RunFrame();

            var events = target.Watches.GetEvents(id);
            Assert.NotEmpty(events);
            Assert.All(events, e => Assert.InRange(e.Address, 0x0010_0000, 0x0010_0003));
            Assert.Contains("PC=A400004C", events[0].Context);
        }

        [Fact]
        public void A_data_breakpoint_halts_after_the_store_that_wrote_it()
        {
            var (core, target) = Load(CountForever);
            target.Breakpoints.AddDataBreakpoint("RDRAM", 0x0010_0003, 1);

            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(1u, core.Bus!.Read32(0x0010_0000));
            Assert.Equal(Entry + 16, core.HaltedAddress);
        }

        [Fact]
        public void The_frame_log_reads_big_endian_at_each_frame_end()
        {
            var (core, _) = Load(CountForever);
            int id = core.FrameLog.AddEntry("RDRAM", 0x0010_0000, 4);

            core.RunFrame();

            FrameLogSample sample = Assert.Single(core.FrameLog.GetSamples(id));
            Assert.Equal(core.Bus!.Read32(0x0010_0000), (uint)sample.Value);
        }

        // The processor's addresses, into memories only - a register can change by being read - see Mars_Debug.md §4.
        [Fact]
        public void The_cpu_space_reads_memories_through_the_segments_and_leaves_registers_alone()
        {
            var (core, target) = Load(CountForever);
            IDebugMemorySpace cpu = target.GetMemorySpaces().Single(s => s.Name == "CPU");
            core.Bus!.Rdram[0x0010_0002] = 0x5A;

            Assert.Equal(0x5A, cpu.Read(unchecked((int)0x8010_0002)));
            Assert.Equal(0x5A, cpu.Read(unchecked((int)0xA010_0002)));
            Assert.Equal(0x3C, cpu.Read(Entry));

            core.Bus.Sp.Write32(0x1C, 0);
            Assert.Equal(0, cpu.Read(unchecked((int)0xA404_001F)));
            Assert.Equal(0u, core.Bus.Sp.Read32(0x1C));

            cpu.Write(unchecked((int)0x8010_0010), 0x77);
            Assert.Equal(0x77, core.Bus.Rdram[0x0010_0010]);

            core.Bus.SpImem[0x10] = 0x99;
            Assert.Equal(0x99, cpu.Read(unchecked((int)0xA400_1010)));
        }

        [Fact]
        public void An_address_resolves_to_the_memory_it_names()
        {
            var (_, target) = Load(CountForever);

            Assert.Equal(("RDRAM", 0x0010_0000, true), Resolved(target, 0x8010_0000));
            Assert.Equal(("IMEM", 0x10, true), Resolved(target, 0xA400_1010));
            Assert.Equal(("ROM", 0x40, true), Resolved(target, 0xB000_0040));
            Assert.False(Resolved(target, 0xA404_0000).Addressable);
        }

        [Fact]
        public void Both_processors_are_listed_and_only_the_cpu_can_be_halted()
        {
            var (_, target) = Load(CountForever);

            Assert.Equal(new[] { "cpu", "rsp" }, target.DebugCpus.Select(c => c.Name));
            Assert.True(target.DebugCpus[0].CanHalt);
            Assert.False(target.DebugCpus[1].CanHalt);
            Assert.Equal(Entry, target.DebugCpus[0].ProgramCounter!());
        }

        // The phase's done-when, through the command itself - see Mars_Gameplan.md §4.6.
        [Fact]
        public void Disasm_cpu_lists_the_code_the_processor_is_about_to_run()
        {
            var (_, target) = Load(CountForever);

            string listing = new DisasmCommand().Execute(target, new[] { "disasm", "cpu", "A4000040", "6" }, null).Output;

            Assert.Contains("A4000040:", listing);
            Assert.Contains("lui", listing);
            Assert.Contains("sw", listing);
            Assert.Contains("0xA4000044", listing);
        }

        // RDRAM's code is seen where KSEG0 puts it, and IMEM's is the RSP's - see Mars_Debug.md §6.
        [Fact]
        public void Each_space_is_decoded_by_the_processor_that_runs_it_at_its_own_address()
        {
            var (core, target) = Load(CountForever);
            core.Bus!.Write32(0x100, 0x1000_0002);
            core.Bus.SpImem[0] = 0x4A;
            core.Bus.SpImem[3] = 0x10;

            DisassembledInstruction branch = target.Disassemble("RDRAM", 0x100, 1)[0];
            DisassembledInstruction vector = target.Disassemble("IMEM", 0, 1)[0];

            Assert.Contains("0x8000010C", branch.OperandText);
            Assert.Equal("vadd", vector.Mnemonic);
        }

        [Fact]
        public void A_jump_and_link_is_classified_as_a_call_to_its_target()
        {
            var (_, target) = Load(CallOnce);

            DisassembledInstruction jal = target.Disassemble("CPU", Entry, 1)[0];

            Assert.Equal((StaticReferenceKind.Call, Entry + 0x20), target.ClassifyStaticReference(jal));
        }

        [Fact]
        public void A_frontend_bundle_wires_the_target_to_the_core()
        {
            string rom = SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build(patches: (0, CountForever)));
            _roms.Add(rom);

            CoreBundle bundle = CoreFactory.Load(rom, headless: true);
            var core = (MarsCore)bundle.Core;

            Assert.Same(core.Breakpoints, bundle.DebugTarget.Breakpoints);
            Assert.Same(core.Coverage, bundle.DebugTarget.Coverage);
            Assert.Same(core.Watches, bundle.DebugTarget.Watches);
        }

        // The two frontends' own routes to a core, with the commands the plan names run on each - see Mars_Gameplan.md §4.6.
        [Fact]
        public void Both_frontends_routes_open_a_z64_and_disassemble_save_and_load_it()
        {
            string rom = SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build(patches: (0, CountForever)));
            _roms.Add(rom);
            string state = Path.Combine(Path.GetTempPath(), $"wiseman_{Guid.NewGuid():N}.state");
            _roms.Add(state);

            // Hotaru: Create with a window, LoadRom, Bundle, and its state command on the core's own save and load.
            ICore hotaru = CoreFactory.Create(rom, headless: false);
            hotaru.LoadRom(rom);
            IDebugTarget target = CoreFactory.Bundle(hotaru).DebugTarget;
            var stateCommand = new StateCommand(hotaru.SaveState, hotaru.LoadState, () => state);

            Assert.Contains("lui", new DisasmCommand().Execute(target, new[] { "disasm", "cpu" }, null).Output);
            Assert.Equal(0, stateCommand.Execute(target, new[] { "state", "save" }, null).ExitCode);
            hotaru.RunFrame();
            Assert.Equal(0, stateCommand.Execute(target, new[] { "state", "load" }, null).ExitCode);
            Assert.Equal(0, target.FrameCount);

            // Mistress: the session a window drives.
            var session = new EmulatorSession();
            session.LoadRom(rom);
            session.RunFrame();

            Assert.IsType<MarsCore>(session.Core);
            Assert.Contains("lui", new DisasmCommand().Execute(session.DebugTarget, new[] { "disasm", "cpu", "A4000040" }, null).Output);
        }

        private static (string Space, int Offset, bool Addressable) Resolved(MarsDebugTarget target, uint address)
        {
            PhysicalAddress where = target.ResolvePhysical(unchecked((int)address))!.Value;
            return (where.Space, where.Offset, where.IsAddressable);
        }
    }
}
