using System;
using System.IO;
using EmuSen.DianaOS;

namespace EmuSen.WiseMan.DianaOS
{
    // HostAction plumbing - DianaOSResult's new optional third field and
    // DianaOSInterpreter's propagation of it through Submit()/RunScript(),
    // added as pure infrastructure ahead of any real command using it
    // (see HostAction.cs's own comment) so it's reviewable/testable
    // independently of Phase 3/4 landing. Two tiny test-only commands
    // (registered via CreateDefault's own extraCommands mechanism, the
    // same one real frontends use for their host-specific commands) stand
    // in for real Shutdown/Resume-shaped commands.
    public class HostActionPlumbingTests
    {
        private sealed class FakeShutdownCommand : IDianaOSCommand
        {
            public string Name => "fakeshutdown";
            public bool IsReadOnly => false;
            public string Usage => "  fakeshutdown    test-only command that always signals HostAction.Shutdown";

            public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
                => DianaOSResult.Ok("shutdown-triggered", new HostAction.Shutdown());
        }

        private sealed class FakeResumeCommand : IDianaOSCommand
        {
            public string Name => "fakeresume";
            public bool IsReadOnly => false;
            public string Usage => "  fakeresume    test-only command that always signals HostAction.Resume";

            public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
                => DianaOSResult.Ok("resume-triggered", new HostAction.Resume());
        }

        private static DianaOSInterpreter NewShell() =>
            DianaOSInterpreter.CreateDefault(null, new IDianaOSCommand[] { new FakeShutdownCommand(), new FakeResumeCommand() });

        [Fact]
        public void Plain_submit_surfaces_the_action()
        {
            var shell = NewShell();

            var result = shell.Submit("fakeshutdown");

            Assert.Equal("shutdown-triggered", result.Output);
            Assert.IsType<HostAction.Shutdown>(result.Action);
        }

        [Fact]
        public void Ordinary_commands_never_produce_an_action()
        {
            var shell = NewShell();

            var result = shell.Submit("echo hi");

            Assert.Null(result.Action);
        }

        [Fact]
        public void Action_does_not_leak_out_of_command_substitution()
        {
            // A real bash subshell's 'exit' doesn't affect the parent
            // shell - $(...) must behave the same way here.
            var shell = NewShell();

            var result = shell.Submit("echo $(fakeshutdown)");

            Assert.Equal("shutdown-triggered", result.Output.Trim());
            Assert.Null(result.Action);
        }

        [Fact]
        public void Later_actionless_command_does_not_clear_an_already_set_action()
        {
            // "fakeshutdown; echo two" - the whole line finishes (echo
            // two still runs) before the action is delivered; echo
            // producing no action of its own must not clear the one
            // fakeshutdown already set.
            var shell = NewShell();

            var result = shell.Submit("fakeshutdown; echo two");

            Assert.Contains("shutdown-triggered", result.Output);
            Assert.Contains("two", result.Output);
            Assert.IsType<HostAction.Shutdown>(result.Action);
        }

        [Fact]
        public void Later_action_producing_command_wins_over_an_earlier_one()
        {
            var shell = NewShell();

            var result = shell.Submit("fakeresume; fakeshutdown");

            Assert.IsType<HostAction.Shutdown>(result.Action);
        }

        [Fact]
        public void A_fresh_submit_never_sees_a_stale_action_from_an_earlier_one()
        {
            var shell = NewShell();

            shell.Submit("fakeshutdown");
            var result = shell.Submit("echo hi");

            Assert.Null(result.Action);
        }

        [Fact]
        public void Action_propagates_out_of_a_sourced_script_and_halts_it()
        {
            string dir = Path.Combine(DianaOSSandbox.RootDirectory, "var", "log", "WiseManHostActionPlumbingTests");
            Directory.CreateDirectory(dir);
            string scriptPath = Path.Combine(dir, $"script_{Guid.NewGuid():N}.txt");
            File.WriteAllText(scriptPath, "echo before\nfakeshutdown\necho after\n");
            try
            {
                var shell = NewShell();

                var result = shell.Submit($"source \"{scriptPath}\"");

                Assert.Contains("before", result.Output);
                Assert.Contains("shutdown-triggered", result.Output);
                Assert.DoesNotContain("after", result.Output);
                Assert.IsType<HostAction.Shutdown>(result.Action);
            }
            finally
            {
                File.Delete(scriptPath);
            }
        }

        [Fact]
        public void No_action_means_a_sourced_script_runs_to_completion()
        {
            string dir = Path.Combine(DianaOSSandbox.RootDirectory, "var", "log", "WiseManHostActionPlumbingTests");
            Directory.CreateDirectory(dir);
            string scriptPath = Path.Combine(dir, $"script_{Guid.NewGuid():N}.txt");
            File.WriteAllText(scriptPath, "echo one\necho two\n");
            try
            {
                var shell = NewShell();

                var result = shell.Submit($"source \"{scriptPath}\"");

                Assert.Equal("one\ntwo", result.Output.Trim());
                Assert.Null(result.Action);
            }
            finally
            {
                File.Delete(scriptPath);
            }
        }
    }
}
