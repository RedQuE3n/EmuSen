using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // Which (element, property) pairs the scene reads, so a count of what is mapped is a count of code, not of intentions - see EmuSen_BigPicture.md §13.3.
    public static class SceneMapping
    {
        // Read by SceneBuilder.Place and Skip for every element that has them.
        public static readonly IReadOnlySet<string> Common = new HashSet<string>(StringComparer.Ordinal)
        {
            "pos", "origin", "size", "rotation", "rotationOrigin", "zIndex", "opacity", "visible", "metadataElement", "scope",
        };

        public static readonly IReadOnlyDictionary<string, IReadOnlySet<string>> Specific = new Dictionary<string, IReadOnlySet<string>>(StringComparer.Ordinal)
        {
            ["image"] = Set("path", "default", "imageType", "maxSize", "cropSize", "cropPos", "tile", "tileSize", "tileHorizontalAlignment", "tileVerticalAlignment",
                "color", "colorEnd", "gradientType", "saturation", "cornerRadius", "interpolation", "scrollFadeIn"),
            ["video"] = Set("imageType", "defaultImage", "imageSize", "imageMaxSize", "imageCropSize", "imageCropPos", "imageCornerRadius", "maxSize", "cropSize",
                "color", "colorEnd", "gradientType", "saturation", "scrollFadeIn"),
            ["text"] = Set("text", "metadata", "systemdata", "defaultValue", "systemNameSuffix", "letterCaseSystemNameSuffix", "container", "containerType", "fontPath",
                "fontSize", "horizontalAlignment", "verticalAlignment", "color", "backgroundColor", "backgroundMargins", "backgroundCornerRadius", "letterCase", "lineSpacing",
                "containerStartDelay", "containerScrollSpeed", "containerResetDelay", "containerScrollGap", "containerVerticalSnap"),
            ["datetime"] = Set("metadata", "defaultValue", "fontPath", "fontSize", "horizontalAlignment", "verticalAlignment", "color", "backgroundColor",
                "backgroundMargins", "backgroundCornerRadius", "letterCase", "lineSpacing", "format", "displayRelative"),
            ["carousel"] = Set("type", "staticImage", "defaultImage", "imageType", "maxItemCount", "itemSize", "itemScale", "imageFit", "imageColor", "imageSelectedColor",
                "imageSaturation", "itemHorizontalAlignment", "itemVerticalAlignment", "unfocusedItemOpacity", "unfocusedItemSaturation", "unfocusedItemDimming",
                "color", "colorEnd", "gradientType", "text", "textRelativeScale", "textColor", "textBackgroundColor", "fontPath", "fontSize", "letterCase", "itemTransitions", "fastScrolling"),
            ["textlist"] = Set("selectorHeight", "selectorVerticalOffset", "selectorColor", "primaryColor", "secondaryColor", "selectedColor", "selectedSecondaryColor",
                "selectedBackgroundColor", "selectedBackgroundMargins", "selectedBackgroundCornerRadius", "fontPath", "fontSize", "horizontalAlignment", "horizontalMargin",
                "letterCase", "lineSpacing", "systemNameSuffix", "letterCaseSystemNameSuffix", "indicators", "textHorizontalScrolling", "textHorizontalScrollSpeed",
                "textHorizontalScrollDelay", "textHorizontalScrollGap"),
            ["grid"] = Set("staticImage", "imageType", "defaultImage", "defaultFolderImage", "itemSize", "itemScale", "itemSpacing", "scaleInwards", "fractionalRows",
                "itemTransitions", "rowTransitions", "unfocusedItemOpacity", "unfocusedItemSaturation", "unfocusedItemDimming", "imageFit", "imageCropPos", "imageInterpolation",
                "imageRelativeScale", "imageCornerRadius", "imageColor", "imageColorEnd", "imageGradientType", "imageSelectedColor", "imageSaturation", "backgroundImage",
                "backgroundRelativeScale", "backgroundCornerRadius", "backgroundColor", "selectorImage", "selectorRelativeScale", "selectorLayer", "selectorCornerRadius",
                "selectorColor", "text", "textRelativeScale", "textBackgroundCornerRadius", "textColor", "textBackgroundColor", "textSelectedColor", "textSelectedBackgroundColor",
                "fontPath", "fontSize", "letterCase", "lineSpacing", "systemNameSuffix", "letterCaseSystemNameSuffix"),
            ["rating"] = Set("hideIfZero", "color", "filledPath", "unfilledPath", "overlay"),
            ["badges"] = Set("horizontalAlignment", "direction", "lines", "itemsPerLine", "itemMargin", "slots", "customBadgeIcon", "badgeIconColor"),
            ["helpsystem"] = Set("textColor", "iconColor", "fontPath", "fontSize", "entries", "entryRelativeScale", "entrySpacing", "iconTextSpacing", "letterCase",
                "backgroundColor", "backgroundHorizontalPadding", "backgroundVerticalPadding", "backgroundCornerRadius", "customButtonIcon"),
            ["clock"] = Set("fontPath", "fontSize", "horizontalAlignment", "verticalAlignment", "color", "backgroundColor", "backgroundColorEnd", "backgroundGradientType",
                "backgroundHorizontalPadding", "backgroundVerticalPadding", "backgroundCornerRadius", "format"),
            ["systemstatus"] = Set("height", "fontPath", "textRelativeScale", "color", "backgroundColor", "backgroundHorizontalPadding", "backgroundVerticalPadding",
                "backgroundCornerRadius", "entries", "entrySpacing", "customIcon"),
        };

        // Element types the scene draws; the others (animation, gamelistinfo, gameselector, sound) are not drawn.
        public static readonly IReadOnlySet<string> Drawn = new HashSet<string>(Specific.Keys, StringComparer.Ordinal);

        private static IReadOnlySet<string> Set(params string[] names) => new HashSet<string>(names, StringComparer.Ordinal);

        public static bool IsMapped(string element, string property, Func<string, bool> declared) =>
            Specific.TryGetValue(element, out IReadOnlySet<string>? own) && (own.Contains(property) || (Common.Contains(property) && declared(property)));

        public static IEnumerable<(string Element, string Property)> Unmapped(IEnumerable<(string Element, string Property)> pairs) =>
            pairs.Where(p => !IsMapped(p.Element, p.Property, _ => true));
    }
}
