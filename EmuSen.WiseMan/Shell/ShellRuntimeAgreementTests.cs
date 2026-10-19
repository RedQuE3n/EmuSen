using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.Shell
{
    // emusen-shell, run as its own process with a game and no display, refuses the C# shell over the Rust runtime's table in words naming both variables, before any window - see EmuSen_Backend.md §29.28.
    public class ShellRuntimeAgreementTests
    {
        [Fact]
        public void The_csharp_shell_over_the_rust_runtime_is_refused_in_words_naming_both()
        {
            var (exit, output) = ProgramRun.Run("emusen-shell.dll", "game.gb", SyntheticGbRom.Build(),
                new Dictionary<string, string?> { [ProgramRun.CrystalVariable] = "1", [ProgramRun.DianaShVariable] = "0" }, "--console", "{game}");
            Assert.Equal(1, exit);
            Assert.Contains($"[ERROR] {DianaShInterpreter.ForeignTableRefusal}", output.Split('\n').Select(l => l.TrimEnd('\r')));
            Assert.DoesNotContain("[CPU HALT]", output);
        }
    }
}
