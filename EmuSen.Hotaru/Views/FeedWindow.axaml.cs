using System;
using System.Runtime.InteropServices;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using Avalonia.Threading;

namespace EmuSen.Hotaru.Views
{
    // The `feed`/`feed -w` window (Program.cs's own RunDebugPrompt) - a
    // live mirror of the actual game picture, not hardware/debug data
    // the way CoretopWindow shows. Exists purely because Raylib supports
    // exactly one native window per process (same constraint
    // AvaloniaHost.cs's own header comment explains for `coretop -w`),
    // so a second, independent view of the live framebuffer has to come
    // from somewhere else entirely.
    //
    // Refreshes much faster than CoretopWindow's 250ms/4Hz debug cadence
    // (~30 times/second) - this is meant to look like actually watching
    // the game, not a slowly-updating stat readout, though it's still
    // well under the SNES's real 60fps since this is a convenience
    // preview, not the primary way to play.
    //
    // Same accepted data-race caveat as CoretopWindow/AvaloniaHost's own
    // comment: the frame-provider delegate reads ICore state from this
    // window's own timer while Program.cs's main loop concurrently calls
    // core.RunFrame() with no synchronization - accepted here for the
    // same reason, a read-mostly view that never feeds back into
    // gameplay logic.
    public partial class FeedWindow : Window
    {
        private static readonly TimeSpan RefreshInterval = TimeSpan.FromMilliseconds(33);

        private readonly DispatcherTimer _timer;
        private readonly Func<(byte[] Rgba, int Width, int Height)> _frameProvider;
        private WriteableBitmap? _bitmap;
        private int _bitmapWidth;
        private int _bitmapHeight;

        public FeedWindow() : this(() => (Array.Empty<byte>(), 0, 0)) { }

        public FeedWindow(Func<(byte[] Rgba, int Width, int Height)> frameProvider)
        {
            InitializeComponent();
            _frameProvider = frameProvider;

            _timer = new DispatcherTimer { Interval = RefreshInterval };
            _timer.Tick += (_, _) => Refresh();
            _timer.Start();
            Closed += (_, _) => _timer.Stop();

            Refresh();
        }

        private void Refresh()
        {
            (byte[] rgba, int width, int height) = _frameProvider();
            if (width <= 0 || height <= 0 || rgba.Length < width * height * 4) return;

            // Reused across frames rather than rebuilt each tick (unlike
            // CoretopWindow's palette/tile images, which really can
            // change size) - the game's screen resolution is fixed for
            // the life of a session, so this only actually resizes once,
            // right after the first real frame comes in.
            if (_bitmap is null || _bitmapWidth != width || _bitmapHeight != height)
            {
                _bitmap = new WriteableBitmap(new PixelSize(width, height), new Vector(96, 96), PixelFormat.Rgba8888, AlphaFormat.Opaque);
                _bitmapWidth = width;
                _bitmapHeight = height;
                FeedImage.Source = _bitmap;
            }

            using (ILockedFramebuffer fb = _bitmap.Lock())
            {
                Marshal.Copy(rgba, 0, fb.Address, width * height * 4);
            }
            FeedImage.InvalidateVisual();
        }
    }
}
