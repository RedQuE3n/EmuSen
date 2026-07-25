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
        // Each BG layer's render method gets called twice per scanline per
        // screen in every mode that splits it by tile priority (once with
        // priorityOnly=false, once with =true - see RenderScanline's mode
        // dispatch tables) - and both calls used to redo the ENTIRE
        // per-pixel decode (tilemap lookup, VRAM reads, CGRAM color
        // conversion) for all 256 pixels, discarding exactly half the work
        // each time via the priority filter. That's real, measured cost:
        // profiling during actual gameplay (not a title/intro screen, which
        // has little on-screen to render) showed PPU rendering jumping
        // from under 1ms/frame to ~13-14ms/frame, right at the 60fps
        // budget.
        //
        // Since PPU register/VRAM/CGRAM state is guaranteed static for the
        // whole duration of one RenderScanline() call (no CPU execution
        // happens between these calls - see VenusCore.RunFrame()), the
        // decode result for a given (layer, isMainScreen, py) is identical
        // between the two priorityOnly calls. This cache computes it once,
        // on whichever call reaches a given scanline first, and both calls
        // just filter the cached per-pixel result by priority - same
        // compositing order, same overwrite semantics, same window
        // masking/mosaic/hi-res-pairing/direct-color logic, just not
        // recomputed twice. Kept per (layer, isMainScreen) rather than one
        // shared cache since window-enable (Tmw/Tsw) and hi-res tile
        // pairing both depend on isMainScreen, so main and sub decode to
        // genuinely different results for the same layer/scanline.
        private sealed class BgLineCache
        {
            public int Py = -1;
            public readonly bool[] Opaque = new bool[ScreenW];
            public readonly bool[] HighPriority = new bool[ScreenW];
            public readonly bool[] WindowMasked = new bool[ScreenW];
            public readonly Color[] PixelColor = new Color[ScreenW];
        }

        private readonly BgLineCache _bg1MainCache = new();
        private readonly BgLineCache _bg1SubCache = new();
        private readonly BgLineCache _bg2MainCache = new();
        private readonly BgLineCache _bg2SubCache = new();
        private readonly BgLineCache _bg3MainCache = new();
        private readonly BgLineCache _bg3SubCache = new();
        private readonly BgLineCache _bg4MainCache = new();
        private readonly BgLineCache _bg4SubCache = new();

        private static int ResolveBgTileIndex(int baseTile, bool tile16, int wx, int wy, bool flipX, bool flipY)
        {
            if (!tile16) return baseTile;
            int subX = (wx >> 3) & 1;
            int subY = (wy >> 3) & 1;
            if (flipX) subX = 1 - subX;
            if (flipY) subY = 1 - subY;
            return baseTile + subX + subY * 16;
        }
        private static int ResolveHiResPairedTile(int tileIndex, int mode, bool tile16, bool flipX, bool isMainScreen)
        {
            if ((mode != 5 && mode != 6) || tile16) return tileIndex;
            int offset = (isMainScreen != flipX) ? 0 : 1;
            return tileIndex + offset;
        }

        private static int GetBgBpp(int mode, int bgIndex)
        {
            switch (mode)
            {
                case 0: return 2;                          // BG1-4 all 2bpp
                case 1: return bgIndex == 2 ? 2 : 4;        // BG1/2 4bpp, BG3 2bpp
                case 2: return 4;                           // BG1/2 4bpp (BG3 = offset table, not rendered)
                case 3: return bgIndex == 0 ? 8 : 4;        // BG1 8bpp, BG2 4bpp
                case 4: return bgIndex == 0 ? 8 : 2;        // BG1 8bpp, BG2 2bpp (BG3 = offset table)
                case 5: return bgIndex == 0 ? 4 : 2;        // BG1 4bpp, BG2 2bpp (hi-res)
                case 6: return 4;                           // BG1 only, 4bpp (hi-res, offset-per-tile)
                default: return 4;
            }
        }

        private static int SampleBgPixel(byte[] vram, int tileAddr, int r, int cbit, int bpp)
        {
            byte p0 = vram[(tileAddr + r * 2) & 0xFFFF];
            byte p1 = vram[(tileAddr + r * 2 + 1) & 0xFFFF];
            int pixel = ((p0 >> cbit) & 1) | (((p1 >> cbit) & 1) << 1);
            if (bpp >= 4)
            {
                byte p2 = vram[(tileAddr + 16 + r * 2) & 0xFFFF];
                byte p3 = vram[(tileAddr + 16 + r * 2 + 1) & 0xFFFF];
                pixel |= ((p2 >> cbit) & 1) << 2 | ((p3 >> cbit) & 1) << 3;
            }
            if (bpp == 8)
            {
                byte p4 = vram[(tileAddr + 32 + r * 2) & 0xFFFF];
                byte p5 = vram[(tileAddr + 32 + r * 2 + 1) & 0xFFFF];
                byte p6 = vram[(tileAddr + 48 + r * 2) & 0xFFFF];
                byte p7 = vram[(tileAddr + 48 + r * 2 + 1) & 0xFFFF];
                pixel |= ((p4 >> cbit) & 1) << 4 | ((p5 >> cbit) & 1) << 5 | ((p6 >> cbit) & 1) << 6 | ((p7 >> cbit) & 1) << 7;
            }
            return pixel;
        }

        private static int BgCgramIndex(int entryPalette, int pixel, int bpp)
        {
            if (bpp == 8) return pixel * 2;
            int colorsPerPalette = bpp == 2 ? 4 : 16;
            return (entryPalette * colorsPerPalette + pixel) * 2;
        }

        private static Color DirectColor(int pixel, int entryPalette, float brightness)
        {
            int r5 = ((pixel & 0x07) << 2) | ((entryPalette & 0x01) << 1);
            int g5 = (((pixel >> 3) & 0x07) << 2) | (((entryPalette >> 1) & 0x01) << 1);
            int b5 = (((pixel >> 6) & 0x03) << 3) | (((entryPalette >> 2) & 0x01) << 2);
            int raw = r5 | (g5 << 5) | (b5 << 10);
            return SnesColor((byte)(raw & 0xFF), (byte)((raw >> 8) & 0xFF), brightness);
        }

        private static (int hofs, int vofs) GetOffsetPerTileScroll(Ppu ppu, int mode, int bgIndex, int screenX, int baseHofs, int baseVofs)
        {
            if (screenX < 8) return (baseHofs, baseVofs);

            int validBit = bgIndex == 0 ? 0x2000 : 0x4000;
            int mapBase = (ppu.BgSc[2] & 0xFC) << 9;
            int sizeBits = ppu.BgSc[2] & 0x03;
            int bg3Vofs = ppu.BgScrollY[2] & ~7;
            int bg3Hofs = ppu.BgScrollX[2] & ~7;
            int tx = ((bg3Hofs + ((screenX - 8) & ~7)) & 0x1FF) >> 3;
            int tyRow0 = bg3Vofs >> 3;

            int hofs = baseHofs;
            int vofs = baseVofs;

            if (mode == 4)
            {
                int addr = BgTilemapEntryAddress(mapBase, sizeBits, tx, tyRow0);
                int entry = ppu.Vram[addr] | (ppu.Vram[(addr + 1) & 0xFFFF] << 8);
                if ((entry & validBit) != 0)
                {
                    if ((entry & 0x8000) != 0) vofs = entry & 0x3FF;
                    else hofs = (baseHofs & 0x07) | (entry & 0x3F8);
                }
                return (hofs, vofs);
            }

            int addrH = BgTilemapEntryAddress(mapBase, sizeBits, tx, tyRow0);
            int entryH = ppu.Vram[addrH] | (ppu.Vram[(addrH + 1) & 0xFFFF] << 8);
            if ((entryH & validBit) != 0) hofs = (baseHofs & 0x07) | (entryH & 0x3F8);

            int addrV = BgTilemapEntryAddress(mapBase, sizeBits, tx, tyRow0 + 1);
            int entryV = ppu.Vram[addrV] | (ppu.Vram[(addrV + 1) & 0xFFFF] << 8);
            if ((entryV & validBit) != 0) vofs = entryV & 0x3FF;

            return (hofs, vofs);
        }

        private void RenderBg1(Ppu ppu, int py, bool priorityOnly, float brightness, Color[] target, int[] targetLayer, int layerId, bool isMainScreen)
        {
            BgLineCache cache = isMainScreen ? _bg1MainCache : _bg1SubCache;

            if (cache.Py != py)
            {
                int mode = ppu.Bgmode & 0x07;
                int bpp = GetBgBpp(mode, 0);
                int tileStride = bpp * 8;
                bool directColor = bpp == 8 && (ppu.Cgwsel & 0x01) != 0;
                bool offsetPerTile = mode == 2 || mode == 4 || mode == 6;

                int mapBase = (ppu.BgSc[0] & 0xFC) << 9;
                int sizeBits = ppu.BgSc[0] & 0x03;
                int chrBase = (ppu.Bg12Nba & 0x0F) << 13;
                bool tile16 = (ppu.Bgmode & 0x10) != 0;

                bool mosaicOn = (ppu.Mosaic & 0x01) != 0;
                int mosaicSize = ((ppu.Mosaic >> 4) & 0x0F) + 1;
                int samplePy = mosaicOn ? py - ((py - ppu.MosaicStartScanline) % mosaicSize) : py;

                int mapH = ((sizeBits & 0x02) != 0 ? 512 : 256) * (tile16 ? 2 : 1);
                int mapW = ((sizeBits & 0x01) != 0 ? 512 : 256) * (tile16 ? 2 : 1);

                for (int px = 0; px < ScreenW; px++)
                {
                    int samplePx = mosaicOn ? px - (px % mosaicSize) : px;

                    int hofsBase = ppu.BgScrollX[0];
                    int vofsBase = ppu.BgScrollY[0];
                    if (offsetPerTile)
                    {
                        (hofsBase, vofsBase) = GetOffsetPerTileScroll(ppu, mode, 0, px, hofsBase, vofsBase);
                    }

                    int wx = (samplePx + hofsBase) % mapW;
                    int tx = wx >> (tile16 ? 4 : 3);
                    int wy = (samplePy + vofsBase) % mapH;
                    int ty = wy >> (tile16 ? 4 : 3);

                    int entryAddr = BgTilemapEntryAddress(mapBase, sizeBits, tx, ty);
                    int entry = ppu.Vram[entryAddr] | (ppu.Vram[(entryAddr + 1) & 0xFFFF] << 8);

                    cache.HighPriority[px] = (entry & 0x2000) != 0;

                    bool flipX = (entry & 0x4000) != 0;
                    bool flipY = (entry & 0x8000) != 0;

                    int r = flipY ? 7 - (wy & 7) : (wy & 7);
                    int cbit = flipX ? (wx & 7) : 7 - (wx & 7);

                    int tileIndex = ResolveBgTileIndex(entry & 0x3FF, tile16, wx, wy, flipX, flipY);
                    tileIndex = ResolveHiResPairedTile(tileIndex, mode, tile16, flipX, isMainScreen);
                    int tileAddr = (chrBase + tileIndex * tileStride) & 0xFFFF;
                    int pixel = SampleBgPixel(ppu.Vram, tileAddr, r, cbit, bpp);

                    cache.Opaque[px] = pixel != 0;
                    cache.WindowMasked[px] = IsWindowMasked(ppu, layerId, isMainScreen, px);

                    if (pixel != 0)
                    {
                        if (directColor)
                        {
                            cache.PixelColor[px] = DirectColor(pixel, (entry >> 10) & 0x07, brightness);
                        }
                        else
                        {
                            int cgIdx = BgCgramIndex((entry >> 10) & 0x07, pixel, bpp);
                            cache.PixelColor[px] = SnesColor(ppu.Cgram[cgIdx & 0x1FF], ppu.Cgram[(cgIdx + 1) & 0x1FF], brightness);
                        }
                    }
                }
                cache.Py = py;
            }

            for (int px = 0; px < ScreenW; px++)
            {
                if (!cache.Opaque[px] || cache.HighPriority[px] != priorityOnly || cache.WindowMasked[px]) continue;
                target[px] = cache.PixelColor[px];
                targetLayer[px] = layerId;
            }
        }

        private void RenderBg2(Ppu ppu, int py, bool priorityOnly, float brightness, Color[] target, int[] targetLayer, int layerId, bool isMainScreen)
        {
            if (py == 0 && DebugSettings.RenderReadLogging)
            {
                Console.WriteLine($"[RENDER-READ] BG2 scanline0 sees BgScrollX[1]={ppu.BgScrollX[1]} BgScrollY[1]={ppu.BgScrollY[1]}");
            }

            int mode = ppu.Bgmode & 0x07;

            if (mode == 6) return;

            BgLineCache cache = isMainScreen ? _bg2MainCache : _bg2SubCache;

            if (cache.Py != py)
            {
                int bpp = GetBgBpp(mode, 1);
                int tileStride = bpp * 8;
                bool offsetPerTile = mode == 2 || mode == 4 || mode == 6;

                int mapBase = (ppu.BgSc[1] & 0xFC) << 9;
                int sizeBits = ppu.BgSc[1] & 0x03;
                int chrBase = ((ppu.Bg12Nba >> 4) & 0x0F) << 13;
                bool tile16 = (ppu.Bgmode & 0x20) != 0;

                bool mosaicOn = (ppu.Mosaic & 0x02) != 0;
                int mosaicSize = ((ppu.Mosaic >> 4) & 0x0F) + 1;
                int samplePy = mosaicOn ? py - ((py - ppu.MosaicStartScanline) % mosaicSize) : py;

                int mapH = ((sizeBits & 0x02) != 0 ? 512 : 256) * (tile16 ? 2 : 1);
                int mapW = ((sizeBits & 0x01) != 0 ? 512 : 256) * (tile16 ? 2 : 1);

                for (int px = 0; px < ScreenW; px++)
                {
                    int samplePx = mosaicOn ? px - (px % mosaicSize) : px;

                    int hofsBase = ppu.BgScrollX[1];
                    int vofsBase = ppu.BgScrollY[1];
                    if (offsetPerTile)
                    {
                        (hofsBase, vofsBase) = GetOffsetPerTileScroll(ppu, mode, 1, px, hofsBase, vofsBase);
                    }

                    int wx = (samplePx + hofsBase) % mapW;
                    int tx = wx >> (tile16 ? 4 : 3);
                    int wy = (samplePy + vofsBase) % mapH;
                    int ty = wy >> (tile16 ? 4 : 3);

                    int entryAddr = BgTilemapEntryAddress(mapBase, sizeBits, tx, ty);
                    int entry = ppu.Vram[entryAddr] | (ppu.Vram[(entryAddr + 1) & 0xFFFF] << 8);

                    cache.HighPriority[px] = (entry & 0x2000) != 0;

                    bool flipX = (entry & 0x4000) != 0;
                    bool flipY = (entry & 0x8000) != 0;

                    int r = flipY ? 7 - (wy & 7) : (wy & 7);
                    int cbit = flipX ? (wx & 7) : 7 - (wx & 7);

                    int tileIndex = ResolveBgTileIndex(entry & 0x3FF, tile16, wx, wy, flipX, flipY);
                    tileIndex = ResolveHiResPairedTile(tileIndex, mode, tile16, flipX, isMainScreen);
                    int tileAddr = (chrBase + tileIndex * tileStride) & 0xFFFF;
                    int pixel = SampleBgPixel(ppu.Vram, tileAddr, r, cbit, bpp);

                    cache.Opaque[px] = pixel != 0;
                    cache.WindowMasked[px] = IsWindowMasked(ppu, layerId, isMainScreen, px);

                    if (pixel != 0)
                    {
                        int cgIdx = BgCgramIndex((entry >> 10) & 0x07, pixel, bpp);
                        cache.PixelColor[px] = SnesColor(ppu.Cgram[cgIdx & 0x1FF], ppu.Cgram[(cgIdx + 1) & 0x1FF], brightness);
                    }
                }
                cache.Py = py;
            }

            for (int px = 0; px < ScreenW; px++)
            {
                if (!cache.Opaque[px] || cache.HighPriority[px] != priorityOnly || cache.WindowMasked[px]) continue;
                target[px] = cache.PixelColor[px];
                targetLayer[px] = layerId;
            }
        }

        private void RenderBg3(Ppu ppu, int py, bool priorityOnly, float brightness, Color[] target, int[] targetLayer, int layerId, bool isMainScreen)
        {
            int mode = ppu.Bgmode & 0x07;

            if (mode == 2 || mode == 3 || mode == 4 || mode == 5 || mode == 6 || mode == 7) return;

            BgLineCache cache = isMainScreen ? _bg3MainCache : _bg3SubCache;

            if (cache.Py != py)
            {
                int bpp = GetBgBpp(mode, 2);
                int tileStride = bpp * 8;

                int mapBase = (ppu.BgSc[2] & 0xFC) << 9;
                int sizeBits = ppu.BgSc[2] & 0x03;
                int chrBase = (ppu.Bg34Nba & 0x0F) << 13;
                bool tile16 = (ppu.Bgmode & 0x40) != 0;

                bool mosaicOn = (ppu.Mosaic & 0x04) != 0;
                int mosaicSize = ((ppu.Mosaic >> 4) & 0x0F) + 1;
                int samplePy = mosaicOn ? py - ((py - ppu.MosaicStartScanline) % mosaicSize) : py;

                int mapH = ((sizeBits & 0x02) != 0 ? 512 : 256) * (tile16 ? 2 : 1);
                int wy = (samplePy + ppu.BgScrollY[2]) % mapH;
                int ty = wy >> (tile16 ? 4 : 3);

                int mapW = ((sizeBits & 0x01) != 0 ? 512 : 256) * (tile16 ? 2 : 1);

                for (int px = 0; px < ScreenW; px++)
                {
                    int samplePx = mosaicOn ? px - (px % mosaicSize) : px;
                    int wx = (samplePx + ppu.BgScrollX[2]) % mapW;
                    int tx = wx >> (tile16 ? 4 : 3);

                    int entryAddr = BgTilemapEntryAddress(mapBase, sizeBits, tx, ty);
                    int entry = ppu.Vram[entryAddr] | (ppu.Vram[(entryAddr + 1) & 0xFFFF] << 8);

                    cache.HighPriority[px] = (entry & 0x2000) != 0;

                    bool flipX = (entry & 0x4000) != 0;
                    bool flipY = (entry & 0x8000) != 0;

                    int r = flipY ? 7 - (wy & 7) : (wy & 7);
                    int cbit = flipX ? (wx & 7) : 7 - (wx & 7);

                    int tileIndex = ResolveBgTileIndex(entry & 0x3FF, tile16, wx, wy, flipX, flipY);
                    int tileAddr = (chrBase + tileIndex * tileStride) & 0xFFFF;
                    int pixel = SampleBgPixel(ppu.Vram, tileAddr, r, cbit, bpp);

                    cache.Opaque[px] = pixel != 0;
                    cache.WindowMasked[px] = IsWindowMasked(ppu, layerId, isMainScreen, px);

                    if (pixel != 0)
                    {
                        int cgIdx = BgCgramIndex((entry >> 10) & 0x07, pixel, bpp);
                        cache.PixelColor[px] = SnesColor(ppu.Cgram[cgIdx & 0x1FF], ppu.Cgram[(cgIdx + 1) & 0x1FF], brightness);
                    }
                }
                cache.Py = py;
            }

            for (int px = 0; px < ScreenW; px++)
            {
                if (!cache.Opaque[px] || cache.HighPriority[px] != priorityOnly || cache.WindowMasked[px]) continue;
                target[px] = cache.PixelColor[px];
                targetLayer[px] = layerId;
            }
        }

        private void RenderBg4(Ppu ppu, int py, bool priorityOnly, float brightness, Color[] target, int[] targetLayer, int layerId, bool isMainScreen)
        {

            int mode = ppu.Bgmode & 0x07;
            if (mode != 0) return;

            BgLineCache cache = isMainScreen ? _bg4MainCache : _bg4SubCache;

            if (cache.Py != py)
            {
                int bpp = GetBgBpp(mode, 3);
                int tileStride = bpp * 8;

                int mapBase = (ppu.BgSc[3] & 0xFC) << 9;
                int sizeBits = ppu.BgSc[3] & 0x03;
                int chrBase = ((ppu.Bg34Nba >> 4) & 0x0F) << 13;
                bool tile16 = (ppu.Bgmode & 0x80) != 0;

                bool mosaicOn = (ppu.Mosaic & 0x08) != 0;
                int mosaicSize = ((ppu.Mosaic >> 4) & 0x0F) + 1;
                int samplePy = mosaicOn ? py - ((py - ppu.MosaicStartScanline) % mosaicSize) : py;

                int mapH = ((sizeBits & 0x02) != 0 ? 512 : 256) * (tile16 ? 2 : 1);
                int wy = (samplePy + ppu.BgScrollY[3]) % mapH;
                int ty = wy >> (tile16 ? 4 : 3);

                int mapW = ((sizeBits & 0x01) != 0 ? 512 : 256) * (tile16 ? 2 : 1);

                for (int px = 0; px < ScreenW; px++)
                {
                    int samplePx = mosaicOn ? px - (px % mosaicSize) : px;
                    int wx = (samplePx + ppu.BgScrollX[3]) % mapW;
                    int tx = wx >> (tile16 ? 4 : 3);

                    int entryAddr = BgTilemapEntryAddress(mapBase, sizeBits, tx, ty);
                    int entry = ppu.Vram[entryAddr] | (ppu.Vram[(entryAddr + 1) & 0xFFFF] << 8);

                    cache.HighPriority[px] = (entry & 0x2000) != 0;

                    bool flipX = (entry & 0x4000) != 0;
                    bool flipY = (entry & 0x8000) != 0;

                    int r = flipY ? 7 - (wy & 7) : (wy & 7);
                    int cbit = flipX ? (wx & 7) : 7 - (wx & 7);

                    int tileIndex = ResolveBgTileIndex(entry & 0x3FF, tile16, wx, wy, flipX, flipY);
                    int tileAddr = (chrBase + tileIndex * tileStride) & 0xFFFF;
                    int pixel = SampleBgPixel(ppu.Vram, tileAddr, r, cbit, bpp);

                    cache.Opaque[px] = pixel != 0;
                    cache.WindowMasked[px] = IsWindowMasked(ppu, layerId, isMainScreen, px);

                    if (pixel != 0)
                    {
                        int cgIdx = BgCgramIndex((entry >> 10) & 0x07, pixel, bpp);
                        cache.PixelColor[px] = SnesColor(ppu.Cgram[cgIdx & 0x1FF], ppu.Cgram[(cgIdx + 1) & 0x1FF], brightness);
                    }
                }
                cache.Py = py;
            }

            for (int px = 0; px < ScreenW; px++)
            {
                if (!cache.Opaque[px] || cache.HighPriority[px] != priorityOnly || cache.WindowMasked[px]) continue;
                target[px] = cache.PixelColor[px];
                targetLayer[px] = layerId;
            }
        }

    }
}
