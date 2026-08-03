using System.Linq;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // The command surface of the range/read/uninit/depth/forbid and cov-map work - see `man bp`, `man cov`.
    public class BreakAndCovCommandTests
    {
        private static SnesDebugTarget BuildTarget()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        }

        private static DianaOSInterpreter Shell(SnesDebugTarget target) => DianaOSInterpreter.CreateDefault(target);

        [Fact]
        public void Bp_add_accepts_a_range_and_reports_it()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("bp add 818000-818FFF").Output;

            Assert.Contains("$818000-$818FFF", output);
            Assert.Equal(0x818FFF, target.Breakpoints.GetBreakpoints().Single().EndAddress);
        }

        [Fact]
        public void Bp_read_registers_a_read_breakpoint()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("bp read WRAM 13c6").Output;

            Assert.Contains("reads of WRAM", output);
            Assert.True(target.Breakpoints.GetDataBreakpoints().Single().OnRead);
        }

        [Fact]
        public void Bp_write_takes_a_changed_flag()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("bp write WRAM 0db3 changed").Output;

            Assert.Contains("only when it changes", output);
            Assert.True(target.Breakpoints.GetDataBreakpoints().Single().ChangedOnly);
        }

        [Fact]
        public void Bp_write_takes_a_value_and_a_changed_flag_together()
        {
            var target = BuildTarget();

            Shell(target).Submit("bp write WRAM 0db3 7f changed");

            var listed = target.Breakpoints.GetDataBreakpoints().Single();
            Assert.Equal(0x7F, listed.Value);
            Assert.True(listed.ChangedOnly);
        }

        [Fact]
        public void Changed_is_refused_on_a_read_breakpoint()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("bp read WRAM 0db3 changed").Output;

            Assert.Contains("only applies to writes", output);
            Assert.Empty(target.Breakpoints.GetDataBreakpoints());
        }

        [Fact]
        public void Bp_uninit_arms_against_a_named_space()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("bp uninit WRAM").Output;

            Assert.Contains("never-written", output);
            Assert.True(target.Breakpoints.IsUninitializedReadBreakArmed);
            Assert.Equal("WRAM", target.Breakpoints.UninitializedReadSpace);
        }

        [Fact]
        public void Bp_depth_arms_and_disarms_the_guard()
        {
            var target = BuildTarget();
            var shell = Shell(target);

            shell.Submit("bp depth 40");
            Assert.Equal(0x40, target.Breakpoints.DepthGuard);

            shell.Submit("bp depth off");
            Assert.Equal(-1, target.Breakpoints.DepthGuard);
        }

        [Fact]
        public void Bp_forbid_registers_a_suppression_range()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("bp forbid 0080C4-0080FF").Output;

            Assert.Contains("nothing halts", output);
            Assert.Single(target.Breakpoints.GetForbidRanges());
        }

        [Fact]
        public void Bp_list_shows_ranges_forbids_and_the_armed_extras()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            shell.Submit("bp add 818000-818FFF");
            shell.Submit("bp read WRAM 0100-01FF");
            shell.Submit("bp forbid 0080C4-0080FF");
            shell.Submit("bp depth 40");
            shell.Submit("bp uninit WRAM");

            string listing = shell.Submit("bp list").Output;

            Assert.Contains("$818000-$818FFF", listing);
            Assert.Contains("read WRAM 0x100-0x1FF", listing);
            Assert.Contains("forbid $0080C4-$0080FF", listing);
            Assert.Contains("depth guard", listing);
            Assert.Contains("uninitialized reads: WRAM", listing);
        }

        [Fact]
        public void A_data_breakpoint_address_still_prints_as_a_space_offset()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            shell.Submit("bp write WRAM 40 7F");

            Assert.Contains("write WRAM 0x40 = 0x7F", shell.Submit("bp list").Output);
        }

        [Fact]
        public void An_unknown_bp_subcommand_lists_the_new_ones()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("bp wibble").Output;

            Assert.Contains("read", output);
            Assert.Contains("uninit", output);
            Assert.Contains("forbid", output);
        }

        [Fact]
        public void A_logpoint_renders_real_expressions_as_name_equals_value()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            shell.Submit("setreg a 1234");
            shell.Submit("setreg x 0056");
            shell.Submit("bp add 808000 log \"a, x\"");

            Assert.False(target.Breakpoints.ShouldBreak(0x808000));

            string logged = shell.Submit("bp log").Output;
            Assert.Contains("a=0x1234", logged);
            Assert.Contains("x=0x56", logged);
        }

        [Fact]
        public void A_logpoint_expression_can_read_memory()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            var wram = target.GetMemorySpaces().Single(s => s.Name == "WRAM");
            wram.Write(0x0DB3, 0x7F);
            shell.Submit("bp add 808000 log [$7E0DB3]");

            target.Breakpoints.ShouldBreak(0x808000);

            Assert.Contains("0x7F", shell.Submit("bp log").Output);
        }

        [Fact]
        public void A_bad_logpoint_expression_records_its_error()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            shell.Submit("bp add 808000 log wibble");

            target.Breakpoints.ShouldBreak(0x808000);

            Assert.Equal(1, target.Breakpoints.LogEntriesRecorded);
            Assert.Contains("wibble", shell.Submit("bp log").Output);
        }

        [Fact]
        public void Bp_log_says_so_when_no_logpoint_exists_and_when_none_has_fired()
        {
            var target = BuildTarget();
            var shell = Shell(target);

            Assert.Contains("No logpoints set", shell.Submit("bp log").Output);

            shell.Submit("bp add 808000 log a");
            Assert.Contains("Nothing logged yet", shell.Submit("bp log").Output);
        }

        [Fact]
        public void Bp_log_clear_empties_the_recording()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            shell.Submit("bp add 808000 log a");
            target.Breakpoints.ShouldBreak(0x808000);

            Assert.Contains("cleared", shell.Submit("bp log clear").Output);
            Assert.Equal(0, target.Breakpoints.LogEntriesRecorded);
        }

        [Fact]
        public void Bp_list_distinguishes_a_logpoint_from_a_breakpoint()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            shell.Submit("bp add 808000 log a");
            shell.Submit("bp add 809000");

            string[] lines = shell.Submit("bp list").Output.Split('\n');

            Assert.Contains("log a", lines.Single(l => l.Contains("#1")));
            Assert.Contains("logged 0x", lines.Single(l => l.Contains("#1")));
            Assert.Contains("hit 0x", lines.Single(l => l.Contains("#2")));
        }

        [Fact]
        public void A_logpoint_combines_with_a_condition_in_either_order()
        {
            var target = BuildTarget();
            var shell = Shell(target);

            shell.Submit("bp add 808000 log a if x == 0");
            shell.Submit("bp add 809000 if x == 0 log a");

            var listed = target.Breakpoints.GetBreakpoints();
            Assert.All(listed, b => Assert.Equal("a", b.LogExpression));
            Assert.All(listed, b => Assert.Equal("x == 0", b.Condition));
        }

        [Fact]
        public void Cov_new_refuses_before_a_mark_and_works_after_one()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            shell.Submit("cov on");

            Assert.Contains("Nothing marked yet", shell.Submit("cov new 808000").Output);

            shell.Submit("cov mark");
            Assert.Contains("since the mark", shell.Submit("cov new 808000 10").Output);
        }

        [Fact]
        public void Cov_funcs_reports_discovered_routines()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            shell.Submit("cov on");
            target.Coverage!.RecordEntryPoint(0x808100, CallFrameKind.Call);

            string output = shell.Submit("cov funcs").Output;

            Assert.Contains("$808100", output);
            Assert.Contains("entered 1x", output);
        }

        [Fact]
        public void Cov_funcs_says_so_when_recording_is_off()
        {
            var target = BuildTarget();

            Assert.Contains("recording is OFF", Shell(target).Submit("cov funcs").Output);
        }

        [Fact]
        public void Cov_save_then_load_round_trips_through_a_file()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            shell.Submit("cov on");
            target.Coverage!.Record(0x818ABC);
            target.Coverage.RecordEntryPoint(0x808100, CallFrameKind.Call);

            string path = System.IO.Path.Combine(DianaOSSandbox.LogsDirectory, target.CoreName, "wiseman-test.covmap");
            try
            {
                Assert.Contains("saved to", shell.Submit("cov save wiseman-test.covmap").Output);

                target.Coverage.Clear();
                Assert.False(target.Coverage.WasExecuted(0x818ABC));

                Assert.Contains("merged from", shell.Submit("cov load wiseman-test.covmap").Output);
                Assert.True(target.Coverage.WasExecuted(0x818ABC));
                Assert.Equal(1, target.Coverage.EntryPointCount);
            }
            finally
            {
                // A test must not leave a map behind in the user's own log directory.
                if (System.IO.File.Exists(path)) System.IO.File.Delete(path);
            }
        }

        [Fact]
        public void Cov_load_of_a_missing_map_fails_cleanly()
        {
            var target = BuildTarget();

            Assert.Contains("No coverage map", Shell(target).Submit("cov load wiseman-does-not-exist.covmap").Output);
        }
    }
}
