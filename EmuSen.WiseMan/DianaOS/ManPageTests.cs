using System.Linq;
using EmuSen.DianaOS;

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
        // The exact Name every command DianaOSInterpreter.CreateDefault
        // registers reports (see that method's own command list) - kept
        // here rather than reflected out of the interpreter itself,
        // since IDianaOSCommand instances aren't otherwise enumerable
        // from outside DianaOSInterpreter and reflecting private fields
        // just to list them would be more fragile than one explicit,
        // readable list that a reviewer can diff against CreateDefault
        // by eye when either one changes.
        private static readonly string[] RegisteredCommandNames =
        {
            "spaces", "mem", "write", "regs", "sprites", "pal", "channels", "mute",
            "watch", "bp", "framelog", "cheat", "search", "snapshot", "diff", "dump",
            "load", "tile", "tilemap", "disasm", "trace", "callers", "writers",
            "readers", "log", "echo", "sed", "grep", "wc", "sort", "uniq", "awk",
            "ls", "cd", "mv", "pwd", "nano", "coretop", "true", "false", "test", "[", "history",
        };

        // The special builtins Dispatch handles directly (help/man
        // itself, plus everything else that isn't an ordinary
        // IDianaOSCommand - export/unset/source, and the parser-level
        // control-flow keywords) - man/help should cover these too, not
        // just the "real" command objects.
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
            RegisteredCommandNames.Concat(SpecialBuiltinNames).Select(n => new object[] { n });

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
