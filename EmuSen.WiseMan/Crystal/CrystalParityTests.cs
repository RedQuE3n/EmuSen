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
    public partial class CrystalParityTests : IDisposable
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
            Compare(managed, native);
        }

        private void Compare(List<string> managed, List<string> native)
        {
            Assert.Equal(managed.Count, native.Count);
            string? first = null;
            for (int i = 0; i < managed.Count; i++)
            {
                if (managed[i] == native[i]) continue;
                int at = Enumerable.Range(0, Math.Min(managed[i].Length, native[i].Length)).FirstOrDefault(k => managed[i][k] != native[i][k], Math.Min(managed[i].Length, native[i].Length));
                int from = Math.Max(0, at - 160);
                _output.WriteLine($"answer {i}, first difference at {at}\nC#:   ...{managed[i][from..Math.Min(managed[i].Length, at + 160)]}\nRust: ...{native[i][from..Math.Min(native[i].Length, at + 160)]}");
                bool equalValues = !managed[i].StartsWith("status") && !native[i].StartsWith("status") && SameJson(managed[i], native[i]);
                first ??= equalValues ? $"answer {i} holds the same values in another order or spelling" : $"answer {i} differs";
            }
            if (first is not null) Assert.Fail(first);
        }

        // Whether two answers are JSON of the same values, false where either is not JSON.
        private static bool SameJson(string a, string b)
        {
            try { return JsonNode.DeepEquals(JsonNode.Parse(a), JsonNode.Parse(b)); }
            catch (System.Text.Json.JsonException) { return false; }
        }

        // The build's own tree: every project core, the two test cores, the build's manifest.
        private static IReadOnlyList<string> BuildTree => new[] { AppContext.BaseDirectory, Path.Combine(AppContext.BaseDirectory, "cores") };

        // A tree of the two test cores and a manifest that vouches for both.
        private string TestTree(bool manifest = true, bool development = false, bool plain = true)
        {
            string dir = Path.Combine(_root, "cores" + Guid.NewGuid().ToString("N")[..6]);
            Directory.CreateDirectory(dir);
            var lines = new List<string>();
            foreach (string name in plain ? new[] { "v1_test_core", "v1_plain_core" } : new[] { "v1_test_core" })
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

        // Every file of the bench of routes asked what firmware it needs, and an archive whose engine can require none left unpacked by both.
        [Fact]
        public void Firmware_needed_and_the_overview_are_the_same_and_an_archive_unpacks_alike()
        {
            IReadOnlyList<string> cases = RouteCases();
            string zippedDisc = Path.Combine(_root, "Roms", "Disc.zip");
            using (var archive = ZipFile.Open(zippedDisc, ZipArchiveMode.Create))
            {
                var disc = SyntheticPsxDisc.Build();
                using (var e = archive.CreateEntry("Disc (Track 1).bin").Open()) e.Write(disc.DataTrack);
                using (var e = archive.CreateEntry("Disc (Track 2).bin").Open()) e.Write(disc.AudioTrack);
                using (var e = new StreamWriter(archive.CreateEntry("Disc.cue").Open())) e.Write(disc.Cue.Replace("{name}", "Disc"));
            }
            var unpacked = new List<string>();
            foreach (CrystalImpl impl in new[] { CrystalImpl.Managed, CrystalImpl.Native })
            {
                if (Directory.Exists(DataStore.Unpacked)) Directory.Delete(DataStore.Unpacked, recursive: true);
                Answers(impl, BuildTree, (i, rt) => new[] { i.FirmwareNeeded(rt, zippedDisc, null) });
                unpacked.Add(!Directory.Exists(DataStore.Unpacked) ? "" : string.Join("\n", Directory.EnumerateFiles(DataStore.Unpacked, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal)
                    .Select(f => $"{Path.GetRelativePath(DataStore.Unpacked, f)} {Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(File.ReadAllBytes(f)))}")));
            }
            Assert.Equal(unpacked[0], unpacked[1]);
            Directory.CreateDirectory(DataStore.Firmware);
            File.WriteAllBytes(Path.Combine(DataStore.Firmware, "scph5501.bin"), new byte[12]);
            Same(BuildTree, (impl, rt) => cases.Select(c => impl.FirmwareNeeded(rt, c, null))
                .Append(impl.FirmwareNeeded(rt, cases[0], "No Such Engine")).Append(impl.FirmwareNeeded(rt, zippedDisc, null))
                .Append(impl.Read("firmware_overview", rt)));
        }

        [Fact]
        public void A_firmware_install_is_the_same_and_writes_the_same_file()
        {
            string tree = TestTree();
            string right = Path.Combine(_root, "picked.rom"), wrong = Path.Combine(_root, "short.rom");
            File.WriteAllBytes(right, Enumerable.Range(0, 64).Select(i => (byte)i).ToArray());
            File.WriteAllBytes(wrong, new byte[3000]);
            string request = "{\"core\":\"V1TestCore\",\"chip\":\"Boot ROM\",\"file\":\"boot.rom\",\"size\":64,\"purpose\":\"optional\",\"alternate_names\":[]}";
            string big = request.Replace("\"size\":64", "\"size\":1024");
            var written = new List<string>();
            foreach (CrystalImpl impl in new[] { CrystalImpl.Managed, CrystalImpl.Native })
            {
                if (Directory.Exists(DataStore.Firmware)) Directory.Delete(DataStore.Firmware, recursive: true);
                Answers(impl, new[] { tree }, (i, rt) => new[] { i.FirmwareInstall(rt, request, right) });
                written.Add(Convert.ToHexString(File.ReadAllBytes(Path.Combine(DataStore.Firmware, "boot.rom"))));
            }
            Assert.Equal(written[0], written[1]);
            Same(new[] { tree }, (impl, rt) => new[]
            {
                impl.FirmwareInstall(rt, request, right), impl.FirmwareInstall(rt, big, wrong), impl.FirmwareInstall(rt, request, Path.Combine(_root, "absent.rom")),
                impl.FirmwareInstall(rt, "{\"core\":\"V1TestCore\"}", right), impl.FirmwareInstall(rt, "{\"size\":64,\"core\":\"V1TestCore\"}", right),
                impl.FirmwareInstall(rt, request, ""), impl.Read("firmware_overview", rt),
            });
        }

        // A tree of save media planted the same way before each implementation runs: drawers, games' own files, stored modes.
        private void PlantCards()
        {
            string saves = DataStore.Saves;
            if (Directory.Exists(saves)) Directory.Delete(saves, recursive: true);
            byte[] Card(params (string Name, int[] Blocks)[] saves) => saves.Aggregate(EmuSen.Crystal.Systems.Psx.PsxCardFormat.Instance.Managed.Format()!, (card, s) => SaveMediaManagerTests.WithSave(card, s.Name, s.Name, s.Blocks, 0x5A));
            string psx = Path.Combine(saves, "PlayStation"), drawer = Path.Combine(psx, "Cards");
            Directory.CreateDirectory(drawer);
            File.WriteAllBytes(Path.Combine(drawer, "Card 1.mcr"), Card(("BASLUS-00001ONE", new[] { 0 })));
            File.WriteAllBytes(Path.Combine(drawer, "mine.mcr"), Card(("BASLUS-00002TWO", new[] { 0, 1 }), ("BASLUS-00003RED", new[] { 2 })));
            File.WriteAllBytes(Path.Combine(drawer, "UPPER.MCR"), Card());
            File.WriteAllBytes(Path.Combine(psx, "Ape Escape.mcr"), Card(("BASCUS-94423APE", new[] { 0 })));
            File.WriteAllBytes(Path.Combine(psx, "Ape Escape.2.mcr"), Card());
            File.WriteAllBytes(Path.Combine(psx, "notes.txt"), new byte[3]);
            Directory.CreateDirectory(Path.Combine(saves, "Sega CD", "Console"));
            var stored = new ConsolesConfig();
            stored.SetValue("PlayStation", "Cards", "named");
            stored.SetValue("PlayStation", "Card1", "mine");
            stored.SetValue("PlayStation", "Card2", ".hidden");
            stored.SetValue("Nintendo 64", "Cards", "named");
            stored.SetValue("Nintendo 64", "Card1", "sub/dir");
            stored.SetValue("Nintendo 64", "Card2", " padded");
            stored.SetValue("Sega CD", "ConsoleMemory", "sideways");
            stored.Save();
        }

        [Fact]
        public void The_card_manager_lists_opens_and_changes_cards_the_same()
        {
            string drawer = Path.Combine(DataStore.Saves, "PlayStation", "Cards");
            string Card(string name) => Path.Combine(drawer, name);
            string J(params (string Key, object Value)[] pairs) => System.Text.Json.JsonSerializer.Serialize(pairs.ToDictionary(p => p.Key, p => p.Value));
            var trees = new List<string>();
            var answers = new List<List<string>>();
            foreach (CrystalImpl impl in new[] { CrystalImpl.Managed, CrystalImpl.Native })
            {
                PlantCards();
                answers.Add(Answers(impl, BuildTree, (i, rt) => new[]
                {
                    i.CardsOp(rt, 0, J(("console", "PlayStation"))), i.CardsOp(rt, 0, J(("console", "Sega CD"))), i.CardsOp(rt, 0, J(("console", "Nintendo 64"))),
                    i.CardsOp(rt, 0, J(("console", "SNES"))), i.CardsOp(rt, 0, J(("console", "Nowhere"))), i.CardsOp(rt, 0, "{}"),
                    i.CardsOp(rt, 1, J(("path", Card("mine.mcr")))), i.CardsOp(rt, 1, J(("path", Card("New.mcr")))), i.CardsOp(rt, 1, J(("path", Card("x.txt")))),
                    i.CardsOp(rt, 6, J(("path", Card("New.mcr")))), i.CardsOp(rt, 6, J(("path", Card("New.mcr")))),
                    i.CardsOp(rt, 2, J(("from", Card("mine.mcr")), ("save", "BASLUS-00002TWO"), ("to", Card("New.mcr")))),
                    i.CardsOp(rt, 2, J(("from", Card("mine.mcr")), ("save", "9"), ("to", Card("New.mcr")))),
                    i.CardsOp(rt, 3, J(("from", Card("mine.mcr")), ("save", "2"), ("to", Card("Card 1.mcr")))),
                    i.CardsOp(rt, 4, J(("card", Card("Card 1.mcr")), ("save", "BASLUS-00001ONE"))),
                    i.CardsOp(rt, 5, J(("to", Card("Card 1.mcr")), ("from", new[] { Card("mine.mcr"), Path.Combine(DataStore.Saves, "PlayStation", "Ape Escape.mcr") }))),
                    i.CardsOp(rt, 5, J(("to", Card("Card 1.mcr")))),
                    i.CardsOp(rt, 7, J(("from", Card("mine.mcr")), ("to", Card("Copy.mcr")))), i.CardsOp(rt, 7, J(("from", Card("mine.mcr")), ("to", Card("Copy.mcr")))),
                    i.CardsOp(rt, 8, J(("from", Card("Copy.mcr")), ("to", Card("Renamed.mcr")))), i.CardsOp(rt, 8, J(("from", Card("Gone.mcr")), ("to", Card("x.mcr")))),
                    i.CardsOp(rt, 99, "{}"), i.CardsOp(rt, 0, J(("console", "PlayStation"))),
                }).ToList());
                trees.Add(string.Join("\n", Directory.EnumerateFiles(DataStore.Saves, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal)
                    .Select(f => $"{Path.GetRelativePath(DataStore.Saves, f)} {Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(File.ReadAllBytes(f)))}")));
            }
            for (int k = 0; k < answers[0].Count; k++)
                if (answers[0][k] != answers[1][k]) _output.WriteLine($"answer {k}\nC#:   {answers[0][k]}\nRust: {answers[1][k]}");
            Assert.Equal(answers[0], answers[1]);
            Assert.Equal(trees[0], trees[1]);
            Assert.Contains("\"done\":true", answers[0][11]);
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
