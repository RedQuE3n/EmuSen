using System;
using System.Globalization;
using System.Linq;
using System.Text;
using Avalonia;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using EmuSen.LunaP.Media;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // Theme values into LunaP values: fractions into pixels by the axis §4.3 names, colours, enums and strftime - see EmuSen_BigPicture.md §13.2.
    public static class SceneUnits
    {
        public static Color ToColor(ThemeColor c) => Color.FromArgb(c.A, c.R, c.G, c.B);

        public static Color ToColor(ThemeColor? c, Color fallback) => c is { } v ? ToColor(v) : fallback;

        // A fraction of an axis in pixels, kept to hundredths so float noise is not ceiled into a whole pixel by layout rounding.
        public static double Px(double fraction, double axis) => Math.Round(fraction * axis * 100) / 100;

        public static Point ToPoint(NormalizedPair? p, Point fallback = default) => p is { } v ? new Point(v.X, v.Y) : fallback;

        public static Size ToSize(NormalizedPair? p) => p is { } v ? new Size(Math.Max(0, v.X), Math.Max(0, v.Y)) : default;

        public static TextAlignment Horizontal(string? value) => value switch { "center" => TextAlignment.Center, "right" => TextAlignment.Right, _ => TextAlignment.Left };

        public static HorizontalAlignment HorizontalLayout(string? value) => value switch { "center" => HorizontalAlignment.Center, "right" => HorizontalAlignment.Right, _ => HorizontalAlignment.Left };

        public static VerticalAlignment Vertical(string? value, VerticalAlignment fallback = VerticalAlignment.Center) =>
            value switch { "top" => VerticalAlignment.Top, "bottom" => VerticalAlignment.Bottom, "center" => VerticalAlignment.Center, _ => fallback };

        public static LetterCase Case(string? value) => value switch { "uppercase" => LetterCase.Upper, "lowercase" => LetterCase.Lower, "capitalize" => LetterCase.Capitalize, _ => LetterCase.None };

        public static string Cased(string text, string? letterCase) => letterCase switch
        {
            "uppercase" => text.ToUpperInvariant(),
            "lowercase" => text.ToLowerInvariant(),
            "capitalize" => string.Concat(text.Select((c, i) => i == 0 || char.IsWhiteSpace(text[i - 1]) ? char.ToUpperInvariant(c) : c)),
            _ => text,
        };

        public static Orientation Gradient(string? value) => value == "vertical" ? Orientation.Vertical : Orientation.Horizontal;

        public static BitmapInterpolationMode Interpolation(string? value) => value == "nearest" ? BitmapInterpolationMode.None : BitmapInterpolationMode.HighQuality;

        // A background fill: one colour, or a two-stop gradient across the box when the end colour differs.
        public static IBrush? Fill(ThemeColor? start, ThemeColor? end, string? gradientType)
        {
            if (start is not { } a) return null;
            ThemeColor b = end ?? a;
            if (a.A == 0 && b.A == 0) return null;
            if (a == b) return new SolidColorBrush(ToColor(a));
            var brush = new LinearGradientBrush
            {
                StartPoint = new RelativePoint(0, 0, RelativeUnit.Relative),
                EndPoint = gradientType == "vertical" ? new RelativePoint(0, 1, RelativeUnit.Relative) : new RelativePoint(1, 0, RelativeUnit.Relative),
            };
            brush.GradientStops.Add(new GradientStop(ToColor(a), 0));
            brush.GradientStops.Add(new GradientStop(ToColor(b), 1));
            return brush;
        }

        // strftime as THEMES.md's datetime and clock formats write it, into a .NET custom format with the literal text quoted.
        public static string DotNetFormat(string strftime)
        {
            var sb = new StringBuilder();
            var literal = new StringBuilder();
            void Flush()
            {
                if (literal.Length == 0) return;
                sb.Append('\'').Append(literal.ToString().Replace("'", "\\'")).Append('\'');
                literal.Clear();
            }

            for (int i = 0; i < strftime.Length; i++)
            {
                char c = strftime[i];
                if (c != '%' || i + 1 >= strftime.Length)
                {
                    literal.Append(c);
                    continue;
                }

                string? token = strftime[++i] switch
                {
                    'Y' => "yyyy", 'y' => "yy", 'm' => "MM", 'd' => "dd", 'e' => "d", 'H' => "HH", 'I' => "hh", 'M' => "mm", 'S' => "ss",
                    'p' => "tt", 'b' or 'h' => "MMM", 'B' => "MMMM", 'a' => "ddd", 'A' => "dddd", '%' => null, _ => null,
                };
                if (token is null)
                {
                    literal.Append(strftime[i] == '%' ? "%" : "%" + strftime[i]);
                    continue;
                }

                Flush();
                sb.Append(token);
            }

            Flush();
            string format = sb.ToString();
            return format.Length == 1 ? "%" + format : format;
        }

        public static string Format(DateTime value, string strftime)
        {
            try { return value.ToString(DotNetFormat(strftime), CultureInfo.InvariantCulture); }
            catch (FormatException) { return value.ToString("yyyy-MM-dd", CultureInfo.InvariantCulture); }
        }

        // Mistress's own words for a date relative to now, since the ones ES-DE prints are its own resources - §3.6.
        public static string Relative(DateTime then, DateTime now)
        {
            TimeSpan ago = now - then;
            if (ago < TimeSpan.Zero) return "in the future";
            if (ago.TotalMinutes < 1) return "just now";
            if (ago.TotalHours < 1) return Count((int)ago.TotalMinutes, "minute");
            if (ago.TotalDays < 1) return Count((int)ago.TotalHours, "hour");
            if (ago.TotalDays < 30) return Count((int)ago.TotalDays, "day");
            if (ago.TotalDays < 365) return Count((int)(ago.TotalDays / 30), "month");
            return Count((int)(ago.TotalDays / 365), "year");
        }

        private static string Count(int n, string unit) => $"{n} {unit}{(n == 1 ? "" : "s")} ago";

        public static string PlayTime(TimeSpan t) =>
            t.TotalHours >= 1 ? $"{(int)t.TotalHours}h {t.Minutes}m" : t.TotalMinutes >= 1 ? $"{(int)t.TotalMinutes}m" : $"{(int)t.TotalSeconds}s";
    }
}
