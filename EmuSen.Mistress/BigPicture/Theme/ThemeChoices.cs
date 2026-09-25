using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Mistress.BigPicture.Theme
{
    public enum ThemeSystemKind { Regular, AutoCollection, CustomCollection }

    // The system a theme is loaded for; its three names feed the system variables - see EmuSen_BigPicture.md §12.2.
    public sealed record ThemeSystem(string Name, string FullName, string Theme, ThemeSystemKind Kind = ThemeSystemKind.Regular)
    {
        // The twelve documented system variables; the kind-specific forms are empty for the other kinds.
        public IReadOnlyDictionary<string, string> Variables()
        {
            var variables = new Dictionary<string, string>(StringComparer.Ordinal);
            foreach ((string key, string value) in new[] { ("name", Name), ("fullName", FullName), ("theme", Theme) })
            {
                variables[$"system.{key}"] = value;
                variables[$"system.{key}.autoCollections"] = Kind == ThemeSystemKind.AutoCollection ? value : "";
                variables[$"system.{key}.customCollections"] = Kind == ThemeSystemKind.CustomCollection ? value : "";
                variables[$"system.{key}.noCollections"] = Kind == ThemeSystemKind.Regular ? value : "";
            }
            return variables;
        }
    }

    // Which media a system's games have, for the variant triggers; null means triggers are not evaluated.
    public sealed record MediaPresence(IReadOnlySet<string> Types)
    {
        public bool HasVideos => Types.Contains("video");

        public static MediaPresence None { get; } = new(new HashSet<string>());
    }

    // The player's choices, as the settings would store them; null means Automatic or not chosen.
    public sealed record ThemeChoices
    {
        public string? Variant { get; init; }
        public string? ColorScheme { get; init; }
        public string? FontSize { get; init; }
        public string? Language { get; init; }
        public string? AspectRatio { get; init; }
        public string? Transitions { get; init; }
        public int ScreenWidth { get; init; } = 1280;
        public int ScreenHeight { get; init; } = 800;
        public bool VariantTriggers { get; init; } = true;
    }

    // The choices after THEMES.md's rules and this loader's fallbacks - see EmuSen_BigPicture.md §12.4.
    public sealed record ThemeSelection(string? Variant, string? ColorScheme, string? FontSize, string? Language, string? AspectRatio, string? TransitionsChoice)
    {
        public bool Vertical => ThemeCapabilities.IsVertical(AspectRatio);

        public static ThemeSelection Resolve(ThemeCapabilities capabilities, ThemeChoices choices)
        {
            string? variant = choices.Variant is { } v && capabilities.IsDeclaredVariant(v)
                ? v
                : (capabilities.Variants.FirstOrDefault(x => x.Selectable) ?? capabilities.Variants.FirstOrDefault())?.Name;

            string? scheme = choices.ColorScheme is { } c && capabilities.ColorSchemes.Any(s => s.Name == c)
                ? c
                : capabilities.ColorSchemes.FirstOrDefault()?.Name;

            string? fontSize = choices.FontSize is { } f && capabilities.FontSizes.Contains(f)
                ? f
                : capabilities.FontSizes.Contains("medium") ? "medium" : capabilities.FontSizes.FirstOrDefault();

            string? language = choices.Language is { } l && capabilities.Languages.Contains(l)
                ? l
                : capabilities.Languages.Contains("en_US") ? "en_US" : null;

            string? aspect = choices.AspectRatio is { } a && a != "automatic" && capabilities.AspectRatios.Contains(a)
                ? a
                : Nearest(capabilities.AspectRatios, choices.ScreenWidth, choices.ScreenHeight);

            return new ThemeSelection(variant, scheme, fontSize, language, aspect, choices.Transitions);
        }

        // Automatic picks the declared ratio nearest the screen's, measured as |log(r1/r2)|.
        public static string? Nearest(IReadOnlyList<string> declared, int width, int height)
        {
            if (declared.Count == 0 || width <= 0 || height <= 0) return declared.FirstOrDefault();
            double screen = (double)width / height;
            return declared.OrderBy(r => Math.Abs(Math.Log(ThemeCapabilities.Ratio(r)!.Value / screen))).First();
        }

        // noMedia before noVideos, one step only, gamelist view only - see EmuSen_BigPicture.md §12.2.
        public static string? ApplyTriggers(ThemeCapabilities capabilities, string? variant, MediaPresence? media, bool enabled, ThemeDiagnostics diagnostics)
        {
            if (!enabled || media is null || variant is null || capabilities.FindVariant(variant) is not { } declared) return variant;

            foreach (string trigger in new[] { "noMedia", "noVideos" })
            {
                foreach (VariantOverride o in declared.Overrides.Where(o => o.Trigger == trigger))
                {
                    bool fires = trigger == "noMedia" ? !o.MediaTypes.Any(media.Types.Contains) : !media.HasVideos;
                    if (!fires) continue;
                    if (capabilities.IsDeclaredVariant(o.UseVariant)) return o.UseVariant;
                    diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.Undeclared, System.IO.Path.Combine(capabilities.Directory, ThemeCapabilitiesReader.FileName), 0,
                        $"override variant \"{o.UseVariant}\" is not declared");
                }
            }
            return variant;
        }
    }
}
