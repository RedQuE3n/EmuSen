using System;
using System.IO;
using EmuSen.Crystal.Client;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Dev;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // `resume`, `shutdown` and `step` and their aliases reach their command and raise the right HostAction with no game, over the runtime's table with no game as a front end's console has it - see EmuSen_Backend.md §40.7.
    [Collection(TestCollections.ProcessGlobals)]
    public class ResumeShutdownStepCommandTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenResumeShutdownStep_" + Guid.NewGuid().ToString("N"));
        private readonly CrystalRuntime _runtime;

        public ResumeShutdownStepCommandTests()
        {
            Directory.CreateDirectory(_root);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _runtime = new CrystalRuntime();
        }

        public void Dispose()
        {
            _runtime.Dispose();
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private DianaShInterpreter NoGame() => DianaShInterpreter.CreateWithoutSession(_runtime.NoGameService());
        [Theory]
        [InlineData("resume")]
        [InlineData("continue")]
        [InlineData("c")]
        public void Resume_and_its_aliases_signal_HostAction_Resume(string commandLine)
        {
            var shell = NoGame();

            var result = shell.Submit(commandLine);

            Assert.IsType<HostAction.Resume>(result.Action);
        }

        [Theory]
        [InlineData("shutdown")]
        [InlineData("quit")]
        public void Shutdown_and_its_alias_signal_HostAction_Shutdown(string commandLine)
        {
            var shell = NoGame();

            var result = shell.Submit(commandLine);

            Assert.IsType<HostAction.Shutdown>(result.Action);
        }

        [Fact]
        public void Step_with_no_target_fails_cleanly_and_signals_no_action()
        {
            var shell = NoGame();

            var result = shell.Submit("step");

            Assert.Contains("No ROM loaded", result.Output);
            Assert.Null(result.Action);
            Assert.Equal("1", shell.Submit("echo $?").Output.Trim());
        }

        [Fact]
        public void Help_lists_each_command_exactly_once_despite_multiple_aliases()
        {
            var shell = NoGame();

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
