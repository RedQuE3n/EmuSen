using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Text.Json.Nodes;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using Xunit;

namespace EmuSen.WiseMan.Crystal
{
    // Game packages and patches written for a test, as pkg make and pkg patch write them - see EmuSen_GamePackages.md §14 and §19.
    internal static class PackageFixtures
    {
        // A linked package for a library file, as pkg make writes one.
        internal static void PackageFor(string folder, string library, string rel, string system)
        {
            string game = Path.Combine(library, rel);
            RomHashes h = RomHash.Of(game);
            var p = GamePackage.Create(Path.GetFileNameWithoutExtension(game), system, "linked", new PackageFile("image", "library", rel, h.Size));
            var seen = new FileInfo(game);
            ((JsonObject)p.Document["game"]!)["seen"] = new JsonObject { ["size"] = seen.Length, ["mtime"] = GamePackagesTime(seen.LastWriteTimeUtc) };
            p.Document["dump"] = new JsonObject { ["file"] = new JsonObject { ["crc32"] = h.Crc32, ["md5"] = h.Md5, ["sha1"] = h.Sha1 } };
            Assert.True(p.Write(Path.Combine(folder, GamePackage.FileName)));
        }

        // A self-contained package for a library file, as pkg make --self-contained writes one: the game copied in with its time, the library file kept as its source.
        internal static void SelfContainedFor(string folder, string library, string rel, string system)
        {
            string game = Path.Combine(library, rel), copy = Path.Combine(folder, Path.GetFileName(game));
            Directory.CreateDirectory(folder);
            File.Copy(game, copy);
            File.SetLastWriteTimeUtc(copy, File.GetLastWriteTimeUtc(game));
            RomHashes h = RomHash.Of(game);
            JsonObject Seen(string at) => new() { ["size"] = new FileInfo(at).Length, ["mtime"] = GamePackagesTime(File.GetLastWriteTimeUtc(at)) };
            var p = GamePackage.Create(Path.GetFileNameWithoutExtension(game), system, "self-contained", new PackageFile("image", "package", Path.GetFileName(game), h.Size));
            ((JsonObject)p.Document["game"]!)["seen"] = Seen(copy);
            p.Document["dump"] = new JsonObject { ["file"] = new JsonObject { ["crc32"] = h.Crc32, ["md5"] = h.Md5, ["sha1"] = h.Sha1 } };
            p.Document["sources"] = new JsonArray(new JsonObject { ["role"] = "image", ["where"] = "library", ["path"] = rel, ["size"] = h.Size, ["seen"] = Seen(game), ["sha1"] = h.Sha1 });
            Assert.True(p.Write(Path.Combine(folder, GamePackage.FileName)));
        }

        // An IPS patch written from its description: records of a three-byte offset, a two-byte size and the bytes, then EOF.
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

        private static string GamePackagesTime(DateTime utc) => new DateTime(utc.Ticks - utc.Ticks % TimeSpan.TicksPerSecond, DateTimeKind.Utc).ToString("yyyy-MM-dd'T'HH:mm:ss'Z'", System.Globalization.CultureInfo.InvariantCulture);
    }
}
