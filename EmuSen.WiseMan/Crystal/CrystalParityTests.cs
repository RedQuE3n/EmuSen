using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Text.Json.Nodes;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Crystal
{
    // The parity class of step B7: each call made of the C# implementation and of Rust's through the C ABI, and their answers and the files they leave compared - see EmuSen_Backend.md §29.7. Part (a)'s first piece: the runtime, its engines and cores, the systems, the shelves and a file's route.
    [Collection(TestCollections.ProcessGlobals)]
    public class CrystalParityTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCrystalParity_" + Guid.NewGuid().ToString("N"));

        public CrystalParityTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
        }

        public void Dispose()
        {
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (Exception e) when (e is IOException or UnauthorizedAccessException) { }
        }

        private static readonly string[] Listings = { "systems", "shelves", "engines", "cores" };

        // Each implementation in turn, a fresh runtime over the same directories, its answers in the order asked.
        private static List<string> Answers(CrystalImpl impl, IReadOnlyList<string> cores, Func<CrystalImpl, nint, IEnumerable<string>> asks, bool development = false)
        {
            // The C# implementation's discovery is the process's, so a flag one runtime was given is put back before the next.
            CoreDiscovery.UseDevelopment(null);
            CoreDiscovery.UseDirectories(cores);
            nint rt = impl.New(cores, development);
            try { return asks(impl, rt).ToList(); }
            finally { impl.Free(rt); }
        }

        private void Same(IReadOnlyList<string> cores, Func<CrystalImpl, nint, IEnumerable<string>> asks, bool development = false, Func<string, string>? normal = null)
        {
            normal ??= s => s;
            List<string> managed = Answers(CrystalImpl.Managed, cores, asks, development).Select(normal).ToList();
            List<string> native = Answers(CrystalImpl.Native, cores, asks, development).Select(normal).ToList();
            Assert.Equal(managed.Count, native.Count);
            string? first = null;
            for (int i = 0; i < managed.Count; i++)
            {
                if (managed[i] == native[i]) continue;
                int at = Enumerable.Range(0, Math.Min(managed[i].Length, native[i].Length)).FirstOrDefault(k => managed[i][k] != native[i][k], Math.Min(managed[i].Length, native[i].Length));
                int from = Math.Max(0, at - 160);
                _output.WriteLine($"answer {i}, first difference at {at}\nC#:   ...{managed[i][from..Math.Min(managed[i].Length, at + 160)]}\nRust: ...{native[i][from..Math.Min(native[i].Length, at + 160)]}");
                bool equalValues = !managed[i].StartsWith("status") && !native[i].StartsWith("status") && JsonNode.DeepEquals(JsonNode.Parse(managed[i]), JsonNode.Parse(native[i]));
                first ??= equalValues ? $"answer {i} holds the same values in another order or spelling" : $"answer {i} differs";
            }
            if (first is not null) Assert.Fail(first);
        }

        // The build's own tree: every project core, the two test cores, the build's manifest.
        private static IReadOnlyList<string> BuildTree => new[] { AppContext.BaseDirectory, Path.Combine(AppContext.BaseDirectory, "cores") };

        // A tree of the two test cores and a manifest that vouches for both.
        private string TestTree(bool manifest = true, bool development = false)
        {
            string dir = Path.Combine(_root, "cores" + Guid.NewGuid().ToString("N")[..6]);
            Directory.CreateDirectory(dir);
            var lines = new List<string>();
            foreach (string name in new[] { "v1_test_core", "v1_plain_core" })
            {
                string source = CoreAdapterTests.LibraryPath(name);
                string copy = Path.Combine(dir, Path.GetFileName(source));
                File.Copy(source, copy);
                CoreSidecar.Write(copy, development && name == "v1_plain_core");
                lines.Add($"{CoreSidecar.HashOf(copy)}  {Path.GetFileName(copy)}");
            }
            if (manifest) File.WriteAllLines(Path.Combine(dir, "cores.manifest"), lines);
            return dir;
        }

        [Fact]
        public void The_listings_of_the_build_s_tree_are_the_same()
        {
            Assert.True(CrystalImpl.Native.Has("systems"), "the platform library exports no emusen_crystal_systems");
            Same(BuildTree, (impl, rt) => Listings.Select(l => impl.Read(l, rt)).Concat(Listings.Select(l => impl.Read(l, rt))));
        }

        [Fact]
        public void The_listings_of_a_test_tree_are_the_same_and_a_development_core_is_listed_on_the_flag_alone()
        {
            string tree = TestTree(development: true);
            Same(new[] { tree }, (impl, rt) => Listings.Select(l => impl.Read(l, rt)));
            Same(new[] { tree }, (impl, rt) => Listings.Select(l => impl.Read(l, rt)), development: true);
            Same(new[] { tree, Path.Combine(_root, "nowhere"), tree }, (impl, rt) => Listings.Select(l => impl.Read(l, rt)));
        }

        [Fact]
        public void A_rescan_lists_the_cores_directories_as_they_now_stand()
        {
            string tree = TestTree();
            string sidecar = Directory.GetFiles(tree, "*v1_plain_core*" + CoreSidecar.Suffix).Single();
            byte[] kept = File.ReadAllBytes(sidecar);
            Same(new[] { tree }, (impl, rt) =>
            {
                string before = impl.Read("engines", rt);
                File.Delete(sidecar);
                CoreDiscovery.Rescan();
                int status = impl.Rescan(rt);
                string after = impl.Read("engines", rt);
                File.WriteAllBytes(sidecar, kept);
                return new[] { before, status.ToString(), after };
            });
        }

        [Fact]
        public void The_first_listing_copies_graphics_json_s_consoles_once_and_writes_the_same_file()
        {
            var bytes = new List<byte[]>();
            foreach (CrystalImpl impl in new[] { CrystalImpl.Managed, CrystalImpl.Native })
            {
                string consoles = Path.Combine(ConfigStore.Directory, ConsolesConfig.FileName);
                if (File.Exists(consoles)) File.Delete(consoles);
                var graphics = new GraphicsConfig();
                graphics.SetValue("Genesis", "pad1", "md.pad6");
                graphics.SetValue("PlayStation", "pad2", "none");
                graphics.SetValue("SNES", "Engine", "Venus (C#)");
                graphics.Save();
                Answers(impl, BuildTree, (i, rt) => new[] { i.Read("systems", rt) });
                bytes.Add(File.ReadAllBytes(consoles));
            }
            Assert.Equal(System.Text.Encoding.UTF8.GetString(bytes[0]), System.Text.Encoding.UTF8.GetString(bytes[1]));
            Same(BuildTree, (impl, rt) => new[] { impl.Read("systems", rt) });
        }

        // Files of every kind routing decides between, each asked of both.
        private IReadOnlyList<string> RouteCases()
        {
            string dir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(dir);
            string W(string name, byte[] bytes)
            {
                string path = Path.Combine(dir, name);
                Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                File.WriteAllBytes(path, bytes);
                return path;
            }
            var cases = new List<string>
            {
                W("Plain.nes", SyntheticNesRom.Build()),
                W("Battery.nes", SyntheticNesRom.Build(prgBanks: 2, chrBanks: 0, mapper: 1, battery: true)),
                W("Short.nes", SyntheticNesRom.Build(prgBanks: 2)[..(16 + 0x4000)]),
                W("NoHeader.nes", new byte[64]),
                W("Game.sfc", SyntheticRom.Build()),
                W("Mono.gb", SyntheticGbRom.Build()),
                W("Colour.gb", SyntheticGbRom.Build(cgbFlag: 0x80)),
                W("Only.gb", SyntheticGbRom.Build(cgbFlag: 0xC0)),
                W("Colour.gbc", SyntheticGbRom.Build()),
                W("Game.z64", SyntheticN64Rom.Build()),
                W("Game.md", SyntheticMdRom.Cartridge()),
                W("Game.bin", SyntheticMdRom.Cartridge()),
                W("SegaCd.iso", SyntheticMdRom.Disc("U")),
                W("Psx.iso", SyntheticPsxDisc.Build().Iso),
                W("Nothing.iso", new byte[0x10000]),
                W(Path.Combine("segacd", "Folder.iso"), new byte[0x10000]),
                W(Path.Combine("psx", "Folder.iso"), new byte[0x10000]),
                W("Unknown.xyz", new byte[16]),
                SyntheticPsxDisc.Build("Europe").WriteSet(dir, "Amber Set"),
                Path.Combine(dir, "Missing.nes"),
            };
            string zip = Path.Combine(dir, "Zipped.zip");
            using (var archive = ZipFile.Open(zip, ZipArchiveMode.Create))
            using (var entry = archive.CreateEntry("inside.nes").Open()) entry.Write(SyntheticNesRom.Build());
            cases.Add(zip);
            string list = Path.Combine(dir, "Two.m3u");
            File.WriteAllText(list, "Amber Set.cue\n");
            cases.Add(list);
            return cases;
        }

        [Fact]
        public void Every_file_routes_the_same_on_the_build_s_tree()
        {
            IReadOnlyList<string> cases = RouteCases();
            var seen = new List<string>();
            Same(BuildTree, (impl, rt) =>
            {
                var answers = cases.Select(c => impl.Route(rt, c)).Append(impl.Route(rt, null)).Append(impl.Route(rt, "")).ToList();
                seen = answers;
                return answers;
            });
            string Of(string name, string key) => JsonNode.Parse(seen[cases.ToList().FindIndex(c => Path.GetFileName(c) == name)])![key]?.GetValue<string>() ?? "null";
            Assert.Equal(("nes", "nes", "gb", "gb", "mcd", "psx", "psx", "mcd"), (Of("Plain.nes", "system"), Of("Zipped.zip", "system"), Of("Colour.gb", "system"), Of("Mono.gb", "system"), Of("SegaCd.iso", "system"), Of("Psx.iso", "system"), Of("Amber Set.cue", "system"), Of("Folder.iso", "system")));
            Assert.Equal(("Game Boy Color (Mercury)", "Game Boy (Mercury)", "null"), (Of("Colour.gb", "shelf"), Of("Mono.gb", "shelf"), Of("Nothing.iso", "system")));
            Assert.StartsWith("status -1104", seen[^1]);
        }

        [Fact]
        public void A_game_s_own_engine_and_its_console_s_are_the_route_s_engine()
        {
            IReadOnlyList<string> cases = RouteCases();
            string nes = cases[0], sfc = cases[4];
            string own = Path.Combine(ConfigStore.Directory, "games", "nes", "Plain.json");
            Directory.CreateDirectory(Path.GetDirectoryName(own)!);
            File.WriteAllText(own, "{\"Engine\":\"Moon (C#)\"}");
            var stored = new ConsolesConfig();
            stored.SetValue("SNES", "Engine", "Some Engine");
            stored.Save();
            Same(BuildTree, (impl, rt) => new[] { impl.Route(rt, nes), impl.Route(rt, sfc) });
        }

        [Fact]
        public void The_test_tree_routes_the_test_core_s_files_the_same()
        {
            string tree = TestTree();
            string game = Path.Combine(_root, "game.tst");
            File.WriteAllBytes(game, CoreAdapterTests.Image(new byte[] { 1, 2, 3, 4 }));
            Same(new[] { tree }, (impl, rt) => new[] { impl.Route(rt, game), impl.Route(rt, Path.Combine(_root, "game.nes")) });
        }
    }
}
