using System.Text;

namespace EmuSen.WiseMan.Fixtures
{
    // A file of a synthetic volume: its name without the version, the sector it starts at and its bytes.
    public sealed record IsoFile(string Name, int Extent, byte[] Data);

    // ECMA-119's volume written into 2,048-byte user sectors: the primary descriptor, the terminator, both path tables and one root directory with its files, for every synthetic disc - see Sapphire_Native.md §14.2.
    public static class Iso9660Image
    {
        public const int PrimaryVolume = 16, Terminator = 17, PathTableL = 18, PathTableM = 19;

        public static int Sectors(int bytes) => Math.Max(1, (bytes + CdSector.User - 1) / CdSector.User);

        // The descriptors at 16 and 17, the path tables at 18 and 19 unless placed elsewhere, and the root at its sector, each file's bytes at its extent; the volume spans every sector given.
        public static void Write(byte[] user, int root, IReadOnlyList<IsoFile> files, string system = "PLAYSTATION", (int L, int M)? pathTables = null)
        {
            var (l, m) = pathTables ?? (PathTableL, PathTableM);
            PrimaryVolumeDescriptor(user.AsSpan(PrimaryVolume * CdSector.User, CdSector.User), user.Length / CdSector.User, root, system, l, m);
            TerminatorAt(user, Terminator);
            PathTable(user.AsSpan(l * CdSector.User), root, littleEndian: true);
            PathTable(user.AsSpan(m * CdSector.User), root, littleEndian: false);
            var dir = user.AsSpan(root * CdSector.User, CdSector.User);
            int at = Record(dir, 0, "\0", root, CdSector.User, directory: true);
            at += Record(dir, at, "\u0001", root, CdSector.User, directory: true);
            foreach (IsoFile f in files)
            {
                at += Record(dir, at, f.Name + ";1", f.Extent, f.Data.Length, directory: false);
                f.Data.CopyTo(user, f.Extent * CdSector.User);
            }
        }

        // The set terminator of ECMA-119 §8.3 at a sector.
        public static void TerminatorAt(byte[] user, int sector)
        {
            user[sector * CdSector.User] = 0xFF;
            Encoding.ASCII.GetBytes("CD001").CopyTo(user, sector * CdSector.User + 1);
            user[sector * CdSector.User + 6] = 1;
        }

        public static void PrimaryVolumeDescriptor(Span<byte> d, int blocks, int root, string system, int pathTableL = PathTableL, int pathTableM = PathTableM)
        {
            d.Fill(0);
            d[0] = 1;
            Text(d, 1, "CD001", 5);
            d[6] = 1;
            Text(d, 8, system, 32);
            Text(d, 40, "EMUSEN_SYNTHETIC", 32);
            Both32(d, 80, (uint)blocks);
            Both16(d, 120, 1);
            Both16(d, 124, 1);
            Both16(d, 128, CdSector.User);
            Both32(d, 132, 10);
            BitConverter.TryWriteBytes(d.Slice(140, 4), (uint)pathTableL);
            Big32(d, 148, (uint)pathTableM);
            Record(d, 156, "\0", root, CdSector.User, directory: true);
            Text(d, 190, "", 128);
            Text(d, 318, "EMUSEN", 128);
            Text(d, 446, "", 128);
            Text(d, 574, system, 128);
            Text(d, 702, "", 37);
            Text(d, 739, "", 37);
            Text(d, 776, "", 37);
            foreach (int at in new[] { 813, 830, 847, 864 }) Text(d, at, at == 813 ? "2026101100000000" : "0000000000000000", 16);
            d[881] = 1;
        }

        private static void PathTable(Span<byte> t, int root, bool littleEndian)
        {
            t[0] = 1;
            if (littleEndian) BitConverter.TryWriteBytes(t.Slice(2, 4), (uint)root);
            else Big32(t, 2, (uint)root);
            t[littleEndian ? 6 : 7] = 1;
        }

        // A directory record of ECMA-119 §9.1, padded to an even length; the length it took.
        private static int Record(Span<byte> d, int at, string name, int extent, int length, bool directory)
        {
            int size = 33 + name.Length + (name.Length % 2 == 0 ? 1 : 0);
            var r = d.Slice(at, size);
            r[0] = (byte)size;
            Both32(r, 2, (uint)extent);
            Both32(r, 10, (uint)length);
            (r[18], r[19], r[20]) = ((byte)126, (byte)10, (byte)11);
            r[25] = (byte)(directory ? 2 : 0);
            Both16(r, 28, 1);
            r[32] = (byte)name.Length;
            Encoding.ASCII.GetBytes(name).CopyTo(r[33..]);
            return size;
        }

        private static void Text(Span<byte> d, int at, string text, int width) => Encoding.ASCII.GetBytes(text.PadRight(width)[..width]).CopyTo(d[at..]);

        private static void Both16(Span<byte> d, int at, ushort v)
        {
            BitConverter.TryWriteBytes(d.Slice(at, 2), v);
            d[at + 2] = (byte)(v >> 8);
            d[at + 3] = (byte)v;
        }

        internal static void Both32(Span<byte> d, int at, uint v)
        {
            BitConverter.TryWriteBytes(d.Slice(at, 4), v);
            Big32(d, at + 4, v);
        }

        private static void Big32(Span<byte> d, int at, uint v)
        {
            d[at] = (byte)(v >> 24);
            d[at + 1] = (byte)(v >> 16);
            d[at + 2] = (byte)(v >> 8);
            d[at + 3] = (byte)v;
        }
    }
}
