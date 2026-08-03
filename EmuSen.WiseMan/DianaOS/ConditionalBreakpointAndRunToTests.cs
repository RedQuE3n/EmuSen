using System.Linq;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // Conditional breakpoints and `runto` - see `man bp` and `man runto`.
    public class ConditionalBreakpointAndRunToTests
    {
        private static SnesDebugTarget BuildTarget()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        }

        private static BreakpointRegistry WithCondition(bool result)
            => new BreakpointRegistry { ConditionEvaluator = _ => (result, null) };

        [Fact]
        public void A_condition_that_holds_lets_the_breakpoint_fire()
        {
            var registry = WithCondition(true);
            registry.AddBreakpoint(0x008000, "a == 5");

            Assert.True(registry.ShouldBreak(0x008000));
        }

        [Fact]
        public void A_condition_that_fails_suppresses_the_halt()
        {
            var registry = WithCondition(false);
            registry.AddBreakpoint(0x008000, "a == 5");

            Assert.False(registry.ShouldBreak(0x008000));
        }

        // The address still has to match - a true condition is not a wildcard.
        [Fact]
        public void A_condition_does_not_widen_which_addresses_match()
        {
            var registry = WithCondition(true);
            registry.AddBreakpoint(0x008000, "1");

            Assert.False(registry.ShouldBreak(0x009000));
        }

        [Fact]
        public void A_suppressed_condition_does_not_count_as_a_hit()
        {
            var registry = WithCondition(false);
            registry.AddBreakpoint(0x008000, "a == 5");

            registry.ShouldBreak(0x008000);

            Assert.Equal(0, registry.GetBreakpoints()[0].HitCount);
        }

        // A typo must halt once with an explanation, not on every instruction
        // forever - see `man bp`.
        [Fact]
        public void A_condition_that_cannot_be_evaluated_is_dropped_after_one_halt()
        {
            var registry = new BreakpointRegistry { ConditionEvaluator = _ => (false, "Unknown symbol 'bogus'.") };
            registry.AddBreakpoint(0x008000, "bogus");

            Assert.True(registry.ShouldBreak(0x008000));
            Assert.Contains("bogus", registry.LastConditionError);
            Assert.Null(registry.GetBreakpoints()[0].Condition);
        }

        // A core with no evaluator wired must still honor the address itself.
        [Fact]
        public void Without_an_evaluator_a_condition_is_treated_as_always_true()
        {
            var registry = new BreakpointRegistry();
            registry.AddBreakpoint(0x008000, "a == 5");

            Assert.True(registry.ShouldBreak(0x008000));
        }

        [Fact]
        public void A_data_breakpoint_honors_its_condition_too()
        {
            var registry = WithCondition(false);
            registry.AddDataBreakpoint("WRAM", 0x40, -1, "a == 5");

            registry.NoteWrite("WRAM", 0x40, 0x01);

            Assert.False(registry.ShouldBreak(0x008000));
        }

        [Fact]
        public void Bp_add_if_records_the_condition_and_shows_it_in_the_listing()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);

            shell.Submit("bp add 008000 if a == 5");

            Assert.Contains("if a == 5", shell.Submit("bp list").Output);
        }

        [Fact]
        public void Bp_write_accepts_a_condition_after_its_value()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);

            shell.Submit("bp write WRAM 40 7F if a == 5");

            string listing = shell.Submit("bp list").Output;
            Assert.Contains("= 0x7F", listing);
            Assert.Contains("if a == 5", listing);
        }

        [Fact]
        public void Bp_write_accepts_a_condition_without_a_value()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);

            shell.Submit("bp write WRAM 40 if a == 5");

            string listing = shell.Submit("bp list").Output;
            Assert.DoesNotContain("= 0x", listing);
            Assert.Contains("if a == 5", listing);
        }

        // End to end: the condition reads the CPU's live A, not a snapshot.
        [Fact]
        public void A_real_conditional_breakpoint_evaluates_against_live_registers()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            target.Breakpoints.AddBreakpoint(0x008000, "a == $BEEF");

            core.Cpu!.A = 0x1234;
            Assert.False(target.Breakpoints.ShouldBreak(0x008000));

            core.Cpu.A = 0xBEEF;
            Assert.True(target.Breakpoints.ShouldBreak(0x008000));
        }

        // Reading WRAM in a condition is the common case and must work.
        [Fact]
        public void A_condition_can_read_memory()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            var wram = target.GetMemorySpaces().First(s => s.Name == "WRAM");
            target.Breakpoints.AddBreakpoint(0x008000, "[$7E0040] == $5A");

            wram.Write(0x40, 0x00);
            Assert.False(target.Breakpoints.ShouldBreak(0x008000));

            wram.Write(0x40, 0x5A);
            Assert.True(target.Breakpoints.ShouldBreak(0x008000));
        }

        [Fact]
        public void Bp_off_and_on_toggle_without_losing_the_breakpoint()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);
            shell.Submit("bp add 008000");

            shell.Submit("bp off 1");
            Assert.Contains("disabled", shell.Submit("bp list").Output);
            Assert.False(target.Breakpoints.ShouldBreak(0x008000));

            shell.Submit("bp on 1");
            Assert.True(target.Breakpoints.ShouldBreak(0x008000));
        }

        [Fact]
        public void Runto_nmi_halts_only_once_an_NMI_is_taken()
        {
            var registry = new BreakpointRegistry();
            registry.ArmRunToInterrupt(CallFrameKind.Nmi);

            Assert.False(registry.ShouldBreak(0x008000));

            registry.NoteInterrupt(CallFrameKind.Nmi);
            Assert.True(registry.ShouldBreak(0x008001));
        }

        [Fact]
        public void Runto_nmi_ignores_an_IRQ()
        {
            var registry = new BreakpointRegistry();
            registry.ArmRunToInterrupt(CallFrameKind.Nmi);

            registry.NoteInterrupt(CallFrameKind.Irq);

            Assert.False(registry.ShouldBreak(0x008000));
        }

        [Fact]
        public void Runto_scanline_halts_on_that_scanline_only()
        {
            var registry = new BreakpointRegistry();
            registry.ArmRunToScanline(100);

            registry.NoteScanline(99);
            Assert.False(registry.ShouldBreak(0x008000));

            registry.NoteScanline(100);
            Assert.True(registry.ShouldBreak(0x008001));
        }

        [Fact]
        public void Runto_frame_halts_once_the_frame_number_is_reached()
        {
            var registry = new BreakpointRegistry();
            registry.ArmRunToFrame(5);

            registry.NoteFrame(4);
            Assert.False(registry.ShouldBreak(0x008000));

            registry.NoteFrame(5);
            Assert.True(registry.ShouldBreak(0x008001));
        }

        // It arms once and is consumed once, the same contract `bp write` has.
        [Fact]
        public void A_runto_event_fires_only_once()
        {
            var registry = new BreakpointRegistry();
            registry.ArmRunToInterrupt(CallFrameKind.Nmi);
            registry.NoteInterrupt(CallFrameKind.Nmi);

            Assert.True(registry.ShouldBreak(0x008000));
            Assert.False(registry.ShouldBreak(0x008001));
        }

        [Fact]
        public void DisarmSteps_cancels_a_pending_runto_but_keeps_real_breakpoints()
        {
            var registry = new BreakpointRegistry();
            registry.AddBreakpoint(0x008000);
            registry.ArmRunToScanline(100);
            registry.NoteScanline(100);

            registry.DisarmSteps();

            Assert.True(registry.ShouldBreak(0x008000));
            Assert.False(registry.ShouldBreak(0x009000));
        }

        [Fact]
        public void Runto_on_a_plain_address_adds_a_breakpoint_and_says_so()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);

            string output = shell.Submit("runto 808000").Output;

            Assert.Contains("bp remove", output);
            Assert.Contains("$808000", shell.Submit("bp list").Output);
        }

        [Fact]
        public void Runto_rejects_an_unknown_target_with_a_suggestion()
        {
            var target = BuildTarget();

            var result = DianaOSInterpreter.CreateDefault(target).Submit("runto nmy");

            Assert.Contains("nmi", result.Output);
        }

        // A scanline notification must reach the registry from the real core.
        [Fact]
        public void The_core_reports_scanlines_to_the_breakpoint_registry()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            target.Breakpoints.ArmRunToScanline(3);

            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
        }
    }
}
