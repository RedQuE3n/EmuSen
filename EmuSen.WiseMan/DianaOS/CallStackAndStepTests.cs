using System.Linq;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // The live call stack and the step modes built on it - see `man bt` and `man step`.
    public class CallStackAndStepTests
    {
        // $008005 JSR $8010 / $008008 JMP $8008 / $008010 NOP NOP RTS.
        // The boot stub owns file offsets 0-4 - see SyntheticRom.Build.
        private const int CallSite = 0x008005;
        private const int AfterCall = 0x008008;
        private const int Subroutine = 0x008010;

        private static (VenusCore Core, SnesDebugTarget Target) BuildCallingRom()
        {
            byte[] rom = SyntheticRom.Build(
                (0x05, new byte[] { 0x20, 0x10, 0x80 }),          // JSR $8010
                (0x08, new byte[] { 0x4C, 0x08, 0x80 }),          // JMP $8008 (spin)
                (0x10, new byte[] { 0xEA, 0xEA, 0x60 }));         // NOP NOP RTS
            var core = SyntheticRom.LoadCore(rom);
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            return (core, target);
        }

        // Runs frames until the core halts, so a test never spins forever on
        // a breakpoint that turns out never to fire.
        private static bool RunUntilHalted(VenusCore core, int maxFrames = 8)
        {
            for (int i = 0; i < maxFrames; i++)
            {
                core.RunFrame();
                if (core.IsHaltedAtBreakpoint) return true;
            }
            return false;
        }

        [Fact]
        public void A_real_JSR_pushes_a_frame_and_RTS_pops_it()
        {
            var (core, target) = BuildCallingRom();
            target.Breakpoints.AddBreakpoint(Subroutine);

            Assert.True(RunUntilHalted(core));

            var frames = target.CallStack!.Backtrace();
            Assert.Single(frames);
            Assert.Equal(Subroutine, frames[0].Target);
            Assert.Equal(CallSite, frames[0].Source);
            Assert.Equal(CallFrameKind.Call, frames[0].Kind);
        }

        [Fact]
        public void Bt_reports_the_call_chain_with_its_source()
        {
            var (core, target) = BuildCallingRom();
            target.Breakpoints.AddBreakpoint(Subroutine);
            Assert.True(RunUntilHalted(core));

            string output = DianaOSInterpreter.CreateDefault(target).Submit("bt").Output;

            Assert.Contains("$008010", output);
            Assert.Contains("$008005", output);
            Assert.Contains("call", output);
        }

        [Fact]
        public void Bt_resolves_addresses_through_the_label_registry()
        {
            var (core, target) = BuildCallingRom();
            target.Breakpoints.AddBreakpoint(Subroutine);
            Assert.True(RunUntilHalted(core));
            var shell = DianaOSInterpreter.CreateDefault(target);
            shell.Submit("label add 008010 DoTheThing");

            Assert.Contains("<DoTheThing>", shell.Submit("bt").Output);
        }

        [Fact]
        public void Bt_reset_empties_the_chain()
        {
            var (core, target) = BuildCallingRom();
            target.Breakpoints.AddBreakpoint(Subroutine);
            Assert.True(RunUntilHalted(core));
            var shell = DianaOSInterpreter.CreateDefault(target);

            shell.Submit("bt reset");

            Assert.Contains("empty", shell.Submit("bt").Output);
        }

        // The whole point of `step out`: resume, run the rest of the
        // subroutine, and halt on the instruction after the call.
        [Fact]
        public void Step_out_halts_at_the_instruction_after_the_call()
        {
            var (core, target) = BuildCallingRom();
            int breakpointId = target.Breakpoints.AddBreakpoint(Subroutine);
            Assert.True(RunUntilHalted(core));
            target.Breakpoints.RemoveBreakpoint(breakpointId);

            DianaOSInterpreter.CreateDefault(target).Submit("step out");
            Assert.True(RunUntilHalted(core));

            Assert.Equal(AfterCall, core.HaltedAddress);
            Assert.Empty(target.CallStack!.Backtrace());
        }

        // Depth never rises on a spin loop, so this behaves like a plain step.
        [Fact]
        public void Step_over_a_non_call_halts_at_the_very_next_instruction()
        {
            var (core, target) = BuildCallingRom();
            int breakpointId = target.Breakpoints.AddBreakpoint(AfterCall);
            Assert.True(RunUntilHalted(core));
            target.Breakpoints.RemoveBreakpoint(breakpointId);

            DianaOSInterpreter.CreateDefault(target).Submit("step over");
            Assert.True(RunUntilHalted(core));

            Assert.Equal(AfterCall, core.HaltedAddress);
        }

        [Fact]
        public void Step_out_refuses_when_the_call_stack_is_already_empty()
        {
            var (_, target) = BuildCallingRom();

            var result = DianaOSInterpreter.CreateDefault(target).Submit("step out");

            Assert.Contains("outermost", result.Output);
        }

        [Fact]
        public void Step_with_a_count_halts_after_that_many_instructions()
        {
            var registry = new BreakpointRegistry();
            registry.ArmStep(3);

            Assert.False(registry.ShouldBreak(0x008000));
            Assert.False(registry.ShouldBreak(0x008001));
            Assert.True(registry.ShouldBreak(0x008002));
        }

        [Fact]
        public void An_armed_step_to_depth_fires_only_once_the_stack_unwinds()
        {
            var stack = new CallStackRegistry();
            var registry = new BreakpointRegistry { CallStack = stack };
            stack.NotePush(0x008005, 0x008010, CallFrameKind.Call);

            registry.ArmStepToDepth(0);
            Assert.False(registry.ShouldBreak(0x008010));

            stack.NotePop();
            Assert.True(registry.ShouldBreak(0x008008));
        }

        [Fact]
        public void An_unmatched_return_is_counted_rather_than_swallowed()
        {
            var stack = new CallStackRegistry();

            stack.NotePop();

            Assert.Equal(1, stack.UnmatchedReturns);
            Assert.Equal(0, stack.Depth);
        }

        [Fact]
        public void Profiling_charges_instructions_to_the_innermost_routine()
        {
            var stack = new CallStackRegistry();
            stack.ArmProfiler();

            stack.NoteInstruction();                                          // outside any call
            stack.NotePush(0x008005, 0x008010, CallFrameKind.Call);
            stack.NoteInstruction();
            stack.NoteInstruction();
            stack.NotePop();
            stack.NoteInstruction();

            var hottest = stack.Hottest(10);
            var routine = hottest.First(r => r.Address == 0x008010);
            Assert.Equal(2, routine.Instructions);
            Assert.Equal(1, routine.Calls);
            Assert.Equal(2, hottest.First(r => r.Address == 0).Instructions);
        }

        [Fact]
        public void Profile_top_reports_percentages_through_the_shell()
        {
            var (core, target) = BuildCallingRom();
            var shell = DianaOSInterpreter.CreateDefault(target);
            shell.Submit("profile on");
            core.RunFrame();

            string output = shell.Submit("profile top 5").Output;

            Assert.Contains("%", output);
            Assert.Contains("instruction(s) profiled", output);
        }
    }
}
