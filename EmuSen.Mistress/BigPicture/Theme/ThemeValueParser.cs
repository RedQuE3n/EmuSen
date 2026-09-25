using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;

namespace EmuSen.Mistress.BigPicture.Theme
{
    // Turns a property's text, variables already substituted, into its documented type - see EmuSen_BigPicture.md §12.3.
    public static class ThemeValueParser
    {
        // The image and video imageType shortcut - see THEMES.md "image".
        public static IReadOnlyList<string> ImageShortcut { get; } = ["miximage", "screenshot", "titlescreen", "cover"];

        public static bool TryParseBool(string text, out bool value)
        {
            switch (text.ToLowerInvariant())
            {
                case "true": case "1": value = true; return true;
                case "false": case "0": value = false; return true;
                default: value = false; return false;
            }
        }

        public static bool TryParseFloat(string text, out float value) =>
            float.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out value) && float.IsFinite(value);

        public static bool TryParsePair(string text, out NormalizedPair pair)
        {
            pair = default;
            string[] parts = text.Split((char[])[' ', '\t', '\r', '\n'], StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length != 2 || !TryParseFloat(parts[0], out float x) || !TryParseFloat(parts[1], out float y)) return false;
            pair = new NormalizedPair(x, y);
            return true;
        }

        // ./ is the file's folder, ~ is home, and backslashes are separators - see EmuSen_BigPicture.md §12.2.
        public static string ResolvePath(string written, string file)
        {
            string text = written.Trim().Replace('\\', '/');
            string folder = Path.GetDirectoryName(file) ?? ".";
            if (text.StartsWith('~'))
                return Path.GetFullPath(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile) + text[1..]);
            if (Path.IsPathRooted(text)) return Path.GetFullPath(text);
            return Path.GetFullPath(Path.Combine(folder, text));
        }

        public static float Clamp(float value, float? min, float? max)
        {
            if (min is { } lo && value < lo) value = lo;
            if (max is { } hi && value > hi) value = hi;
            return value;
        }

        public static NormalizedPair ClampPair(NormalizedPair pair, ThemePropertySpec spec)
        {
            float Axis(float v, float? max)
            {
                switch (spec.Rule)
                {
                    case PairRule.ZeroAxisAuto when v == 0:
                    case PairRule.OneAxisOnly when v == 0:
                    case PairRule.MinusOneMatchesOther when v == -1:
                        return v;
                    default:
                        return Clamp(v, spec.Min, max);
                }
            }

            if (spec.Rule == PairRule.ZeroAxisAuto && pair.X == 0 && pair.Y == 0 && spec.Min is > 0 and { } min)
                return new NormalizedPair(min, min);
            return new NormalizedPair(Axis(pair.X, spec.Max), Axis(pair.Y, spec.MaxY ?? spec.Max));
        }
    }
}
