using System;

namespace EmuSen.Cores.Nintendo.Mercury.Video
{
    // Line composition: background, window over it, then sprites - see Mercury_Ppu.md §4 and §5.
    public sealed partial class Ppu
    {
        // Four neutral greys, not the panel's green cast - see Mercury_Ppu.md §6.
        public static readonly byte[] DmgShades =
        {
            0xFF, 0xFF, 0xFF,
            0xAA, 0xAA, 0xAA,
            0x55, 0x55, 0x55,
            0x00, 0x00, 0x00,
        };

        internal void RenderScanline(int line)
        {
            bool windowOnThisLine = WindowEnabled && WindowTriggered && Wx <= 166;

            if (!SkipRendering)
            {
                RenderBackground(line, windowOnThisLine);
                if (SpritesEnabled) RenderSprites(line);
            }

            // Bookkeeping outside the render guard, or fast-forward would desync the window.
            if (windowOnThisLine) WindowLine++;
        }

        private void RenderBackground(int line, bool windowOnThisLine)
        {
            // On a DMG, clearing LCDC bit 0 blanks both layers; on a CGB the same bit means something else - see Mercury_Cgb.md §3.
            if (!Cgb && !BgEnabled)
            {
                Array.Clear(_bgColorIndex);
                Array.Clear(_bgPriority);
                for (int x = 0; x < ScreenWidth; x++) WritePixel(line, x, 0);
                return;
            }

            byte[] vram = _bus.Vram;
            int windowStartX = Wx - 7;
            int backgroundY = (line + Scy) & 0xFF;

            for (int x = 0; x < ScreenWidth; x++)
            {
                bool inWindow = windowOnThisLine && x >= windowStartX;

                int mapBase = inWindow ? WindowTileMapBase : BgTileMapBase;
                int mapX = inWindow ? x - windowStartX : (x + Scx) & 0xFF;
                int mapY = inWindow ? WindowLine : backgroundY;

                int mapEntry = mapBase + ((mapY >> 3) << 5) + ((mapX >> 3) & 0x1F);
                byte tile = vram[mapEntry];

                // The attribute for a map entry lives at the same address in VRAM bank 1.
                byte attributes = Cgb ? vram[mapEntry + VramBankStride] : (byte)0x00;

                int row = mapY & 0x07;
                if ((attributes & 0x40) != 0) row = 7 - row;

                int address = TileRowAddress(tile, row) + ((attributes & 0x08) != 0 ? VramBankStride : 0);

                int column = mapX & 0x07;
                int bit = (attributes & 0x20) != 0 ? column : 7 - column;
                int color = ((vram[address] >> bit) & 0x01) | (((vram[address + 1] >> bit) & 0x01) << 1);

                _bgColorIndex[x] = (byte)color;
                _bgPriority[x] = (attributes & 0x80) != 0;

                if (Cgb) WriteColorPixel(line, x, BgPaletteRam, attributes & 0x07, color);
                else WritePixel(line, x, Shade(Bgp, color));
            }
        }

        // $8000 addressing indexes unsigned from the base; $8800 addressing is signed around $9000.
        private int TileRowAddress(byte tile, int row) =>
            (TileDataIsUnsigned ? tile * 16 : 0x1000 + ((sbyte)tile * 16)) + (row * 2);

        private void RenderSprites(int line)
        {
            SelectSprites(line);
            if (_spriteCount == 0) return;

            Array.Clear(_spriteClaimed);

            byte[] oam = _bus.Oam;
            byte[] vram = _bus.Vram;
            int height = SpriteHeight;

            for (int s = 0; s < _spriteCount; s++)
            {
                int entry = _spriteIndices[s] * 4;

                int spriteY = oam[entry] - 16;
                int spriteX = oam[entry + 1] - 8;
                byte tile = oam[entry + 2];
                byte attributes = oam[entry + 3];

                int row = line - spriteY;
                if ((attributes & 0x40) != 0) row = height - 1 - row;

                // An 8x16 sprite's tile number has its low bit ignored; row 8-15 walks into the pair's second tile.
                if (height == 16) tile &= 0xFE;

                int address = (tile * 16) + (row * 2);
                if (Cgb && (attributes & 0x08) != 0) address += VramBankStride;

                byte low = vram[address];
                byte high = vram[address + 1];

                byte palette = (attributes & 0x10) != 0 ? Obp1 : Obp0;
                bool behindBackground = (attributes & 0x80) != 0;

                for (int column = 0; column < 8; column++)
                {
                    int x = spriteX + column;
                    if ((uint)x >= ScreenWidth || _spriteClaimed[x]) continue;

                    int bit = (attributes & 0x20) != 0 ? column : 7 - column;
                    int color = ((low >> bit) & 0x01) | (((high >> bit) & 0x01) << 1);
                    if (color == 0) continue;

                    // The pixel is spoken for even when the background wins it - see Mercury_Ppu.md §5.2.
                    _spriteClaimed[x] = true;
                    if (BackgroundWins(x, behindBackground)) continue;

                    if (Cgb) WriteColorPixel(line, x, ObjPaletteRam, attributes & 0x07, color);
                    else WritePixel(line, x, Shade(palette, color));
                }
            }
        }

        // On a CGB the background gets a second way to claim the pixel, and LCDC bit 0 can waive both - see Mercury_Cgb.md §3.
        private bool BackgroundWins(int x, bool spriteIsBehind)
        {
            if (_bgColorIndex[x] == 0) return false;
            if (!Cgb) return spriteIsBehind;

            return BgEnabled && (spriteIsBehind || _bgPriority[x]);
        }

        // Ten per line, taken in OAM order; then reordered so the leftmost sprite draws first - see Mercury_Ppu.md §5.1.
        private void SelectSprites(int line)
        {
            _spriteCount = 0;

            byte[] oam = _bus.Oam;
            int height = SpriteHeight;

            for (int i = 0; i < SpriteCount && _spriteCount < SpritesPerLine; i++)
            {
                int spriteY = oam[i * 4] - 16;
                if (line < spriteY || line >= spriteY + height) continue;

                _spriteIndices[_spriteCount++] = i;
            }

            // A CGB resolves overlap by OAM index alone, so the selection order is already the priority order.
            if (!Cgb) SortByDmgPriority(oam);
        }

        // Insertion sort on X, and it must be stable so equal X keeps the lower OAM index in front.
        private void SortByDmgPriority(byte[] oam)
        {
            for (int i = 1; i < _spriteCount; i++)
            {
                int index = _spriteIndices[i];
                int x = oam[(index * 4) + 1];

                int j = i - 1;
                while (j >= 0 && oam[(_spriteIndices[j] * 4) + 1] > x)
                {
                    _spriteIndices[j + 1] = _spriteIndices[j];
                    j--;
                }

                _spriteIndices[j + 1] = index;
            }
        }

        private static int Shade(byte palette, int color) => (palette >> (color * 2)) & 0x03;

        private void WritePixel(int line, int x, int shade)
        {
            int source = shade * 3;
            int offset = ((line * ScreenWidth) + x) * 4;

            FrameRgba[offset] = DmgShades[source];
            FrameRgba[offset + 1] = DmgShades[source + 1];
            FrameRgba[offset + 2] = DmgShades[source + 2];
            FrameRgba[offset + 3] = 0xFF;
        }

        internal void ClearScreen()
        {
            for (int i = 0; i < FrameRgba.Length; i++) FrameRgba[i] = 0xFF;
        }
    }
}
