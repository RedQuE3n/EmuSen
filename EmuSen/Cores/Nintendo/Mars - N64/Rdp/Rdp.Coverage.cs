using System;
using System.Numerics;

namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // Coverage: eight of each pixel's sixteen sub-samples, taken from the walker's sub-scanline edges - see Mars_RdpCoverage.md §2.
    public sealed partial class Rdp
    {
        private byte[] _coverage = new byte[SpanRows];

        // Each sub-scanline owns two samples of a pixel's eight, the pairs offset by one column on alternate lines - see §2.1.
        private void RowCoverage(int row, int left, int right)
        {
            Array.Fill(_coverage, (byte)0xFF, left, right - left + 1);
            Array.Fill(_coverageStamp, _rowStamp, left, right - left + 1);

            for (int sub = 0; sub < 4; sub++)
            {
                int samples = 0xA >> (sub & 1);
                int shift = (sub - 2) & 4;
                byte cleared = (byte)~(samples << shift);
                int k = row * 4 + sub;

                if (_edgeInvalid[k])
                {
                    for (int x = left; x <= right; x++) _coverage[x] &= cleared;
                    continue;
                }

                int leftEdge = _edgeLeft[k], rightEdge = _edgeRight[k];
                int leftPixel = leftEdge >> 3, rightPixel = rightEdge >> 3;

                for (int x = left; x <= leftPixel; x++) _coverage[x] &= cleared;
                for (int x = rightPixel; x <= right; x++) _coverage[x] &= cleared;

                if (rightPixel > leftPixel)
                {
                    _coverage[leftPixel] |= (byte)(LeftSamples(leftEdge, samples) << shift);
                    _coverage[rightPixel] |= (byte)(RightSamples(rightEdge, samples) << shift);
                }
                else if (rightPixel == leftPixel)
                {
                    _coverage[leftPixel] |= (byte)((LeftSamples(leftEdge, samples) & RightSamples(rightEdge, samples)) << shift);
                }
            }
        }

        // An edge's fraction of a pixel, in eighths, decides how many of the pair's samples lie on the covered side.
        private static int LeftSamples(int eighths, int samples) => (0x0F >> (((eighths & 7) + 1) >> 1)) & samples;

        private static int RightSamples(int eighths, int samples) => (0xF0 >> (((eighths & 7) + 1) >> 1)) & samples;

        private static int CoverageCount(byte mask) => BitOperations.PopCount(mask);

        private static bool CoverageBit(byte mask) => (mask & 0x80) != 0;

        // What is written back beside the colour: clamped, wrapped, forced full, or left as memory had it - see §3.
        private int FinalCoverage(bool blend, int coverage, int memory)
        {
            switch (CoverageDestination)
            {
                case 0:
                    int sum = blend ? coverage + memory : coverage - 1;
                    return (sum & 8) != 0 ? 7 : sum & 7;
                case 1: return (coverage + memory) & 7;
                case 2: return 7;
                default: return memory;
            }
        }
    }
}
