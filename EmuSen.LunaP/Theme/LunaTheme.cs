using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;
using EmuSen.Galaxia;

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

        private static readonly ConfigFile<ThemeChoice> Choice = new("luna.json");

        // The one dictionary a theme's keys live in; kept so applying a second theme replaces rather than stacks.
        private static ResourceDictionary? _applied;

        public static string Current { get; private set; } = BuiltIn;

        // /etc/EmuSen/themes, the same category shape cheats/<name>.json already uses - see `man hier`.
        public static string Directory => Path.GetDirectoryName(ConfigStore.For("themes", "x"))!;

        // Built-in first, then whatever is on disk, alphabetically.
        public static IReadOnlyList<string> Available()
        {
            var names = new List<string> { BuiltIn };
            if (!System.IO.Directory.Exists(Directory)) return names;

            names.AddRange(System.IO.Directory
                .EnumerateFiles(Directory, "*" + Extension)
                .Select(Path.GetFileNameWithoutExtension)
                .Where(n => !string.IsNullOrEmpty(n) && !string.Equals(n, BuiltIn, StringComparison.OrdinalIgnoreCase))
                .OrderBy(n => n, StringComparer.OrdinalIgnoreCase)!);

            return names;
        }

        // Applies and persists. False means the theme was unreadable and the previous one is still in force.
        public static bool Apply(string name)
        {
            if (!TryApply(name)) return false;

            Current = name;
            Choice.Save(new ThemeChoice { Name = name });
            return true;
        }

        // The saved name, whether or not it still resolves to a readable theme.
        public static string Saved => Choice.Load(() => new ThemeChoice()).Name;

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
                Remove(app);
                Current = BuiltIn;
                return true;
            }

            string path = Path.Combine(Directory, name + Extension);
            if (!File.Exists(path))
            {
                ConfigDiagnostics.Report($"theme '{name}' not found at {path}.");
                return false;
            }

            ResourceDictionary? loaded = Read(path);
            if (loaded is null) return false;

            // Merged last, so its keys win over Theme/Palette.axaml's; every consumer uses DynamicResource and updates live.
            Remove(app);
            app.Resources.MergedDictionaries.Add(loaded);
            _applied = loaded;
            Current = name;
            return true;
        }

        private static ResourceDictionary? Read(string path)
        {
            try
            {
                return AvaloniaRuntimeXamlLoader.Load(File.ReadAllText(path)) as ResourceDictionary
                    ?? Reported(path, "the file is not a ResourceDictionary");
            }
            catch (Exception ex)
            {
                // A broken theme must never take the program down with it - the same rule Galaxia applies to config.
                return Reported(path, ex.Message);
            }
        }

        private static ResourceDictionary? Reported(string path, string why)
        {
            ConfigDiagnostics.Report($"{path}: {why} Falling back to the previous theme.");
            return null;
        }

        private static void Remove(Application app)
        {
            if (_applied is null) return;

            app.Resources.MergedDictionaries.Remove(_applied);
            _applied = null;
        }
    }
}
