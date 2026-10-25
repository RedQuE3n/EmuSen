using System;
using System.IO;
using System.Text.Json.Nodes;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Galaxia
{
    // A game package's manifest through the facade, in whichever half the switch chose - see EmuSen_GamePackages.md §3.3 and EmuSen_Galaxia.md §3.10.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class GamePackageTests : IDisposable
    {
        private readonly string _temp = Path.Combine(Path.GetTempPath(), "EmuSenPackage_" + Guid.NewGuid().ToString("N"));

        public GamePackageTests() => Directory.CreateDirectory(_temp);

        public void Dispose()
        {
            ConfigDiagnostics.Sink = null;
            Links.DeleteTree(_temp);
        }

        private static string HandWritten => Path.Combine(AppContext.BaseDirectory, "Galaxia", "HandWrittenPackage.json");

        [Fact]
        public void A_manifest_written_by_hand_in_an_editor_loads()
        {
            GamePackage package = GamePackage.Read(HandWritten);

            Assert.Equal(("1.0", null, "psx", "linked"), (package.Version, package.Title, package.System, package.Mode));
            Assert.Equal(new PackageFile("image", "library", "psx/Coral Gate (USA).cue"), package.Game);
            Assert.Equal("bought at a car boot sale, 1999", (string?)package.Document["my notes"]);
            Assert.Equal(70560, (long)package.Document["dump"]!["tracks"]![1]!["size"]!);
        }

        [Fact]
        public void A_manifest_round_trips_through_its_file_with_unknown_fields_in_their_places()
        {
            GamePackage made = GamePackage.Create("Aurora Drift", "snes", "linked", new PackageFile("image", "library", "SNES/Aurora Drift (USA).sfc", 4096));
            made.Document.Insert(0, "x-before", new JsonObject { ["kept"] = JsonNode.Parse("1.50") });
            ((JsonObject)made.Document["game"]!)["x-inside"] = "é & <ok>";
            string path = Path.Combine(_temp, "Packages", "snes", "Aurora Drift (USA).sfc", GamePackage.FileName);

            Assert.True(made.Write(path));
            string written = File.ReadAllText(path);
            GamePackage read = GamePackage.Read(path);

            Assert.Equal(made.Format(), written);
            Assert.StartsWith("{" + Environment.NewLine + "  \"x-before\": {" + Environment.NewLine + "    \"kept\": 1.50", written);
            Assert.EndsWith("}" + Environment.NewLine, written);
            Assert.Equal(written, read.Format());
            Assert.Equal("é & <ok>", (string?)read.Document["game"]!["x-inside"]);
            Assert.Equal(new PackageFile("image", "library", "SNES/Aurora Drift (USA).sfc", 4096), read.Game);
            Assert.True(read.Write(path));
            Assert.Equal(written, File.ReadAllText(path));
            Assert.False(File.Exists(path + AtomicFile.TempSuffix));
        }

        [Fact]
        public void A_newer_major_is_refused_with_words_and_nothing_is_written_over_a_good_manifest()
        {
            string path = Path.Combine(_temp, "newer", GamePackage.FileName);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllText(path, "{\"format\": \"emusen-package\", \"version\": \"2.1\", \"anything\": [\"a later build's\"]}");

            PackageRefusedException newer = Assert.Throws<PackageRefusedException>(() => GamePackage.Read(path));
            Assert.True(newer.Newer);
            Assert.Equal($"{path}: the package format is 2.1, written by a newer EmuSen; this build reads 1.x", newer.Message);

            GamePackage good = GamePackage.Create(null, "nes", "self-contained", new PackageFile("image", "package", "g.nes"));
            Assert.True(good.Write(path));
            string kept = File.ReadAllText(path);
            good.Document["game"]!["path"] = "../escape.nes";
            PackageRefusedException bad = Assert.Throws<PackageRefusedException>(() => good.Write(path));
            Assert.False(bad.Newer);
            Assert.Equal("game.path must be a path inside its root, with no .. and not absolute", bad.Message);
            Assert.Equal(kept, File.ReadAllText(path));
        }

        [Fact]
        public void A_file_that_is_no_manifest_is_refused_and_a_missing_one_is_not_found()
        {
            string path = Path.Combine(_temp, "broken.json");
            File.WriteAllText(path, "{\"format\": \"emusen-package\", ");
            Assert.False(Assert.Throws<PackageRefusedException>(() => GamePackage.Read(path)).Newer);
            File.WriteAllText(path, "{\"format\": \"emusen-package\", \"version\": \"1.0\", \"system\": \"nes\", \"mode\": \"linked\"}");
            Assert.Equal($"{path}: game is missing", Assert.Throws<PackageRefusedException>(() => GamePackage.Read(path)).Message);
            Assert.Throws<FileNotFoundException>(() => GamePackage.Read(Path.Combine(_temp, "absent.json")));
        }

        [Fact]
        public void An_unwritable_place_reports_and_returns_false()
        {
            string reported = "";
            ConfigDiagnostics.Sink = m => reported = m;
            string blocker = Path.Combine(_temp, "a file");
            File.WriteAllText(blocker, "x");
            GamePackage package = GamePackage.Create("G", "nes", "linked", new PackageFile("image", "library", "NES/g.nes"));

            Assert.False(package.Write(Path.Combine(blocker, GamePackage.FileName)));
            Assert.Contains("The previous file is unchanged.", reported);
        }
    }
}
