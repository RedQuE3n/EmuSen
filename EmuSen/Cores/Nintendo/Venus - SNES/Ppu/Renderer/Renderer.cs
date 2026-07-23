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

        private RenderTexture2D _renderTarget;
        private Texture2D _screenTex;
        private Texture2D _sheetTex;
        
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
        // non-Raylib frontend (e.g. the Avalonia EmuSen.Frontend project) is
        // driving frames itself and just wants the finished pixel buffer via
        // GetFrameBufferRgba(), not an on-screen Raylib window. RenderScanline
        // and the Bg/Obj compositing it calls have no Raylib dependency at all
        // (they only touch the plain Color[] buffers below), so headless mode
        // is exactly this constructor doing less - nothing elsewhere changes.
        private readonly bool _headless;

        public Renderer(bool headless = false)
        {
            _headless = headless;
            if (headless) return;

            ConfigFlags flags = 0;
            if (GraphicsSettings.WindowResizable) flags |= ConfigFlags.ResizableWindow;
            if (GraphicsSettings.VSyncEnabled) flags |= ConfigFlags.VSyncHint;
            Raylib.SetConfigFlags(flags);

            Raylib.InitWindow(GraphicsSettings.WindowWidth, GraphicsSettings.WindowHeight, GraphicsSettings.WindowTitle);
            Raylib.SetTargetFPS(GraphicsSettings.TargetFps);

            _renderTarget = Raylib.LoadRenderTexture(GraphicsSettings.WindowWidth, GraphicsSettings.WindowHeight);
            Raylib.SetTextureFilter(_renderTarget.Texture, GraphicsSettings.BilinearFiltering ? TextureFilter.Bilinear : TextureFilter.Point);

            Image img = Raylib.GenImageColor(MaxOutputW, ScreenH, new Color(0, 0, 0, 255));
            _screenTex = Raylib.LoadTextureFromImage(img);
            Raylib.UnloadImage(img);

            Image sheet = Raylib.GenImageColor(SheetW, SheetH, new Color(0, 0, 0, 255));
            _sheetTex = Raylib.LoadTextureFromImage(sheet);
            Raylib.UnloadImage(sheet);
        }

        public bool IsOpen() => _headless || !Raylib.WindowShouldClose();

        public void Shutdown()
        {
            if (_headless) return;
            Raylib.UnloadRenderTexture(_renderTarget);
            Raylib.UnloadTexture(_screenTex);
            Raylib.UnloadTexture(_sheetTex);
            Raylib.CloseWindow();
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

        public void DrawFrame(MemoryBus bus, long frame)
        {
            if (_headless) return;

            Ppu ppu = bus.Ppu;

            if (Raylib.IsKeyPressed(KeyboardKey.O))
            {
                Console.WriteLine($"[OAM DUMP] --- Frame {frame} ---");
                DumpActiveOam(ppu);
                DumpBlackBg1Tiles(ppu);

                // Log the exact backdrop compositing math, since we now suspect the
                // "black squares" are actually transparent BG1 pixels correctly
                // revealing a WRONG backdrop, not bad tile/palette data (both of
                // which just checked out fine).
                float brightness = (ppu.Inidisp & 0x0F) / 15f;
                Color mainBackdrop = SnesColor(ppu.Cgram[0], ppu.Cgram[1], brightness);
                Color subBackdrop = new Color(
                    (byte)(((ppu.FixedColorR & 0x1F) << 3) * brightness),
                    (byte)(((ppu.FixedColorG & 0x1F) << 3) * brightness),
                    (byte)(((ppu.FixedColorB & 0x1F) << 3) * brightness),
                    (byte)255
                );
                bool subtractMode = (ppu.Cgadsub & 0x80) != 0;
                bool halfMode = (ppu.Cgadsub & 0x40) != 0;
                bool backdropMathEnabled = (ppu.Cgadsub & 0x20) != 0;
                Color blended = backdropMathEnabled ? BlendColors(mainBackdrop, subBackdrop, subtractMode, halfMode) : mainBackdrop;

                Console.WriteLine($"[BACKDROP] CGRAM[0]=0x{ppu.Cgram[0]:X2}{ppu.Cgram[1]:X2} (ever written: {ppu.WasCgramTouched(0) || ppu.WasCgramTouched(1)}) -> mainBackdrop=({mainBackdrop.R},{mainBackdrop.G},{mainBackdrop.B})");
                Console.WriteLine($"[BACKDROP] FixedColor R={ppu.FixedColorR} G={ppu.FixedColorG} B={ppu.FixedColorB} (2132 ever written: {ppu.FixedColorEverWritten}) -> subBackdrop=({subBackdrop.R},{subBackdrop.G},{subBackdrop.B})");
                Console.WriteLine($"[BACKDROP] CGADSUB=0x{ppu.Cgadsub:X2} backdropMathEnabled={backdropMathEnabled} subtract={subtractMode} half={halfMode} -> FINAL BACKDROP=({blended.R},{blended.G},{blended.B})");

                Console.WriteLine($"[WINDOW] W12SEL=0x{ppu.W12Sel:X2} W34SEL=0x{ppu.W34Sel:X2} WOBJSEL=0x{ppu.WObjSel:X2}");
                Console.WriteLine($"[WINDOW] WH0(w1left)={ppu.Wh0} WH1(w1right)={ppu.Wh1} WH2(w2left)={ppu.Wh2} WH3(w2right)={ppu.Wh3}");
                Console.WriteLine($"[WINDOW] WBGLOG=0x{ppu.WBgLog:X2} WOBJLOG=0x{ppu.WObjLog:X2} TMW=0x{ppu.Tmw:X2} TSW=0x{ppu.Tsw:X2}");
            }

            if (GraphicsSettings.ShowDebugPanels)
            {
                RenderVramSheet(ppu);
            }

            Raylib.UpdateTexture(_screenTex, _screenPixels);
            Raylib.UpdateTexture(_sheetTex, _sheetPixels);

            Raylib.BeginTextureMode(_renderTarget);
            Raylib.ClearBackground(GraphicsSettings.PanelBackgroundColor);

            Color label = new Color(200, 200, 210, 255);

            if (GraphicsSettings.ShowDebugPanels)
            {
                Raylib.DrawText($"Frame {frame}   BGMODE={ppu.Bgmode:X2}  TM={ppu.Tm:X2}  TS={ppu.Ts:X2}  CGWSEL={ppu.Cgwsel:X2}  CGADSUB={ppu.Cgadsub:X2}  INIDISP={ppu.Inidisp:X2}", 16, 12, 18, label);
                Raylib.DrawText($"W12SEL={ppu.W12Sel:X2} W34SEL={ppu.W34Sel:X2} WOBJSEL={ppu.WObjSel:X2} WH0={ppu.Wh0} WH1={ppu.Wh1} WH2={ppu.Wh2} WH3={ppu.Wh3} WBGLOG={ppu.WBgLog:X2} WOBJLOG={ppu.WObjLog:X2} TMW={ppu.Tmw:X2} TSW={ppu.Tsw:X2}", 16, 590, 14, label);
                Raylib.DrawText($"FixedColor R={ppu.FixedColorR} G={ppu.FixedColorG} B={ppu.FixedColorB}   CGRAM[0]=0x{ppu.Cgram[0]:X2}{ppu.Cgram[1]:X2}", 16, 585, 16, label);
                Raylib.DrawText("BG1", 16, 40, 16, label);
            }
            // Source rect limited to the actual current frame width. Known
            // cosmetic-only limitation: a hi-res 512-wide frame still draws
            // at the same fixed 2x scale here and visually runs into the
            // VRAM sheet panel - debug-console view only; the Avalonia
            // frontend sizes its bitmap to FrameWidth directly instead.
            Rectangle screenSourceRec = new Rectangle(0, 0, _frameWidth, ScreenH);
            Raylib.DrawTexturePro(_screenTex, screenSourceRec, new Rectangle(16, 60, _frameWidth * 2, ScreenH * 2), new Vector2(0, 0), 0f, new Color(255, 255, 255, 255));

            if (GraphicsSettings.ShowDebugPanels)
            {
                Raylib.DrawText("VRAM tiles (4bpp)", 560, 40, 16, label);
                Raylib.DrawTextureEx(_sheetTex, new Vector2(560, 60), 0f, 1f, new Color(255, 255, 255, 255));
                Raylib.DrawText("CGRAM", 850, 40, 16, label);

                for (int i = 0; i < 256; i++)
                {
                    Color c = SnesColor(ppu.Cgram[i * 2], ppu.Cgram[i * 2 + 1], 1f);
                    Raylib.DrawRectangle(850 + (i % 16) * 12, 60 + (i / 16) * 12, 11, 11, c);
                }
            }
            Raylib.EndTextureMode();

            Raylib.BeginDrawing();
            Raylib.ClearBackground(GraphicsSettings.LetterboxColor);

            float scale = Math.Min((float)Raylib.GetScreenWidth() / GraphicsSettings.WindowWidth, (float)Raylib.GetScreenHeight() / GraphicsSettings.WindowHeight);
            Rectangle sourceRec = new Rectangle(0, 0, (float)_renderTarget.Texture.Width, -(float)_renderTarget.Texture.Height);
            Rectangle destRec = new Rectangle(
                (Raylib.GetScreenWidth() - (GraphicsSettings.WindowWidth * scale)) * 0.5f,
                (Raylib.GetScreenHeight() - (GraphicsSettings.WindowHeight * scale)) * 0.5f,
                GraphicsSettings.WindowWidth * scale, GraphicsSettings.WindowHeight * scale
            );

            Raylib.DrawTexturePro(_renderTarget.Texture, sourceRec, destRec, new Vector2(0, 0), 0.0f, new Color(255, 255, 255, 255));
            Raylib.EndDrawing();
        }
    }
}