using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // capabilities.xml, the player's choices and the variant triggers, on synthetic themes - see EmuSen_BigPicture.md §12.2 and §12.4.
    public class ThemeCapabilitiesTests : IDisposable
    {
        private readonly SyntheticTheme _theme = new();

        public void Dispose() => _theme.Dispose();

        private const string Variants = """
            <variant name="withVideos">
                <label>Textlist with videos</label>
                <label language="sv_SE">Textlista med video</label>
                <selectable>true</selectable>
                <override><trigger>noVideos</trigger><useVariant>withoutVideos</useVariant></override>
                <override><trigger>noMedia</trigger><mediaType>miximage, screenshot cover</mediaType><useVariant>noGameMedia</useVariant></override>
            </variant>
            <variant name="withoutVideos">
                <selectable>false</selectable>
                <override><trigger>noMedia</trigger><useVariant>noGameMedia</useVariant></override>
            </variant>
            <variant name="noGameMedia"><selectable>false</selectable></variant>
            """;

        [Fact]
        public void Capabilities_are_read_with_labels_overrides_and_defaults()
        {
            _theme.Capabilities($"<themeName>Synthetic</themeName>{Variants}<colorScheme name=\"dark\"><label>Dark</label></colorScheme>").Theme("");
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(_theme.Root);

            Assert.Equal("Synthetic", caps.ThemeName);
            Assert.Equal(["withVideos", "withoutVideos", "noGameMedia"], caps.Variants.Select(v => v.Name));
            ThemeVariant with = caps.Variants[0];
            Assert.Equal("Textlist with videos", with.Labels["en_US"]);
            Assert.Equal("Textlista med video", with.Labels["sv_SE"]);
            Assert.Equal(["noVideos", "noMedia"], with.Overrides.Select(o => o.Trigger));
            Assert.Equal(["miximage", "screenshot", "cover"], with.Overrides[1].MediaTypes);
            Assert.Equal(["miximage"], caps.Variants[1].Overrides[0].MediaTypes);
            Assert.False(caps.Variants[1].Selectable);
            Assert.Equal("Dark", caps.ColorSchemes.Single().Labels["en_US"]);
            Assert.Empty(caps.Diagnostics);
        }

        [Fact]
        public void The_theme_name_falls_back_to_the_folder_name_in_capitals()
        {
            _theme.Capabilities("").Theme("");
            Assert.Equal(System.IO.Path.GetFileName(_theme.Root).ToUpperInvariant(), ThemeCapabilitiesReader.Read(_theme.Root).ThemeName);
        }

        [Fact]
        public void An_empty_or_comment_only_capabilities_file_declares_nothing_and_loads()
        {
            _theme.File("capabilities.xml", "<!-- this theme declares nothing -->").Theme("<view name=\"system\"><image name=\"a\"><pos>0 0</pos></image></view>");
            ResolvedTheme theme = _theme.Load();
            Assert.True(theme.IsThemed);
            Assert.Empty(theme.Capabilities.Variants);
            Assert.NotNull(theme.SystemView.Find("image", "a"));

            _theme.File("capabilities.xml", "");
            Assert.True(_theme.Load().IsThemed);
        }

        [Fact]
        public void Without_capabilities_the_theme_is_not_loaded()
        {
            _theme.Theme("<view name=\"system\"><image name=\"a\"><pos>0 0</pos></image></view>");
            ResolvedTheme theme = _theme.Load();
            Assert.False(theme.IsThemed);
            Assert.Contains(theme.Errors, d => d.Code == ThemeDiagnosticCode.CapabilitiesMissing);
            Assert.Empty(theme.SystemView.Elements);
        }

        [Fact]
        public void Font_sizes_and_aspect_ratios_are_listed_in_the_documented_order_and_bad_entries_are_dropped()
        {
            _theme.Capabilities("""
                <fontSize>x-small</fontSize><fontSize>small</fontSize><fontSize>medium</fontSize><fontSize>huge</fontSize>
                <aspectRatio>4:3_vertical</aspectRatio><aspectRatio>1:1</aspectRatio><aspectRatio>16:10</aspectRatio>
                <aspectRatio>4:3</aspectRatio><aspectRatio>17:9</aspectRatio><aspectRatio>16:10</aspectRatio><aspectRatio>1:1_vertical</aspectRatio>
                """).Theme("");
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(_theme.Root);

            Assert.Equal(["medium", "small", "x-small"], caps.FontSizes);
            Assert.Equal(["16:10", "4:3", "4:3_vertical", "1:1"], caps.AspectRatios);
            Assert.Equal(3, caps.Diagnostics.Count(d => d.Code == ThemeDiagnosticCode.InvalidValue));
            Assert.Single(caps.Diagnostics, d => d.Code == ThemeDiagnosticCode.Duplicate);
            Assert.All(caps.Diagnostics, d => Assert.Equal(ThemeSeverity.Warning, d.Severity));
        }

        [Fact]
        public void Languages_without_en_US_are_not_loaded()
        {
            _theme.Capabilities("<language>sv_SE</language><language>de_DE</language>").Theme("");
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(_theme.Root);
            Assert.Empty(caps.Languages);
            Assert.Contains(caps.Diagnostics, d => d.Code == ThemeDiagnosticCode.LanguageWithoutEnglish);

            _theme.Capabilities("<language>en_US</language><language>sv_SE</language><language>pt_PR</language>");
            caps = ThemeCapabilitiesReader.Read(_theme.Root);
            Assert.Equal(["en_US", "sv_SE"], caps.Languages);
        }

        [Fact]
        public void Duplicate_and_reserved_names_are_warned_and_not_loaded()
        {
            _theme.Capabilities("""
                <variant name="a"/><variant name="a"/><variant name="all"/>
                <colorScheme name="x"/><colorScheme name="x"/>
                <transitions name="builtin-fade"><systemToSystem>fade</systemToSystem></transitions>
                """).Theme("");
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(_theme.Root);
            Assert.Equal(["a"], caps.Variants.Select(v => v.Name));
            Assert.Equal(["x"], caps.ColorSchemes.Select(v => v.Name));
            Assert.Empty(caps.Transitions);
            Assert.Equal(2, caps.Diagnostics.Count(d => d.Code == ThemeDiagnosticCode.Duplicate));
            Assert.Equal(2, caps.Diagnostics.Count(d => d.Code == ThemeDiagnosticCode.Reserved));
        }

        [Fact]
        public void Transition_kinds_left_out_are_instant_except_startup_which_follows_its_view()
        {
            _theme.Capabilities("""
                <transitions name="p"><label>P</label><systemToSystem>fade</systemToSystem><gamelistToGamelist>slide</gamelistToGamelist></transitions>
                <transitions name="empty"><label>E</label></transitions>
                <suppressTransitionProfiles><entry>builtin-slide</entry><entry>builtin-bogus</entry></suppressTransitionProfiles>
                """).Theme("");
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(_theme.Root);
            TransitionProfile p = caps.Transitions.Single();
            Assert.Equal(TransitionAnimation.Fade, p.SystemToSystem);
            Assert.Equal(TransitionAnimation.Instant, p.SystemToGamelist);
            Assert.Equal(TransitionAnimation.Slide, p.GamelistToGamelist);
            Assert.Equal(TransitionAnimation.Instant, p.GamelistToSystem);
            Assert.Equal(TransitionAnimation.Fade, p.StartupToSystem);
            Assert.Equal(TransitionAnimation.Slide, p.StartupToGamelist);
            Assert.True(p.Selectable);
            Assert.Equal(["builtin-slide"], caps.SuppressedTransitions);
        }

        [Fact]
        public void The_default_variant_is_the_first_selectable_and_a_chosen_one_must_be_declared()
        {
            _theme.Capabilities("<variant name=\"hidden\"><selectable>false</selectable></variant><variant name=\"first\"/><variant name=\"second\"/>").Theme("");
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(_theme.Root);
            Assert.Equal("first", ThemeSelection.Resolve(caps, new ThemeChoices()).Variant);
            Assert.Equal("second", ThemeSelection.Resolve(caps, new ThemeChoices { Variant = "second" }).Variant);
            Assert.Equal("hidden", ThemeSelection.Resolve(caps, new ThemeChoices { Variant = "hidden" }).Variant);
            Assert.Equal("first", ThemeSelection.Resolve(caps, new ThemeChoices { Variant = "undeclared" }).Variant);
        }

        [Fact]
        public void Scheme_font_size_and_language_fall_back_as_documented_or_chosen()
        {
            _theme.Capabilities("""
                <colorScheme name="dark"/><colorScheme name="light"/>
                <fontSize>small</fontSize><fontSize>medium</fontSize><fontSize>large</fontSize>
                <language>en_US</language><language>sv_SE</language>
                """).Theme("");
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(_theme.Root);
            ThemeSelection auto = ThemeSelection.Resolve(caps, new ThemeChoices { ColorScheme = "sepia", FontSize = "x-large", Language = "fr_FR" });
            Assert.Equal("dark", auto.ColorScheme);
            Assert.Equal("medium", auto.FontSize);
            Assert.Equal("en_US", auto.Language);

            ThemeSelection chosen = ThemeSelection.Resolve(caps, new ThemeChoices { ColorScheme = "light", FontSize = "large", Language = "sv_SE" });
            Assert.Equal(("light", "large", "sv_SE"), (chosen.ColorScheme, chosen.FontSize, chosen.Language));
        }

        [Theory]
        [InlineData(1280, 800, "16:10")]
        [InlineData(1920, 1200, "16:10")]
        [InlineData(1920, 1080, "16:9")]
        [InlineData(1024, 768, "4:3")]
        [InlineData(1280, 1024, "4:3")]
        [InlineData(800, 1280, "16:10_vertical")]
        [InlineData(2560, 1080, "16:9")]
        public void Automatic_aspect_ratio_is_the_declared_one_nearest_the_screen(int width, int height, string expected)
        {
            _theme.Capabilities("<aspectRatio>16:9</aspectRatio><aspectRatio>16:10</aspectRatio><aspectRatio>4:3</aspectRatio><aspectRatio>16:10_vertical</aspectRatio>").Theme("");
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(_theme.Root);
            Assert.Equal(expected, ThemeSelection.Resolve(caps, new ThemeChoices { ScreenWidth = width, ScreenHeight = height }).AspectRatio);
            Assert.Equal("4:3", ThemeSelection.Resolve(caps, new ThemeChoices { AspectRatio = "4:3", ScreenWidth = width, ScreenHeight = height }).AspectRatio);
        }

        [Fact]
        public void Without_declared_aspect_ratios_none_is_selected()
        {
            _theme.Capabilities("").Theme("");
            Assert.Null(ThemeSelection.Resolve(ThemeCapabilitiesReader.Read(_theme.Root), new ThemeChoices()).AspectRatio);
        }

        public static TheoryData<string[], string> Triggers => new()
        {
            { ["miximage", "screenshot", "cover", "video"], "withVideos" },
            { ["screenshot"], "withoutVideos" },
            { ["video"], "noGameMedia" },
            { [], "noGameMedia" },
            { ["marquee"], "noGameMedia" },
        };

        [Theory]
        [MemberData(nameof(Triggers))]
        public void NoMedia_is_tried_before_noVideos_and_the_triggers_are_one_step_deep(string[] present, string expected)
        {
            _theme.Capabilities(Variants).Theme("");
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(_theme.Root);
            string? result = ThemeSelection.ApplyTriggers(caps, "withVideos", new MediaPresence(present.ToHashSet()), true, new ThemeDiagnostics());
            Assert.Equal(expected, result);
        }

        [Fact]
        public void A_variant_reached_by_a_trigger_does_not_trigger_again()
        {
            _theme.Capabilities(Variants).Theme("");
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(_theme.Root);
            string? result = ThemeSelection.ApplyTriggers(caps, "withVideos", new MediaPresence(new HashSet<string> { "screenshot" }), true, new ThemeDiagnostics());
            Assert.Equal("withoutVideos", result);
            Assert.NotEqual("noGameMedia", result);
        }

        [Fact]
        public void Triggers_are_off_when_disabled_or_when_media_is_unknown()
        {
            _theme.Capabilities(Variants).Theme("");
            ThemeCapabilities caps = ThemeCapabilitiesReader.Read(_theme.Root);
            Assert.Equal("withVideos", ThemeSelection.ApplyTriggers(caps, "withVideos", MediaPresence.None, false, new ThemeDiagnostics()));
            Assert.Equal("withVideos", ThemeSelection.ApplyTriggers(caps, "withVideos", null, true, new ThemeDiagnostics()));
        }

        [Fact]
        public void Triggers_change_the_gamelist_view_only()
        {
            _theme.Capabilities(Variants).Theme("""
                <variant name="withVideos"><view name="system, gamelist"><text name="which"><text>with</text></text></view></variant>
                <variant name="noGameMedia"><view name="system, gamelist"><text name="which"><text>none</text></text></view></variant>
                """);
            ResolvedTheme theme = _theme.Load(media: MediaPresence.None);
            Assert.Equal("withVideos", theme.Selection.Variant);
            Assert.Equal("noGameMedia", theme.GamelistVariant);
            Assert.Equal("with", theme.SystemView.Find("text", "which")!.String("text"));
            Assert.Equal("none", theme.GamelistView.Find("text", "which")!.String("text"));
        }

        [Fact]
        public void Transitions_automatic_takes_the_variant_profile_then_the_first_declared_and_a_choice_overrides_both()
        {
            _theme.Capabilities("""
                <variant name="a"/><variant name="b"/>
                <transitions name="one"><systemToSystem>slide</systemToSystem></transitions>
                <transitions name="two"><systemToSystem>fade</systemToSystem></transitions>
                <suppressTransitionProfiles><entry>builtin-fade</entry></suppressTransitionProfiles>
                """).Theme("<variant name=\"b\"><transitions>two</transitions></variant>");
            Assert.Equal("one", _theme.Load(new ThemeChoices { Variant = "a" }).Transitions.Name);
            Assert.Equal("two", _theme.Load(new ThemeChoices { Variant = "b" }).Transitions.Name);
            Assert.Equal("one", _theme.Load(new ThemeChoices { Variant = "b", Transitions = "one" }).Transitions.Name);
            Assert.Equal("builtin-slide", _theme.Load(new ThemeChoices { Transitions = "builtin-slide" }).Transitions.Name);
            Assert.Equal("one", _theme.Load(new ThemeChoices { Variant = "a", Transitions = "builtin-fade" }).Transitions.Name);
        }

        [Fact]
        public void With_no_profiles_at_all_transitions_are_instant()
        {
            _theme.Capabilities("<suppressTransitionProfiles><entry>builtin-instant</entry><entry>builtin-slide</entry><entry>builtin-fade</entry></suppressTransitionProfiles>").Theme("");
            TransitionProfile t = _theme.Load().Transitions;
            Assert.Equal(TransitionAnimation.Instant, t.SystemToGamelist);
            Assert.Equal(TransitionAnimation.Instant, t.StartupToSystem);
        }
    }
}
