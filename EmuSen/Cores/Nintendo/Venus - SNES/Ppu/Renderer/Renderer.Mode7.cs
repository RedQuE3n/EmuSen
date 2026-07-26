using System;
using System.Numerics;
using Raylib_cs;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Shell;
using EmuSen.Graphics;

namespace EmuSen.Cores.Nintendo.Venus.Video
{
    public partial class Renderer
    {
        // Mode 7 - BG1 as a single affine-transformed layer. See
        // Venus_PPU.md §3 for the transform formula, VRAM layout, and what's
        // not implemented.
        private void RenderMode7(Ppu ppu, int py, float brightness, Color[] target, int[] targetLayer, int layerId, bool isMainScreen)
        {
            bool hFlip = (ppu.M7Sel & 0x01) != 0;
            bool vFlip = (ppu.M7Sel & 0x02) != 0;
            bool screenOverEnabled = (ppu.M7Sel & 0x80) != 0;
            bool fillWithChar0 = (ppu.M7Sel & 0x40) != 0;

            int sy = vFlip ? 255 - py : py;
            int relY = sy + ppu.M7VOfs - ppu.M7Y;

            for (int px = 0; px < ScreenW; px++)
            {
                int raw = SampleMode7Pixel(ppu, px, relY, hFlip, screenOverEnabled, fillWithChar0);
                if (raw < 0) continue;

                byte colorIndex = (byte)raw;
                if (colorIndex != 0 && !IsWindowMasked(ppu, layerId, isMainScreen, px))
                {
                    // 8bpp indexes the full 256-color CGRAM directly - no
                    // palette-group offset needed, unlike 2bpp/4bpp modes.
                    int cgIdx = colorIndex * 2;
                    target[px] = SnesColor(ppu.Cgram[cgIdx & 0x1FF], ppu.Cgram[(cgIdx + 1) & 0x1FF], brightness);
                    targetLayer[px] = layerId;
                }
            }
        }

        // EXTBG (SETINI bit 6) - see Venus_PPU.md §3.4.
        private void RenderMode7Bg2Extbg(Ppu ppu, int py, float brightness, Color[] target, int[] targetLayer, int layerId, bool isMainScreen, bool highPriorityOnly)
        {
            bool hFlip = (ppu.M7Sel & 0x01) != 0;
            bool vFlip = (ppu.M7Sel & 0x02) != 0;
            bool screenOverEnabled = (ppu.M7Sel & 0x80) != 0;
            bool fillWithChar0 = (ppu.M7Sel & 0x40) != 0;

            int sy = vFlip ? 255 - py : py;
            int relY = sy + ppu.M7VOfs - ppu.M7Y;

            for (int px = 0; px < ScreenW; px++)
            {
                int raw = SampleMode7Pixel(ppu, px, relY, hFlip, screenOverEnabled, fillWithChar0);
                if (raw < 0) continue;

                bool highPriority = (raw & 0x80) != 0;
                if (highPriority != highPriorityOnly) continue;

                int colorIndex = raw & 0x7F;
                if (colorIndex != 0 && !IsWindowMasked(ppu, layerId, isMainScreen, px))
                {
                    int cgIdx = colorIndex * 2;
                    target[px] = SnesColor(ppu.Cgram[cgIdx & 0x1FF], ppu.Cgram[(cgIdx + 1) & 0x1FF], brightness);
                    targetLayer[px] = layerId;
                }
            }
        }

        // Shared by RenderMode7 (BG1) and RenderMode7Bg2Extbg - both sample
        // the identical transformed texture; only the interpretation of the
        // returned byte differs afterward. Returns -1 for "draw nothing at
        // this pixel" (transparent, or out-of-range with screen-over wrap
        // disabled and no character-0 fill), matching the same three cases
        // the original single-layer version handled inline.
        private int SampleMode7Pixel(Ppu ppu, int px, int relY, bool hFlip, bool screenOverEnabled, bool fillWithChar0)
        {
            int sx = hFlip ? 255 - px : px;
            int relX = sx + ppu.M7HOfs - ppu.M7X;

            // Matrix multiply in 8.8 fixed point, then back to whole
            // pixels (>>8) and re-add the pivot point.
            int texX = ((ppu.M7A * relX + ppu.M7B * relY) >> 8) + ppu.M7X;
            int texY = ((ppu.M7C * relX + ppu.M7D * relY) >> 8) + ppu.M7Y;

            bool outOfRange = texX < 0 || texX > 1023 || texY < 0 || texY > 1023;
            if (outOfRange && screenOverEnabled)
            {
                if (!fillWithChar0)
                {
                    return -1; // transparent - nothing drawn at this pixel
                }
                // Character 0 fill: repeats tile 0's own 8x8 pattern
                // using just the low 3 bits of each coordinate.
                return SampleMode7Tile(ppu, 0, texX & 7, texY & 7);
            }

            // Wrap within the 1024x1024 playing field (also the behavior
            // when screen-over is disabled entirely).
            int wrappedX = texX & 1023;
            int wrappedY = texY & 1023;
            int tx = wrappedX >> 3;
            int ty = wrappedY >> 3;

            int tilemapEntryAddr = (ty * 128 + tx) * 2; // low byte of this VRAM word
            int tileIndex = ppu.Vram[tilemapEntryAddr & 0xFFFF];

            return SampleMode7Tile(ppu, tileIndex, wrappedX & 7, wrappedY & 7);
        }

        // Reads one pixel's 8bpp color index from Mode 7 character data.
        // Character data lives in the HIGH byte of VRAM words (interleaved
        // with the tilemap's low bytes) - each 8x8 8bpp tile is 64
        // consecutive words (one word's high byte per pixel), unlike every
        // other mode's bitplane-interleaved storage.
        private byte SampleMode7Tile(Ppu ppu, int tileIndex, int col, int row)
        {
            int pixelWordAddr = tileIndex * 64 + row * 8 + col;
            int pixelByteAddr = pixelWordAddr * 2 + 1; // high byte
            return ppu.Vram[pixelByteAddr & 0xFFFF];
        }
    }
}
