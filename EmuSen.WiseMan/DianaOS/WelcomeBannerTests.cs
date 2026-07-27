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
        public void Points_at_help_and_man_instead_of_listing_every_command()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            string banner = shell.GetWelcomeBanner(new[] { "SNES (Venus)" });

            Assert.Contains("Type \"help\" for a list of commands. Man is supported for each.", banner);
            Assert.DoesNotContain("Available commands:", banner); // the full listing used to be dumped inline - no longer
            Assert.DoesNotContain("echo <text...>", banner); // spot-check a real command's own Usage line does NOT leak in
        }

        [Fact]
        public void Does_not_pollute_command_history()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            shell.GetWelcomeBanner(new[] { "SNES (Venus)" });

            Assert.Empty(shell.History.Entries);
        }
    }
}
