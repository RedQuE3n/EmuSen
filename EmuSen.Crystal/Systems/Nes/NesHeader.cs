using System.IO;

namespace EmuSen.Crystal.Systems.Nes
{
    // In the order the state's board fields number them - see Moon_Memory.md §3.
    public enum NesMirroring
    {
        Horizontal,
        Vertical,
        SingleScreenLower,
        SingleScreenUpper,
        FourScreen,
    }

    // How much of the header survived the copiers - see Moon_Memory.md §2.2.
    public enum NesHeaderTrust
    {
        Clean,
        Unverifiable,
        Archaic,
    }

    // An iNES or NES 2.0 header read without building a board; PRG RAM is 8 KB on every board; emusen-systems' reader unless EMUSEN_SYSTEMS_NATIVE=0 - see Moon_Memory.md §2, EmuSen_Backend.md §13.3.
    public sealed record NesHeader(int Mapper, int PrgBanks, int ChrBanks, bool HasBattery, bool HasTrainer, bool IsNes20, NesHeaderTrust Trust, NesMirroring Mirroring)
    {
        public const int Size = 16;
        public const int PrgBankSize = 0x4000;
        public const int ChrBankSize = 0x2000;
        public const int TrainerSize = 512;
        public const int PrgRamSize = 0x2000;

        public bool ChrIsRam => ChrBanks == 0;

        public static NesHeader Read(byte[] image) => SystemCode.Active ? SystemCode.NesHeader(image) : ReadManaged(image);

        // The C# reader alone, the failover and the parity class's oracle.
        public static NesHeader ReadManaged(byte[] image)
        {
            if (image.Length < Size || image[0] != 'N' || image[1] != 'E' || image[2] != 'S' || image[3] != 0x1A)
                throw new InvalidDataException("Not an iNES image: missing the \"NES\\x1A\" magic.");
            // Byte 7 bits 2-3 reading exactly 0b10 is the whole NES 2.0 detection - see Moon_Memory.md §2.1.
            bool nes20 = (image[7] & 0x0C) == 0x08;
            bool archaic = !nes20 && ((image[7] & 0x0C) != 0x00 || image[12] != 0 || image[13] != 0 || image[14] != 0 || image[15] != 0);
            int prg = image[4], chr = image[5];
            int mapper = image[6] >> 4;
            if (!archaic) mapper |= image[7] & 0xF0;
            var trust = archaic ? NesHeaderTrust.Archaic : nes20 || (image[7] & 0xF0) == 0 ? NesHeaderTrust.Clean : NesHeaderTrust.Unverifiable;
            if (nes20)
            {
                mapper |= (image[8] & 0x0F) << 8;
                prg |= (image[9] & 0x0F) << 8;
                chr |= (image[9] & 0xF0) << 4;
            }
            var mirroring = (image[6] & 0x08) != 0 ? NesMirroring.FourScreen : (image[6] & 0x01) != 0 ? NesMirroring.Vertical : NesMirroring.Horizontal;
            bool trainer = (image[6] & 0x04) != 0;
            int offset = Size + (trainer ? TrainerSize : 0);
            if (offset + (long)prg * PrgBankSize > image.Length)
                throw new InvalidDataException($"Header claims {prg} PRG bank(s) ({prg * PrgBankSize} bytes) but the file holds {image.Length - offset}.");
            return new NesHeader(mapper, prg, chr, (image[6] & 0x02) != 0, trainer, nes20, trust, mirroring);
        }
    }
}
