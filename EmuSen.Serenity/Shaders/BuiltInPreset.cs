using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Galaxia.Library;
using EmuSen.Serenity.Native;

namespace EmuSen.Serenity.Shaders
{
    // A built-in filter written as a slang preset for the chain, which the platform library alone translates and writes - see EmuSen_DevicePicture.md §3 and §20 and EmuSen_Serenity.md §3.13.
    public static class BuiltInPreset
    {
        // The parameters the runner sets at each draw, which no settings window lists, by the names the library writes them under - see EmuSen_DevicePicture.md §3.2.
        public const string PixelScale = "emusen_pixel_scale", OriginX = "emusen_pixel_origin_x", OriginY = "emusen_pixel_origin_y";
        public const string CropLeft = "emusen_crop_left", CropTop = "emusen_crop_top", CropWidth = "emusen_crop_width", CropHeight = "emusen_crop_height";
        public const string RawWidth = "emusen_raw_width", RawHeight = "emusen_raw_height";

        // How many rows of a frame are one of the console's lines, for a filter that counts lines and keeps no history; one where the frame is not a whole multiple.
        public static int ReduceBy(ScreenFilter filter, int width, int height, int lines) => FilterLibrary.ReduceBy(filter, width, height, lines);

        // The chosen values a filter's passes are written from, its structural ones; every other value reaches the chain at each draw - see EmuSen_DevicePicture.md §19.
        public static Dictionary<string, float>? StructuralOf(ScreenFilter filter, IReadOnlyDictionary<string, float>? chosen)
        {
            if (chosen is null || filter.Structural is not { } ids) return null;
            var kept = new Dictionary<string, float>(StringComparer.Ordinal);
            foreach (string id in ids) if (chosen.TryGetValue(id, out float value)) kept[id] = value;
            return kept;
        }

        // Where generated presets are kept, a folder per content; a test sets it before any is written.
        internal static string? RootOverride;

        public static string Root => RootOverride ?? Path.Combine(DataStore.Shaders, "builtin");

        // A frame drawn at a whole multiple of the console's picture, brought back to it - see EmuSen_CRT.md §16.
        public static string ReduceGlsl => FilterCatalog.ReduceGlsl;

        // One translated pass: its slang text and what the preset says of it.
        public sealed record Pass(string FileName, string Text, string? Alias, PassSize Width, PassSize Height, bool Float, bool LinearSampler);

        // The passes a filter draws on a console, reducing pass first; `nudge` shifts every sampling point for the tie test - see EmuSen_DevicePicture.md §12.
        public static IReadOnlyList<Pass> Translate(ScreenFilter filter, IReadOnlyDictionary<string, float> values, int reduceBy = 1, (float X, float Y) nudge = default) =>
            FilterLibrary.Translate(filter, values, reduceBy, nudge);

        // The values a filter's preset is written with on a console: each parameter's default there, then the console's constants.
        public static Dictionary<string, float> ValuesFor(ScreenFilter filter, string? console, IReadOnlyDictionary<string, float>? chosen = null) => filter.ValuesFor(console, chosen);

        // The lookup image a pass reads the raw frame from.
        public const string RawTexture = "raw";

        // Writes the preset into a folder named by its content and returns the preset's path; the same content is written once.
        public static string Write(ScreenFilter filter, string? console, IReadOnlyDictionary<string, float>? chosen = null, int reduceBy = 1, (float X, float Y) nudge = default) =>
            WriteInto(Root, filter, console, chosen, reduceBy, nudge);

        // The same into a root of the caller's, which a test names so that no other test's presets are beside its own.
        internal static string WriteInto(string root, ScreenFilter filter, string? console, IReadOnlyDictionary<string, float>? chosen, int reduceBy, (float X, float Y) nudge) =>
            FilterLibrary.Write(root, filter, console, chosen, reduceBy, nudge);
    }
}
