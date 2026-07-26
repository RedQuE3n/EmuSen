using EmuSen.DianaOS;

namespace EmuSen.WiseMan.DianaOS
{
    // ResumeCommand/ShutdownCommand/StepCommand (EmuSen/DianaOS/Commands/)
    // and their alias normalization in DianaOSInterpreter.Dispatch
    // ('continue'/'c' -> 'resume', 'quit' -> 'shutdown', 's' -> 'step').
    // Dispatched against a null target the same way DianaOSDispatchTests
    // does for every other command - the "does this reach the command and
    // produce the right HostAction" smoke test.
    public class ResumeShutdownStepCommandTests
    {
        [Theory]
        [InlineData("resume")]
        [InlineData("continue")]
        [InlineData("c")]
        public void Resume_and_its_aliases_signal_HostAction_Resume(string commandLine)
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit(commandLine);

            Assert.IsType<HostAction.Resume>(result.Action);
        }

        [Theory]
        [InlineData("shutdown")]
        [InlineData("quit")]
        public void Shutdown_and_its_alias_signal_HostAction_Shutdown(string commandLine)
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit(commandLine);

            Assert.IsType<HostAction.Shutdown>(result.Action);
        }

        [Fact]
        public void Step_with_no_target_fails_cleanly_and_signals_no_action()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit("step");

            Assert.Contains("No ROM loaded", result.Output);
            Assert.Null(result.Action);
            Assert.Equal("1", shell.Submit("echo $?").Output.Trim());
        }

        [Fact]
        public void Help_lists_each_command_exactly_once_despite_multiple_aliases()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            string help = shell.Submit("help").Output;

            Assert.Equal(1, CountOccurrences(help, "resume | continue | c"));
            Assert.Equal(1, CountOccurrences(help, "shutdown | quit"));
            Assert.Equal(1, CountOccurrences(help, "step | s"));
        }

        private static int CountOccurrences(string haystack, string needle)
        {
            int count = 0;
            int index = 0;
            while ((index = haystack.IndexOf(needle, index, System.StringComparison.Ordinal)) >= 0)
            {
                count++;
                index += needle.Length;
            }
            return count;
        }
    }
}
