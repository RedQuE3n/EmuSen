using System;
using System.Text;
using EmuSen.Crystal.Systems;

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
        public byte[]? RootFile(string name) => Reach(name, out _);

        // The image's bytes up to the end of sector n, in this head's sector size.
        private long EndOf(long n) => (n + 1) * (_raw ? Raw : User);

        // A root file's bytes, or null with needs the image's bytes it lies within where it is past the head's end, else 0 - see Sapphire_Native.md §3.2.
        public byte[]? Reach(string name, out long needs)
        {
            needs = 0;
            var pvd = Sector(16);
            if (pvd.Length < 190 || pvd[0] != 1 || !pvd.Slice(1, 5).SequenceEqual("CD001"u8)) return null;
            long extent = BitConverter.ToInt32(pvd.Slice(156 + 2, 4)), size = BitConverter.ToInt32(pvd.Slice(156 + 10, 4));
            for (long block = 0; block * User < size; block++)
            {
                var dir = Sector(extent + block);
                if (dir.IsEmpty && extent + block >= 0)
                {
                    needs = EndOf(extent + block);
                    return null;
                }
                for (int at = 0; at + 33 < dir.Length && dir[at] != 0; at += dir[at])
                {
                    if (at + 33 + dir[at + 32] > dir.Length) break;
                    string entry = Encoding.ASCII.GetString(dir.Slice(at + 33, dir[at + 32]));
                    if (!string.Equals(entry.Split(';')[0], name, StringComparison.OrdinalIgnoreCase)) continue;
                    long start = BitConverter.ToInt32(dir.Slice(at + 2, 4));
                    int length = BitConverter.ToInt32(dir.Slice(at + 10, 4));
                    if (length < 0 || start < 0) return null;
                    for (long done = 0, s = start; done < length; done += User, s++)
                        if (Sector(s).IsEmpty)
                        {
                            needs = EndOf(start + (length - 1) / User);
                            return null;
                        }
                    var file = new byte[length];
                    for (int done = 0; done < length; done += User) Sector(start + done / User)[..Math.Min(User, length - done)].CopyTo(file.AsSpan(done));
                    return file;
                }
            }
            return null;
        }

        // How far into an image a recogniser reads for a root file, past where a DVD's bridge volume puts its directory and first files - see Sapphire_Native.md §3.2.
        public const int RootFileLimit = 32 << 20;

        // A root file read through the probe from the first asked bytes on, asking for as much as the volume places it in, up to limit; null where absent, cut short or past the limit.
        public static byte[]? RootFileThrough(IMediaProbe probe, int asked, byte[] head, string name, int limit)
        {
            while (true)
            {
                if (new DiscSectors(head).Reach(name, out long needs) is { } file) return file;
                if (needs <= asked || needs > limit || head.Length < asked) return null;
                asked = (int)needs;
                head = probe.Read(asked);
            }
        }
    }
}
