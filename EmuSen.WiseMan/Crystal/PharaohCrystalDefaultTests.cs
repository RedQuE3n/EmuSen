using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.Crystal
{
    // Pharaoh, run as its own process, is on the Rust Crystal by default and on the C# one variable away - see EmuSen_Backend.md §29.27.
    public class PharaohCrystalDefaultTests
    {
        // The line Pharaoh says its runtime in, after one frame of a synthetic NES image, with the variable unset or set.
        private static string RuntimeLine(string? variable)
        {
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenPharaohDefault", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(dir);
            try
            {
                string rom = Path.Combine(dir, "game.nes");
                File.WriteAllBytes(rom, SyntheticNesRom.Build());
                var start = new ProcessStartInfo("dotnet") { RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false };
                foreach (string a in new[] { Path.Combine(AppContext.BaseDirectory, "EmuSen.Pharaoh.dll"), rom, "1", "--nobattery" }) start.ArgumentList.Add(a);
                start.Environment.Remove("EMUSEN_CRYSTAL_NATIVE");
                if (variable is not null) start.Environment["EMUSEN_CRYSTAL_NATIVE"] = variable;
                using var process = Process.Start(start)!;
                var errors = process.StandardError.ReadToEndAsync();
                string output = process.StandardOutput.ReadToEnd();
                process.WaitForExit();
                Assert.True(process.ExitCode == 0, $"Pharaoh exited {process.ExitCode}: {output}{errors.Result}");
                return output.Split('\n').Select(l => l.TrimEnd('\r')).Single(l => l.StartsWith("[ROM] Runtime: ", StringComparison.Ordinal));
            }
            finally
            {
                Directory.Delete(dir, recursive: true);
            }
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
    }
}
