using System;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Q47: the loader reads the six refused forms as ES-DE 3.4.1 was measured to read them - see EmuSen_BigPicture.md §31.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemeEsdeRulesTests : IDisposable
    {
        private readonly SyntheticTheme _theme = new();
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenQ47", Guid.NewGuid().ToString("N"));

        public ThemeEsdeRulesTests() => DataStore.OverrideDirectory = _root;

        public void Dispose()
        {
            _theme.Dispose();
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private ResolvedTheme Gamelist(string elements)
        {
            _theme.Capabilities("").Theme($"<view name=\"gamelist\">{elements}</view>");
            return _theme.Load();
        }

        private static ResolvedElement Only(ResolvedTheme theme, string type, string name)
        {
            Assert.True(theme.IsThemed, string.Join("; ", theme.Errors));
            return theme.GamelistView.Find(type, name)!;
        }

        [Theory]
        [InlineData("0.85 0.9", 0.85f, true)]
        [InlineData("0.85", 0.85f, false)]
        [InlineData("0.9abc", 0.9f, true)]
        [InlineData(" 0.3", 0.3f, false)]
        [InlineData("abc", 0.1f, true)]
        public void A_float_is_its_leading_number(string written, float expected, bool warned)
        {
            ResolvedTheme theme = Gamelist($"<badges name=\"b\"><folderLinkSize>{written}</folderLinkSize></badges>");
            Assert.Equal(expected, Only(theme, "badges", "b").Float("folderLinkSize")!.Value, 5);
            Assert.Equal(warned, theme.Diagnostics.Any(d => d.Code == ThemeDiagnosticCode.LenientValue && d.Severity == ThemeSeverity.Warning));
        }

        [Theory]
        [InlineData("w 0.02", 0f, 0.02f)]
        [InlineData("0.5 w", 0.5f, 0f)]
        [InlineData("0.75 0.3 0.9", 0.75f, 0.3f)]
        [InlineData(" 0.02 0.62", 0f, 0.02f)]
        [InlineData("  0.1 0.8", 0f, 0.1f)]
        [InlineData("0.6  0.5", 0.6f, 0.5f)]
        [InlineData("abc def", 0f, 0f)]
        public void A_pair_is_split_at_its_first_space_and_each_side_read_as_a_float(string written, float x, float y)
        {
            ResolvedTheme theme = Gamelist($"<gamelistinfo name=\"g\"><size>1 1</size><pos>{written}</pos></gamelistinfo>");
            NormalizedPair pair = Only(theme, "gamelistinfo", "g").Pair("pos")!.Value;
            Assert.Equal(x, pair.X, 5);
            Assert.Equal(y, pair.Y, 5);
        }

        [Theory]
        [InlineData("w0.02")]
        [InlineData("0.6")]
        [InlineData("0.6\t0.5")]
        [InlineData("0.6\n0.5")]
        [InlineData("0.75,0.6")]
        public void A_pair_with_no_space_unthemes_the_system_as_in_ES_DE(string written)
        {
            ResolvedTheme theme = Gamelist($"<gamelistinfo name=\"g\"><size>{written}</size></gamelistinfo>");
            Assert.False(theme.IsThemed);
            ThemeDiagnostic error = Assert.Single(theme.Errors);
            Assert.Equal(ThemeDiagnosticCode.BadFormat, error.Code);
            Assert.Contains("no space", error.Message);
        }

        [Theory]
        [InlineData("true", true)] [InlineData("TRUE", true)] [InlineData("1", true)] [InlineData("yes", true)] [InlineData("y", true)]
        [InlineData("t", true)] [InlineData("truex", true)] [InlineData("1.0", true)] [InlineData("true ", true)]
        [InlineData("false", false)] [InlineData("0", false)] [InlineData("no", false)] [InlineData("flase", false)] [InlineData("False", false)]
        [InlineData("on", false)] [InlineData("2", false)] [InlineData("-1", false)] [InlineData(" true ", false)] [InlineData(" 1", false)]
        public void A_boolean_is_true_when_its_first_character_is_t_y_or_1(string written, bool expected)
        {
            ResolvedTheme theme = Gamelist($"<text name=\"v\"><text>v</text><visible>{written}</visible><metadataElement>{written}</metadataElement></text>");
            ResolvedElement text = Only(theme, "text", "v");
            Assert.Equal(expected, ((BoolValue)text.Explicit["visible"]).Value);
            Assert.Equal(expected, ((BoolValue)text.Explicit["metadataElement"]).Value);
        }

        [Fact]
        public void A_bare_ampersand_is_kept_as_text_and_references_are_decoded()
        {
            _theme.File("capabilities.xml", "<themeCapabilities><colorScheme name=\"a\"><label>Game & Watch</label></colorScheme>" +
                "<colorScheme name=\"b\"><label>E &foo; F</label></colorScheme><colorScheme name=\"c\"><label>G &amp; H</label></colorScheme>" +
                "<colorScheme name=\"d\"><label>I &#74; K</label></colorScheme><colorScheme name=\"e\"><label>L &amp M</label></colorScheme></themeCapabilities>")
                .Theme("<view name=\"gamelist\"><text name=\"t\"><text>Q & A</text></text></view>");
            ResolvedTheme theme = _theme.Load();
            Assert.Equal(["Game & Watch", "E &foo; F", "G & H", "I J K", "L &amp M"], theme.Capabilities.ColorSchemes.Select(c => c.Labels["en_US"]));
            Assert.Equal("Q & A", Only(theme, "text", "t").String("text"));
            Assert.Equal(2, theme.Diagnostics.Count(d => d.Code == ThemeDiagnosticCode.LenientXml && d.Severity == ThemeSeverity.Warning));
        }

        [Fact]
        public void Text_outside_the_root_element_is_ignored()
        {
            _theme.Capabilities("")
                .File("before.xml", "f<!-- a stray letter -->\n<theme><view name=\"gamelist\"><text name=\"b\"><text>b</text></text></view></theme>")
                .File("after.xml", "<theme><view name=\"gamelist\"><text name=\"a\"><text>a</text></text></view></theme>\nzz")
                .Theme("<include>./before.xml</include><include>./after.xml</include>");
            ResolvedTheme theme = _theme.Load();
            Assert.NotNull(Only(theme, "text", "b"));
            Assert.NotNull(theme.GamelistView.Find("text", "a"));
        }

        [Fact]
        public void Mismatched_tags_still_untheme_the_system()
        {
            _theme.Capabilities("").File("broken.xml", "<theme><view name=\"gamelist\"><text name=\"x\"><text>x</text></view></theme>")
                .Theme("<include>./broken.xml</include>");
            ResolvedTheme theme = _theme.Load();
            Assert.False(theme.IsThemed);
            Assert.Contains(theme.Errors, d => d.Code == ThemeDiagnosticCode.MalformedXml);
        }

        private const string Profiles =
            "<transitions name=\"slideprof\"><gamelistToGamelist>slide</gamelistToGamelist></transitions>" +
            "<transitions name=\"fadeprof\"><gamelistToGamelist>fade</gamelistToGamelist></transitions><variant name=\"main\"/>";

        [Fact]
        public void Transitions_directly_in_theme_are_ignored_with_a_warning()
        {
            _theme.Capabilities(Profiles).Theme("<transitions>fadeprof</transitions><variant name=\"main\"><view name=\"gamelist\"><text name=\"t\"><text>t</text></text></view></variant>");
            ResolvedTheme theme = _theme.Load(new ThemeChoices { Variant = "main" });
            Assert.True(theme.IsThemed, string.Join("; ", theme.Errors));
            Assert.Equal("slideprof", theme.Transitions.Name);
            Assert.Contains(theme.Diagnostics, d => d.Code == ThemeDiagnosticCode.IgnoredTag && d.Severity == ThemeSeverity.Warning);
        }

        [Fact]
        public void Transitions_at_the_top_of_a_file_included_from_a_variant_apply_to_that_variant()
        {
            _theme.Capabilities(Profiles).File("v.xml", "<theme><transitions>fadeprof</transitions></theme>")
                .Theme("<variant name=\"main\"><include>./v.xml</include><view name=\"gamelist\"><text name=\"t\"><text>t</text></text></view></variant>");
            ResolvedTheme theme = _theme.Load(new ThemeChoices { Variant = "main" });
            Assert.True(theme.IsThemed, string.Join("; ", theme.Errors));
            Assert.Equal("fadeprof", theme.Transitions.Name);
            Assert.DoesNotContain(theme.Diagnostics, d => d.Code == ThemeDiagnosticCode.IgnoredTag);
        }

        private static FakeTheme Case(string repository, string gamelist, string? capabilities = null, string? top = null)
        {
            var t = new FakeTheme { Name = repository, Source = new ThemeSource("q47", repository, "main") };
            if (capabilities is not null) t.CapabilitiesText = capabilities;
            t.ThemeText = "<theme>" + (top ?? "") + "<view name=\"system\"><carousel name=\"c\"><pos>0 0.2</pos><size>1 0.5</size></carousel></view>" +
                          "<view name=\"gamelist\"><textlist name=\"l\"><pos>0.05 0.1</pos><size>0.5 0.8</size></textlist>" + gamelist + "</view></theme>";
            return t;
        }

        // Reported 2026-09-27: the browser refused Artflix (Revisited) and CarAlt; the install gate is the loader, so it follows the rules above.
        [Fact]
        public void A_download_installs_where_ES_DE_draws_the_theme_and_is_refused_where_it_does_not()
        {
            var hosts = new FakeThemeHosts();
            FakeTheme folderLink = Case("pair-float-es-de", "<badges name=\"b\"><folderLinkSize>0.85 0.9</folderLinkSize></badges>");
            FakeTheme ampersand = Case("ampersand-es-de", "", "<themeCapabilities><themeName>Amp</themeName><colorScheme name=\"w\"><label>Game & Watch</label></colorScheme></themeCapabilities>");
            FakeTheme gamelistInfo = Case("gamelistinfo-es-de", "<gamelistinfo name=\"g\"><size>w 0.02</size></gamelistinfo>");
            FakeTheme transitions = Case("transitions-es-de", "", top: "<transitions>instant</transitions>");
            FakeTheme visible = Case("visible-es-de", "<text name=\"t\"><text>t</text><visible>no</visible></text>");
            FakeTheme metadata = Case("metadata-es-de", "<text name=\"t\"><text>t</text><metadataElement>flase</metadataElement></text>");
            FakeTheme strayVariant = Case("stray-es-de", "", top: "<variant name=\"list\"><include>./v.xml</include></variant>");
            strayVariant.Extra["v.xml"] = "f<!-- -->\n<theme><transitions>instant</transitions></theme>";
            FakeTheme noSpace = Case("no-space-es-de", "<gamelistinfo name=\"g\"><size>w0.02</size></gamelistinfo>");
            FakeTheme mismatched = Case("mismatched-es-de", "<text name=\"t\"><text>t</text>");
            hosts.Themes.AddRange([folderLink, ampersand, gamelistInfo, transitions, visible, metadata, strayVariant, noSpace, mismatched]);

            foreach (FakeTheme accepted in new[] { folderLink, ampersand, gamelistInfo, transitions, visible, metadata, strayVariant })
            {
                hosts.Install(accepted.Source);
                Assert.True(File.Exists(Path.Combine(ThemeDownloads.DirectoryFor(accepted.Source), "theme.xml")), accepted.Name);
            }

            var refused = Assert.Throws<InvalidDataException>(() => hosts.Install(noSpace.Source));
            Assert.Contains("\"w0.02\", which has no space between two values", refused.Message);
            Assert.False(Directory.Exists(ThemeDownloads.DirectoryFor(noSpace.Source)));
            refused = Assert.Throws<InvalidDataException>(() => hosts.Install(mismatched.Source));
            Assert.StartsWith("The download is not a theme that loads:", refused.Message);
            Assert.False(Directory.Exists(ThemeDownloads.DirectoryFor(mismatched.Source)));
        }
    }
}
