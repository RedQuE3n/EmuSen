using System;
using System.Numerics;

namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // The depth buffer: its sixteen-bit encoding, the four compare modes, and the store - see Mars_RdpDepth.md §3.
    public sealed partial class Rdp
    {
        private uint _depthImage;

        // A coverage mask's first covered sample row and that row's first covered column, which shade and depth correction step by - see §2.
        private static readonly (byte X, byte Y)[] CoverageOffsets = BuildCoverageOffsets();

        // Eighteen bits of depth kept as a three-bit exponent of leading ones and an eleven-bit mantissa - see §3.1.
        private static int CompressDepth(int z)
        {
            int leading = BitOperations.LeadingZeroCount(~((uint)(z & 0x3FFFF) << 14)) switch { > 7 => 7, var n => n };
            int mantissa = leading <= 4 ? z >> (4 - leading) : z << Math.Min(leading - 4, 2);
            return (mantissa & 0x1FFC) | (leading << 13);
        }

        private static int DecompressDepth(int stored)
        {
            int packed = (stored >> 2) & 0x3FFF;
            int exponent = (packed >> 11) & 7;
            return (((packed & 0x7FF) << Math.Max(6 - exponent, 0)) + (0x40000 - (0x40000 >> exponent))) & 0x3FFFF;
        }

        // Four bits of a power of two's exponent.
        private static int DeltaZEncoding(int value) =>
            ((value & 0xFF00) != 0 ? 8 : 0) | ((value & 0xF0F0) != 0 ? 4 : 0) | ((value & 0xCCCC) != 0 ? 2 : 0) | ((value & 0xAAAA) != 0 ? 1 : 0);

        // The pixel's depth slope as the power of two above its magnitude; the reference's special value for one is indistinguishable - see §1.3.
        private static int NormalizeDeltaZ(int sum)
        {
            if ((sum & 0xC000) != 0) return 0x8000;
            if ((sum & 0xFFFF) == 0) return 1;
            return HighestBit(sum) << 1;
        }

        private static int HighestBit(int value) => value == 0 ? 0 : 1 << (31 - BitOperations.LeadingZeroCount((uint)value));

        // Returns whether the pixel survives; sets the blend decision, and may scale coverage between interpenetrating surfaces - see §3.2.
        private bool CompareDepth(uint index, int z, int deltaZ, int deltaZEncoded, int memoryCoverage, ref int coverage, out bool blend, out bool overflow)
        {
            z &= 0x3FFFF;
            overflow = ((memoryCoverage + coverage) & 8) != 0;

            // Shifts for memory alpha: this pixel's for the last blend, and the two-cycle mode's first blend from the previous pixel's slope - see Mars_RdpTwoCycle.md §4.
            bool twoCycle = CycleType == TwoCycle;
            bool shifts = (twoCycle ? SecondBlendCycle.SecondAlpha : BlendSecondAlpha) == 1;
            bool pastShifts = twoCycle && BlendSecondAlpha == 1;

            if (!DepthCompare)
            {
                blend = ForceBlend || (!overflow && Antialias);
                int far = deltaZEncoded < 0xB ? 4 : 0xF - deltaZEncoded;
                if (shifts) (_blendShiftA, _blendShiftB) = (0, far);
                if (pastShifts) (_pastShiftA, _pastShiftB) = (0, far);
                _pastStoredEncoded = 0xF;
                return true;
            }

            byte[] rdram = _bus.Rdram;
            Touch(index * 2);
            bool valid = index * 2 + 1 < rdram.Length;
            int stored = valid ? (rdram[index * 2] << 8) | rdram[index * 2 + 1] : 0;
            int storedHidden = valid ? _bus.RdramHidden[index] : 0;

            int old = DecompressDepth(stored);
            int storedEncoded = ((stored & 3) << 2) | storedHidden;
            int storedSlope = 1 << storedEncoded;

            if (shifts)
            {
                _blendShiftA = Math.Clamp(deltaZEncoded - storedEncoded, 0, 4);
                _blendShiftB = Math.Clamp(storedEncoded - deltaZEncoded, 0, 4);
            }

            if (pastShifts)
            {
                _pastShiftA = Math.Clamp(deltaZEncoded - _pastStoredEncoded, 0, 4);
                _pastShiftB = Math.Clamp(_pastStoredEncoded - deltaZEncoded, 0, 4);
            }

            _pastStoredEncoded = storedEncoded;

            // At low precision the stored slope widens; its largest value makes every margin exceed the depth range, which is all the reference's coplanar flag does - see §3.2.
            if (((stored >> 13) & 0xF) < 3)
            {
                storedSlope = storedSlope != 0x8000 ? Math.Max(storedSlope << 1, 16 >> ((stored >> 13) & 0xF)) : 0xFFFF;
            }

            int slope = HighestBit((deltaZ | storedSlope) & 0xFFFF);
            int margin = slope << 3;

            bool farther = z + margin >= old;
            blend = ForceBlend || (!overflow && Antialias && farther);

            bool inFront = z < old;
            bool nearer = z - margin <= old;
            bool maximum = old == 0x3FFFF;

            switch (DepthMode)
            {
                case 0:
                    return maximum || (overflow ? inFront : nearer);

                case 1:
                    if (!inFront || !farther || !overflow) return maximum || (overflow ? inFront : nearer);

                    int shift = DeltaZEncoding(slope & 0xFFFF);
                    coverage = ((((old >> shift) - (z >> shift)) & 0xF) * coverage >> 3) & 0xF;
                    return true;

                case 2:
                    return inFront || maximum;

                default:
                    return farther && nearer && !maximum;
            }
        }

        private void StoreDepth(uint index, int z, int deltaZEncoded)
        {
            byte[] rdram = _bus.Rdram;
            Touch(index * 2);
            if (index * 2 + 1 >= rdram.Length) return;

            int stored = CompressDepth(z & 0x3FFFF) | (deltaZEncoded >> 2);
            rdram[index * 2] = (byte)(stored >> 8);
            rdram[index * 2 + 1] = (byte)stored;
            _bus.RdramHidden[index] = (byte)(deltaZEncoded & 3);
        }

        // Shade scaled back from its extra precision, or moved to the first covered sample of a partial pixel - see §2.
        private int CorrectShade(int value, int correctDx, int correctDy, (byte X, byte Y) offset, int coverage)
        {
            int corrected = coverage == 8 ? value >> 2 : ((value << 2) + offset.X * correctDx + offset.Y * correctDy) >> 4;
            return Clamp9(corrected & 0x1FF);
        }

        private int CorrectDepth(int z, (byte X, byte Y) offset, int coverage)
        {
            int corrected = coverage == 8 ? z >> 3 : ((z << 2) + offset.X * _depthCorrectDx + offset.Y * _depthCorrectDy) >> 5;

            return ((corrected & 0x60000) >> 17) switch
            {
                2 => 0x3FFFF,
                3 => 0,
                _ => corrected & 0x3FFFF,
            };
        }

        private static (byte X, byte Y)[] BuildCoverageOffsets()
        {
            var table = new (byte X, byte Y)[256];

            for (int mask = 0; mask < 256; mask++)
            {
                int samples = (mask & 0x5) | ((mask & 0x5A) << 4) | ((mask & 0xA0) << 8);

                int row = 0;
                while (row < 3 && (samples & (0xF000 >> (row * 4))) == 0) row++;
                if ((samples & (0xF000 >> (row * 4))) == 0) row = 0;

                int columns = (samples >> (12 - row * 4)) & 0xF;
                int column = columns == 0 ? 0 : 3 - (31 - BitOperations.LeadingZeroCount((uint)columns));

                table[mask] = ((byte)column, (byte)row);
            }

            return table;
        }
    }
}
