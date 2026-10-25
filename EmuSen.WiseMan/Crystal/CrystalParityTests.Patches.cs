using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json.Nodes;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Cores;

namespace EmuSen.WiseMan.Crystal
{
    // Step 6's gate in both runtimes: a patched package's core is handed the patched image and the file is never written, a state records the patched image, and a changed patch refuses the game - see EmuSen_GamePackages.md §15.
    public partial class CrystalParityTests
    {
        private static string Hex(byte[] hash) => Convert.ToHexString(hash).ToLowerInvariant();

        // An IPS patch written here from its description: records of a three-byte offset, a two-byte size and the bytes, then EOF.
        internal static byte[] Ips(params (int Offset, byte[] Bytes)[] records)
        {
            var b = new List<byte>(Encoding.ASCII.GetBytes("PATCH"));
            foreach (var (offset, bytes) in records)
            {
                b.AddRange(new[] { (byte)(offset >> 16), (byte)(offset >> 8), (byte)offset, (byte)(bytes.Length >> 8), (byte)bytes.Length });
                b.AddRange(bytes);
            }
            b.AddRange(Encoding.ASCII.GetBytes("EOF"));
            return b.ToArray();
        }

        // The package's list given one IPS patch copied in under name, pinned to the original and with the patched image's hashes, as pkg patch add records them.
        internal static void PatchPackage(string folder, string name, byte[] patch, byte[] original, byte[] target, string identity = "original")
        {
            File.WriteAllBytes(Path.Combine(folder, name), patch);
            GamePackage p = GamePackage.Read(Path.Combine(folder, GamePackage.FileName));
            string sha = Hex(SHA1.HashData(patch));
            RomHashes t = RomHash.Of(target);
            p.Document["patches"] = new JsonArray(new JsonObject { ["file"] = name, ["format"] = "ips", ["sha1"] = sha, ["source_sha1"] = Hex(SHA1.HashData(original)) });
            p.Document["patched"] = new JsonObject
            {
                ["patches_sha1"] = Hex(SHA1.HashData(Encoding.UTF8.GetBytes($"ips {sha}\n"))), ["size"] = t.Size, ["crc32"] = t.Crc32, ["md5"] = t.Md5, ["sha1"] = t.Sha1,
            };
            p.Document["identity"] = identity;
            Assert.True(p.Write(Path.Combine(folder, GamePackage.FileName)));
        }

        [Fact]
        public void A_patched_package_s_core_is_handed_the_patched_image_and_the_file_is_never_written_both_ways()
        {
            string tree = TestTree(plain: false);
            string library = Path.Combine(_root, "Library");
            string game = Path.Combine(library, "TEST", "hack.tst"), target = Path.Combine(library, "TEST", "target.tst");
            string package = Path.Combine(DataStore.Packages, "TEST", "hack.tst");
            byte[] original = CoreAdapterTests.Image(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
            byte[] patched = original.ToArray();
            patched[6 + 2] = 0x42;
            patched[6 + 9] = 0x99;
            byte[] ips = Ips((8, new byte[] { 0x42 }), (15, new byte[] { 0x99 }));
            void Plant()
            {
                foreach (string d in new[] { library, DataStore.Packages })
                    if (Directory.Exists(d)) Directory.Delete(d, recursive: true);
                Directory.CreateDirectory(Path.GetDirectoryName(game)!);
                File.WriteAllBytes(game, original);
                File.WriteAllBytes(target, patched);
                new AppSettings { RomDirectory = library }.Save();
                PackageFor(package, library, "TEST/hack.tst", "test");
                PatchPackage(package, "hack.ips", ips, original, patched);
            }
            SameWithFiles(new[] { tree }, Plant, (i, rt) =>
            {
                var a = new List<string>();
                byte[] Played(string opened)
                {
                    uint s = Id(i.Open(rt, opened));
                    Assert.True(s != 999, opened);
                    for (int f = 0; f < 20; f++) i.Frame(rt, s);
                    a.Add(i.SlotCall("state_slot_save", rt, s, 1));
                    string state = JsonNode.Parse(i.Read("state_slots", rt, s))!.AsArray().First(e => (int)e!["slot"]! == 1)!["path"]!.GetValue<string>();
                    a.Add(Timeless(i.Read("state_record", rt, state)).Replace(_root, "<root>"));
                    a.Add(i.Call("close", rt, s));
                    return File.ReadAllBytes(state);
                }
                byte[] fromPackage = Played(game);
                byte[] fromTarget = Played(target);
                Assert.Equal(fromTarget, fromPackage);
                Assert.Contains($"\"rom_md5\":\"{RomHash.Of(patched).Md5}\"", a[1]);
                Assert.Equal(original, File.ReadAllBytes(game));
                File.WriteAllBytes(Path.Combine(package, "hack.ips"), Ips((8, new byte[] { 0x43 })));
                string refused = i.Open(rt, package);
                Assert.Contains("hack.ips is not the patch that was added", refused);
                a.Add(refused.Replace(_root, "<root>"));
                return a;
            }, also: () => TreeOf(library));
        }
    }
}
