using System;
using System.IO;
using System.Linq;
using System.Text.Json.Nodes;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Discovery from sidecars alone, the checks before and after loading, and the engine row and factory answering a v1 engine with no code naming it - see EmuSen_CoreAPI.md §7.1, §19.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoreDiscoveryTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCoreDiscovery_" + Guid.NewGuid().ToString("N"));

        public CoreDiscoveryTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            CoreDiscovery.UseDirectories(null);
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // A copy of a test core in its own directory, with its sidecar written by loading the copy, as the build step does.
        private string Install(string name, string dir)
        {
            Directory.CreateDirectory(dir);
            string source = CoreAdapterTests.LibraryPath(name);
            string copy = Path.Combine(dir, Path.GetFileName(source));
            File.Copy(source, copy);
            CoreSidecar.Write(copy);
            return copy;
        }

        // The same library under a new directory, its sidecar copied and edited by edit; nothing is loaded.
        private string Place(string library, string dir, Action<JsonObject>? edit = null)
        {
            Directory.CreateDirectory(dir);
            string copy = Path.Combine(dir, Path.GetFileName(library));
            File.Copy(library, copy);
            var sidecar = (JsonObject)JsonNode.Parse(File.ReadAllText(CoreSidecar.PathFor(library)))!;
            edit?.Invoke(sidecar);
            File.WriteAllText(CoreSidecar.PathFor(copy), sidecar.ToJsonString());
            return copy;
        }

        private static void ClaimSnes(JsonObject sidecar)
        {
            var system = (JsonObject)sidecar["info"]!["systems"]![0]!;
            system["id"] = "snes";
            system["extensions"] = new JsonArray(".sfc", ".smc");
        }

        [Fact]
        public void A_sidecar_lists_the_core_without_loading_it()
        {
            string built = Install("v1_test_core", Path.Combine(_root, "build"));
            string listed = Place(built, Path.Combine(_root, "cores"));
            CoreDiscovery.UseDirectories(new[] { Path.Combine(_root, "cores") });

            var found = CoreDiscovery.Found.Single();
            Assert.Equal(("v1-test-core", "V1TestCore (test)", listed), (found.Info.Id, found.EngineName, found.Sidecar.LibraryPath));
            Assert.Equal(CoreSidecar.HashOf(listed), found.Sidecar.Sha256);
            Assert.Single(CoreDiscovery.ForExtension(".tst"));
            Assert.False(CoreLibrary.IsOpen(listed), "listing opened the library");
            Assert.True(CoreFactory.IsSupported("game.tst"));
            Assert.False(CoreLibrary.IsOpen(listed), "asking what is supported opened the library");

            Assert.NotNull(found.Open());
            Assert.True(CoreLibrary.IsOpen(listed));
        }

        // The build step's tool writes the sidecar; the host's discovery reads it and opens the library it describes.
        [Fact]
        public void A_sidecar_the_kits_runner_writes_is_read_by_discovery_and_its_library_opens()
        {
            string dir = Path.Combine(_root, "runner");
            Directory.CreateDirectory(dir);
            string source = CoreAdapterTests.LibraryPath("v1_plain_core");
            string library = Path.Combine(dir, Path.GetFileName(source));
            File.Copy(source, library);
            string runner = Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "emusen-core-conform.exe" : "emusen-core-conform");
            var start = new System.Diagnostics.ProcessStartInfo(runner) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add("--sidecar");
            start.ArgumentList.Add(library);
            using var process = System.Diagnostics.Process.Start(start)!;
            string output = process.StandardOutput.ReadToEnd() + process.StandardError.ReadToEnd();
            process.WaitForExit();
            Assert.True(process.ExitCode == 0, output);
            Assert.Equal(CoreSidecar.PathFor(library), output.Trim());

            CoreDiscovery.UseDirectories(new[] { dir });
            var found = CoreDiscovery.Found.Single();
            Assert.Equal(("v1-plain-core", 0UL, "1.0", CoreSidecar.HashOf(library)), (found.Info.Id, found.Sidecar.Capabilities, found.Sidecar.Abi, found.Sidecar.Sha256));
            Assert.False(CoreLibrary.IsOpen(library));
            Assert.NotNull(found.Open());
            Assert.Equal("not yet loaded", new DiscoveredCore(found.Sidecar).Report);
            Assert.EndsWith("core ABI 1.0", found.Report);
        }

        [Fact]
        public void A_library_that_is_not_the_one_its_sidecar_describes_is_never_loaded()
        {
            string built = Install("v1_test_core", Path.Combine(_root, "build"));
            string swapped = Place(built, Path.Combine(_root, "swapped"));
            File.Copy(CoreAdapterTests.LibraryPath("v1_plain_core"), swapped, overwrite: true);
            CoreDiscovery.UseDirectories(new[] { Path.Combine(_root, "swapped") });

            var found = CoreDiscovery.Found.Single();
            Assert.Null(found.Open());
            Assert.Contains("its SHA-256 differs", found.Report);
            Assert.False(CoreLibrary.IsOpen(swapped));
        }

        [Fact]
        public void A_sidecar_whose_info_was_altered_is_refused_once_the_library_answers()
        {
            string built = Install("v1_test_core", Path.Combine(_root, "build"));
            Place(built, Path.Combine(_root, "altered"), ClaimSnes);
            CoreDiscovery.UseDirectories(new[] { Path.Combine(_root, "altered") });

            var found = CoreDiscovery.Found.Single();
            Assert.Null(found.Open());
            Assert.Contains("refused as altered", found.Report);
        }

        [Fact]
        public void A_v1_engine_of_a_known_system_joins_its_row_and_the_factory_honours_or_refuses_it_by_name()
        {
            Assert.Null(CoreCatalog.EngineFor("SNES"));
            string built = Install("v1_test_core", Path.Combine(_root, "build"));
            Place(built, Path.Combine(_root, "snes"), ClaimSnes);
            CoreDiscovery.UseDirectories(new[] { Path.Combine(_root, "snes") });

            var row = CoreCatalog.EngineFor("SNES")!;
            Assert.Equal(new[] { CoreCatalog.VenusEngine, "V1TestCore (test)" }, row.Choices);
            Assert.Equal(CoreCatalog.VenusEngine, row.Default);
            Assert.Equal(new[] { CoreCatalog.MoonEngine, CoreCatalog.MoonRtEngine }, CoreCatalog.EngineFor("NES")!.Choices);
            Assert.IsType<VenusCore>(CoreFactory.Create("game.sfc"));

            ICore core = CoreFactory.Create("game.sfc", engine: "V1TestCore (test)");
            Assert.IsType<VenusCore>(core);
            string notice = CoreFactory.EngineNotice("game.sfc", "V1TestCore (test)", core)!;
            Assert.StartsWith("V1TestCore (test) is not available (", notice);
            Assert.EndsWith("refused as altered); Venus (C#) is running.", notice);
        }

        [Fact]
        public void A_game_no_csharp_core_claims_opens_on_the_v1_engine_with_its_debug_target()
        {
            Install("v1_test_core", Path.Combine(_root, "cores"));
            CoreDiscovery.UseDirectories(new[] { Path.Combine(_root, "cores") });
            string rom = Path.Combine(_root, "game.tst");
            File.WriteAllBytes(rom, CoreAdapterTests.Image(1, 2));

            var bundle = CoreFactory.Load(rom);
            using var engine = Assert.IsType<CoreEngine>(bundle.Core);
            Assert.IsType<CoreDebugTarget>(bundle.DebugTarget);
            Assert.Null(bundle.Notice);
            engine.RunFrame();
            Assert.Equal(1, engine.TotalFrames);
            Assert.Equal("V1TestCore (test)", CoreFactory.EngineNotice(rom, null, engine) ?? engine.Info.DisplayName);
        }
    }
}
