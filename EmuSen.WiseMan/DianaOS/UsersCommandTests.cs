using System;
using System.IO;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // whoami/who/su/useradd/userdel/passwd + DianaOSUserRegistry - see EmuSen_Debugging_Tools_Reference_v5.md §3.18.
    public class UsersCommandTests
    {
        private static string UniqueName(string prefix) => $"{prefix}_{Guid.NewGuid():N}";

        [Fact]
        public void Every_shell_starts_as_root()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            Assert.Equal("root", shell.Submit("whoami").Output.Trim());
        }

        [Fact]
        public void Root_can_useradd_and_then_su_to_the_new_account()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string name = UniqueName("kid");

            var addResult = shell.Submit($"useradd {name}");
            Assert.Contains("Added user", addResult.Output);

            var suResult = shell.Submit($"su {name}");
            Assert.Contains("Switched to", suResult.Output);
            Assert.Equal(name, shell.Submit("whoami").Output.Trim());
        }

        [Fact]
        public void Useradd_rejects_a_duplicate_name()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string name = UniqueName("dup");
            shell.Submit($"useradd {name}");

            var result = shell.Submit($"useradd {name}");

            Assert.Contains("already exists", result.Output);
            Assert.Equal("1", shell.Submit("echo $?").Output.Trim());
        }

        [Fact]
        public void Useradd_requires_root()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string nonRoot = UniqueName("nonroot");
            shell.Submit($"useradd {nonRoot}");
            shell.Submit($"su {nonRoot}");

            var result = shell.Submit($"useradd {UniqueName("other")}");

            Assert.Contains("only root can add users", result.Output);
        }

        [Fact]
        public void Su_to_a_nonexistent_user_fails_cleanly()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit($"su {UniqueName("ghost")}");

            Assert.Contains("no such user", result.Output);
            Assert.Equal("root", shell.Submit("whoami").Output.Trim());
        }

        [Fact]
        public void Su_with_no_argument_returns_to_root()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string name = UniqueName("kid");
            shell.Submit($"useradd {name}");
            shell.Submit($"su {name}");

            shell.Submit("su");

            Assert.Equal("root", shell.Submit("whoami").Output.Trim());
        }

        [Fact]
        public void Userdel_requires_root()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string nonRoot = UniqueName("nonroot");
            string victim = UniqueName("victim");
            shell.Submit($"useradd {nonRoot}");
            shell.Submit($"useradd {victim}");
            shell.Submit($"su {nonRoot}");

            var result = shell.Submit($"userdel {victim}");

            Assert.Contains("only root can remove users", result.Output);
        }

        [Fact]
        public void Userdel_refuses_to_remove_root()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit("userdel root");

            Assert.Contains("refusing to remove 'root'", result.Output);
        }

        [Fact]
        public void Userdel_refuses_an_account_active_in_a_live_session()
        {
            var (shell, sessions) = DianaShellFixtures.NewShellWithSessions();
            string name = UniqueName("kid");
            shell.Submit($"useradd {name}");
            shell.Submit($"su {name}");

            var result = shell.Submit($"su root"); // back to root so THIS shell can attempt the userdel
            Assert.Contains("Switched to", result.Output);
            // Re-become the target in a second session so it's active
            // somewhere while root (this session) tries to remove it.
            sessions.CreateSession(() => DianaOSInterpreter.CreateDefault(null, sessions: sessions), "other");
            sessions.Current!.Interpreter.CurrentUser = name;
            sessions.TrySwitch("main", out _);

            var delResult = shell.Submit($"userdel {name}");

            Assert.Contains("currently in use by session", delResult.Output);
        }

        [Fact]
        public void Userdel_succeeds_for_an_unused_account()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string name = UniqueName("unused");
            shell.Submit($"useradd {name}");

            var result = shell.Submit($"userdel {name}");

            Assert.Contains("Removed user", result.Output);
            Assert.Contains("no such user", shell.Submit($"su {name}").Output);
        }

        [Fact]
        public void Passwd_on_your_own_account_is_always_allowed()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string name = UniqueName("kid");
            shell.Submit($"useradd {name}");
            shell.Submit($"su {name}");

            var result = shell.Submit("passwd hunter2");

            Assert.Contains("Password updated", result.Output);
        }

        [Fact]
        public void Passwd_on_another_account_requires_root()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string a = UniqueName("a");
            string b = UniqueName("b");
            shell.Submit($"useradd {a}");
            shell.Submit($"useradd {b}");
            shell.Submit($"su {a}");

            var result = shell.Submit($"passwd {b} hunter2");

            Assert.Contains("only root can change another user's password", result.Output);
        }

        [Fact]
        public void Who_lists_every_session_with_its_current_user()
        {
            var (shell, sessions) = DianaShellFixtures.NewShellWithSessions();
            string name = UniqueName("kid");
            shell.Submit($"useradd {name}");
            shell.Submit("tmux new investigation");
            shell.Submit($"su {name}");

            var result = shell.Submit("who");

            Assert.Contains("root", result.Output);
            Assert.Contains(name, result.Output);
            Assert.Contains("main", result.Output);
            Assert.Contains("investigation", result.Output);
        }

        [Fact]
        public void Who_with_no_session_manager_falls_back_to_this_shells_own_user()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit("who");

            Assert.Contains("root", result.Output);
        }

        [Fact]
        public void Tmux_new_inherits_the_creating_sessions_current_user()
        {
            var (shell, _) = DianaShellFixtures.NewShellWithSessions();
            string name = UniqueName("kid");
            shell.Submit($"useradd {name}");
            shell.Submit($"su {name}");

            shell.Submit("tmux new investigation");

            Assert.Equal(name, shell.Submit("whoami").Output.Trim());
        }

        [Fact]
        public void RebuildAll_preserves_each_sessions_current_user()
        {
            var (shell, sessions) = DianaShellFixtures.NewShellWithSessions();
            string name = UniqueName("kid");
            shell.Submit($"useradd {name}");
            shell.Submit($"su {name}");

            sessions.RebuildAll(() => DianaOSInterpreter.CreateDefault(null, sessions: sessions));

            Assert.Equal(name, sessions.Current!.Interpreter.Submit("whoami").Output.Trim());
        }

        [Fact]
        public void Command_substitution_inherits_the_current_user()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string name = UniqueName("kid");
            shell.Submit($"useradd {name}");
            shell.Submit($"su {name}");

            var result = shell.Submit("echo $(whoami)");

            Assert.Equal(name, result.Output.Trim());
        }

        [Fact]
        public void Useradd_creates_a_real_home_directory()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string name = UniqueName("kid");

            shell.Submit($"useradd {name}");

            Assert.True(Directory.Exists(DianaOSSandbox.HomeDirectory(name)));
        }

        [Fact]
        public void Cd_with_no_argument_goes_home_whoever_is_logged_in()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            string name = UniqueName("kid");
            shell.Submit($"useradd {name}");
            shell.Submit($"su {name}");
            shell.Submit("cd /tmp");

            shell.Submit("cd");

            Assert.Equal(DianaOSSandbox.HomeDirectory(name), shell.Submit("pwd").Output.Trim());
        }

        [Fact]
        public void Cd_with_no_argument_goes_home_for_root_too()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            shell.Submit("cd /tmp");

            shell.Submit("cd");

            Assert.Equal(DianaOSSandbox.HomeDirectory("root"), shell.Submit("pwd").Output.Trim());
        }
    }
}
