using System;
using System.IO;
using System.Linq;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The theme grammar on synthetic themes: parsing order, includes, variables, selection blocks and merging - see EmuSen_BigPicture.md §12.2.
    public class ThemeLoaderTests : IDisposable
    {
        private readonly SyntheticTheme _theme = new();

        public void Dispose() => _theme.Dispose();

        private const string Selections = """
            <variant name="lightMode"/><variant name="lightModeNoVideo"/><variant name="dark"/>
            <colorScheme name="blue"/><colorScheme name="red"/>
            <fontSize>medium</fontSize><fontSize>small</fontSize>
            <aspectRatio>16:9</aspectRatio><aspectRatio>4:3</aspectRatio><aspectRatio>16:10</aspectRatio>
            """;

        private static string Color(ResolvedTheme theme, string name, string view = "gamelist") =>
            theme.View(view).Find("text", name)!.Color("color")!.Value.ToString();

        [Fact]
        public void THEMES_md_variables_example_general_config_is_parsed_before_the_variant_redefines_the_variable()
        {
            _theme.Capabilities(Selections).Theme("""
                <variables>
                    <colorRed>8b0000</colorRed>
                    <themeColor>${colorRed}</themeColor>
                </variables>
                <variant name="lightMode, lightModeNoVideo">
                    <variables><themeColor>6533ff</themeColor></variables>
                </variant>
                <view name="gamelist">
                    <text name="infoText01"><pos>0.3 0.56</pos><color>${themeColor}</color></text>
                </view>
                <variant name="lightMode, lightModeNoVideo">
                    <view name="gamelist"><text name="gameName"><pos>0.8 0.12</pos><color>${themeColor}</color></text></view>
                </variant>
                """);
            ResolvedTheme theme = _theme.Load(new ThemeChoices { Variant = "lightMode" });
            Assert.True(theme.IsThemed);
            Assert.Equal("8B0000FF", Color(theme, "infoText01"));
            Assert.Equal("6533FFFF", Color(theme, "gameName"));
        }

        [Fact]
        public void Variant_configuration_written_above_general_configuration_is_still_applied_after_it()
        {
            _theme.Capabilities(Selections).Theme("""
                <variant name="dark"><view name="gamelist"><text name="t"><color>000000</color></text></view></variant>
                <view name="gamelist"><text name="t"><color>FFFFFF</color></text></view>
                """);
            Assert.Equal("000000FF", Color(_theme.Load(new ThemeChoices { Variant = "dark" }), "t"));
            Assert.Equal("FFFFFFFF", Color(_theme.Load(new ThemeChoices { Variant = "lightMode" }), "t"));
        }

        [Fact]
        public void Aspect_ratio_configuration_is_applied_after_variants()
        {
            _theme.Capabilities(Selections).Theme("""
                <aspectRatio name="4:3"><view name="gamelist"><text name="t"><color>0000AA</color></text></view></aspectRatio>
                <variant name="all"><view name="gamelist"><text name="t"><color>00AA00</color></text></view></variant>
                <view name="gamelist"><text name="t"><color>AA0000</color></text></view>
                """);
            Assert.Equal("0000AAFF", Color(_theme.Load(new ThemeChoices { AspectRatio = "4:3" }), "t"));
            Assert.Equal("00AA00FF", Color(_theme.Load(new ThemeChoices { AspectRatio = "16:9" }), "t"));
        }

        [Fact]
        public void The_steps_run_in_the_documented_order_whatever_the_file_order()
        {
            // 2 variables, 3 colour schemes, 4 font sizes, 6 includes, 7 general, 8 variants, 9 aspect ratios.
            _theme.Capabilities(Selections)
                .Theme("<variables><fromInclude>${fromFontSize}</fromInclude></variables>", "inc.xml")
                .Theme("""
                    <aspectRatio name="16:9"><variables><v>ratio</v></variables></aspectRatio>
                    <variant name="dark"><variables><v>${v}+variant</v></variables></variant>
                    <view name="gamelist"><text name="general"><text>${v}|${fromInclude}</text></text></view>
                    <include>./inc.xml</include>
                    <fontSize name="medium"><variables><fromFontSize>${fromScheme}+size</fromFontSize></variables></fontSize>
                    <colorScheme name="blue"><variables><fromScheme>${v}+scheme</fromScheme></variables></colorScheme>
                    <variables><v>base</v></variables>
                    <aspectRatio name="16:9"><view name="gamelist"><text name="ratio"><text>${v}</text></text></view></aspectRatio>
                    <variant name="dark"><view name="gamelist"><text name="variant"><text>${v}</text></text></view></variant>
                    """);
            ResolvedTheme theme = _theme.Load(new ThemeChoices { Variant = "dark", AspectRatio = "16:9" });
            Assert.True(theme.IsThemed, string.Join("\n", theme.Diagnostics));
            Assert.Equal("base|base+scheme+size", theme.GamelistView.Find("text", "general")!.String("text"));
            Assert.Equal("base+variant", theme.GamelistView.Find("text", "variant")!.String("text"));
            Assert.Equal("ratio", theme.GamelistView.Find("text", "ratio")!.String("text"));
        }

        [Fact]
        public void An_include_runs_all_nine_steps_at_its_own_place()
        {
            _theme.Capabilities(Selections)
                .Theme("""
                    <variables><c>111111</c></variables>
                    <view name="gamelist"><text name="t"><color>${c}</color></text></view>
                    <variant name="dark"><view name="gamelist"><text name="v"><color>${c}</color></text></view></variant>
                    """, "inc.xml")
                .Theme("""
                    <variables><c>222222</c></variables>
                    <include>./inc.xml</include>
                    <view name="gamelist"><text name="after"><color>${c}</color></text></view>
                    """);
            ResolvedTheme theme = _theme.Load(new ThemeChoices { Variant = "dark" });
            Assert.Equal("111111FF", Color(theme, "t"));
            Assert.Equal("111111FF", Color(theme, "v"));
            Assert.Equal("111111FF", Color(theme, "after"));
        }

        [Fact]
        public void Colour_scheme_blocks_apply_in_file_order_the_last_matching_winning()
        {
            _theme.Capabilities(Selections).Theme("""
                <variables><c>000000</c></variables>
                <colorScheme name="blue, red"><variables><c>0000FF</c><d>0000FF</d></variables></colorScheme>
                <colorScheme name="red"><variables><c>FF0000</c></variables></colorScheme>
                <view name="gamelist"><text name="c"><color>${c}</color></text><text name="d"><color>${d}</color></text></view>
                """);
            ResolvedTheme blue = _theme.Load(new ThemeChoices { ColorScheme = "blue" });
            ResolvedTheme red = _theme.Load(new ThemeChoices { ColorScheme = "red" });
            Assert.Equal("0000FFFF", Color(blue, "c"));
            Assert.Equal("FF0000FF", Color(red, "c"));
            Assert.Equal("0000FFFF", Color(red, "d"));
        }

        [Fact]
        public void A_colour_scheme_inside_a_variant_or_aspect_ratio_applies_only_there()
        {
            _theme.Capabilities(Selections).Theme("""
                <variables><c>000000</c></variables>
                <variant name="dark"><colorScheme name="red"><variables><c>FF0000</c></variables></colorScheme>
                    <view name="gamelist"><text name="v"><color>${c}</color></text></view></variant>
                <aspectRatio name="4:3"><colorScheme name="red"><variables><c>AA0000</c></variables></colorScheme>
                    <view name="gamelist"><text name="a"><color>${c}</color></text></view></aspectRatio>
                """);
            ResolvedTheme theme = _theme.Load(new ThemeChoices { Variant = "dark", ColorScheme = "red", AspectRatio = "4:3" });
            Assert.Equal("FF0000FF", Color(theme, "v"));
            Assert.Equal("AA0000FF", Color(theme, "a"));
            Assert.Equal("000000FF", Color(_theme.Load(new ThemeChoices { Variant = "dark", ColorScheme = "blue", AspectRatio = "4:3" }), "v"));
        }

        [Fact]
        public void Font_size_blocks_follow_the_choice_and_name_lists()
        {
            _theme.Capabilities(Selections).Theme("""
                <fontSize name="medium"><variables><s>0.03</s></variables></fontSize>
                <fontSize name="small, x-small"><variables><s>0.02</s></variables></fontSize>
                <view name="gamelist"><text name="t"><fontSize>${s}</fontSize></text></view>
                """);
            Assert.Equal(0.03f, _theme.Load().GamelistView.Find("text", "t")!.Float("fontSize"));
            Assert.Equal(0.02f, _theme.Load(new ThemeChoices { FontSize = "small" }).GamelistView.Find("text", "t")!.Float("fontSize"));
        }

        [Fact]
        public void A_colour_scheme_or_font_size_may_hold_only_variables()
        {
            _theme.Capabilities(Selections).Theme("""
                <colorScheme name="blue"><panelColor>74747488</panelColor></colorScheme>
                <view name="gamelist"><text name="t"><color>${panelColor}</color></text></view>
                """);
            ResolvedTheme theme = _theme.Load();
            Assert.Equal("74747488", Color(theme, "t"));
            Assert.Contains(theme.Diagnostics, d => d.Code == ThemeDiagnosticCode.MisplacedTag && d.Severity == ThemeSeverity.Warning);
        }

        [Fact]
        public void Language_blocks_apply_for_the_chosen_declared_language_and_are_skipped_when_none_is_declared()
        {
            const string body = """
                <language name="en_US"><variables><label>Developer</label></variables></language>
                <language name="sv_SE"><variables><label>Utvecklare</label></variables><include>./sv.xml</include></language>
                <variables><label>none</label></variables>
                <view name="gamelist"><text name="t"><text>${label}</text></text></view>
                """;
            _theme.Capabilities("<language>en_US</language><language>sv_SE</language>").Theme(body)
                .Theme("<view name=\"gamelist\"><text name=\"sv\"><text>ja</text></text></view>", "sv.xml");
            Assert.Equal("Developer", _theme.Load().GamelistView.Find("text", "t")!.String("text"));
            ResolvedTheme swedish = _theme.Load(new ThemeChoices { Language = "sv_SE" });
            Assert.Equal("Utvecklare", swedish.GamelistView.Find("text", "t")!.String("text"));
            Assert.NotNull(swedish.GamelistView.Find("text", "sv"));

            _theme.Capabilities("");
            Assert.Equal("none", _theme.Load(new ThemeChoices { Language = "sv_SE" }).GamelistView.Find("text", "t")!.String("text"));
        }

        [Fact]
        public void Nested_variables_resolve_through_every_level()
        {
            _theme.Capabilities("").Theme("""
                <variables>
                    <a>12</a>
                    <b>${a}34</b>
                    <c>${b}56</c>
                </variables>
                <view name="gamelist"><text name="t"><color>${c}</color></text><text name="u"><color>${c}C0</color></text></view>
                """);
            ResolvedTheme theme = _theme.Load();
            Assert.True(theme.IsThemed, string.Join("\n", theme.Diagnostics));
            Assert.Equal("123456FF", Color(theme, "t"));
            Assert.Equal("123456C0", Color(theme, "u"));
        }

        [Fact]
        public void A_variable_is_substituted_as_it_was_when_it_was_defined()
        {
            _theme.Capabilities("").Theme("""
                <variables><a>111111</a><b>${a}</b><a>222222</a></variables>
                <view name="gamelist"><text name="b"><color>${b}</color></text><text name="a"><color>${a}</color></text></view>
                """);
            ResolvedTheme theme = _theme.Load();
            Assert.Equal("111111FF", Color(theme, "b"));
            Assert.Equal("222222FF", Color(theme, "a"));
        }

        [Fact]
        public void The_twelve_system_variables_have_their_kind_specific_forms_empty_for_other_kinds()
        {
            _theme.Capabilities("").Theme("""
                <view name="system">
                    <text name="n"><text>[${system.name}|${system.fullName}|${system.theme}]</text></text>
                    <text name="k"><text>[${system.name.noCollections}|${system.fullName.autoCollections}|${system.theme.customCollections}]</text></text>
                </view>
                """);
            ResolvedTheme snes = _theme.Load();
            Assert.Equal("[snes|Super Nintendo|snes]", snes.SystemView.Find("text", "n")!.String("text"));
            Assert.Equal("[snes||]", snes.SystemView.Find("text", "k")!.String("text"));

            ResolvedTheme favourites = _theme.Load(system: new ThemeSystem("favorites", "Favorites", "auto-favorites", ThemeSystemKind.AutoCollection));
            Assert.Equal("[|Favorites|]", favourites.SystemView.Find("text", "k")!.String("text"));
            ResolvedTheme custom = _theme.Load(system: new ThemeSystem("mine", "Mine", "custom-collections", ThemeSystemKind.CustomCollection));
            Assert.Equal("[||custom-collections]", custom.SystemView.Find("text", "k")!.String("text"));
        }

        [Fact]
        public void An_undefined_variable_in_a_property_unthemes_the_system()
        {
            _theme.Capabilities("").Theme("<view name=\"gamelist\"><text name=\"t\"><color>${nowhere}</color></text></view>");
            ResolvedTheme theme = _theme.Load();
            Assert.False(theme.IsThemed);
            Assert.Contains(theme.Errors, d => d.Code == ThemeDiagnosticCode.UndefinedVariable);
        }

        [Fact]
        public void A_variable_redefined_in_a_variant_or_aspect_ratio_changes_the_global_one()
        {
            _theme.Capabilities(Selections).Theme("""
                <variables><c>000000</c></variables>
                <variant name="dark"><variables><c>0000FF</c></variables></variant>
                <aspectRatio name="4:3"><view name="gamelist"><text name="t"><color>${c}</color></text></view></aspectRatio>
                """);
            Assert.Equal("0000FFFF", Color(_theme.Load(new ThemeChoices { Variant = "dark", AspectRatio = "4:3" }), "t"));
        }

        [Fact]
        public void Include_paths_resolve_from_the_including_file_not_the_theme_root()
        {
            _theme.Capabilities("")
                .Asset("core/images/frame.png")
                .Asset("snes/logo.png")
                .Theme("<include>./../core/fonts.xml</include><view name=\"gamelist\"><image name=\"logo\"><path>./logo.png</path></image></view>", "snes/theme.xml")
                .Theme("<include>./nested/deeper.xml</include>", "core/fonts.xml")
                .Theme("<view name=\"gamelist\"><image name=\"frame\"><path>./../images/frame.png</path></image></view>", "core/nested/deeper.xml")
                .Theme("");
            ResolvedTheme theme = _theme.Load();
            Assert.True(theme.IsThemed, string.Join("\n", theme.Diagnostics));
            Assert.Equal(_theme.PathOf("core/images/frame.png"), theme.GamelistView.Find("image", "frame")!.Path("path")!.Absolute);
            Assert.Equal(_theme.PathOf("snes/logo.png"), theme.GamelistView.Find("image", "logo")!.Path("path")!.Absolute);
            Assert.True(theme.GamelistView.Find("image", "frame")!.Path("path")!.Exists);
        }

        [Fact]
        public void A_system_folder_theme_xml_is_used_before_the_root_one()
        {
            _theme.Capabilities("")
                .Theme("<view name=\"system\"><text name=\"which\"><text>root</text></text></view>")
                .Theme("<view name=\"system\"><text name=\"which\"><text>snes</text></text></view>", "snes/theme.xml");
            Assert.Equal("snes", _theme.Load().SystemView.Find("text", "which")!.String("text"));
            Assert.Equal("root", _theme.Load(system: new ThemeSystem("nes", "Nintendo", "nes")).SystemView.Find("text", "which")!.String("text"));
        }

        [Fact]
        public void Backslashes_and_tilde_are_resolved_as_documented()
        {
            string file = _theme.PathOf("theme.xml");
            Assert.Equal(_theme.PathOf("core/a.png"), ThemeValueParser.ResolvePath(".\\core\\a.png", file));
            string home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            Assert.Equal(Path.GetFullPath(home + "/themes/x.png"), ThemeValueParser.ResolvePath("~/themes/x.png", file));
            Assert.Equal("/abs/x.png", ThemeValueParser.ResolvePath("/abs/x.png", file));
        }

        [Fact]
        public void A_missing_include_written_out_is_an_error()
        {
            _theme.Capabilities("").Theme("<include>./colors_dark.xml</include>");
            ResolvedTheme theme = _theme.Load();
            Assert.False(theme.IsThemed);
            Assert.Contains(theme.Errors, d => d.Code == ThemeDiagnosticCode.IncludeMissing);
        }

        [Fact]
        public void A_missing_include_built_from_a_variable_is_skipped_silently()
        {
            _theme.Capabilities("").Theme("<include>./${system.theme}/colors.xml</include><view name=\"system\"><text name=\"t\"><text>x</text></text></view>");
            ResolvedTheme theme = _theme.Load();
            Assert.True(theme.IsThemed);
            ThemeDiagnostic skipped = Assert.Single(theme.Diagnostics);
            Assert.Equal((ThemeSeverity.Debug, ThemeDiagnosticCode.IncludeSkipped), (skipped.Severity, skipped.Code));
        }

        [Fact]
        public void An_include_naming_an_undefined_variable_is_skipped_as_Art_Book_Next_colors_xml_needs()
        {
            _theme.Capabilities("<colorScheme name=\"dark\"/><colorScheme name=\"custom\"/>")
                .Theme("<view name=\"gamelist\"><text name=\"custom\"><text>yes</text></text></view>", "custom/colors.xml")
                .Theme("""
                    <colorScheme name="custom"><variables><customizationPath>./custom/colors.xml</customizationPath></variables></colorScheme>
                    <include>${customizationPath}</include>
                    <view name="gamelist"><text name="t"><text>x</text></text></view>
                    """);
            ResolvedTheme dark = _theme.Load(new ThemeChoices { ColorScheme = "dark" });
            Assert.True(dark.IsThemed);
            Assert.Single(dark.Diagnostics, d => d.Code == ThemeDiagnosticCode.IncludeSkipped && d.Severity == ThemeSeverity.Debug);
            Assert.Null(dark.GamelistView.Find("text", "custom"));

            ResolvedTheme custom = _theme.Load(new ThemeChoices { ColorScheme = "custom" });
            Assert.True(custom.IsThemed);
            Assert.NotNull(custom.GamelistView.Find("text", "custom"));
        }

        [Fact]
        public void A_circular_include_is_refused_rather_than_hanging()
        {
            _theme.Capabilities("")
                .Theme("<include>./b.xml</include>", "a.xml")
                .Theme("<include>./a.xml</include>", "b.xml")
                .Theme("<include>./a.xml</include>");
            ResolvedTheme theme = _theme.Load();
            Assert.False(theme.IsThemed);
            Assert.Contains(theme.Errors, d => d.Code == ThemeDiagnosticCode.IncludeLoop);
        }

        [Fact]
        public void The_same_file_may_be_included_twice_without_being_a_loop()
        {
            _theme.Capabilities(Selections)
                .Theme("<view name=\"gamelist\"><text name=\"t\"><text>x</text></text></view>", "shared.xml")
                .Theme("<include>./shared.xml</include><variant name=\"dark\"><include>./shared.xml</include></variant>");
            Assert.True(_theme.Load(new ThemeChoices { Variant = "dark" }).IsThemed);
        }

        [Fact]
        public void Includes_are_allowed_in_theme_variant_and_aspect_ratio_but_not_view()
        {
            _theme.Capabilities(Selections)
                .Theme("<view name=\"gamelist\"><text name=\"v\"><text>v</text></text></view>", "v.xml")
                .Theme("<view name=\"gamelist\"><text name=\"a\"><text>a</text></text></view>", "a.xml")
                .Theme("<variant name=\"dark\"><include>./v.xml</include></variant><aspectRatio name=\"4:3\"><include>./a.xml</include></aspectRatio>");
            ResolvedTheme theme = _theme.Load(new ThemeChoices { Variant = "dark", AspectRatio = "4:3" });
            Assert.True(theme.IsThemed);
            Assert.NotNull(theme.GamelistView.Find("text", "v"));
            Assert.NotNull(theme.GamelistView.Find("text", "a"));

            _theme.Theme("<view name=\"gamelist\"><include>./v.xml</include></view>");
            ResolvedTheme bad = _theme.Load();
            Assert.False(bad.IsThemed);
            Assert.Contains(bad.Errors, d => d.Code == ThemeDiagnosticCode.MisplacedTag);
        }

        [Theory]
        [InlineData("<view name=\"gamelist\"><variant name=\"dark\"/></view>")]
        [InlineData("<view name=\"gamelist\"><aspectRatio name=\"4:3\"/></view>")]
        [InlineData("<variant name=\"dark\"><variant name=\"dark\"/></variant>")]
        [InlineData("<aspectRatio name=\"4:3\"><aspectRatio name=\"4:3\"/></aspectRatio>")]
        public void Selection_blocks_in_the_wrong_place_unthemes_the_system(string body)
        {
            _theme.Capabilities(Selections).Theme(body);
            ResolvedTheme theme = _theme.Load(new ThemeChoices { Variant = "dark", AspectRatio = "4:3" });
            Assert.False(theme.IsThemed);
            Assert.Contains(theme.Errors, d => d.Code == ThemeDiagnosticCode.MisplacedTag);
        }

        [Fact]
        public void The_all_variant_applies_whichever_variant_is_chosen()
        {
            _theme.Capabilities(Selections).Theme("<variant name=\"all\"><view name=\"gamelist\"><text name=\"t\"><text>all</text></text></view></variant>");
            Assert.NotNull(_theme.Load(new ThemeChoices { Variant = "dark" }).GamelistView.Find("text", "t"));
            Assert.NotNull(_theme.Load(new ThemeChoices { Variant = "lightMode" }).GamelistView.Find("text", "t"));
        }

        [Fact]
        public void Variant_and_aspect_ratio_name_lists_take_commas_and_whitespace()
        {
            _theme.Capabilities(Selections).Theme("""
                <variant name="lightMode,
                    dark	lightModeNoVideo"><view name="gamelist"><text name="v"><text>v</text></text></view></variant>
                <aspectRatio name="16:9,4:3"><view name="gamelist"><text name="a"><text>a</text></text></view></aspectRatio>
                """);
            ResolvedTheme theme = _theme.Load(new ThemeChoices { Variant = "dark", AspectRatio = "4:3" });
            Assert.NotNull(theme.GamelistView.Find("text", "v"));
            Assert.NotNull(theme.GamelistView.Find("text", "a"));
            Assert.Null(_theme.Load(new ThemeChoices { AspectRatio = "16:10" }).GamelistView.Find("text", "a"));
        }

        [Fact]
        public void Undeclared_names_in_selection_blocks_are_warned_once_and_never_selected()
        {
            _theme.Capabilities(Selections).Theme("""
                <variant name="ghost"><view name="gamelist"><text name="g"><text>g</text></text></view></variant>
                <variant name="ghost"/>
                <aspectRatio name="17:9"/>
                <colorScheme name="purple"/>
                """);
            ResolvedTheme theme = _theme.Load(new ThemeChoices { Variant = "ghost" });
            Assert.True(theme.IsThemed);
            Assert.Null(theme.GamelistView.Find("text", "g"));
            Assert.Equal(3, theme.Diagnostics.Count(d => d.Code == ThemeDiagnosticCode.Undeclared));
        }

        [Fact]
        public void Definitions_of_one_type_and_name_merge_the_last_value_winning()
        {
            _theme.Capabilities("").Theme("<view name=\"gamelist\"><text name=\"t\"><color>FF0000</color><pos>0.1 0.2</pos></text></view>", "colors.xml").Theme("""
                <view name="gamelist"><text name="t"><fontSize>0.035</fontSize><color>00FF00</color></text></view>
                <include>./colors.xml</include>
                <view name="gamelist"><text name="t"><color>0000FF</color><color>123456</color></text></view>
                """);
            ResolvedElement t = _theme.Load().GamelistView.Find("text", "t")!;
            Assert.Equal("123456FF", t.Color("color")!.Value.ToString());
            Assert.Equal(new NormalizedPair(0.1f, 0.2f), t.Pair("pos"));
            Assert.Equal(0.035f, t.Float("fontSize"));
        }

        [Fact]
        public void The_same_name_on_another_type_or_view_is_another_element()
        {
            _theme.Capabilities("").Asset("a.png").Theme("""
                <view name="gamelist"><text name="systemName"><pos>0.27 0.32</pos></text><image name="systemName"><path>./a.png</path></image></view>
                <view name="system"><text name="systemName"><pos>0.04 0.73</pos></text></view>
                """);
            ResolvedTheme theme = _theme.Load();
            Assert.Equal(new NormalizedPair(0.27f, 0.32f), theme.GamelistView.Find("text", "systemName")!.Pair("pos"));
            Assert.NotNull(theme.GamelistView.Find("image", "systemName"));
            Assert.Equal(new NormalizedPair(0.04f, 0.73f), theme.SystemView.Find("text", "systemName")!.Pair("pos"));
        }

        [Fact]
        public void Several_views_and_several_element_names_in_one_definition()
        {
            _theme.Capabilities("").Theme("""
                <view name="system,
                    gamelist">
                    <text name="labelRating, labelReleasedate labelDeveloper
                            labelPublisher,    labelGenre"><color>48474D</color></text>
                </view>
                """);
            ResolvedTheme theme = _theme.Load();
            foreach (string view in new[] { "system", "gamelist" })
            {
                Assert.Equal(5, theme.View(view).OfType("text").Count());
                Assert.All(theme.View(view).OfType("text"), t => Assert.Equal("48474DFF", t.Color("color")!.Value.ToString()));
            }
        }

        [Fact]
        public void Navigation_sounds_come_from_the_all_view()
        {
            _theme.Capabilities("").Asset("sounds/select.wav").Theme("""
                <view name="all">
                    <sound name="select"><path>./sounds/select.wav</path></sound>
                    <sound name="back"><path>./sounds/back.wav</path></sound>
                </view>
                <view name="gamelist"><sound name="scroll"><path>./sounds/select.wav</path></sound></view>
                """);
            ResolvedTheme theme = _theme.Load();
            Assert.True(theme.Sounds["select"].Exists);
            Assert.False(theme.Sounds["back"].Exists);
            Assert.False(theme.Sounds.ContainsKey("scroll"));
            Assert.Contains(theme.Diagnostics, d => d.Code == ThemeDiagnosticCode.ViewRestricted);
            Assert.Contains(theme.Diagnostics, d => d.Code == ThemeDiagnosticCode.PathMissing && d.Severity == ThemeSeverity.Warning);
        }

        [Fact]
        public void Elements_are_ordered_by_zIndex_ties_by_first_definition_and_the_special_three_on_top()
        {
            _theme.Capabilities("").Theme("""
                <view name="gamelist">
                    <helpsystem name="help"><fontSize>0.03</fontSize></helpsystem>
                    <text name="text40"><text>t</text></text>
                    <image name="imageA"><pos>0 0</pos></image>
                    <textlist name="list"><pos>0 0</pos></textlist>
                    <image name="imageB"><pos>0 0</pos></image>
                    <clock name="clock"><pos>0 0</pos></clock>
                    <rating name="rating"><zIndex>10</zIndex></rating>
                    <badges name="badges"><pos>0 0</pos></badges>
                    <systemstatus name="status"><pos>0 0</pos></systemstatus>
                    <datetime name="date"><zIndex>30</zIndex></datetime>
                </view>
                """);
            ResolvedView view = _theme.Load().GamelistView;
            Assert.Equal(["rating", "imageA", "imageB", "date", "badges", "text40", "list", "help", "clock", "status"], view.Elements.Select(e => e.Name));
            Assert.Null(view.Find("helpsystem", "help")!.ZIndex);
            Assert.Equal(50f, view.Find("textlist", "list")!.ZIndex);
        }
    }
}
