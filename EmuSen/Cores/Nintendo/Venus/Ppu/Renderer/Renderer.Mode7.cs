using System;
using System.Numerics;
using Raylib_cs;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Debug;
using EmuSen.Graphics;

namespace EmuSen.Cores.Nintendo.Venus.Video
{
    public partial class Renderer
    {
        // Mode 7: BG1 becomes a single affine-transformed 1024x1024-pixel
        // layer instead of a normal scrolled tilemap. No priority-bit split
        // (unlike RenderBg1-4), no bit-depth variation (always 8bpp) - this
        // is genuinely a different rendering algorithm, not a variant of the
        // tilemap path the other RenderBg* methods share.
        //
        // Formula verified against two independent sources (SNESdev wiki's
        // "Mode 7 transform" page and a NESDev forum matrix-form writeup)
        // that agree byte-for-byte:
        //   [X]   [A B]   [SX + HOFS - CX]   [CX]
        //   [ ] = [   ] * [                ] + [  ]
        //   [Y]   [C D]   [SY + VOFS - CY]   [CY]
        // where (SX,SY) is the screen pixel, (CX,CY) = (M7X,M7Y) is the
        // pivot point, (HOFS,VOFS) = (M7HOFS,M7VOFS) is Mode 7's own scroll,
        // and A/B/C/D are signed 8.8 fixed-point (raw value / 256.0).
        //
        // VRAM layout is also genuinely different from every other mode:
        // the 128x128-tile tilemap lives in the LOW byte of each VRAM word
        // starting at word 0 (one byte per entry - just an 8-bit tile
        // index, no flip/priority/palette bits), and the 8bpp tile/character
        // data lives in the HIGH byte of those same words, interleaved.
        // Confirmed against the SNESdev wiki's Tilemaps and Tiles pages and
        // SnesLab's Mode 7 VRAM Map page independently.
        //
        // Not implemented in this pass: the documented 13-bit CLIP()
        // precision quirk on the intermediate SX+HOFS-CX/SY+VOFS-CY values
        // (an obscure edge case that only matters for extreme
        // rotation/scale parameters) - flagged rather than silently
        // approximated. EXTBG (see RenderMode7Bg2Extbg below) IS now
        // implemented, unlike the note that used to be here.
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

        // EXTBG (SETINI $2133 bit 6) gives Mode 7 a second layer, BG2, which
        // samples the EXACT SAME affine-transformed texture as BG1 -
        // identical tilemap, identical tile data, identical M7 matrix and
        // scroll registers ($210D/$210E - not the normal BG2 scroll
        // registers) - confirmed via the Super Famicom Dev wiki's
        // Backgrounds page. The only difference is interpretation: BG2
        // reinterprets the sampled pixel byte's high bit as a priority flag
        // instead of a color bit, so it only has 128 distinct colors (0-127)
        // where BG1 has the full 256. That same page documents the resulting
        // priority order this creates - BG2's priority-1 pixels sit above
        // BG1, its priority-0 pixels sit below BG1, with sprites interleaved
        // at their usual 4 priority levels around both - which is why this
        // needs its own two-pass call (see the isMode7 branch in
        // RenderScanline, which places these two passes at the correct
        // points relative to BG1 and the sprite priority levels, rather
        // than reusing the normal-mode BG1/BG2/OBJ interleave order that
        // doesn't apply here).
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
