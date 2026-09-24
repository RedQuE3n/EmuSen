using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.LunaP.Controls;
using EmuSen.Serenity.Shaders;

namespace EmuSen.Mistress.Library
{
    // One row of the Shaders window: the value stored under ScreenFilter, a readable name, its group and, for a preset, its path in the pack - see EmuSen_Settings_Reference.md §4.48.1.
    public sealed record ShaderEntry(string Stored, string Name, string Group, string? Relative, bool Recent = false)
    {
        public bool IsPreset => Relative is not null;

        public override string ToString() => Name;
    }

    // The built-in filters and the pack's presets as one list for a console, grouped and named for reading - see EmuSen_Settings_Reference.md §4.48.1.
    public static class ShaderCatalog
    {
        public const string SlangPrefix = "slang:";
        public const string BuiltIn = "Built into EmuSen";
        public const string RecentGroup = "Recently used";
        public const string AllCategories = "All";
        public const int RecentKept = 5;

        // A file or folder name as words: the extension dropped, underscores as spaces, runs of spaces as one.
        public static string Readable(string name)
        {
            if (name.EndsWith(".slangp", StringComparison.OrdinalIgnoreCase)) name = name[..^".slangp".Length];
            return string.Join(' ', name.Replace('_', ' ').Split(' ', StringSplitOptions.RemoveEmptyEntries));
        }

        // A preset's folder inside the pack, each level readable, joined by " / "; a preset at the top is "Other".
        public static string Folder(string relative)
        {
            int slash = relative.LastIndexOf('/');
            return slash < 0 ? "Other" : string.Join(" / ", relative[..slash].Split('/').Select(Readable));
        }

        // The top-level folder, which is what the category dropdown offers.
        public static string Category(string relative) => relative.Contains('/') ? Readable(relative[..relative.IndexOf('/')]) : "Other";

        public static ShaderEntry ForPreset(string relative) =>
            new(SlangPrefix + relative, Readable(relative[(relative.LastIndexOf('/') + 1)..]), Folder(relative), relative);

        // Any stored value as an entry; a built-in name no longer offered, or none, is None.
        public static ShaderEntry For(string? stored) => stored is not null && stored.StartsWith(SlangPrefix, StringComparison.Ordinal)
            ? ForPreset(stored[SlangPrefix.Length..])
            : new ShaderEntry(ScreenFilters.Find(stored).Name, ScreenFilters.Find(stored).Name, BuiltIn, null);

        public static IReadOnlyList<ShaderEntry> BuiltIns(string console) =>
            ScreenFilters.NamesFor(console).Select(name => new ShaderEntry(name, name, BuiltIn, null)).ToArray();

        public static IReadOnlyList<ShaderEntry> Presets(IEnumerable<string> relatives) =>
            relatives.Select(ForPreset).OrderBy(e => e.Group, StringComparer.OrdinalIgnoreCase).ThenBy(e => e.Name, StringComparer.OrdinalIgnoreCase).ToArray();

        // None first, whatever is searched; recents only when nothing is, since a search already finds them in their folder.
        public static IReadOnlyList<ShaderEntry> Shown(IReadOnlyList<ShaderEntry> builtIns, IReadOnlyList<ShaderEntry> presets,
            IReadOnlyList<string> recent, string? search, string category)
        {
            var shown = new List<ShaderEntry>();
            bool searching = !string.IsNullOrWhiteSpace(search);
            if (!searching && category == AllCategories)
                foreach (string stored in recent)
                    if (Find(builtIns, presets, stored) is { } used) shown.Add(used with { Group = RecentGroup, Recent = true });

            foreach (ShaderEntry entry in builtIns)
                if (entry.Stored == ScreenFilters.None || ((category is AllCategories or BuiltIn) && Matches(entry, search))) shown.Add(entry);
            foreach (ShaderEntry entry in presets)
                if ((category == AllCategories || Category(entry.Relative!) == category) && Matches(entry, search)) shown.Add(entry);
            return shown;
        }

        public static bool Matches(ShaderEntry entry, string? search) => FilterBar.MatchesWords(search, entry.Name, entry.Group, entry.Relative);

        public static IReadOnlyList<string> Categories(IReadOnlyList<ShaderEntry> presets) =>
            new[] { AllCategories, BuiltIn }.Concat(presets.Select(p => Category(p.Relative!)).Distinct(StringComparer.OrdinalIgnoreCase)
                .OrderBy(c => c, StringComparer.OrdinalIgnoreCase)).ToArray();

        private static ShaderEntry? Find(IReadOnlyList<ShaderEntry> builtIns, IReadOnlyList<ShaderEntry> presets, string stored) =>
            builtIns.FirstOrDefault(e => e.Stored == stored && e.Stored != ScreenFilters.None) ?? presets.FirstOrDefault(e => e.Stored == stored);
    }
}
