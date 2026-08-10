using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;
using Avalonia.Styling;
using EmuSen.LunaP.Settings;

namespace EmuSen.LunaP.Theme
{
    // Which theme is in use, persisted next to the rest of the config.
    public sealed class ThemeChoice
    {
        public string Name { get; set; } = LunaTheme.BuiltIn;
    }

    // Loads a user theme over the built-in palette - see EmuSen_LunaP.md §13.
    public static class LunaTheme
    {
        public const string BuiltIn = "Built-in";
        public const string Extension = ".axaml";
        public const string CssExtension = ".css";

        // Tried in this order, so a name spelled both ways resolves to the .axaml - see EmuSen_LunaP.md §12.2.
        public static readonly IReadOnlyList<string> Extensions = new[] { Extension, CssExtension };

        public const string ChoiceFileName = "luna.json";

        // The category the themes folder is, which is also the shape cheats/<name>.json already uses - see `man hier`.
        public const string ThemeCategory = "themes";

        // The one dictionary a theme's keys live in; kept so applying a second theme replaces rather than stacks.
        private static ResourceDictionary? _applied;

        // The styles half, which only a .css theme produces; removed with the dictionary.
        private static Styles? _appliedStyles;

        public static string Current { get; private set; } = BuiltIn;

        // Raised only when Application.Styles changed, which is the one case an open window must be restyled - see EmuSen_LunaP.md §12.3.
        public static event Action? StylesChanged;

        // Detaching and reattaching the content is what re-runs the style pass over controls that are already realized.
        public static void Restyle(ContentControl root)
        {
            object? content = root.Content;
            if (content is null) return;

            root.Content = null;
            root.Content = content;
        }

        public static string Directory => LunaSettings.Store.Directory(ThemeCategory);

        // Built-in first, then whatever is on disk, alphabetically. A name is listed once however many formats spell it.
        public static IReadOnlyList<string> Available()
        {
            var names = new List<string> { BuiltIn };
            if (!System.IO.Directory.Exists(Directory)) return names;

            names.AddRange(Extensions
                .SelectMany(ext => System.IO.Directory.EnumerateFiles(Directory, "*" + ext))
                .Select(Path.GetFileNameWithoutExtension)
                .Where(n => !string.IsNullOrEmpty(n) && !string.Equals(n, BuiltIn, StringComparison.OrdinalIgnoreCase))
                .Distinct(StringComparer.OrdinalIgnoreCase)
                .OrderBy(n => n, StringComparer.OrdinalIgnoreCase)!);

            return names;
        }

        // Applies and persists. False means the theme was unreadable and the previous one is still in force.
        public static bool Apply(string name)
        {
            if (!TryApply(name)) return false;

            Current = name;
            LunaSettings.Store.Save(null, ChoiceFileName, new ThemeChoice { Name = name });
            return true;
        }

        // The saved name, whether or not it still resolves to a readable theme.
        public static string Saved => (LunaSettings.Store.Load<ThemeChoice>(null, ChoiceFileName) ?? new ThemeChoice()).Name;

        // Called once at startup. A theme since deleted or broken falls back to built-in without overwriting the saved choice,
        // so fixing the file and restarting is enough to get it back.
        public static void ApplySaved()
        {
            if (TryApply(Saved)) return;

            TryApply(BuiltIn);
        }

        private static bool TryApply(string name)
        {
            if (Application.Current is not { } app) return false;

            if (string.Equals(name, BuiltIn, StringComparison.OrdinalIgnoreCase))
            {
                if (Remove(app)) StylesChanged?.Invoke();
                Current = BuiltIn;
                return true;
            }

            string? path = Resolve(name);
            if (path is null)
            {
                LunaSettings.Report($"theme '{name}' not found in {Directory}.");
                return false;
            }

            if (Read(path) is not { } content) return false;

            (ResourceDictionary loaded, Styles? styles) = content;
            bool touchedStyles = Remove(app);

            // Merged last, so its keys win over Theme/Palette.axaml's; every consumer uses DynamicResource and updates live.
            app.Resources.MergedDictionaries.Add(loaded);
            _applied = loaded;

            // Appended last for the same reason, so a theme's rules beat Theme/Controls.axaml's.
            if (styles is { Count: > 0 })
            {
                app.Styles.Add(styles);
                _appliedStyles = styles;
                touchedStyles = true;
            }

            Current = name;
            if (touchedStyles) StylesChanged?.Invoke();
            return true;
        }

        // The first format that exists wins; a theme is one name, whatever it is written in.
        private static string? Resolve(string name) =>
            Extensions.Select(ext => Path.Combine(Directory, name + ext)).FirstOrDefault(File.Exists);

        private static (ResourceDictionary Resources, Styles? Styles)? Read(string path)
        {
            try
            {
                return Path.GetExtension(path).Equals(CssExtension, StringComparison.OrdinalIgnoreCase)
                    ? ReadCss(path)
                    : AvaloniaRuntimeXamlLoader.Load(File.ReadAllText(path)) is ResourceDictionary dictionary
                        ? (dictionary, null)
                        : Reported(path, "the file is not a ResourceDictionary");
            }
            catch (Exception ex)
            {
                // A broken theme must never take the program down with it - the same rule Galaxia applies to config.
                return Reported(path, ex.Message);
            }
        }

        private static (ResourceDictionary, Styles?) ReadCss(string path)
        {
            CssThemeResult css = CssTheme.Parse(File.ReadAllText(path));

            // Skipped rules are reported but do not refuse the theme - see EmuSen_LunaP.md §12.2.
            if (css.Warnings.Count > 0) LunaSettings.Report($"{path}: {string.Join(" ", css.Warnings)}");

            return (css.Resources, css.Styles);
        }

        private static (ResourceDictionary, Styles?)? Reported(string path, string why)
        {
            LunaSettings.Report($"{path}: {why} Falling back to the previous theme.");
            return null;
        }

        // True when Application.Styles was touched, which is the case a realized control cannot survive on its own.
        private static bool Remove(Application app)
        {
            bool touchedStyles = _appliedStyles is not null;
            if (_appliedStyles is not null)
            {
                app.Styles.Remove(_appliedStyles);
                _appliedStyles = null;
            }

            if (_applied is null) return touchedStyles;

            app.Resources.MergedDictionaries.Remove(_applied);
            _applied = null;
            return touchedStyles;
        }
    }
}
