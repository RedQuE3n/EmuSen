using System;
using System.Diagnostics;
using System.IO;
using System.Threading.Tasks;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Galaxia
{
    // The default log folder in a home with no .config: absolute, never EmuSen/Logs under the working folder - see EmuSen_Galaxia.md §3.6.
    [Collection(TestCollections.ProcessGlobals)]
    public class LogFolderWithoutConfigTests : IDisposable
    {
        private const string MarkVariable = "EMUSEN_TEST_EMPTY_HOME_MARK";

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenEmptyHome_" + Guid.NewGuid().ToString("N"));

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // The library takes .NET's folder once, at its first call, so only a process started with the empty home shows what it is handed.
        [Fact]
        public void A_process_whose_home_has_no_config_folder_logs_under_that_home()
        {
            // Only Linux takes .NET's application-data folder from the home's .config; Windows and macOS ask the system for a folder of their own.
            if (!OperatingSystem.IsLinux()) return;
            string home = Path.Combine(_root, "home"), cwd = Path.Combine(_root, "cwd"), mark = Path.Combine(_root, "mark.txt");
            Directory.CreateDirectory(home);
            Directory.CreateDirectory(cwd);
            var start = new ProcessStartInfo("dotnet") { RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false, WorkingDirectory = cwd };
            foreach (string argument in new[] { "test", typeof(LogFolderWithoutConfigTests).Assembly.Location, "--filter", $"FullyQualifiedName={typeof(LogFolderWithoutConfigTests).FullName}.{nameof(TheLogFolderInAProcessWithAnEmptyHome)}" })
                start.ArgumentList.Add(argument);
            start.Environment["HOME"] = home;
            start.Environment.Remove("XDG_CONFIG_HOME");
            // The child measures both halves itself, so a parent run on the C# half must not switch the library off there too.
            start.Environment.Remove("EMUSEN_GALAXIA_NATIVE");
            start.Environment["DOTNET_CLI_TELEMETRY_OPTOUT"] = "1";
            start.Environment["DOTNET_NOLOGO"] = "1";
            start.Environment["DOTNET_SKIP_FIRST_TIME_EXPERIENCE"] = "1";
            start.Environment[MarkVariable] = mark;

            using Process child = Process.Start(start)!;
            Task<string> output = child.StandardOutput.ReadToEndAsync();
            Task<string> errors = child.StandardError.ReadToEndAsync();
            Assert.True(child.WaitForExit(240_000), "the child test run did not finish");
            Assert.True(child.ExitCode == 0, output.Result + errors.Result);
            Assert.True(File.Exists(mark), "the child ran nothing:\n" + output.Result);

            string[] found = File.ReadAllLines(mark);
            string logs = Path.Combine(home, ".config", "EmuSen", "Logs");
            Assert.False(Directory.Exists(Path.Combine(cwd, "EmuSen")), "a folder was made under the working folder: " + string.Join(" | ", found));
            Assert.Equal("True", found[0]);
            Assert.Equal(logs, found[1]);
            Assert.StartsWith(logs + Path.DirectorySeparatorChar, found[2]);
            Assert.Equal(logs, found[3]);
            Assert.StartsWith(logs + Path.DirectorySeparatorChar, found[4]);
            Assert.Equal(Path.Combine(home, ".config", "EmuSen"), found[5]);
        }

        // Run only by the test above, in its own process; it does nothing unless that process set the mark.
        [Fact]
        public void TheLogFolderInAProcessWithAnEmptyHome()
        {
            if (Environment.GetEnvironmentVariable(MarkVariable) is not { } mark) return;

            // The test host chooses its own working folder, so the child moves to the one the parent inspects.
            string? configWas = ConfigStore.OverrideDirectory, cwdWas = Environment.CurrentDirectory;
            ConfigStore.OverrideDirectory = Path.Combine(Path.GetDirectoryName(mark)!, "config");
            ErrorLog.DirectoryOverride = null;
            Environment.CurrentDirectory = Path.Combine(Path.GetDirectoryName(mark)!, "cwd");
            try
            {
                string native = ErrorLog.DefaultRoot, nativeFile = ErrorLog.Error("test", "written by the library") ?? "";
                string managed = ErrorLog.Managed.DefaultRoot, managedFile = ErrorLog.Managed.Write(DateTime.Now, "ERROR", "test", "written by the C#", null, null) ?? "";
                ConfigStore.OverrideDirectory = null;
                string legacy = ConfigStore.Managed.LegacyDirectory;
                File.WriteAllLines(mark, new[] { GalaxiaNative.Active.ToString(), native, nativeFile, managed, managedFile, legacy });
            }
            finally
            {
                ConfigStore.OverrideDirectory = configWas;
                Environment.CurrentDirectory = cwdWas;
            }
        }
    }
}
