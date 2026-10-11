using System.Text;

namespace EmuSen.WiseMan.Fixtures
{
    // A synthetic PlayStation disc: its data track as user sectors and as raw ones, a CD-DA track and the cue sheet joining them.
    public sealed record PsxDisc(byte[] Iso, byte[] DataTrack, byte[] AudioTrack, string Cue)
    {
        // The set as a dump holds it, the sheet naming the two track files beside it; the sheet's path.
        public string WriteSet(string directory, string name)
        {
            File.WriteAllBytes(Path.Combine(directory, $"{name} (Track 1).bin"), DataTrack);
            File.WriteAllBytes(Path.Combine(directory, $"{name} (Track 2).bin"), AudioTrack);
            string cue = Path.Combine(directory, $"{name}.cue");
            File.WriteAllText(cue, Cue.Replace("{name}", name));
            return cue;
        }
    }

    // Builds the disc from ECMA-119's volume and directory records and psx-spx's system area, never real game data - see Diamond_Native.md §5.2.
    public static class SyntheticPsxDisc
    {
        public const string ExeName = "EMUS_000.01", ConfigName = "SYSTEM.CNF";
        public const int PrimaryVolume = 16, PathTableL = 18, PathTableM = 19, RootDirectory = 20, FirstFile = 21;

        // The licence's two lines for each market (psx-spx "System Area").
        public static string LicenceLine2(string market) => market switch
        {
            "Japan" => "Sony Computer Entertainment Inc.\n",
            "Europe" => "Sony Computer Entertainment Euro pe   ",
            _ => "Sony Computer Entertainment Amer  ica ",
        };

        public static string SystemCnf => $"BOOT = cdrom:\\{ExeName};1\r\nTCB = 4\r\nEVENT = 10\r\nSTACK = 801FFFF0\r\n";

        // Mode 2 form 1 data, as PlayStation discs carry, or mode 1; the audio track's tone lasts the seconds given after its two-second pregap.
        public static PsxDisc Build(string market = "Americas", int mode = 2, byte[]? exe = null, int audioSeconds = 1)
        {
            exe ??= SyntheticPsExe.Build(market: market == "Americas" ? "North America" : market);
            byte[] config = Encoding.ASCII.GetBytes(SystemCnf);
            int exeAt = FirstFile + Sectors(config.Length);
            int total = exeAt + Sectors(exe.Length);
            var user = new byte[total * CdSector.User];

            byte[] licence = Encoding.ASCII.GetBytes("          Licensed  by          " + LicenceLine2(market));
            licence.CopyTo(user, 4 * CdSector.User);
            PrimaryVolumeDescriptor(user.AsSpan(PrimaryVolume * CdSector.User, CdSector.User), total);
            user[17 * CdSector.User] = 0xFF;
            Encoding.ASCII.GetBytes("CD001").CopyTo(user, 17 * CdSector.User + 1);
            user[17 * CdSector.User + 6] = 1;
            PathTable(user.AsSpan(PathTableL * CdSector.User), littleEndian: true);
            PathTable(user.AsSpan(PathTableM * CdSector.User), littleEndian: false);
            var root = user.AsSpan(RootDirectory * CdSector.User, CdSector.User);
            int at = Record(root, 0, "\0", RootDirectory, CdSector.User, directory: true);
            at += Record(root, at, "\u0001", RootDirectory, CdSector.User, directory: true);
            at += Record(root, at, ExeName + ";1", exeAt, exe.Length, directory: false);
            Record(root, at, ConfigName + ";1", FirstFile, config.Length, directory: false);
            config.CopyTo(user, FirstFile * CdSector.User);
            exe.CopyTo(user, exeAt * CdSector.User);

            var ends = new HashSet<int> { 17, RootDirectory, exeAt - 1, total - 1 };
            var data = new byte[total * CdSector.Raw];
            for (int block = 0; block < total; block++)
            {
                var sector = user.AsSpan(block * CdSector.User, CdSector.User);
                byte[] raw = mode == 1 ? CdSector.Mode1(block, sector)
                    : block is >= 12 and < 16 ? CdSector.Mode2Form2(block, new byte[CdSector.Form2User])
                    : CdSector.Mode2Form1(block, sector, ends.Contains(block) ? (byte)0x89 : (byte)0x08);
                raw.CopyTo(data, block * CdSector.Raw);
            }

            string cue = $"FILE \"{{name}} (Track 1).bin\" BINARY\n  TRACK 01 MODE{mode}/2352\n    INDEX 01 00:00:00\n"
                + "FILE \"{name} (Track 2).bin\" BINARY\n  TRACK 02 AUDIO\n    INDEX 00 00:00:00\n    INDEX 01 00:02:00\n";
            return new PsxDisc(user, data, Tone(audioSeconds), cue);
        }

        // Two seconds of silence, then a square wave of 441 Hz, left and right in opposite phase, as 16-bit little-endian stereo.
        public static byte[] Tone(int seconds)
        {
            const int pregap = 2 * 75 * CdSector.Raw;
            var audio = new byte[pregap + seconds * 75 * CdSector.Raw];
            for (int i = 0, frame = 0; pregap + i < audio.Length; i += 4, frame++)
            {
                short left = (short)(frame / 50 % 2 == 0 ? 8000 : -8000);
                BitConverter.TryWriteBytes(audio.AsSpan(pregap + i, 2), left);
                BitConverter.TryWriteBytes(audio.AsSpan(pregap + i + 2, 2), (short)-left);
            }
            return audio;
        }

        public static int Sectors(int bytes) => Math.Max(1, (bytes + CdSector.User - 1) / CdSector.User);

        private static void PrimaryVolumeDescriptor(Span<byte> d, int blocks)
        {
            d.Fill(0);
            d[0] = 1;
            Text(d, 1, "CD001", 5);
            d[6] = 1;
            Text(d, 8, "PLAYSTATION", 32);
            Text(d, 40, "EMUSEN_SYNTHETIC", 32);
            Both32(d, 80, (uint)blocks);
            Both16(d, 120, 1);
            Both16(d, 124, 1);
            Both16(d, 128, CdSector.User);
            Both32(d, 132, 10);
            BitConverter.TryWriteBytes(d.Slice(140, 4), (uint)PathTableL);
            Big32(d, 148, PathTableM);
            Record(d, 156, "\0", RootDirectory, CdSector.User, directory: true);
            Text(d, 190, "", 128);
            Text(d, 318, "EMUSEN", 128);
            Text(d, 446, "", 128);
            Text(d, 574, "PLAYSTATION", 128);
            Text(d, 702, "", 37);
            Text(d, 739, "", 37);
            Text(d, 776, "", 37);
            foreach (int at in new[] { 813, 830, 847, 864 }) Text(d, at, at == 813 ? "2026101100000000" : "0000000000000000", 16);
            d[881] = 1;
        }

        private static void PathTable(Span<byte> t, bool littleEndian)
        {
            t[0] = 1;
            if (littleEndian) BitConverter.TryWriteBytes(t.Slice(2, 4), (uint)RootDirectory);
            else Big32(t, 2, RootDirectory);
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

        private static void Both32(Span<byte> d, int at, uint v)
        {
            BitConverter.TryWriteBytes(d.Slice(at, 4), v);
            Big32(d, at + 4, v);
        }

        private static void Big32(Span<byte> d, int at, int v) => Big32(d, at, (uint)v);

        private static void Big32(Span<byte> d, int at, uint v)
        {
            d[at] = (byte)(v >> 24);
            d[at + 1] = (byte)(v >> 16);
            d[at + 2] = (byte)(v >> 8);
            d[at + 3] = (byte)v;
        }
    }
}
