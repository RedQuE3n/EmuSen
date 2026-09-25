using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // A SceneView shown in a headless window, rendered at chosen times on its own clock, and strips of those frames written as one PNG - see EmuSen_BigPicture.md §14.6.
    public sealed class SceneMotionHost : IDisposable
    {
        private readonly Window _window;

        public SceneMotionHost(SceneView view) : this(view.Root, view.Data.Screen.Width, view.Data.Screen.Height) => _view = view;

        public SceneMotionHost(Control root, double width, double height)
        {
            _window = new Window { Width = width, Height = height, Content = root, Background = Brushes.Black, SizeToContent = SizeToContent.Manual };
            _window.Show();
            Dispatcher.UIThread.RunJobs();
        }

        private readonly SceneView? _view;

        public SceneView View => _view ?? throw new InvalidOperationException("this host shows a bare control");

        public Window Window => _window;

        // The frame at a time: the view's clock stepped there, then a full redraw read back.
        public RenderedFrame At(TimeSpan now)
        {
            View.Advance(now);
            return Frame();
        }

        public RenderedFrame Frame()
        {
            Dispatcher.UIThread.RunJobs();
            _window.CaptureRenderedFrame()?.Dispose();
            foreach (Visual v in _window.GetSelfAndVisualDescendants()) v.InvalidateVisual();
            using WriteableBitmap bitmap = _window.CaptureRenderedFrame() ?? throw new InvalidOperationException("no frame");
            return UiTest.Capture(bitmap);
        }

        public void Dispose() => _window.Close();

        // Frames side by side, each scaled down by a whole factor, with a white rule between them.
        public static void Strip(string path, IReadOnlyList<RenderedFrame> frames, int shrink = 2)
        {
            int w = frames[0].Width / shrink, h = frames[0].Height / shrink, gap = 4;
            int total = frames.Count * w + (frames.Count - 1) * gap;
            var rgba = new byte[total * h * 4];
            for (int i = 0; i < rgba.Length; i += 4) rgba[i] = rgba[i + 1] = rgba[i + 2] = rgba[i + 3] = 255;
            for (int f = 0; f < frames.Count; f++)
            {
                RenderedFrame frame = frames[f];
                for (int y = 0; y < h; y++)
                    for (int x = 0; x < w; x++)
                    {
                        int[] sum = new int[4];
                        for (int dy = 0; dy < shrink; dy++)
                            for (int dx = 0; dx < shrink; dx++)
                            {
                                int s = ((y * shrink + dy) * frame.Width + x * shrink + dx) * 4;
                                for (int c = 0; c < 4; c++) sum[c] += frame.Rgba[s + c];
                            }

                        int o = (y * total + f * (w + gap) + x) * 4;
                        for (int c = 0; c < 4; c++) rgba[o + c] = (byte)(sum[c] / (shrink * shrink));
                    }
            }

            new RenderedFrame(rgba, total, h).SavePng(path);
        }
    }
}
