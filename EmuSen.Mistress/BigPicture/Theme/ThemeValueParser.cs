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

        // ES-DE's reading of a FLOAT: the leading number after any whitespace, hexadecimal, inf and nan included, else 0 - see EmuSen_BigPicture.md §35.2.
        public static float EsdeFloat(string text, out bool exact)
        {
            int start = SkipCSpace(text, 0);
            int end = NumberEnd(text, start, out double value, out bool plain);
            exact = end > start && plain && text[end..].Trim().Length == 0;
            return end == start ? 0f : (float)value;
        }

        // A value ES-DE could not draw at an unbounded property, such as nan or an infinite position, is taken as 0 - see §35.2.
        public static float Finite(float value) => float.IsFinite(value) ? value : 0f;

        // ES-DE's reading of an UNSIGNED_INTEGER: a C reading with the base from the prefix, kept to 32 bits, 0 when none - see §35.2.
        public static uint EsdeUInt(string text, out bool exact)
        {
            int i = SkipCSpace(text, 0);
            bool negative = false;
            if (i < text.Length && text[i] is '+' or '-') negative = text[i++] == '-';
            int radix = 10;
            if (i < text.Length && text[i] == '0')
            {
                if (i + 2 < text.Length && text[i + 1] is 'x' or 'X' && Uri.IsHexDigit(text[i + 2])) { radix = 16; i += 2; }
                else radix = 8;
            }
            ulong value = 0;
            bool overflow = false;
            for (; i < text.Length && DigitValue(text[i]) is int d && d < radix; i++)
            {
                ulong next = unchecked(value * (ulong)radix + (ulong)d);
                if (value > (ulong.MaxValue - (ulong)d) / (ulong)radix) overflow = true;
                value = next;
            }
            if (overflow) value = ulong.MaxValue;
            if (negative) value = unchecked(0UL - value);
            string trimmed = text.Trim();
            exact = trimmed.Length > 0 && trimmed.All(char.IsAsciiDigit) && (trimmed.Length == 1 || trimmed[0] != '0') && !overflow && value <= uint.MaxValue;
            return unchecked((uint)value);
        }

        // ES-DE's reading of a COLOR: 6 or 8 characters as written, else refused; then read as hexadecimal the C way - see §35.2.
        public static bool EsdeColor(string text, out ThemeColor color, out bool exact)
        {
            color = default;
            exact = false;
            if (text.Length != 6 && text.Length != 8) return false;
            int i = SkipCSpace(text, 0);
            bool negative = false;
            if (i < text.Length && text[i] is '+' or '-') negative = text[i++] == '-';
            if (i + 2 < text.Length && text[i] == '0' && text[i + 1] is 'x' or 'X' && Uri.IsHexDigit(text[i + 2])) i += 2;
            uint value = 0;
            for (; i < text.Length && Uri.IsHexDigit(text[i]); i++) value = unchecked(value * 16 + (uint)DigitValue(text[i])!.Value);
            if (negative) value = unchecked(0U - value);
            color = new ThemeColor(text.Length == 6 ? unchecked((value << 8) | 0xFF) : value);
            exact = text.All(Uri.IsHexDigit);
            return true;
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

        // C's whitespace, which ES-DE's readings skip; char.IsWhiteSpace would take more.
        private static int SkipCSpace(string text, int i)
        {
            while (i < text.Length && text[i] is ' ' or '\t' or '\n' or '\v' or '\f' or '\r') i++;
            return i;
        }

        private static int? DigitValue(char c) => c switch
        {
            >= '0' and <= '9' => c - '0',
            >= 'a' and <= 'z' => c - 'a' + 10,
            >= 'A' and <= 'Z' => c - 'A' + 10,
            _ => null,
        };

        // The end of the longest number at start, as C reads one: decimal with an exponent, hexadecimal with a binary one, inf, infinity, nan.
        private static int NumberEnd(string text, int start, out double value, out bool plain)
        {
            value = 0;
            plain = false;
            int i = start;
            bool negative = false;
            if (i < text.Length && text[i] is '+' or '-') negative = text[i++] == '-';
            string rest = text[i..];
            if (rest.StartsWith("infinity", StringComparison.OrdinalIgnoreCase)) { value = negative ? double.NegativeInfinity : double.PositiveInfinity; return i + 8; }
            if (rest.StartsWith("inf", StringComparison.OrdinalIgnoreCase)) { value = negative ? double.NegativeInfinity : double.PositiveInfinity; return i + 3; }
            if (rest.StartsWith("nan", StringComparison.OrdinalIgnoreCase))
            {
                value = double.NaN;
                int close = rest.Length > 3 && rest[3] == '(' ? rest.IndexOf(')', 4) : -1;
                return close > 0 && rest[4..close].All(c => char.IsAsciiLetterOrDigit(c) || c == '_') ? i + close + 1 : i + 3;
            }
            if (rest.Length > 2 && rest[0] == '0' && rest[1] is 'x' or 'X' && (Uri.IsHexDigit(rest[2]) || (rest[2] == '.' && rest.Length > 3 && Uri.IsHexDigit(rest[3]))))
                return HexEnd(text, i + 2, negative, out value);

            int digits = 0;
            int j = i;
            while (j < text.Length && char.IsAsciiDigit(text[j])) { j++; digits++; }
            if (j < text.Length && text[j] == '.')
            {
                j++;
                while (j < text.Length && char.IsAsciiDigit(text[j])) { j++; digits++; }
            }
            if (digits == 0) return start;
            if (j < text.Length && text[j] is 'e' or 'E')
            {
                int k = j + 1;
                if (k < text.Length && text[k] is '+' or '-') k++;
                int m = k;
                while (m < text.Length && char.IsAsciiDigit(text[m])) m++;
                if (m > k) j = m;
            }
            value = double.Parse(text[start..j], NumberStyles.Float, CultureInfo.InvariantCulture);
            plain = true;
            return j;
        }

        // Hexadecimal digits, an optional point and more, then an optional p exponent in powers of two.
        private static int HexEnd(string text, int i, bool negative, out double value)
        {
            double mantissa = 0;
            int scale = 0;
            while (i < text.Length && Uri.IsHexDigit(text[i])) mantissa = mantissa * 16 + DigitValue(text[i++])!.Value;
            if (i < text.Length && text[i] == '.')
            {
                i++;
                while (i < text.Length && Uri.IsHexDigit(text[i])) { mantissa = mantissa * 16 + DigitValue(text[i++])!.Value; scale -= 4; }
            }
            if (i < text.Length && text[i] is 'p' or 'P')
            {
                int k = i + 1;
                bool minus = false;
                if (k < text.Length && text[k] is '+' or '-') minus = text[k++] == '-';
                int m = k, exponent = 0;
                while (m < text.Length && char.IsAsciiDigit(text[m])) exponent = Math.Min(exponent * 10 + (text[m++] - '0'), 100000);
                if (m > k) { scale += minus ? -exponent : exponent; i = m; }
            }
            value = mantissa * Math.Pow(2, scale);
            if (negative) value = -value;
            return i;
        }

        // ./ is the file's folder, ~ is home, backslashes are separators, nothing is trimmed, and any other relative path is the working directory's - see §35.2.
        public static string ResolvePath(string written, string file)
        {
            string text = written.Replace('\\', '/');
            string folder = Path.GetDirectoryName(file) ?? ".";
            if (text.Length == 0) return Path.GetFullPath(folder);
            if (text.StartsWith("./", StringComparison.Ordinal)) return Path.GetFullPath(folder + "/" + text[2..]);
            if (text.StartsWith('~'))
                return Path.GetFullPath(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile) + text[1..]);
            return Path.GetFullPath(text);
        }

        public static uint ClampUInt(uint value, float? min, float? max)
        {
            if (min is { } lo && value < lo) return (uint)lo;
            if (max is { } hi && value > hi) return (uint)hi;
            return value;
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
