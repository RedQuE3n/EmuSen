using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.Galaxia.Native;

namespace EmuSen.WiseMan.Galaxia
{
    // A game package's manifest checked, laid out, written and read by both halves - see EmuSen_GamePackages.md §3.3 and EmuSen_Galaxia.md §3.10.
    public partial class GalaxiaParityTests
    {
        private const string Base = "\"format\": \"emusen-package\", \"version\": \"1.0\", \"system\": \"nes\", \"mode\": \"linked\"";
        private const string Game = "\"game\": {\"role\": \"image\", \"where\": \"library\", \"path\": \"NES/g.nes\"}";

        // Manifests the schema takes: unknown fields at every level, number tokens, text outside ASCII, comments and a trailing comma.
        private static IEnumerable<string> Accepted()
        {
            yield return File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "Galaxia", "HandWrittenPackage.json"));
            yield return $"{{{Base}, {Game}}}";
            yield return $"{{\"x\": 1, {Base}, {Game}, \"y\": [1.50, -0, 1e2, 1E+2, 12345678901234567890123, 0.000001, true, false, null, {{}}, []]}}";
            yield return $"{{{Base}, \"title\": \"P\u00f6k\u00e9mon \u2014 \U0001F3AE <&>'+`\\\"\\\\\\/\\b\\f\\n\\r\\t\\u0001\", {Game}}}";
            yield return $"{{ /* a */ {Base}, // b\n {Game}, \"companions\": [{{\"role\": \"extra\", \"where\": \"absolute\", \"path\": \"/m/a.pcm\", \"size\": 0, \"seen\": {{\"size\": 0, \"mtime\": \"t\", \"z\": 1}}}},], }}";
            yield return $"{{{Base}, {Game}, \"dump\": {{\"file\": {{\"crc32\": \"0123abcd\", \"md5\": \"{new string('a', 32)}\", \"sha1\": \"{new string('F', 40)}\"}}, \"transformed\": null, \"tracks\": [{{\"file\": \"t1.bin\", \"size\": 2352}}], \"sheet\": {{\"file\": \"g.cue\"}}, \"form\": \"cue-bin\", \"extra\": {{\"deep\": [[[1]]]}}}}}}";
            yield return $"{{{Base}, {Game}, \"patches\": [{{\"file\": \"g.bps\", \"format\": \"bps\", \"sha1\": null, \"source_crc32\": \"00000000\"}}], \"patched\": {{\"patches_sha1\": \"{new string('0', 40)}\"}}, \"identity\": \"patched\", \"container\": {{\"program\": \"chdman 0.289\"}}}}";
            yield return $"{{\"format\": \"emusen-package\", \"version\": \"1.42\", \"system\": \"psx\", \"mode\": \"self-contained\", \"game\": {{\"role\": \"image\", \"where\": \"package\", \"path\": \"a/b/c.chd\"}}, \"title\": null}}";
            yield return "{\"format\": \"emusen-package\", \"version\": \"1.0\", \"system\": \"n64\", \"mode\": \"linked\", \"game\": {\"role\": \"image\", \"where\": \"absolute\", \"path\": \"C:\\\\Roms\\\\g.z64\"}}";
        }

        // Texts the schema or the reader refuses, each for one reason.
        private static IEnumerable<string> Refused()
        {
            yield return "[]";
            yield return "\"a string\"";
            yield return "{";
            yield return "{\"format\": \"emusen-package\" \"version\": \"1.0\"}";
            yield return "{\"version\": \"1.0\"}";
            yield return "{\"format\": \"Emusen-Package\", \"version\": \"1.0\"}";
            yield return "{\"format\": \"emusen-package\"}";
            foreach (string v in new[] { "\"1\"", "\"1.\"", "\".1\"", "\"1.0.0\"", "\"1.x\"", "\"\u0661.0\"", "1.0", "null", "\"0.9\"", "\"00.1\"" })
                yield return $"{{\"format\": \"emusen-package\", \"version\": {v}}}";
            foreach (string v in new[] { "\"2.0\"", "\"10.0\"", "\"0002.5\"", "\"99999999999999999999999.0\"" })
                yield return $"{{\"format\": \"emusen-package\", \"version\": {v}, \"game\": 5}}";
            yield return $"{{{Base}, {Game}, \"format\": \"emusen-package\"}}";
            yield return $"{{{Base}, {Game}, \"x\": {{\"a\": 1, \"b\": {{\"c\": 1, \"c\": 2}}}}}}";
            yield return $"{{{Base}, {Game}, \"x\": [\"ok\", \"\\ud800\"]}}";
            yield return $"{{{Base}, {Game}, \"\\udc00\": 1}}";
            string[] fields =
            {
                "\"title\": 3", "\"title\": []", "\"dump\": 1", "\"dump\": {\"file\": []}", "\"dump\": {\"file\": {\"crc32\": 1}}", "\"dump\": {\"file\": {\"crc32\": \"0123abc\"}}",
                "\"dump\": {\"file\": {\"md5\": \"g0000000000000000000000000000000\"}}", "\"dump\": {\"transformed\": {\"sha1\": \"00\"}}", "\"dump\": {\"tracks\": {}}", "\"dump\": {\"tracks\": [1]}",
                "\"dump\": {\"tracks\": [{\"file\": \"\"}]}", "\"dump\": {\"tracks\": [{\"file\": \"a\", \"size\": \"1\"}]}", "\"dump\": {\"sheet\": {\"crc32\": \"00000000\"}}", "\"dump\": {\"form\": \"gdi\"}",
                "\"companions\": {}", "\"companions\": [1]", "\"companions\": [{\"role\": \"x\", \"where\": \"library\"}]", "\"patches\": [{}]", "\"patches\": [{\"file\": \"a\", \"format\": \"IPS\"}]",
                "\"patches\": [{\"file\": \"a\", \"format\": \"ips\", \"source_sha1\": \"0\"}]", "\"patches\": [{\"file\": \"a\", \"format\": \"ups\", \"target_crc32\": \"zzzzzzzz\"}]", "\"patched\": []",
                "\"patched\": {\"md5\": \"0\"}", "\"identity\": \"both\"", "\"identity\": null", "\"container\": []", "\"container\": 0",
            };
            foreach (string f in fields) yield return $"{{{Base}, {Game}, {f}}}";
            string[] games =
            {
                "5", "{}", "{\"role\": 1, \"where\": \"library\", \"path\": \"a\"}", "{\"role\": \"image\", \"where\": \"Library\", \"path\": \"a\"}", "{\"role\": \"image\", \"where\": \"library\", \"path\": \"\"}",
                "{\"role\": \"image\", \"where\": \"library\", \"path\": \"/a\"}", "{\"role\": \"image\", \"where\": \"compressed\", \"path\": \"a\\\\..\\\\b\"}", "{\"role\": \"image\", \"where\": \"package\", \"path\": \"z:x\"}",
                "{\"role\": \"image\", \"where\": \"absolute\", \"path\": \"rel\"}", "{\"role\": \"image\", \"where\": \"absolute\", \"path\": \"C:rel\"}", "{\"role\": \"image\", \"where\": \"library\", \"path\": \"a\", \"size\": -1}",
                "{\"role\": \"image\", \"where\": \"library\", \"path\": \"a\", \"size\": 1e3}", "{\"role\": \"image\", \"where\": \"library\", \"path\": \"a\", \"size\": 99999999999999999999}",
                "{\"role\": \"image\", \"where\": \"library\", \"path\": \"a\", \"seen\": 1}", "{\"role\": \"image\", \"where\": \"library\", \"path\": \"a\", \"seen\": {\"mtime\": \"t\"}}",
                "{\"role\": \"image\", \"where\": \"library\", \"path\": \"a\", \"seen\": {\"size\": 1, \"mtime\": 2}}",
            };
            foreach (string g in games) yield return $"{{{Base}, \"game\": {g}}}";
            yield return "{\"format\": \"emusen-package\", \"version\": \"1.0\", \"mode\": \"linked\", \"system\": \"\", \"game\": 1}";
            yield return "{\"format\": \"emusen-package\", \"version\": \"1.0\", \"system\": \"nes\", \"game\": 1}";
            yield return "{\"format\": \"emusen-package\", \"version\": \"1.0\", \"system\": \"nes\", \"mode\": \"copy\"}";
            yield return new string('[', 70) + new string(']', 70);
        }

        [Fact]
        public void A_manifest_is_taken_and_laid_out_to_the_same_bytes_both_ways()
        {
            foreach (string text in Accepted())
            {
                string managed = GamePackage.Managed.Format(GamePackage.Managed.Parse(text));
                Assert.Equal(managed, GalaxiaNative.PackageFormat(text));
                Assert.Equal(managed, GalaxiaNative.PackageFormat(managed));
                Assert.Equal(managed, GamePackage.Managed.Format(GamePackage.Managed.Parse(managed)));
            }
        }

        [Fact]
        public void A_manifest_is_refused_alike_both_ways_with_the_same_words()
        {
            int n = 0;
            foreach (string text in Refused())
            {
                PackageRefusedException managed = Assert.Throws<PackageRefusedException>(() => GamePackage.Managed.Parse(text));
                PackageRefusedException native = Assert.Throws<PackageRefusedException>(() => GalaxiaNative.PackageFormat(text));
                Assert.True(managed.Newer == native.Newer, text);
                // A text that is not JSON is refused by each reader in its own words; every other refusal is worded alike.
                if (managed.Message.StartsWith("the manifest is not JSON", StringComparison.Ordinal)) Assert.StartsWith("the manifest is not JSON", native.Message);
                else Assert.True(managed.Message == native.Message, $"{text}\n  C#:   {managed.Message}\n  Rust: {native.Message}");
                n++;
            }
            Assert.True(n > 60);
        }

        [Fact]
        public void A_manifest_is_written_and_read_back_to_the_same_bytes_both_ways()
        {
            int n = 0;
            foreach (string text in Accepted())
            {
                string managedFile = At("packages", $"managed {n}", "package.json"), nativeFile = At("packages", $"native {n++}", "package.json");
                var document = GamePackage.Managed.Parse(text);
                Assert.True(AtomicFile.Managed.Write(managedFile, new UTF8Encoding(false).GetBytes(GamePackage.Managed.Format(document))));
                Assert.True(GalaxiaNative.PackageWrite(nativeFile, document.ToJsonString()));
                Assert.Equal(File.ReadAllBytes(managedFile), File.ReadAllBytes(nativeFile));
                Assert.Equal(GamePackage.Managed.Format(GamePackage.Managed.Parse(File.ReadAllText(nativeFile))), GalaxiaNative.PackageFormat(GalaxiaNative.PackageRead(managedFile)));
            }
            string blocked = Put(At("packages", "a file"));
            Assert.False(GalaxiaNative.PackageWrite(Path.Combine(blocked, "package.json"), GamePackage.Managed.Parse(Accepted().Skip(1).First()).ToJsonString()));
            Assert.Single(Reported());
            Assert.False(AtomicFile.Managed.Write(Path.Combine(blocked, "package.json"), new byte[1]));
            Assert.Single(Reported());
            foreach (string missing in new[] { At("packages", "absent.json"), At("packages", "no", "folder", "package.json") })
            {
                Exception managed = Assert.ThrowsAny<Exception>(() => GamePackage.Managed.Parse(File.ReadAllText(missing)));
                Exception native = Assert.ThrowsAny<Exception>(() => GalaxiaNative.PackageRead(missing));
                Assert.True(managed.GetType() == native.GetType(), $"{missing}: {managed.GetType().Name} and {native.GetType().Name}");
            }
        }
    }
}
