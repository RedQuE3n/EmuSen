using System;
using System.IO;
using EmuSen.Cores.Nintendo.Mars.Rom;

namespace EmuSen.WiseMan.Fixtures
{
    // Builds a synthetic Nintendo 64 image - never real game data. The Mars counterpart of SyntheticGbRom.
    public static class SyntheticN64Rom
    {
        // Where IPL3 would hand control over, and where <patches> offset 0 lands - see Mars_Rom.md §2.
        public const uint DefaultEntryPoint = 0x80000400;

        public static byte[] Build(
            int length = RomImage.MinimumLength,
            uint entryPoint = DefaultEntryPoint,
            string title = "WISEMAN",
            string uniqueCode = "WM",
            char categoryCode = 'N',
            char destinationCode = 'E',
            byte version = 0x00,
            params (int Offset, byte[] Bytes)[] patches)
        {
            var image = new byte[Math.Max(RomImage.MinimumLength, length)];

            WriteUInt32(image, 0x00, RomImage.Magic);
            WriteUInt32(image, 0x04, 0x0000000F);
            WriteUInt32(image, 0x08, entryPoint);
            WriteUInt32(image, 0x0C, 0x0000144C);

            // Not the real polynomial over the boot code; nothing in Phase 0 verifies it - see Mars_Rom.md §2.3.
            WriteUInt32(image, 0x10, 0xDEADBEEF);
            WriteUInt32(image, 0x14, 0xFEEDFACE);

            for (int i = 0; i < RomImage.TitleLength; i++) image[RomImage.TitleOffset + i] = (byte)' ';
            for (int i = 0; i < title.Length && i < RomImage.TitleLength; i++)
            {
                image[RomImage.TitleOffset + i] = (byte)title[i];
            }

            image[0x3B] = (byte)categoryCode;
            image[0x3C] = (byte)uniqueCode[0];
            image[0x3D] = (byte)uniqueCode[1];
            image[0x3E] = (byte)destinationCode;
            image[0x3F] = version;

            foreach (var (offset, bytes) in patches) bytes.CopyTo(image, RomImage.HeaderLength + offset);
            return image;
        }

        // An ED64-convention header, which is the only way an image can state its own save type.
        public static byte[] BuildHomebrew(N64SaveType saveType, bool realTimeClock = false, bool regionFree = false)
        {
            byte nibble = saveType switch
            {
                N64SaveType.None => 0,
                N64SaveType.Eeprom4k => 1,
                N64SaveType.Eeprom16k => 2,
                N64SaveType.Sram256k => 3,
                N64SaveType.SramBanked768k => 4,
                N64SaveType.FlashRam => 5,
                N64SaveType.Sram1M => 6,
                _ => 0xF,
            };

            byte version = (byte)((nibble << 4) | (realTimeClock ? 0x01 : 0x00) | (regionFree ? 0x02 : 0x00));
            return Build(uniqueCode: "ED", version: version);
        }

        // The same image as a Doctor V64 dump: every halfword swapped.
        public static byte[] ToByteSwapped(byte[] bigEndian)
        {
            var swapped = (byte[])bigEndian.Clone();
            for (int i = 0; i + 1 < swapped.Length; i += 2)
            {
                (swapped[i], swapped[i + 1]) = (swapped[i + 1], swapped[i]);
            }

            return swapped;
        }

        // ...and as a little-endian dump: every word reversed.
        public static byte[] ToLittleEndian(byte[] bigEndian)
        {
            var words = (byte[])bigEndian.Clone();
            for (int i = 0; i + 3 < words.Length; i += 4)
            {
                (words[i], words[i + 3]) = (words[i + 3], words[i]);
                (words[i + 1], words[i + 2]) = (words[i + 2], words[i + 1]);
            }

            return words;
        }

        public static string WriteTemp(byte[] image, string extension = ".z64")
        {
            string path = Path.Combine(Path.GetTempPath(), $"wiseman_{Guid.NewGuid():N}{extension}");
            File.WriteAllBytes(path, image);
            return path;
        }

        private static void WriteUInt32(byte[] image, int offset, uint value)
        {
            image[offset] = (byte)(value >> 24);
            image[offset + 1] = (byte)(value >> 16);
            image[offset + 2] = (byte)(value >> 8);
            image[offset + 3] = (byte)value;
        }
    }
}
