using System;
using Raylib_cs;
using EmuSen.Graphics;

namespace EmuSen.Cores.Nintendo.Venus.Video
{
    public partial class Renderer
    {
        private const int ScreenW = 256;
        private const int ScreenH = 224;
        private const int SheetW = 256;  
        private const int SheetH = 512;  

        // Pseudo-hi-res column interleave (Phase A) vs. true Mode 5/6
        // hi-res (Phase B, not yet done) - see Venus_PPU.md §8.
        private const int MaxOutputW = 512;
        private int _frameWidth = ScreenW;

        // Current output frame's actual pixel width - see Venus_PPU.md §8.
        public int FrameWidth => _frameWidth;

        // Layer IDs used to track which layer "won" each pixel while building a
        // scanline, so the final color-math blend step (which is per-layer, not
        // global) knows whether that specific pixel is eligible for blending.
        private const int LayerBackdrop = 0;
        private const int LayerBg1 = 1;
        private const int LayerBg2 = 2;
        private const int LayerBg3 = 3;
        private const int LayerBg4 = 4;
        private const int LayerObj = 5;

        // Allocated at max width so a hi-res toggle never needs
        // reallocation - see Venus_PPU.md §8.
        private Color[] _screenPixels = new Color[MaxOutputW * ScreenH];
        private Color[] _sheetPixels = new Color[SheetW * SheetH];

        // Per-scanline working buffers for the main and sub screens. Real hardware
        // computes both independently, then blends them per CGADSUB - these hold one
        // scanline's worth at a time rather than a full frame, since they're
        // discarded once RenderScanline finishes blending into _screenPixels.
        private Color[] _mainLineBuf = new Color[ScreenW];
        private Color[] _subLineBuf = new Color[ScreenW];
        private int[] _mainLineLayer = new int[ScreenW];
        private int[] _subLineLayer = new int[ScreenW];

        // headless=true is a no-op flag today - kept for call-site/save-
        // state-format stability (every VenusCore(headless:) call site and
        // EmulatorSession still pass one) after the on-window Raylib debug
        // overlay (VRAM tile sheet, CGRAM swatch, PPU register text - see
        // git history for DrawDebugPanels/Shutdown) was removed entirely
        // rather than ported: DianaOS's regs/sprites/pal/tile/vramsheet/
        // paletteswatch commands and the coretop dashboard already cover
        // the exact same data, and EmuSen.Mistress (always headless: true)
        // never had this overlay to begin with, proving it wasn't load-
        // bearing. RenderScanline and the Bg/Obj compositing it calls have
        // no rendering-API dependency at all (they only touch the plain
        // Color[] buffers below), so this flag no longer gates anything.
        private readonly bool _headless;

        public Renderer(bool headless = false)
        {
            _headless = headless;
        }

        // Plain RGBA8888 copy of the finished frame - the boundary non-
        // Raylib frontends should use instead of reaching for
        // _screenPixels/Color directly. Callers MUST check FrameWidth
        // rather than assuming a fixed size - see Venus_PPU.md §8.
        public byte[] GetFrameBufferRgba()
        {
            byte[] buffer = new byte[_frameWidth * ScreenH * 4];
            int o = 0;
            for (int py = 0; py < ScreenH; py++)
            {
                int rowStart = py * MaxOutputW;
                for (int px = 0; px < _frameWidth; px++)
                {
                    Color c = _screenPixels[rowStart + px];
                    buffer[o] = c.R;
                    buffer[o + 1] = c.G;
                    buffer[o + 2] = c.B;
                    buffer[o + 3] = c.A;
                    o += 4;
                }
            }
            return buffer;
        }

        // Headless-safe VRAM tile-sheet export - same 4bpp grayscale decode
        // DrawDebugPanels already uses for its on-screen texture, just
        // returned as a plain RGBA buffer instead of pushed into a Raylib
        // Texture2D. RenderVramSheet itself makes no GPU calls (it only
        // writes _sheetPixels), so calling it here works fine even with no
        // window at all - see the headless-mode comment on _headless above.
        public (byte[] Rgba, int Width, int Height) GetVramTileSheetRgba(Ppu ppu)
        {
            RenderVramSheet(ppu);
            byte[] buffer = new byte[SheetW * SheetH * 4];
            int o = 0;
            for (int i = 0; i < _sheetPixels.Length; i++)
            {
                Color c = _sheetPixels[i];
                buffer[o] = c.R;
                buffer[o + 1] = c.G;
                buffer[o + 2] = c.B;
                buffer[o + 3] = c.A;
                o += 4;
            }
            return (buffer, SheetW, SheetH);
        }

        // Headless-safe CGRAM palette export - same 256-color, 16-column
        // grid and SnesColor conversion DrawDebugPanels uses for its swatch
        // panel, just written straight into an RGBA byte array instead of
        // Raylib.DrawRectangle calls (which need an active render target).
        public (byte[] Rgba, int Width, int Height) GetPaletteSwatchRgba(Ppu ppu, int swatchSize = 12)
        {
            const int cols = 16;
            const int rows = 16;
            int width = cols * swatchSize;
            int height = rows * swatchSize;
            byte[] buffer = new byte[width * height * 4];

            for (int i = 0; i < 256; i++)
            {
                Color c = SnesColor(ppu.Cgram[i * 2], ppu.Cgram[i * 2 + 1], 1f);
                int originX = (i % cols) * swatchSize;
                int originY = (i / cols) * swatchSize;
                for (int y = 0; y < swatchSize; y++)
                {
                    int rowStart = ((originY + y) * width + originX) * 4;
                    for (int x = 0; x < swatchSize; x++)
                    {
                        int o = rowStart + x * 4;
                        buffer[o] = c.R;
                        buffer[o + 1] = c.G;
                        buffer[o + 2] = c.B;
                        buffer[o + 3] = c.A;
                    }
                }
            }
            return (buffer, width, height);
        }

        private readonly Color[] _paletteColor = new Color[256];
        private float _paletteBrightness = float.NaN;

        // Rebuilt on a CGRAM write, a brightness change, or the top of a frame - see Venus_PPU.md §7.2.
        private void EnsurePaletteColors(Ppu ppu, int py, float brightness)
        {
            if (py != 0 && !ppu.CgramChanged && _paletteBrightness == brightness) return;

            for (int i = 0; i < 256; i++)
            {
                _paletteColor[i] = SnesColor(ppu.Cgram[i * 2], ppu.Cgram[i * 2 + 1], brightness);
            }
            _paletteBrightness = brightness;
            ppu.CgramChanged = false;
        }

        // cgIdx is always an even byte offset, so this is the entry it names.
        private Color PaletteColor(int cgIdx) => _paletteColor[(cgIdx >> 1) & 0xFF];

        private static Color SnesColor(byte lo, byte hi, float brightness)
        {
            int c = lo | (hi << 8);
            int r = (c & 0x1F) << 3;
            int g = ((c >> 5) & 0x1F) << 3;
            int b = ((c >> 10) & 0x1F) << 3;
            return new Color((byte)(r * brightness), (byte)(g * brightness), (byte)(b * brightness), (byte)255);
        }

        private static int BgTilemapEntryAddress(int mapBase, int sizeBits, int tx, int ty)
        {
            bool wide = (sizeBits & 0x01) != 0;
            bool tall = (sizeBits & 0x02) != 0;

            int screenOffset = 0;
            if (wide && (tx & 0x20) != 0) screenOffset += 0x800;
            if (tall && (ty & 0x20) != 0) screenOffset += wide ? 0x1000 : 0x800;

            return (mapBase + screenOffset + (((ty & 0x1F) * 32 + (tx & 0x1F)) * 2)) & 0xFFFF;
        }

        // Checks CGADSUB's per-layer color math enable bits.
        private static bool LayerParticipatesInColorMath(byte cgadsub, int layer, int objPalette = 0)
        {
            switch (layer)
            {
                case LayerBg1: return (cgadsub & 0x01) != 0;
                case LayerBg2: return (cgadsub & 0x02) != 0;
                case LayerBg3: return (cgadsub & 0x04) != 0;
                case LayerBg4: return (cgadsub & 0x08) != 0;
                // OBJ color math restriction - see Venus_PPU.md §5.
                case LayerObj: return (cgadsub & 0x10) != 0 && objPalette >= 4;
                case LayerBackdrop: return (cgadsub & 0x20) != 0;
                default: return false;
            }
        }

        // Combines a main-screen pixel with the corresponding sub-screen pixel per
        // CGADSUB's add/subtract and half-color bits.
        private Color BlendColors(Color main, Color sub, bool subtract, bool half)
        {
            if (subtract)
            {
                // Subtract RGB channels, clamp to 0
                int r = Math.Max(0, main.R - sub.R);
                int g = Math.Max(0, main.G - sub.G);
                int b = Math.Max(0, main.B - sub.B);
                
                if (half)
                {
                    r /= 2; g /= 2; b /= 2;
                }

                // CRITICAL: alpha must stay 255 here - see Venus_PPU.md §5.
                return new Color((byte)r, (byte)g, (byte)b, (byte)255);
            }
            else
            {
                int r = Math.Min(255, main.R + sub.R);
                int g = Math.Min(255, main.G + sub.G);
                int b = Math.Min(255, main.B + sub.B);

                if (half)
                {
                    r /= 2; g /= 2; b /= 2;
                }

                return new Color((byte)r, (byte)g, (byte)b, (byte)255);
            }
        }

    }
}