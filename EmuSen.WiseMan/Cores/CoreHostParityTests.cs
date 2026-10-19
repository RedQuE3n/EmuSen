using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json.Nodes;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Managed;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Discovery's listing as JSON from the C# host and from the Rust core host, equal over the build's tree, faulty trees and the player's directory - see EmuSen_Backend.md §27.6, §29.40.9.
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
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UsePlayerDirectories(null);
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }

        private static string Tool => Environment.GetEnvironmentVariable(ToolVariable) is { Length: > 0 } tool ? tool
            : Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "emusen-core-host.exe" : "emusen-core-host");

        // The C# host's listing, as the frontend API's cores and engines answer it, with every core opened first when asked.
        private static JsonObject CSharp(string[] directories, bool open, params string[] player)
        {
            CoreDiscovery.UseDirectories(directories);
            CoreDiscovery.UsePlayerDirectories(player);
            if (open) foreach (DiscoveredCore core in CoreDiscovery.Found) core.Open();
            return new JsonObject
            {
                ["cores"] = JsonNode.Parse(ManagedRuntime.CoresJson()),
                ["engines"] = JsonNode.Parse(ManagedRuntime.EnginesJson()),
            };
        }

        // The Rust core host's listing of the same directories, from its tool in a process of its own.
        private static JsonObject Rust(string[] directories, bool open, params string[] player)
        {
            var start = new ProcessStartInfo(Tool) { RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false };
            start.Environment.Remove(CoreDiscovery.DevelopmentVariable);
            start.ArgumentList.Add("list");
            if (open) start.ArgumentList.Add("--open");
            start.ArgumentList.Add("--approvals");
            start.ArgumentList.Add(CoreTrust.ApprovalsPath);
            foreach (string d in player) { start.ArgumentList.Add("--player"); start.ArgumentList.Add(d); }
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
        private static void BothEqual(string[] directories, string what, params string[] player)
        {
            AssertEqual(CSharp(directories, open: false, player), Rust(directories, open: false, player), $"{what}, as listed");
            AssertEqual(CSharp(directories, open: true, player), Rust(directories, open: true, player), $"{what}, opened");
        }

        // The player's approvals' file in the test's own config directory.
        private static void Approve(params string[] libraries)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(CoreTrust.ApprovalsPath)!);
            File.WriteAllText(CoreTrust.ApprovalsPath, "# approved by the test\n" + string.Concat(libraries.Select(l => $"{CoreSidecar.HashOf(l)}  {Path.GetFileName(l)}\n")) + "not a line of hashes\n");
        }

        private static string Lib(string stem) => OperatingSystem.IsWindows() ? $"{stem}.dll" : OperatingSystem.IsMacOS() ? $"lib{stem}.dylib" : $"lib{stem}.so";

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
            Assert.False(File.Exists(CoreTrust.ApprovalsPath), "the build's cores load with no approval at all");
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

        // §27.11's pinned difference, closed: both hosts refuse a library in no manifest until the player approves it (EmuSen_Backend.md §29.40.6).
        [Fact]
        public void An_unapproved_library_is_refused_by_both_hosts_until_approved()
        {
            string dir = Path.Combine(_root, "unapproved");
            string full = Install("v1_test_core", dir);
            string plain = Install("v1_plain_core", dir);
            Manifest(dir, full);
            BothEqual(new[] { dir }, "an unapproved library");
            string words = $"{Path.GetFileName(plain)} is in no cores manifest and has not been approved; it was not loaded";
            var opened = CSharp(new[] { dir }, open: true);
            Assert.Contains(opened["cores"]!.AsArray(), c => (string)c!["report"]! == words && (string)c["trust"]! == "needs_approval");
            Approve(plain);
            BothEqual(new[] { dir }, "the same library approved by its hash");
            Assert.All(CSharp(new[] { dir }, open: true)["cores"]!.AsArray(), c => Assert.EndsWith("core ABI 1.15", (string)c!["report"]!));
        }

        [Fact]
        public void The_players_directory_lists_the_same_from_both_hosts()
        {
            string dir = Path.Combine(_root, "installed");
            Manifest(dir, Install("v1_test_core", dir));
            string player = Path.Combine(_root, "Cores");
            Install("v1_test_core", player, Lib("shadow"));
            string own = Install("v1_plain_core", player, Lib("own"));
            Manifest(player, own);
            BothEqual(new[] { dir }, "a player's directory with a project core's id, an unapproved core and a manifest of its own", player);
            var listed = CSharp(new[] { dir }, open: false, player);
            Assert.Equal(new[] { "manifest", "needs_approval" }, listed["cores"]!.AsArray().Select(c => (string)c!["trust"]!));
            Assert.Equal("player", (string)listed["cores"]![1]!["directory"]!);
            Approve(own);
            BothEqual(new[] { dir }, "the player's core approved", player);
            Assert.EndsWith("core ABI 1.15", (string)CSharp(new[] { dir }, open: true, player)["cores"]![1]!["report"]!);
        }

        [Fact]
        public void Libraries_without_a_sidecar_and_with_a_stale_one_list_the_same_from_both_hosts()
        {
            string player = Path.Combine(_root, "Cores");
            Directory.CreateDirectory(player);
            File.Copy(CoreAdapterTests.LibraryPath("v1_plain_core"), Path.Combine(player, Lib("bare")));
            string stale = Install("v1_test_core", player, Lib("stale"));
            File.AppendAllText(stale, "\0");
            File.WriteAllText(Path.Combine(player, "notes.txt"), "not a library");
            BothEqual(Array.Empty<string>(), "a library with no sidecar and one whose sidecar is stale", player);
            var cores = CSharp(Array.Empty<string>(), open: false, player)["cores"]!.AsArray();
            Assert.Equal(2, cores.Count);
            Assert.All(cores, c => { Assert.Equal("not_read", (string)c!["trust"]!); Assert.Equal("not yet read", (string)c["report"]!); Assert.Null(c["sidecar"]); });
            Assert.Equal(CoreSidecar.HashOf(stale), (string)cores[1]!["sha256"]!);
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
            Assert.All(CSharp(new[] { dir }, open: true)["cores"]!.AsArray(), c => Assert.EndsWith("core ABI 1.15", (string)c!["report"]!));
        }

        [Fact]
        public void A_published_tree_lists_the_same_from_both_hosts_when_one_is_named()
        {
            string? tree = Environment.GetEnvironmentVariable(PublishedVariable);
            if (string.IsNullOrEmpty(tree)) return;
            Assert.True(File.Exists(Path.Combine(tree, "cores.manifest")), $"{tree} holds no cores.manifest");
            BothEqual(new[] { tree }, $"the published tree {tree}");
            Assert.False(File.Exists(CoreTrust.ApprovalsPath), "the published cores load with no approval at all");
            Assert.All(CSharp(new[] { tree }, open: true)["cores"]!.AsArray(), c => { Assert.Equal("manifest", (string)c!["trust"]!); Assert.Contains("core ABI 1.", (string)c["report"]!); });
        }
    }
}
