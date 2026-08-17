using System;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Moon.Memory.Mappers;
using EmuSen.Galaxia.Library;

namespace EmuSen.Cores.Nintendo.Moon.Memory
{
    // How much of the header survived the copiers - see Moon_Memory.md §2.2.
    public enum HeaderTrust
    {
        Clean,

        // Byte 7 carried a mapper nibble this loader had to accept on faith.
        Unverifiable,

        // Byte 7 held a copier signature, so only byte 6 was believed.
        Archaic,
    }

    // One iNES/NES 2.0 image and the board it describes - see Moon_Memory.md §2.
    public sealed class Cartridge
    {
        public const int PrgBankSize = 0x4000;
        public const int ChrBankSize = 0x2000;
        public const int TrainerSize = 512;
        public const int HeaderSize = 16;

        [SkipInState] public byte[] PrgRom = Array.Empty<byte>();

        // CHR ROM when the board shipped one, otherwise 8K of CHR RAM the game writes itself.
        public byte[] Chr = Array.Empty<byte>();

        public byte[] PrgRam = Array.Empty<byte>();

        [SkipInState] public bool ChrIsRam;
        [SkipInState] public int MapperNumber;

        // How far the header can be trusted, which a differential run must know - see Moon_Memory.md §2.2.
        [SkipInState] public HeaderTrust Trust;
        [SkipInState] public bool HasBattery;
        [SkipInState] public bool IsNes20;
        [SkipInState] public Mirroring HeaderMirroring;
        [SkipInState] public string RomPath = "";

        [SkipInState] public IMapper Mapper = null!;

        // Stamped by the bus before every mapper write, for boards that reject back-to-back ones - see Moon_Memory.md §4.4.
        public long CpuCycle;

        [SkipInState] private string? _savePath;

        public int PrgBanks => PrgRom.Length / PrgBankSize;
        public int ChrBanks => Chr.Length / ChrBankSize;

        public static Cartridge Load(string path)
        {
            byte[] image = File.ReadAllBytes(path);
            var cart = new Cartridge { RomPath = path };
            cart.Parse(image);
            cart.Mapper = CreateMapper(cart);
            cart.LoadSram();
            return cart;
        }

        // The header alone, with no board built - see EmuSen_Galaxia.md §7.
        public static Cartridge Describe(byte[] image)
        {
            var cart = new Cartridge();
            cart.Parse(image);
            return cart;
        }

        public static bool IsBoardImplemented(int mapperNumber)
        {
            try
            {
                CreateMapper(new Cartridge { MapperNumber = mapperNumber, PrgRom = new byte[0x4000], Chr = new byte[0x2000] });
                return true;
            }
            catch (NotSupportedException)
            {
                return false;
            }
        }

        // Kept separate from Load so a test can build an image in memory - see EmuSen.WiseMan's SyntheticNesRom.
        public static Cartridge FromImage(byte[] image)
        {
            var cart = new Cartridge();
            cart.Parse(image);
            cart.Mapper = CreateMapper(cart);
            return cart;
        }

        private void Parse(byte[] image)
        {
            if (image.Length < HeaderSize ||
                image[0] != 'N' || image[1] != 'E' || image[2] != 'S' || image[3] != 0x1A)
            {
                throw new InvalidDataException("Not an iNES image: missing the \"NES\\x1A\" magic.");
            }

            // Byte 7 bits 2-3 reading exactly 0b10 is the whole NES 2.0 detection - see Moon_Memory.md §2.1.
            IsNes20 = (image[7] & 0x0C) == 0x08;

            // Either tell means byte 7 is not a header byte at all - see Moon_Memory.md §2.2.
            bool archaic = !IsNes20 &&
                ((image[7] & 0x0C) != 0x00 ||
                 image[12] != 0 || image[13] != 0 || image[14] != 0 || image[15] != 0);

            int prgBanks = image[4];
            int chrBanks = image[5];

            MapperNumber = image[6] >> 4;
            if (!archaic) MapperNumber |= image[7] & 0xF0;

            Trust = archaic ? HeaderTrust.Archaic
                : IsNes20 || (image[7] & 0xF0) == 0 ? HeaderTrust.Clean
                : HeaderTrust.Unverifiable;

            if (IsNes20)
            {
                MapperNumber |= (image[8] & 0x0F) << 8;
                prgBanks |= (image[9] & 0x0F) << 8;
                chrBanks |= (image[9] & 0xF0) << 4;
            }

            HasBattery = (image[6] & 0x02) != 0;
            bool hasTrainer = (image[6] & 0x04) != 0;

            HeaderMirroring = (image[6] & 0x08) != 0
                ? Mirroring.FourScreen
                : (image[6] & 0x01) != 0 ? Mirroring.Vertical : Mirroring.Horizontal;

            int offset = HeaderSize + (hasTrainer ? TrainerSize : 0);

            int prgLength = prgBanks * PrgBankSize;
            if (offset + prgLength > image.Length)
            {
                throw new InvalidDataException(
                    $"Header claims {prgBanks} PRG bank(s) ({prgLength} bytes) but the file holds {image.Length - offset}.");
            }

            PrgRom = new byte[prgLength];
            Array.Copy(image, offset, PrgRom, 0, prgLength);
            offset += prgLength;

            if (chrBanks == 0)
            {
                Chr = new byte[ChrBankSize];
                ChrIsRam = true;
            }
            else
            {
                int chrLength = chrBanks * ChrBankSize;
                Chr = new byte[chrLength];
                Array.Copy(image, offset, Chr, 0, Math.Min(chrLength, image.Length - offset));
                ChrIsRam = false;
            }

            // Header byte 8's PRG-RAM size is unreliable on iNES 1.0, so every board just gets 8K.
            PrgRam = new byte[0x2000];
        }

        private static IMapper CreateMapper(Cartridge cart) => cart.MapperNumber switch
        {
            0 => new Nrom(cart),
            1 => new Mmc1(cart),
            2 => new UxRom(cart),
            3 => new CnRom(cart),
            4 => new Mmc3(cart),
            7 => new AxRom(cart),
            9 => new Mmc2(cart),
            11 => new ColorDreams(cart),
            64 => new Rambo1(cart),
            65 => new IremH3001(cart),
            66 => new GxRom(cart),
            67 => new Sunsoft3(cart),
            68 => new Sunsoft4(cart),
            69 => new SunsoftFme7(cart),
            71 => new Camerica(cart),
            79 => new Nina003(cart),
            _ => throw new NotSupportedException(
                $"iNES mapper {cart.MapperNumber} is not implemented - see Moon_Memory.md §4 for what is."),
        };

        // --nobattery leaves _savePath null, which also disables SaveSram - see Moon_Memory.md §6.
        private void LoadSram()
        {
            if (!HasBattery || EmuSen.Cores.CoreOptions.BatteryRamDisabled || string.IsNullOrEmpty(RomPath)) return;

            // Still beside the ROM this pass; relocation is its own change - see EmuSen_Galaxia.md §6.
            _savePath = Path.ChangeExtension(RomPath, SaveLibrary.SramExtension);
            if (AtomicFile.TryRead(_savePath) is not { } saved) return;

            Array.Copy(saved, PrgRam, Math.Min(saved.Length, PrgRam.Length));
        }

        public void SaveSram()
        {
            if (!HasBattery || _savePath is null) return;
            AtomicFile.Write(_savePath, PrgRam);
        }
    }
}
