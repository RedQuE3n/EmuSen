using System;
using System.Numerics;
using Raylib_cs;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.Graphics;

namespace EmuSen.Cores.Nintendo.Venus.Video
{
    public partial class Renderer
    {
        private static readonly (int w, int h)[] ObjSmallSize = { (8, 8), (8, 8), (8, 8), (16, 16), (16, 16), (32, 32), (16, 32), (16, 32) };
        private static readonly (int w, int h)[] ObjLargeSize = { (16, 16), (32, 32), (64, 64), (32, 32), (64, 64), (64, 64), (32, 64), (32, 32) };

        private readonly Color[] _objColor = new Color[ScreenW];
        private readonly int[] _objPriority = new int[ScreenW];
        private readonly bool[] _objSet = new bool[ScreenW];

        private readonly int[] _objPalette = new int[ScreenW];
        private readonly int[] _scanlineSprites = new int[32];

        private void EvaluateSpritesForScanline(Ppu ppu, int py, float brightness)
        {
            Array.Clear(_objSet, 0, _objSet.Length);

            int objBase = (ppu.Obsel & 0x07) << 14;
            int sizeSelect = (ppu.Obsel >> 5) & 0x07;
            (int w, int h) small = ObjSmallSize[sizeSelect];
            (int w, int h) large = ObjLargeSize[sizeSelect];

            // --- Step 1: collect up to 32 on-screen sprites, in index order ---
            int scanlineSpriteCount = 0;
            int first = ppu.FirstSpriteIndex;
            for (int n = 0; n < 128 && scanlineSpriteCount < 32; n++)
            {
                int i = (first + n) & 0x7F;
                int oamIdx = i * 4;
                int y = ppu.Oam[oamIdx + 1];

                int highTableIdx = 512 + (i / 4);
                int highBits = (ppu.Oam[highTableIdx] >> ((i % 4) * 2)) & 0x03;
                int spriteH = (highBits & 0x02) != 0 ? large.h : small.h;

                int spriteRow = (py - y) & 0xFF;
                if (spriteRow >= spriteH) continue;

                _scanlineSprites[scanlineSpriteCount++] = i;
            }
            if (scanlineSpriteCount >= 32) ppu.RangeOver = true;

            int sliversUsed = 0;
            bool budgetExhausted = false;

            for (int k = scanlineSpriteCount - 1; k >= 0 && !budgetExhausted; k--)
            {
                int i = _scanlineSprites[k];
                int oamIdx = i * 4;
                int x = ppu.Oam[oamIdx];
                int y = ppu.Oam[oamIdx + 1];
                int tileLow = ppu.Oam[oamIdx + 2];
                int attr = ppu.Oam[oamIdx + 3];
                int spritePriority = (attr >> 4) & 0x03;

                int highTableIdx = 512 + (i / 4);
                int highBits = (ppu.Oam[highTableIdx] >> ((i % 4) * 2)) & 0x03;
                if ((highBits & 0x01) != 0) x -= 256;

                int spriteW = (highBits & 0x02) != 0 ? large.w : small.w;
                int spriteH = (highBits & 0x02) != 0 ? large.h : small.h;

                int spriteRow = (py - y) & 0xFF;

                bool flipX = (attr & 0x40) != 0;
                bool flipY = (attr & 0x80) != 0;
                int pal = 8 + ((attr & 0x0E) >> 1);

                bool useSecondTable = (attr & 0x01) != 0;
                int tileTableBase = objBase;
                if (useSecondTable)
                {
                    int nameOffset = ((ppu.Obsel >> 3) & 0x03) + 1;
                    tileTableBase = (objBase + (nameOffset << 13)) & 0xFFFF;
                }

                int fetchedRow = flipY ? (spriteH - 1 - spriteRow) : spriteRow;
            
                int subRow = (((tileLow >> 4) & 0x0F) + (fetchedRow / 8)) & 0x0F;
                int r = fetchedRow % 8;

                int widthTiles = spriteW / 8;

                // Slivers consumed left-to-right on screen, matching the
                // documented culling order - not the VRAM/flip order.
                for (int screenTile = 0; screenTile < widthTiles; screenTile++)
                {
                    if (sliversUsed >= 34)
                    {
                        budgetExhausted = true;
                        break;
                    }
                    sliversUsed++;

                    int vramTx = flipX ? (widthTiles - 1 - screenTile) : screenTile;
                    int subCol = ((tileLow & 0x0F) + vramTx) & 0x0F;
                    int tileAddr = (tileTableBase + ((subRow << 4) | subCol) * 32) & 0xFFFF;

                    byte p0 = ppu.Vram[(tileAddr + r * 2) & 0xFFFF];
                    byte p1 = ppu.Vram[(tileAddr + r * 2 + 1) & 0xFFFF];
                    byte p2 = ppu.Vram[(tileAddr + 16 + r * 2) & 0xFFFF];
                    byte p3 = ppu.Vram[(tileAddr + 16 + r * 2 + 1) & 0xFFFF];

                    for (int c = 0; c < 8; c++)
                    {
                        int screenX = x + screenTile * 8 + (flipX ? 7 - c : c);
                        if (screenX < 0 || screenX >= ScreenW) continue;

                        int cbit = 7 - c;
                        int pixel = ((p0 >> cbit) & 1) | (((p1 >> cbit) & 1) << 1) | (((p2 >> cbit) & 1) << 2) | (((p3 >> cbit) & 1) << 3);
                        if (pixel == 0) continue;

                        int cgIdx = (pal * 16 + pixel) * 2;
                        _objColor[screenX] = SnesColor(ppu.Cgram[cgIdx & 0x1FF], ppu.Cgram[(cgIdx + 1) & 0x1FF], brightness);
                        _objPriority[screenX] = spritePriority;
                        _objPalette[screenX] = pal - 8; // OBJ-relative 0-7, for color math's palette-4-7 rule
                        _objSet[screenX] = true;
                    }
                }
            }

            if (sliversUsed > 34) ppu.TimeOver = true;
        }

        private void RenderObj(Ppu ppu, int py, float brightness, Color[] target, int[] targetLayer, int layerId, int priorityFilter, bool isMainScreen)
        {
            for (int x = 0; x < ScreenW; x++)
            {
                if (!_objSet[x] || _objPriority[x] != priorityFilter) continue;
                if (IsWindowMasked(ppu, layerId, isMainScreen, x)) continue;

                target[x] = _objColor[x];
                targetLayer[x] = layerId;
            }
        }
    }
}
