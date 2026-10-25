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
        public const int RootDirectory = 20, FirstFile = 21;

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
            int exeAt = FirstFile + Iso9660Image.Sectors(config.Length);
            int total = exeAt + Iso9660Image.Sectors(exe.Length);
            var user = new byte[total * CdSector.User];

            byte[] licence = Encoding.ASCII.GetBytes("          Licensed  by          " + LicenceLine2(market));
            licence.CopyTo(user, 4 * CdSector.User);
            Iso9660Image.Write(user, RootDirectory, new[] { new IsoFile(ExeName, exeAt, exe), new IsoFile(ConfigName, FirstFile, config) });

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
    }
}
