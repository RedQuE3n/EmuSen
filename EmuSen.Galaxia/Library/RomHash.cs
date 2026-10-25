using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text.Json;
using System.Threading;
using EmuSen.Galaxia.Media;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Library
{
    // A dump's three hashes, lower-case hexadecimal, and the bytes they cover - see EmuSen_GamePackages.md §3.3.
    public sealed record RomHashes(string Md5, string Crc32, string Sha1, long Size);

    // One track of a cue sheet as dumped: its number, its file as the sheet names it, where it begins there, its hashes.
    public sealed record TrackHashes(int Track, string File, long Offset, RomHashes Hashes);

    // A dump's identity: whole, a range, a system pack's form, a sheet's tracks - see EmuSen_Galaxia.md §5.4 and EmuSen_GamePackages.md §2.2.
    public static class RomHash
    {
        public static string Md5(string path) => GalaxiaNative.Takes(path) ? GalaxiaNative.RomMd5(path) : Managed.Md5(path);

        // CRC-32, MD5 and SHA-1 of a whole file in one read.
        public static RomHashes Of(string path, CancellationToken stop = default) => Of(path, 0, -1, stop);

        // The same of length bytes from offset, fewer where the file ends; a length of -1 reads to its end.
        public static RomHashes Of(string path, long offset, long length, CancellationToken stop = default)
        {
            ArgumentOutOfRangeException.ThrowIfNegative(offset);
            return GalaxiaNative.Takes(path) ? FromJson(GalaxiaNative.RomHashes(path, offset, length, stop)) : Managed.Of(path, offset, length, stop);
        }

        public static RomHashes Of(ReadOnlySpan<byte> bytes) => GalaxiaNative.Active ? FromJson(GalaxiaNative.RomHashesOf(bytes)) : Managed.Of(bytes);

        // The hashes of the comparable form a system pack's transform makes, or null where it changes nothing; this knows no console.
        public static RomHashes? Transformed(byte[] file, Func<byte[], byte[]?> transform)
        {
            byte[]? changed = transform(file);
            return changed is null || changed.AsSpan().SequenceEqual(file) ? null : Of(changed);
        }

        // Each track of a cue sheet, hashed over the bytes emusen-disc places it in; InvalidDataException for a sheet that is not one.
        public static IReadOnlyList<TrackHashes> Tracks(string sheetPath, CancellationToken stop = default) =>
            GalaxiaNative.Takes(sheetPath) ? TracksFromJson(GalaxiaNative.RomTracks(sheetPath, stop)) : Managed.Tracks(sheetPath, stop);

        internal static RomHashes FromJson(string json)
        {
            using JsonDocument doc = JsonDocument.Parse(json);
            return FromElement(doc.RootElement);
        }

        private static RomHashes FromElement(JsonElement e) =>
            new(e.GetProperty("md5").GetString()!, e.GetProperty("crc32").GetString()!, e.GetProperty("sha1").GetString()!, e.GetProperty("size").GetInt64());

        internal static IReadOnlyList<TrackHashes> TracksFromJson(string json)
        {
            using JsonDocument doc = JsonDocument.Parse(json);
            return doc.RootElement.EnumerateArray()
                .Select(t => new TrackHashes(t.GetProperty("track").GetInt32(), t.GetProperty("file").GetString()!, t.GetProperty("offset").GetInt64(), FromElement(t)))
                .ToArray();
        }

        // The C# rule: the default, and what the library's is held to until Galaxia's gate - see EmuSen_RustPlatform.md §3.9.
        internal static class Managed
        {
            public static string Md5(string path)
            {
                using FileStream stream = new(path, FileMode.Open, FileAccess.Read, FileShare.Read, 1 << 16, FileOptions.SequentialScan);
                return Convert.ToHexStringLower(MD5.HashData(stream));
            }

            public static RomHashes Of(string path, long offset, long length, CancellationToken stop)
            {
                using var md5 = IncrementalHash.CreateHash(HashAlgorithmName.MD5);
                using var sha1 = IncrementalHash.CreateHash(HashAlgorithmName.SHA1);
                var crc = new Crc32();
                byte[] buffer = new byte[1 << 16];
                long size = 0, left = length < 0 ? long.MaxValue : length;
                using FileStream stream = new(path, FileMode.Open, FileAccess.Read, FileShare.Read, 1 << 16, FileOptions.SequentialScan);
                if (offset > 0) stream.Seek(offset, SeekOrigin.Begin);
                while (left > 0)
                {
                    stop.ThrowIfCancellationRequested();
                    int read = stream.Read(buffer, 0, (int)Math.Min(buffer.Length, left));
                    if (read == 0) break;
                    md5.AppendData(buffer, 0, read);
                    sha1.AppendData(buffer, 0, read);
                    crc.Append(buffer.AsSpan(0, read));
                    size += read;
                    left -= read;
                }
                return new RomHashes(Convert.ToHexStringLower(md5.GetHashAndReset()), crc.Hex, Convert.ToHexStringLower(sha1.GetHashAndReset()), size);
            }

            public static RomHashes Of(ReadOnlySpan<byte> bytes)
            {
                var crc = new Crc32();
                crc.Append(bytes);
                return new RomHashes(Convert.ToHexStringLower(MD5.HashData(bytes)), crc.Hex, Convert.ToHexStringLower(SHA1.HashData(bytes)), bytes.Length);
            }

            // Where a track lies is emusen-disc's alone, asked through the platform library as ImageSets asks; the reading and hashing are C#.
            public static IReadOnlyList<TrackHashes> Tracks(string sheetPath, CancellationToken stop)
            {
                string text = File.ReadAllText(sheetPath);
                IReadOnlyList<string> names = ImageSets.CompanionNames(text, Path.GetFileName(sheetPath));
                IReadOnlyList<string> paths = ImageSets.Companions(sheetPath);
                return ImageSets.TrackSpans(text, Path.GetFileName(sheetPath))
                    .Select(s => new TrackHashes(s.Track, names[s.File], s.Offset, Of(paths[s.File], s.Offset, s.Length ?? -1, stop)))
                    .ToArray();
            }
        }
    }

    // CRC-32 as zip and ScreenScraper compute it: polynomial 0xEDB88320, reflected, initial and final value all ones.
    public sealed class Crc32
    {
        private static readonly uint[] Table = BuildTable();
        private uint _value = 0xFFFFFFFF;

        public void Append(ReadOnlySpan<byte> bytes)
        {
            uint value = _value;
            foreach (byte b in bytes) value = Table[(value ^ b) & 0xFF] ^ (value >> 8);
            _value = value;
        }

        public uint Value => ~_value;

        public string Hex => Value.ToString("x8");

        private static uint[] BuildTable()
        {
            var table = new uint[256];
            for (uint i = 0; i < 256; i++)
            {
                uint c = i;
                for (int k = 0; k < 8; k++) c = (c & 1) != 0 ? 0xEDB88320 ^ (c >> 1) : c >> 1;
                table[i] = c;
            }
            return table;
        }
    }
}
