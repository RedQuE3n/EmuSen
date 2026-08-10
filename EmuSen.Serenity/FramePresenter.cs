using System;
using System.Threading;
using Avalonia.Controls;
using Avalonia.Threading;
using EmuSen.Graphics;
using EmuSen.LunaP.Threading;

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

        // "Newest wins, at most one UI-thread callback outstanding" - the mechanism this class,
        // Mistress's MainWindow and Hotaru's GameWindow each wrote out identically, now the
        // toolkit's. LunaP.md §22.1 records both the generalisation and the defect all three
        // copies shared: they cleared the scheduled flag AFTER the hand-off, so a frame submitted
        // while the UI thread was inside UpdateFrame could neither schedule a callback nor be
        // collected by the running one, and sat there until the next frame pushed it out.
        //
        // At 60 fps that is invisible - the next frame arrives 16 ms later carrying the fix. It
        // shows when the stream STOPS: pause the emulator and the frame at risk is the last one,
        // which is the frame somebody is about to sit and look at.
        private readonly Latest<FrameData> _frames;

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
            _frames = new Latest<FrameData>(PresentPendingFrame);
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

        public void Present(byte[] rgba, int width, int height) =>
            _frames.Offer(new FrameData { Rgba = rgba, Width = width, Height = height });

        // Presents the newest frame at the moment it runs; stale ones drop by design - see EmuSen_Serenity.md §4.
        private void PresentPendingFrame(FrameData frame)
        {
            if (!_isOpen) return;

            _control.UpdateFrame(frame.Rgba, frame.Width, frame.Height);
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
