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
        private int _colorImageSize;
        private int _colorImageFormat;

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
            _colorImageFormat = (int)(word >> 53) & 7;
            _colorImageSize = (int)(word >> 51) & 3;
            _colorImageBytes = _colorImageSize switch { 1 => 1, 2 => 2, 3 => 4, _ => 0 };
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

        private void Fill(ulong word)
        {
            ClearAttributes();
            Draw(WalkRectangle(word), majorOnLeft: true, tile: 0, maxLevel: 0);
        }

        // A rectangle is walked as a primitive with its major edge on the left and no slope - see Mars_RdpTriangles.md §3.
        private (int First, int Last) WalkRectangle(ulong word)
        {
            int right = Quarters(word >> 44);
            int bottom = Quarters(word >> 32) | (CycleType >= 2 ? 3 : 0);
            int left = Quarters(word >> 12);
            int top = Quarters(word);

            int rightX = ((right >> 2) << 16) | ((right & 3) << 14);
            int leftX = ((left >> 2) << 16) | ((left & 3) << 14);

            return Walk(majorOnLeft: true, top, bottom, bottom, leftX, rightX, rightX, 0, 0, 0, majorSlopeNegative: false);
        }

        // The first four words of every triangle command, whatever else it carries - see Mars_RdpTriangles.md §1.
        private void Triangle(uint id)
        {
            ulong edges = _command[0];
            bool majorOnLeft = ((edges >> 55) & 1) != 0;
            DecodeAttributes(id);

            Draw(Walk(
                majorOnLeft,
                yh: SignExtend(edges, 14),
                ym: SignExtend(edges >> 16, 14),
                yl: SignExtend(edges >> 32, 14),
                xh: SignExtend(_command[2] >> 32, 28),
                xm: SignExtend(_command[3] >> 32, 28),
                xl: SignExtend(_command[1] >> 32, 28),
                dxhdy: SignExtend(_command[2], 30),
                dxmdy: SignExtend(_command[3], 30),
                dxldy: SignExtend(_command[1], 30),
                majorSlopeNegative: (int)_command[2] < 0), majorOnLeft, tile: (int)(edges >> 48) & 7, maxLevel: (int)(edges >> 51) & 7);
        }

        // Shade in the eight words after the edges, texture in the next eight, depth in the last two, each value an integer half and a fraction half - see Mars_RdpDepth.md §1.1.
        private void DecodeAttributes(uint id)
        {
            ClearAttributes();
            int at = 4;

            if ((id & 4) != 0)
            {
                for (int c = 0; c < 4; c++)
                {
                    int shift = 48 - 16 * c;
                    _attributeValue[c] = Joined(_command[at], _command[at + 2], shift);
                    _attributeDx[c] = Joined(_command[at + 1], _command[at + 3], shift);
                    _attributeDe[c] = Joined(_command[at + 4], _command[at + 6], shift);
                    _attributeDy[c] = Joined(_command[at + 5], _command[at + 7], shift);
                }

                at += 8;
            }

            if ((id & 2) != 0)
            {
                for (int c = 0; c < 3; c++)
                {
                    int shift = 48 - 16 * c;
                    _attributeValue[AttributeS + c] = Joined(_command[at], _command[at + 2], shift);
                    _attributeDx[AttributeS + c] = Joined(_command[at + 1], _command[at + 3], shift);
                    _attributeDe[AttributeS + c] = Joined(_command[at + 4], _command[at + 6], shift);
                    _attributeDy[AttributeS + c] = Joined(_command[at + 5], _command[at + 7], shift);
                }

                at += 8;
            }

            if ((id & 1) != 0)
            {
                _attributeValue[AttributeZ] = (int)(_command[at] >> 32);
                _attributeDx[AttributeZ] = (int)_command[at];
                _attributeDe[AttributeZ] = (int)(_command[at + 1] >> 32);
                _attributeDy[AttributeZ] = (int)_command[at + 1];
            }
        }

        private void ClearAttributes()
        {
            Array.Clear(_attributeValue);
            Array.Clear(_attributeDx);
            Array.Clear(_attributeDe);
            Array.Clear(_attributeDy);
        }

        private static int Joined(ulong whole, ulong fraction, int shift) => (int)((((whole >> shift) & 0xFFFF) << 16) | ((fraction >> shift) & 0xFFFF));

        // The fill cycle and the one-cycle mode draw every primitive, textured or not; the two-cycle and copy modes draw nothing yet - see Mars_RdpTextures.md §6.
        private void Draw((int First, int Last) rows, bool majorOnLeft, int tile, int maxLevel)
        {
            if (CycleType == FillCycle) FillSpans(rows);
            else if (CycleType == OneCycle) DrawOneCycle(rows, majorOnLeft, tile, maxLevel);
        }

        // A four-bit image has nothing to fill - see Mars_Rdp.md §5.3.
        private void FillSpans((int First, int Last) rows)
        {
            if (_colorImageBytes == 0) return;

            uint image = _colorImage & ~(uint)(_colorImageBytes - 1);

            for (int y = rows.First; y <= rows.Last; y++)
            {
                if (!_spanDrawn[y]) continue;

                for (int x = _spanLeft[y]; x <= _spanRight[y]; x++)
                {
                    FillPixel(image + (uint)((y * _colorImageWidth + x) * _colorImageBytes));
                }
            }
        }

        // Each byte takes the lane of the fill colour its address holds in a big-endian word, and each word's low byte its hidden bits - see §5.1.
        private void FillPixel(uint address)
        {
            byte[] rdram = _bus.Rdram;

            for (uint i = 0; i < _colorImageBytes; i++)
            {
                uint at = address + i;
                if (at >= rdram.Length) continue;

                byte value = (byte)(_fillColor >> (int)(24 - 8 * (at & 3)));
                rdram[at] = value;
                if ((at & 1) != 0) _bus.RdramHidden[at >> 1] = (byte)((value & 1) * 3);
            }
        }

        private static int Quarters(ulong field) => (int)(field & 0xFFF);
    }
}
