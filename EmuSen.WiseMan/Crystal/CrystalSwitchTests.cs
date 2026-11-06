using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.WiseMan.Fixtures;
using Xunit;
using CrystalContract = EmuSen.Crystal.Client.Contract.Crystal;

namespace EmuSen.WiseMan.Crystal
{
    // The binding is the platform library's alone, and a leftover EMUSEN_CRYSTAL_NATIVE=0 is ignored with one line - see EmuSen_Backend.md §40.8.
    public class CrystalSwitchTests
    {
        private static (int Exit, string Output) Pharaoh(string? crystal, string? dianash = null) =>
            ProgramRun.Run("EmuSen.Pharaoh.dll", "game.nes", SyntheticNesRom.Build(),
                new Dictionary<string, string?> { [ProgramRun.CrystalVariable] = crystal, [ProgramRun.DianaShVariable] = dianash }, "{game}", "1", "--nobattery");

        private static IEnumerable<string> Lines(string output) => output.Split('\n').Select(l => l.TrimEnd('\r'));

        // The line Pharaoh says its runtime in, after one frame of a synthetic NES image.
        private static (string Line, string Output) RuntimeLine(string? crystal)
        {
            var (exit, output) = Pharaoh(crystal);
            Assert.True(exit == 0, $"Pharaoh exited {exit}: {output}");
            return (Lines(output).Single(l => l.StartsWith("[ROM] Runtime: ", StringComparison.Ordinal)), output);
        }

        [Fact]
        public void The_suite_s_binding_is_the_platform_library_s() => Assert.Contains(", frontend API 1.", CrystalContract.Report);

        [Fact]
        public void With_the_variable_unset_Pharaoh_runs_on_the_platform_library_and_says_nothing_of_it()
        {
            var (line, output) = RuntimeLine(null);
            Assert.Contains("frontend API 1.", line);
            Assert.DoesNotContain("is ignored", output);
        }

        // The line goes to the host's sink and the error log, so it is read from the binding rather than from a program's output.
        [Theory]
        [InlineData("0")]
        [InlineData("csharp")]
        public void A_leftover_variable_is_ignored_with_one_line(string value) =>
            Assert.StartsWith($"{CrystalContract.Variable}={value} is ignored: the frontend API's C# implementation was deleted, and ", CrystalContract.Ignored(value));

        [Theory]
        [InlineData(null)]
        [InlineData("")]
        [InlineData("1")]
        public void An_unset_variable_or_1_is_said_nothing_of(string? value) => Assert.Null(CrystalContract.Ignored(value));

        // Since 2026-11-06 the C# shell's variable stops nothing: the run is the run with it unset - see EmuSen_Backend.md §40.7.
        [Fact]
        public void The_retired_csharp_shell_s_variable_no_longer_stops_a_run()
        {
            var (exit, output) = Pharaoh(null, "0");
            Assert.True(exit == 0, $"Pharaoh exited {exit}: {output}");
            Assert.DoesNotContain("[ERROR]", output);
        }
    }
}
