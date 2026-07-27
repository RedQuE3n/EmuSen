using EmuSen.DianaOS;

namespace EmuSen.WiseMan.DianaOS
{
    // Every registered shell/debug command, dispatched against a null
    // IDebugTarget (no ROM loaded) - the "does this even reach the
    // command and fail cleanly" smoke test every command should pass
    // regardless of whether real hardware access is involved.
    // DianaOSInterpreter.Submit already catches everything internally and
    // renders it as "Error: ..." text, so a thrown exception escaping
    // Submit here would mean that guard itself broke, not just an
    // individual command's own logic.
    public class DianaOSDispatchTests
    {
        [Theory]
        [InlineData("help")]
        [InlineData("spaces")]
        [InlineData("mem CpuBus 0 16")]
        [InlineData("regs")]
        [InlineData("watch add WRAM 0 4")]
        [InlineData("watch list")]
        [InlineData("bp add 8000")]
        [InlineData("bp list")]
        [InlineData("bp remove 1")]
        [InlineData("framelog add WRAM 0 1")]
        [InlineData("cheat poke WRAM 10 9 test")]
        [InlineData("cheat list")]
        [InlineData("search WRAM 0")]
        [InlineData("snapshot WRAM foo")]
        [InlineData("diff foo")]
        [InlineData("dump WRAM 0 16 test.bin")]
        [InlineData("tile WRAM 0 4")]
        [InlineData("tilemap WRAM 0 2 2")]
        [InlineData("disasm CpuBus 8000 4")]
        [InlineData("trace 5")]
        [InlineData("callers 8000")]
        [InlineData("writers 8000")]
        [InlineData("readers 8000")]
        [InlineData("log status")]
        [InlineData("sprites")]
        [InlineData("pal")]
        [InlineData("channels")]
        [InlineData("mute 0 on")]
        [InlineData("clear")]
        [InlineData("echo hi | sed s/hi/bye/")]
        [InlineData("export X=5; echo $X")]
        [InlineData("for i in 1 2 3; do echo $i; done")]
        [InlineData("while false; do echo never; done; echo after-while")]
        [InlineData("if true; then echo yes; else echo no; fi")]
        [InlineData("for i in 1 2 3; do if [ $i -eq 2 ]; then break; fi; echo $i; done")]
        [InlineData("for i in 1 2 3; do if [ $i -eq 2 ]; then continue; fi; echo $i; done")]
        [InlineData("break")]
        [InlineData("continue")]
        [InlineData("true && echo and-ok")]
        [InlineData("false || echo or-ok")]
        [InlineData("echo $(echo nested)")]
        public void Dispatches_without_throwing(string commandLine)
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            Exception? ex = Record.Exception(() => shell.Submit(commandLine));
            Assert.Null(ex);
        }

        [Fact]
        public void Loop_body_break_exits_after_matching_iteration()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            var result = shell.Submit("for i in 1 2 3; do if [ $i -eq 2 ]; then break; fi; echo $i; done");
            Assert.Equal("1", result.Output.Trim());
        }

        [Fact]
        public void Loop_body_continue_skips_matching_iteration()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            var result = shell.Submit("for i in 1 2 3; do if [ $i -eq 2 ]; then continue; fi; echo $i; done");
            Assert.Equal("1\n3", result.Output.Trim());
        }

        [Fact]
        public void Command_substitution_and_pipes_work()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            Assert.Equal("bye", shell.Submit("echo hi | sed s/hi/bye/").Output.Trim());
            Assert.Equal("nested", shell.Submit("echo $(echo nested)").Output.Trim());
        }

        [Fact]
        public void Commands_without_a_target_report_a_clean_error_not_a_crash()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            var result = shell.Submit("regs");
            Assert.Contains("No ROM loaded", result.Output);
        }

        // A frontend-supplied extraCommand sharing a name with one of the
        // standard registry (e.g. EmuSen.Mistress9 replacing `coretop`'s
        // raw-terminal implementation with a windowed one) should REPLACE
        // the default rather than throwing on a duplicate key - see
        // CreateDefault's own comment.
        private sealed class FakeOverrideCommand : IDianaOSCommand
        {
            public string Name => "echo";
            public bool IsReadOnly => true;
            public string Usage => "  echo (overridden for this test)";
            public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin) => "overridden";
        }

        [Fact]
        public void ExtraCommand_sharing_a_default_name_replaces_the_default_instead_of_throwing()
        {
            var ex = Record.Exception(() => DianaOSInterpreter.CreateDefault(null, new IDianaOSCommand[] { new FakeOverrideCommand() }));
            Assert.Null(ex);

            var shell = DianaOSInterpreter.CreateDefault(null, new IDianaOSCommand[] { new FakeOverrideCommand() });
            Assert.Equal("overridden", shell.Submit("echo hi").Output.Trim());
        }

        [Fact]
        public void Unrelated_extra_commands_still_add_normally_alongside_the_defaults()
        {
            var shell = DianaOSInterpreter.CreateDefault(null, new IDianaOSCommand[] { new FakeOverrideCommand() });
            // A non-colliding default command is untouched by an
            // unrelated override.
            Assert.Contains("No ROM loaded", shell.Submit("regs").Output);
        }
    }
}
