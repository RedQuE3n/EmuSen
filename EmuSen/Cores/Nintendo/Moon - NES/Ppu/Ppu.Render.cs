using System;

namespace EmuSen.Cores.Nintendo.Moon.Video
{
    // Scanline composition: background, then sprites over it - see Moon_PPU.md §3.
    public sealed partial class Ppu
    {
        // The 64 NTSC colours as RGB triples; emphasis bits are not applied - see Moon_PPU.md §3.5.
        public static readonly byte[] NesPalette =
        {
            84, 84, 84,     0, 30, 116,     8, 16, 144,    48, 0, 136,    68, 0, 100,    92, 0, 48,     84, 4, 0,      60, 24, 0,
            32, 42, 0,      8, 58, 0,       0, 64, 0,      0, 60, 0,      0, 50, 60,     0, 0, 0,       0, 0, 0,       0, 0, 0,
            152, 150, 152,  8, 76, 196,     48, 50, 236,   92, 30, 228,   136, 20, 176,  160, 20, 100,  152, 34, 32,   120, 60, 0,
            84, 90, 0,      40, 114, 0,     8, 124, 0,     0, 118, 40,    0, 102, 120,   0, 0, 0,       0, 0, 0,       0, 0, 0,
            236, 238, 236,  76, 154, 236,   120, 124, 236, 176, 98, 236,  228, 84, 236,  236, 88, 180,  236, 106, 100, 212, 136, 32,
            160, 170, 0,    116, 196, 0,    76, 208, 32,   56, 204, 108,  56, 180, 204,  60, 60, 60,    0, 0, 0,       0, 0, 0,
            236, 238, 236,  168, 204, 236,  188, 188, 236, 212, 178, 236, 236, 174, 236, 236, 174, 212, 236, 180, 176, 228, 196, 144,
            204, 210, 120,  180, 222, 120,  168, 226, 144, 152, 226, 180, 160, 214, 228, 160, 162, 160, 0, 0, 0,       0, 0, 0,
        };

        // Called once per scanline by MoonCore, after that line's CPU time - see Moon_PPU.md §1.1.
        public void EndScanline(int line)
        {
            if (line < VisibleScanlines)
            {
                RenderScanline(line);
                AdvanceScroll();
                if (RenderingEnabled) _cart.Mapper.OnScanline();
            }
            else if (line == VBlankScanline)
            {
                VBlankFlag = true;
            }
            else if (line == PreRenderScanline)
            {
                VBlankFlag = false;
                Sprite0Hit = false;
                SpriteOverflow = false;
                AdvanceScroll();
                if (RenderingEnabled) CopyVertical();
                FrameCount++;
            }
        }

        private void AdvanceScroll()
        {
            if (!RenderingEnabled) return;
            IncrementY();
            CopyHorizontal();
        }

        // Coarse X wraps by flipping the horizontal nametable bit, not by carrying into coarse Y.
        private void IncrementCoarseX(ref ushort v)
        {
            if ((v & 0x001F) == 0x001F)
            {
                v = (ushort)((v & ~0x001F) ^ 0x0400);
            }
            else
            {
                v++;
            }
        }

        // Coarse Y wraps at 29, not 31 - rows 30 and 31 are the attribute table, not tiles.
        private void IncrementY()
        {
            if ((V & 0x7000) != 0x7000)
            {
                V += 0x1000;
                return;
            }

            V &= unchecked((ushort)~0x7000);
            int y = (V & 0x03E0) >> 5;

            if (y == 29)
            {
                y = 0;
                V ^= 0x0800;
            }
            else if (y == 31)
            {
                y = 0;
            }
            else
            {
                y++;
            }

            V = (ushort)((V & ~0x03E0) | (y << 5));
        }

        private void CopyHorizontal() => V = (ushort)((V & ~0x041F) | (T & 0x041F));

        private void CopyVertical() => V = (ushort)((V & ~0x7BE0) | (T & 0x7BE0));

        private void RenderScanline(int line)
        {
            Array.Clear(_bgLine);

            if (ShowBackground) RenderBackground();
            EvaluateSprites(line);
            Composite(line);
        }

        // Walks 33 tiles so fine X can shift the first one partly off the left edge.
        private void RenderBackground()
        {
            ushort v = V;
            int fineY = (v >> 12) & 0x07;
            int screenX = -FineX;

            while (screenX < ScreenWidth)
            {
                byte tile = ReadCiram((ushort)(0x2000 | (v & 0x0FFF)));

                ushort attributeAddress = (ushort)(0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07));
                byte attribute = ReadCiram(attributeAddress);

                // Which 2x2-tile quadrant of the attribute byte this tile sits in.
                int quadrant = ((v >> 4) & 0x04) | (v & 0x02);
                int paletteNumber = (attribute >> quadrant) & 0x03;

                int patternAddress = BackgroundPatternBase + (tile * 16) + fineY;
                byte low = _cart.Mapper.ReadChr((ushort)patternAddress);
                byte high = _cart.Mapper.ReadChr((ushort)(patternAddress + 8));

                for (int pixel = 0; pixel < 8; pixel++)
                {
                    int x = screenX + pixel;
                    if ((uint)x >= ScreenWidth) continue;

                    int bit = 7 - pixel;
                    int color = ((low >> bit) & 0x01) | (((high >> bit) & 0x01) << 1);
                    _bgLine[x] = (byte)(color == 0 ? 0 : (paletteNumber * 4) + color);
                }

                screenX += 8;
                IncrementCoarseX(ref v);
            }
        }

        // Eight per line, in OAM order; the ninth only sets the overflow flag.
        private void EvaluateSprites(int line)
        {
            _spriteCount = 0;
            int height = SpritesAre8x16 ? 16 : 8;

            for (int i = 0; i < 64; i++)
            {
                int row = line - Oam[i * 4] - 1;
                if (row < 0 || row >= height) continue;

                if (_spriteCount == SpritesPerLine)
                {
                    SpriteOverflow = true;
                    break;
                }

                _spriteIndices[_spriteCount++] = i;
            }
        }

        private void Composite(int line)
        {
            int height = SpritesAre8x16 ? 16 : 8;
            int frameOffset = line * ScreenWidth * 4;

            for (int x = 0; x < ScreenWidth; x++)
            {
                int bgEntry = _bgLine[x];
                bool bgOpaque = bgEntry != 0 && (x >= 8 || ShowBackgroundLeft);

                int spriteEntry = 0;
                bool spriteInFront = false;
                bool spriteIsZero = false;

                if (ShowSprites && (x >= 8 || ShowSpritesLeft))
                {
                    for (int s = 0; s < _spriteCount; s++)
                    {
                        int index = _spriteIndices[s];
                        int spriteX = Oam[(index * 4) + 3];
                        int column = x - spriteX;
                        if (column < 0 || column >= 8) continue;

                        int color = SpritePixel(index, line, column, height);
                        if (color == 0) continue;

                        byte attributes = Oam[(index * 4) + 2];
                        spriteEntry = 0x10 + ((attributes & 0x03) * 4) + color;
                        spriteInFront = (attributes & 0x20) == 0;
                        spriteIsZero = index == 0;
                        break;
                    }
                }

                // The rightmost pixel never reports a hit, and both layers must be on.
                if (spriteIsZero && bgOpaque && x != ScreenWidth - 1 && ShowBackground)
                {
                    Sprite0Hit = true;
                }

                int entry = spriteEntry != 0 && (spriteInFront || !bgOpaque)
                    ? spriteEntry
                    : bgOpaque ? bgEntry : 0;

                if (SkipRendering) continue;

                byte nesColor = PaletteRam[PaletteOffset((ushort)(0x3F00 + entry))];
                if (Grayscale) nesColor &= 0x30;

                int paletteOffset = (nesColor & 0x3F) * 3;
                int pixelOffset = frameOffset + (x * 4);

                FrameRgba[pixelOffset] = NesPalette[paletteOffset];
                FrameRgba[pixelOffset + 1] = NesPalette[paletteOffset + 1];
                FrameRgba[pixelOffset + 2] = NesPalette[paletteOffset + 2];
                FrameRgba[pixelOffset + 3] = 0xFF;
            }
        }

        private int SpritePixel(int index, int line, int column, int height)
        {
            byte tile = Oam[(index * 4) + 1];
            byte attributes = Oam[(index * 4) + 2];

            int row = line - Oam[index * 4] - 1;
            if ((attributes & 0x80) != 0) row = height - 1 - row;
            if ((attributes & 0x40) != 0) column = 7 - column;

            int patternAddress;
            if (SpritesAre8x16)
            {
                // Bit 0 of the tile number picks the pattern table, and the pair is always even-aligned.
                int table = (tile & 0x01) * 0x1000;
                int tileIndex = tile & 0xFE;
                if (row >= 8)
                {
                    tileIndex++;
                    row -= 8;
                }
                patternAddress = table + (tileIndex * 16) + row;
            }
            else
            {
                patternAddress = SpritePatternBase + (tile * 16) + row;
            }

            byte low = _cart.Mapper.ReadChr((ushort)patternAddress);
            byte high = _cart.Mapper.ReadChr((ushort)(patternAddress + 8));

            int bit = 7 - column;
            return ((low >> bit) & 0x01) | (((high >> bit) & 0x01) << 1);
        }
    }
}
