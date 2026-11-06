using EmuSen.Cores;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // Mistyped command and subcommand names get a suggestion instead of only
    // a list - see EmuSen_Config_Reference.md §6.1.
    public class SuggestionMessageTests
    {
        private static DianaShInterpreter Shell()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return DianaShInterpreter.CreateDefault(SyntheticRom.TargetOf(core));
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

        [Fact]
        public void A_mistyped_cheat_subcommand_is_suggested()
        {
            string output = Shell().Execute("cheat lst");

            Assert.Contains("Unknown 'cheat' subcommand 'lst'", output);
            Assert.Contains("Did you mean 'list'?", output);
        }

        [Fact]
        public void A_mistyped_watch_subcommand_is_suggested()
        {
            string output = Shell().Execute("watch sumary");

            Assert.Contains("Did you mean 'summary'?", output);
        }

        [Fact]
        public void A_mistyped_breakpoint_subcommand_is_suggested()
        {
            string output = Shell().Execute("bp lst");

            Assert.Contains("Did you mean 'list'?", output);
        }

        [Fact]
        public void A_mistyped_framelog_subcommand_is_suggested()
        {
            string output = Shell().Execute("framelog shwo");

            Assert.Contains("Did you mean 'show'?", output);
        }

        // The "Try a/b/c" list is generated from the same array the suggestion
        // searches, so the two can't drift apart.
        [Fact]
        public void The_try_list_still_names_every_subcommand()
        {
            string output = Shell().Execute("cheat zzzzzzzz");

            foreach (string sub in new[] { "add", "poke", "gg", "rompatch", "list", "enable", "disable", "remove", "clear", "save", "load", "files" })
            {
                Assert.Contains(sub, output);
            }
        }
    }
}
