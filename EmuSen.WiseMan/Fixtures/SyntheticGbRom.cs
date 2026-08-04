using System;
using System.IO;
using EmuSen.Cores.Nintendo.Mercury.Memory;

namespace EmuSen.WiseMan.Fixtures
{
    // Builds a synthetic Game Boy image - never real game data. The Mercury counterpart of SyntheticNesRom.
    public static class SyntheticGbRom
    {
        public const int RomBankSize = Cartridge.RomBankSize;

        // The entry point the boot ROM jumps to, and where <patches> offset 0 lands.
        public const int EntryPoint = 0x0150;

        public static byte[] Build(
            int romBanks = 2,
            byte cartridgeType = 0x00,
            byte ramSizeCode = 0x00,
            byte cgbFlag = 0x00,
            string title = "WISEMAN",
            params (int Offset, byte[] Bytes)[] patches)
        {
            var image = new byte[Math.Max(2, romBanks) * RomBankSize];

            // NOP fill: zero already decodes as NOP here, but being explicit keeps it obvious.
            Array.Fill(image, (byte)0x00);

            // The real entry stub: reset lands at $0100, and every cart jumps straight past the header.
            image[0x0100] = 0x00;
            image[0x0101] = 0xC3;
            image[0x0102] = (byte)(EntryPoint & 0xFF);
            image[0x0103] = (byte)(EntryPoint >> 8);

            for (int i = 0; i < title.Length && i < 11; i++) image[0x0134 + i] = (byte)title[i];

            image[0x0143] = cgbFlag;
            image[0x0147] = cartridgeType;
            image[0x0148] = RomSizeCode(Math.Max(2, romBanks));
            image[0x0149] = ramSizeCode;

            foreach (var (offset, bytes) in patches) bytes.CopyTo(image, EntryPoint + offset);

            image[Cartridge.HeaderChecksumAddress] = Cartridge.ComputeHeaderChecksum(image);
            return image;
        }

        // 0 is 2 banks and each step doubles, so the code is just log2 of the bank count minus one.
        private static byte RomSizeCode(int romBanks)
        {
            byte code = 0;
            while (2 << code < romBanks) code++;
            return code;
        }

        public static string WriteTemp(byte[] image)
        {
            string path = Path.Combine(Path.GetTempPath(), $"wiseman_{Guid.NewGuid():N}.gb");
            File.WriteAllBytes(path, image);
            return path;
        }
    }
}
