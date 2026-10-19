using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;

namespace EmuSen.WiseMan.Fixtures
{
    // One of the programs beside the tests run as its own process on a synthetic image, with the switches' variables as given and no display - see EmuSen_Backend.md §29.27 and §29.28.
    public static class ProgramRun
    {
        public const string CrystalVariable = "EMUSEN_CRYSTAL_NATIVE", DianaShVariable = "EMUSEN_DIANASH_NATIVE";

        // The program's exit code and its standard output; a variable given null is unset, and a run past 120 s is killed.
        public static (int Exit, string Output) Run(string assembly, string imageName, byte[] image, IReadOnlyDictionary<string, string?> variables, params string[] arguments)
        {
            string dir = Path.Combine(Path.GetTempPath(), "EmuSenProgramRun", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(dir);
            try
            {
                string game = Path.Combine(dir, imageName);
                File.WriteAllBytes(game, image);
                var start = new ProcessStartInfo("dotnet") { RedirectStandardOutput = true, RedirectStandardError = true, RedirectStandardInput = true, UseShellExecute = false };
                start.ArgumentList.Add(Path.Combine(AppContext.BaseDirectory, assembly));
                foreach (string a in arguments) start.ArgumentList.Add(a.Replace("{game}", game));
                foreach (string name in new[] { CrystalVariable, DianaShVariable, "DISPLAY", "WAYLAND_DISPLAY" }) start.Environment.Remove(name);
                foreach (var (name, value) in variables) if (value is not null) start.Environment[name] = value;
                using var process = Process.Start(start)!;
                process.StandardInput.Close();
                var errors = process.StandardError.ReadToEndAsync();
                var output = process.StandardOutput.ReadToEndAsync();
                if (!process.WaitForExit(120_000)) process.Kill(entireProcessTree: true);
                process.WaitForExit();
                return (process.ExitCode, output.Result + errors.Result);
            }
            finally
            {
                Directory.Delete(dir, recursive: true);
            }
        }
    }
}
