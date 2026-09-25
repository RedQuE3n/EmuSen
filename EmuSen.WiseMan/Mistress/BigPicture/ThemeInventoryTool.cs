using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The theme inventory as a tool: EMUSEN_THEME_INSPECT=<theme folder> runs it - see EmuSen_BigPicture.md §12.5.
    public class ThemeInventoryTool
    {
        private readonly ITestOutputHelper _output;

        public ThemeInventoryTool(ITestOutputHelper output) => _output = output;

        // EMUSEN_THEME_CHOICES is "system=snes;variant=...;scheme=...;font=...;aspect=16:10;media=cover,video"; EMUSEN_THEME_INSPECT_OUT names a file to write.
        [Fact]
        public void Inventory_of_the_theme_named_by_the_environment()
        {
            string? folder = Environment.GetEnvironmentVariable("EMUSEN_THEME_INSPECT");
            if (string.IsNullOrEmpty(folder)) return;

            Dictionary<string, string> choices = (Environment.GetEnvironmentVariable("EMUSEN_THEME_CHOICES") ?? "")
                .Split(';', StringSplitOptions.RemoveEmptyEntries)
                .Select(p => p.Split('=', 2))
                .ToDictionary(p => p[0].Trim(), p => p.Length > 1 ? p[1].Trim() : "");

            string systemName = choices.GetValueOrDefault("system", "snes");
            var system = new ThemeSystem(systemName, choices.GetValueOrDefault("fullname", systemName), systemName,
                systemName.StartsWith("auto-", StringComparison.Ordinal) ? ThemeSystemKind.AutoCollection
                : systemName == "custom-collections" ? ThemeSystemKind.CustomCollection : ThemeSystemKind.Regular);
            MediaPresence? media = choices.TryGetValue("media", out string? m) ? new MediaPresence(ThemeLists.Split(m).ToHashSet()) : null;

            ResolvedTheme theme = ThemeLoader.Load(folder, system, new ThemeChoices
            {
                Variant = choices.GetValueOrDefault("variant"),
                ColorScheme = choices.GetValueOrDefault("scheme"),
                FontSize = choices.GetValueOrDefault("font"),
                AspectRatio = choices.GetValueOrDefault("aspect"),
                Language = choices.GetValueOrDefault("language"),
                Transitions = choices.GetValueOrDefault("transitions"),
            }, media);

            string report = ThemeInventory.Describe(theme, choices.ContainsKey("defaults"));
            if (Environment.GetEnvironmentVariable("EMUSEN_THEME_INSPECT_OUT") is { Length: > 0 } outFile) File.WriteAllText(outFile, report);
            else _output.WriteLine(report);
        }
    }
}
