using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json.Nodes;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;

namespace EmuSen.WiseMan.Galaxia
{
    // The claim index, "one game, one save" and a packaged game's save names, by both halves over the same generated trees - see EmuSen_GamePackages.md §14 and EmuSen_Galaxia.md §3.11.
    public partial class GalaxiaParityTests
    {
        private static byte[] Cartridge(int seed, int length = 4096) => Enumerable.Range(0, length).Select(i => (byte)(i * 7 + seed)).ToArray();

        // A linked package for a library file, its dump and seen as pkg make records them.
        private static void LinkedPackage(string folder, string library, string rel, string system = "snes", string? extra = null)
        {
            string game = Path.Combine(library, rel);
            RomHashes h = RomHash.Managed.Of(game, 0, -1, default);
            var p = GamePackage.Create(Path.GetFileNameWithoutExtension(game), system, "linked", new PackageFile("image", "library", rel, h.Size));
            var seen = GamePackages.Managed.SeenOf(game)!.Value;
            ((JsonObject)p.Document["game"]!)["seen"] = new JsonObject { ["size"] = seen.Size, ["mtime"] = seen.Mtime };
            p.Document["dump"] = new JsonObject { ["file"] = new JsonObject { ["crc32"] = h.Crc32, ["md5"] = h.Md5, ["sha1"] = h.Sha1 } };
            if (extra is not null) foreach (var (k, v) in JsonNode.Parse(extra)!.AsObject().ToList()) p.Document[k] = v?.DeepClone();
            Directory.CreateDirectory(folder);
            Assert.True(GamePackage.Managed.Format(p.Document).Length > 0);
            File.WriteAllText(Path.Combine(folder, GamePackage.FileName), GamePackage.Managed.Format(p.Document));
        }

        // One tree: linked packages, a converted disc's, a nested console folder, a refused manifest, a hidden folder, a package with no library game, and loose files.
        private PackagePlaces ClaimsTree(string side)
        {
            string root = At("claims", side), library = Path.Combine(root, "Roms"), packages = Path.Combine(root, "Packages"), compressed = Path.Combine(root, "Compressed");
            Put(Path.Combine(library, "SNES", "Alpha.sfc"));
            File.WriteAllBytes(Path.Combine(library, "SNES", "Alpha.sfc"), Cartridge(1));
            File.WriteAllBytes(Put(Path.Combine(library, "SNES", "USA", "Beta (U).sfc")), Cartridge(2));
            File.WriteAllBytes(Put(Path.Combine(library, "SNES", "Gamma.sfc")), Cartridge(3));
            File.WriteAllBytes(Put(Path.Combine(library, "NES", "Delta.nes")), Cartridge(4, 2048));
            Put(Path.Combine(library, "psx", "Disc.cue"), "FILE \"Disc.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n");
            File.WriteAllBytes(Put(Path.Combine(library, "psx", "Disc.bin")), new byte[2352]);
            File.WriteAllBytes(Put(Path.Combine(compressed, "psx", "Disc.chd")), Cartridge(9, 100));
            LinkedPackage(Path.Combine(packages, "SNES", "Alpha.sfc"), library, "SNES/Alpha.sfc");
            LinkedPackage(Path.Combine(packages, "SNES", "USA", "Beta (U).sfc"), library, "SNES/USA/Beta (U).sfc", extra: "{\"companions\": [{\"role\": \"sbi\", \"where\": \"library\", \"path\": \"SNES/USA/Beta (U).sbi\"}, {\"role\": \"x\", \"where\": \"absolute\", \"path\": \"/nowhere/at/all.bin\"}]}");
            LinkedPackage(Path.Combine(packages, "NES", "Delta.nes"), library, "NES/Delta.nes", "nes");
            Put(Path.Combine(packages, "PlayStation", "Disc.chd", GamePackage.FileName),
                "{\"format\": \"emusen-package\", \"version\": \"1.0\", \"system\": \"psx\", \"mode\": \"linked\", \"game\": {\"role\": \"image\", \"where\": \"compressed\", \"path\": \"psx/Disc.chd\"}, " +
                "\"container\": {\"type\": \"chd\", \"sources\": [{\"where\": \"library\", \"path\": \"psx/Disc.cue\"}, {\"where\": \"library\", \"path\": \"psx/Disc.bin\"}, 5, {\"where\": \"library\"}]}}");
            Put(Path.Combine(packages, "Broken", "Bad.sfc", GamePackage.FileName), "{ not json");
            Put(Path.Combine(packages, "Newer", "New.sfc", GamePackage.FileName), "{\"format\": \"emusen-package\", \"version\": \"2.0\"}");
            Put(Path.Combine(packages, ".hidden", "Hidden.sfc", GamePackage.FileName), "{}");
            Put(Path.Combine(packages, "SNES", "Alpha.sfc", "saves", "Alpha.srm"));
            Put(Path.Combine(packages, "loose.txt"));
            return new PackagePlaces(packages, library, compressed);
        }

        private static string Shown(PackageIndex i) =>
            string.Join("\n", i.Folders) + "\n--\n" + string.Join("\n", i.Packages.Select(p => $"{p.Folder} | {p.System} | {p.Game} | {string.Join(", ", p.Claims)}"));

        private static string Answer(Func<PackageResolution> resolve)
        {
            try { PackageResolution r = resolve(); return $"{r.Game} | {r.Package ?? "(none)"} | {r.Moved}"; }
            catch (IOException e) { return $"{e.GetType().Name}: {e.Message}"; }
        }

        [Fact]
        public void The_claim_index_is_the_same_both_ways()
        {
            PackagePlaces places = ClaimsTree("index");
            string managed = Shown(GamePackages.Managed.Index(places));
            Assert.Equal(managed, Shown(GamePackages.Index(places)));
            Assert.Contains("Disc.cue", managed);
            Assert.DoesNotContain("Hidden", managed);
            Assert.Contains(Path.Combine(places.Packages, "Broken", "Bad.sfc"), managed);
            PackagePlaces noLibrary = places with { Library = null };
            Assert.Equal(Shown(GamePackages.Managed.Index(noLibrary)), Shown(GamePackages.Index(noLibrary)));
            Assert.Equal("\n--\n", Shown(GamePackages.Index(places with { Packages = At("claims", "none") })));
        }

        // Each half resolves the same paths over its own copy of the tree, since resolving may rewrite a manifest, and the two trees end alike.
        [Fact]
        public void A_path_resolves_alike_both_ways_and_each_rewrites_its_manifests_alike()
        {
            var runs = new List<List<string>>();
            foreach (string side in new[] { "managed", "native" })
            {
                PackagePlaces p = ClaimsTree(side);
                Func<string, PackageResolution> resolve = side == "managed" ? at => GamePackages.Managed.Resolve(p, at) : at => GamePackages.Resolve(at, p);
                string lib = p.Library!;
                var a = new List<string>();
                string[] paths =
                {
                    Path.Combine(lib, "SNES", "Alpha.sfc"), Path.Combine(lib, "SNES", "..", "SNES", "Alpha.sfc"), Path.Combine(p.Packages, "SNES", "Alpha.sfc"),
                    Path.Combine(p.Packages, "SNES", "Alpha.sfc", GamePackage.FileName), Path.Combine(lib, "SNES", "Gamma.sfc"), Path.Combine(lib, "absent.sfc"),
                    Path.Combine(lib, "psx", "Disc.cue"), Path.Combine(lib, "psx", "Disc.bin"), Path.Combine(p.Packages, "PlayStation", "Disc.chd"), Path.Combine(lib, "NES", "Delta.nes"),
                    Path.Combine(p.Packages, "Newer", "New.sfc"), Path.Combine(p.Packages, "SNES"),
                };
                foreach (string at in paths) a.Add(Answer(() => resolve(at)));
                // Moved within the library: the plain path at its new place is found by size and hashes, and opened by its package after a second move.
                Directory.CreateDirectory(Path.Combine(lib, "SNES", "Europe"));
                File.Move(Path.Combine(lib, "SNES", "USA", "Beta (U).sfc"), Path.Combine(lib, "SNES", "Europe", "Beta (E).sfc"));
                a.Add(Answer(() => resolve(Path.Combine(lib, "SNES", "Europe", "Beta (E).sfc"))));
                a.Add(Answer(() => resolve(Path.Combine(lib, "SNES", "Europe", "Beta (E).sfc"))));
                File.Move(Path.Combine(lib, "SNES", "Europe", "Beta (E).sfc"), Path.Combine(lib, "SNES", "Beta.sfc"));
                a.Add(Answer(() => resolve(Path.Combine(p.Packages, "SNES", "USA", "Beta (U).sfc"))));
                // A touched file keeps its package and has its time recorded; another dump of the same size is refused with both hashes; a game gone and not found is refused.
                File.SetLastWriteTimeUtc(Path.Combine(lib, "SNES", "Alpha.sfc"), new DateTime(2030, 1, 2, 3, 4, 5, DateTimeKind.Utc));
                a.Add(Answer(() => resolve(Path.Combine(lib, "SNES", "Alpha.sfc"))));
                File.WriteAllBytes(Path.Combine(lib, "NES", "Delta.nes"), Cartridge(5, 2048));
                File.SetLastWriteTimeUtc(Path.Combine(lib, "NES", "Delta.nes"), new DateTime(2031, 1, 1, 0, 0, 0, DateTimeKind.Utc));
                a.Add(Answer(() => resolve(Path.Combine(lib, "NES", "Delta.nes"))));
                File.Delete(Path.Combine(lib, "NES", "Delta.nes"));
                a.Add(Answer(() => resolve(Path.Combine(p.Packages, "NES", "Delta.nes"))));
                string root = At("claims", side);
                a.Add(string.Join("\n", Directory.EnumerateFiles(Path.Combine(root, "Packages"), "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal).Select(f => Path.GetRelativePath(root, f) + "\n" + File.ReadAllText(f))));
                runs.Add(a.Select(x => x.Replace(root, "<root>")).ToList());
            }
            for (int i = 0; i < runs[0].Count; i++) Assert.True(runs[0][i] == runs[1][i], $"answer {i}\n  C#:   {runs[0][i]}\n  Rust: {runs[1][i]}");
            List<string> m = runs[0];
            Assert.EndsWith("<root>/Packages/SNES/Alpha.sfc | False", m[0]);
            Assert.StartsWith("<root>/Compressed/psx/Disc.chd | <root>/Packages/PlayStation/Disc.chd", m[6]);
            Assert.EndsWith("(none) | False", m[4]);
            Assert.EndsWith("| True", m[12]);
            Assert.EndsWith("| False", m[13]);
            Assert.StartsWith("<root>/Roms/SNES/Beta.sfc |", m[14]);
            Assert.Contains("has SHA-1", m[16]);
            Assert.Contains("which is gone", m[17]);
            Assert.Contains("\"path\": \"SNES/Beta.sfc\"", m[18]);
            Assert.Contains("\"mtime\": \"2030-01-02T03:04:05Z\"", m[18]);
        }

        [Fact]
        public void A_packaged_game_s_saves_and_states_are_named_alike_both_ways_by_the_package_s_own_name()
        {
            string[] roots = { At("Packages", "SNES", "Zelda (U).sfc"), At("p.q") + Path.DirectorySeparatorChar, "relative/pkg", At("Packages", "PlayStation", "a.b.chd") };
            foreach (string root in roots)
            {
                Assert.Equal(SaveLibrary.Managed.PackageSramPathFor(root), GalaxiaNative.SavePath(GalaxiaSave.PackageSram, root));
                Assert.Equal(SaveLibrary.Managed.PackageResumeStatePathFor(root), GalaxiaNative.SavePath(GalaxiaSave.PackageResumeState, root));
                foreach (int slot in new[] { 1, 2, 0, -1, int.MaxValue })
                    Assert.Equal(SaveLibrary.Managed.PackageStatePathFor(root, slot), GalaxiaNative.SavePath(GalaxiaSave.PackageState, root, null, slot));
            }
            Assert.Equal(Path.Combine(roots[0], "saves", "Zelda (U).srm"), SaveLibrary.SramPathFor("/roms/SNES/USA/Zelda (moved).sfc", "SNES", roots[0]));
            Assert.Equal(Path.Combine(roots[3], "states", "a.b.slot3.state"), SaveLibrary.StatePathFor("/roms/psx/a.b.cue", 3, At("ignored"), roots[3]));
            Assert.Equal(Path.Combine(roots[1], "states", "p.resume.state"), SaveLibrary.ResumeStatePathFor("x.sfc", null, roots[1]));
            Assert.Equal(SaveLibrary.StatePathFor("/r/a.sfc", 2, null), SaveLibrary.StatePathFor("/r/a.sfc", 2, null, " "));
        }

        // A packaged game's cheat list is its package's once written, the console's read until then; nothing else changes.
        [Fact]
        public void A_packaged_game_s_cheat_list_is_kept_in_its_package()
        {
            ConfigStore.OverrideDirectory = At("config");
            string package = At("Packages", "SNES", "Alpha.sfc");
            GameCheats plain = GameCheats.For("SNES", "/roms/SNES/Alpha.sfc")!;
            Assert.Null(plain.PackageFile);
            var list = new CheatFile();
            list.Cheats.Add(new CheatFileEntry { Description = "Lives", Address = "7E0010", Value = "63" });
            Assert.True(plain.Save(list));
            GameCheats packaged = GameCheats.For("SNES", "/roms/SNES/Alpha.sfc", package)!;
            Assert.Equal(Path.Combine(package, "cheats.json"), packaged.PackageFile);
            Assert.Equal("Lives", packaged.Load()!.Cheats.Single().Description);
            list.Cheats.Add(new CheatFileEntry { Description = "Map", Address = "7E0011", Value = "01" });
            Assert.True(packaged.Save(list));
            Assert.True(File.Exists(packaged.PackageFile));
            Assert.Equal(2, packaged.Load()!.Cheats.Count);
            Assert.Single(plain.Load()!.Cheats);
        }
    }
}
