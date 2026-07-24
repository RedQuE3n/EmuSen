using System;
using System.Collections.Generic;
using System.Numerics;
using Raylib_cs;
using EmuSen.Presentation.Shaders;
using EmuSen.Graphics;

namespace EmuSen.Presentation
{
    // Built-in post-processing effects, applied as a single GLSL fragment
    // shader pass on the upscaled game screen before it's composited with
    // the debug overlay. Prototype for RetroArch-style shaders - real
    // multi-pass chaining and a loadable preset format come later; this is
    // just proving the seam works and looks right. None is always the
    // fallback (bit-identical to no shader pipeline at all).
    public enum ShaderEffect
    {
        None,
        Scanlines,
        Crt,
    }

    // Core-agnostic on-screen presentation: takes any ICore's raw RGBA8888
    // frame buffer (ICore.GetFrameBufferRgba(), ICore.ScreenWidth/Height)
    // and gets it on screen via Raylib, with no knowledge of which console
    // produced it. Lives in its own project (EmuSen.Presentation), separate
    // from both the emulation core (EmuSen.csproj) and any one frontend, so
    // every frontend depends on the same presentation/shader code instead
    // of a hand-copied duplicate - the console build (EmuSen.csproj's
    // Frontend/Program.cs) is the first consumer; Avalonia's EmulatorSession
    // already goes through the same ICore contract and can adopt this once
    // it has a GPU-capable rendering surface of its own (it currently
    // presents via a CPU-side WriteableBitmap, which has no shader hook).
    //
    // drawOverlay in Present() is the seam for core-specific debug drawing
    // (VRAM sheet, CGRAM swatches, register text) that still needs direct
    // access to a concrete core's internals - see Cores/ICore.cs's own note
    // on why that stays out of the agnostic contract. It runs while this
    // class's offscreen render target is still the active Raylib draw
    // target, so it can free-draw into the same frame this presents.
    // Deliberately runs AFTER the shader pass below, so debug text/panels
    // stay crisp instead of getting scanlined along with the game screen.
    public sealed class FramePresenter : IDisposable
    {
        private RenderTexture2D _renderTarget;
        private Texture2D _screenTex;
        private int _texWidth;
        private int _texHeight;

        // Ping-pong target the active shader renders into, sized to the
        // upscaled draw rect (not the console's native resolution) since
        // these are simple post-process effects meant to operate on the
        // final on-screen pixels, not the source frame.
        private RenderTexture2D _passTarget;
        private int _passWidth;
        private int _passHeight;
        private bool _passTargetReady;

        private readonly Dictionary<ShaderEffect, Shader> _shaders = new();
        private readonly Dictionary<ShaderEffect, int> _outputSizeLoc = new();
        public ShaderEffect Effect { get; set; } = ShaderEffect.None;

        // Cycles None -> Scanlines -> Crt -> None, for quick manual A/B
        // testing before there's any real settings UI for this.
        public ShaderEffect CycleEffect()
        {
            Effect = Effect switch
            {
                ShaderEffect.None => ShaderEffect.Scanlines,
                ShaderEffect.Scanlines => ShaderEffect.Crt,
                _ => ShaderEffect.None,
            };
            return Effect;
        }

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

        private void EnsurePassTarget(int width, int height)
        {
            if (_passTargetReady && _passWidth == width && _passHeight == height) return;

            if (_passTargetReady) Raylib.UnloadRenderTexture(_passTarget);

            _passTarget = Raylib.LoadRenderTexture(width, height);
            _passWidth = width;
            _passHeight = height;
            _passTargetReady = true;
        }

        // Lazily compiles a built-in shader on first use - avoids paying
        // for shader compilation at startup for effects nobody selects.
        // vsCode is null so Raylib falls back to its own default vertex
        // shader; see BuiltInShaders' own note on why that's sufficient.
        private Shader GetShader(ShaderEffect effect)
        {
            if (_shaders.TryGetValue(effect, out Shader existing)) return existing;

            string fsCode = effect switch
            {
                ShaderEffect.Scanlines => BuiltInShaders.ScanlinesFs,
                ShaderEffect.Crt => BuiltInShaders.CrtFs,
                _ => throw new ArgumentOutOfRangeException(nameof(effect), effect, "No shader source for this effect."),
            };

            Shader shader = Raylib.LoadShaderFromMemory(null, fsCode);
            _shaders[effect] = shader;
            _outputSizeLoc[effect] = Raylib.GetShaderLocation(shader, "outputSize");
            return shader;
        }

        // Draws the game screen into whatever render target is currently
        // active, running it through the selected shader pass if one is
        // active. width/height are the source frame's native pixel size;
        // the draw itself always targets the same upscaled on-screen rect
        // regardless of shader state, so switching effects never shifts
        // the image.
        private void DrawGameScreen(int width, int height)
        {
            Rectangle destRec = new Rectangle(16, 60, width * 2, height * 2);

            if (Effect == ShaderEffect.None)
            {
                Rectangle sourceRec = new Rectangle(0, 0, width, height);
                Raylib.DrawTexturePro(_screenTex, sourceRec, destRec, Vector2.Zero, 0f, new Color(255, 255, 255, 255));
                return;
            }

            int passWidth = width * 2;
            int passHeight = height * 2;
            EnsurePassTarget(passWidth, passHeight);

            Shader shader = GetShader(Effect);
            int outputSizeLoc = _outputSizeLoc[Effect];

            Raylib.BeginTextureMode(_passTarget);
            Raylib.ClearBackground(new Color(0, 0, 0, 255));
            Raylib.BeginShaderMode(shader);
            if (outputSizeLoc >= 0)
            {
                Raylib.SetShaderValue(shader, outputSizeLoc, new Vector2(passWidth, passHeight), ShaderUniformDataType.Vec2);
            }
            Raylib.DrawTexturePro(_screenTex, new Rectangle(0, 0, width, height), new Rectangle(0, 0, passWidth, passHeight), Vector2.Zero, 0f, new Color(255, 255, 255, 255));
            Raylib.EndShaderMode();
            Raylib.EndTextureMode();

            // Render textures are stored bottom-up in OpenGL - negative
            // source height un-flips it, same trick already used below
            // for _renderTarget's own final blit to the window.
            Rectangle passSourceRec = new Rectangle(0, 0, passWidth, -passHeight);
            Raylib.DrawTexturePro(_passTarget.Texture, passSourceRec, destRec, Vector2.Zero, 0f, new Color(255, 255, 255, 255));
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

            DrawGameScreen(width, height);

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
            foreach (Shader shader in _shaders.Values)
            {
                Raylib.UnloadShader(shader);
            }
            _shaders.Clear();

            if (_passTargetReady) Raylib.UnloadRenderTexture(_passTarget);
            Raylib.UnloadRenderTexture(_renderTarget);
            if (_texWidth != 0) Raylib.UnloadTexture(_screenTex);
            Raylib.CloseWindow();
        }

        public void Dispose() => Shutdown();
    }
}
