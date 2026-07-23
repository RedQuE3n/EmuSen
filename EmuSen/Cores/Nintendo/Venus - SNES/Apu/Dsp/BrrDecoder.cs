using System;

namespace EmuSen.Cores.Nintendo.Venus.Apu
{
    // Decodes SNES BRR (Bit Rate Reduction) compressed audio - see Venus_APU.md §5.
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

        // Decodes one 9-byte BRR block into 16 signed 16-bit PCM samples.
        // block must contain at least 9 bytes starting at offset: 1 header byte
        // followed by 8 bytes of packed 4-bit nibbles (high nibble first per byte).
        public short[] DecodeBlock(byte[] block, int offset = 0)
        {
            byte header = block[offset];
            int shift = (header >> 4) & 0x0F;
            int filter = (header >> 2) & 0x03;
            // Loop/end flags (bits 1 and 0) are playback-control metadata, not
            // needed to decode the samples themselves - see IsEndBlock/IsLoopBlock,
            // which the future playback layer reads from the same header byte to
            // decide whether to stop or loop once this block finishes.

            short[] output = new short[16];

            for (int i = 0; i < 16; i++)
            {
                byte dataByte = block[offset + 1 + (i / 2)];
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

                // Wraps, doesn't saturate - see Venus_APU.md §5.
                short sample = (short)predicted;

                output[i] = sample;
                _prev2 = _prev1;
                _prev1 = sample;
            }

            return output;
        }

        // Header flag helpers - bits 1 and 0 of the same header byte DecodeBlock reads.
        public static bool IsEndBlock(byte header) => (header & 0x01) != 0;
        public static bool IsLoopBlock(byte header) => (header & 0x02) != 0;
    }
}
