namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx
{
    // The plot hardware: PLOT/RPIX/COLOR/CMODE plus the 8-pixel cache that makes them practical - see Venus_SuperFX.md §6.
    public sealed partial class SuperFx
    {
        // One tile row of pending pixels, index 0 leftmost.
        private byte[] _pixelCache = new byte[8];
        private int _pixelCacheAddress = -1;
        private byte _pixelCachePending;

        // Answers "is this game plotting at all, and in which layout" without instrumentation - POR/SCMR - see Venus_SuperFX.md §8.
        public long DebugPlotCount { get; private set; }
        public long DebugPlotObjCount { get; private set; }
        public byte DebugPlotScmr { get; private set; }
        public byte DebugPlotPor { get; private set; }
        public byte DebugPlotScbr { get; private set; }
        public int DebugPlotMaxX { get; private set; }
        public int DebugPlotMaxY { get; private set; }
        public long DebugPlotXHigh { get; private set; }
        public long DebugPlotYOdd { get; private set; }
        public int DebugPlotMinAddr { get; private set; } = int.MaxValue;
        public int DebugPlotMaxAddr { get; private set; }

        private void ResetPixelCache()
        {
            _pixelCacheAddress = -1;
            _pixelCachePending = 0;
        }

        // SCMR bits 1-0 pick the colour depth; 8bpp is the only one that uses a whole byte per pixel.
        private int ColorDepth => (_scmr & 0x03) switch
        {
            0 => 2,
            3 => 8,
            _ => 4,
        };

        // HT0 is bit 2 and HT1 is bit 5 - the two halves of the field are not adjacent.
        private int ScreenHeightMode => (((_scmr >> 5) & 1) << 1) | ((_scmr >> 2) & 1);

        // CMODE bit 4 selects OBJ mode independently of SCMR's height field - see Venus_SuperFX.md §6.2.
        private bool ObjMode => ScreenHeightMode == 3 || (_por & 0x10) != 0;

        private int ColumnHeightTiles => ScreenHeightMode switch
        {
            0 => 16,  // 128 pixels
            1 => 20,  // 160 pixels
            2 => 24,  // 192 pixels
            _ => 16,  // OBJ mode - see Venus_SuperFX.md §6.2
        };

        // Tiles are stored in vertical strips, which is why the mode register specifies a height: it is the - see Venus_SuperFX.md §6.1.
        private int TileRowAddress(int x, int y)
        {
            int bytesPerTile = 8 * ColorDepth;

            int tile;
            if (ObjMode)
            {
                int page = ((y >> 7) << 1) | ((x >> 7) & 1);
                tile = (page << 8) | (((y >> 3) & 0x0F) << 4) | ((x >> 3) & 0x0F);
            }
            else
            {
                tile = ((x >> 3) * ColumnHeightTiles) + (y >> 3);
            }

            return (_scbr << 10) + (tile * bytesPerTile) + ((y & 7) * 2);
        }

        private byte ColorValue(byte source)
        {
            byte result = source;
            if ((_por & 0x04) != 0) result = (byte)((result & 0xF0) | (result >> 4));
            if ((_por & 0x08) != 0) result = (byte)((_colr & 0xF0) | (result & 0x0F));
            return result;
        }

        private int OpColor()
        {
            _colr = ColorValue((byte)Src);
            return 1;
        }

        private int OpCmode()
        {
            _por = (byte)(Src & 0x1F);
            return 1;
        }

        private void LoadColorFromRomBuffer() => _colr = ColorValue(_romBuffer);

        // R1/R2 are full 16-bit registers, but the plot hardware only sees their low bytes - R1 runs past 255 - see Venus_SuperFX.md §6.
        private int OpPlot()
        {
            Plot((byte)R[1], (byte)R[2]);
            WriteReg(1, (ushort)(R[1] + 1));
            return 1;
        }

        private int OpRpix()
        {
            FlushPixelCache();
            ushort value = ReadPixel((byte)R[1], (byte)R[2]);
            Dst(value);
            SetZS(value);
            return 5;
        }

        // POR bit 3 freezes the high nibble, so only the low one decides.
        private bool IsTransparent()
        {
            byte color = (_por & 0x08) != 0 ? (byte)(_colr & 0x0F) : _colr;
            return ColorDepth switch
            {
                2 => (color & 0x03) == 0,
                8 => color == 0,
                _ => (color & 0x0F) == 0,
            };
        }

        // What the chip was actually asked to draw, which per-frame register sampling cannot see - see Venus_SuperFX.md §8.
        private void RecordPlotForDebug(int x, int y)
        {
            DebugPlotCount++;
            if (ObjMode) DebugPlotObjCount++;
            DebugPlotScmr = _scmr;
            DebugPlotPor = _por;
            DebugPlotScbr = _scbr;
            if (x > DebugPlotMaxX) DebugPlotMaxX = x;
            if (y > DebugPlotMaxY) DebugPlotMaxY = y;
            if ((x & 0x80) != 0) DebugPlotXHigh++;
            if ((y & 0x08) != 0) DebugPlotYOdd++;

            if (EmuSen.Debug.DebugSettings.SuperFxPlotTraceSkip > 0)
            {
                EmuSen.Debug.DebugSettings.SuperFxPlotTraceSkip--;
            }
            else if (EmuSen.Debug.DebugSettings.SuperFxPlotTraceCountdown > 0)
            {
                EmuSen.Debug.DebugSettings.SuperFxPlotTraceCountdown--;
                System.Console.WriteLine(
                    $"[PLOT] x={x:D3} y={y:D3} r1={R[1]:X4} r2={R[2]:X4} colr={_colr:X2} "
                    + $"por={_por:X2} scmr={_scmr:X2} scbr={_scbr:X2} addr={TileRowAddress(x, y):X4} pc={_pbr:X2}:{R[15]:X4}");
            }
        }

        private void Plot(int x, int y)
        {
            RecordPlotForDebug(x, y);

            // Colour 0 is transparent unless POR bit 0 says otherwise, and the test reads COLR before dithering - see Venus_SuperFX.md §6.
            if ((_por & 0x01) == 0 && IsTransparent()) return;

            byte color = _colr;

            // Dithering alternates which nibble of COLR is used per pixel.
            if ((_por & 0x02) != 0 && ColorDepth != 8)
            {
                if (((x ^ y) & 1) != 0) color >>= 4;
                color &= 0x0F;
            }

            int address = TileRowAddress(x, y);
            if (address < DebugPlotMinAddr) DebugPlotMinAddr = address;
            if (address > DebugPlotMaxAddr) DebugPlotMaxAddr = address;

            if (address != _pixelCacheAddress)
            {
                FlushPixelCache();
                _pixelCacheAddress = address;
            }

            int index = x & 7;
            _pixelCache[index] = color;
            _pixelCachePending |= (byte)(1 << index);
        }

        // Writes the pending pixels out as bitplanes, preserving whatever the untouched pixel positions.
        private void FlushPixelCache()
        {
            if (_pixelCachePending == 0 || _pixelCacheAddress < 0) return;

            int depth = ColorDepth;
            for (int plane = 0; plane < depth; plane++)
            {
                int address = _pixelCacheAddress + ((plane >> 1) * 16) + (plane & 1);
                byte value = ReadRam(address);

                for (int i = 0; i < 8; i++)
                {
                    if ((_pixelCachePending & (1 << i)) == 0) continue;
                    int bit = 7 - i;
                    if (((_pixelCache[i] >> plane) & 1) != 0) value |= (byte)(1 << bit);
                    else value &= (byte)~(1 << bit);
                }

                WriteRam(address, value);
            }

            _pixelCachePending = 0;
        }

        private ushort ReadPixel(int x, int y)
        {
            int address = TileRowAddress(x, y);
            int bit = 7 - (x & 7);
            int depth = ColorDepth;

            int color = 0;
            for (int plane = 0; plane < depth; plane++)
            {
                byte value = ReadRam(address + ((plane >> 1) * 16) + (plane & 1));
                color |= ((value >> bit) & 1) << plane;
            }
            return (ushort)color;
        }
    }
}
