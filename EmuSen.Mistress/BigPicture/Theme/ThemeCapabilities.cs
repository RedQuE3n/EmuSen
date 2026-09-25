using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using System.Xml;
using System.Xml.Linq;

namespace EmuSen.Mistress.BigPicture.Theme
{
    public enum TransitionAnimation { Instant, Slide, Fade }

    // A variant trigger: noVideos or noMedia, the media it looks for, and the variant it switches to.
    public sealed record VariantOverride(string Trigger, IReadOnlyList<string> MediaTypes, string UseVariant);

    public sealed record ThemeVariant(string Name, IReadOnlyDictionary<string, string> Labels, bool Selectable, IReadOnlyList<VariantOverride> Overrides);

    public sealed record ThemeColorScheme(string Name, IReadOnlyDictionary<string, string> Labels);

    // A transitions profile; kinds the file leaves out are filled by THEMES.md's rule - see EmuSen_BigPicture.md §12.2.
    public sealed record TransitionProfile(string Name, IReadOnlyDictionary<string, string> Labels, bool Selectable,
        TransitionAnimation SystemToSystem, TransitionAnimation SystemToGamelist, TransitionAnimation GamelistToGamelist,
        TransitionAnimation GamelistToSystem, TransitionAnimation StartupToSystem, TransitionAnimation StartupToGamelist);

    // What capabilities.xml declares, in its own order except where THEMES.md fixes the menu order.
    public sealed record ThemeCapabilities(
        string Directory,
        string ThemeName,
        IReadOnlyList<ThemeVariant> Variants,
        IReadOnlyList<ThemeColorScheme> ColorSchemes,
        IReadOnlyList<string> FontSizes,
        IReadOnlyList<string> Languages,
        IReadOnlyList<string> AspectRatios,
        IReadOnlyList<TransitionProfile> Transitions,
        IReadOnlyList<string> SuppressedTransitions,
        IReadOnlyList<ThemeDiagnostic> Diagnostics)
    {
        // THEMES.md's menu order for font sizes.
        public static IReadOnlyList<string> FontSizeOrder { get; } = ["medium", "large", "small", "x-large", "x-small"];

        // The fixed aspect-ratio table, horizontal names in menu order; each but 1:1 has a _vertical form.
        public static IReadOnlyList<(string Name, double Width, double Height)> AspectRatioTable { get; } =
        [
            ("16:9", 16, 9), ("16:10", 16, 10), ("3:2", 3, 2), ("4:3", 4, 3), ("5:3", 5, 3), ("5:4", 5, 4),
            ("8:7", 8, 7), ("19.5:9", 19.5, 9), ("20:9", 20, 9), ("21:9", 21, 9), ("32:9", 32, 9), ("1:1", 1, 1),
        ];

        public static IReadOnlyList<string> LanguageTable { get; } =
        [
            "en_US", "en_GB", "bs_BA", "ca_ES", "de_DE", "es_ES", "fr_FR", "hr_HR", "it_IT", "nl_NL", "pl_PL",
            "pt_BR", "pt_PT", "ro_RO", "ru_RU", "sr_RS", "sv_SE", "ja_JP", "ko_KR", "zh_CN", "zh_TW",
        ];

        public static IReadOnlyList<string> BuiltInTransitions { get; } = ["builtin-instant", "builtin-slide", "builtin-fade"];

        public static IReadOnlyList<string> MediaTypes { get; } =
            ["miximage", "marquee", "screenshot", "titlescreen", "cover", "backcover", "3dbox", "physicalmedia", "fanart", "video"];

        public bool IsDeclaredVariant(string name) => Variants.Any(v => v.Name == name);

        public ThemeVariant? FindVariant(string name) => Variants.FirstOrDefault(v => v.Name == name);

        // Width over height for a table name, vertical forms inverted; null for a name not in the table.
        public static double? Ratio(string name)
        {
            bool vertical = name.EndsWith("_vertical", StringComparison.Ordinal);
            string horizontal = vertical ? name[..^"_vertical".Length] : name;
            foreach ((string n, double w, double h) in AspectRatioTable)
            {
                if (n != horizontal) continue;
                if (vertical && n == "1:1") return null;
                return vertical ? h / w : w / h;
            }
            return null;
        }

        public static bool IsVertical(string? aspectRatio) => aspectRatio is not null && aspectRatio.EndsWith("_vertical", StringComparison.Ordinal);
    }

    // Reads capabilities.xml; without it ES-DE does not load the theme - see EmuSen_BigPicture.md §12.2.
    public static class ThemeCapabilitiesReader
    {
        public const string FileName = "capabilities.xml";

        public static ThemeCapabilities Read(string themeDirectory)
        {
            var diagnostics = new ThemeDiagnostics();
            string file = System.IO.Path.Combine(themeDirectory, FileName);
            string fallbackName = System.IO.Path.GetFileName(System.IO.Path.TrimEndingDirectorySeparator(themeDirectory)).ToUpperInvariant();

            if (!File.Exists(file))
            {
                diagnostics.Add(ThemeSeverity.Error, ThemeDiagnosticCode.CapabilitiesMissing, file, 0, "capabilities.xml is missing, so the theme is not loaded");
                return Empty(themeDirectory, fallbackName, diagnostics);
            }

            string text = File.ReadAllText(file);
            // An empty file, or one holding only a comment, is the documented way to declare nothing.
            if (Regex.Replace(text, @"<!--.*?-->|<\?xml.*?\?>", "", RegexOptions.Singleline).Trim().Length == 0)
                return Empty(themeDirectory, fallbackName, diagnostics);

            XDocument document;
            try
            {
                document = XDocument.Parse(text, LoadOptions.SetLineInfo);
            }
            catch (XmlException e)
            {
                diagnostics.Add(ThemeSeverity.Error, ThemeDiagnosticCode.MalformedXml, file, e.LineNumber, e.Message);
                return Empty(themeDirectory, fallbackName, diagnostics);
            }

            XElement root = document.Root!;
            if (root.Name.LocalName != "themeCapabilities")
                diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.WrongRoot, file, Line(root), $"root is <{root.Name.LocalName}>, not <themeCapabilities>");

            string themeName = root.Element("themeName")?.Value.Trim() is { Length: > 0 } n ? n : fallbackName;
            var variants = new List<ThemeVariant>();
            var schemes = new List<ThemeColorScheme>();
            var fontSizes = new List<string>();
            var languages = new List<string>();
            var aspectRatios = new List<string>();
            var transitions = new List<TransitionProfile>();
            var suppressed = new List<string>();

            foreach (XElement child in root.Elements())
            {
                switch (child.Name.LocalName)
                {
                    case "themeName":
                        break;
                    case "aspectRatio":
                        AddChecked(aspectRatios, child, ThemeCapabilities.Ratio(child.Value.Trim()) is not null, "is not in the aspect-ratio table", diagnostics, file);
                        break;
                    case "fontSize":
                        AddChecked(fontSizes, child, ThemeCapabilities.FontSizeOrder.Contains(child.Value.Trim()), "is not one of the five font sizes", diagnostics, file);
                        break;
                    case "language":
                        AddChecked(languages, child, ThemeCapabilities.LanguageTable.Contains(child.Value.Trim()), "is not a supported language", diagnostics, file);
                        break;
                    case "colorScheme":
                        if (NamedEntry(child, schemes.Select(s => s.Name), "colour scheme", diagnostics, file) is { } schemeName)
                            schemes.Add(new ThemeColorScheme(schemeName, Labels(child)));
                        break;
                    case "variant":
                        if (NamedEntry(child, variants.Select(v => v.Name), "variant", diagnostics, file) is { } variantName)
                            variants.Add(ReadVariant(child, variantName, diagnostics, file));
                        break;
                    case "transitions":
                        if (NamedEntry(child, transitions.Select(t => t.Name), "transitions profile", diagnostics, file) is { } profileName)
                        {
                            if (ThemeCapabilities.BuiltInTransitions.Contains(profileName))
                                diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.Reserved, file, Line(child), $"transitions name \"{profileName}\" is reserved");
                            else if (ReadTransitions(child, profileName, diagnostics, file) is { } profile)
                                transitions.Add(profile);
                        }
                        break;
                    case "suppressTransitionProfiles":
                        foreach (XElement entry in child.Elements("entry"))
                        {
                            string value = entry.Value.Trim();
                            if (ThemeCapabilities.BuiltInTransitions.Contains(value)) suppressed.Add(value);
                            else diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.InvalidValue, file, Line(entry), $"\"{value}\" is not a built-in transitions profile");
                        }
                        break;
                    default:
                        diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.UnknownTag, file, Line(child), $"<{child.Name.LocalName}> is not a capabilities tag");
                        break;
                }
            }

            if (languages.Count > 0 && !languages.Contains("en_US"))
            {
                diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.LanguageWithoutEnglish, file, 0, "languages are declared without en_US, so the language configuration is not loaded");
                languages.Clear();
            }

            // Aspect ratios are listed in the table's order whatever order the file uses, a row's vertical form after its horizontal.
            List<string> orderedRatios = aspectRatios
                .OrderBy(IndexInTable)
                .ThenBy(r => ThemeCapabilities.IsVertical(r) ? 1 : 0)
                .ToList();
            List<string> orderedFontSizes = fontSizes.OrderBy(f => ((List<string>)[.. ThemeCapabilities.FontSizeOrder]).IndexOf(f)).ToList();

            return new ThemeCapabilities(themeDirectory, themeName, variants, schemes, orderedFontSizes, languages, orderedRatios, transitions, suppressed, diagnostics.Items);
        }

        private static int IndexInTable(string ratio)
        {
            string horizontal = ThemeCapabilities.IsVertical(ratio) ? ratio[..^"_vertical".Length] : ratio;
            for (int i = 0; i < ThemeCapabilities.AspectRatioTable.Count; i++)
                if (ThemeCapabilities.AspectRatioTable[i].Name == horizontal) return i;
            return int.MaxValue;
        }

        private static ThemeCapabilities Empty(string directory, string name, ThemeDiagnostics diagnostics) =>
            new(directory, name, [], [], [], [], [], [], [], diagnostics.Items);

        private static void AddChecked(List<string> list, XElement element, bool valid, string problem, ThemeDiagnostics diagnostics, string file)
        {
            string value = element.Value.Trim();
            if (!valid)
                diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.InvalidValue, file, Line(element), $"<{element.Name.LocalName}>{value}</{element.Name.LocalName}> {problem} and is not loaded");
            else if (list.Contains(value))
                diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.Duplicate, file, Line(element), $"{element.Name.LocalName} \"{value}\" is declared twice");
            else
                list.Add(value);
        }

        private static string? NamedEntry(XElement element, IEnumerable<string> existing, string what, ThemeDiagnostics diagnostics, string file)
        {
            string? name = element.Attribute("name")?.Value.Trim();
            if (string.IsNullOrEmpty(name))
            {
                diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.MissingName, file, Line(element), $"a {what} has no name and is not loaded");
                return null;
            }
            if (existing.Contains(name))
            {
                diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.Duplicate, file, Line(element), $"{what} \"{name}\" is declared twice; the second is not loaded");
                return null;
            }
            if (what == "variant" && name == "all")
            {
                diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.Reserved, file, Line(element), "\"all\" is a reserved variant name");
                return null;
            }
            return name;
        }

        // A label without a language attribute is en_US's.
        private static IReadOnlyDictionary<string, string> Labels(XElement element)
        {
            var labels = new Dictionary<string, string>(StringComparer.Ordinal);
            foreach (XElement label in element.Elements("label"))
                labels[label.Attribute("language")?.Value ?? "en_US"] = label.Value.Trim();
            return labels;
        }

        private static ThemeVariant ReadVariant(XElement element, string name, ThemeDiagnostics diagnostics, string file)
        {
            // THEMES.md gives no default for a variant's selectable; true is this loader's - see EmuSen_BigPicture.md §12.4.
            bool selectable = ReadBool(element.Element("selectable"), true, diagnostics, file);
            var overrides = new List<VariantOverride>();
            foreach (XElement o in element.Elements("override"))
            {
                List<XElement> triggers = o.Elements("trigger").ToList();
                if (triggers.Count != 1)
                {
                    diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.InvalidValue, file, Line(o), $"an override needs exactly one trigger, not {triggers.Count}, and is ignored");
                    continue;
                }
                string trigger = triggers[0].Value.Trim();
                if (trigger != "noVideos" && trigger != "noMedia")
                {
                    diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.InvalidValue, file, Line(o), $"trigger \"{trigger}\" is not noVideos or noMedia");
                    continue;
                }
                string? use = o.Element("useVariant")?.Value.Trim();
                if (string.IsNullOrEmpty(use))
                {
                    diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.InvalidValue, file, Line(o), "an override has no useVariant");
                    continue;
                }
                IReadOnlyList<string> media = ["video"];
                if (trigger == "noMedia")
                {
                    media = ThemeLists.Split(o.Element("mediaType")?.Value);
                    if (media.Count == 0) media = ["miximage"];
                    foreach (string m in media.Where(m => !ThemeCapabilities.MediaTypes.Contains(m)))
                        diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.InvalidValue, file, Line(o), $"mediaType \"{m}\" is not a media type");
                    media = media.Where(ThemeCapabilities.MediaTypes.Contains).ToArray();
                }
                overrides.Add(new VariantOverride(trigger, media, use));
            }
            return new ThemeVariant(name, Labels(element), selectable, overrides);
        }

        private static TransitionProfile? ReadTransitions(XElement element, string name, ThemeDiagnostics diagnostics, string file)
        {
            TransitionAnimation? Kind(string tag)
            {
                XElement? e = element.Element(tag);
                if (e is null) return null;
                switch (e.Value.Trim())
                {
                    case "instant": return TransitionAnimation.Instant;
                    case "slide": return TransitionAnimation.Slide;
                    case "fade": return TransitionAnimation.Fade;
                    default:
                        diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.InvalidValue, file, Line(e), $"<{tag}> \"{e.Value.Trim()}\" is not instant, slide or fade");
                        return null;
                }
            }

            TransitionAnimation? s2s = Kind("systemToSystem"), s2g = Kind("systemToGamelist"), g2g = Kind("gamelistToGamelist"),
                g2s = Kind("gamelistToSystem"), u2s = Kind("startupToSystem"), u2g = Kind("startupToGamelist");
            if (s2s is null && s2g is null && g2g is null && g2s is null && u2s is null && u2g is null)
            {
                diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.InvalidValue, file, Line(element), $"transitions \"{name}\" defines none of the six types and is not loaded");
                return null;
            }
            TransitionAnimation systemToSystem = s2s ?? TransitionAnimation.Instant, gamelistToGamelist = g2g ?? TransitionAnimation.Instant;
            return new TransitionProfile(name, Labels(element), ReadBool(element.Element("selectable"), true, diagnostics, file),
                systemToSystem, s2g ?? TransitionAnimation.Instant, gamelistToGamelist, g2s ?? TransitionAnimation.Instant,
                u2s ?? systemToSystem, u2g ?? gamelistToGamelist);
        }

        private static bool ReadBool(XElement? element, bool fallback, ThemeDiagnostics diagnostics, string file)
        {
            if (element is null) return fallback;
            if (ThemeValueParser.TryParseBool(element.Value.Trim(), out bool value)) return value;
            diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.InvalidValue, file, Line(element), $"<{element.Name.LocalName}> \"{element.Value.Trim()}\" is not a boolean");
            return fallback;
        }

        internal static int Line(XObject node) => node is IXmlLineInfo info && info.HasLineInfo() ? info.LineNumber : 0;
    }
}
