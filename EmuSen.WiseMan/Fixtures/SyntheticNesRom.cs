using EmuSen.Cores.Nintendo.Moon.Memory;

namespace EmuSen.WiseMan.Fixtures
{
    // Builds a synthetic iNES image - never real game data. The NES counterpart of SyntheticRom.
    public static class SyntheticNesRom
    {
        public const int PrgBankSize = Cartridge.PrgBankSize;
        public const int ChrBankSize = Cartridge.ChrBankSize;

        // NOP fill, not zero: zero decodes as BRK, which vectors and misaligns a linear disassembly.
        public const byte Filler = 0xEA;

        // <patches> are (offset within PRG, bytes) pairs; PRG offset 0 is CPU $8000 on a 16K NROM.
        public static byte[] Build(
            int prgBanks = 1,
            int chrBanks = 1,
            int mapper = 0,
            Mirroring mirroring = Mirroring.Horizontal,
            bool battery = false,
            params (int Offset, byte[] Bytes)[] patches)
        {
            int prgLength = prgBanks * PrgBankSize;
            int chrLength = chrBanks * ChrBankSize;

            var image = new byte[Cartridge.HeaderSize + prgLength + chrLength];

            image[0] = (byte)'N';
            image[1] = (byte)'E';
            image[2] = (byte)'S';
            image[3] = 0x1A;
            image[4] = (byte)prgBanks;
            image[5] = (byte)chrBanks;

            byte flags6 = (byte)((mapper & 0x0F) << 4);
            if (mirroring == Mirroring.Vertical) flags6 |= 0x01;
            if (mirroring == Mirroring.FourScreen) flags6 |= 0x08;
            if (battery) flags6 |= 0x02;
            image[6] = flags6;
            image[7] = (byte)(mapper & 0xF0);

            var prg = new System.Span<byte>(image, Cartridge.HeaderSize, prgLength);
            prg.Fill(Filler);

            foreach (var (offset, bytes) in patches)
            {
                bytes.CopyTo(prg[offset..]);
            }

            // Reset lands at $8000, which is PRG offset 0 of the bank the board maps there.
            WriteVector(prg, 0xFFFC, 0x8000);
            WriteVector(prg, 0xFFFA, 0x8000);
            WriteVector(prg, 0xFFFE, 0x8000);

            return image;
        }

        // $C000-$FFFF is the last bank on every board here, and a 16K one mirrors its only bank there.
        private static void WriteVector(System.Span<byte> prg, int cpuAddress, int target)
        {
            int offset = prg.Length - PrgBankSize + (cpuAddress - 0xC000);
            prg[offset] = (byte)target;
            prg[offset + 1] = (byte)(target >> 8);
        }

        public static Cartridge LoadCartridge(params (int Offset, byte[] Bytes)[] patches) =>
            Cartridge.FromImage(Build(patches: patches));

        // MoonCore.LoadRom takes a path, so a test that needs a whole core needs a real file.
        public static string WriteTemp(byte[] image)
        {
            string path = Path.Combine(Path.GetTempPath(), $"moon-{Guid.NewGuid():N}.nes");
            File.WriteAllBytes(path, image);
            return path;
        }
    }
}
