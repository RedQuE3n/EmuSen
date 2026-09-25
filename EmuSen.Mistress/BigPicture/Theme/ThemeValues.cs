using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;

namespace EmuSen.Mistress.BigPicture.Theme
{
    // THEMES.md's seven property data types - see EmuSen_BigPicture.md §12.2.
    public enum ThemePropertyType { NormalizedPair, Path, Boolean, Color, UnsignedInteger, Float, String }

    // Two fractions of the parent, which is nearly always the screen; y points down.
    public readonly record struct NormalizedPair(float X, float Y)
    {
        public override string ToString() => string.Create(CultureInfo.InvariantCulture, $"{X:0.######} {Y:0.######}");
    }

    // An RGBA colour, as THEMES.md writes it: 6 hex digits are opaque, 8 carry alpha last.
    public readonly record struct ThemeColor(uint Rgba)
    {
        public byte R => (byte)(Rgba >> 24);
        public byte G => (byte)(Rgba >> 16);
        public byte B => (byte)(Rgba >> 8);
        public byte A => (byte)Rgba;

        public static bool TryParse(string text, out ThemeColor color)
        {
            color = default;
            if ((text.Length != 6 && text.Length != 8) || !text.All(Uri.IsHexDigit)) return false;
            uint value = uint.Parse(text, NumberStyles.HexNumber, CultureInfo.InvariantCulture);
            color = new ThemeColor(text.Length == 6 ? (value << 8) | 0xFF : value);
            return true;
        }

        public override string ToString() => Rgba.ToString("X8", CultureInfo.InvariantCulture);
    }

    // A file a property names: as written, as resolved, and whether it was built from a variable - see EmuSen_BigPicture.md §12.2.
    public sealed record ThemePath(string Written, string Absolute, bool FromVariable, bool Exists)
    {
        public override string ToString() => Absolute;
    }

    // One typed, resolved property value; the subtype is the documented type.
    public abstract record ThemeValue
    {
        public abstract ThemePropertyType Type { get; }
    }

    public sealed record PairValue(NormalizedPair Value) : ThemeValue
    {
        public override ThemePropertyType Type => ThemePropertyType.NormalizedPair;
        public override string ToString() => Value.ToString();
    }

    public sealed record PathValue(ThemePath Value) : ThemeValue
    {
        public override ThemePropertyType Type => ThemePropertyType.Path;
        public override string ToString() => Value.ToString();
    }

    public sealed record BoolValue(bool Value) : ThemeValue
    {
        public override ThemePropertyType Type => ThemePropertyType.Boolean;
        public override string ToString() => Value ? "true" : "false";
    }

    public sealed record ColorValue(ThemeColor Value) : ThemeValue
    {
        public override ThemePropertyType Type => ThemePropertyType.Color;
        public override string ToString() => Value.ToString();
    }

    public sealed record UIntValue(uint Value) : ThemeValue
    {
        public override ThemePropertyType Type => ThemePropertyType.UnsignedInteger;
        public override string ToString() => Value.ToString(CultureInfo.InvariantCulture);
    }

    public sealed record FloatValue(float Value) : ThemeValue
    {
        public override ThemePropertyType Type => ThemePropertyType.Float;
        public override string ToString() => Value.ToString("0.######", CultureInfo.InvariantCulture);
    }

    public sealed record StringValue(string Value) : ThemeValue
    {
        public override ThemePropertyType Type => ThemePropertyType.String;
        public override string ToString() => Value;
    }

    // A STRING property that holds a list, delimited by commas or whitespace (imageType, slots, entries).
    public sealed record ListValue(IReadOnlyList<string> Items) : ThemeValue
    {
        public override ThemePropertyType Type => ThemePropertyType.String;
        public override string ToString() => string.Join(",", Items);
        public bool Equals(ListValue? other) => other is not null && Items.SequenceEqual(other.Items);
        public override int GetHashCode() => Items.Count;
    }

    // A PATH property keyed by an attribute, such as customBadgeIcon badge="favorite".
    public sealed record KeyedPathsValue(IReadOnlyDictionary<string, ThemePath> Paths) : ThemeValue
    {
        public override ThemePropertyType Type => ThemePropertyType.Path;
        public override string ToString() => string.Join(",", Paths.OrderBy(p => p.Key, StringComparer.Ordinal).Select(p => $"{p.Key}={p.Value.Absolute}"));
        public bool Equals(KeyedPathsValue? other) => other is not null && Paths.Count == other.Paths.Count && Paths.All(p => other.Paths.TryGetValue(p.Key, out ThemePath? o) && o == p.Value);
        public override int GetHashCode() => Paths.Count;
    }

    // The documented separators for name lists and value lists: commas, tabs, spaces or line breaks.
    public static class ThemeLists
    {
        private static readonly char[] Separators = [',', ' ', '\t', '\r', '\n'];

        public static IReadOnlyList<string> Split(string? text) =>
            text is null ? [] : text.Split(Separators, StringSplitOptions.RemoveEmptyEntries);
    }
}
