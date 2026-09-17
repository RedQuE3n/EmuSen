using System;

namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // The colour image, the scissor and the fill cycle - see Mars_Rdp.md §5.
    public sealed partial class Rdp
    {
        private const int FillCycle = 3;

        private uint _fillColor;

        private uint _colorImage;
        private int _colorImageWidth;
        private int _colorImageBytes;

        // Quarter pixels, as the command carries them.
        private int _scissorLeft;
        private int _scissorTop;
        private int _scissorRight;
        private int _scissorBottom;
        private bool _scissorField;
        private bool _scissorKeepOdd;

        // The width is stored one short, and a four-bit image has no bytes to fill - see §5.
        private void ColorImage(ulong word)
        {
            _colorImageBytes = ((word >> 51) & 3) switch { 1 => 1, 2 => 2, 3 => 4, _ => 0 };
            _colorImageWidth = (int)((word >> 32) & 0x3FF) + 1;
            _colorImage = (uint)word & 0x00FF_FFFF;
        }

        private void Scissor(ulong word)
        {
            _scissorLeft = Quarters(word >> 44);
            _scissorTop = Quarters(word >> 32);
            _scissorRight = Quarters(word >> 12);
            _scissorBottom = Quarters(word);
            _scissorField = ((word >> 25) & 1) != 0;
            _scissorKeepOdd = ((word >> 24) & 1) != 0;
        }

        // Each row's span is the two edges clipped in eighth pixels; rows are chosen by quarter-pixel sub-scanlines - see §5.2.
        private void Fill(ulong word)
        {
            if (CycleType != FillCycle || _colorImageBytes == 0) return;

            int right = Quarters(word >> 44);
            int bottom = Quarters(word >> 32) | 3;
            int left = Quarters(word >> 12);
            int top = Quarters(word);

            if (right < left) return;

            (int first, bool firstUnder, bool firstOver) = Clip(left * 2);
            (int last, bool lastUnder, bool lastOver) = Clip(right * 2);
            if ((firstUnder && lastUnder) || (firstOver && lastOver)) return;

            int upper = Math.Max(top, _scissorTop);
            int lower = Math.Min(bottom, _scissorBottom);
            if (upper >= lower) return;

            uint image = _colorImage & ~(uint)(_colorImageBytes - 1);

            for (int y = upper >> 2; y << 2 < lower; y++)
            {
                if (_scissorField && ((y & 1) == 1) != _scissorKeepOdd) continue;

                for (int x = first >> 3; x <= last >> 3; x++)
                {
                    FillPixel(image + (uint)((y * _colorImageWidth + x) * _colorImageBytes));
                }
            }
        }

        // Moved onto the scissor's left edge if under it, then onto its right edge if at or past it, in that order - see §5.2.
        private (int At, bool Under, bool Over) Clip(int eighths)
        {
            bool under = eighths < _scissorLeft * 2;
            int at = under ? _scissorLeft * 2 : eighths;
            bool over = at >= _scissorRight * 2;

            return (over ? _scissorRight * 2 : at, under, over);
        }

        // Each byte takes the lane of the fill colour its address holds in a big-endian word - see §5.1.
        private void FillPixel(uint address)
        {
            byte[] rdram = _bus.Rdram;

            for (uint i = 0; i < _colorImageBytes; i++)
            {
                uint at = address + i;
                if (at < rdram.Length) rdram[at] = (byte)(_fillColor >> (int)(24 - 8 * (at & 3)));
            }
        }

        private static int Quarters(ulong field) => (int)(field & 0xFFF);
    }
}
