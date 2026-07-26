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

        // Debug-panel-only texture (VRAM tile sheet). The actual game
        // screen no longer gets its own Raylib texture/window here - that
        // moved to EmuSen.Presentation.FramePresenter, which drives every
        // core through the agnostic GetFrameBufferRgba() contract below
        // instead of reaching into this class's internals. This texture is
        // created lazily on first DrawDebugPanels() call, once
        // FramePresenter's window already exists.
        // A raw GPU handle, not emulated state - nothing to restore it TO
        // even in windowed mode (there's no "sheet texture" content that
        // isn't already re-derivable from VRAM), and StateSerializer has
        // no case for Texture2D's underlying IntPtr regardless. Surfaced
        // by the headless debug harness's own --savestate/--loadstate
        // smoke test: SaveState() threw on every headless run before this,
        // since Renderer (and this field) exist unconditionally even with
        // headless: true.
        [EmuSen.Common.SkipInState] private Texture2D _sheetTex;
        private bool _sheetTexReady;

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

        // headless=true skips window/texture creation entirely - used when a
        // non-Raylib frontend (e.g. the Avalonia EmuSen.TestingStudio project) is
        // driving frames itself and just wants the finished pixel buffer via
        // GetFrameBufferRgba(), not an on-screen Raylib window. RenderScanline
        // and the Bg/Obj compositing it calls have no Raylib dependency at all
        // (they only touch the plain Color[] buffers below), so headless mode
        // just means DrawDebugPanels() no-ops - nothing elsewhere changes.
        //
        // No window/screen-texture setup happens here anymore - the console
        // build creates its window via EmuSen.Presentation.FramePresenter
        // before driving any frames, so by the time DrawDebugPanels() lazily
        // allocates _sheetTex a window is already guaranteed to exist.
        private readonly bool _headless;

        public Renderer(bool headless = false)
        {
            _headless = headless;
        }

        public void Shutdown()
        {
            if (_headless || !_sheetTexReady) return;
            Raylib.UnloadTexture(_sheetTex);
            _sheetTexReady = false;
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

        // Draws debug-only overlays (VRAM tile sheet, CGRAM swatches, PPU
        // register text) into whichever Raylib render target is currently
        // active. Meant to run as FramePresenter.Present()'s drawOverlay
        // callback, after the game screen itself has already been drawn
        // there - see FramePresenter for why the game screen no longer
        // goes through this class at all. No-ops if debug panels are
        // switched off or this Renderer is headless (Avalonia).
        public void DrawDebugPanels(MemoryBus bus, long frame)
        {
            if (_headless || !GraphicsSettings.ShowDebugPanels) return;

            Ppu ppu = bus.Ppu;

            if (!_sheetTexReady)
            {
                Image sheet = Raylib.GenImageColor(SheetW, SheetH, new Color(0, 0, 0, 255));
                _sheetTex = Raylib.LoadTextureFromImage(sheet);
                Raylib.UnloadImage(sheet);
                _sheetTexReady = true;
            }

            RenderVramSheet(ppu);
            Raylib.UpdateTexture(_sheetTex, _sheetPixels);

            Color label = new Color(200, 200, 210, 255);

            Raylib.DrawText($"Frame {frame}   BGMODE={ppu.Bgmode:X2}  TM={ppu.Tm:X2}  TS={ppu.Ts:X2}  CGWSEL={ppu.Cgwsel:X2}  CGADSUB={ppu.Cgadsub:X2}  INIDISP={ppu.Inidisp:X2}", 16, 12, 18, label);
            Raylib.DrawText($"W12SEL={ppu.W12Sel:X2} W34SEL={ppu.W34Sel:X2} WOBJSEL={ppu.WObjSel:X2} WH0={ppu.Wh0} WH1={ppu.Wh1} WH2={ppu.Wh2} WH3={ppu.Wh3} WBGLOG={ppu.WBgLog:X2} WOBJLOG={ppu.WObjLog:X2} TMW={ppu.Tmw:X2} TSW={ppu.Tsw:X2}", 16, 590, 14, label);
            Raylib.DrawText($"FixedColor R={ppu.FixedColorR} G={ppu.FixedColorG} B={ppu.FixedColorB}   CGRAM[0]=0x{ppu.Cgram[0]:X2}{ppu.Cgram[1]:X2}", 16, 585, 16, label);
            Raylib.DrawText("BG1", 16, 40, 16, label);

            Raylib.DrawText("VRAM tiles (4bpp)", 560, 40, 16, label);
            Raylib.DrawTextureEx(_sheetTex, new Vector2(560, 60), 0f, 1f, new Color(255, 255, 255, 255));
            Raylib.DrawText("CGRAM", 850, 40, 16, label);

            for (int i = 0; i < 256; i++)
            {
                Color c = SnesColor(ppu.Cgram[i * 2], ppu.Cgram[i * 2 + 1], 1f);
                Raylib.DrawRectangle(850 + (i % 16) * 12, 60 + (i / 16) * 12, 11, 11, c);
            }
        }
    }
}