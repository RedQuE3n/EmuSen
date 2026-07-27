using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // PsCommand/KillCommand (EmuSen.DianaOS/Commands/) - the unified
    // "list every breakpoint/watch/session as one table" / "remove any of
    // them by that same id" pair. `ps`'s ids ("bp<N>"/"watch<N>"/a session
    // name) are exactly what `kill` parses, so most of these tests round-
    // trip through both: add something, confirm `ps` shows it with the
    // expected id shape, then `kill` that exact id and confirm it's gone.
    public class PsKillCommandTests
    {
        private static SnesDebugTarget BuildTarget()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        }

        [Fact]
        public void Ps_with_nothing_active_reports_an_empty_table()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit("ps");

            Assert.Contains("No active", result.Output);
        }

        [Fact]
        public void Jobs_is_a_plain_alias_for_ps()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            Assert.Equal(shell.Submit("ps").Output, shell.Submit("jobs").Output);
        }

        [Fact]
        public void Ps_lists_a_breakpoint_with_the_bpN_id_kill_expects()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);
            shell.Submit("bp add 8000");

            var result = shell.Submit("ps");

            Assert.Contains("bp1", result.Output);
            Assert.Contains("breakpoint", result.Output);
            Assert.Contains("$008000", result.Output);
        }

        [Fact]
        public void Kill_bpN_removes_that_breakpoint()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);
            shell.Submit("bp add 8000");

            var killResult = shell.Submit("kill bp1");

            Assert.Contains("Breakpoint #1 removed", killResult.Output);
            Assert.DoesNotContain("bp1", shell.Submit("ps").Output);
        }

        [Fact]
        public void Kill_on_a_breakpoint_id_that_no_longer_exists_fails_cleanly()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);

            var result = shell.Submit("kill bp99");

            Assert.Contains("no breakpoint #99", result.Output);
            Assert.Equal("1", shell.Submit("echo $?").Output.Trim());
        }

        [Fact]
        public void Ps_lists_a_watch_with_the_watchN_id_kill_expects()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);
            shell.Submit("watch add WRAM 0 10");

            var result = shell.Submit("ps");

            Assert.Contains("watch1", result.Output);
            Assert.Contains("WRAM", result.Output);
        }

        [Fact]
        public void Kill_watchN_removes_that_watch()
        {
            var target = BuildTarget();
            var shell = DianaOSInterpreter.CreateDefault(target);
            shell.Submit("watch add WRAM 0 10");

            var killResult = shell.Submit("kill watch1");

            Assert.Contains("Watch #1 removed", killResult.Output);
            Assert.DoesNotContain("watch1", shell.Submit("ps").Output);
        }

        [Fact]
        public void Ps_lists_every_session_marking_the_current_one()
        {
            var (shell, _) = DianaShellFixtures.NewShellWithSessions(null);
            shell.Submit("tmux new investigation");

            var result = shell.Submit("ps");

            Assert.Contains("main", result.Output);
            Assert.Contains("investigation", result.Output);
            Assert.Contains("(current)", result.Output);
        }

        [Fact]
        public void Kill_a_non_current_session_by_name_removes_it_without_switching()
        {
            var (shell, sessions) = DianaShellFixtures.NewShellWithSessions(null);
            sessions.CreateSession(() => DianaOSInterpreter.CreateDefault(null, sessions: sessions), "investigation");
            sessions.TrySwitch("main", out _);

            var result = shell.Submit("kill investigation");

            Assert.Contains("removed", result.Output);
            Assert.Null(result.Action);
            Assert.DoesNotContain("investigation", shell.Submit("ps").Output);
        }

        [Fact]
        public void Killing_the_current_session_switches_away_first()
        {
            var (shell, sessions) = DianaShellFixtures.NewShellWithSessions(null);
            sessions.CreateSession(() => DianaOSInterpreter.CreateDefault(null, sessions: sessions), "investigation");

            var result = shell.Submit("kill investigation");

            var switched = Assert.IsType<HostAction.SwitchSession>(result.Action);
            Assert.Equal("main", switched.Name);
        }

        [Fact]
        public void Kill_refuses_to_remove_the_only_remaining_session()
        {
            var (shell, _) = DianaShellFixtures.NewShellWithSessions(null);

            var result = shell.Submit("kill main");

            Assert.Contains("can't kill the only remaining session", result.Output);
        }

        [Fact]
        public void Kill_with_no_session_manager_and_an_unrecognized_id_fails_cleanly()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit("kill nosuchid");

            Assert.Contains("no session manager", result.Output);
            Assert.Equal("1", shell.Submit("echo $?").Output.Trim());
        }
    }
}
