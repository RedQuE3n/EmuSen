using System;
using System.IO;
using System.Text;

namespace EmuSen.Cores.Nintendo.Mars.Rom
{
    // How a file's words are arranged before normalisation; the extension is not evidence - see Mars_Rom.md §1.
    public enum RomByteOrder
    {
        BigEndian,
        ByteSwapped,
        LittleEndian,
    }

    // What the ED64 homebrew convention declares; a commercial cartridge declares nothing - see Mars_Rom.md §3.
    public enum N64SaveType
    {
        Unknown,
        None,
        Eeprom4k,
        Eeprom16k,
        Sram256k,
        SramBanked768k,
        FlashRam,
        Sram1M,
    }

    // One Nintendo 64 image, normalised to big-endian, with its header read - see Mars_Rom.md §2.
    public sealed class RomImage
    {
        public const uint Magic = 0x80371240;

        public const int HeaderLength = 0x40;
        public const int BootCodeLength = 0xFC0;

        // The header plus IPL3, which is the least a bootable image can be - see Mars_Rom.md §2.1.
        public const int MinimumLength = HeaderLength + BootCodeLength;

        public const int TitleOffset = 0x20;
        public const int TitleLength = 20;

        public byte[] Rom = Array.Empty<byte>();
        public string RomPath = "";

        public RomByteOrder SourceByteOrder;

        public uint ClockRate;
        public uint EntryPoint;
        public uint LibultraVersion;
        public uint Crc1;
        public uint Crc2;

        public string Title = "";
        public char CategoryCode;
        public string UniqueCode = "";
        public char DestinationCode;
        public byte Version;

        // A convention over the destination code rather than a field of its own - see Mars_Rom.md §2.2.
        public bool IsPal;

        // Read off the boot code rather than the header, which does not name it - see Mars_Boot.md §6.1.
        public CicChip CicChip;

        public bool HasEd64Header;
        public N64SaveType SaveType;
        public bool HasRealTimeClock;
        public bool IsRegionFree;

        public static RomImage Load(string path)
        {
            var image = FromImage(File.ReadAllBytes(path));
            image.RomPath = path;
            return image;
        }

        public static RomImage FromImage(byte[] file)
        {
            RomByteOrder order = DetectByteOrder(file);
            byte[] rom = Normalize(file, order);

            if (rom.Length < MinimumLength)
            {
                throw new InvalidDataException(
                    $"Not a Nintendo 64 image: {rom.Length} bytes is shorter than the {MinimumLength}-byte header and boot code.");
            }

            var image = new RomImage { Rom = rom, SourceByteOrder = order };

            image.ClockRate = ReadUInt32(rom, 0x04);
            image.EntryPoint = ReadUInt32(rom, 0x08);
            image.LibultraVersion = ReadUInt32(rom, 0x0C);
            image.Crc1 = ReadUInt32(rom, 0x10);
            image.Crc2 = ReadUInt32(rom, 0x14);

            image.Title = ReadTitle(rom);
            image.CategoryCode = (char)rom[0x3B];
            image.UniqueCode = $"{(char)rom[0x3C]}{(char)rom[0x3D]}";
            image.DestinationCode = (char)rom[0x3E];
            image.Version = rom[0x3F];

            image.IsPal = IsPalDestination(image.DestinationCode);
            image.CicChip = Cic.Identify(rom);
            image.ReadEd64Fields();
            return image;
        }

        // The first word in each of the three orders; anything else is not an N64 image - see Mars_Rom.md §1.1.
        public static RomByteOrder DetectByteOrder(byte[] file)
        {
            if (file.Length < 4)
            {
                throw new InvalidDataException($"Not a Nintendo 64 image: {file.Length} bytes is too short to hold a magic word.");
            }

            uint word = ReadUInt32(file, 0);
            return word switch
            {
                Magic => RomByteOrder.BigEndian,
                0x37804012 => RomByteOrder.ByteSwapped,
                0x40123780 => RomByteOrder.LittleEndian,
                _ => throw new InvalidDataException(
                    $"Not a Nintendo 64 image: first word is 0x{word:X8}, which is 0x{Magic:X8} in none of the three container orders."),
            };
        }

        // Returns the file itself when it is already big-endian, so the common case copies nothing.
        public static byte[] Normalize(byte[] file, RomByteOrder order)
        {
            switch (order)
            {
                case RomByteOrder.BigEndian:
                    return file;

                case RomByteOrder.ByteSwapped:
                    RequireMultipleOf(file, 2, order);
                    var halfwords = (byte[])file.Clone();
                    for (int i = 0; i < halfwords.Length; i += 2)
                    {
                        (halfwords[i], halfwords[i + 1]) = (halfwords[i + 1], halfwords[i]);
                    }
                    return halfwords;

                case RomByteOrder.LittleEndian:
                    RequireMultipleOf(file, 4, order);
                    var words = (byte[])file.Clone();
                    for (int i = 0; i < words.Length; i += 4)
                    {
                        (words[i], words[i + 3]) = (words[i + 3], words[i]);
                        (words[i + 1], words[i + 2]) = (words[i + 2], words[i + 1]);
                    }
                    return words;

                default:
                    throw new ArgumentOutOfRangeException(nameof(order), order, "Unknown container order.");
            }
        }

        // Refused rather than normalised as far as it goes, because a half-swapped tail is silent corruption.
        private static void RequireMultipleOf(byte[] file, int size, RomByteOrder order)
        {
            if (file.Length % size == 0) return;

            throw new InvalidDataException(
                $"Truncated {order} image: {file.Length} bytes is not a multiple of {size}, so the last word cannot be unswapped.");
        }

        // Bits the standard header does not have, claimed by the flashcart convention - see Mars_Rom.md §3.
        private void ReadEd64Fields()
        {
            HasEd64Header = UniqueCode == "ED";
            if (!HasEd64Header)
            {
                SaveType = N64SaveType.Unknown;
                return;
            }

            SaveType = (Version >> 4) switch
            {
                0 => N64SaveType.None,
                1 => N64SaveType.Eeprom4k,
                2 => N64SaveType.Eeprom16k,
                3 => N64SaveType.Sram256k,
                4 => N64SaveType.SramBanked768k,
                5 => N64SaveType.FlashRam,
                6 => N64SaveType.Sram1M,
                _ => N64SaveType.Unknown,
            };

            HasRealTimeClock = (Version & 0x01) != 0;
            IsRegionFree = (Version & 0x02) != 0;
        }

        // Space-padded rather than NUL-terminated, and not guaranteed to be ASCII.
        private static string ReadTitle(byte[] rom)
        {
            var text = new StringBuilder(TitleLength);
            for (int i = 0; i < TitleLength; i++)
            {
                byte b = rom[TitleOffset + i];
                if (b == 0x00) break;
                text.Append(b >= 0x20 && b < 0x7F ? (char)b : ' ');
            }

            return text.ToString().TrimEnd();
        }

        private static bool IsPalDestination(char destination) =>
            destination is 'D' or 'F' or 'I' or 'P' or 'S' or 'U' or 'X' or 'Y';

        private static uint ReadUInt32(byte[] bytes, int offset) =>
            (uint)((bytes[offset] << 24) | (bytes[offset + 1] << 16) | (bytes[offset + 2] << 8) | bytes[offset + 3]);
    }
}
