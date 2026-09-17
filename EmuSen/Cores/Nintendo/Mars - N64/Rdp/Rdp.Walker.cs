using System;

namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // A primitive's two edges walked in quarter-pixel sub-scanlines into one span per row - see Mars_RdpTriangles.md §2.
    public sealed partial class Rdp
    {
        private const int SpanRows = 1024;

        private readonly bool[] _spanDrawn = new bool[SpanRows];
        private readonly int[] _spanLeft = new int[SpanRows];
        private readonly int[] _spanRight = new int[SpanRows];

        // Each sub-scanline's clipped edges in eighth pixels, indexed by sub-scanline, which coverage reads - see Mars_RdpCoverage.md §2.
        private readonly int[] _edgeLeft = new int[SpanRows * 4];
        private readonly int[] _edgeRight = new int[SpanRows * 4];
        private readonly bool[] _edgeInvalid = new bool[SpanRows * 4];

        // The rows it returns are the only ones whose spans it wrote, and so the only ones to draw.
        private (int First, int Last) Walk(bool majorOnLeft, int yh, int ym, int yl, int xh, int xm, int xl, int dxhdy, int dxmdy, int dxldy)
        {
            _combined = default;

            int upper = UpperLimit(yh);
            int lower = LowerLimit(yl);
            int first = upper >> 2, last = lower >> 2;
            if (first > last) return (first, last);

            // A guard only: every row returned is written below, which a mutation confirms - see §2.4.
            Array.Clear(_spanDrawn, first, last - first + 1);

            int far = lower | 3;
            int close = upper & ~3;

            int major = xh & ~1, minor = xm & ~1;
            int majorStep = (dxhdy >> 2) & ~1, minorStep = (dxmdy >> 2) & ~1;

            int left = 0, right = 0;
            bool over = true, under = true, outside = true;

            for (int k = yh & ~3; k <= far; k++)
            {
                if (k == ym)
                {
                    minor = xl & ~1;
                    minorStep = (dxldy >> 2) & ~1;
                }

                if (k >= close)
                {
                    if ((k & 3) == 0)
                    {
                        left = 0xFFF;
                        right = 0;
                        over = under = outside = true;
                    }

                    int leftEdge = majorOnLeft ? major : minor;
                    int rightEdge = majorOnLeft ? minor : major;

                    (int leftAt, bool leftUnder, bool leftOver) = ClipEdge(leftEdge);
                    (int rightAt, bool rightUnder, bool rightOver) = ClipEdge(rightEdge);
                    over &= leftOver && rightOver;
                    under &= leftUnder && rightUnder;

                    bool invalid = k < upper || k >= lower || QuarterPixel(rightEdge) < QuarterPixel(leftEdge);
                    outside &= invalid;

                    _edgeLeft[k] = leftAt & 0x1FFF;
                    _edgeRight[k] = rightAt & 0x1FFF;
                    _edgeInvalid[k] = invalid;

                    if (!invalid)
                    {
                        left = Math.Min(left, leftAt >> 3);
                        right = Math.Max(right, rightAt >> 3);
                    }

                    if ((k & 3) == 3)
                    {
                        int row = k >> 2;
                        _spanDrawn[row] = !outside && !over && !under && FieldKeeps(row);
                        _spanLeft[row] = left;
                        _spanRight[row] = right;
                    }
                }

                major += majorStep;
                minor += minorStep;
            }

            return (first, last);
        }

        // A sixteen-fraction-bit x taken to eighth pixels, whose sticky bit no fill can see, moved onto the scissor's left edge if under it and then its right if at or past it - see §2.3.
        private (int At, bool Under, bool Over) ClipEdge(int x)
        {
            int sticky = ((x >> 1) & 0x1FFF) != 0 ? 1 : 0;
            int clipLeft = _scissorLeft * 2, clipRight = _scissorRight * 2;

            int at = ((x >> 13) & 0x1FFE) | sticky;
            bool under = (x & 0x0800_0000) != 0 || (at < clipLeft && (x & 0x0400_0000) == 0);

            at = under ? clipLeft : ((x >> 13) & 0x3FFE) | sticky;
            bool over = (at & 0x2000) != 0 || (at & 0x1FFF) >= clipRight;

            return (over ? clipRight : at, under, over);
        }

        // The edges compared for crossing at quarter-pixel precision, as signed twelve-bit whole parts.
        private static int QuarterPixel(int x) => (x ^ (1 << 27)) & (0x3FFF << 14);

        // Negative tops start at the scissor and tops past the last row stand; otherwise the lower of the two wins - see §2.2.
        private int UpperLimit(int yh) =>
            (yh & 0x2000) != 0 ? _scissorTop : (yh & 0x1000) != 0 ? yh : Math.Max(yh, _scissorTop);

        private int LowerLimit(int yl) =>
            (yl & 0x2000) != 0 ? yl : (yl & 0x1000) != 0 ? _scissorBottom : Math.Min(yl, _scissorBottom);

        private bool FieldKeeps(int row) => !_scissorField || ((row & 1) == 1) == _scissorKeepOdd;

        private static int SignExtend(ulong field, int bits) => (int)((uint)field << (32 - bits)) >> (32 - bits);
    }
}
