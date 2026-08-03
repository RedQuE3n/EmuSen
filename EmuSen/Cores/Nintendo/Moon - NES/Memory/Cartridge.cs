using System;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Moon.Memory.Mappers;

namespace EmuSen.Cores.Nintendo.Moon.Memory
{
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

            int prgBanks = image[4];
            int chrBanks = image[5];
            MapperNumber = (image[6] >> 4) | (image[7] & 0xF0);

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
            7 => new AxRom(cart),
            _ => throw new NotSupportedException(
                $"iNES mapper {cart.MapperNumber} is not implemented - see Moon_Memory.md §4 for what is."),
        };

        private void LoadSram()
        {
            if (!HasBattery || string.IsNullOrEmpty(RomPath)) return;

            _savePath = Path.ChangeExtension(RomPath, ".srm");
            if (!File.Exists(_savePath)) return;

            byte[] saved = File.ReadAllBytes(_savePath);
            Array.Copy(saved, PrgRam, Math.Min(saved.Length, PrgRam.Length));
        }

        public void SaveSram()
        {
            if (!HasBattery || _savePath is null) return;
            File.WriteAllBytes(_savePath, PrgRam);
        }
    }
}
