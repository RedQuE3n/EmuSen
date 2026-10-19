using System.Globalization;
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

        // A folder each script's answers are written into, for reading them.
        public const string KeepVariable = "EMUSEN_CRYSTAL_DEBUG_KEEP";

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

        // A built engine's library copied under the test's folder, whose path has no space for a script to split.
        private string Engine(string id)
        {
            string file = OperatingSystem.IsWindows() ? $"{id}.dll" : OperatingSystem.IsMacOS() ? $"lib{id}.dylib" : $"lib{id}.so";
            string built = Path.Combine(AppContext.BaseDirectory, file);
            Assert.True(File.Exists(built), $"{built} is built beside the tests");
            return Write(file, File.ReadAllBytes(built));
        }

        // Code at a base address with a subroutine the loop calls: the bytes, the call's address and the subroutine's.
        private static (byte[] Code, int Call, int Sub) WithCall(int origin, byte[] head, Func<int, byte[]> call, byte[] tail, byte[] sub)
        {
            int at = head.Length;
            byte[] placeholder = call(0);
            int subAt = origin + head.Length + placeholder.Length + tail.Length;
            byte[] code = head.Concat(call(subAt)).Concat(tail).Concat(sub).ToArray();
            return (code, origin + at, subAt);
        }

        // A 16 by 16 PNG, which the encoder takes as a frame.
        private const string Picture = "iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAIAAACQkWg2AAABlklEQVR42hXRURVEIQhFUSMYgQhGMAIRiGCEE8EIRiACEYhABCLMG7/ZrMt1jMEcyGAN9kAHNjgDBnfwBj6IQQ5q0IMxJnMikzXZE53Y5EyY3Mmb+CQmOalJzw8IUxBhCVtQwYQjIFzhCS6EkEIJLR9YzIUs1mIvdGGLs2BxF2/hi1jkoha9PrCZG9mszd7oxjZnw+Zu3sY3sclNbXp/QJmKKEvZiiqmHAXlKk9xJZRUSmn9gDENMZaxDTXMOAbGNZ7hRhhplNH2gcM8yGEd9kEPdjgHDvfwDn6IQx7q0OcD/wK/Sr4jv9hfkG/1N/x/Fx44BCQU9Pc94zIvclmXfdGLXc79j9/Lu/glLnmpS98PPOZDHuuxH/qwx3n/5ffxHv6IRz7q0e8DznTEWc521DHn+D/KdZ7jTjjplNP+gWAGEqxgBxpYcOIf/AYv8CCCDCro+EAyE0lWshNNLDn5P/MmL/Ekkkwq6fxAMQspVrELLaw49S/lFq/wIoosquj6QDMbaVazG22sOf2v8Dav8SaabKrp5geIAnAQC3NfwAAAAABJRU5ErkJggg==";

        [Fact]
        public void The_recorder_answers_alike()
        {
            string picture = Write("picture.png", Convert.FromBase64String(Picture));
            string managed = Directory.CreateDirectory(Path.Combine(_root, "cs")).FullName, native = Directory.CreateDirectory(Path.Combine(_root, "rs")).FullName;
            Same("recorder", new Dictionary<string, string> { ["BASE"] = managed, ["PNG"] = picture }, new Dictionary<string, string> { ["BASE"] = native, ["PNG"] = picture });
            // The archives' entries, each a frame's picture, are the same files.
            static Dictionary<string, string> Entries(string root) =>
                Directory.EnumerateFiles(root, "frames.zip", SearchOption.AllDirectories).SelectMany(z =>
                {
                    using var zip = System.IO.Compression.ZipFile.OpenRead(z);
                    return zip.Entries.Select(e =>
                    {
                        using var s = e.Open();
                        using var m = new MemoryStream();
                        s.CopyTo(m);
                        return (e.FullName, Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(m.ToArray())));
                    }).ToList();
                }).ToDictionary(e => e.Item1, e => e.Item2);
            var a = Entries(managed);
            var b = Entries(native);
            Assert.NotEmpty(a);
            Assert.Equal(a.OrderBy(e => e.Key), b.OrderBy(e => e.Key));
        }

        [Fact]
        public void The_recorder_without_an_encoder_answers_alike()
        {
            string picture = Write("picture.png", Convert.FromBase64String(Picture));
            string managed = Directory.CreateDirectory(Path.Combine(_root, "cs")).FullName, native = Directory.CreateDirectory(Path.Combine(_root, "rs")).FullName;
            string? path = Environment.GetEnvironmentVariable("PATH");
            Environment.SetEnvironmentVariable("PATH", Directory.CreateDirectory(Path.Combine(_root, "nothing")).FullName);
            try { Same("recorder-alone", new Dictionary<string, string> { ["BASE"] = managed, ["PNG"] = picture }, new Dictionary<string, string> { ["BASE"] = native, ["PNG"] = picture }); }
            finally { Environment.SetEnvironmentVariable("PATH", path); }
        }

        [Fact]
        public void The_debugger_over_MoonRT_answers_alike()
        {
            byte[] head =
            {
                0xA2, 0x05, 0xA9, 0x10, 0x8D, 0x00, 0x20, 0xA9, 0x03, 0x8D, 0x05, 0x20, 0x8D, 0x05, 0x20, 0xA9, 0x21, 0x8D, 0x06, 0x20, 0xA9, 0x08, 0x8D, 0x06, 0x20,
                0xA9, 0xC0, 0x8D, 0x00, 0x40, 0xA9, 0x3F, 0x8D, 0x06, 0x20, 0xA9, 0x00, 0x8D, 0x06, 0x20, 0xA9, 0x21, 0x8D, 0x07, 0x20, 0xA9, 0x16, 0x8D, 0x07, 0x20,
                0xA9, 0x00, 0x8D, 0x03, 0x20, 0xA9, 0x40, 0x8D, 0x04, 0x20, 0xA9, 0x05, 0x8D, 0x04, 0x20, 0xA9, 0x41, 0x8D, 0x04, 0x20, 0xA9, 0x80, 0x8D, 0x04, 0x20,
                0xE6, 0x42,
            };
            var (code, call, sub) = WithCall(0x8000, head, s => new byte[] { 0x20, (byte)s, (byte)(s >> 8) }, new byte[] { 0x4C, 0x00, 0x80 }, new byte[] { 0xA5, 0x42, 0x60 });
            Same("moonrt", new Dictionary<string, string>
            {
                ["CORE"] = Engine("moonrt"),
                ["IMAGE"] = Write("game.nes", SyntheticNesRom.Build(patches: new (int, byte[])[] { (0, code) })),
                ["CALL"] = call.ToString(CultureInfo.InvariantCulture),
                ["SUB"] = sub.ToString(CultureInfo.InvariantCulture),
                ["INC"] = (call - 2).ToString(CultureInfo.InvariantCulture),
            });
        }

        [Fact]
        public void The_debugger_over_VenusRT_answers_alike()
        {
            byte[] program = { 0xA9, 0x12, 0x8D, 0x42, 0x00, 0x20, 0x0F, 0x80, 0x80, 0xF6, 0x60 };
            Same("venusrt", new Dictionary<string, string>
            {
                ["CORE"] = Engine("venusrt"),
                ["IMAGE"] = Write("game.sfc", SyntheticRom.Build((5, program))),
            });
        }

        [Fact]
        public void The_debugger_over_MercuryRT_answers_alike()
        {
            var (code, call, sub) = WithCall(0x0150, new byte[] { 0x3E, 0x12, 0xEA, 0x00, 0xC0 }, s => new byte[] { 0xCD, (byte)s, (byte)(s >> 8) }, new byte[] { 0xC3, 0x50, 0x01 }, new byte[] { 0xC9 });
            Same("mercuryrt", new Dictionary<string, string>
            {
                ["CORE"] = Engine("mercuryrt"),
                ["IMAGE"] = Write("game.gb", SyntheticGbRom.Build(patches: new (int, byte[])[] { (0, code) })),
                ["COLOR"] = Write("game.gbc", SyntheticGbRom.Build(cgbFlag: 0x80, patches: new (int, byte[])[] { (0, code) })),
                ["CALL"] = call.ToString(CultureInfo.InvariantCulture),
                ["SUB"] = sub.ToString(CultureInfo.InvariantCulture),
            });
        }

        [Fact]
        public void The_debugger_over_MarsRT_answers_alike()
        {
            Same("marsrt", new Dictionary<string, string>
            {
                ["CORE"] = Engine("marsrt"),
                ["IMAGE"] = Write("game.z64", SyntheticN64Rom.Build(patches: (0, EmuSen.WiseMan.Cores.MarsDebugClaims.CountForever))),
            });
        }

        [Fact]
        public void The_debugger_over_Nephrite_answers_alike() => Same("nephrite", new Dictionary<string, string>
        {
            ["CORE"] = Engine("nephrite"),
            ["IMAGE"] = Write("game.md", SyntheticMdRom.Cartridge()),
        });

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
        private static void Same(string name, IReadOnlyDictionary<string, string>? values = null, IReadOnlyDictionary<string, string>? nativeValues = null)
        {
            values ??= new Dictionary<string, string>();
            string path = ScriptPath(name);
            // Compared as the bytes a client reads, so a lone surrogate is the replacement character on both sides.
            string managed = System.Text.Encoding.UTF8.GetString(System.Text.Encoding.UTF8.GetBytes(CrystalDebugScript.Run(File.ReadAllText(path), values)));
            string native = Rust(path, nativeValues ?? values);
            // A value the Rust is given of its own, as its own folder, is read as the C#'s.
            foreach (var (key, value) in nativeValues ?? new Dictionary<string, string>()) native = native.Replace(value, values[key], StringComparison.Ordinal);
            if (Environment.GetEnvironmentVariable(KeepVariable) is { Length: > 0 } keep) File.WriteAllText(Path.Combine(keep, name + ".answers"), native);
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
