using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.Crystal
{
    // Pharaoh, run as its own process, is on the Rust Crystal by default and on the C# one variable away, and the C# shell's variable alone is refused in words - see EmuSen_Backend.md §29.27 and §29.28.
    public class PharaohCrystalDefaultTests
    {
        private static (int Exit, string Output) Pharaoh(string? crystal, string? dianash) =>
            ProgramRun.Run("EmuSen.Pharaoh.dll", "game.nes", SyntheticNesRom.Build(),
                new Dictionary<string, string?> { [ProgramRun.CrystalVariable] = crystal, [ProgramRun.DianaShVariable] = dianash }, "{game}", "1", "--nobattery");

        // The line Pharaoh says its runtime in, after one frame of a synthetic NES image.
        private static string RuntimeLine(string? crystal, string? dianash = null)
        {
            var (exit, output) = Pharaoh(crystal, dianash);
            Assert.True(exit == 0, $"Pharaoh exited {exit}: {output}");
            return output.Split('\n').Select(l => l.TrimEnd('\r')).Single(l => l.StartsWith("[ROM] Runtime: ", StringComparison.Ordinal));
        }

        [Fact]
        public void With_the_variable_unset_Pharaoh_runs_on_the_rust()
        {
            string line = RuntimeLine(null);
            Assert.Contains("frontend API 1.", line);
            Assert.DoesNotContain("C# implementation", line);
        }

        [Fact]
        public void The_variable_0_puts_Pharaoh_on_the_csharp()
        {
            Assert.Equal("[ROM] Runtime: the C# implementation (EMUSEN_CRYSTAL_NATIVE is 0)", RuntimeLine("0"));
        }

        [Fact]
        public void The_csharp_shell_s_variable_alone_is_refused_in_words_naming_both()
        {
            var (exit, output) = Pharaoh(null, "0");
            Assert.Equal(1, exit);
            Assert.Contains($"[ERROR] {DianaShInterpreter.ForeignTableRefusal}", output.Split('\n').Select(l => l.TrimEnd('\r')));
            Assert.DoesNotContain("Unhandled exception", output);
        }

        [Fact]
        public void Both_variables_0_run_on_the_csharp_shell_and_runtime()
        {
            Assert.Equal("[ROM] Runtime: the C# implementation (EMUSEN_CRYSTAL_NATIVE is 0)", RuntimeLine("0", "0"));
        }
    }
}
