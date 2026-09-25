using System.Collections.Generic;
using System.Linq;
using System.Text;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.WiseMan.Fixtures
{
    // Prints what a theme resolves to, per element and property, and what is unknown or unsupported - see EmuSen_BigPicture.md §12.5.
    public static class ThemeInventory
    {
        public static string Describe(ResolvedTheme theme, bool withDefaults = false)
        {
            var text = new StringBuilder();
            ThemeSelection s = theme.Selection;
            text.AppendLine($"theme {theme.Capabilities.ThemeName} | system {theme.System.Theme} | variant {s.Variant} (gamelist {theme.GamelistVariant}) | " +
                $"scheme {s.ColorScheme} | font size {s.FontSize} | aspect {s.AspectRatio} | language {s.Language ?? "-"} | transitions {theme.Transitions.Name}");
            text.AppendLine($"themed {(theme.IsThemed ? "yes" : "NO")} | files {theme.FilesRead.Count} | " +
                $"errors {Count(theme, ThemeSeverity.Error)} warnings {Count(theme, ThemeSeverity.Warning)} debug {Count(theme, ThemeSeverity.Debug)}");

            foreach (ResolvedView view in new[] { theme.SystemView, theme.GamelistView })
            {
                text.AppendLine($"view {view.Name}: {view.Elements.Count} elements");
                foreach (ResolvedElement e in view.Elements)
                {
                    text.AppendLine($"  {e.Type} \"{e.Name}\" z={(e.ZIndex is { } z ? z.ToString("0.###", System.Globalization.CultureInfo.InvariantCulture) : "top")}" +
                        (e.Bindings.Count > 0 ? $" binds {string.Join(" ", e.Bindings)}" : ""));
                    foreach ((string name, ThemeValue value) in e.Explicit.OrderBy(p => p.Key, System.StringComparer.Ordinal))
                        text.AppendLine($"    {name} = {Show(value)}{(e.Spec.Find(name)?.Deferred is not null ? "   [unsupported: video playback deferred]" : "")}");
                    if (withDefaults)
                        foreach ((string name, ThemeValue value) in e.Effective.Where(p => !e.Explicit.ContainsKey(p.Key)).OrderBy(p => p.Key, System.StringComparer.Ordinal))
                            text.AppendLine($"    {name} = {Show(value)}   (default)");
                }
            }
            if (theme.Sounds.Count > 0)
                text.AppendLine($"sounds: {string.Join(", ", theme.Sounds.Select(p => $"{p.Key}{(p.Value.Exists ? "" : " (missing)")}"))}");

            text.AppendLine($"unknown: {string.Join("; ", Unknown(theme).Select(d => d.ToString()).DefaultIfEmpty("none"))}");
            text.AppendLine($"unsupported: {string.Join(", ", Unsupported(theme).Select(u => $"{u.Type}.{u.Property}").Distinct().DefaultIfEmpty("none"))}");
            foreach (ThemeDiagnostic d in theme.Diagnostics)
                text.AppendLine($"  {d}");
            return text.ToString();
        }

        public static int Count(ResolvedTheme theme, ThemeSeverity severity) => theme.Diagnostics.Count(d => d.Severity == severity);

        public static IEnumerable<ThemeDiagnostic> Unknown(ResolvedTheme theme) => theme.Diagnostics.Where(d =>
            d.Code is ThemeDiagnosticCode.UnknownElement or ThemeDiagnosticCode.UnknownProperty or ThemeDiagnosticCode.UnknownTag);

        // Properties the theme sets that the loader resolves but no stage yet draws.
        public static IEnumerable<(string Type, string Property)> Unsupported(ResolvedTheme theme) =>
            Views(theme).SelectMany(e => e.Explicit.Keys.Where(k => e.Spec.Find(k)?.Deferred is not null).Select(k => (e.Type, k)));

        // Every (element type, property) pair the theme sets explicitly in either view or among the sounds.
        public static IEnumerable<(string Type, string Property)> SetPairs(ResolvedTheme theme) =>
            Views(theme).SelectMany(e => e.Explicit.Keys.Select(k => (e.Type, k)));

        private static IEnumerable<ResolvedElement> Views(ResolvedTheme theme) =>
            theme.SystemView.Elements.Concat(theme.GamelistView.Elements).Concat(theme.SoundElements);

        private static string Show(ThemeValue value) => value switch
        {
            PathValue p => $"{p.Value.Written}{(p.Value.Exists ? "" : " (missing)")}",
            KeyedPathsValue k => string.Join(", ", k.Paths.OrderBy(p => p.Key, System.StringComparer.Ordinal).Select(p => $"{p.Key}={p.Value.Written}")),
            _ => value.ToString() ?? "",
        };
    }
}
