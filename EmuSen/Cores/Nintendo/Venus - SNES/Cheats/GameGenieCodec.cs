using System;
using System.Linq;

namespace EmuSen.Cores.Nintendo.Venus.Cheats
{
    // A real two-stage transposition cipher, not a hex substitution - see EmuSen_Cheats.md §2.
    public static class GameGenieCodec
    {
        // A letter's index is the nibble it decodes to; this arrangement is SNES-specific.
        private const string Alphabet = "DF4709156BC8A23E";

        // True if <code> looks like a SNES Game Genie code at all - 8 letters from Alphabet once non-alphabet.
        public static bool CanDecode(string code) => TryNormalize(code, out _);

        public static (int Address, byte Value) Decode(string code)
        {
            if (!TryNormalize(code, out string letters))
            {
                throw new FormatException($"'{code}' isn't a valid SNES Game Genie code - expected 8 letters from the alphabet {Alphabet}.");
            }

            // Stage one: each of the 8 letters is 4 scrambled bits - assemble all 32 into one bit array.
            bool[] bits = new bool[32];
            for (int i = 0; i < 8; i++)
            {
                int nibble = Alphabet.IndexOf(letters[i]);
                for (int b = 0; b < 4; b++) bits[i * 4 + b] = (nibble & (0x8 >> b)) != 0;
            }

            // Stage two: un-scramble.
            int n0 = ReadBits(bits, 18, 4);
            int n1 = ReadBits(bits, 26, 4);
            int n2 = ReadBits(bits, 8, 4);
            int n3 = (ReadBits(bits, 30, 2) << 2) | ReadBits(bits, 16, 2);
            int n4 = ReadBits(bits, 12, 4);
            int n5 = ReadBits(bits, 22, 4);
            int n6 = ReadBits(bits, 0, 4);
            int n7 = ReadBits(bits, 4, 4);

            // n0..n5 are the address's 6 hex digits (most significant first), n6..n7 are the value's 2 hex digits.
            int address = (n0 << 20) | (n1 << 16) | (n2 << 12) | (n3 << 8) | (n4 << 4) | n5;
            byte value = (byte)((n6 << 4) | n7);
            return (address, value);
        }

        // Reads <count> bits from <start>, most significant first.
        private static int ReadBits(bool[] bits, int start, int count)
        {
            int v = 0;
            for (int i = 0; i < count; i++) v = (v << 1) | (bits[start + i] ? 1 : 0);
            return v;
        }

        private static bool TryNormalize(string code, out string letters)
        {
            letters = new string((code ?? "")
                .Where(c => Alphabet.IndexOf(char.ToUpperInvariant(c)) >= 0)
                .Select(char.ToUpperInvariant)
                .ToArray());
            return letters.Length == 8;
        }
    }
}
