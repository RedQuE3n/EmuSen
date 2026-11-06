using System;
using System.Diagnostics;
using System.Linq;
using System.IO;
using System.Security.Cryptography;
using System.Threading.Tasks;
using EmuSen.Serenity.Native;
using EmuSen.Serenity.Slang;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.Serenity
{
    // Shaderc loaded by the platform library from the copy the program ships, and by the tests' own binding on a distribution Silk.NET's resolver does not name - see EmuSen_RustPlatform.md §33 and EmuSen_Backend.md §40.6.
    public class SlangShadercLoadTests : IDisposable
    {
        private const string MarkVariable = "EMUSEN_SHADERC_LOAD_MARK";
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenShadercLoad", Guid.NewGuid().ToString("N"));

        private const string Fragment = """
            #version 450
            layout(location = 0) out vec4 FragColor;
            void main() { FragColor = vec4(1.0); }
            """;

        private const string Compute = """
            #version 450
            layout(local_size_x = 1) in;
            void main() { }
            """;

        public SlangShadercLoadTests() => Directory.CreateDirectory(_root);

        public void Dispose()
        {
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        // The copy beside the program, or in its runtime's native folder, is the one the library compiles with: its identity carries that file's SHA-256.
        [Fact]
        public void The_library_compiles_with_the_shaderc_the_program_ships()
        {
            string file = OperatingSystem.IsWindows() ? "shaderc_shared.dll" : OperatingSystem.IsMacOS() ? "libshaderc_shared.dylib" : "libshaderc_shared.so";
            string? shipped = new[] { Path.Combine(AppContext.BaseDirectory, file), Path.Combine(AppContext.BaseDirectory, "runtimes", System.Runtime.InteropServices.RuntimeInformation.RuntimeIdentifier, "native", file) }
                .FirstOrDefault(File.Exists) ?? Directory.EnumerateFiles(Path.Combine(AppContext.BaseDirectory, "runtimes"), file, SearchOption.AllDirectories).FirstOrDefault(p => p.Contains(OperatingSystem.IsLinux() ? "linux-x64" : "x64", StringComparison.Ordinal));
            Assert.NotNull(shipped);
            Assert.NotEmpty(SlangCompiler.Compile(Fragment, SlangStage.Fragment, "fragment.slang"));
            Assert.StartsWith($"shaderc {Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(shipped!)))}; ", SerenityNative.Identity());
        }

        // A process loads Shaderc once, so only a process started as Ubuntu's runner starts is an honest test of that runner.
        [Fact]
        public void On_a_distribution_silk_does_not_name_both_compile_with_the_copy_beside_the_program()
        {
            if (!OperatingSystem.IsLinux()) return;
            string mark = Path.Combine(_root, "loaded.txt");
            var start = new ProcessStartInfo("dotnet") { RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false };
            foreach (string argument in new[] { "test", typeof(SlangShadercLoadTests).Assembly.Location, "--filter", $"FullyQualifiedName={typeof(SlangShadercLoadTests).FullName}.{nameof(ShadercInAProcessThatIsUbuntu)}" })
                start.ArgumentList.Add(argument);
            start.Environment["DOTNET_RUNTIME_ID"] = "ubuntu.24.04-x64";
            start.Environment[MarkVariable] = mark;

            using Process child = Process.Start(start)!;
            Task<string> output = child.StandardOutput.ReadToEndAsync();
            Task<string> errors = child.StandardError.ReadToEndAsync();
            Assert.True(child.WaitForExit(240_000), "the child test run did not finish");
            Assert.True(child.ExitCode == 0, output.Result + errors.Result);
            Assert.True(File.Exists(mark), "the child ran nothing:\n" + output.Result);
            Assert.Equal(SerenityNative.Identity(), File.ReadAllText(mark).Trim());
        }

        // Run only by the test above, in its own process; it asserts nothing unless that process was started as Ubuntu.
        [Fact]
        public void ShadercInAProcessThatIsUbuntu()
        {
            if (Environment.GetEnvironmentVariable(MarkVariable) is not { } mark) return;

            Assert.NotEmpty(SlangCompiler.Compile(Fragment, SlangStage.Fragment, "fragment.slang"));
            Assert.NotEmpty(ShaderCompiler.Compile(Compute, "compute.comp"));
            File.WriteAllText(mark, SerenityNative.Identity());
        }
    }
}
