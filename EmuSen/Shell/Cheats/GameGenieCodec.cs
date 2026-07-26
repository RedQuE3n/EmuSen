using System;
using System.Linq;

namespace EmuSen.Shell.Cheats
{
    // Decodes real SNES Game Genie codes (Galoob) - a completely different
    // mechanism from ActionReplayCodec, and not just a different letter
    // alphabet: SNES Game Genie is a genuine two-stage transposition
    // cipher. Stage one substitutes each letter for a scrambled 4-bit
    // nibble (Alphabet below); stage two reassembles those 32 scrambled
    // bits into 8 output nibbles by pulling from specific, non-contiguous
    // bit positions (see Decode) - nothing here is a straight hex
    // substitution the way Pro Action Replay's codes are.
    //
    // Verified by hand-tracing a full encode-then-decode round-trip
    // through an independently-published, MIT-licensed reference
    // implementation of this exact algorithm before writing this port -
    // getting a SNES Game Genie decode subtly wrong would silently
    // corrupt whatever ROM byte a mistranslated address happened to land
    // on, which is worse than not having this feature at all, so this
    // wasn't implemented from memory alone.
    //
    // Format: 8 letters (conventionally displayed "XXXX-XXXX"; the dash,
    // like any other non-alphabet character, is ignored here) decoding to
    // a 3-byte ROM/cartridge address plus a 1-byte value. Unlike NES/
    // Genesis Game Genie, real SNES codes carry no compare byte at all -
    // Galoob's SNES Game Genie patches ROM instructions directly using 3
    // full address bytes instead of 2, so it never needed one. Every
    // decoded code becomes an unconditional RomPatch (Compare = null) -
    // see CheatCommand's `gg` sub-command.
    public static class GameGenieCodec
    {
        // The 16-letter substitution alphabet - a letter's INDEX in this
        // string is the nibble value it decodes to (Alphabet[0]='D' means
        // the letter 'D' is nibble 0x0, Alphabet[15]='E' means 'E' is
        // nibble 0xF, etc). Different from every other Game Genie
        // variant's own alphabet (NES, Genesis, and Game Boy each use a
        // different arrangement) - this one is SNES-specific.
        private const string Alphabet = "DF4709156BC8A23E";

        // True if <code> looks like a SNES Game Genie code at all - 8
        // letters from Alphabet once non-alphabet separators are
        // stripped. Meant for a future auto-detecting `cheat add` that
        // tries each known codec in turn (see the phased cheat-engine
        // plan) - Game Genie codes are letters-only, so they're already
        // trivially distinguishable from ActionReplayCodec's hex-digit
        // codes.
        public static bool CanDecode(string code) => TryNormalize(code, out _);

        public static (int Address, byte Value) Decode(string code)
        {
            if (!TryNormalize(code, out string letters))
            {
                throw new FormatException($"'{code}' isn't a valid SNES Game Genie code - expected 8 letters from the alphabet {Alphabet}.");
            }

            // Stage one: each of the 8 letters is 4 scrambled bits -
            // assemble all 32 into one bit array, most-significant-bit
            // first per letter, matching the reference algorithm's own
            // bit-string assembly before its transposition step.
            bool[] bits = new bool[32];
            for (int i = 0; i < 8; i++)
            {
                int nibble = Alphabet.IndexOf(letters[i]);
                for (int b = 0; b < 4; b++) bits[i * 4 + b] = (nibble & (0x8 >> b)) != 0;
            }

            // Stage two: un-scramble. Each output nibble pulls its 4 bits
            // from specific positions in the 32-bit stream above - n3 is
            // the odd one out, split 2+2 across two non-adjacent spans.
            // This exact mapping (and no other) is what makes SNES Game
            // Genie decoding a real cipher rather than a straight hex
            // substitution - verified against the reference algorithm's
            // own bit offsets, not derived independently.
            int n0 = ReadBits(bits, 18, 4);
            int n1 = ReadBits(bits, 26, 4);
            int n2 = ReadBits(bits, 8, 4);
            int n3 = (ReadBits(bits, 30, 2) << 2) | ReadBits(bits, 16, 2);
            int n4 = ReadBits(bits, 12, 4);
            int n5 = ReadBits(bits, 22, 4);
            int n6 = ReadBits(bits, 0, 4);
            int n7 = ReadBits(bits, 4, 4);

            // n0..n5 are the address's 6 hex digits (most significant
            // first), n6..n7 are the value's 2 hex digits.
            int address = (n0 << 20) | (n1 << 16) | (n2 << 12) | (n3 << 8) | (n4 << 4) | n5;
            byte value = (byte)((n6 << 4) | n7);
            return (address, value);
        }

        // Reads <count> consecutive bits starting at <start> as a
        // most-significant-bit-first integer - the array equivalent of
        // the reference algorithm's bits.substr(start, count) followed by
        // parseInt(_, 2).
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
