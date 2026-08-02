using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // Mistyped command and subcommand names get a suggestion instead of only
    // a list - see EmuSen_Config_Reference.md §6.1.
    public class SuggestionMessageTests
    {
        private static DianaOSInterpreter Shell()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return DianaOSInterpreter.CreateDefault(new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!));
        }

        private static string Submit(string line)
        {
            (_, string output, _) = Shell().Submit(line);
            return output;
        }

        [Theory]
        [InlineData("chaet", "cheat")]
        [InlineData("wathc", "watch")]
        [InlineData("serach", "search")]
        public void A_mistyped_command_is_named_and_suggested(string typed, string expected)
        {
            string output = Submit(typed);

            Assert.Contains($"Unknown command '{typed}'", output);
            Assert.Contains($"Did you mean '{expected}'?", output);
        }

        // The list is still there - the suggestion is a guess, not a verdict.
        [Fact]
        public void The_help_pointer_survives_alongside_the_suggestion()
        {
            string output = Submit("chaet");

            Assert.Contains("Did you mean", output);
            Assert.Contains("Type 'help' for a list.", output);
        }

        [Fact]
        public void A_command_unlike_anything_registered_gets_no_guess()
        {
            string output = Submit("qwertyuiop");

            Assert.Contains("Unknown command 'qwertyuiop'", output);
            Assert.DoesNotContain("Did you mean", output);
        }

        private static SnesDebugTarget Target()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        }

        [Fact]
        public void A_mistyped_cheat_subcommand_is_suggested()
        {
            string output = new CheatCommand().Execute(Target(), new[] { "cheat", "lst" }, null).Output;

            Assert.Contains("Unknown 'cheat' subcommand 'lst'", output);
            Assert.Contains("Did you mean 'list'?", output);
        }

        [Fact]
        public void A_mistyped_watch_subcommand_is_suggested()
        {
            string output = new WatchCommand().Execute(Target(), new[] { "watch", "sumary" }, null).Output;

            Assert.Contains("Did you mean 'summary'?", output);
        }

        [Fact]
        public void A_mistyped_breakpoint_subcommand_is_suggested()
        {
            string output = new BreakCommand().Execute(Target(), new[] { "bp", "lst" }, null).Output;

            Assert.Contains("Did you mean 'list'?", output);
        }

        [Fact]
        public void A_mistyped_framelog_subcommand_is_suggested()
        {
            string output = new FrameLogCommand().Execute(Target(), new[] { "framelog", "shwo" }, null).Output;

            Assert.Contains("Did you mean 'show'?", output);
        }

        // The "Try a/b/c" list is generated from the same array the suggestion
        // searches, so the two can't drift apart.
        [Fact]
        public void The_try_list_still_names_every_subcommand()
        {
            string output = new CheatCommand().Execute(Target(), new[] { "cheat", "zzzzzzzz" }, null).Output;

            foreach (string sub in new[] { "add", "poke", "gg", "rompatch", "list", "enable", "disable", "remove", "clear", "save", "load", "files" })
            {
                Assert.Contains(sub, output);
            }
        }
    }
}
