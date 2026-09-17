namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // The level of detail: how far a pixel's texture coordinates move, the tile that picks and the fraction the combiner reads - see Mars_RdpLod.md.
    public sealed partial class Rdp
    {
        private static readonly byte[] Log2 = BuildLog2();

        // The next pixel's coordinates against the one after, or near a span's end the one before, or the next row's - see §2.
        private (int Tile, int Fraction) PixelLevelOfDetail(int s, int t, int w, int ds, int dt, int dw, int nextRow, bool nextRowDrawn,
            bool end, bool beforeEnd, bool longSpan, bool midSpan, int tile, int maxLevel)
        {
            if (nextRowDrawn && end && longSpan)
            {
                (int rs, int rt, int rw) = RowStart(nextRow);
                return LevelOfDetail(rs, rt, rw, rs + ds, rt + dt, rw + dw, tile, maxLevel);
            }

            return nextRowDrawn && (beforeEnd && longSpan || end && midSpan)
                ? LevelOfDetail(s + ds, t + dt, w + dw, s - ds, t - dt, w - dw, tile, maxLevel)
                : LevelOfDetail(s + ds, t + dt, w + dw, s + (ds << 1), t + (dt << 1), w + (dw << 1), tile, maxLevel);
        }

        // The tile of a span's last texel 1, measured from the pixel after the span by the span's length - see §2.
        private int AfterSpanTile(int s, int t, int w, int ds, int dt, int dw, int nextRow, bool nextRowDrawn, bool longSpan, bool midSpan,
            bool oneBeforeMid, int tile, int maxLevel)
        {
            if (nextRowDrawn && (longSpan || midSpan))
            {
                (int rs, int rt, int rw) = RowStart(nextRow);
                if (longSpan) (rs, rt, rw) = (rs + ds, rt + dt, rw + dw);
                return LevelOfDetail(rs, rt, rw, rs + ds, rt + dt, rw + dw, tile, maxLevel).Tile;
            }

            return nextRowDrawn && oneBeforeMid
                ? LevelOfDetail(s + ds, t + dt, w + dw, s - ds, t - dt, w - dw, tile, maxLevel).Tile
                : LevelOfDetail(s + ds, t + dt, w + dw, s + (ds << 1), t + (dt << 1), w + (dw << 1), tile, maxLevel).Tile;
        }

        private (int S, int T, int W) RowStart(int row)
        {
            int at = row * Attributes;
            return (_spanAttributes[at + AttributeS], _spanAttributes[at + AttributeT], _spanAttributes[at + AttributeW]);
        }

        // The larger of s's and t's movement, from coordinates that did not overflow the divider, gives the tile and fraction - see §3.
        private (int Tile, int Fraction) LevelOfDetail(int nearS, int nearT, int nearW, int farS, int farT, int farW, int tile, int maxLevel)
        {
            (int ns, int nt) = DividedCoordinates(nearS, nearT, nearW);
            (int fs, int ft) = DividedCoordinates(farS, farT, farW);

            bool overflow = ((ns | nt | fs | ft) & 0x60000) != 0;
            int lod = 0;
            if (!overflow)
            {
                int movement = System.Math.Max(Movement(ns, fs), Movement(nt, ft));
                lod = movement & 0x7FFF;
                if ((movement & 0x1C000) != 0) lod |= 0x4000;
            }

            bool magnify, distant;
            int level, fraction;
            bool plain = !SharpenEnabled && !DetailEnabled;

            if ((lod & 0x4000) != 0 || overflow)
            {
                (magnify, level, distant, fraction) = (false, 0, true, 0xFF);
            }
            else if (lod < 32)
            {
                (magnify, level, distant) = (true, 0, maxLevel == 0);
                fraction = plain ? (distant ? 0xFF : 0) : (System.Math.Max(lod, _minLevel) << 3) | (SharpenEnabled ? 0x100 : 0);
            }
            else
            {
                (magnify, level) = (false, Log2[(lod >> 5) & 0xFF]);
                distant = maxLevel == 0 || (lod & 0x6000) != 0 || level >= maxLevel;
                fraction = plain && distant ? 0xFF : ((lod << 3) >> level) & 0xFF;
            }

            if (!LodEnabled) return (tile, fraction);

            if (distant) level = maxLevel;
            return ((tile + level + (DetailEnabled && !magnify ? 1 : 0)) & 7, fraction);
        }

        // Seventeen-bit coordinates subtracted, and a negative difference taken as its complement rather than negated - see §3.
        private static int Movement(int from, int to)
        {
            int difference = SignExtend((uint)to, 17) - SignExtend((uint)from, 17);
            return (difference & 0x20000) != 0 ? ~difference & 0x1FFFF : difference;
        }

        private static byte[] BuildLog2()
        {
            var table = new byte[256];
            for (int i = 2; i < table.Length; i++) table[i] = (byte)(31 - System.Numerics.BitOperations.LeadingZeroCount((uint)i));
            return table;
        }
    }
}
