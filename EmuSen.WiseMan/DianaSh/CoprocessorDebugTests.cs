using EmuSen.Cores;
using System.Linq;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.Crystal.Debugger;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // Named debug CPUs and the scope word every command takes - see `man cpus`.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoprocessorDebugTests
    {
        private static IDebugTarget BuildTarget()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return SyntheticRom.TargetOf(core);
        }

        [Fact]
        public void A_plain_cartridge_still_reports_its_two_built_in_processors()
        {
            var cpus = BuildTarget().DebugCpus;

            Assert.Equal(new[] { "cpu", "spc" }, cpus.Select(c => c.Name));
        }

        // The main CPU has to come first: it is what an unscoped command means.
        [Fact]
        public void The_main_cpu_is_first_and_reports_a_live_program_counter()
        {
            var target = BuildTarget();

            var main = target.DebugCpus[0];

            Assert.Equal(DebugCpus.MainName, main.Name);
            Assert.NotNull(main.ProgramCounter);
            Assert.Equal(0x008000, main.ProgramCounter!() & 0xFFFF00);
        }

        [Fact]
        public void Cpus_lists_each_chip_with_what_it_supports()
        {
            var shell = DianaShInterpreter.CreateDefault(BuildTarget());

            string output = shell.Submit("cpus").Output;

            Assert.Contains("cpu", output);
            Assert.Contains("spc", output);
            Assert.Contains("code space: APURAM", output);
            Assert.Contains("disasm", output);
        }

        [Fact]
        public void A_scope_word_selects_that_chips_breakpoint_registry()
        {
            var target = BuildTarget();
            var shell = DianaShInterpreter.CreateDefault(target);

            shell.Submit("bp spc add 05A5");

            Assert.Empty(target.Breakpoints.GetBreakpoints());
            Assert.Single(target.DebugCpus.First(c => c.Name == "spc").Breakpoints.GetBreakpoints());
        }

        // Every pre-existing unscoped form has to keep meaning the main CPU.
        [Fact]
        public void An_unscoped_command_still_targets_the_main_cpu()
        {
            var target = BuildTarget();

            DianaShInterpreter.CreateDefault(target).Submit("bp add 008000");

            Assert.Single(target.Breakpoints.GetBreakpoints());
        }

        [Fact]
        public void An_unknown_scope_word_is_not_swallowed_as_one()
        {
            var target = BuildTarget();
            var (cpu, next) = EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers
                .ResolveCpu(target, new[] { "bp", "add", "8000" }, 1);

            Assert.Equal(DebugCpus.MainName, cpu!.Name);
            Assert.Equal(1, next);
        }

        [Fact]
        public void The_cop_alias_still_resolves_to_the_cartridge_coprocessor()
        {
            var target = BuildTarget();

            // A blank cartridge carries none, so `cop` resolves to nothing at all.
            var (cpu, next) = EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers
                .ResolveCpu(target, new[] { "bp", "cop", "add" }, 1);

            Assert.Null(cpu);
            Assert.Equal(2, next);
        }

        [Fact]
        public void Regs_can_be_narrowed_to_one_chip()
        {
            var shell = DianaShInterpreter.CreateDefault(BuildTarget());

            string output = shell.Submit("regs spc").Output;

            Assert.StartsWith("SPC registers:", output);
            Assert.DoesNotContain("BGMODE", output);
        }

        [Fact]
        public void Regs_for_an_unknown_chip_lists_the_real_ones()
        {
            var shell = DianaShInterpreter.CreateDefault(BuildTarget());

            string output = shell.Submit("regs nope").Output;

            Assert.Contains("cpu", output);
            Assert.Contains("spc", output);
        }

        // The SPC700 is a real halt target now, not just a register dump.
        [Fact]
        public void A_breakpoint_on_the_sound_cpu_halts_the_core()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = SyntheticRom.TargetOf(core);
            var spc = target.DebugCpus.First(c => c.Name == "spc");
            spc.Breakpoints.AddBreakpoint(spc.ProgramCounter!());

            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.True(core is ICoprocessorHalt { IsHaltedOnCoprocessor: true });
        }

        // Resuming must arm the halted chip's own skip, not the main CPU's.
        [Fact]
        public void Resuming_a_sound_cpu_halt_does_not_instantly_re_halt()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = SyntheticRom.TargetOf(core);
            var spc = target.DebugCpus.First(c => c.Name == "spc");
            int id = spc.Breakpoints.AddBreakpoint(spc.ProgramCounter!());
            core.RunFrame();
            Assert.True(core.IsHaltedAtBreakpoint);
            spc.Breakpoints.RemoveBreakpoint(id);

            core.RunFrame();

            Assert.False(core.IsHaltedAtBreakpoint);
        }

        [Fact]
        public void Coverage_is_recorded_per_chip_not_shared()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = SyntheticRom.TargetOf(core);
            var spc = target.DebugCpus.First(c => c.Name == "spc");
            spc.Coverage!.Arm();

            core.RunFrame();

            Assert.True(spc.Coverage.InstructionsRecorded > 0);
            Assert.Equal(0, target.Coverage!.InstructionsRecorded);
        }

        // A bare chip name has to stay an expression, or `eval a` breaks.
        [Fact]
        public void A_lone_chip_name_is_still_treated_as_an_expression()
        {
            var shell = DianaShInterpreter.CreateDefault(BuildTarget());

            string output = shell.Submit("eval spc").Output;

            Assert.Contains("Unknown symbol", output);
        }

        [Fact]
        public void Disasm_by_chip_name_uses_that_chips_code_space_and_isa()
        {
            var shell = DianaShInterpreter.CreateDefault(BuildTarget());

            string bySpace = shell.Submit("disasm APURAM 0 4").Output;
            string byName = shell.Submit("disasm spc 0 4").Output;

            Assert.Equal(bySpace, byName);
        }

        [Fact]
        public void Disasm_with_no_address_starts_where_the_chip_is_executing()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = SyntheticRom.TargetOf(core);
            var shell = DianaShInterpreter.CreateDefault(target);

            string output = shell.Submit("disasm cpu").Output;

            Assert.Contains($"{target.DebugCpus.First(c => c.Name == "cpu").ProgramCounter!():X6}:", output);
        }

        [Fact]
        public void Bt_on_a_chip_with_no_call_stack_says_so_rather_than_inventing_one()
        {
            var shell = DianaShInterpreter.CreateDefault(BuildTarget());

            string output = shell.Submit("bt spc").Output;

            Assert.Contains("does not report a call stack", output);
        }

        [Fact]
        public void Step_over_on_a_chip_with_no_call_stack_is_refused()
        {
            var shell = DianaShInterpreter.CreateDefault(BuildTarget());

            string output = shell.Submit("step spc over").Output;

            Assert.Contains("call stack", output);
        }

        [Fact]
        public void Step_with_a_count_still_works_on_a_chip_without_a_call_stack()
        {
            var target = BuildTarget();

            var result = DianaShInterpreter.CreateDefault(target).Submit("step spc 3");

            Assert.Contains("Stepping 3", result.Output);
        }
    }
}
