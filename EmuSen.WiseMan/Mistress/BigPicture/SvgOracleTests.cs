using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using System.Xml;
using System.Xml.Linq;
using Avalonia;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using EmuSen.LunaP.Media;
using EmuSen.WiseMan.Fixtures;
using SkiaSharp;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // LunaP's SVG renderer graded against Svg.Skia on every SVG of Art Book Next, read in place - see EmuSen_BigPicture.md §13.7.
    public class SvgOracleTests
    {
        private readonly ITestOutputHelper _output;

        public SvgOracleTests(ITestOutputHelper output) => _output = output;

        private const int N = 256;

        // The six files that hold text, filter or script, and the one with a group inside a clip path.
        private static readonly string[] Refused = ["coco.svg", "emulators.svg", "epic.svg", "lowresnx.svg", "symbian.svg", "vpinball.svg", "windows3x.svg"];

        // Both renderers are told the same 256-pixel viewport in the markup, so neither infers it.
        private static string Sized(string markup)
        {
            try
            {
                XDocument x = XDocument.Parse(markup, LoadOptions.PreserveWhitespace);
                x.Root!.SetAttributeValue("width", N.ToString());
                x.Root!.SetAttributeValue("height", N.ToString());
                return x.ToString(SaveOptions.DisableFormatting);
            }
            catch (XmlException) { return markup; }
        }

        private static byte[] Ours(SvgDocument doc)
        {
            using var target = new RenderTargetBitmap(new PixelSize(N, N));
            using (DrawingContext dc = target.CreateDrawingContext()) doc.Draw(dc, new Rect(0, 0, N, N));
            var px = new byte[N * N * 4];
            GCHandle h = GCHandle.Alloc(px, GCHandleType.Pinned);
            try { target.CopyPixels(new PixelRect(0, 0, N, N), h.AddrOfPinnedObject(), px.Length, N * 4); }
            finally { h.Free(); }
            return px;
        }

        private static byte[]? Theirs(string markup)
        {
            using var svg = new Svg.Skia.SKSvg();
            if (svg.FromSvg(markup) is not { } picture) return null;
            var info = new SKImageInfo(N, N, SKColorType.Bgra8888, SKAlphaType.Premul);
            using SKSurface surface = SKSurface.Create(info);
            surface.Canvas.Clear(SKColors.Transparent);
            surface.Canvas.ClipRect(new SKRect(0, 0, N, N));
            surface.Canvas.DrawPicture(picture);
            var px = new byte[N * N * 4];
            GCHandle h = GCHandle.Alloc(px, GCHandleType.Pinned);
            try { surface.ReadPixels(info, h.AddrOfPinnedObject(), N * 4, 0, 0); }
            finally { h.Free(); }
            return px;
        }

        // Coverage IoU with alpha as a weight, and the mean colour difference where both are at least half opaque.
        public static (double IoU, double Colour) Compare(byte[] a, byte[] b)
        {
            double inter = 0, union = 0, colour = 0;
            long both = 0;
            for (int i = 0; i < a.Length; i += 4)
            {
                inter += Math.Min(a[i + 3], b[i + 3]);
                union += Math.Max(a[i + 3], b[i + 3]);
                if (a[i + 3] >= 128 && b[i + 3] >= 128)
                {
                    colour += (Math.Abs(a[i] - b[i]) + Math.Abs(a[i + 1] - b[i + 1]) + Math.Abs(a[i + 2] - b[i + 2])) / 3.0;
                    both++;
                }
            }

            return (union == 0 ? 1 : inter / union, both == 0 ? 0 : colour / both);
        }

        [ArtBookNextFact]
        public Task Every_Art_Book_Next_svg_is_drawn_as_Svg_Skia_draws_it_or_refused_whole() => UiTest.Run(() =>
        {
            string[] files = Directory.GetFiles(ArtBookNextFactAttribute.Folder, "*.svg", SearchOption.AllDirectories).OrderBy(f => f, StringComparer.Ordinal).ToArray();
            var refused = new List<string>();
            var scores = new List<(double IoU, double Colour, string File)>();
            foreach (string file in files)
            {
                string markup = Sized(File.ReadAllText(file));
                SvgDocument doc = SvgDocument.Parse(markup);
                if (doc.IsRefused)
                {
                    refused.Add(Path.GetFileName(file));
                    continue;
                }

                byte[] theirs = Theirs(markup) ?? throw new InvalidOperationException($"Svg.Skia could not read {file}");
                (double iou, double colour) = Compare(Ours(doc), theirs);
                scores.Add((iou, colour, Path.GetRelativePath(ArtBookNextFactAttribute.Folder, file)));
            }

            double[] ious = scores.Select(s => s.IoU).OrderBy(v => v).ToArray();
            _output.WriteLine($"{files.Length} files: {refused.Count} refused, {scores.Count} drawn; IoU min {ious[0]:F4}, median {ious[ious.Length / 2]:F4}; "
                + $"largest mean colour difference {scores.Max(s => s.Colour):F2} of 255");
            foreach (var s in scores.OrderBy(s => s.IoU).Take(5)) _output.WriteLine($"  {s.IoU:F4} colour {s.Colour:F2} {s.File}");
            Assert.Equal(Refused, refused.OrderBy(f => f, StringComparer.Ordinal));
            Assert.All(scores, s => Assert.True(s.IoU >= 0.98, $"{s.File}: IoU {s.IoU:F4}"));
        });
    }
}
