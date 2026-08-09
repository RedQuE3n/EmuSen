using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;
using Avalonia.Styling;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Bin.Commands;
using EmuSen.LunaP.Theme;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.LunaP
{
    // `man theme` is the definition of the theme format, and this is what stops it describing one that does not exist - see EmuSen_LunaP.md §12.2.
    public class ThemeVocabularyTests
    {
        private static string Page => ManPages.Lookup("theme")
            ?? throw new InvalidOperationException("there is no `man theme` page.");

        // Palette.axaml is the authority on what keys exist; the Color aliases are the brush keys' other half, not separate tokens.
        private static IReadOnlyList<string> PaletteKeys()
        {
            var dictionary = (ResourceDictionary)AvaloniaXamlLoader.Load(
                new Uri("avares://EmuSen.LunaP/Theme/Palette.axaml"));

            return dictionary.Keys.OfType<string>()
                .Where(key => !key.EndsWith("Color", StringComparison.Ordinal))
                .OrderBy(key => key, StringComparer.Ordinal)
                .ToList();
        }

        [Fact]
        public Task Every_palette_key_is_documented_as_a_token() => UiTest.Run(() =>
        {
            string page = Page;
            foreach (string key in PaletteKeys())
            {
                Assert.Contains(CssTheme.TokenFor(key), page);
            }
        });

        // The other direction: a token documented for a key that no longer exists would read as a working theme that silently does nothing.
        [Fact]
        public Task Every_documented_token_still_resolves() => UiTest.Run(() =>
        {
            var tokens = Regex.Matches(Page, @"--luna-[a-z-]+").Select(m => m.Value).Distinct().ToList();

            Assert.NotEmpty(tokens);
            foreach (string token in tokens)
            {
                string key = "Luna" + string.Concat(token["--luna-".Length..].Split('-')
                    .Select(word => char.ToUpperInvariant(word[0]) + word[1..]));

                Assert.True(Application.Current!.TryGetResource(key, ThemeVariant.Dark, out _),
                    $"`man theme` documents {token}, but {key} does not resolve.");
            }
        });

        // Set equality, not containment: a control added to the kit and left undocumented fails here too.
        [Fact]
        public void The_documented_elements_are_exactly_the_ones_the_parser_accepts()
        {
            Assert.Equal(
                CssTheme.ElementNames.OrderBy(n => n, StringComparer.Ordinal),
                Section("kebab case:", "States:").OrderBy(n => n, StringComparer.Ordinal));
        }

        [Fact]
        public void The_documented_properties_are_exactly_the_ones_the_parser_accepts()
        {
            Assert.Equal(
                CssTheme.PropertyNames.OrderBy(n => n, StringComparer.Ordinal),
                Section("Properties:", "A value may").OrderBy(n => n, StringComparer.Ordinal));
        }

        [Fact]
        public void Every_state_and_part_of_every_element_is_documented()
        {
            string page = Page;
            foreach (string element in CssTheme.ElementNames)
            {
                foreach (string state in CssTheme.StatesOf(element))
                {
                    Assert.Contains($"{element}.{state}", page);
                }

                foreach (string part in CssTheme.PartsOf(element))
                {
                    Assert.Contains($".{part}", page);
                }
            }
        }

        // The page is reachable the way a user reaches it, not only through Lookup.
        [Fact]
        public void The_shell_prints_the_theme_page()
        {
            var shell = DianaOSInterpreter.CreateDefault(null);

            var result = shell.Submit("man theme");

            Assert.Contains("NAME", result.Output);
            Assert.Contains("/etc/EmuSen/themes", result.Output);
        }

        // The prose between two markers, split on the punctuation a list uses - defined regions rather than a scan of the whole page.
        private static IEnumerable<string> Section(string from, string to)
        {
            string page = Page;
            int start = page.IndexOf(from, StringComparison.Ordinal);
            Assert.True(start >= 0, $"`man theme` no longer contains '{from}'.");

            start += from.Length;
            int end = page.IndexOf(to, start, StringComparison.Ordinal);
            Assert.True(end >= 0, $"`man theme` no longer contains '{to}' after '{from}'.");

            return page[start..end]
                .Split(new[] { ' ', '\n', '\r', ',', '.' }, StringSplitOptions.RemoveEmptyEntries)
                .Where(word => word.Length > 1);
        }
    }
}
