using System;
using System.Linq;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // THEMES.md's error behaviour: what unthemes a system, what resets a value, what is silent - see EmuSen_BigPicture.md §12.3.
    public class ThemeErrorTests : IDisposable
    {
        private readonly SyntheticTheme _theme = new();

        public void Dispose() => _theme.Dispose();

        private ResolvedTheme LoadView(string elements, string view = "gamelist")
        {
            _theme.Capabilities("").Theme($"<view name=\"{view}\">{elements}</view>");
            return _theme.Load();
        }

        private static void AssertUnthemed(ResolvedTheme theme, ThemeDiagnosticCode code)
        {
            Assert.False(theme.IsThemed);
            Assert.Contains(theme.Errors, d => d.Code == code);
        }

        [Fact]
        public void Malformed_XML_unthemes_the_system()
        {
            _theme.Capabilities("").Theme("<view name=\"gamelist\"><text name=\"t\"><text>a && b</text></text></view>");
            AssertUnthemed(_theme.Load(), ThemeDiagnosticCode.MalformedXml);
        }

        [Fact]
        public void Malformed_XML_in_an_included_file_unthemes_the_system()
        {
            _theme.Capabilities("").File("colors.xml", "<theme><view name=\"gamelist\"></theme>").Theme("<include>./colors.xml</include>");
            AssertUnthemed(_theme.Load(), ThemeDiagnosticCode.MalformedXml);
        }

        [Fact]
        public void Malformed_capabilities_unthemes_every_system()
        {
            _theme.File("capabilities.xml", "<themeCapabilities><variant name=\"a\"></themeCapabilities>").Theme("");
            AssertUnthemed(_theme.Load(), ThemeDiagnosticCode.MalformedXml);
        }

        [Fact]
        public void A_file_whose_root_is_not_theme_unthemes_the_system()
        {
            _theme.Capabilities("").File("theme.xml", "<themes><view name=\"gamelist\"/></themes>");
            AssertUnthemed(_theme.Load(), ThemeDiagnosticCode.WrongRoot);
        }

        [Fact]
        public void A_missing_theme_file_unthemes_the_system()
        {
            _theme.Capabilities("");
            AssertUnthemed(_theme.Load(), ThemeDiagnosticCode.NoThemeFile);
        }

        [Fact]
        public void An_unknown_element_unthemes_the_system()
        {
            AssertUnthemed(LoadView("<stackpanel name=\"s\"><pos>0 0</pos></stackpanel>"), ThemeDiagnosticCode.UnknownElement);
        }

        [Fact]
        public void An_unknown_property_unthemes_the_system()
        {
            AssertUnthemed(LoadView("<image name=\"i\"><logoAlignment>left</logoAlignment></image>"), ThemeDiagnosticCode.UnknownProperty);
        }

        [Fact]
        public void An_unknown_tag_at_theme_level_unthemes_the_system()
        {
            _theme.Capabilities("").Theme("<formatVersion>7</formatVersion>");
            AssertUnthemed(_theme.Load(), ThemeDiagnosticCode.UnknownTag);
        }

        [Fact]
        public void The_legacy_extra_attribute_unthemes_the_system()
        {
            AssertUnthemed(LoadView("<image name=\"i\" extra=\"true\"><pos>0 0</pos></image>"), ThemeDiagnosticCode.LegacySyntax);
        }

        [Fact]
        public void An_unknown_view_unthemes_the_system()
        {
            AssertUnthemed(LoadView("<image name=\"i\"><pos>0 0</pos></image>", "detailed"), ThemeDiagnosticCode.UnknownView);
        }

        [Fact]
        public void An_element_without_a_name_unthemes_the_system()
        {
            AssertUnthemed(LoadView("<image><pos>0 0</pos></image>"), ThemeDiagnosticCode.MissingName);
        }

        [Fact]
        public void A_property_with_no_value_unthemes_the_system()
        {
            AssertUnthemed(LoadView("<image name=\"i\"><origin></origin></image>"), ThemeDiagnosticCode.NoValue);
        }

        [Theory]
        [InlineData("<image name=\"i\"><pos>0.5</pos></image>")]
        [InlineData("<image name=\"i\"><pos>0.5 0.5 0.5</pos></image>")]
        [InlineData("<image name=\"i\"><color>FFF</color></image>")]
        [InlineData("<image name=\"i\"><color>GGGGGG</color></image>")]
        [InlineData("<image name=\"i\"><tile>yes</tile></image>")]
        [InlineData("<image name=\"i\"><rotation>ninety</rotation></image>")]
        [InlineData("<badges name=\"b\"><lines>-1</lines></badges>")]
        [InlineData("<badges name=\"b\"><lines>1.5</lines></badges>")]
        public void A_value_in_the_wrong_format_unthemes_the_system(string element)
        {
            AssertUnthemed(LoadView(element), ThemeDiagnosticCode.BadFormat);
        }

        [Fact]
        public void An_invalid_value_of_a_valid_format_is_reset_to_its_default_with_a_warning()
        {
            ResolvedTheme theme = LoadView("<badges name=\"b\"><horizontalAlignment>leftr</horizontalAlignment><direction>diagonal</direction></badges>");
            Assert.True(theme.IsThemed);
            ResolvedElement b = theme.GamelistView.Find("badges", "b")!;
            Assert.Null(b.String("horizontalAlignment"));
            Assert.Equal("row", b.String("direction"));
            Assert.Equal(2, theme.Diagnostics.Count(d => d.Code == ThemeDiagnosticCode.InvalidValue && d.Severity == ThemeSeverity.Warning));
        }

        [Fact]
        public void An_invalid_imageType_stops_the_element_rendering()
        {
            ResolvedTheme theme = LoadView("<video name=\"gamelistVideo\"><imageType>covr</imageType></video><image name=\"kept\"><imageType>cover</imageType></image>");
            Assert.Null(theme.GamelistView.Find("video", "gamelistVideo"));
            Assert.NotNull(theme.GamelistView.Find("image", "kept"));
            Assert.Contains(theme.Diagnostics, d => d.Code == ThemeDiagnosticCode.InvalidImageType && d.Severity == ThemeSeverity.Error);
        }

        [Fact]
        public void A_repeated_imageType_is_ignored()
        {
            ResolvedTheme theme = LoadView("<image name=\"i\"><imageType>cover, screenshot cover</imageType></image>");
            Assert.True(theme.IsThemed);
            Assert.Empty(theme.GamelistView.Find("image", "i")!.List("imageType"));
        }

        [Fact]
        public void A_carousel_uses_at_most_two_imageTypes()
        {
            ResolvedTheme theme = LoadView("<carousel name=\"c\"><imageType>cover screenshot marquee</imageType></carousel>");
            Assert.Equal(["cover", "screenshot"], theme.GamelistView.Find("carousel", "c")!.List("imageType"));
        }

        [Fact]
        public void An_element_in_a_view_it_does_not_support_is_ignored_with_a_warning()
        {
            ResolvedTheme theme = LoadView("<badges name=\"b\"><pos>0 0</pos></badges><gamelistinfo name=\"g\"><pos>0 0</pos></gamelistinfo>", "system");
            Assert.True(theme.IsThemed);
            Assert.Empty(theme.SystemView.Elements);
            Assert.Equal(2, theme.Diagnostics.Count(d => d.Code == ThemeDiagnosticCode.ViewRestricted));
        }

        [Fact]
        public void A_property_limited_to_the_other_view_is_ignored_with_a_warning()
        {
            _theme.Capabilities("").Asset("a.png").Theme("""
                <view name="system, gamelist"><carousel name="c"><staticImage>./a.png</staticImage><imageType>cover</imageType></carousel></view>
                """);
            ResolvedTheme theme = _theme.Load();
            Assert.True(theme.IsThemed);
            Assert.NotNull(theme.SystemView.Find("carousel", "c")!.Path("staticImage"));
            Assert.Empty(theme.SystemView.Find("carousel", "c")!.List("imageType"));
            Assert.Null(theme.GamelistView.Find("carousel", "c")!.Path("staticImage"));
            Assert.Equal(["cover"], theme.GamelistView.Find("carousel", "c")!.List("imageType"));
            Assert.Equal(2, theme.Diagnostics.Count(d => d.Code == ThemeDiagnosticCode.ViewRestricted));
        }

        [Fact]
        public void A_second_primary_element_in_a_view_is_ignored_with_a_warning()
        {
            ResolvedTheme theme = LoadView("<textlist name=\"list\"><pos>0 0</pos></textlist><carousel name=\"c\"><pos>0 0</pos></carousel>");
            Assert.Equal("list", theme.GamelistView.Primary!.Name);
            Assert.Null(theme.GamelistView.Find("carousel", "c"));
            Assert.Contains(theme.Diagnostics, d => d.Code == ThemeDiagnosticCode.SecondPrimary);
        }

        [Fact]
        public void A_missing_file_written_out_is_a_warning_and_one_from_a_variable_is_a_debug_note()
        {
            _theme.Capabilities("").Theme("""
                <variables><logos>./logos</logos></variables>
                <view name="gamelist"><image name="i"><path>./none.png</path><default>${logos}/none.svg</default></image></view>
                """);
            ResolvedTheme theme = _theme.Load();
            Assert.True(theme.IsThemed);
            ResolvedElement image = theme.GamelistView.Find("image", "i")!;
            Assert.False(image.Path("path")!.Exists);
            Assert.True(image.Path("default")!.FromVariable);
            Assert.Single(theme.Diagnostics, d => d.Code == ThemeDiagnosticCode.PathMissing && d.Severity == ThemeSeverity.Warning);
            Assert.Single(theme.Diagnostics, d => d.Code == ThemeDiagnosticCode.PathMissing && d.Severity == ThemeSeverity.Debug);
        }

        [Fact]
        public void A_keyed_property_needs_its_key_and_ignores_an_unknown_one()
        {
            ResolvedTheme missing = LoadView("<badges name=\"b\"><customBadgeIcon>./x.svg</customBadgeIcon></badges>");
            AssertUnthemed(missing, ThemeDiagnosticCode.MissingName);

            ResolvedTheme unknown = LoadView("<badges name=\"b\"><customBadgeIcon badge=\"shiny\">./x.svg</customBadgeIcon><customBadgeIcon badge=\"favorite\">./f.svg</customBadgeIcon></badges>");
            Assert.True(unknown.IsThemed);
            Assert.Equal(["favorite"], unknown.GamelistView.Find("badges", "b")!.Keyed("customBadgeIcon").Keys);
        }

        [Fact]
        public void A_property_made_empty_by_its_variables_is_ignored_with_a_warning()
        {
            _theme.Capabilities("").Theme("<variables><maker/></variables><view name=\"gamelist\"><text name=\"t\"><text>${maker}</text></text></view>");
            ResolvedTheme theme = _theme.Load();
            Assert.True(theme.IsThemed);
            Assert.Null(theme.GamelistView.Find("text", "t")!.String("text"));
            Assert.Contains(theme.Diagnostics, d => d.Code == ThemeDiagnosticCode.NoValue && d.Severity == ThemeSeverity.Warning);
        }

        [Fact]
        public void An_unused_attribute_is_ignored_with_a_warning()
        {
            ResolvedTheme theme = LoadView("<image name=\"i\" ifSubset=\"x\"><pos>0 0</pos></image>");
            Assert.True(theme.IsThemed);
            Assert.Contains(theme.Diagnostics, d => d.Code == ThemeDiagnosticCode.UnusedAttribute);
        }

        [Fact]
        public void A_Batocera_edition_theme_does_not_load()
        {
            _theme.Capabilities("").Theme("""
                <formatVersion>7</formatVersion>
                <subset name="ratio" appliesTo="system"><include name="16/10">./16-10.xml</include></subset>
                <view name="system"><stackpanel name="s"/></view>
                """);
            ResolvedTheme theme = _theme.Load();
            Assert.False(theme.IsThemed);
            Assert.Contains(theme.Errors, d => d.Code == ThemeDiagnosticCode.UnknownElement);
            Assert.True(theme.Errors.Count(d => d.Code == ThemeDiagnosticCode.UnknownTag) >= 2);
        }
    }
}
