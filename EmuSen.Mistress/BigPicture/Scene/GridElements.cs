using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // The grid, the third primary element, as LunaP's ImageGrid, laid out by the rules measured from ES-DE - see EmuSen_BigPicture.md §16.
    internal static class GridElements
    {
        // A pair in pixels, -1 on one axis taking the other axis's pixels, as THEMES.md's itemSize and itemSpacing allow.
        internal static Size? Pixels(NormalizedPair? pair, double w, double h)
        {
            if (pair is not { } p) return null;
            double x = p.X * w, y = p.Y * h;
            if (p.X < 0 && p.Y >= 0) x = y;
            if (p.Y < 0 && p.X >= 0) y = x;
            return new Size(System.Math.Max(0, x), System.Math.Max(0, y));
        }

        internal static Control? Grid(SceneBuilder b, ResolvedElement e)
        {
            bool systems = b.View.Name == "system";
            IReadOnlyList<CarouselItem> items = systems ? SystemItems(b, e) : GameItems(b, e);
            if (items.Count == 0) return null;
            ThemeColor imageColor = e.Color("imageColor") ?? new ThemeColor(0xFFFFFFFF);
            ThemeColor? imageEnd = e.Color("imageColorEnd");
            return new ImageGrid
            {
                Items = items,
                SelectedIndex = systems ? b.Data.SystemIndex : b.Data.GameIndex,
                ItemSize = Pixels(e.Pair("itemSize"), b.W, b.H) ?? new Size(0.15 * b.W, 0.25 * b.H),
                ItemSpacing = Pixels(e.Pair("itemSpacing"), b.W, b.H),
                ItemScale = e.Float("itemScale") ?? 1.05f,
                ScaleInwards = e.Bool("scaleInwards") == true,
                FractionalRows = e.Bool("fractionalRows") == true,
                UnfocusedItemOpacity = e.Float("unfocusedItemOpacity") ?? 1,
                UnfocusedItemSaturation = e.Float("unfocusedItemSaturation") ?? 1,
                UnfocusedItemDimming = e.Float("unfocusedItemDimming") ?? 1,
                ImageFit = e.String("imageFit") switch { "fill" => ImageFit.Fill, "cover" => ImageFit.Cover, _ => ImageFit.Contain },
                ImageCropPosition = SceneUnits.ToPoint(e.Pair("imageCropPos"), new Point(0.5, 0.5)),
                ImageInterpolation = SceneUnits.Interpolation(e.String("imageInterpolation")),
                ImageRelativeScale = e.Float("imageRelativeScale") ?? 1,
                ImageCornerRadius = SceneUnits.Px(e.Float("imageCornerRadius") ?? 0, b.W),
                ImageTint = SceneUnits.ToColor(imageColor),
                ImageTintEnd = imageEnd is { } end && end != imageColor ? SceneUnits.ToColor(end) : null,
                ImageTintDirection = SceneUnits.Gradient(e.String("imageGradientType")),
                ImageSelectedTint = e.Color("imageSelectedColor") is { } selected && selected != imageColor ? SceneUnits.ToColor(selected) : null,
                ImageSaturation = e.Float("imageSaturation") ?? 1,
                ItemBackground = e.Color("backgroundColor") is { } bg ? SceneUnits.ToColor(bg) : null,
                ItemBackgroundImage = ImageElements.Existing(e.Path("backgroundImage")),
                ItemBackgroundRelativeScale = e.Float("backgroundRelativeScale") ?? 1,
                ItemBackgroundCornerRadius = SceneUnits.Px(e.Float("backgroundCornerRadius") ?? 0, b.W),
                SelectorColor = e.Color("selectorColor") is { } sel ? SceneUnits.ToColor(sel) : null,
                SelectorImage = ImageElements.Existing(e.Path("selectorImage")),
                SelectorRelativeScale = e.Float("selectorRelativeScale") ?? 1,
                SelectorCornerRadius = SceneUnits.Px(e.Float("selectorCornerRadius") ?? 0, b.W),
                SelectorLayer = e.String("selectorLayer") switch { "bottom" => GridSelectorLayer.Bottom, "middle" => GridSelectorLayer.Middle, _ => GridSelectorLayer.Top },
                FontPath = ImageElements.Existing(e.Path("fontPath")),
                FontSize = SceneUnits.Px((e.Float("fontSize") ?? 0.045f) * (e.Float("textRelativeScale") ?? 1), b.H),
                LineSpacing = e.Float("lineSpacing") ?? 1.5f,
                TextColor = SceneUnits.ToColor(e.Color("textColor"), Colors.Black),
                TextSelectedColor = e.Color("textSelectedColor") is { } ts ? SceneUnits.ToColor(ts) : null,
                TextBackground = SceneUnits.ToColor(e.Color("textBackgroundColor"), Colors.Transparent),
                TextSelectedBackground = e.Color("textSelectedBackgroundColor") is { } tsb ? SceneUnits.ToColor(tsb) : null,
                TextBackgroundCornerRadius = SceneUnits.Px(e.Float("textBackgroundCornerRadius") ?? 0, b.W),
                LetterCase = SceneUnits.Case(e.String("letterCase")),
            };
        }

        // One item per system: its own theme's staticImage, else its defaultImage, else the text.
        private static IReadOnlyList<CarouselItem> SystemItems(SceneBuilder b, ResolvedElement e) => b.Data.Systems.Select(s =>
        {
            ResolvedElement? own = s.Theme.SystemView.Primary ?? e;
            string? image = ImageElements.Existing(own.Path("staticImage")) ?? ImageElements.Existing(own.Path("defaultImage"));
            return new CarouselItem(image, own.String("text") ?? s.System.FullName);
        }).ToList();

        // One item per game: the first of its imageType media it has, else the folder or default image, else its name.
        private static IReadOnlyList<CarouselItem> GameItems(SceneBuilder b, ResolvedElement e)
        {
            SceneSystem system = b.Data.System;
            IReadOnlyList<string> types = e.Bindings.FirstOrDefault(x => x.Kind == "media")?.Names ?? [];
            string? fallback = ImageElements.Existing(e.Path("defaultImage")), folder = ImageElements.Existing(e.Path("defaultFolderImage")) ?? fallback;
            bool suffix = e.Bool("systemNameSuffix") != false && system.System.Kind != ThemeSystemKind.Regular;
            string suffixCase = e.String("letterCaseSystemNameSuffix") ?? "uppercase";
            return system.Games.Select(g => new CarouselItem(
                types.Select(t => b.Data.Media?.Find(system.System, g, t)).FirstOrDefault(p => p is not null) ?? (g.Folder ? folder : fallback),
                suffix ? $"{g.Name} [{SceneUnits.Cased(system.System.Name, suffixCase)}]" : g.Name)).ToList();
        }
    }
}
