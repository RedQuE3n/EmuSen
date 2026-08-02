namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx
{
    // The plot hardware: PLOT/RPIX/COLOR/CMODE plus the 8-pixel cache that
    // makes them practical. The framebuffer lives in Game Pak RAM already in
    // SNES planar tile format, so the S-CPU can DMA it straight to VRAM - see
    // Venus_SuperFX.md §6.
    public sealed partial class SuperFx
    {
        // One tile row of pending pixels, index 0 leftmost.
        private byte[] _pixelCache = new byte[8];
        private int _pixelCacheAddress = -1;
        private byte _pixelCachePending;

        private void ResetPixelCache()
        {
            _pixelCacheAddress = -1;
            _pixelCachePending = 0;
        }

        // SCMR bits 1-0 pick the colour depth; 8bpp is the only one that uses
        // a whole byte per pixel.
        private int ColorDepth => (_scmr & 0x03) switch
        {
            0 => 2,
            3 => 8,
            _ => 4,
        };

        // HT1 is bit 2 and HT0 is bit 5 - the two halves of the field are not adjacent.
        private int ScreenHeightMode => (((_scmr >> 2) & 1) << 1) | ((_scmr >> 5) & 1);

        private int ColumnHeightTiles => ScreenHeightMode switch
        {
            0 => 16,  // 128 pixels
            1 => 20,  // 160 pixels
            2 => 24,  // 192 pixels
            _ => 16,  // OBJ mode - see Venus_SuperFX.md §6.2
        };

        // Tiles are stored in vertical strips, which is why the mode register
        // specifies a height: it is the column stride - see Venus_SuperFX.md §6.1.
        // OBJ mode instead lays the buffer out the way the PPU wants sprite
        // tiles, in 128x128 pages of 16x16 tiles - see Venus_SuperFX.md §6.2.
        private int TileRowAddress(int x, int y)
        {
            int bytesPerTile = 8 * ColorDepth;

            int tile;
            if (ScreenHeightMode == 3)
            {
                int page = ((y >> 7) << 1) | ((x >> 7) & 1);
                tile = (page << 8) | (((x >> 3) & 0x0F) << 4) | ((y >> 3) & 0x0F);
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

        private int OpPlot()
        {
            Plot(R[1], R[2]);
            WriteReg(1, (ushort)(R[1] + 1));
            return 1;
        }

        private int OpRpix()
        {
            FlushPixelCache();
            ushort value = ReadPixel(R[1], R[2]);
            Dst(value);
            SetZS(value);
            return 5;
        }

        private void Plot(int x, int y)
        {
            byte color = _colr;

            // Dithering alternates which nibble of COLR is used per pixel.
            if ((_por & 0x02) != 0 && ColorDepth != 8)
            {
                if (((x ^ y) & 1) != 0) color >>= 4;
                color &= 0x0F;
            }

            // Colour 0 is transparent unless POR bit 0 says otherwise.
            if ((_por & 0x01) == 0 && (_por & 0x10) == 0)
            {
                if (ColorDepth == 8)
                {
                    if ((_por & 0x04) != 0 ? (color & 0xF0) == 0 : color == 0) return;
                }
                else if ((color & 0x0F) == 0)
                {
                    return;
                }
            }

            int address = TileRowAddress(x, y);
            if (address != _pixelCacheAddress)
            {
                FlushPixelCache();
                _pixelCacheAddress = address;
            }

            int index = x & 7;
            _pixelCache[index] = color;
            _pixelCachePending |= (byte)(1 << index);
        }

        // Writes the pending pixels out as bitplanes, preserving whatever the
        // untouched pixel positions already held.
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
