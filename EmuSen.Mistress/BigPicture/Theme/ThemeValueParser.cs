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

        // ES-DE's reading of a BOOLEAN: true when the first character is t, T, y, Y or 1, never an error - see EmuSen_BigPicture.md §31.2.
        public static bool EsdeBool(string text, out bool exact)
        {
            exact = text.ToLowerInvariant() is "true" or "false" or "1" or "0";
            return text.Length > 0 && text[0] is 't' or 'T' or 'y' or 'Y' or '1';
        }

        // ES-DE's reading of a FLOAT: the leading number after any whitespace, 0 when there is none - see EmuSen_BigPicture.md §31.2.
        public static float EsdeFloat(string text, out bool exact)
        {
            int start = 0;
            while (start < text.Length && char.IsWhiteSpace(text[start])) start++;
            int end = NumberEnd(text, start);
            exact = end > start && text[end..].Trim().Length == 0;
            if (end == start || !TryParseFloat(text[start..end], out float value)) return 0f;
            return value;
        }

        // ES-DE's reading of a NORMALIZED_PAIR: split at the first space, each side read as a FLOAT; false when there is no space - see §31.2.
        public static bool EsdePair(string text, out NormalizedPair pair, out bool exact)
        {
            pair = default;
            exact = false;
            int space = text.IndexOf(' ');
            if (space < 0) return false;
            float x = EsdeFloat(text[..space], out bool xExact);
            float y = EsdeFloat(text[(space + 1)..], out bool yExact);
            pair = new NormalizedPair(x, y);
            exact = xExact && yExact && TryParsePair(text, out _);
            return true;
        }

        // The end of the longest decimal number at start: sign, digits, a point, digits, an exponent.
        private static int NumberEnd(string text, int start)
        {
            int i = start;
            if (i < text.Length && text[i] is '+' or '-') i++;
            int digits = 0;
            while (i < text.Length && char.IsAsciiDigit(text[i])) { i++; digits++; }
            if (i < text.Length && text[i] == '.')
            {
                i++;
                while (i < text.Length && char.IsAsciiDigit(text[i])) { i++; digits++; }
            }
            if (digits == 0) return start;
            if (i < text.Length && text[i] is 'e' or 'E')
            {
                int j = i + 1;
                if (j < text.Length && text[j] is '+' or '-') j++;
                int k = j;
                while (k < text.Length && char.IsAsciiDigit(text[k])) k++;
                if (k > j) i = k;
            }
            return i;
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
