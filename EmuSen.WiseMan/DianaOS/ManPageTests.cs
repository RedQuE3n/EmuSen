using System.Linq;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.WiseMan.DianaOS
{
    // `man`/`help` (DianaOSInterpreter.Man/Help, backed by ManPages.cs) -
    // two separate commands, not aliases of each other (see Dispatch's
    // own comment): `help` always lists every command with a brief
    // explanation, ignoring any arguments; `man [command]` is the detail
    // lookup, falling back to that same listing only when given no
    // argument. Every command DianaOSInterpreter.CreateDefault registers
    // is expected to have a real ManPages entry, not just a fallback to
    // its one-line Usage - this is the regression test that catches a
    // newly-added command (or one that gets renamed) quietly falling
    // through to that fallback and staying that way forever.
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
