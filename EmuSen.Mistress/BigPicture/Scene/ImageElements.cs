using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // image and video (shown as its image) as a FittedImage - see EmuSen_BigPicture.md §13.2.
    internal static class ImageElements
    {
        internal static string? Existing(ThemePath? path) => path is { Exists: true } p ? p.Absolute : null;

        // The first media type a game has, in the order the element lists them.
        internal static string? Media(SceneBuilder b, ResolvedElement e, string kind)
        {
            if (b.Data.Media is not { } media || b.Data.Game is not { } game) return null;
            IReadOnlyList<string> types = e.Bindings.FirstOrDefault(x => x.Kind == kind)?.Names ?? [];
            return types.Select(t => media.Find(b.Data.System.System, game, t)).FirstOrDefault(p => p is not null);
        }

        internal static Control? Image(SceneBuilder b, ResolvedElement e)
        {
            bool typed = e.List("imageType").Count > 0;
            string? source = (typed ? Media(b, e, "media") : Existing(e.Path("path"))) ?? Existing(e.Path("default"));
            if (source is null) return null;
            var image = new FittedImage { Source = source };
            Size(b, image, e, e.Pair("size"), e.Pair("maxSize"), e.Pair("cropSize"), e.Pair("cropPos"));
            if (e.Bool("tile") == true)
            {
                image.Fit = ImageFit.Tile;
                Size tile = SceneUnits.ToSize(e.Pair("tileSize"));
                image.TileSize = new Size(SceneUnits.Px(tile.Width, b.W), SceneUnits.Px(tile.Height, b.H));
                image.TileHorizontalAlignment = e.String("tileHorizontalAlignment") == "right" ? Avalonia.Layout.HorizontalAlignment.Right : Avalonia.Layout.HorizontalAlignment.Left;
                image.TileVerticalAlignment = e.String("tileVerticalAlignment") == "top" ? Avalonia.Layout.VerticalAlignment.Top : Avalonia.Layout.VerticalAlignment.Bottom;
            }

            Paint(b, image, e, "cornerRadius");
            return image;
        }

        // A video element with no video playing: its image, fitted by the image sizes, which default to the video's own.
        internal static Control? Video(SceneBuilder b, ResolvedElement e)
        {
            string? source = Media(b, e, "videoFallbackImage") ?? Existing(e.Path("defaultImage"));
            if (source is null) return null;
            var image = new FittedImage { Source = source };
            bool own = e.Explicit.ContainsKey("imageSize") || e.Explicit.ContainsKey("imageMaxSize") || e.Explicit.ContainsKey("imageCropSize");
            NormalizedPair? Own(string name) => own ? (e.Explicit.GetValueOrDefault(name) as PairValue)?.Value : e.Pair(name);
            Size(b, image, e, Own("imageSize"), Own("imageMaxSize"), Own("imageCropSize"), e.Pair("imageCropPos"));
            Paint(b, image, e, "imageCornerRadius");
            return image;
        }

        // size stretches, maxSize fits inside, cropSize covers; with none, the image's own pixel size.
        private static void Size(SceneBuilder b, FittedImage image, ResolvedElement e, NormalizedPair? size, NormalizedPair? max, NormalizedPair? crop, NormalizedPair? cropPos)
        {
            if (size is { } s)
            {
                image.Fit = ImageFit.Fill;
                NormalizedCanvas.SetSize(image, SceneUnits.ToSize(s));
            }
            else if (max is { } m)
            {
                image.Fit = ImageFit.Contain;
                NormalizedCanvas.SetMaxSize(image, SceneUnits.ToSize(m));
                NormalizedCanvas.SetSize(image, new Size(0, 0));
            }
            else if (crop is { } c)
            {
                image.Fit = ImageFit.Cover;
                image.CropPosition = SceneUnits.ToPoint(cropPos, new Point(0.5, 0.5));
                NormalizedCanvas.SetSize(image, SceneUnits.ToSize(c));
            }
            else
            {
                image.Fit = ImageFit.Contain;
            }
        }

        private static void Paint(SceneBuilder b, FittedImage image, ResolvedElement e, string cornerProperty)
        {
            ThemeColor color = e.Color("color") ?? new ThemeColor(0xFFFFFFFF);
            ThemeColor end = e.Color("colorEnd") ?? color;
            image.Tint = SceneUnits.ToColor(color);
            image.TintEnd = end == color ? null : SceneUnits.ToColor(end);
            image.TintDirection = SceneUnits.Gradient(e.String("gradientType"));
            image.Saturation = e.Float("saturation") ?? 1;
            image.CornerRadius = SceneUnits.Px(e.Float(cornerProperty) ?? 0, b.W);
            image.Interpolation = SceneUnits.Interpolation(e.String("interpolation"));
        }
    }
}
