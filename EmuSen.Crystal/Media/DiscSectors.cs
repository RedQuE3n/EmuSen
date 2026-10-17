using System;
using System.Text;

namespace EmuSen.Crystal.Media
{
    // A data track's first sectors as user data, from an .iso's 2,048-byte sectors or a raw track's 2,352-byte ones, and the ISO 9660 lookup a pack's recogniser reads a file by - see EmuSen_CoreAPI_Extensions.md §8.4.
    public sealed class DiscSectors
    {
        public const int User = 2048, Raw = 2352;

        private static readonly byte[] Sync = { 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00 };

        private readonly byte[] _bytes;
        private readonly bool _raw;

        public DiscSectors(byte[] head)
        {
            _bytes = head;
            _raw = head.Length >= Raw && head.AsSpan(0, 12).SequenceEqual(Sync);
        }

        // Whether the bytes are a raw track, every sector with its sync and header.
        public bool IsRaw => _raw;

        // Sector n's user data, or empty where the head does not reach it or n is before the first.
        public ReadOnlySpan<byte> Sector(long n)
        {
            if (n < 0) return ReadOnlySpan<byte>.Empty;
            if (!_raw) return (n + 1) * User <= _bytes.Length ? _bytes.AsSpan((int)(n * User), User) : ReadOnlySpan<byte>.Empty;
            long at = n * Raw;
            if (at + Raw > _bytes.Length) return ReadOnlySpan<byte>.Empty;
            int data = _bytes[at + 15] == 1 ? 16 : 24;
            return _bytes.AsSpan((int)at + data, User);
        }

        // The bytes of a file in the volume's root directory, by its name without the version, where the head holds them; null otherwise, a record running past its sector ending that sector's scan - see EmuSen_Backend.md §13.3.
        public byte[]? RootFile(string name)
        {
            var pvd = Sector(16);
            if (pvd.Length < 190 || pvd[0] != 1 || !pvd.Slice(1, 5).SequenceEqual("CD001"u8)) return null;
            long extent = BitConverter.ToInt32(pvd.Slice(156 + 2, 4)), size = BitConverter.ToInt32(pvd.Slice(156 + 10, 4));
            for (long block = 0; block * User < size; block++)
            {
                var dir = Sector(extent + block);
                for (int at = 0; at + 33 < dir.Length && dir[at] != 0; at += dir[at])
                {
                    if (at + 33 + dir[at + 32] > dir.Length) break;
                    string entry = Encoding.ASCII.GetString(dir.Slice(at + 33, dir[at + 32]));
                    if (!string.Equals(entry.Split(';')[0], name, StringComparison.OrdinalIgnoreCase)) continue;
                    long start = BitConverter.ToInt32(dir.Slice(at + 2, 4));
                    int length = BitConverter.ToInt32(dir.Slice(at + 10, 4));
                    if (length < 0) return null;
                    for (long done = 0, s = start; done < length; done += User, s++)
                        if (Sector(s).IsEmpty) return null;
                    var file = new byte[length];
                    for (int done = 0; done < length; done += User) Sector(start + done / User)[..Math.Min(User, length - done)].CopyTo(file.AsSpan(done));
                    return file;
                }
            }
            return null;
        }
    }
}
