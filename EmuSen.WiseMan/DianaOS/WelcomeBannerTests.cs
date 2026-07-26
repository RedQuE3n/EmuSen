using EmuSen.DianaOS;

namespace EmuSen.WiseMan.DianaOS
{
    // GetWelcomeBanner() - the banner both EmuSen.Hotaru's RunStandaloneShell
    // and EmuSen.Mistress9's DianaOSConsoleWindow print once, at actual
    // shell launch. Headless-testable content only (text/formatting) -
    // there's no visual rendering to check here, same limitation every
    // other frontend-display feature in this project has.
    public class WelcomeBannerTests
    {
        [Fact]
        public void Includes_version_and_every_supplied_core()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            string banner = shell.GetWelcomeBanner(new[] { "SNES (Venus)", "Some Other Core" });

            Assert.Contains($"DianaOS v{DianaOSInterpreter.Version}", banner);
            Assert.Contains("SNES (Venus)", banner);
            Assert.Contains("Some Other Core", banner);
        }

        [Fact]
        public void Includes_the_full_command_listing()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            string banner = shell.GetWelcomeBanner(new[] { "SNES (Venus)" });

            Assert.Contains("Available commands:", banner);
            Assert.Contains("echo <text...>", banner); // spot-check one real command's own Usage line made it in
        }

        [Fact]
        public void Does_not_pollute_command_history()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            shell.GetWelcomeBanner(new[] { "SNES (Venus)" });

            Assert.Empty(shell.History.Entries);
        }

        [Fact]
        public void Reflects_extra_commands_registered_on_this_shell()
        {
            var shell = DianaOSInterpreter.CreateDefault(null,
                new IDianaOSCommand[] { new FakeExtraCommand() });

            string banner = shell.GetWelcomeBanner(new[] { "SNES (Venus)" });

            Assert.Contains("totally-not-a-real-command", banner);
        }

        private sealed class FakeExtraCommand : IDianaOSCommand
        {
            public string Name => "faketest";
            public string Usage => "  faketest                      totally-not-a-real-command, just here to prove extraCommands show up";

            public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin) => DianaOSResult.Ok("");
        }
    }
}
