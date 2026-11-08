// ThemeLoaderTests re-expressed: the grammar on synthetic themes, parsing order, includes, variables, selection blocks and merging (EmuSen_BigPicture.md §12.2).
mod common;

use common::*;
use emusen_esde::loader::values::{full_path, resolve_path};
use emusen_esde::loader::{Code, Pair, Severity, SystemKind, ThemeSystem};

const SELECTIONS: &str = r#"
    <variant name="lightMode"/><variant name="lightModeNoVideo"/><variant name="dark"/>
    <colorScheme name="blue"/><colorScheme name="red"/>
    <fontSize>medium</fontSize><fontSize>small</fontSize>
    <aspectRatio>16:9</aspectRatio><aspectRatio>4:3</aspectRatio><aspectRatio>16:10</aspectRatio>
"#;

#[test]
fn themes_md_variables_example_general_config_is_parsed_before_the_variant_redefines_the_variable() {
    let t = Synth::new();
    t.capabilities(SELECTIONS).theme(
        r#"<variables>
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
        </variant>"#,
    );
    let theme = t.load_choices(&ch().variant("lightMode"));
    assert!(theme.is_themed());
    assert_eq!(color(&theme, "infoText01"), "8B0000FF");
    assert_eq!(color(&theme, "gameName"), "6533FFFF");
}

#[test]
fn variant_configuration_written_above_general_configuration_is_still_applied_after_it() {
    let t = Synth::new();
    t.capabilities(SELECTIONS).theme(
        r#"<variant name="dark"><view name="gamelist"><text name="t"><color>000000</color></text></view></variant>
        <view name="gamelist"><text name="t"><color>FFFFFF</color></text></view>"#,
    );
    assert_eq!(color(&t.load_choices(&ch().variant("dark")), "t"), "000000FF");
    assert_eq!(color(&t.load_choices(&ch().variant("lightMode")), "t"), "FFFFFFFF");
}

#[test]
fn aspect_ratio_configuration_is_applied_after_variants() {
    let t = Synth::new();
    t.capabilities(SELECTIONS).theme(
        r#"<aspectRatio name="4:3"><view name="gamelist"><text name="t"><color>0000AA</color></text></view></aspectRatio>
        <variant name="all"><view name="gamelist"><text name="t"><color>00AA00</color></text></view></variant>
        <view name="gamelist"><text name="t"><color>AA0000</color></text></view>"#,
    );
    assert_eq!(color(&t.load_choices(&ch().aspect("4:3")), "t"), "0000AAFF");
    assert_eq!(color(&t.load_choices(&ch().aspect("16:9")), "t"), "00AA00FF");
}

#[test]
fn the_steps_run_in_the_documented_order_whatever_the_file_order() {
    let t = Synth::new();
    t.capabilities(SELECTIONS).theme_at("<variables><fromInclude>${fromFontSize}</fromInclude></variables>", "inc.xml").theme(
        r#"<aspectRatio name="16:9"><variables><v>ratio</v></variables></aspectRatio>
        <variant name="dark"><variables><v>${v}+variant</v></variables></variant>
        <view name="gamelist"><text name="general"><text>${v}|${fromInclude}</text></text></view>
        <include>./inc.xml</include>
        <fontSize name="medium"><variables><fromFontSize>${fromScheme}+size</fromFontSize></variables></fontSize>
        <colorScheme name="blue"><variables><fromScheme>${v}+scheme</fromScheme></variables></colorScheme>
        <variables><v>base</v></variables>
        <aspectRatio name="16:9"><view name="gamelist"><text name="ratio"><text>${v}</text></text></view></aspectRatio>
        <variant name="dark"><view name="gamelist"><text name="variant"><text>${v}</text></text></view></variant>"#,
    );
    let theme = t.load_choices(&ch().variant("dark").aspect("16:9"));
    assert!(theme.is_themed(), "{}", shown(&theme));
    assert_eq!(text(&theme, "gamelist", "general").as_deref(), Some("base|base+scheme+size"));
    assert_eq!(text(&theme, "gamelist", "variant").as_deref(), Some("base+variant"));
    assert_eq!(text(&theme, "gamelist", "ratio").as_deref(), Some("ratio"));
}

#[test]
fn an_include_runs_all_nine_steps_at_its_own_place() {
    let t = Synth::new();
    t.capabilities(SELECTIONS)
        .theme_at(
            r#"<variables><c>111111</c></variables>
            <view name="gamelist"><text name="t"><color>${c}</color></text></view>
            <variant name="dark"><view name="gamelist"><text name="v"><color>${c}</color></text></view></variant>"#,
            "inc.xml",
        )
        .theme(
            r#"<variables><c>222222</c></variables>
            <include>./inc.xml</include>
            <view name="gamelist"><text name="after"><color>${c}</color></text></view>"#,
        );
    let theme = t.load_choices(&ch().variant("dark"));
    assert_eq!(color(&theme, "t"), "111111FF");
    assert_eq!(color(&theme, "v"), "111111FF");
    assert_eq!(color(&theme, "after"), "111111FF");
}

#[test]
fn colour_scheme_blocks_apply_in_file_order_the_last_matching_winning() {
    let t = Synth::new();
    t.capabilities(SELECTIONS).theme(
        r#"<variables><c>000000</c></variables>
        <colorScheme name="blue, red"><variables><c>0000FF</c><d>0000FF</d></variables></colorScheme>
        <colorScheme name="red"><variables><c>FF0000</c></variables></colorScheme>
        <view name="gamelist"><text name="c"><color>${c}</color></text><text name="d"><color>${d}</color></text></view>"#,
    );
    let blue = t.load_choices(&ch().scheme("blue"));
    let red = t.load_choices(&ch().scheme("red"));
    assert_eq!(color(&blue, "c"), "0000FFFF");
    assert_eq!(color(&red, "c"), "FF0000FF");
    assert_eq!(color(&red, "d"), "0000FFFF");
}

#[test]
fn a_colour_scheme_inside_a_variant_or_aspect_ratio_applies_only_there() {
    let t = Synth::new();
    t.capabilities(SELECTIONS).theme(
        r#"<variables><c>000000</c></variables>
        <variant name="dark"><colorScheme name="red"><variables><c>FF0000</c></variables></colorScheme>
            <view name="gamelist"><text name="v"><color>${c}</color></text></view></variant>
        <aspectRatio name="4:3"><colorScheme name="red"><variables><c>AA0000</c></variables></colorScheme>
            <view name="gamelist"><text name="a"><color>${c}</color></text></view></aspectRatio>"#,
    );
    let theme = t.load_choices(&ch().variant("dark").scheme("red").aspect("4:3"));
    assert_eq!(color(&theme, "v"), "FF0000FF");
    assert_eq!(color(&theme, "a"), "AA0000FF");
    assert_eq!(color(&t.load_choices(&ch().variant("dark").scheme("blue").aspect("4:3")), "v"), "000000FF");
}

#[test]
fn font_size_blocks_follow_the_choice_and_name_lists() {
    let t = Synth::new();
    t.capabilities(SELECTIONS).theme(
        r#"<fontSize name="medium"><variables><s>0.03</s></variables></fontSize>
        <fontSize name="small, x-small"><variables><s>0.02</s></variables></fontSize>
        <view name="gamelist"><text name="t"><fontSize>${s}</fontSize></text></view>"#,
    );
    assert_eq!(t.load().gamelist_view.find("text", "t").unwrap().float("fontSize"), Some(0.03));
    assert_eq!(t.load_choices(&ch().font("small")).gamelist_view.find("text", "t").unwrap().float("fontSize"), Some(0.02));
}

#[test]
fn a_colour_scheme_or_font_size_may_hold_only_variables() {
    let t = Synth::new();
    t.capabilities(SELECTIONS).theme(
        r#"<colorScheme name="blue"><panelColor>74747488</panelColor></colorScheme>
        <view name="gamelist"><text name="t"><color>${panelColor}</color></text></view>"#,
    );
    let theme = t.load();
    assert_eq!(color(&theme, "t"), "74747488");
    assert!(count_at(&theme, Code::MisplacedTag, Severity::Warning) > 0);
}

#[test]
fn language_blocks_apply_for_the_chosen_declared_language_and_are_skipped_when_none_is_declared() {
    let t = Synth::new();
    let body = r#"<language name="en_US"><variables><label>Developer</label></variables></language>
        <language name="sv_SE"><variables><label>Utvecklare</label></variables><include>./sv.xml</include></language>
        <variables><label>none</label></variables>
        <view name="gamelist"><text name="t"><text>${label}</text></text></view>"#;
    t.capabilities("<language>en_US</language><language>sv_SE</language>")
        .theme(body)
        .theme_at("<view name=\"gamelist\"><text name=\"sv\"><text>ja</text></text></view>", "sv.xml");
    assert_eq!(text(&t.load(), "gamelist", "t").as_deref(), Some("Developer"));
    let swedish = t.load_choices(&ch().language("sv_SE"));
    assert_eq!(text(&swedish, "gamelist", "t").as_deref(), Some("Utvecklare"));
    assert!(swedish.gamelist_view.find("text", "sv").is_some());
    t.capabilities("");
    assert_eq!(text(&t.load_choices(&ch().language("sv_SE")), "gamelist", "t").as_deref(), Some("none"));
}

#[test]
fn nested_variables_resolve_through_every_level() {
    let t = Synth::new();
    t.capabilities("").theme(
        r#"<variables>
            <a>12</a>
            <b>${a}34</b>
            <c>${b}56</c>
        </variables>
        <view name="gamelist"><text name="t"><color>${c}</color></text><text name="u"><color>${c}C0</color></text></view>"#,
    );
    let theme = t.load();
    assert!(theme.is_themed(), "{}", shown(&theme));
    assert_eq!(color(&theme, "t"), "123456FF");
    assert_eq!(color(&theme, "u"), "123456C0");
}

#[test]
fn a_variable_is_substituted_as_it_was_when_it_was_defined() {
    let t = Synth::new();
    t.capabilities("").theme(
        r#"<variables><a>111111</a><b>${a}</b><a>222222</a></variables>
        <view name="gamelist"><text name="b"><color>${b}</color></text><text name="a"><color>${a}</color></text></view>"#,
    );
    let theme = t.load();
    assert_eq!(color(&theme, "b"), "111111FF");
    assert_eq!(color(&theme, "a"), "222222FF");
}

#[test]
fn the_twelve_system_variables_have_their_kind_specific_forms_empty_for_other_kinds() {
    let t = Synth::new();
    t.capabilities("").theme(
        r#"<view name="system">
            <text name="n"><text>[${system.name}|${system.fullName}|${system.theme}]</text></text>
            <text name="k"><text>[${system.name.noCollections}|${system.fullName.autoCollections}|${system.theme.customCollections}]</text></text>
        </view>"#,
    );
    let snes = t.load();
    assert_eq!(text(&snes, "system", "n").as_deref(), Some("[snes|Super Nintendo|snes]"));
    assert_eq!(text(&snes, "system", "k").as_deref(), Some("[snes||]"));
    let favourites =
        t.load_with(&ch(), &ThemeSystem::of_kind("favorites", "Favorites", "auto-favorites", SystemKind::AutoCollection), None);
    assert_eq!(text(&favourites, "system", "k").as_deref(), Some("[|Favorites|]"));
    let custom = t.load_with(&ch(), &ThemeSystem::of_kind("mine", "Mine", "custom-collections", SystemKind::CustomCollection), None);
    assert_eq!(text(&custom, "system", "k").as_deref(), Some("[||custom-collections]"));
}

// ES-DE 3.4.1, measured: A${nosuch}B drew "AB" with no log line, and ${nosuch} alone refused the system as a property with no value (§39).
#[test]
fn an_undefined_variable_in_a_property_is_read_as_empty_and_a_property_left_empty_unthemes() {
    let t = Synth::new();
    t.capabilities("").theme("<view name=\"gamelist\"><text name=\"t\"><text>A${nowhere}B</text></text></view>");
    let theme = t.load();
    assert!(theme.is_themed());
    assert_eq!(text(&theme, "gamelist", "t").as_deref(), Some("AB"));
    assert!(count_at(&theme, Code::UndefinedVariable, Severity::Warning) > 0);
    t.theme("<view name=\"gamelist\"><text name=\"t\"><color>${nowhere}</color></text></view>");
    let empty = t.load();
    assert!(!empty.is_themed());
    assert!(has_error(&empty, Code::NoValue));
}

#[test]
fn a_variable_redefined_in_a_variant_or_aspect_ratio_changes_the_global_one() {
    let t = Synth::new();
    t.capabilities(SELECTIONS).theme(
        r#"<variables><c>000000</c></variables>
        <variant name="dark"><variables><c>0000FF</c></variables></variant>
        <aspectRatio name="4:3"><view name="gamelist"><text name="t"><color>${c}</color></text></view></aspectRatio>"#,
    );
    assert_eq!(color(&t.load_choices(&ch().variant("dark").aspect("4:3")), "t"), "0000FFFF");
}

#[test]
fn include_paths_resolve_from_the_including_file_not_the_theme_root() {
    let t = Synth::new();
    t.capabilities("")
        .asset("core/images/frame.png")
        .asset("snes/logo.png")
        .theme_at(
            "<include>./../core/fonts.xml</include><view name=\"gamelist\"><image name=\"logo\"><path>./logo.png</path></image></view>",
            "snes/theme.xml",
        )
        .theme_at("<include>./nested/deeper.xml</include>", "core/fonts.xml")
        .theme_at(
            "<view name=\"gamelist\"><image name=\"frame\"><path>./../images/frame.png</path></image></view>",
            "core/nested/deeper.xml",
        )
        .theme("");
    let theme = t.load();
    assert!(theme.is_themed(), "{}", shown(&theme));
    let frame = theme.gamelist_view.find("image", "frame").unwrap().path("path").unwrap();
    assert_eq!(frame.absolute, t.path_of("core/images/frame.png"));
    assert_eq!(theme.gamelist_view.find("image", "logo").unwrap().path("path").unwrap().absolute, t.path_of("snes/logo.png"));
    assert!(frame.exists);
}

#[test]
fn a_system_folder_theme_xml_is_used_before_the_root_one() {
    let t = Synth::new();
    t.capabilities("")
        .theme("<view name=\"system\"><text name=\"which\"><text>root</text></text></view>")
        .theme_at("<view name=\"system\"><text name=\"which\"><text>snes</text></text></view>", "snes/theme.xml");
    assert_eq!(text(&t.load(), "system", "which").as_deref(), Some("snes"));
    assert_eq!(text(&t.load_with(&ch(), &ThemeSystem::new("nes", "Nintendo", "nes"), None), "system", "which").as_deref(), Some("root"));
}

#[test]
fn backslashes_and_tilde_are_resolved_as_documented() {
    let t = Synth::new();
    let file = t.path_of("theme.xml");
    assert_eq!(resolve_path(".\\core\\a.png", &file), t.path_of("core/a.png"));
    let home = std::env::var("HOME").unwrap();
    assert_eq!(resolve_path("~/themes/x.png", &file), full_path(&format!("{home}/themes/x.png")));
    assert_eq!(resolve_path("/abs/x.png", &file), "/abs/x.png");
}

#[test]
fn a_missing_include_written_out_is_an_error() {
    let t = Synth::new();
    t.capabilities("").theme("<include>./colors_dark.xml</include>");
    assert_unthemed(&t.load(), Code::IncludeMissing);
}

#[test]
fn a_missing_include_built_from_a_variable_is_skipped_silently() {
    let t = Synth::new();
    t.capabilities("")
        .theme("<include>./${system.theme}/colors.xml</include><view name=\"system\"><text name=\"t\"><text>x</text></text></view>");
    let theme = t.load();
    assert!(theme.is_themed());
    assert_eq!(theme.diagnostics.len(), 1, "{}", shown(&theme));
    assert_eq!((theme.diagnostics[0].severity, theme.diagnostics[0].code), (Severity::Debug, Code::IncludeSkipped));
}

#[test]
fn an_include_naming_an_undefined_variable_is_skipped_as_art_book_next_colors_xml_needs() {
    let t = Synth::new();
    t.capabilities("<colorScheme name=\"dark\"/><colorScheme name=\"custom\"/>")
        .theme_at("<view name=\"gamelist\"><text name=\"custom\"><text>yes</text></text></view>", "custom/colors.xml")
        .theme(
            r#"<colorScheme name="custom"><variables><customizationPath>./custom/colors.xml</customizationPath></variables></colorScheme>
            <include>${customizationPath}</include>
            <view name="gamelist"><text name="t"><text>x</text></text></view>"#,
        );
    let dark = t.load_choices(&ch().scheme("dark"));
    assert!(dark.is_themed());
    assert_eq!(count_at(&dark, Code::IncludeSkipped, Severity::Debug), 1);
    assert!(dark.gamelist_view.find("text", "custom").is_none());
    let custom = t.load_choices(&ch().scheme("custom"));
    assert!(custom.is_themed());
    assert!(custom.gamelist_view.find("text", "custom").is_some());
}

#[test]
fn a_circular_include_is_refused_rather_than_hanging() {
    let t = Synth::new();
    t.capabilities("")
        .theme_at("<include>./b.xml</include>", "a.xml")
        .theme_at("<include>./a.xml</include>", "b.xml")
        .theme("<include>./a.xml</include>");
    assert_unthemed(&t.load(), Code::IncludeLoop);
}

#[test]
fn the_same_file_may_be_included_twice_without_being_a_loop() {
    let t = Synth::new();
    t.capabilities(SELECTIONS)
        .theme_at("<view name=\"gamelist\"><text name=\"t\"><text>x</text></text></view>", "shared.xml")
        .theme("<include>./shared.xml</include><variant name=\"dark\"><include>./shared.xml</include></variant>");
    assert!(t.load_choices(&ch().variant("dark")).is_themed());
}

#[test]
fn includes_are_allowed_in_theme_variant_and_aspect_ratio_but_not_view() {
    let t = Synth::new();
    t.capabilities(SELECTIONS)
        .theme_at("<view name=\"gamelist\"><text name=\"v\"><text>v</text></text></view>", "v.xml")
        .theme_at("<view name=\"gamelist\"><text name=\"a\"><text>a</text></text></view>", "a.xml")
        .theme(
            "<variant name=\"dark\"><include>./v.xml</include></variant><aspectRatio name=\"4:3\"><include>./a.xml</include></aspectRatio>",
        );
    let theme = t.load_choices(&ch().variant("dark").aspect("4:3"));
    assert!(theme.is_themed());
    assert!(theme.gamelist_view.find("text", "v").is_some());
    assert!(theme.gamelist_view.find("text", "a").is_some());
    t.theme("<view name=\"gamelist\"><include>./v.xml</include></view>");
    assert_unthemed(&t.load(), Code::MisplacedTag);
}

#[test]
fn selection_blocks_in_the_wrong_place_untheme_the_system() {
    for body in [
        "<view name=\"gamelist\"><variant name=\"dark\"/></view>",
        "<view name=\"gamelist\"><aspectRatio name=\"4:3\"/></view>",
        "<variant name=\"dark\"><variant name=\"dark\"/></variant>",
        "<aspectRatio name=\"4:3\"><aspectRatio name=\"4:3\"/></aspectRatio>",
    ] {
        let t = Synth::new();
        t.capabilities(SELECTIONS).theme(body);
        assert_unthemed(&t.load_choices(&ch().variant("dark").aspect("4:3")), Code::MisplacedTag);
    }
}

#[test]
fn the_all_variant_applies_whichever_variant_is_chosen() {
    let t = Synth::new();
    t.capabilities(SELECTIONS)
        .theme("<variant name=\"all\"><view name=\"gamelist\"><text name=\"t\"><text>all</text></text></view></variant>");
    assert!(t.load_choices(&ch().variant("dark")).gamelist_view.find("text", "t").is_some());
    assert!(t.load_choices(&ch().variant("lightMode")).gamelist_view.find("text", "t").is_some());
}

#[test]
fn variant_and_aspect_ratio_name_lists_take_commas_and_whitespace() {
    let t = Synth::new();
    t.capabilities(SELECTIONS).theme(
        "<variant name=\"lightMode,\n            dark\tlightModeNoVideo\"><view name=\"gamelist\"><text name=\"v\"><text>v</text></text></view></variant>
        <aspectRatio name=\"16:9,4:3\"><view name=\"gamelist\"><text name=\"a\"><text>a</text></text></view></aspectRatio>",
    );
    let theme = t.load_choices(&ch().variant("dark").aspect("4:3"));
    assert!(theme.gamelist_view.find("text", "v").is_some());
    assert!(theme.gamelist_view.find("text", "a").is_some());
    assert!(t.load_choices(&ch().aspect("16:10")).gamelist_view.find("text", "a").is_none());
}

#[test]
fn undeclared_names_in_selection_blocks_are_warned_once_and_never_selected() {
    let t = Synth::new();
    t.capabilities(SELECTIONS).theme(
        r#"<variant name="ghost"><view name="gamelist"><text name="g"><text>g</text></text></view></variant>
        <variant name="ghost"/>
        <aspectRatio name="17:9"/>
        <colorScheme name="purple"/>"#,
    );
    let theme = t.load_choices(&ch().variant("ghost"));
    assert!(theme.is_themed());
    assert!(theme.gamelist_view.find("text", "g").is_none());
    assert_eq!(count(&theme, Code::Undeclared), 3);
}

#[test]
fn definitions_of_one_type_and_name_merge_the_last_value_winning() {
    let t = Synth::new();
    t.capabilities("")
        .theme_at("<view name=\"gamelist\"><text name=\"t\"><color>FF0000</color><pos>0.1 0.2</pos></text></view>", "colors.xml")
        .theme(
            r#"<view name="gamelist"><text name="t"><fontSize>0.035</fontSize><color>00FF00</color></text></view>
        <include>./colors.xml</include>
        <view name="gamelist"><text name="t"><color>0000FF</color><color>123456</color></text></view>"#,
        );
    let theme = t.load();
    let e = theme.gamelist_view.find("text", "t").unwrap();
    assert_eq!(e.color("color").unwrap().to_string(), "123456FF");
    assert_eq!(e.pair("pos"), Some(Pair::new(0.1, 0.2)));
    assert_eq!(e.float("fontSize"), Some(0.035));
}

#[test]
fn the_same_name_on_another_type_or_view_is_another_element() {
    let t = Synth::new();
    t.capabilities("").asset("a.png").theme(
        r#"<view name="gamelist"><text name="systemName"><pos>0.27 0.32</pos></text><image name="systemName"><path>./a.png</path></image></view>
        <view name="system"><text name="systemName"><pos>0.04 0.73</pos></text></view>"#,
    );
    let theme = t.load();
    assert_eq!(theme.gamelist_view.find("text", "systemName").unwrap().pair("pos"), Some(Pair::new(0.27, 0.32)));
    assert!(theme.gamelist_view.find("image", "systemName").is_some());
    assert_eq!(theme.system_view.find("text", "systemName").unwrap().pair("pos"), Some(Pair::new(0.04, 0.73)));
}

#[test]
fn several_views_and_several_element_names_in_one_definition() {
    let t = Synth::new();
    t.capabilities("").theme(
        r#"<view name="system,
            gamelist">
            <text name="labelRating, labelReleasedate labelDeveloper
                    labelPublisher,    labelGenre"><color>48474D</color></text>
        </view>"#,
    );
    let theme = t.load();
    for view in ["system", "gamelist"] {
        let texts: Vec<_> = theme.view(view).of_kind("text").collect();
        assert_eq!(texts.len(), 5);
        assert!(texts.iter().all(|e| e.color("color").unwrap().to_string() == "48474DFF"));
    }
}

#[test]
fn navigation_sounds_come_from_the_all_view() {
    let t = Synth::new();
    t.capabilities("").asset("sounds/select.wav").theme(
        r#"<view name="all">
            <sound name="select"><path>./sounds/select.wav</path></sound>
            <sound name="back"><path>./sounds/back.wav</path></sound>
        </view>
        <view name="gamelist"><sound name="scroll"><path>./sounds/select.wav</path></sound></view>"#,
    );
    let theme = t.load();
    let sounds = theme.sounds();
    assert!(sounds["select"].exists);
    assert!(!sounds["back"].exists);
    assert!(!sounds.contains_key("scroll"));
    assert!(count(&theme, Code::ViewRestricted) > 0);
    assert!(count_at(&theme, Code::PathMissing, Severity::Warning) > 0);
}

#[test]
fn elements_are_ordered_by_z_index_ties_by_first_definition_and_the_special_three_on_top() {
    let t = Synth::new();
    t.capabilities("").theme(
        r#"<view name="gamelist">
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
        </view>"#,
    );
    let theme = t.load();
    let view = &theme.gamelist_view;
    let names: Vec<&str> = view.elements.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["rating", "imageA", "imageB", "date", "badges", "text40", "list", "help", "clock", "status"]);
    assert_eq!(view.find("helpsystem", "help").unwrap().z_index, None);
    assert_eq!(view.find("textlist", "list").unwrap().z_index, Some(50.0));
}
