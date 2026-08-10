using System.Linq;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;
using static EmuSen.WiseMan.Fixtures.NecDspFirmwareBuilder;

namespace EmuSen.WiseMan.DianaOS
{
    // The NEC DSP as a first-class debug CPU, through the shell - see `man cpus`.
    [Collection(TestCollections.ProcessGlobals)]
    public class NecDspDebugTargetTests
    {
        // 0: CALL 3   1: LD A   2: LD A   3: RET
        private static byte[] Firmware() =>
            Blob(Dsp1(Jp(JumpCall, 3), Ld(0x0001, DestA), Ld(0x0002, DestA), Rt(Op())));

        private static (SnesDebugTarget Target, EmuSen.Cores.Nintendo.Venus.VenusCore Core) Build()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildNecDsp("PILOTWINGS", Firmware()));
            return (new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!), core);
        }

        private static DianaOSInterpreter Shell(SnesDebugTarget target) => DianaOSInterpreter.CreateDefault(target);

        [Fact]
        public void Cpus_now_reports_the_dsp_as_haltable_with_a_stack()
        {
            var (target, _) = Build();

            string output = Shell(target).Submit("cpus").Output;

            var dsp = target.DebugCpus.Single(c => c.Name == "dsp");
            Assert.True(dsp.CanHalt);
            Assert.NotNull(dsp.Coverage);
            Assert.NotNull(dsp.CallStack);
            Assert.Contains("dsp", output);
        }

        [Fact]
        public void Bp_dsp_add_is_accepted_rather_than_refused()
        {
            var (target, _) = Build();

            string output = Shell(target).Submit("bp dsp add 3").Output;

            Assert.DoesNotContain("cannot be halted", output);
            Assert.Contains("DSP breakpoint #1", output);
            var dsp = target.DebugCpus.Single(c => c.Name == "dsp");
            Assert.Equal(3, dsp.Breakpoints.GetBreakpoints().Single().Address);
        }

        [Fact]
        public void Step_dsp_is_accepted_rather_than_refused()
        {
            var (target, _) = Build();

            Assert.DoesNotContain("cannot be stepped", Shell(target).Submit("step dsp").Output);
        }

        [Fact]
        public void A_dsp_breakpoint_really_halts_the_frame_on_the_dsp()
        {
            var (target, core) = Build();
            Shell(target).Submit("bp dsp add 3"); // the RET, reached only through the CALL

            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal("dsp", core.HaltedCpu);
            Assert.Equal(3, core.HaltedAddress);
        }

        [Fact]
        public void Resuming_a_dsp_halt_leaves_the_breakpoint_it_is_sitting_on()
        {
            // 0: LD A   1: park on itself, so address 0 is reached exactly once.
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildNecDsp("PILOTWINGS",
                Blob(Dsp1(Ld(0x1234, DestA), Jp(JumpAlways, 1)))));
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            Shell(target).Submit("bp dsp add 0");

            core.RunFrame();
            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(0, core.HaltedAddress);

            core.RunFrame();

            // The LD ran on resume rather than the halt firing again on the same PC.
            Assert.False(core.IsHaltedAtBreakpoint);
            Assert.Equal(1, target.DebugCpus.Single(c => c.Name == "dsp").ProgramCounter!());
        }

        [Fact]
        public void The_dsp_call_stack_is_readable_with_bt()
        {
            var (target, core) = Build();
            Shell(target).Submit("bp dsp add 3");

            core.RunFrame();

            string output = Shell(target).Submit("bt dsp").Output;
            Assert.DoesNotContain("no call stack", output);
            var frames = target.DebugCpus.Single(c => c.Name == "dsp").CallStack!.Frames;
            Assert.Equal(3, Assert.Single(frames).Target);
        }

        [Fact]
        public void Cov_dsp_records_the_firmware_addresses_that_ran()
        {
            var (target, core) = Build();
            var shell = Shell(target);
            shell.Submit("cov dsp on");

            core.RunFrame();

            var coverage = target.DebugCpus.Single(c => c.Name == "dsp").Coverage!;
            Assert.True(coverage.WasExecuted(0)); // the CALL
            Assert.True(coverage.WasExecuted(3)); // its target, reached only through it
            // Well past the assembled program, so nothing ever gets there.
            Assert.False(coverage.WasExecuted(0x100));
        }

        [Fact]
        public void Cov_dsp_funcs_discovers_the_called_routine()
        {
            var (target, core) = Build();
            var shell = Shell(target);
            shell.Submit("cov dsp on");

            core.RunFrame();

            string output = shell.Submit("cov dsp funcs").Output;
            Assert.Contains("$000003", output);
        }
    }
}
