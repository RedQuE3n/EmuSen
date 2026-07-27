using System;
using System.Threading;
using Avalonia.Controls;
using Avalonia.Threading;
using EmuSen.Graphics;

namespace EmuSen.Serenity
{
    // Built-in post-processing effects, applied as a single Skia SkSL
    // shader pass on the upscaled game screen - see GameFrameControl and
    // Shaders/BuiltInShaders.cs. Same enum shape as the old Raylib/GLSL
    // version (see git history) - None is always the fallback (bit-
    // identical to no shader pipeline at all).
    public enum ShaderEffect
    {
        None,
        Scanlines,
        Crt,
    }

    // Core-agnostic on-screen presentation: takes any ICore's raw RGBA8888
    // frame buffer (ICore.GetFrameBufferRgba(), ICore.ScreenWidth/Height)
    // and gets it on screen via Avalonia's own Skia backend, with no
    // knowledge of which console produced it. Lives in its own project
    // (EmuSen.Serenity), separate from both the emulation core
    // (EmuSen.csproj) and any one frontend, so every frontend depends on
    // the same presentation/shader code instead of a hand-copied
    // duplicate - both EmuSen.Hotaru and EmuSen.Mistress9 use
    // GameFrameControl directly now (see git history for the Raylib/
    // WriteableBitmap predecessors); this FramePresenter bundle itself
    // isn't consumed by either yet.
    //
    // Present() may be called from any thread (in practice, a frontend's
    // own background emulation thread - see EmuSen.Hotaru's own comment
    // on why gameplay and presentation run on separate threads) - it
    // dispatches to the Avalonia UI thread itself, coalescing exactly
    // like EmuSen.Mistress9's own MainWindow.SubmitFrame/
    // PresentPendingFrame: only one Present is ever in flight, and a
    // frame that arrives while one's already dispatched just overwrites
    // the pending one rather than queuing - a slow UI thread drops
    // intermediate frames instead of backing up a queue of them.
    public sealed class FramePresenter : IDisposable
    {
        private readonly Window _window;
        private readonly GameFrameControl _control;
        private volatile bool _isOpen = true;

        private sealed class FrameData
        {
            public required byte[] Rgba;
            public required int Width;
            public required int Height;
        }

        private FrameData? _pendingFrame;
        private int _presentScheduled;

        public ShaderEffect Effect
        {
            get => _control.ActiveEffect;
            set => _control.ActiveEffect = value;
        }

        // Cycles None -> Scanlines -> Crt -> None, for quick manual A/B
        // testing before there's any real settings UI for this.
        public ShaderEffect CycleEffect()
        {
            _control.ActiveEffect = NextEffect(_control.ActiveEffect);
            return _control.ActiveEffect;
        }

        // Pure enum-cycling logic, extracted so it's unit-testable without
        // instantiating a FramePresenter (which opens a real Window).
        public static ShaderEffect NextEffect(ShaderEffect current) => current switch
        {
            ShaderEffect.None => ShaderEffect.Scanlines,
            ShaderEffect.Scanlines => ShaderEffect.Crt,
            _ => ShaderEffect.None,
        };

        public FramePresenter()
        {
            _control = new GameFrameControl();
            _window = new Window
            {
                Title = GraphicsSettings.WindowTitle,
                Width = GraphicsSettings.WindowWidth,
                Height = GraphicsSettings.WindowHeight,
                CanResize = GraphicsSettings.WindowResizable,
                Content = _control,
            };
            _window.Closed += (_, _) => _isOpen = false;
            _window.Show();
        }

        public bool IsOpen() => _isOpen;

        public void Present(byte[] rgba, int width, int height)
        {
            Interlocked.Exchange(ref _pendingFrame, new FrameData { Rgba = rgba, Width = width, Height = height });

            if (Interlocked.CompareExchange(ref _presentScheduled, 1, 0) == 0)
            {
                Dispatcher.UIThread.Post(PresentPendingFrame);
            }
        }

        // Runs on the UI thread. Always presents whatever the newest
        // frame is at the moment it actually runs, which may not be the
        // same frame that triggered this dispatch if the caller has since
        // produced newer ones - that's the intended drop-stale-frames
        // behavior, not a bug.
        private void PresentPendingFrame()
        {
            try
            {
                FrameData? frame = Interlocked.Exchange(ref _pendingFrame, null);
                if (frame == null || !_isOpen) return;

                _control.UpdateFrame(frame.Rgba, frame.Width, frame.Height);
            }
            finally
            {
                // Reset only after the frame above is actually handed to
                // the control, so while a Present is actively running,
                // the caller keeps overwriting _pendingFrame without
                // scheduling another one - the coalescing this exists for.
                Interlocked.Exchange(ref _presentScheduled, 0);
            }
        }

        public void Shutdown()
        {
            if (!_isOpen) return;
            _isOpen = false;
            _window.Close();
        }

        public void Dispose() => Shutdown();
    }
}
