using System;
using System.Linq;

namespace EmuSen.Cores.Nintendo.Moon.Cheats
{
    // Decodes real NES Game Genie codes (Galoob). A different cipher and a
    // different alphabet from the SNES device - see Moon_Cheats.md §2 for the
    // bit tables and why they were ported rather than derived.
    public static class NesGameGenieCodec
    {
        // A letter's INDEX here is the nibble it decodes to. NES-specific;
        // every Game Genie variant arranges its own sixteen differently.
        private const string Alphabet = "APZLGITYEOXUKSVN";

        // Which bit of the assembled 32-bit code each output bit comes from,
        // most-significant first. Taken from Mesen's ConvertFromNesGameGenie.
        private static readonly int[] AddressBits = { 14, 13, 12, 19, 22, 21, 20, 7, 10, 9, 8, 15, 18, 17, 16 };
        private static readonly int[] ValueBits = { 3, 6, 5, 4, 23, 2, 1, 0 };
        private static readonly int[] CompareBits = { 27, 30, 29, 28, 23, 26, 25, 24 };

        // Bit 5 of the value moves for an 8-letter code, because the compare
        // byte takes the place it occupied.
        private const int EightLetterValueBit4 = 31;

        public static bool CanDecode(string code) => TryNormalize(code, out _);

        // True for the 8-letter form, which is the one that carries a compare.
        public static bool HasCompare(string code) => TryNormalize(code, out string letters) && letters.Length == 8;

        public static (int Address, byte Value) Decode(string code)
        {
            (int address, byte value, byte? _) = DecodeFull(code);
            return (address, value);
        }

        public static byte? DecodeCompare(string code)
        {
            (int _, byte __, byte? compare) = DecodeFull(code);
            return compare;
        }

        public static (int Address, byte Value, byte? Compare) DecodeFull(string code)
        {
            if (!TryNormalize(code, out string letters))
            {
                throw new FormatException(
                    $"'{code}' isn't a valid NES Game Genie code - expected 6 or 8 letters from the alphabet {Alphabet}.");
            }

            // Each letter contributes its nibble at its own 4-bit slot, so
            // letter 0 is the low nibble rather than the high one.
            int raw = 0;
            for (int i = 0; i < letters.Length; i++) raw |= Alphabet.IndexOf(letters[i]) << (i * 4);

            int[] valueBits = ValueBits;
            byte? compare = null;

            if (letters.Length == 8)
            {
                valueBits = (int[])ValueBits.Clone();
                valueBits[4] = EightLetterValueBit4;
                compare = (byte)Gather(raw, CompareBits);
            }

            // The 15 address bits are an offset into the cartridge's half of
            // the CPU map, which is why the code can only ever reach $8000 up.
            int address = Gather(raw, AddressBits) + 0x8000;
            return (address, (byte)Gather(raw, valueBits), compare);
        }

        private static int Gather(int raw, int[] bitIndexes)
        {
            int result = 0;
            foreach (int bit in bitIndexes) result = (result << 1) | ((raw >> bit) & 0x01);
            return result;
        }

        private static bool TryNormalize(string code, out string letters)
        {
            letters = new string((code ?? "")
                .Where(c => Alphabet.IndexOf(char.ToUpperInvariant(c)) >= 0)
                .Select(char.ToUpperInvariant)
                .ToArray());
            return letters.Length == 6 || letters.Length == 8;
        }
    }
}
