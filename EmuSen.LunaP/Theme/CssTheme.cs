using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Markup.Xaml.MarkupExtensions;
using Avalonia.Media;
using Avalonia.Media.Immutable;
using Avalonia.Styling;
using EmuSen.LunaP.Controls;

namespace EmuSen.LunaP.Theme
{
    // What a .css theme compiled to: palette resources, control styles, and every declaration that was skipped.
    public sealed class CssThemeResult
    {
        public ResourceDictionary Resources { get; } = new();

        public Styles Styles { get; } = new();

        // Not fatal by design: a theme written against a newer LunaP still loads - see EmuSen_LunaP.md §12.2.
        public List<string> Warnings { get; } = new();
    }

    // The restricted CSS a theme may be written in - see EmuSen_LunaP.md §12.2 for the grammar and its limits.
    public static class CssTheme
    {
        public const string RootSelector = ":root";
        public const string TokenPrefix = "--luna-";

        // A syntax error refuses the whole file, the way a malformed .axaml theme already does.
        public static CssThemeResult Parse(string css) => new Parser(css ?? string.Empty).Run();

        // Which control each element name selects, and the pseudo-classes and template parts it admits.
        private sealed record ElementSpec(
            Type Target,
            IReadOnlyDictionary<string, string> Classes,
            IReadOnlyDictionary<string, PartSpec> Parts);

        private sealed record PartSpec(Type Target, string Name);

        private static readonly IReadOnlyDictionary<string, string> NoClasses = new Dictionary<string, string>();
        private static readonly IReadOnlyDictionary<string, PartSpec> NoParts = new Dictionary<string, PartSpec>();

        private static readonly IReadOnlyDictionary<string, ElementSpec> Elements = BuildElements();

        // Named after the control, so a rename moves the CSS name with it rather than leaving a selector that matches nothing.
        private static IReadOnlyDictionary<string, ElementSpec> BuildElements()
        {
            var specs = new[]
            {
                Element(typeof(SectionHeader)),
                Element(typeof(HintText)),
                Element(typeof(MonoText)),
                Element(typeof(MeterRow),
                    classes: new Dictionary<string, string> { ["nominal"] = ":nominal", ["busy"] = ":busy", ["hot"] = ":hot" },
                    parts: new Dictionary<string, PartSpec> { ["bar"] = new(typeof(ProgressBar), "PART_Bar") }),
                Element(typeof(MeterList)),
                Element(typeof(FieldRow)),
                Element(typeof(PathPickerRow)),
                Element(typeof(ButtonBar)),
                Element(typeof(StatusBar)),
                Element(typeof(FilterBar), parts: new Dictionary<string, PartSpec>
                {
                    ["search"] = new(typeof(TextBox), "PART_Search"),
                    ["facet"] = new(typeof(TextBlock), "PART_Facet"),
                }),
                Element(typeof(ConsolePane), parts: new Dictionary<string, PartSpec>
                {
                    ["output"] = new(typeof(SelectableTextBlock), "PART_Output"),
                    ["input"] = new(typeof(TextBox), "PART_Input"),
                    ["prompt"] = new(typeof(TextBlock), "PART_Prompt"),
                }),
                Element(typeof(RgbaImageView)),
                Element(typeof(LunaSwitch)),
                Element(typeof(Dropdown)),
                Element(typeof(Tabs)),
            };

            return specs.ToDictionary(s => Kebab(s.Target.Name), s => s, StringComparer.Ordinal);
        }

        private static ElementSpec Element(
            Type target,
            IReadOnlyDictionary<string, string>? classes = null,
            IReadOnlyDictionary<string, PartSpec>? parts = null) =>
            new(target, classes ?? NoClasses, parts ?? NoParts);

        private static readonly IReadOnlyDictionary<string, string> Properties = new Dictionary<string, string>(StringComparer.Ordinal)
        {
            ["color"] = "Foreground",
            ["background"] = "Background",
            ["background-color"] = "Background",
            ["font-family"] = "FontFamily",
            ["font-size"] = "FontSize",
            ["font-weight"] = "FontWeight",
        };

        private sealed class Parser
        {
            private readonly string _css;
            private readonly CssThemeResult _result = new();

            public Parser(string css) => _css = Decommented(css);

            public CssThemeResult Run()
            {
                foreach (Block block in Blocks())
                {
                    if (string.Equals(block.Selector, RootSelector, StringComparison.Ordinal)) Palette(block);
                    else Rules(block);
                }

                return _result;
            }

            private readonly record struct Block(string Selector, int BodyStart, int BodyEnd, int Line);

            private IEnumerable<Block> Blocks()
            {
                int i = 0;
                while (true)
                {
                    while (i < _css.Length && char.IsWhiteSpace(_css[i])) i++;
                    if (i >= _css.Length) yield break;

                    int open = _css.IndexOf('{', i);
                    if (open < 0) throw Fail(LineOf(i), "a selector with no '{'");

                    string selector = _css[i..open].Trim();
                    if (selector.Length == 0) throw Fail(LineOf(open), "a rule with no selector");

                    // Checked before the brace pairing below, since an at-rule is the construct whose body nests.
                    if (selector.StartsWith("@", StringComparison.Ordinal))
                        throw Fail(LineOf(i), $"at-rules are not supported ('{selector}')");

                    int close = _css.IndexOf('}', open + 1);
                    if (close < 0) throw Fail(LineOf(open), "a rule with no '}'");

                    int nested = _css.IndexOf('{', open + 1);
                    if (nested >= 0 && nested < close) throw Fail(LineOf(nested), "nested rules are not supported");

                    yield return new Block(selector, open + 1, close, LineOf(i));
                    i = close + 1;
                }
            }

            private IEnumerable<(string Name, string Value, int Line)> Declarations(Block block)
            {
                int start = block.BodyStart;
                for (int i = block.BodyStart; i <= block.BodyEnd; i++)
                {
                    if (i < block.BodyEnd && _css[i] != ';') continue;

                    string text = _css[start..i].Trim();
                    int at = start;
                    start = i + 1;
                    if (text.Length == 0) continue;

                    // The first colon separates name from value; any later one belongs to a url or a var().
                    int colon = text.IndexOf(':');
                    if (colon <= 0) throw Fail(LineOf(at), $"'{text}' is not a 'name: value' declaration");

                    yield return (text[..colon].Trim(), text[(colon + 1)..].Trim(), LineOf(at));
                }
            }

            private void Palette(Block block)
            {
                foreach ((string name, string value, int line) in Declarations(block))
                {
                    if (!name.StartsWith(TokenPrefix, StringComparison.Ordinal))
                    {
                        Warn(line, $"'{name}' is not a palette token; ':root' takes only '{TokenPrefix}…' properties");
                        continue;
                    }

                    string key = ResourceKey(name);
                    if (key.EndsWith("Size", StringComparison.Ordinal))
                    {
                        if (TryNumber(value, out double number)) _result.Resources[key] = number;
                        else Warn(line, $"'{value}' is not a number");
                        continue;
                    }

                    if (key.EndsWith("Font", StringComparison.Ordinal))
                    {
                        _result.Resources[key] = new FontFamily(FontList(value));
                        continue;
                    }

                    if (!TryColour(value, out Color colour))
                    {
                        Warn(line, $"'{value}' is not a colour");
                        continue;
                    }

                    // Palette.axaml spells every colour twice, as a Color and as a brush; a theme overriding one must override both.
                    string brushKey = key.EndsWith("Color", StringComparison.Ordinal) ? key[..^"Color".Length] : key;
                    _result.Resources[brushKey + "Color"] = colour;
                    _result.Resources[brushKey] = new ImmutableSolidColorBrush(colour);
                }
            }

            private void Rules(Block block)
            {
                foreach (string text in block.Selector.Split(','))
                {
                    string selectorText = text.Trim();
                    if (selectorText.Length == 0) continue;

                    if (!TryCompile(selectorText, out Selector? selector, out Type? target, out string why))
                    {
                        Warn(block.Line, $"'{selectorText}': {why}");
                        continue;
                    }

                    var style = new Style { Selector = selector };
                    foreach ((string name, string value, int line) in Declarations(block))
                    {
                        if (!Properties.TryGetValue(name, out string? propertyName))
                        {
                            Warn(line, $"'{name}' is not a themeable property");
                            continue;
                        }

                        AvaloniaProperty? property = AvaloniaPropertyRegistry.Instance.FindRegistered(target!, propertyName);
                        if (property is null)
                        {
                            Warn(line, $"'{name}' does not apply to '{selectorText}'");
                            continue;
                        }

                        if (!TryValue(property, value, out object? converted, out string reason))
                        {
                            Warn(line, $"'{value}': {reason}");
                            continue;
                        }

                        style.Setters.Add(new Setter(property, converted));
                    }

                    if (style.Setters.Count > 0) _result.Styles.Add(style);
                }
            }

            private static bool TryCompile(string text, out Selector? selector, out Type? target, out string why)
            {
                selector = null;
                target = null;

                string[] words = text.Split(' ', StringSplitOptions.RemoveEmptyEntries);
                if (words.Length > 2)
                {
                    why = "only 'element' or 'element part' selectors are supported";
                    return false;
                }

                string[] head = words[0].Split('.');
                if (!Elements.TryGetValue(head[0], out ElementSpec? spec))
                {
                    why = $"'{head[0]}' is not a LunaP control";
                    return false;
                }

                Selector built = ((Selector?)null).OfType(spec.Target);
                foreach (string name in head.Skip(1))
                {
                    if (!spec.Classes.TryGetValue(name, out string? pseudo))
                    {
                        why = $"'{head[0]}' has no state '.{name}'";
                        return false;
                    }

                    built = built.Class(pseudo);
                }

                target = spec.Target;
                if (words.Length == 2)
                {
                    string partName = words[1].TrimStart('.');
                    if (!spec.Parts.TryGetValue(partName, out PartSpec? part))
                    {
                        why = $"'{head[0]}' has no part '{words[1]}'";
                        return false;
                    }

                    built = built.Template().OfType(part.Target).Name(part.Name);
                    target = part.Target;
                }

                selector = built;
                why = string.Empty;
                return true;
            }

            private static bool TryValue(AvaloniaProperty property, string text, out object? value, out string why)
            {
                value = null;
                why = string.Empty;

                // A rule may point at a palette token rather than restate its colour, and then it follows the token.
                if (text.StartsWith("var(", StringComparison.Ordinal) && text.EndsWith(")", StringComparison.Ordinal))
                {
                    string token = text[4..^1].Trim();
                    if (!token.StartsWith(TokenPrefix, StringComparison.Ordinal))
                    {
                        why = $"var() takes a '{TokenPrefix}…' token";
                        return false;
                    }

                    value = new DynamicResourceExtension(ResourceKey(token));
                    return true;
                }

                Type type = property.PropertyType;
                if (typeof(IBrush).IsAssignableFrom(type))
                {
                    if (!TryColour(text, out Color colour))
                    {
                        why = "not a colour";
                        return false;
                    }

                    value = new ImmutableSolidColorBrush(colour);
                    return true;
                }

                if (type == typeof(double))
                {
                    if (!TryNumber(text, out double number))
                    {
                        why = "not a number";
                        return false;
                    }

                    value = number;
                    return true;
                }

                if (type == typeof(FontFamily))
                {
                    value = new FontFamily(FontList(text));
                    return true;
                }

                if (type == typeof(FontWeight))
                {
                    if (!Enum.TryParse(text, ignoreCase: true, out FontWeight weight))
                    {
                        why = "not a font weight";
                        return false;
                    }

                    value = weight;
                    return true;
                }

                why = $"'{type.Name}' values cannot be written in a theme";
                return false;
            }

            private void Warn(int line, string message) => _result.Warnings.Add($"line {line}: {message}.");

            private int LineOf(int index)
            {
                int line = 1;
                for (int i = 0; i < index && i < _css.Length; i++)
                {
                    if (_css[i] == '\n') line++;
                }

                return line;
            }

            // Comments become whitespace rather than disappearing, so a reported line number still matches the file.
            private static string Decommented(string css)
            {
                var sb = new StringBuilder(css.Length);
                for (int i = 0; i < css.Length; i++)
                {
                    if (css[i] != '/' || i + 1 >= css.Length || css[i + 1] != '*')
                    {
                        sb.Append(css[i]);
                        continue;
                    }

                    int end = css.IndexOf("*/", i + 2, StringComparison.Ordinal);
                    if (end < 0) throw new FormatException("unterminated comment");

                    for (int j = i; j < end + 2; j++) sb.Append(css[j] == '\n' ? '\n' : ' ');
                    i = end + 1;
                }

                return sb.ToString();
            }

            private static FormatException Fail(int line, string what) => new($"line {line}: {what}.");
        }

        // --luna-section-header becomes LunaSectionHeader, the key Palette.axaml already spells.
        private static string ResourceKey(string token)
        {
            var sb = new StringBuilder();
            foreach (string word in token[2..].Split('-', StringSplitOptions.RemoveEmptyEntries))
            {
                sb.Append(char.ToUpperInvariant(word[0])).Append(word[1..].ToLowerInvariant());
            }

            return sb.ToString();
        }

        private static string Kebab(string name)
        {
            var sb = new StringBuilder();
            for (int i = 0; i < name.Length; i++)
            {
                if (char.IsUpper(name[i]) && i > 0) sb.Append('-');
                sb.Append(char.ToLowerInvariant(name[i]));
            }

            return sb.ToString();
        }

        private static string FontList(string text) =>
            string.Join(',', text.Split(',').Select(part => part.Trim().Trim('"', '\'')).Where(part => part.Length > 0));

        private static bool TryNumber(string text, out double value)
        {
            string trimmed = text.EndsWith("px", StringComparison.OrdinalIgnoreCase) ? text[..^2] : text;
            return double.TryParse(trimmed.Trim(), NumberStyles.Float, CultureInfo.InvariantCulture, out value);
        }

        // CSS channel order, not Avalonia's: #RRGGBBAA and #RGBA put alpha last, which is what someone writing CSS expects.
        private static bool TryColour(string text, out Color colour)
        {
            colour = default;
            text = text.Trim();

            if (text.StartsWith("rgb(", StringComparison.OrdinalIgnoreCase) ||
                text.StartsWith("rgba(", StringComparison.OrdinalIgnoreCase))
            {
                return TryRgb(text, out colour);
            }

            if (text.Length is 9 or 5 && text[0] == '#')
            {
                string digits = text[1..];
                int step = digits.Length / 4;
                if (!Nibbles(digits, 0, step, out byte r) || !Nibbles(digits, step, step, out byte g) ||
                    !Nibbles(digits, step * 2, step, out byte b) || !Nibbles(digits, step * 3, step, out byte a))
                {
                    return false;
                }

                colour = Color.FromArgb(a, r, g, b);
                return true;
            }

            return Color.TryParse(text, out colour);
        }

        private static bool Nibbles(string digits, int start, int length, out byte value)
        {
            value = 0;
            if (!byte.TryParse(digits.AsSpan(start, length), NumberStyles.HexNumber, CultureInfo.InvariantCulture, out byte parsed))
                return false;

            value = length == 1 ? (byte)(parsed * 17) : parsed;
            return true;
        }

        private static bool TryRgb(string text, out Color colour)
        {
            colour = default;
            int open = text.IndexOf('(');
            if (!text.EndsWith(")", StringComparison.Ordinal)) return false;

            string[] parts = text[(open + 1)..^1].Split(',', StringSplitOptions.TrimEntries | StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length is not (3 or 4)) return false;

            var channels = new byte[3];
            for (int i = 0; i < 3; i++)
            {
                if (!byte.TryParse(parts[i], NumberStyles.Integer, CultureInfo.InvariantCulture, out channels[i])) return false;
            }

            byte alpha = 255;
            if (parts.Length == 4)
            {
                // CSS spells alpha 0..1, unlike every other channel.
                if (!double.TryParse(parts[3], NumberStyles.Float, CultureInfo.InvariantCulture, out double fraction)) return false;
                alpha = (byte)Math.Round(Math.Clamp(fraction, 0, 1) * 255);
            }

            colour = Color.FromArgb(alpha, channels[0], channels[1], channels[2]);
            return true;
        }
    }
}
