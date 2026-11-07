using System;
using System.Diagnostics;
using System.IO;
using System.Security.Cryptography;
using System.Text.Json.Nodes;

namespace EmuSen.WiseMan.Fixtures
{
    // A core library's sidecar, written by the conformance kit's runner as the build writes them, and the trust files' names - see EmuSen_CoreAPI.md §7.1 and EmuSen_Backend.md §40.10.
    public static class CoreSidecars
    {
        public const string Suffix = ".core.json";
        public const string ManifestName = "cores.manifest";
        public const string ApprovalsName = "cores.approved";

        public static string PathFor(string library) => library + Suffix;

        // A library the build put beside the test assemblies, by its crate's name.
        public static string LibraryPath(string name) => Path.Combine(AppContext.BaseDirectory,
            OperatingSystem.IsWindows() ? $"{name}.dll" : OperatingSystem.IsMacOS() ? $"lib{name}.dylib" : $"lib{name}.so");

        public static string HashOf(string path)
        {
            using var file = File.OpenRead(path);
            return Convert.ToHexStringLower(SHA256.HashData(file));
        }

        // The runner the build made under EmuSen/obj, which loads the library once and writes LIB.core.json beside it.
        public static string Runner => Path.Combine(ShaderCompiler.RepositoryRoot, "EmuSen", "obj", "emusen-core-conform", "release",
            OperatingSystem.IsWindows() ? "emusen-core-conform.exe" : "emusen-core-conform");

        // Whether the build put a core beside the tests with its sidecar, in words when it did not.
        public static (bool Built, string Report) Beside(string name)
        {
            string library = LibraryPath(name);
            return File.Exists(library) && File.Exists(PathFor(library)) ? (true, "") : (false, $"{Path.GetFileName(library)} or its sidecar is not beside the tests");
        }

        public static void Write(string library, bool development = false)
        {
            var start = new ProcessStartInfo(Runner) { RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false };
            start.ArgumentList.Add("--sidecar");
            start.ArgumentList.Add(library);
            if (development) start.ArgumentList.Add("--development");
            using Process run = Process.Start(start) ?? throw new InvalidOperationException($"{Runner} did not start");
            string said = run.StandardError.ReadToEnd() + run.StandardOutput.ReadToEnd();
            run.WaitForExit();
            if (run.ExitCode != 0 || !File.Exists(PathFor(library))) throw new InvalidOperationException($"no sidecar for {library}: {said}");
        }

        // The library a sidecar of version 1 names, beside it; null for anything else.
        public static string? LibraryOf(string sidecar)
        {
            try
            {
                if (JsonNode.Parse(File.ReadAllText(sidecar)) is not JsonObject doc || doc["sidecar"]?.GetValue<int>() != 1) return null;
                string? name = doc["library"]?.GetValue<string>();
                return name is null || name != Path.GetFileName(name) ? null : Path.Combine(Path.GetDirectoryName(sidecar)!, name);
            }
            catch (Exception e) when (e is System.Text.Json.JsonException or IOException or InvalidOperationException or FormatException) { return null; }
        }
    }
}
