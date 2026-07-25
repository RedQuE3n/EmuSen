using System;

namespace EmuSen.Cores.Nintendo.Venus.Apu
{
    // Decodes SNES BRR (Bit Rate Reduction) compressed audio - see Venus_APU.md §5.
    //
    // Decodes 4 samples (2 packed bytes) at a time via DecodeQuad, not a
    // whole 16-sample block in one call - ported this way (ground-truthed
    // against Mesen2's DspVoice::DecodeBrrSample) specifically so DspVoice
    // can interleave decoding with Gaussian interpolation exactly the way
    // real hardware's S-DSP does: a block's 4 nibble-pairs get decoded one
    // pair at a time, as playback actually reaches each one, into a
    // circular buffer that still holds a few of the PREVIOUS quad's
    // samples too (needed as interpolation lookback across a quad
    // boundary, not just a block boundary).
    public class BrrDecoder
    {
        // The two most recently decoded samples - required by filters 1-3 to predict
        // the next one. Reset to 0 at the start of a new sound (a filter-0 block,
        // which every sample is required to start with, ignores these anyway).
        private int _prev1;
        private int _prev2;

        public void Reset()
        {
            _prev1 = 0;
            _prev2 = 0;
        }

        // Decodes exactly 4 signed samples from one BRR header byte plus 2
        // consecutive packed data bytes (b0's nibbles, then b1's nibbles -
        // matching real hardware reading "the current byte" and "the next
        // byte" to produce one quad). Returns the samples already in the
        // hardware's own internal representation: clamped to 15 bits then
        // DOUBLED (and wrapped via a 16-bit truncation, not re-clamped) -
        // this is a documented real quirk, not a mistake: the filter
        // prediction for later samples uses these values HALVED back down,
        // and the doubling+truncation (rather than a second clamp) is
        // exactly how the chip is documented to behave on overflow.
        public short[] DecodeQuad(byte header, byte b0, byte b1)
        {
            int shift = (header >> 4) & 0x0F;
            int filter = (header >> 2) & 0x03;

            Span<byte> bytes = stackalloc byte[2] { b0, b1 };
            short[] output = new short[4];

            for (int i = 0; i < 4; i++)
            {
                byte dataByte = bytes[i / 2];
                int nibble = (i % 2 == 0) ? (dataByte >> 4) & 0x0F : dataByte & 0x0F;

                // Sign-extend the 4-bit nibble to a signed value in -8..7.
                if (nibble >= 8) nibble -= 16;

                // Shift 13-15 is a documented hardware quirk - see Venus_APU.md §5.
                int raw;
                if (shift <= 12)
                {
                    raw = (nibble << shift) >> 1;
                }
                else
                {
                    raw = nibble < 0 ? -2048 : 0;
                }

                int predicted;
                switch (filter)
                {
                    default:
                    case 0:
                        predicted = raw;
                        break;
                    case 1:
                        // p1 * 15/16
                        predicted = raw + _prev1 + ((-_prev1) >> 4);
                        break;
                    case 2:
                        // p1 * 61/32 - p2 * 15/16
                        predicted = raw + (_prev1 * 2) + ((-_prev1 * 3) >> 5) - _prev2 + (_prev2 >> 4);
                        break;
                    case 3:
                        // p1 * 115/64 - p2 * 13/16
                        predicted = raw + (_prev1 * 2) + ((-_prev1 * 13) >> 6) - _prev2 + ((_prev2 * 3) >> 4);
                        break;
                }

                // Clamp to 16 bits, then double (may itself overflow 16
                // bits - that overflow WRAPS via the truncating cast to
                // short, deliberately not re-clamped) - see this method's
                // own comment above for why.
                int clamped = Math.Clamp(predicted, short.MinValue, short.MaxValue);
                short sample = (short)(clamped * 2);

                output[i] = sample;
                _prev2 = _prev1;
                _prev1 = sample >> 1;
            }

            return output;
        }

        // Header flag helpers - bits 1 and 0 of the same header byte DecodeQuad reads.
        public static bool IsEndBlock(byte header) => (header & 0x01) != 0;
        public static bool IsLoopBlock(byte header) => (header & 0x02) != 0;
    }
}
