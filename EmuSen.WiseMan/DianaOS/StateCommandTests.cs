using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;

namespace EmuSen.WiseMan.DianaOS
{
    // StateCommand's own logic - fake Action<string>/Func<string>
    // callbacks stand in for a real ICore/EmulatorSession, matching the
    // pattern EmuSen.Hotaru/EmuSen.Mistress9 actually wire it with (see
    // that file's own header comment on why it's constructor-injected
    // delegates, not IDebugTarget access).
    public class StateCommandTests
    {
        private static string[] Args(params string[] words) => words;

        [Fact]
        public void Save_with_no_path_uses_default_path()
        {
            string? savedPath = null;
            var command = new StateCommand(p => savedPath = p, _ => { }, () => "default.state");

            DianaOSResult result = command.Execute(null, Args("state", "save"), null);

            Assert.Equal("default.state", savedPath);
            Assert.Equal(0, result.ExitCode);
            Assert.Contains("default.state", result.Output);
        }

        [Fact]
        public void Save_with_explicit_path_overrides_default()
        {
            string? savedPath = null;
            var command = new StateCommand(p => savedPath = p, _ => { }, () => "default.state");

            command.Execute(null, Args("state", "save", "custom.state"), null);

            Assert.Equal("custom.state", savedPath);
        }

        [Fact]
        public void Load_with_no_path_uses_default_path()
        {
            string? loadedPath = null;
            var command = new StateCommand(_ => { }, p => loadedPath = p, () => "default.state");

            command.Execute(null, Args("state", "load"), null);

            Assert.Equal("default.state", loadedPath);
        }

        [Fact]
        public void Save_failure_is_reported_without_throwing()
        {
            var command = new StateCommand(
                _ => throw new System.IO.IOException("disk full"),
                _ => { },
                () => "default.state");

            DianaOSResult result = command.Execute(null, Args("state", "save"), null);

            Assert.NotEqual(0, result.ExitCode);
            Assert.Contains("disk full", result.Output);
        }

        [Fact]
        public void Missing_subcommand_reports_usage()
        {
            var command = new StateCommand(_ => { }, _ => { }, () => "default.state");

            DianaOSResult result = command.Execute(null, Args("state"), null);

            Assert.NotEqual(0, result.ExitCode);
            Assert.Contains("Usage", result.Output);
        }

        [Fact]
        public void Unknown_subcommand_reports_usage()
        {
            var command = new StateCommand(_ => { }, _ => { }, () => "default.state");

            DianaOSResult result = command.Execute(null, Args("state", "frobnicate"), null);

            Assert.NotEqual(0, result.ExitCode);
            Assert.Contains("Usage", result.Output);
        }
    }
}
