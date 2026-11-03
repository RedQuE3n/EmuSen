using System;
using System.Globalization;
using EmuSen.Galaxia.Models;
using EmuSen.Serenity;

namespace EmuSen.Mistress.Library
{
    // What a console's stored crop means: which consoles had an overscan, and what a console's choice hides at each edge - see EmuSen_Settings_Reference.md §4.99.
    public static class ConsoleCrop
    {
        // The edges of a custom crop in the order they are shown, with the key a console's value is stored under.
        public static readonly (string Key, string Label)[] Edges = { ("CropLeft", "Left"), ("CropRight", "Right"), ("CropTop", "Top"), ("CropBottom", "Bottom") };

        // The most one edge may hide, as a percentage.
        public static double MostPercent => PictureCrop.MostAtAnEdge * 100;

        // The consoles that drove a television, an attachment by its host's screen; an LCD showed every pixel - see EmuSen_Serenity.md §3.12.
        public static bool HasOverscan(string console) => EmuSen.Serenity.Shaders.CrtFilter.TelevisionConsoles.Contains(RuntimeCatalog.HostOf(console));

        public static PictureCrop CropFor(GraphicsConfig config, string console)
        {
            if (!HasOverscan(console)) return PictureCrop.None;
            double Edge(int i) => ParsePercent(config.Value(console, Edges[i].Key)) ?? 0;
            return config.Value(console, GraphicsKeys.Overscan) switch
            {
                GraphicsKeys.TvOverscan => PictureCrop.Television,
                GraphicsKeys.CustomCrop => PictureCrop.Percent(Edge(0), Edge(2), Edge(1), Edge(3)),
                _ => PictureCrop.None,
            };
        }

        // A percentage as typed, with a point or a comma, held to what one edge may hide; null when it is no number.
        public static double? ParsePercent(string? text) =>
            double.TryParse(text?.Trim().Replace(',', '.'), NumberStyles.Float, CultureInfo.InvariantCulture, out double value) && double.IsFinite(value)
                ? Math.Clamp(Math.Round(value, 1), 0, MostPercent) : null;

        public static string Text(double percent) => percent.ToString("0.#", CultureInfo.InvariantCulture);
    }
}
