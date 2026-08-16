using System.Linq;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Bin.Commands;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.WiseMan.DianaOS
{
    // Catches a command quietly falling through to its one-line Usage forever - see `man man`.
    public class ManPageTests
    {
        // Read from a live interpreter, not a hand list - see EmuSen_Debugging_Tools_Reference_v5.md §3.18.
        private static string[] RegisteredCommandNames =>
            DianaOSInterpreter.CreateDefault(null).CommandNames.ToArray();

        // Special builtins Dispatch handles directly - not IDianaOSCommand instances, so CommandNames can't cover them.
        private static readonly string[] SpecialBuiltinNames =
        {
            "man", "help", "summary", "export", "unset", "source",
            "if", "for", "while", "break", "continue",
        };

        [Theory]
        [MemberData(nameof(AllNames))]
        public void Every_command_and_builtin_has_a_real_man_page(string name)
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            var result = shell.Submit($"man {name}");

            Assert.DoesNotContain("No detailed manual page yet", result.Output);
            Assert.DoesNotContain("No manual entry", result.Output);
            Assert.Contains("NAME", result.Output);
        }

        public static System.Collections.Generic.IEnumerable<object[]> AllNames() =>
            RegisteredCommandNames.Concat(SpecialBuiltinNames).Distinct().Select(n => new object[] { n });

        // A frontend's own extraCommands never reach CreateDefault(null) above - see EmuSen_Settings_Reference.md §4.21.
        [Theory]
        [InlineData("pause")]
        [InlineData("feed")]
        [InlineData("coretop")]
        [InlineData("vstop")]
        [InlineData("resume")]
        [InlineData("state")]
        public void Every_frontend_registered_command_has_a_real_man_page(string name)
        {
            string? page = ManPages.Lookup(name);

            Assert.NotNull(page);
            Assert.Contains("NAME", page);
            Assert.Contains("DESCRIPTION", page);
        }

        [Fact]
        public void Help_with_no_argument_still_lists_every_command()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            var result = shell.Submit("help");

            Assert.StartsWith("Available commands:", result.Output);
            foreach (string name in RegisteredCommandNames)
            {
                Assert.Contains(name, result.Output);
            }
        }

        [Fact]
        public void Man_with_no_argument_behaves_the_same_as_bare_help()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            Assert.Equal(shell.Submit("help").Output, shell.Submit("man").Output);
        }

        [Fact]
        public void Help_ignores_arguments_and_always_lists_everything()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            Assert.Equal(shell.Submit("help").Output, shell.Submit("help mem").Output);
            Assert.NotEqual(shell.Submit("man mem").Output, shell.Submit("help mem").Output);
        }

        [Fact]
        public void Man_of_an_unknown_name_reports_a_clean_error_with_a_nonzero_exit_code()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);
            var result = shell.Submit("man totally_not_a_real_command");
            Assert.Contains("No manual entry", result.Output);

            // Submit itself doesn't surface an exit code directly - $? is
            // how a script/condition would observe it, same as real bash.
            Assert.Equal("1", shell.Submit("echo $?").Output.Trim());
        }
    }
}
