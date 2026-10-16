using System;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Bin.Native;
using EmuSen.DianaSh.DianaSh.Var.Shell;
using EmuSen.WiseMan.Fixtures;
using Xunit;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.DianaSh
{
    // The switch chose what the variable says, so a default run is a run of the Rust shell and not a silent fallback - see EmuSen_DianaSh.md §13.10.
    [Collection(TestCollections.ProcessGlobals)]
    public class NativeSwitchTests(ITestOutputHelper output)
    {
        // The line a run's log shows; EMUSEN_DIANASH_EXPECT=rust makes a run that fell back to the C# fail, as CI's Rust steps set it.
        [Fact]
        public void The_run_says_which_shell_it_uses_and_the_rust_shell_answers_with_its_modules()
        {
            DianaShInterpreter shell = DianaShInterpreter.CreateDefault(null);
            string which = shell.NativeShellOf is null ? "C#" : "Rust";
            output.WriteLine($"DianaSh in this run: the {which} shell ({Environment.OSVersion.Platform}, {System.Runtime.InteropServices.RuntimeInformation.RuntimeIdentifier})");
            if (Environment.GetEnvironmentVariable("EMUSEN_DIANASH_EXPECT") is { Length: > 0 } expected)
                Assert.Equal(expected, which, ignoreCase: true);
            if (shell.NativeShellOf is not { } native) return;
            var kinds = native.Commands(shell.NativeSessionId).ToDictionary(c => c.Name, c => c.Kind);
            foreach (string name in new[] { "mem", "disasm", "label", "bp", "watch", "step", "cov", "summary", "grep" })
                Assert.Equal("module", kinds[name]);
        }

        // State 2: the Rust shell unless the variable is 0, where the library this run was built with is there to choose.
        private static bool Native => Environment.GetEnvironmentVariable("EMUSEN_DIANASH_NATIVE") != "0" && DianaShNative.Ready;

        [Fact]
        public void The_shell_is_the_one_the_variable_chose()
        {
            DianaShInterpreter shell = DianaShInterpreter.CreateDefault(null);
            Assert.Equal(Native, shell.NativeShellOf is not null);
            Assert.Equal("hello", shell.Submit("echo hello").Output);
        }

        [Fact]
        public void Sessions_made_with_a_manager_live_in_one_shell_and_keep_their_objects()
        {
            var (shell, sessions) = DianaShellFixtures.NewShellWithSessions();
            shell.Submit("tmux new second");
            Assert.Equal(new[] { "main", "second" }, Names(sessions));
            Assert.Same(sessions.Current, sessions.Current);
            Assert.Equal("second", sessions.Current!.Name);
            DianaShSession second = sessions.Current;
            Assert.True(sessions.TrySwitch("MAIN", out _));
            Assert.Same(shell, sessions.Current!.Interpreter);
            Assert.True(sessions.TryKill("second", out _));
            Assert.False(sessions.TryKill("main", out string error));
            Assert.Equal("can't kill the only remaining session", error);
            // A killed session's interpreter still answers, as the C# object did after its manager let it go.
            Assert.Equal("x", second.Interpreter.Submit("echo x").Output);
            if (Native) Assert.Same(shell.NativeShellOf, second.Interpreter.NativeShellOf);
        }

        private static string[] Names(DianaShSessionManager sessions) => Array.ConvertAll(System.Linq.Enumerable.ToArray(sessions.Sessions), s => s.Name);
    }
}
