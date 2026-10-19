using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json.Nodes;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Managed;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Discovery's listing as JSON from the C# host and from the Rust core host, equal over the build's tree and trees with a missing, altered and unapproved library - see EmuSen_Backend.md §27.6.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoreHostParityTests : IDisposable
    {
        // A tool other than the one beside the tests, for a seeded fault's build.
        public const string ToolVariable = "EMUSEN_CORE_HOST_TOOL";

        // A published installation's library folder, compared when set.
        public const string PublishedVariable = "EMUSEN_PUBLISHED_TREE";

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCoreHostParity_" + Guid.NewGuid().ToString("N"));

        public CoreHostParityTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }

        private static string Tool => Environment.GetEnvironmentVariable(ToolVariable) is { Length: > 0 } tool ? tool
            : Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "emusen-core-host.exe" : "emusen-core-host");

        // The C# host's listing, as the frontend API's cores and engines answer it, with every core opened first when asked.
        private static JsonObject CSharp(string[] directories, bool open)
        {
            CoreDiscovery.UseDirectories(directories);
            if (open) foreach (DiscoveredCore core in CoreDiscovery.Found) core.Open();
            return new JsonObject
            {
                ["cores"] = JsonNode.Parse(ManagedRuntime.CoresJson()),
                ["engines"] = JsonNode.Parse(ManagedRuntime.EnginesJson()),
            };
        }

        // The Rust core host's listing of the same directories, from its tool in a process of its own.
        private static JsonObject Rust(string[] directories, bool open, params string[] approvals)
        {
            var start = new ProcessStartInfo(Tool) { RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false };
            start.Environment.Remove(CoreDiscovery.DevelopmentVariable);
            start.ArgumentList.Add("list");
            if (open) start.ArgumentList.Add("--open");
            foreach (string a in approvals) { start.ArgumentList.Add("--approve"); start.ArgumentList.Add(a); }
            foreach (string d in directories) start.ArgumentList.Add(d);
            using var process = Process.Start(start)!;
            string output = process.StandardOutput.ReadToEnd();
            string error = process.StandardError.ReadToEnd();
            process.WaitForExit();
            Assert.True(process.ExitCode == 0, $"{Tool} exited {process.ExitCode}: {error}");
            return (JsonObject)JsonNode.Parse(output)!;
        }

        private static void AssertEqual(JsonNode expected, JsonNode actual, string what) =>
            Assert.True(JsonNode.DeepEquals(expected, actual), $"{what}\nC#:   {expected.ToJsonString()}\nRust: {actual.ToJsonString()}");

        // Both listings of one tree, as listed and after every core is opened, equal value for value.
        private static void BothEqual(string[] directories, string what)
        {
            AssertEqual(CSharp(directories, open: false), Rust(directories, open: false), $"{what}, as listed");
            AssertEqual(CSharp(directories, open: true), Rust(directories, open: true), $"{what}, opened");
        }

        // A test core with its sidecar written in a build folder, as the build writes it, then copied with it into dir.
        private string Install(string name, string dir, string? file = null)
        {
            string build = Path.Combine(_root, "build", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(build);
            Directory.CreateDirectory(dir);
            string source = CoreAdapterTests.LibraryPath(name);
            string built = Path.Combine(build, file ?? Path.GetFileName(source));
            File.Copy(source, built);
            CoreSidecar.Write(built);
            string copy = Path.Combine(dir, Path.GetFileName(built));
            File.Copy(built, copy);
            File.Copy(CoreSidecar.PathFor(built), CoreSidecar.PathFor(copy));
            return copy;
        }

        // The installation's manifest naming these libraries, in sha256sum's format.
        private static void Manifest(string dir, params string[] libraries) =>
            File.WriteAllText(Path.Combine(dir, "cores.manifest"), string.Concat(libraries.Select(l => $"{CoreSidecar.HashOf(l)}  {Path.GetFileName(l)}\n")));

        private string GoodTree(string name, out string full, out string plain)
        {
            string dir = Path.Combine(_root, name);
            full = Install("v1_test_core", dir);
            plain = Install("v1_plain_core", dir);
            Manifest(dir, full, plain);
            return dir;
        }

        [Fact]
        public void The_builds_tree_lists_the_same_from_both_hosts()
        {
            string dir = AppContext.BaseDirectory;
            Assert.True(File.Exists(Path.Combine(dir, "cores.manifest")), "the build writes cores.manifest beside the cores");
            var listed = CSharp(new[] { dir }, open: false);
            Assert.True(listed["cores"]!.AsArray().Count >= 6, listed.ToJsonString());
            BothEqual(new[] { dir }, "the build's tree");
            Assert.All(CSharp(new[] { dir }, open: true)["cores"]!.AsArray(), c => Assert.Contains("core ABI 1.", (string)c!["report"]!));
        }

        [Fact]
        public void A_generated_tree_lists_the_same_from_both_hosts()
        {
            string dir = GoodTree("good", out _, out _);
            BothEqual(new[] { dir }, "a tree of two approved cores");
            BothEqual(new[] { dir, Path.Combine(dir, "cores"), dir }, "the same with a directory that is not there and one named twice");
        }

        [Fact]
        public void A_missing_library_lists_the_same_from_both_hosts()
        {
            string dir = GoodTree("missing", out _, out _);
            string gone = Install("v1_test_core", dir, OperatingSystem.IsWindows() ? "gone.dll" : "libgone.so");
            var sidecar = (JsonObject)JsonNode.Parse(File.ReadAllText(CoreSidecar.PathFor(gone)))!;
            sidecar["info"]!["id"] = "v1-gone-core";
            File.WriteAllText(CoreSidecar.PathFor(gone), sidecar.ToJsonString());
            File.Delete(gone);
            BothEqual(new[] { dir }, "a tree with a missing library");
            Assert.Contains(CSharp(new[] { dir }, open: true)["cores"]!.AsArray(), c => ((string)c!["report"]!).EndsWith("is missing beside its sidecar", StringComparison.Ordinal));
        }

        [Fact]
        public void An_altered_library_lists_the_same_from_both_hosts()
        {
            string dir = GoodTree("altered", out _, out string plain);
            byte[] bytes = File.ReadAllBytes(plain);
            bytes[^1] ^= 0xFF;
            File.WriteAllBytes(plain, bytes);
            BothEqual(new[] { dir }, "a tree with a library changed after its sidecar was written");
            Assert.Contains(CSharp(new[] { dir }, open: true)["cores"]!.AsArray(), c => ((string)c!["report"]!).Contains("its SHA-256 differs", StringComparison.Ordinal));
        }

        // The one designed difference: the C# host has no manifest, so it opens what the Rust refuses; closes at B7 (EmuSen_Backend.md §27.9 Q1).
        [Fact]
        public void An_unapproved_library_differs_only_in_its_report_once_opened()
        {
            string dir = Path.Combine(_root, "unapproved");
            string full = Install("v1_test_core", dir);
            string plain = Install("v1_plain_core", dir);
            Manifest(dir, full);
            AssertEqual(CSharp(new[] { dir }, open: false), Rust(new[] { dir }, open: false), "an unapproved library, as listed");

            JsonObject expected = CSharp(new[] { dir }, open: true);
            string words = $"{Path.GetFileName(plain)} is in no cores manifest and has not been approved; it was not loaded";
            int changed = 0;
            foreach (JsonNode? c in expected["cores"]!.AsArray())
                if ((string)c!["library"]! == plain) { Assert.EndsWith("core ABI 1.14", (string)c["report"]!); c["report"] = words; changed++; }
            foreach (JsonNode? e in expected["engines"]!.AsArray())
                if ((string)e!["id"]! == "v1-plain-core") { e["report"] = words; changed++; }
            Assert.Equal(2, changed);
            AssertEqual(expected, Rust(new[] { dir }, open: true), "an unapproved library, opened: equal but for its report");

            AssertEqual(CSharp(new[] { dir }, open: true), Rust(new[] { dir }, open: true, CoreSidecar.HashOf(plain)), "the same library approved by its hash");
        }

        [Fact]
        public void A_development_core_and_a_repeated_id_list_the_same_from_both_hosts()
        {
            string first = Path.Combine(_root, "first");
            string second = Path.Combine(_root, "second");
            string a = Install("v1_test_core", first);
            string b = Install("v1_test_core", second, OperatingSystem.IsWindows() ? "again.dll" : "libagain.so");
            string dev = Install("v1_plain_core", second);
            var sidecar = (JsonObject)JsonNode.Parse(File.ReadAllText(CoreSidecar.PathFor(dev)))!;
            sidecar["development"] = true;
            File.WriteAllText(CoreSidecar.PathFor(dev), sidecar.ToJsonString());
            string two = Install("v1_plain_core", second, OperatingSystem.IsWindows() ? "two.dll" : "libtwo.so");
            var major = (JsonObject)JsonNode.Parse(File.ReadAllText(CoreSidecar.PathFor(two)))!;
            major["abi"] = "2.0";
            major["info"]!["id"] = "v1-major-two-core";
            File.WriteAllText(CoreSidecar.PathFor(two), major.ToJsonString());
            Manifest(first, a);
            Manifest(second, b, dev, two);
            BothEqual(new[] { first, second }, "a repeated id, a core in development and a sidecar of another major");
        }

        [Fact]
        public void A_sidecar_spelt_another_way_lists_the_same_from_both_hosts()
        {
            string dir = GoodTree("spelt", out string full, out _);
            var sidecar = (JsonObject)JsonNode.Parse(File.ReadAllText(CoreSidecar.PathFor(full)))!;
            sidecar["sha256"] = ((string)sidecar["sha256"]!).ToUpperInvariant();
            var info = (JsonObject)sidecar["info"]!;
            var reversed = new JsonObject();
            foreach (var (key, value) in info.Reverse().ToList()) reversed[key] = value?.DeepClone();
            sidecar["info"] = reversed;
            File.WriteAllText(CoreSidecar.PathFor(full), sidecar.ToJsonString());
            BothEqual(new[] { dir }, "a sidecar whose hash is in capitals and whose info's members are in another order");
            Assert.All(CSharp(new[] { dir }, open: true)["cores"]!.AsArray(), c => Assert.EndsWith("core ABI 1.14", (string)c!["report"]!));
        }

        [Fact]
        public void A_published_tree_lists_the_same_from_both_hosts_when_one_is_named()
        {
            string? tree = Environment.GetEnvironmentVariable(PublishedVariable);
            if (string.IsNullOrEmpty(tree)) return;
            Assert.True(File.Exists(Path.Combine(tree, "cores.manifest")), $"{tree} holds no cores.manifest");
            BothEqual(new[] { tree }, $"the published tree {tree}");
        }
    }
}
