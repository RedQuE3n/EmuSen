using System.Text;

namespace EmuSen.WiseMan.Fixtures
{
    // A synthetic PlayStation 2 disc: its volume as 2,048-byte sectors, and for a CD the same as a raw mode 2 track.
    public sealed record Ps2Disc(byte[] Iso, byte[] DataTrack);

    // Builds an ECMA-119 volume whose root holds SYSTEM.CNF with a BOOT2 line and a program, never real game data - see Sapphire_Native.md §5.
    public static class SyntheticPs2Disc
    {
        public const string Program = "EMUS_000.02", Serial = "EMUS-00002";
        public const int CdRoot = 20, DvdRoot = 300;

        public static string SystemCnf(string vmode = "NTSC") => $"BOOT2 = cdrom0:\\{Program};1\r\nVER = 1.00\r\nVMODE = {vmode}\r\n";

        // A CD's volume with its root after the descriptors; Sony's licence text in sector 4 only where asked, to show it decides nothing.
        public static Ps2Disc Cd(string? config = null, bool licence = false) => Build(CdRoot, config ?? SystemCnf(), licence, raw: true);

        // A DVD's volume, 2,048-byte sectors only, its root and files past the first 600 KiB as a bridge volume places them [A] - see Sapphire_Native.md §5.
        public static Ps2Disc Dvd(string? config = null) => Build(DvdRoot, config ?? SystemCnf("PAL"), licence: false, raw: false);

        private static Ps2Disc Build(int root, string config, bool licence, bool raw)
        {
            byte[] cnf = Encoding.ASCII.GetBytes(config);
            byte[] program = SyntheticPs2Elf.Build();
            int cnfAt = root + 1, programAt = cnfAt + SyntheticPsxDisc.Sectors(cnf.Length), total = programAt + SyntheticPsxDisc.Sectors(program.Length);
            var user = new byte[total * CdSector.User];
            if (licence) Encoding.ASCII.GetBytes("          Licensed  by          Sony Computer Entertainment Amer  ica ").CopyTo(user, 4 * CdSector.User);
            var pvd = user.AsSpan(16 * CdSector.User, CdSector.User);
            pvd[0] = 1;
            Encoding.ASCII.GetBytes("CD001").CopyTo(pvd[1..]);
            pvd[6] = 1;
            Encoding.ASCII.GetBytes("PLAYSTATION".PadRight(32)).CopyTo(pvd[8..]);
            Both32(pvd, 80, (uint)total);
            Both16(pvd, 128, CdSector.User);
            Record(pvd, 156, "\0", root, CdSector.User, directory: true);
            user[17 * CdSector.User] = 0xFF;
            Encoding.ASCII.GetBytes("CD001").CopyTo(user, 17 * CdSector.User + 1);
            user[17 * CdSector.User + 6] = 1;
            var dir = user.AsSpan(root * CdSector.User, CdSector.User);
            int at = Record(dir, 0, "\0", root, CdSector.User, directory: true);
            at += Record(dir, at, "\u0001", root, CdSector.User, directory: true);
            at += Record(dir, at, Program + ";1", programAt, program.Length, directory: false);
            Record(dir, at, "SYSTEM.CNF;1", cnfAt, cnf.Length, directory: false);
            cnf.CopyTo(user, cnfAt * CdSector.User);
            program.CopyTo(user, programAt * CdSector.User);
            if (!raw) return new Ps2Disc(user, Array.Empty<byte>());
            var data = new byte[total * CdSector.Raw];
            for (int block = 0; block < total; block++)
                CdSector.Mode2Form1(block, user.AsSpan(block * CdSector.User, CdSector.User), block == total - 1 || block == root ? (byte)0x89 : (byte)0x08).CopyTo(data, block * CdSector.Raw);
            return new Ps2Disc(user, data);
        }

        // A directory record of ECMA-119 §9.1, padded to an even length; the length it took.
        private static int Record(Span<byte> d, int at, string name, int extent, int length, bool directory)
        {
            int size = 33 + name.Length + (name.Length % 2 == 0 ? 1 : 0);
            var r = d.Slice(at, size);
            r[0] = (byte)size;
            Both32(r, 2, (uint)extent);
            Both32(r, 10, (uint)length);
            r[25] = (byte)(directory ? 2 : 0);
            Both16(r, 28, 1);
            r[32] = (byte)name.Length;
            Encoding.ASCII.GetBytes(name).CopyTo(r[33..]);
            return size;
        }

        private static void Both16(Span<byte> d, int at, ushort v)
        {
            BitConverter.TryWriteBytes(d.Slice(at, 2), v);
            (d[at + 2], d[at + 3]) = ((byte)(v >> 8), (byte)v);
        }

        private static void Both32(Span<byte> d, int at, uint v)
        {
            BitConverter.TryWriteBytes(d.Slice(at, 4), v);
            (d[at + 4], d[at + 5], d[at + 6], d[at + 7]) = ((byte)(v >> 24), (byte)(v >> 16), (byte)(v >> 8), (byte)v);
        }
    }
}
