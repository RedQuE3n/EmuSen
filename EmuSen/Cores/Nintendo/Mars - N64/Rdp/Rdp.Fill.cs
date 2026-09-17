using System;

namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // The colour image, the scissor and the fill cycle, in whole pixels - see Mars_Rdp.md §5.
    public sealed partial class Rdp
    {
        private const int FillCycle = 3;

        private uint _fillColor;

        private uint _colorImage;
        private int _colorImageWidth;
        private int _colorImageBytes;

        private int _scissorLeft;
        private int _scissorTop;
        private int _scissorRight;
        private int _scissorBottom;

        // The width is stored one short, and a four-bit image has no bytes to fill - see §5.
        private void ColorImage(ulong word)
        {
            _colorImageBytes = ((word >> 51) & 3) switch { 1 => 1, 2 => 2, 3 => 4, _ => 0 };
            _colorImageWidth = (int)((word >> 32) & 0x3FF) + 1;
            _colorImage = (uint)word & 0x00FF_FFFF;
        }

        private void Scissor(ulong word)
        {
            _scissorLeft = WholePixel(word >> 44);
            _scissorTop = WholePixel(word >> 32);
            _scissorRight = WholePixel(word >> 12);
            _scissorBottom = WholePixel(word);
        }

        // The rectangle's far edges are inside it and the scissor's are not; the fractional rule is unbuilt - see §5.2.
        private void Fill(ulong word)
        {
            if (CycleType != FillCycle || _colorImageBytes == 0) return;

            int right = Math.Min(WholePixel(word >> 44) + 1, _scissorRight);
            int bottom = Math.Min(WholePixel(word >> 32) + 1, _scissorBottom);
            int left = Math.Max(WholePixel(word >> 12), _scissorLeft);
            int top = Math.Max(WholePixel(word), _scissorTop);

            uint image = _colorImage & ~(uint)(_colorImageBytes - 1);

            for (int y = top; y < bottom; y++)
            {
                for (int x = left; x < right; x++)
                {
                    FillPixel(image + (uint)((y * _colorImageWidth + x) * _colorImageBytes));
                }
            }
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

        private static int WholePixel(ulong field) => (int)(field & 0xFFF) >> 2;
    }
}
