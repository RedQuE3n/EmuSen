using System.Collections.Generic;
using EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp;
using EmuSen.DianaOS.DianaOS.Var;
using static EmuSen.WiseMan.Fixtures.NecDspFirmwareBuilder;

namespace EmuSen.WiseMan.Coprocessors
{
    // The DSP's per-instruction debug seam: breakpoints, budget, CALL/RET - see Venus_NecDSP.md §9.
    public class NecDspDebugSeamTests
    {
        private static NecDsp Chip(params uint[] program) =>
            new(NecDspVariant.Dsp1, Dsp1(program), hiRom: false);

        private static void Steps(NecDsp dsp, int count)
        {
            for (int i = 0; i < count; i++) dsp.Run(3);
        }

        [Fact]
        public void The_checker_sees_the_word_pc_before_each_instruction()
        {
            var dsp = Chip(Ld(0x1111, DestA), Ld(0x2222, DestB), Ld(0x3333, DestDr));
            var seen = new List<int>();
            dsp.BreakpointChecker = pc => { seen.Add(pc); return false; };

            Steps(dsp, 3);

            Assert.Equal(new[] { 0, 1, 2 }, seen);
        }

        [Fact]
        public void A_break_stops_before_the_instruction_runs()
        {
            var dsp = Chip(Ld(0x1234, DestDr));
            dsp.BreakpointChecker = pc => pc == 0;

            Steps(dsp, 1);

            Assert.True(dsp.HaltedAtBreakpoint);
            Assert.Equal(0, dsp.HaltedAddress);
            // Nothing ran, so DR never got its value and the host sees no request.
            Assert.Equal(0x00, dsp.ReadRegister(0x30C000));
        }

        [Fact]
        public void Resuming_skips_exactly_one_check_so_continue_leaves_the_breakpoint()
        {
            var dsp = Chip(Ld(0x1234, DestDr), Ld(0x5678, DestA));
            int halts = 0;
            dsp.BreakpointChecker = pc => { if (pc == 0) { halts++; return true; } return false; };

            Steps(dsp, 1);
            Assert.True(dsp.HaltedAtBreakpoint);

            dsp.ResumeFromBreakpoint();
            Steps(dsp, 2);

            Assert.False(dsp.HaltedAtBreakpoint);
            Assert.Equal(1, halts);
        }

        [Fact]
        public void A_halt_keeps_the_unspent_clock_budget()
        {
            var dsp = Chip(Ld(0x0001, DestA), Ld(0x0002, DestA), Ld(0x1234, DestDr));
            dsp.BreakpointChecker = pc => pc == 1;

            // One generous budget: the halt lands mid-way through spending it.
            dsp.Run(9);
            Assert.True(dsp.HaltedAtBreakpoint);

            dsp.ResumeFromBreakpoint();
            dsp.Run(0);

            // The carried budget alone retired the remaining two instructions.
            Assert.Equal(0x80, dsp.ReadRegister(0x30C000));
        }

        [Fact]
        public void Call_and_return_drive_a_real_call_stack()
        {
            var stack = new CallStackRegistry();
            // 0: CALL 3   1: LD A   2: (unreached)   3: RET
            var dsp = Chip(Jp(JumpCall, 3), Ld(0x0001, DestA), Ld(0x0002, DestA), Rt(Op()));
            dsp.CallObserver = (source, target) => stack.NotePush(source, target, CallFrameKind.Call);
            dsp.ReturnObserver = stack.NotePop;

            Steps(dsp, 1);
            var frame = Assert.Single(stack.Frames);
            Assert.Equal(0, frame.Source);
            Assert.Equal(3, frame.Target);

            Steps(dsp, 1);
            Assert.Empty(stack.Frames);
        }

        [Fact]
        public void A_discovered_call_target_reaches_the_coverage_registry()
        {
            var coverage = new CoverageRegistry();
            coverage.Arm();
            var stack = new CallStackRegistry { EntryPointObserver = coverage.RecordEntryPoint };
            var dsp = Chip(Jp(JumpCall, 3), Ld(0x0001, DestA), Ld(0x0002, DestA), Rt(Op()));
            dsp.CallObserver = (source, target) => stack.NotePush(source, target, CallFrameKind.Call);

            Steps(dsp, 1);

            var entry = Assert.Single(coverage.EntryPoints(10));
            Assert.Equal(3, entry.Address);
            Assert.Equal(CallFrameKind.Call, entry.Kind);
        }

        [Fact]
        public void With_no_checker_wired_the_chip_runs_exactly_as_before()
        {
            var dsp = Chip(Ld(0x1234, DestDr));

            Steps(dsp, 1);

            Assert.False(dsp.HaltedAtBreakpoint);
            Assert.Equal(0x80, dsp.ReadRegister(0x30C000));
        }
    }
}
