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
            int lod = overflow ? 0 : Saturated(System.Math.Max(Movement(ns, fs), Movement(nt, ft)));

            (int level, bool magnify, bool distant, int fraction) = LevelSignals(lod, overflow, maxLevel);
            if (!LodEnabled) return (tile, fraction);

            if (distant) level = maxLevel;
            return ((tile + level + (DetailEnabled && !magnify ? 1 : 0)) & 7, fraction);
        }

        // Movement past fourteen bits keeps its low fifteen with bit 14 set.
        private static int Saturated(int movement) => (movement & 0x7FFF) | ((movement & 0x1C000) != 0 ? 0x4000 : 0);

        // Saturated, magnified or minified, with the level, whether the pixel is distant, and the fraction each gives - see §3.2.
        private (int Level, bool Magnify, bool Distant, int Fraction) LevelSignals(int lod, bool overflow, int maxLevel)
        {
            bool plain = !SharpenEnabled && !DetailEnabled;

            if ((lod & 0x4000) != 0 || overflow) return (0, false, true, 0xFF);

            if (lod < 32)
            {
                bool magnifiedDistant = maxLevel == 0;
                int magnified = plain ? (magnifiedDistant ? 0xFF : 0) : (System.Math.Max(lod, _minLevel) << 3) | (SharpenEnabled ? 0x100 : 0);
                return (0, true, magnifiedDistant, magnified);
            }

            int level = Log2[(lod >> 5) & 0xFF];
            bool distant = maxLevel == 0 || (lod & 0x6000) != 0 || level >= maxLevel;
            return (level, false, distant, plain && distant ? 0xFF : ((lod << 3) >> level) & 0xFF);
        }

        // The two-cycle mode measures from the pixel itself, across to the next pixel and down to the next row, for two tiles - see Mars_RdpTwoCycle.md §3.
        private (int First, int Second, int Fraction) TwoCycleLevelOfDetail(int s, int t, int w, int ds, int dt, int dw, int tile, int maxLevel)
        {
            (int cs, int ct) = DividedCoordinates(s, t, w);
            (int xs, int xt) = DividedCoordinates(s + ds, t + dt, w + dw);
            (int ys, int yt) = DividedCoordinates(s + DownStep(AttributeS), t + DownStep(AttributeT), w + DownStep(AttributeW));

            bool overflow = ((cs | ct | xs | xt | ys | yt) & 0x60000) != 0;
            int lod = 0;
            if (!overflow)
            {
                lod = Saturated(System.Math.Max(Movement(cs, xs), Movement(ct, xt)));
                lod = Saturated(System.Math.Max(lod, System.Math.Max(Movement(cs, ys), Movement(ct, yt))));
            }

            (int level, bool magnify, bool distant, int fraction) = LevelSignals(lod, overflow, maxLevel);
            if (!LodEnabled) return (tile, (tile + 1) & 7, fraction);

            if (distant) level = maxLevel;
            if (!DetailEnabled)
            {
                int first = (tile + level) & 7;
                return (first, distant || !SharpenEnabled && magnify ? first : (first + 1) & 7, fraction);
            }

            return ((tile + level + (magnify ? 0 : 1)) & 7, (tile + level + (!distant && !magnify ? 2 : 1)) & 7, fraction);
        }

        // Past a row's end, the next row's first pixel is measured down alone for the fraction and the first tile, and also across for the second - see Mars_RdpTwoCycle.md §3.
        private (int First, int Second, int Fraction) NextRowLevelOfDetail(int nextRow, int ds, int dt, int dw, int tile, int maxLevel)
        {
            (int rs, int rt, int rw) = RowStart(nextRow);
            (int cs, int ct) = DividedCoordinates(rs, rt, rw);
            (int ys, int yt) = DividedCoordinates(rs + DownStep(AttributeS), rt + DownStep(AttributeT), rw + DownStep(AttributeW));

            bool overflow = ((cs | ct | ys | yt) & 0x60000) != 0;
            int lod = overflow ? 0 : Saturated(System.Math.Max(Movement(cs, ys), Movement(ct, yt)));

            (int level, bool magnify, bool distant, int fraction) = LevelSignals(lod, overflow, maxLevel);
            if (!LodEnabled) return (tile, tile, fraction);

            int first = (tile + (distant ? maxLevel : level) + (DetailEnabled && !magnify ? 1 : 0)) & 7;

            (int xs, int xt) = DividedCoordinates(rs + ds, rt + dt, rw + dw);
            overflow |= ((xs | xt) & 0x60000) != 0;
            if (!overflow) lod = Saturated(System.Math.Max(lod, System.Math.Max(Movement(cs, xs), Movement(ct, xt))));

            (level, magnify, distant, _) = LevelSignals(lod, overflow, maxLevel);
            return (first, (tile + (distant ? maxLevel : level) + (DetailEnabled && !magnify ? 1 : 0)) & 7, fraction);
        }

        // A texture coordinate's step down one row, with its low fifteen bits cleared.
        private int DownStep(int attribute) => _attributeDy[attribute] & ~0x7FFF;

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
