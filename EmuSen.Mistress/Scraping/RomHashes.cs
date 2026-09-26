using System;
using System.IO;
using System.Security.Cryptography;

namespace EmuSen.Mistress.Scraping
{
    // The three hashes jeuInfos asks for, from one read; Md5 is RomHash's value when the bytes are the whole file - see EmuSen_BigPicture.md §5.2.
    public sealed record RomHashes(string Md5, string Crc32, string Sha1, long Size)
    {
        public static RomHashes Of(string path)
        {
            using var md5 = IncrementalHash.CreateHash(HashAlgorithmName.MD5);
            using var sha1 = IncrementalHash.CreateHash(HashAlgorithmName.SHA1);
            var crc = new Crc32();
            byte[] buffer = new byte[1 << 16];
            long size = 0;
            using FileStream stream = new(path, FileMode.Open, FileAccess.Read, FileShare.Read, 1 << 16, FileOptions.SequentialScan);
            for (int read; (read = stream.Read(buffer, 0, buffer.Length)) > 0; size += read)
            {
                md5.AppendData(buffer, 0, read);
                sha1.AppendData(buffer, 0, read);
                crc.Append(buffer.AsSpan(0, read));
            }
            return new RomHashes(Convert.ToHexStringLower(md5.GetHashAndReset()), crc.Hex, Convert.ToHexStringLower(sha1.GetHashAndReset()), size);
        }

        public static RomHashes Of(byte[] bytes)
        {
            var crc = new Crc32();
            crc.Append(bytes);
            return new RomHashes(Convert.ToHexStringLower(MD5.HashData(bytes)), crc.Hex, Convert.ToHexStringLower(SHA1.HashData(bytes)), bytes.Length);
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
