using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Skips visibly, with the reason in the test report, when the reading clone of the ES-DE edition is absent.
    public sealed class ArtBookNextFactAttribute : FactAttribute
    {
        public static string Folder => Environment.GetEnvironmentVariable("EMUSEN_ARTBOOKNEXT") is { Length: > 0 } set
            ? set
            : Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), "Projects", "art-book-next-es-de-reference");

        public ArtBookNextFactAttribute()
        {
            if (!File.Exists(Path.Combine(Folder, "capabilities.xml")))
                Skip = $"Art Book Next (ES-DE edition) is not cloned at {Folder}; clone it there or set EMUSEN_ARTBOOKNEXT - see EmuSen_BigPicture.md §12.5";
        }
    }

    // The real ES-DE edition of Art Book Next, read in place and never copied into the repository - see EmuSen_BigPicture.md §12.5.
    public class ArtBookNextReferenceTests
    {
        private readonly ITestOutputHelper _output;

        public ArtBookNextReferenceTests(ITestOutputHelper output) => _output = output;

        public static IReadOnlyList<ThemeSystem> Systems { get; } =
        [
            new("nes", "Nintendo Entertainment System", "nes"),
            new("snes", "Super Nintendo", "snes"),
            new("n64", "Nintendo 64", "n64"),
            new("gb", "Game Boy", "gb"),
            new("gbc", "Game Boy Color", "gbc"),
            new("all", "All Games", "auto-allgames", ThemeSystemKind.AutoCollection),
            new("favorites", "Favorites", "auto-favorites", ThemeSystemKind.AutoCollection),
            new("recent", "Last Played", "auto-lastplayed", ThemeSystemKind.AutoCollection),
            new("collections", "Collections", "custom-collections", ThemeSystemKind.CustomCollection),
        ];

        private static readonly string[] Ratios = ["16:10", "16:9", "4:3"];
        private static readonly string[] Schemes = ["dark-screenshots", "light-noir", "snes-outline", "oled-original", "custom"];

        [ArtBookNextFact]
        public void Every_variant_scheme_and_ratio_run_resolves_with_no_errors_and_nothing_unknown()
        {
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(ArtBookNextFactAttribute.Folder);
            Assert.Empty(caps.Diagnostics);
            Assert.Equal(20, caps.Variants.Count);
            Assert.Equal(31, caps.ColorSchemes.Count);
            Assert.Equal(12, caps.AspectRatios.Count);

            int loads = 0;
            foreach (ThemeSystem system in Systems)
                foreach (ThemeVariant variant in caps.Variants)
                    foreach (string ratio in Ratios)
                        foreach (string scheme in Schemes)
                        {
                            ResolvedTheme theme = ThemeLoader.Load(caps, system, new ThemeChoices { Variant = variant.Name, AspectRatio = ratio, ColorScheme = scheme });
                            string where = $"{system.Theme} {variant.Name} {ratio} {scheme}";
                            Assert.True(theme.IsThemed, where + "\n" + string.Join("\n", theme.Errors));
                            Assert.Empty(ThemeInventory.Unknown(theme));
                            Assert.DoesNotContain(theme.Diagnostics, d => d.Severity == ThemeSeverity.Warning);
                            ThemeDiagnostic skipped = Assert.Single(theme.Diagnostics);
                            Assert.Equal(ThemeDiagnosticCode.IncludeSkipped, skipped.Code);
                            Assert.Contains("${customizationPath}", skipped.Message);
                            Assert.NotNull(theme.SystemView.Primary);
                            Assert.NotNull(theme.GamelistView.Primary);
                            loads++;
                        }
            _output.WriteLine($"{loads} loads");
        }

        [ArtBookNextFact]
        public void Every_aspect_ratio_and_font_size_resolves()
        {
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(ArtBookNextFactAttribute.Folder);
            foreach (string ratio in caps.AspectRatios)
                foreach (string size in caps.FontSizes)
                {
                    ResolvedTheme theme = ThemeLoader.Load(caps, Systems[1], new ThemeChoices { AspectRatio = ratio, FontSize = size });
                    Assert.True(theme.IsThemed, $"{ratio} {size}\n{string.Join("\n", theme.Errors)}");
                    Assert.Equal(ratio, theme.Selection.AspectRatio);
                }
        }

        [ArtBookNextFact]
        public void With_no_variant_chosen_the_first_declared_is_used_and_16_10_is_automatic_at_1280_by_800()
        {
            ResolvedTheme theme = ThemeLoader.Load(ArtBookNextFactAttribute.Folder, Systems[1], new ThemeChoices());
            Assert.Equal("gamelist-list-metadata-cover", theme.Selection.Variant);
            Assert.Equal("16:10", theme.Selection.AspectRatio);
            Assert.Equal("dark-screenshots", theme.Selection.ColorScheme);
            Assert.Equal("medium", theme.Selection.FontSize);
            Assert.Equal("instant", theme.Transitions.Name);
            Assert.Equal(7, theme.Sounds.Count);
            Assert.All(theme.Sounds.Values, s => Assert.True(s.Exists));
        }

        [ArtBookNextFact]
        public void With_no_media_every_noMedia_override_falls_to_the_basic_list_in_the_gamelist_view_only()
        {
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(ArtBookNextFactAttribute.Folder);
            int overridden = 0;
            foreach (ThemeVariant variant in caps.Variants)
            {
                ResolvedTheme theme = ThemeLoader.Load(caps, Systems[1], new ThemeChoices { Variant = variant.Name }, MediaPresence.None);
                Assert.Equal(variant.Name, theme.Selection.Variant);
                if (variant.Overrides.Count == 0)
                {
                    Assert.Equal(variant.Name, theme.GamelistVariant);
                    continue;
                }
                string expected = variant.Name.EndsWith("-nh", StringComparison.Ordinal) ? "gamelist-list-basic-nh" : "gamelist-list-basic";
                Assert.Equal(expected, theme.GamelistVariant);
                Assert.True(theme.IsThemed);
                overridden++;
            }
            Assert.Equal(12, overridden);
        }

        [ArtBookNextFact]
        public void The_union_of_properties_set_over_the_runs_and_the_load_time()
        {
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(ArtBookNextFactAttribute.Folder);
            var union = new HashSet<(string, string)>();
            foreach (ThemeSystem system in Systems)
                foreach (ThemeVariant variant in caps.Variants)
                    foreach (string ratio in caps.AspectRatios)
                        foreach (string scheme in new[] { "dark-screenshots", "custom" })
                            union.UnionWith(ThemeInventory.SetPairs(ThemeLoader.Load(caps, system, new ThemeChoices { Variant = variant.Name, AspectRatio = ratio, ColorScheme = scheme })));

            foreach (IGrouping<string, (string Type, string Property)> type in union.GroupBy(p => p.Item1).OrderBy(g => g.Key, StringComparer.Ordinal))
                _output.WriteLine($"{type.Key} {type.Count()}: {string.Join(" ", type.Select(p => p.Property).OrderBy(p => p, StringComparer.Ordinal))}");
            _output.WriteLine($"union {union.Count} of {ThemeCatalog.PropertyCount}");

            for (int warm = 0; warm < 20; warm++) ThemeLoader.Load(caps, Systems[1], new ThemeChoices());
            var clock = Stopwatch.StartNew();
            const int runs = 200;
            for (int i = 0; i < runs; i++) ThemeLoader.Load(caps, Systems[i % Systems.Count], new ThemeChoices());
            double perLoad = clock.Elapsed.TotalMilliseconds / runs;
            _output.WriteLine($"load {perLoad:0.00} ms per system (both views), warm, {runs} runs");
            Assert.True(union.Count > 150, $"union {union.Count}");
        }
    }
}
