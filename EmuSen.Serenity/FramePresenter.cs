using System;
using System.Threading;
using Avalonia.Controls;
using Avalonia.Threading;
using EmuSen.Graphics;

namespace EmuSen.Serenity
{
    // The built-in single-pass effects; None is always the fallback - see EmuSen_Serenity.md §3.
    public enum ShaderEffect
    {
        None,
        Scanlines,
        Crt,
    }

    // A Window + GameFrameControl bundle, coalescing frames from any thread - see EmuSen_Serenity.md §4.
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

        // Cycles None -> Scanlines -> Crt -> None for manual A/B testing - see EmuSen_Serenity.md §4.
        public ShaderEffect CycleEffect()
        {
            _control.ActiveEffect = NextEffect(_control.ActiveEffect);
            return _control.ActiveEffect;
        }

        // Pure, so it is testable without opening a real Window.
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

        // Presents the newest frame at the moment it runs; stale ones drop by design - see EmuSen_Serenity.md §4.
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
                // Reset after the hand-off, so an in-flight Present keeps coalescing - see EmuSen_Serenity.md §4.
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
