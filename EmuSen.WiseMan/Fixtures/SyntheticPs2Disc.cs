using System.Text;

namespace EmuSen.WiseMan.Fixtures
{
    // A synthetic PlayStation 2 disc: its volume as 2,048-byte sectors; a CD's raw mode 2 track and cue sheet; a dual-layer DVD's first sector of layer 1.
    public sealed record Ps2Disc(byte[] Iso, byte[] DataTrack, string? Cue, int? LayerBreak)
    {
        public bool IsCd => Cue is not null;

        // The set as a dump holds it, a CD as its sheet and track and a DVD as one image; the path to open.
        public string WriteSet(string directory, string name)
        {
            if (!IsCd)
            {
                string iso = Path.Combine(directory, $"{name}.iso");
                File.WriteAllBytes(iso, Iso);
                return iso;
            }
            File.WriteAllBytes(Path.Combine(directory, $"{name}.bin"), DataTrack);
            string cue = Path.Combine(directory, $"{name}.cue");
            File.WriteAllText(cue, Cue!.Replace("{name}", name));
            return cue;
        }
    }

    // Builds an ECMA-119 volume whose root holds SYSTEM.CNF with a BOOT2 line and a program, as a CD, a DVD or a dual-layer DVD, never real game data - see Sapphire_Native.md §5 and §14.2.
    public static class SyntheticPs2Disc
    {
        public const string Program = "EMUS_000.02", Serial = "EMUS-00002", Layer1File = "LAYER1.BIN";
        public const int CdRoot = 20, DvdRoot = 300, Layer1FileOffset = 32;

        public static string SystemCnf(string vmode = "NTSC") => $"BOOT2 = cdrom0:\\{Program};1\r\nVER = 1.00\r\nVMODE = {vmode}\r\n";

        // A CD's volume with its root after the descriptors, its raw track and sheet; Sony's licence text in sector 4 only where asked, to show it decides nothing.
        public static Ps2Disc Cd(string? config = null, bool licence = false, byte[]? program = null) =>
            Build(CdRoot, config ?? SystemCnf(), licence, raw: true, program, layer0: null);

        // A DVD's volume, 2,048-byte sectors only, its root and files past the first 600 KiB as a bridge volume places them [A] - see Sapphire_Native.md §5.
        public static Ps2Disc Dvd(string? config = null, byte[]? program = null) =>
            Build(DvdRoot, config ?? SystemCnf("PAL"), licence: false, raw: false, program, layer0: null);

        // Layer 0 of the sectors given, a multiple of 16 (ECMA-267 §25), a file past the break and the descriptors again at layer 1's sector 16 [A] - see Sapphire_Native.md §14.2.
        public static Ps2Disc DualLayerDvd(int layer0 = 1024, string? config = null, byte[]? program = null)
        {
            if (layer0 % 16 != 0) throw new ArgumentException($"layer 0 of {layer0} sectors is no multiple of 16", nameof(layer0));
            return Build(DvdRoot, config ?? SystemCnf("PAL"), licence: false, raw: false, program, layer0);
        }

        // The file past the break: each byte its offset's low byte mixed with its sector, so a sector read from the wrong place shows.
        public static byte[] Layer1Pattern(int length) => Enumerable.Range(0, length).Select(i => (byte)(i ^ (i >> 11) * 0x5B)).ToArray();

        private static Ps2Disc Build(int root, string config, bool licence, bool raw, byte[]? program, int? layer0)
        {
            byte[] cnf = Encoding.ASCII.GetBytes(config);
            program ??= SyntheticPs2Elf.Build();
            int cnfAt = root + 1, programAt = cnfAt + Iso9660Image.Sectors(cnf.Length), total = programAt + Iso9660Image.Sectors(program.Length);
            var files = new List<IsoFile> { new(Program, programAt, program), new("SYSTEM.CNF", cnfAt, cnf) };
            if (layer0 is int l0)
            {
                if (l0 < total) throw new ArgumentException($"layer 0 of {l0} sectors cannot hold the {total} the volume's start needs", nameof(layer0));
                byte[] far = Layer1Pattern(3 * CdSector.User + 100);
                files.Insert(1, new IsoFile(Layer1File, l0 + Layer1FileOffset, far));
                total = l0 + Layer1FileOffset + Iso9660Image.Sectors(far.Length);
            }
            var user = new byte[total * CdSector.User];
            if (licence) Encoding.ASCII.GetBytes("          Licensed  by          Sony Computer Entertainment Amer  ica ").CopyTo(user, 4 * CdSector.User);
            Iso9660Image.Write(user, root, files);
            if (layer0 is int l1)
            {
                user.AsSpan(Iso9660Image.PrimaryVolume * CdSector.User, CdSector.User).CopyTo(user.AsSpan((l1 + Iso9660Image.PrimaryVolume) * CdSector.User));
                Iso9660Image.TerminatorAt(user, l1 + Iso9660Image.Terminator);
            }
            if (!raw) return new Ps2Disc(user, Array.Empty<byte>(), null, layer0);
            var data = new byte[total * CdSector.Raw];
            for (int block = 0; block < total; block++)
                CdSector.Mode2Form1(block, user.AsSpan(block * CdSector.User, CdSector.User), block == total - 1 || block == root ? (byte)0x89 : (byte)0x08).CopyTo(data, block * CdSector.Raw);
            return new Ps2Disc(user, data, "FILE \"{name}.bin\" BINARY\n  TRACK 01 MODE2/2352\n    INDEX 01 00:00:00\n", null);
        }
    }
}
