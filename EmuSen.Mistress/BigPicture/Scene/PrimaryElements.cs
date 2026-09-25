using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Layout;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // carousel and textlist, the views' primary elements, at rest - see EmuSen_BigPicture.md §13.2.
    internal static class PrimaryElements
    {
        internal static Control? Carousel(SceneBuilder b, ResolvedElement e)
        {
            bool systems = b.View.Name == "system";
            IReadOnlyList<CarouselItem> items = systems ? SystemItems(b) : GameItems(b, e);
            if (items.Count == 0) return null;
            string type = e.String("type") ?? "horizontal";
            var carousel = new ImageCarousel
            {
                Items = items,
                SelectedIndex = systems ? b.Data.SystemIndex : b.Data.GameIndex,
                Orientation = type.StartsWith("vertical") ? Orientation.Vertical : Orientation.Horizontal,
                MaxItemCount = e.Float("maxItemCount") ?? 3,
                ItemScale = e.Float("itemScale") ?? 1.2f,
                UnfocusedItemOpacity = e.Float("unfocusedItemOpacity") ?? 0.5f,
                UnfocusedItemSaturation = e.Float("unfocusedItemSaturation") ?? 1,
                UnfocusedItemDimming = e.Float("unfocusedItemDimming") ?? 1,
                ImageTint = SceneUnits.ToColor(e.Color("imageColor"), Colors.White),
                ImageSelectedTint = e.Color("imageSelectedColor") is { } selected ? SceneUnits.ToColor(selected) : null,
                ImageSaturation = e.Float("imageSaturation") ?? 1,
                ImageFit = e.String("imageFit") switch { "fill" => ImageFit.Fill, "cover" => ImageFit.Cover, _ => ImageFit.Contain },
                ItemVerticalAlignment = SceneUnits.Vertical(e.String("itemVerticalAlignment")),
                ItemHorizontalAlignment = SceneUnits.HorizontalLayout(e.String("itemHorizontalAlignment") ?? "center"),
                Background = SceneUnits.Fill(e.Color("color"), e.Color("colorEnd"), e.String("gradientType")),
                FontPath = ImageElements.Existing(e.Path("fontPath")),
                FontSize = SceneUnits.Px((e.Float("fontSize") ?? 0.085f) * (e.Float("textRelativeScale") ?? 1), b.H),
                TextColor = SceneUnits.ToColor(e.Color("textColor"), Colors.Black),
                TextBackground = SceneUnits.ToColor(e.Color("textBackgroundColor"), Colors.Transparent),
                LetterCase = SceneUnits.Case(e.String("letterCase")),
            };
            Size item = SceneUnits.ToSize(e.Pair("itemSize"));
            carousel.ItemSize = new Size(SceneUnits.Px(item.Width, b.W), SceneUnits.Px(item.Height, b.H));
            return carousel;
        }

        // One item per system, each drawn with the artwork its own resolved theme names.
        private static IReadOnlyList<CarouselItem> SystemItems(SceneBuilder b) => b.Data.Systems.Select(s =>
        {
            ResolvedElement? own = s.Theme.SystemView.Primary;
            string? image = own is null ? null : ImageElements.Existing(own.Path("staticImage")) ?? ImageElements.Existing(own.Path("defaultImage"));
            return new CarouselItem(image, own?.String("text") ?? s.System.FullName);
        }).ToList();

        private static IReadOnlyList<CarouselItem> GameItems(SceneBuilder b, ResolvedElement e)
        {
            SceneSystem system = b.Data.System;
            IReadOnlyList<string> types = e.Bindings.FirstOrDefault(x => x.Kind == "media")?.Names ?? [];
            string? fallback = ImageElements.Existing(e.Path("defaultImage"));
            return system.Games.Select(g => new CarouselItem(
                types.Select(t => b.Data.Media?.Find(system.System, g, t)).FirstOrDefault(p => p is not null) ?? fallback, g.Name)).ToList();
        }

        // ES-DE marks favourites and folders before the name unless indicators is none; the marks are LunaP's own drawings (§3.6).
        private static TextRowMarker Marker(ResolvedElement e, SceneGame g) =>
            e.String("indicators") == "none" ? TextRowMarker.None : g.Folder ? TextRowMarker.Folder : g.Favorite ? TextRowMarker.Star : TextRowMarker.None;

        internal static Control? TextList(SceneBuilder b, ResolvedElement e)
        {
            SceneSystem system = b.Data.System;
            bool suffix = e.Bool("systemNameSuffix") == true && system.System.Kind != ThemeSystemKind.Regular;
            string suffixCase = e.String("letterCaseSystemNameSuffix") ?? "uppercase";
            IReadOnlyList<TextRow> rows = b.View.Name == "system"
                ? b.Data.Systems.Select(s => new TextRow(s.System.FullName)).ToList()
                : system.Games.Select(g => new TextRow(suffix ? $"{g.Name} [{SceneUnits.Cased(system.System.Name, suffixCase)}]" : g.Name, g.Folder, Marker(e, g))).ToList();
            float fontSize = e.Float("fontSize") ?? 0.045f;
            Size margins = SceneUnits.ToSize(e.Pair("selectedBackgroundMargins"));
            return new TextRowList
            {
                Items = rows,
                SelectedIndex = b.View.Name == "system" ? b.Data.SystemIndex : b.Data.GameIndex,
                FontPath = ImageElements.Existing(e.Path("fontPath")),
                FontSize = SceneUnits.Px(fontSize, b.H),
                LineSpacing = e.Float("lineSpacing") ?? 1.5f,
                PrimaryColor = SceneUnits.ToColor(e.Color("primaryColor"), Colors.Blue),
                SecondaryColor = SceneUnits.ToColor(e.Color("secondaryColor"), Colors.Lime),
                SelectedColor = SceneUnits.ToColor(e.Color("selectedColor") ?? e.Color("primaryColor"), Colors.Blue),
                SelectedSecondaryColor = e.Color("selectedSecondaryColor") is { } ss ? SceneUnits.ToColor(ss) : null,
                SelectorColor = SceneUnits.ToColor(e.Color("selectorColor"), Color.FromRgb(0x33, 0x33, 0x33)),
                SelectorHeight = SceneUnits.Px(e.Float("selectorHeight") ?? fontSize * 1.5f, b.H),
                TextBandHeight = SceneUnits.Px(e.Float("selectorHeight") ?? fontSize * 1.5f, b.H),
                SelectedBackgroundFitsText = true,
                SelectorOffsetY = SceneUnits.Px(e.Float("selectorVerticalOffset") ?? 0, b.H),
                SelectedBackgroundColor = SceneUnits.ToColor(e.Color("selectedBackgroundColor"), Colors.Transparent),
                SelectedBackgroundMargins = new Thickness(SceneUnits.Px(margins.Width, b.W), 0, SceneUnits.Px(margins.Height, b.W), 0),
                SelectedBackgroundCornerRadius = SceneUnits.Px(e.Float("selectedBackgroundCornerRadius") ?? 0, b.W),
                TextAlignment = SceneUnits.Horizontal(e.String("horizontalAlignment")),
                HorizontalMargin = SceneUnits.Px(e.Float("horizontalMargin") ?? 0, b.W),
                LetterCase = SceneUnits.Case(e.String("letterCase")),
            };
        }
    }
}
