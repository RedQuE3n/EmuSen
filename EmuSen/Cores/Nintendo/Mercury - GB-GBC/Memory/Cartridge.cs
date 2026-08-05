using System;
using System.IO;
using System.Text;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Mercury.Memory.Mappers;
using EmuSen.Galaxia.Library;

namespace EmuSen.Cores.Nintendo.Mercury.Memory
{
    // What the $0143 flag says about colour support - see Mercury_Memory.md §2.1.
    public enum CgbSupport
    {
        None,
        Enhanced,
        Required,
    }

    // One Game Boy image and the board it describes - see Mercury_Memory.md §2.
    public sealed class Cartridge
    {
        public const int RomBankSize = 0x4000;
        public const int RamBankSize = 0x2000;

        public const int HeaderStart = 0x0134;
        public const int HeaderChecksumAddress = 0x014D;

        [SkipInState] public byte[] Rom = Array.Empty<byte>();

        public byte[] Ram = Array.Empty<byte>();

        [SkipInState] public string Title = "";
        [SkipInState] public byte CartridgeType;
        [SkipInState] public CgbSupport Cgb;
        [SkipInState] public bool SuperGameBoy;
        [SkipInState] public bool HasBattery;
        [SkipInState] public bool HasTimer;
        [SkipInState] public bool HasRumble;
        [SkipInState] public bool HeaderChecksumValid;
        [SkipInState] public string RomPath = "";

        [SkipInState] public IMapper Mapper = null!;

        private string? _savePath;

        public int RomBanks => Rom.Length / RomBankSize;
        public int RamBanks => Ram.Length / RamBankSize;

        public static Cartridge Load(string path)
        {
            var cart = FromImage(File.ReadAllBytes(path));
            cart.RomPath = path;
            cart._savePath = Path.ChangeExtension(path, ".srm");
            cart.LoadSram();
            return cart;
        }

        public static Cartridge FromImage(byte[] image)
        {
            if (image.Length < 0x0150)
            {
                throw new InvalidDataException($"Not a Game Boy image: {image.Length} bytes is shorter than the 336-byte header.");
            }

            var cart = new Cartridge { Rom = image };

            cart.CartridgeType = image[0x0147];
            cart.Cgb = image[0x0143] switch
            {
                0xC0 => CgbSupport.Required,
                0x80 => CgbSupport.Enhanced,
                _ => CgbSupport.None,
            };
            cart.SuperGameBoy = image[0x0146] == 0x03;
            cart.Title = ReadTitle(image, cart.Cgb != CgbSupport.None);

            DescribeType(cart.CartridgeType, out bool battery, out bool timer, out bool rumble);
            cart.HasBattery = battery;
            cart.HasTimer = timer;
            cart.HasRumble = rumble;

            cart.Ram = new byte[RamSizeFor(image[0x0149], cart.CartridgeType)];
            cart.HeaderChecksumValid = ComputeHeaderChecksum(image) == image[HeaderChecksumAddress];

            // The header's own size byte is advisory; the file is the authority - see Mercury_Memory.md §2.2.
            cart.Mapper = BuildMapper(cart);
            return cart;
        }

        // Trailing NULs and, on a colour cart, the manufacturer/CGB bytes are not part of the name.
        private static string ReadTitle(byte[] image, bool isCgb)
        {
            int length = isCgb ? 11 : 16;
            var text = new StringBuilder(length);

            for (int i = 0; i < length; i++)
            {
                byte b = image[HeaderStart + i];
                if (b == 0x00) break;
                text.Append(b >= 0x20 && b < 0x7F ? (char)b : ' ');
            }

            return text.ToString().TrimEnd();
        }

        // x = x - byte - 1 across $0134-$014C; a real boot ROM locks up when this disagrees.
        public static byte ComputeHeaderChecksum(byte[] image)
        {
            byte checksum = 0;
            for (int address = HeaderStart; address <= 0x014C; address++)
            {
                checksum = (byte)(checksum - image[address] - 1);
            }
            return checksum;
        }

        private static void DescribeType(byte type, out bool battery, out bool timer, out bool rumble)
        {
            battery = type is 0x03 or 0x06 or 0x09 or 0x0D or 0x0F or 0x10 or 0x13 or 0x1B or 0x1E or 0x22 or 0xFF;
            timer = type is 0x0F or 0x10;
            rumble = type is 0x1C or 0x1D or 0x1E or 0x22;
        }

        // MBC2 carries 512 half-bytes on the chip itself and reports no size in the header.
        private static int RamSizeFor(byte code, byte cartridgeType)
        {
            if (cartridgeType is 0x05 or 0x06) return 512;

            return code switch
            {
                0x02 => 8 * 1024,
                0x03 => 32 * 1024,
                0x04 => 128 * 1024,
                0x05 => 64 * 1024,
                _ => 0,
            };
        }

        private static IMapper BuildMapper(Cartridge cart) => cart.CartridgeType switch
        {
            0x00 or 0x08 or 0x09 => new NoMbc(cart),
            >= 0x01 and <= 0x03 => new Mbc1(cart),
            0x05 or 0x06 => new Mbc2(cart),
            >= 0x0F and <= 0x13 => new Mbc3(cart),
            >= 0x19 and <= 0x1E => new Mbc5(cart),
            var other => throw new NotSupportedException(
                $"Cartridge type ${other:X2} is not implemented - see Mercury_Memory.md §4."),
        };

        private void LoadSram()
        {
            if (!HasBattery || _savePath is null || Ram.Length == 0) return;
            if (AtomicFile.TryRead(_savePath) is not { } saved) return;

            Array.Copy(saved, Ram, Math.Min(saved.Length, Ram.Length));
        }

        public void SaveSram()
        {
            if (!HasBattery || _savePath is null || Ram.Length == 0) return;
            AtomicFile.Write(_savePath, Ram);
        }
    }
}
