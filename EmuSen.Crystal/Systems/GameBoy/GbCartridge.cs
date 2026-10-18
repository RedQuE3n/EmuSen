using System;
using System.IO;

namespace EmuSen.Crystal.Systems.GameBoy
{
    // What a Game Boy image's header says about its board, every type named, an engine refusing at create what it does not build; emusen-systems' reader unless EMUSEN_SYSTEMS_NATIVE=0 - see Mercury_Memory.md §2, EmuSen_Backend.md §17.2 L1.
    public readonly record struct GbCartridge(byte Type, string Board, int RomBanks, int RamLength, bool HasBattery, bool HasTimer, bool HasRumble, byte CgbFlag)
    {
        public const int HeaderLength = 0x0150;
        public const int RomBankSize = 0x4000;

        // The header's facts; an image shorter than the header is refused.
        public static GbCartridge Read(byte[] image) => SystemCode.Active ? SystemCode.GbHeader(image) : ReadManaged(image);

        // The C# reader alone, the failover and the parity class's oracle.
        public static GbCartridge ReadManaged(byte[] image) => ReadManaged(image, image.Length);

        // The header from an image's first bytes and its whole length, so a view reads 336 bytes of a ROM and not all of it.
        public static GbCartridge ReadManaged(ReadOnlySpan<byte> image, int length)
        {
            if (length < HeaderLength || image.Length < HeaderLength) throw new InvalidDataException($"Not a Game Boy image: {length} bytes is shorter than the 336-byte header.");
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
                >= 0x0B and <= 0x0D => "MMM01",
                0x20 => "MBC6",
                0x22 => "MBC7",
                0xFC => "Pocket Camera",
                0xFD => "TAMA5",
                0xFE => "HuC3",
                0xFF => "HuC1",
                _ => "unknown",
            };
            bool battery = type is 0x03 or 0x06 or 0x09 or 0x0D or 0x0F or 0x10 or 0x13 or 0x1B or 0x1E or 0x22 or 0xFF;
            return new GbCartridge(type, board, length / RomBankSize, RamLengthFor(image[0x0149], type), battery, timer, rumble, image[0x0143]);
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
