using System;
using System.IO;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // What a Game Boy image's header says about its board, read before the image reaches the library, with the two refusals the board makes - see Mercury_Memory.md §2.
    public readonly record struct GbCartridge(byte Type, string Board, int RomBanks, int RamLength, bool HasBattery, bool HasTimer, bool HasRumble, byte CgbFlag)
    {
        public const int HeaderLength = 0x0150;
        public const int RomBankSize = 0x4000;

        // The header's facts; an image shorter than the header, or of a type no board implements, is refused as the core refuses it.
        public static GbCartridge Read(byte[] image)
        {
            if (image.Length < HeaderLength) throw new InvalidDataException($"Not a Game Boy image: {image.Length} bytes is shorter than the 336-byte header.");
            byte type = image[0x0147];
            bool timer = type is 0x0F or 0x10;
            bool rumble = type is 0x1C or 0x1D or 0x1E or 0x22;
            string board = type switch
            {
                0x00 or 0x08 or 0x09 => "ROM",
                >= 0x01 and <= 0x03 => "MBC1",
                0x05 or 0x06 => "MBC2",
                >= 0x0F and <= 0x13 => timer ? "MBC3+RTC" : "MBC3",
                >= 0x19 and <= 0x1E => rumble ? "MBC5+Rumble" : "MBC5",
                _ => throw new NotSupportedException($"Cartridge type ${type:X2} is not implemented - see Mercury_Memory.md §4."),
            };
            bool battery = type is 0x03 or 0x06 or 0x09 or 0x0D or 0x0F or 0x10 or 0x13 or 0x1B or 0x1E or 0x22 or 0xFF;
            return new GbCartridge(type, board, image.Length / RomBankSize, RamLengthFor(image[0x0149], type), battery, timer, rumble, image[0x0143]);
        }

        // MBC2 carries 512 half-bytes on the chip itself and reports no size in the header.
        public static int RamLengthFor(byte code, byte type) => type is 0x05 or 0x06 ? 512 : code switch
        {
            0x02 => 8 * 1024,
            0x03 => 32 * 1024,
            0x04 => 128 * 1024,
            0x05 => 64 * 1024,
            _ => 0,
        };

        // A colour game by its $0143 flag, $80 or $C0.
        public bool ColourGame => CgbFlag is 0x80 or 0xC0;
    }
}
