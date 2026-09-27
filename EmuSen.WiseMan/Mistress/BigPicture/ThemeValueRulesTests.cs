using System;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Q70 and Q71: COLOR, UNSIGNED_INTEGER, FLOAT, STRING, PATH and selectable read as ES-DE 3.4.1 was measured to read them - see EmuSen_BigPicture.md §35.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemeValueRulesTests : IDisposable
    {
        private readonly SyntheticTheme _theme = new();
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenQ70", Guid.NewGuid().ToString("N"));

        public ThemeValueRulesTests() => DataStore.OverrideDirectory = _root;

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

        private static bool Warned(ResolvedTheme theme) =>
            theme.Diagnostics.Any(d => d.Code == ThemeDiagnosticCode.LenientValue && d.Severity == ThemeSeverity.Warning);

        // Where ES-DE drew the colour transparent, only its alpha was measured; the other channels follow the same reading.
        [Theory]
        [InlineData("FF0000", "FF0000FF", false)] [InlineData("ff0000", "FF0000FF", false)] [InlineData("FF000080", "FF000080", false)]
        [InlineData("FF0000FF", "FF0000FF", false)] [InlineData("0000FF80", "0000FF80", false)]
        [InlineData("0xFF0000", "00FF0000", true)] [InlineData("GG0000", "000000FF", true)] [InlineData("FF00ZZ", "00FF00FF", true)]
        [InlineData("ffff00zz", "00FFFF00", true)] [InlineData("FF00G0", "00FF00FF", true)] [InlineData("0000GG", "000000FF", true)]
        [InlineData("GGFF0000", "00000000", true)] [InlineData("0x00FF00", "0000FF00", true)] [InlineData("+0FF0000", "00FF0000", true)]
        [InlineData(" 00FF000", "000FF000", true)] [InlineData("-0000001", "FFFFFFFF", true)] [InlineData("0x0000FF", "000000FF", true)]
        [InlineData("FF00FF 0", "00FF00FF", true)] [InlineData("0xFFFFFF", "00FFFFFF", true)] [InlineData("0xFF00", "00FF00FF", true)]
        [InlineData(" FF000", "0FF000FF", true)] [InlineData("-00001", "FFFFFFFF", true)] [InlineData("+0FF00", "00FF00FF", true)]
        [InlineData("0x00FF", "0000FFFF", true)] [InlineData("0X0000", "000000FF", true)] [InlineData("  FF00", "00FF00FF", true)]
        public void A_colour_of_six_or_eight_characters_is_read_as_hexadecimal_the_C_way(string written, string rgba, bool warned)
        {
            ResolvedTheme theme = Gamelist($"<text name=\"t\"><text>t</text><backgroundColor>{written}</backgroundColor></text>");
            Assert.Equal(rgba, Only(theme, "text", "t").Color("backgroundColor")!.Value.ToString());
            Assert.Equal(warned, Warned(theme));
        }

        [Theory]
        [InlineData("#FF0000")] [InlineData("F00")] [InlineData("FF000")] [InlineData("FF00000")] [InlineData("4c94ff6")]
        [InlineData("FF0000FF0")] [InlineData(" FF0000")] [InlineData("FF0000 ")] [InlineData("red")] [InlineData("0")]
        [InlineData("-FF0000")] [InlineData("1FFFFFFFF")] [InlineData("FFFFFFFFFFFFFFFFFF")]
        public void A_colour_of_any_other_length_unthemes_the_system_as_in_ES_DE(string written)
        {
            ResolvedTheme theme = Gamelist($"<text name=\"t\"><text>t</text><color>{written}</color></text>");
            Assert.False(theme.IsThemed);
            ThemeDiagnostic error = Assert.Single(theme.Errors);
            Assert.Equal(ThemeDiagnosticCode.BadFormat, error.Code);
            Assert.Contains("not 6 or 8 characters long", error.Message);
        }

        [Theory]
        [InlineData("3", 3u, false)] [InlineData("3.5", 3u, true)] [InlineData("3.9", 3u, true)] [InlineData("2.9", 2u, true)]
        [InlineData(" 3", 3u, false)] [InlineData("3 ", 3u, false)] [InlineData("\t3", 3u, false)] [InlineData("+3", 3u, true)]
        [InlineData("-1", 4294967295u, true)] [InlineData("abc", 0u, true)] [InlineData("3abc", 3u, true)] [InlineData("0x3", 3u, true)]
        [InlineData("0xA", 10u, true)] [InlineData("0x", 0u, true)] [InlineData("1e1", 1u, true)] [InlineData(".5", 0u, true)]
        [InlineData("3 4", 3u, true)] [InlineData("3,5", 3u, true)] [InlineData("-0", 0u, true)] [InlineData("4294967296", 0u, true)]
        [InlineData("4294967298", 2u, true)] [InlineData("-4294967294", 2u, true)] [InlineData("4294967295", 4294967295u, false)]
        [InlineData("010", 8u, true)] [InlineData("011", 9u, true)] [InlineData("012", 10u, true)] [InlineData("0010", 8u, true)]
        [InlineData("08", 0u, true)] [InlineData("09", 0u, true)] [InlineData("0x8", 8u, true)] [InlineData("0X9", 9u, true)] [InlineData(" 010", 8u, true)]
        [InlineData("99999999999", 1215752191u, true)] [InlineData("99999999999999999999", 4294967295u, true)]
        public void A_whole_number_is_read_the_C_way_with_its_base_from_its_prefix_and_kept_to_32_bits(string written, uint expected, bool warned)
        {
            Assert.Equal(expected, ThemeValueParser.EsdeUInt(written, out bool exact));
            Assert.Equal(warned, !exact);
        }

        [Theory]
        [InlineData("1", 1u)] [InlineData("10", 10u)] [InlineData("3.5", 3u)] [InlineData("010", 8u)] [InlineData("0xA", 10u)] [InlineData("1e1", 1u)]
        [InlineData("0", null)] [InlineData("11", null)] [InlineData("-1", null)] [InlineData("abc", null)] [InlineData("08", null)] [InlineData("4294967296", null)]
        public void Badge_lines_and_items_per_line_keep_1_to_10_and_reset_the_rest_to_the_default_with_a_warning(string written, uint? expected)
        {
            ResolvedTheme theme = Gamelist($"<badges name=\"b\"><itemsPerLine>{written}</itemsPerLine><lines>{written}</lines></badges>");
            ResolvedElement b = Only(theme, "badges", "b");
            Assert.Equal(expected ?? 4u, b.UInt("itemsPerLine"));
            Assert.Equal(expected ?? 3u, b.UInt("lines"));
            Assert.Equal(expected is null ? 2 : 0, theme.Diagnostics.Count(d => d.Code == ThemeDiagnosticCode.InvalidValue && d.Severity == ThemeSeverity.Warning));
        }

        // P173: Canvas and Iconic write 3.5, which ES-DE draws as 3; the probe drew 1.5 and 1.9 as 1.
        [Theory]
        [InlineData("3.5", 3u)] [InlineData("4.5", 4u)] [InlineData("3.25", 3u)] [InlineData("1.5", 1u)] [InlineData("1.9", 1u)] [InlineData("abc", 0u)] [InlineData("-1", 20u)]
        public void A_carousel_count_written_as_a_fraction_keeps_its_whole_part_and_the_system_themed(string written, uint expected)
        {
            ResolvedTheme theme = Gamelist($"<carousel name=\"c\"><itemsBeforeCenter>{written}</itemsBeforeCenter><itemsAfterCenter>{written}</itemsAfterCenter></carousel>");
            ResolvedElement c = Only(theme, "carousel", "c");
            Assert.Equal(expected, c.UInt("itemsBeforeCenter"));
            Assert.Equal(expected, c.UInt("itemsAfterCenter"));
        }

        [Theory]
        [InlineData("1e-1", 0.1f, false)] [InlineData("5E-2", 0.05f, false)] [InlineData("0x1p-2", 0.25f, true)] [InlineData("0x0.4p0", 0.25f, true)]
        [InlineData("0x0.8", 0.5f, true)] [InlineData(".5", 0.5f, false)] [InlineData("5e-1", 0.5f, false)] [InlineData("+0.5", 0.5f, false)]
        [InlineData("0.1e", 0.1f, true)] [InlineData("0.1e+", 0.1f, true)] [InlineData("0.1e-", 0.1f, true)] [InlineData("0x", 0f, true)]
        [InlineData("0.3.4", 0.3f, true)] [InlineData("-0x0.1p0", -0.0625f, true)] [InlineData("inf", 0f, true)] [InlineData("nan", 0f, true)]
        public void A_float_reads_exponents_and_hexadecimal_and_an_unbounded_infinity_is_0(string written, float x, bool warned)
        {
            ResolvedTheme theme = Gamelist($"<text name=\"t\"><text>t</text><pos>{written} 0.5</pos></text>");
            Assert.Equal(x, Only(theme, "text", "t").Pair("pos")!.Value.X, 6);
            Assert.Equal(warned, Warned(theme));
        }

        [Theory]
        [InlineData("1", 1f)] [InlineData("0", 0f)] [InlineData("0.5", 0.5f)] [InlineData("inf", 1f)] [InlineData("-inf", 0f)] [InlineData("nan", 0f)]
        [InlineData("infinity", 1f)] [InlineData("1e999", 1f)] [InlineData("-1e999", 0f)] [InlineData("0x1p-1", 0.5f)] [InlineData("1e", 1f)]
        [InlineData("NAN", 0f)] [InlineData("INF", 1f)] [InlineData("1E0", 1f)]
        public void An_infinite_float_is_clamped_to_the_range_and_nan_draws_nothing(string written, float opacity)
        {
            ResolvedTheme theme = Gamelist($"<text name=\"t\"><text>t</text><opacity>{written}</opacity></text>");
            Assert.Equal(opacity, Only(theme, "text", "t").Float("opacity")!.Value, 6);
        }

        [Theory]
        [InlineData("right", "right")] [InlineData("center", "center")] [InlineData("left", "left")]
        [InlineData("RIGHT", null)] [InlineData(" right", null)] [InlineData("right ", null)] [InlineData("Right", null)] [InlineData("middle", null)]
        [InlineData("LEFT", null)] [InlineData("Left", null)]
        public void An_enumerated_string_must_match_exactly_and_untrimmed_or_the_default_applies(string written, string? expected)
        {
            ResolvedTheme theme = Gamelist($"<text name=\"t\"><text>t</text><horizontalAlignment>{written}</horizontalAlignment><metadata>{written}</metadata></text>");
            ResolvedElement t = Only(theme, "text", "t");
            Assert.Equal(expected, ((StringValue?)t.Explicit.GetValueOrDefault("horizontalAlignment"))?.Value);
            Assert.False(t.Explicit.ContainsKey("metadata"));
            Assert.Equal(expected is null ? 2 : 1, theme.Diagnostics.Count(d => d.Code == ThemeDiagnosticCode.InvalidValue && d.Severity == ThemeSeverity.Warning));
        }

        [Theory]
        [InlineData(" name")] [InlineData("Name")] [InlineData("name ")] [InlineData("nosuchfield")]
        public void A_metadata_name_that_is_not_exact_leaves_the_text_shown(string written)
        {
            ResolvedTheme theme = Gamelist($"<text name=\"t\"><text>TEXT</text><metadata>{written}</metadata></text>");
            ResolvedElement t = Only(theme, "text", "t");
            Assert.Null(t.String("metadata"));
            Assert.Equal("TEXT", t.String("text"));
        }

        [Fact]
        public void Text_keeps_its_spaces()
        {
            Assert.Equal("  PADDED  ", Only(Gamelist("<text name=\"t\"><text>  PADDED  </text></text>"), "text", "t").String("text"));
        }

        [Theory]
        [InlineData("cover,screenshot", true)] [InlineData("cover screenshot", true)] [InlineData("cover, screenshot", true)]
        [InlineData(" cover", true)] [InlineData("cover ", true)] [InlineData("Cover", false)] [InlineData("boxart", false)]
        [InlineData("cover;screenshot", false)] [InlineData("screenshot,boxart", false)]
        public void An_unknown_imageType_hides_the_element_and_keeps_the_system_themed(string written, bool drawn)
        {
            ResolvedTheme theme = Gamelist($"<image name=\"i\"><imageType>{written}</imageType></image><text name=\"t\"><text>t</text></text>");
            Assert.True(theme.IsThemed, string.Join("; ", theme.Errors));
            Assert.Equal(drawn, theme.GamelistView.Find("image", "i") is not null);
            Assert.NotNull(theme.GamelistView.Find("text", "t"));
        }

        private string Unique(string stem) => $"{stem}-{Guid.NewGuid():N}";

        [Fact]
        public void A_path_is_the_theme_folder_only_after_dot_slash_and_is_not_trimmed()
        {
            string png = Unique("q70") + ".png";
            _theme.Capabilities("").Asset($"img/{png}").Asset($"sub/img/{png}")
                .Theme($"<variables><imgdir>./img/</imgdir></variables><view name=\"gamelist\">" +
                       $"<image name=\"dot\"><path>./img/{png}</path></image><image name=\"bare\"><path>img/{png}</path></image>" +
                       $"<image name=\"back\"><path>.\\img\\{png}</path></image><image name=\"lead\"><path> ./img/{png}</path></image>" +
                       $"<image name=\"trail\"><path>./img/{png} </path></image><image name=\"case\"><path>./IMG/{png.ToUpperInvariant()}</path></image>" +
                       $"<image name=\"up\"><path>./../{Path.GetFileName(_theme.Root)}/img/{png}</path></image><image name=\"upbare\"><path>../{Path.GetFileName(_theme.Root)}/img/{png}</path></image>" +
                       $"<image name=\"here\"><path>./img/./{png}</path></image></view><include>./sub/inc.xml</include>")
                .Theme($"<view name=\"gamelist\"><image name=\"inc\"><path>./img/{png}</path></image><image name=\"var\"><path>${{imgdir}}{png}</path></image></view>", "sub/inc.xml");
            ResolvedTheme theme = _theme.Load();
            ThemePath P(string name) => Only(theme, "image", name).Path("path")!;

            foreach (string found in new[] { "dot", "back", "up", "here" })
            {
                Assert.True(P(found).Exists, found);
                Assert.Equal(_theme.PathOf($"img/{png}"), P(found).Absolute);
            }
            Assert.Equal(_theme.PathOf($"sub/img/{png}"), P("inc").Absolute);
            Assert.Equal(_theme.PathOf($"sub/img/{png}"), P("var").Absolute);
            Assert.Equal(Path.GetFullPath($"img/{png}"), P("bare").Absolute);
            foreach (string missing in new[] { "bare", "lead", "trail", "case", "upbare" })
                Assert.False(P(missing).Exists, missing);
            Assert.Equal(5, theme.Diagnostics.Count(d => d.Code == ThemeDiagnosticCode.PathMissing && d.Severity == ThemeSeverity.Warning));
        }

        [Theory]
        [InlineData("{0}", false)] [InlineData(" ./{0} ", false)] [InlineData("./{0}\n", false)] [InlineData("./{1}", false)]
        [InlineData(".\\{0}", true)] [InlineData("./{0}", true)]
        public void An_include_is_read_untrimmed_and_a_bare_name_is_not_the_theme_folder(string written, bool loads)
        {
            string name = Unique("inc") + ".xml";
            string text = string.Format(written, name, name.ToUpperInvariant());
            _theme.Capabilities("").File(name, "<theme><view name=\"gamelist\"><text name=\"inc\"><text>i</text></text></view></theme>")
                .Theme($"<include>{text}</include><view name=\"gamelist\"><text name=\"t\"><text>t</text></text></view>");
            ResolvedTheme theme = _theme.Load();
            Assert.Equal(loads, theme.IsThemed);
            if (loads) Assert.NotNull(theme.GamelistView.Find("text", "inc"));
            else Assert.Contains(theme.Errors, d => d.Code == ThemeDiagnosticCode.IncludeMissing);
        }

        [Theory]
        [InlineData("true", true)] [InlineData("yes", true)] [InlineData("1", true)] [InlineData("TRUE", true)] [InlineData(" true ", true)]
        [InlineData("t", true)] [InlineData("Y", true)] [InlineData("", true)] [InlineData("  ", true)] [InlineData("x", true)] [InlineData("2", true)]
        [InlineData("on", true)] [InlineData(" false", true)] [InlineData(" no", true)] [InlineData("\ntrue\n", true)] [InlineData(" yes", true)]
        [InlineData("\tfalse", true)] [InlineData("flase", false)] [InlineData("0", false)] [InlineData("no", false)] [InlineData("false", false)]
        [InlineData("N", false)] [InlineData("f", false)] [InlineData("F", false)] [InlineData("False", false)] [InlineData("NO", false)]
        [InlineData("0.0", false)] [InlineData("FALSE", false)]
        public void Selectable_is_false_only_when_its_first_character_is_0_f_or_n(string written, bool selectable)
        {
            _theme.Capabilities($"<variant name=\"v\"><selectable>{written}</selectable></variant>" +
                                $"<transitions name=\"p\"><selectable>{written}</selectable><systemToGamelist>fade</systemToGamelist></transitions>").Theme("");
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(_theme.Root);
            Assert.Equal(selectable, caps.Variants.Single().Selectable);
            Assert.Equal(selectable, caps.Transitions.Single().Selectable);
        }

        // Q71: in ES-DE a variant without <selectable> is not offered, while a transitions profile without one is.
        [Fact]
        public void A_variant_without_selectable_is_not_offered_and_a_profile_without_one_is()
        {
            _theme.Capabilities("<variant name=\"aNone\"/><variant name=\"bTrue\"><selectable>true</selectable></variant>" +
                                "<transitions name=\"tNone\"><systemToGamelist>fade</systemToGamelist></transitions>").Theme("");
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(_theme.Root);
            Assert.Equal(["bTrue"], caps.Variants.Where(v => v.Selectable).Select(v => v.Name));
            Assert.True(caps.Transitions.Single().Selectable);
            Assert.Equal("bTrue", ThemeSelection.Resolve(caps, new ThemeChoices()).Variant);
            Assert.Equal("aNone", ThemeSelection.Resolve(caps, new ThemeChoices { Variant = "aNone" }).Variant);
            Assert.Empty(caps.Diagnostics);
        }

        [Fact]
        public void With_no_variant_offered_the_first_declared_is_drawn()
        {
            _theme.Capabilities("<variant name=\"cNone\"/><variant name=\"dNone\"/>").Theme("");
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(_theme.Root);
            Assert.DoesNotContain(caps.Variants, v => v.Selectable);
            Assert.Equal("cNone", ThemeSelection.Resolve(caps, new ThemeChoices()).Variant);
        }

        private static FakeTheme Case(string repository, string gamelist)
        {
            var t = new FakeTheme { Name = repository, Source = new ThemeSource("q70", repository, "main") };
            t.ThemeText = "<theme><view name=\"system\"><carousel name=\"c\"><pos>0 0.2</pos><size>1 0.5</size></carousel></view>" +
                          "<view name=\"gamelist\"><textlist name=\"l\"><pos>0.05 0.1</pos><size>0.5 0.8</size></textlist>" + gamelist + "</view></theme>";
            return t;
        }

        [Fact]
        public void A_download_installs_where_ES_DE_reads_the_value_and_is_refused_where_it_does_not()
        {
            var hosts = new FakeThemeHosts();
            FakeTheme fraction = Case("fraction-es-de", "<carousel name=\"g\"><itemsBeforeCenter>3.5</itemsBeforeCenter></carousel>");
            FakeTheme imageType = Case("imagetype-es-de", "<image name=\"i\"><imageType>boxart</imageType></image>");
            FakeTheme colour = Case("colour-es-de", "<text name=\"t\"><text>t</text><color>GG0000</color></text>");
            FakeTheme hash = Case("hash-es-de", "<text name=\"t\"><text>t</text><color>#FF0000</color></text>");
            hosts.Themes.AddRange([fraction, imageType, colour, hash]);

            foreach (FakeTheme accepted in new[] { fraction, imageType, colour })
            {
                hosts.Install(accepted.Source);
                Assert.True(File.Exists(Path.Combine(ThemeDownloads.DirectoryFor(accepted.Source), "theme.xml")), accepted.Name);
            }
            var refused = Assert.Throws<InvalidDataException>(() => hosts.Install(hash.Source));
            Assert.Contains("\"#FF0000\", which is not 6 or 8 characters long", refused.Message);
            Assert.False(Directory.Exists(ThemeDownloads.DirectoryFor(hash.Source)));
        }
    }
}
