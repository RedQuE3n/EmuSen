using System;
using System.Numerics;
using Raylib_cs;
using EmuSen.Graphics;

namespace EmuSen.Frontend
{
    // Core-agnostic on-screen presentation: takes any ICore's raw RGBA8888
    // frame buffer (ICore.GetFrameBufferRgba(), ICore.ScreenWidth/Height)
    // and gets it on screen via Raylib, with no knowledge of which console
    // produced it. Window/render-target ownership lives here now, not on
    // Renderer, specifically so a future post-processing shader pass chain
    // has one place to hook in - between the texture upload and the final
    // blit below - that works for every core, not just Venus/SNES.
    //
    // drawOverlay in Present() is the seam for core-specific debug drawing
    // (VRAM sheet, CGRAM swatches, register text) that still needs direct
    // access to a concrete core's internals - see ICore.cs's own note on
    // why that stays out of the agnostic contract. It runs while this
    // class's offscreen render target is still the active Raylib draw
    // target, so it can free-draw into the same frame this presents.
    public sealed class FramePresenter : IDisposable
    {
        private RenderTexture2D _renderTarget;
        private Texture2D _screenTex;
        private int _texWidth;
        private int _texHeight;

        public FramePresenter()
        {
            ConfigFlags flags = 0;
            if (GraphicsSettings.WindowResizable) flags |= ConfigFlags.ResizableWindow;
            if (GraphicsSettings.VSyncEnabled) flags |= ConfigFlags.VSyncHint;
            Raylib.SetConfigFlags(flags);

            Raylib.InitWindow(GraphicsSettings.WindowWidth, GraphicsSettings.WindowHeight, GraphicsSettings.WindowTitle);
            Raylib.SetTargetFPS(GraphicsSettings.TargetFps);

            _renderTarget = Raylib.LoadRenderTexture(GraphicsSettings.WindowWidth, GraphicsSettings.WindowHeight);
            Raylib.SetTextureFilter(_renderTarget.Texture, GraphicsSettings.BilinearFiltering ? TextureFilter.Bilinear : TextureFilter.Point);
        }

        public bool IsOpen() => !Raylib.WindowShouldClose();

        private void EnsureScreenTexture(int width, int height)
        {
            if (_texWidth == width && _texHeight == height) return;

            if (_texWidth != 0) Raylib.UnloadTexture(_screenTex);

            Image img = Raylib.GenImageColor(width, height, new Color(0, 0, 0, 255));
            _screenTex = Raylib.LoadTextureFromImage(img);
            Raylib.UnloadImage(img);
            _texWidth = width;
            _texHeight = height;
        }

        // Uploads one core-agnostic RGBA8888 frame and blits it to the
        // window, letterboxed to preserve aspect ratio. drawOverlay, if
        // given, runs after the game screen is drawn but before the
        // offscreen render target is blitted to the window.
        public void Present(byte[] rgba, int width, int height, Action? drawOverlay = null)
        {
            EnsureScreenTexture(width, height);
            Raylib.UpdateTexture(_screenTex, rgba);

            Raylib.BeginTextureMode(_renderTarget);
            Raylib.ClearBackground(GraphicsSettings.PanelBackgroundColor);

            Rectangle screenSourceRec = new Rectangle(0, 0, width, height);
            Raylib.DrawTexturePro(_screenTex, screenSourceRec, new Rectangle(16, 60, width * 2, height * 2), new Vector2(0, 0), 0f, new Color(255, 255, 255, 255));

            drawOverlay?.Invoke();

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

        public void Shutdown()
        {
            Raylib.UnloadRenderTexture(_renderTarget);
            if (_texWidth != 0) Raylib.UnloadTexture(_screenTex);
            Raylib.CloseWindow();
        }

        public void Dispose() => Shutdown();
    }
}
