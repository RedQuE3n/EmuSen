using System;

namespace EmuSen.Cores.Nintendo.Mars.Rom
{
    // The cartridge's security chip, which an image names only through the boot code it carries - see Mars_Boot.md §6.
    public enum CicChip
    {
        Unknown,
        Nus6101,
        Nus6102,
        Nus6103,
        Nus6105,
        Nus6106,
    }

    public static class Cic
    {
        // Every word of IPL3 summed as a 64-bit total, which is how both software references tell the chips apart - see Mars_Boot.md §6.1.
        public static CicChip Identify(byte[] rom)
        {
            if (rom.Length < RomImage.MinimumLength) return CicChip.Unknown;

            ulong sum = 0;
            for (int i = RomImage.HeaderLength; i < RomImage.MinimumLength; i += 4)
            {
                sum += (uint)((rom[i] << 24) | (rom[i + 1] << 16) | (rom[i + 2] << 8) | rom[i + 3]);
            }

            return sum switch
            {
                0x0000_00D0_027F_DF31 or 0x0000_00CF_FB63_1223 => CicChip.Nus6101,
                0x0000_00D0_57C8_5244 or 0x0000_007C_5624_2373 => CicChip.Nus6102,
                0x0000_00D6_497E_414B => CicChip.Nus6103,
                0x0000_011A_49F6_0E96 => CicChip.Nus6105,
                0x0000_00D6_D5BE_5580 => CicChip.Nus6106,
                _ => CicChip.Unknown,
            };
        }

        // The seed IPL3's checksum starts from; an unknown chip gets the 6102's, as both software references do - see Mars_Boot.md §6.2.
        public static byte Seed(CicChip chip) => chip switch
        {
            CicChip.Nus6103 => 0x78,
            CicChip.Nus6105 => 0x91,
            CicChip.Nus6106 => 0x85,
            _ => 0x3F,
        };

        private static readonly byte[] Lut =
        {
            0x4, 0x7, 0xA, 0x7, 0xE, 0x5, 0xE, 0x1, 0xC, 0xF, 0x8, 0xF, 0x6, 0x3, 0x6, 0x9,
            0x4, 0x1, 0xA, 0x7, 0xE, 0x5, 0xE, 0x1, 0xC, 0x9, 0x8, 0x5, 0x6, 0x3, 0xC, 0x9,
        };

        // The 6105's reply, a nibble at a time from the high one; challenge and response may be the same span - see Mars_Boot.md §7.1.
        public static void Respond(ReadOnlySpan<byte> challenge, Span<byte> response)
        {
            int key = 0xB;
            int mode = 0;

            for (int i = 0; i < challenge.Length; i++)
            {
                byte both = challenge[i];
                int high = Step(both >> 4, ref key, ref mode);
                int low = Step(both & 0xF, ref key, ref mode);
                response[i] = (byte)((high << 4) | low);
            }
        }

        private static int Step(int nibble, ref int key, ref int mode)
        {
            int result = (key + 5 * nibble) & 0xF;
            int sign = result >> 3;
            int magnitude = (sign == 1 ? ~result : result) & 0x7;

            key = Lut[(mode << 4) | result];

            int next = magnitude % 3 == 1 ? sign : 1 - sign;
            if (mode == 1 && result is 0x1 or 0x9) next = 1;
            if (mode == 1 && result is 0xB or 0xE) next = 0;

            mode = next;
            return result;
        }
    }
}
