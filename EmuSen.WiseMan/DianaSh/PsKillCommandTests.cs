using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Dev;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // `ps` and `kill`, the shell's builtins - the unified
    // "list every breakpoint/watch/session as one table" / "remove any of
    // them by that same id" pair. `ps`'s ids ("bp<N>"/"watch<N>"/a session
    // name) are exactly what `kill` parses, so most of these tests round-
    // trip through both: add something, confirm `ps` shows it with the
    // expected id shape, then `kill` that exact id and confirm it's gone.
    public class PsKillCommandTests
    {
        [Fact]
        public void Ps_with_nothing_active_reports_an_empty_table()
        {
            var shell = DianaShInterpreter.CreateWithoutSession();

            var result = shell.Submit("ps");

            Assert.Contains("No active", result.Output);
        }

        [Fact]
        public void Jobs_is_a_plain_alias_for_ps()
        {
            var shell = DianaShInterpreter.CreateWithoutSession();

            Assert.Equal(shell.Submit("ps").Output, shell.Submit("jobs").Output);
        }

        [Fact]
        public void Ps_lists_every_session_marking_the_current_one()
        {
            var (shell, _) = DianaShellFixtures.NewShellWithSessions();
            shell.Submit("tmux new investigation");

            var result = shell.Submit("ps");

            Assert.Contains("main", result.Output);
            Assert.Contains("investigation", result.Output);
            Assert.Contains("(current)", result.Output);
        }

        [Fact]
        public void Kill_a_non_current_session_by_name_removes_it_without_switching()
        {
            var (shell, sessions) = DianaShellFixtures.NewShellWithSessions();
            sessions.CreateSession(() => DianaShInterpreter.CreateWithoutSession(sessions: sessions), "investigation");
            sessions.TrySwitch("main", out _);

            var result = shell.Submit("kill investigation");

            Assert.Contains("removed", result.Output);
            Assert.Null(result.Action);
            Assert.DoesNotContain("investigation", shell.Submit("ps").Output);
        }

        [Fact]
        public void Killing_the_current_session_switches_away_first()
        {
            var (shell, sessions) = DianaShellFixtures.NewShellWithSessions();
            sessions.CreateSession(() => DianaShInterpreter.CreateWithoutSession(sessions: sessions), "investigation");

            var result = shell.Submit("kill investigation");

            var switched = Assert.IsType<HostAction.SwitchSession>(result.Action);
            Assert.Equal("main", switched.Name);
        }

        [Fact]
        public void Kill_refuses_to_remove_the_only_remaining_session()
        {
            var (shell, _) = DianaShellFixtures.NewShellWithSessions();

            var result = shell.Submit("kill main");

            Assert.Contains("can't kill the only remaining session", result.Output);
        }

        [Fact]
        public void Kill_with_no_session_manager_and_an_unrecognized_id_fails_cleanly()
        {
            var shell = DianaShInterpreter.CreateWithoutSession();

            var result = shell.Submit("kill nosuchid");

            Assert.Contains("no session manager", result.Output);
            Assert.Equal("1", shell.Submit("echo $?").Output.Trim());
        }
    }
}
