using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json.Nodes;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;

namespace EmuSen.WiseMan.Galaxia
{
    // A self-contained package's claims by both halves: its copy and the library file it was copied from, which opens the copy only while it is still that file; a package's zip, written here by another archiver, which the C# half leaves to the platform library - see EmuSen_GamePackages.md §19.
    public partial class GalaxiaParityTests
    {
        private PackagePlaces SelfContainedTree(string side)
        {
            string root = At("self-contained", side), library = Path.Combine(root, "Roms"), packages = Path.Combine(root, "Packages");
            string game = Put(Path.Combine(library, "NES", "Copied.nes"));
            File.WriteAllBytes(game, Cartridge(6, 2048));
            File.SetLastWriteTimeUtc(game, new DateTime(2026, 10, 25, 9, 0, 0, DateTimeKind.Utc));
            string folder = Path.Combine(packages, "NES", "Copied.nes"), copy = Path.Combine(folder, "Copied.nes");
            Directory.CreateDirectory(folder);
            File.Copy(game, copy);
            RomHashes h = RomHash.Managed.Of(game, 0, -1, default);
            var p = GamePackage.Create("Copied", "nes", "self-contained", new PackageFile("image", "package", "Copied.nes", h.Size));
            var seen = GamePackages.Managed.SeenOf(copy)!.Value;
            var source = GamePackages.Managed.SeenOf(game)!.Value;
            ((JsonObject)p.Document["game"]!)["seen"] = new JsonObject { ["size"] = seen.Size, ["mtime"] = seen.Mtime };
            p.Document["dump"] = new JsonObject { ["file"] = new JsonObject { ["crc32"] = h.Crc32, ["md5"] = h.Md5, ["sha1"] = h.Sha1 } };
            p.Document["sources"] = new JsonArray(new JsonObject { ["role"] = "image", ["where"] = "library", ["path"] = "NES/Copied.nes", ["size"] = h.Size, ["seen"] = new JsonObject { ["size"] = source.Size, ["mtime"] = source.Mtime }, ["sha1"] = h.Sha1 });
            File.WriteAllText(Path.Combine(folder, GamePackage.FileName), GamePackage.Managed.Format(p.Document));
            return new PackagePlaces(packages, library, Path.Combine(root, "Compressed"));
        }

        [Fact]
        public void A_self_contained_package_claims_its_copy_and_its_source_alike_both_ways_and_a_changed_source_opens_plainly()
        {
            var runs = new List<List<string>>();
            foreach (string side in new[] { "managed", "native" })
            {
                PackagePlaces p = SelfContainedTree(side);
                Func<string, PackageResolution> resolve = side == "managed" ? at => GamePackages.Managed.Resolve(p, at) : at => GamePackages.Resolve(at, p);
                string game = Path.Combine(p.Library!, "NES", "Copied.nes"), folder = Path.Combine(p.Packages, "NES", "Copied.nes");
                var a = new List<string> { Shown(side == "managed" ? GamePackages.Managed.Index(p) : GamePackages.Index(p)) };
                a.Add(Answer(() => resolve(game)));
                a.Add(Answer(() => resolve(folder)));
                File.SetLastWriteTimeUtc(game, new DateTime(2030, 1, 1, 0, 0, 0, DateTimeKind.Utc));
                a.Add(Answer(() => resolve(game)));
                File.WriteAllBytes(game, Cartridge(7, 2048));
                File.SetLastWriteTimeUtc(game, new DateTime(2031, 1, 1, 0, 0, 0, DateTimeKind.Utc));
                a.Add(Answer(() => resolve(game)));
                File.Delete(game);
                a.Add(Answer(() => resolve(folder)));
                string root = At("self-contained", side);
                runs.Add(a.Select(x => x.Replace(root, "<root>")).ToList());
            }
            for (int i = 0; i < runs[0].Count; i++) Assert.True(runs[0][i] == runs[1][i], $"answer {i}\n  C#:   {runs[0][i]}\n  Rust: {runs[1][i]}");
            List<string> m = runs[0];
            Assert.Contains("<root>/Packages/NES/Copied.nes/Copied.nes, <root>/Roms/NES/Copied.nes", m[0]);
            Assert.Equal("<root>/Packages/NES/Copied.nes/Copied.nes | <root>/Packages/NES/Copied.nes | False", m[1]);
            Assert.Equal(m[1], m[3]);
            Assert.Equal("<root>/Roms/NES/Copied.nes | (none) | False", m[4]);
            Assert.Equal(m[1], m[5]);
        }

        [Fact]
        public void A_package_s_zip_is_unpacked_by_the_platform_library_and_refused_with_words_by_the_C_sharp_half()
        {
            PackagePlaces p = SelfContainedTree("zip");
            string folder = Path.Combine(p.Packages, "NES", "Copied.nes");
            Directory.CreateDirectory(At("zip-elsewhere"));
            string carried = Path.Combine(At("zip-elsewhere"), "Copied.nes.zip");
            using (var archive = System.IO.Compression.ZipFile.Open(carried, System.IO.Compression.ZipArchiveMode.Create))
                foreach (string f in Directory.EnumerateFiles(folder, "*", SearchOption.AllDirectories))
                    System.IO.Compression.ZipFileExtensions.CreateEntryFromFile(archive, f, "Copied.nes/" + Path.GetRelativePath(folder, f).Replace('\\', '/'));
            Assert.Equal("Copied.nes", GamePackages.ZipManifest(carried)?.Top);
            Assert.Contains("platform library", Answer(() => GamePackages.Managed.Resolve(p, carried)));
            Directory.Move(folder, folder + ".away");
            PackageResolution r = GamePackages.Resolve(carried, p);
            Assert.Equal((Path.Combine(p.Packages, "Copied.nes", "Copied.nes"), Path.Combine(p.Packages, "Copied.nes")), (r.Game, r.Package));
            Assert.Null(GamePackages.ZipManifest(Path.Combine(p.Packages, "Copied.nes", "Copied.nes")));
            Assert.Equal(r.Package, GamePackages.Resolve(carried, p).Package);
        }
    }
}
