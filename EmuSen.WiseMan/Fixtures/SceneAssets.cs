using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.WiseMan.Fixtures
{
    // Pictures, an icon and a font the scene tests write for themselves, a media source over them, and a headless render of a scene - see EmuSen_BigPicture.md §13.5.
    public static class SceneAssets
    {
        public static readonly string Folder = Path.Combine(Path.GetTempPath(), "EmuSenSceneAssets", Environment.ProcessId.ToString());

        // A picture split into a left and a right colour, so crops, fits and tiles all show in its pixels.
        public static string Halves(string name, int w, int h, Color left, Color right)
        {
            string path = Path.Combine(Folder, name + ".png");
            if (File.Exists(path)) return path;
            Directory.CreateDirectory(Folder);
            using var target = new RenderTargetBitmap(new PixelSize(w, h));
            using (DrawingContext dc = target.CreateDrawingContext())
            {
                dc.FillRectangle(new SolidColorBrush(left), new Rect(0, 0, w / 2.0, h));
                dc.FillRectangle(new SolidColorBrush(right), new Rect(w / 2.0, 0, w / 2.0, h));
                dc.FillRectangle(Brushes.White, new Rect(0, 0, w, h / 8.0));
            }

            target.Save(path);
            return path;
        }

        public static string Svg(string name, string body)
        {
            Directory.CreateDirectory(Folder);
            string path = Path.Combine(Folder, name + ".svg");
            File.WriteAllText(path, $"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 20 20'>{body}</svg>");
            return path;
        }

        // One of Avalonia.Fonts.Inter's faces as a file, since a theme names its fonts by path.
        public static string Font(string face)
        {
            Directory.CreateDirectory(Folder);
            string path = Path.Combine(Folder, face + ".ttf");
            if (File.Exists(path)) return path;
            using Stream source = AssetLoader.Open(new Uri($"avares://Avalonia.Fonts.Inter/Assets/{face}.ttf"));
            using FileStream target = File.Create(path);
            source.CopyTo(target);
            return path;
        }

        // Every game has every media type, each a different picture.
        public sealed class Media : ISceneMedia
        {
            public string? Find(ThemeSystem system, SceneGame game, string mediaType) => mediaType switch
            {
                "cover" => Halves("media-cover", 60, 80, Colors.DarkOrange, Colors.Teal),
                "screenshot" => Halves("media-screenshot", 80, 70, Colors.Purple, Colors.Gold),
                "marquee" => Halves("media-marquee", 100, 30, Colors.Crimson, Colors.Olive),
                "miximage" or "titlescreen" or "backcover" or "3dbox" or "physicalmedia" or "fanart" => Halves("media-other", 70, 70, Colors.SteelBlue, Colors.Salmon),
                _ => null,
            };
        }

        public static RenderedFrame Render(SceneBuilder scene, Action<Window>? adjust = null)
        {
            var window = new Window { Width = scene.W, Height = scene.H, Content = scene.Canvas, Background = Brushes.Black };
            window.Show();
            Dispatcher.UIThread.RunJobs();
            adjust?.Invoke(window);
            Dispatcher.UIThread.RunJobs();
            window.CaptureRenderedFrame()?.Dispose();
            foreach (Visual v in window.GetSelfAndVisualDescendants()) v.InvalidateVisual();
            using WriteableBitmap bitmap = window.CaptureRenderedFrame() ?? throw new InvalidOperationException("no frame");
            RenderedFrame frame = UiTest.Capture(bitmap);
            window.Close();
            return frame;
        }

        public static int Differing(RenderedFrame a, RenderedFrame b)
        {
            int n = 0;
            for (int i = 0; i < a.Rgba.Length; i += 4)
                if (a.Rgba[i] != b.Rgba[i] || a.Rgba[i + 1] != b.Rgba[i + 1] || a.Rgba[i + 2] != b.Rgba[i + 2]) n++;
            return n;
        }
    }
}
