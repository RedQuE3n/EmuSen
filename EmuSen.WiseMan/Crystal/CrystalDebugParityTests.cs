using System.Diagnostics;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The Rust debugger held to the C# one: each script under DebugScripts run by both, every answer compared line for line - see EmuSen_Backend.md §29.30.
    [Collection(TestCollections.ProcessGlobals)]
    public partial class CrystalDebugParityTests : IDisposable
    {
        // A tool other than the one beside the tests, for a seeded fault's build.
        public const string ToolVariable = "EMUSEN_CRYSTAL_DEBUG_TOOL";

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDebugParity_" + Guid.NewGuid().ToString("N"));

        public CrystalDebugParityTests()
        {
            Directory.CreateDirectory(_root);
            EmuSen.Galaxia.Library.DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            EmuSen.Galaxia.Library.DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // An image written under the test's folder, its path for both scripts.
        private string Write(string name, byte[] image)
        {
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, image);
            return path;
        }

        private static string TestCore => EmuSen.WiseMan.Cores.CoreAdapterTests.LibraryPath("v1_test_core");

        private static byte[] TestImage(params byte[] payload) => EmuSen.WiseMan.Cores.CoreAdapterTests.Image(payload);

        private static string Tool => Environment.GetEnvironmentVariable(ToolVariable) is { Length: > 0 } tool ? tool
            : Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "emusen-crystal-debug.exe" : "emusen-crystal-debug");

        private static string ScriptPath(string name) => Path.Combine(AppContext.BaseDirectory, "Crystal", "DebugScripts", name + ".script");

        // The Rust tool's answers to a script, from a process of its own.
        private static string Rust(string path, IReadOnlyDictionary<string, string> values)
        {
            var start = new ProcessStartInfo(Tool) { RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false };
            start.ArgumentList.Add(path);
            foreach (var (name, value) in values) start.ArgumentList.Add($"{name}={value}");
            using var process = Process.Start(start)!;
            string output = process.StandardOutput.ReadToEnd();
            string error = process.StandardError.ReadToEnd();
            process.WaitForExit();
            Assert.True(process.ExitCode == 0, $"{Tool} exited {process.ExitCode}: {error}");
            return output;
        }

        // Both answers to one script, the first line that differs named.
        private static void Same(string name, IReadOnlyDictionary<string, string>? values = null)
        {
            values ??= new Dictionary<string, string>();
            string path = ScriptPath(name);
            // Compared as the bytes a client reads, so a lone surrogate is the replacement character on both sides.
            string managed = System.Text.Encoding.UTF8.GetString(System.Text.Encoding.UTF8.GetBytes(CrystalDebugScript.Run(File.ReadAllText(path), values)));
            string native = Rust(path, values);
            string[] a = managed.Split('\n'), b = native.Split('\n');
            for (int i = 0; i < Math.Max(a.Length, b.Length); i++)
            {
                string x = i < a.Length ? a[i] : "<none>", y = i < b.Length ? b[i] : "<none>";
                if (x != y) Assert.Fail($"{name}, line {i + 1}\nC#:   {x}\nRust: {y}");
            }
            Assert.True(a.Length > 1, $"{name} answered nothing");
        }

        [Fact]
        public void The_expression_language_answers_alike() => Same("expressions");

        [Fact]
        public void The_contexts_answer_alike() => Same("contexts");

        [Fact]
        public void The_labels_answer_alike() => Same("labels");

        [Fact]
        public void The_breakpoints_answer_alike() => Same("breakpoints");

        [Fact]
        public void The_watches_frame_log_and_freezes_answer_alike() => Same("watches");

        [Fact]
        public void Coverage_and_the_call_stack_answer_alike() => Same("coverage");

        [Fact]
        public void The_cheat_registry_answers_alike() => Same("cheats");

        [Fact]
        public void The_debugger_over_the_test_core_answers_alike() => Same("testcore", new Dictionary<string, string>
        {
            ["CORE"] = Write(Path.GetFileName(TestCore), File.ReadAllBytes(TestCore)),
            ["IMAGE"] = Write("game.tst", TestImage(1, 2, 3, 4, 5, 6, 7, 8)),
            ["HELPER"] = Write("helper.tst", TestImage(0xC2, 9, 8, 7)),
            ["UNLOGGED"] = Write("irq.tst", TestImage(0x16, 1, 2)),
        });
    }
}
