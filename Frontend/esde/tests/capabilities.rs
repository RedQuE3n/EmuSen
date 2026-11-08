// ThemeCapabilitiesTests re-expressed: capabilities.xml, the player's choices and the variant triggers, on synthetic themes (EmuSen_BigPicture.md §12.2, §12.4).
mod common;

use common::*;
use emusen_esde::loader::capabilities::{self, Animation};
use emusen_esde::loader::choices::{apply_triggers, Selection};
use emusen_esde::loader::{Code, Diagnostics, MediaPresence, Severity};

const VARIANTS: &str = r#"
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
"#;

#[test]
fn capabilities_are_read_with_labels_overrides_and_defaults() {
    let t = Synth::new();
    t.capabilities(&format!("<themeName>Synthetic</themeName>{VARIANTS}<colorScheme name=\"dark\"><label>Dark</label></colorScheme>"))
        .theme("");
    let caps = capabilities::read(&t.dir());
    assert_eq!(caps.theme_name, "Synthetic");
    assert_eq!(caps.variants.iter().map(|v| v.name.as_str()).collect::<Vec<_>>(), ["withVideos", "withoutVideos", "noGameMedia"]);
    let with = &caps.variants[0];
    assert_eq!(with.labels["en_US"], "Textlist with videos");
    assert_eq!(with.labels["sv_SE"], "Textlista med video");
    assert_eq!(with.overrides.iter().map(|o| o.trigger.as_str()).collect::<Vec<_>>(), ["noVideos", "noMedia"]);
    assert_eq!(with.overrides[1].media_types, ["miximage", "screenshot", "cover"]);
    assert_eq!(caps.variants[1].overrides[0].media_types, ["miximage"]);
    assert!(!caps.variants[1].selectable);
    assert_eq!(caps.color_schemes[0].labels["en_US"], "Dark");
    assert!(caps.diagnostics.is_empty(), "{:?}", caps.diagnostics);
}

#[test]
fn the_theme_name_falls_back_to_the_folder_name_in_capitals() {
    let t = Synth::new();
    t.capabilities("").theme("");
    assert_eq!(capabilities::read(&t.dir()).theme_name, t.root.file_name().unwrap().to_string_lossy().to_uppercase());
}

#[test]
fn an_empty_or_comment_only_capabilities_file_declares_nothing_and_loads() {
    let t = Synth::new();
    t.file("capabilities.xml", "<!-- this theme declares nothing -->")
        .theme("<view name=\"system\"><image name=\"a\"><pos>0 0</pos></image></view>");
    let theme = t.load();
    assert!(theme.is_themed());
    assert!(theme.capabilities.variants.is_empty());
    assert!(theme.system_view.find("image", "a").is_some());
    t.file("capabilities.xml", "");
    assert!(t.load().is_themed());
}

#[test]
fn without_capabilities_the_theme_is_not_loaded() {
    let t = Synth::new();
    t.theme("<view name=\"system\"><image name=\"a\"><pos>0 0</pos></image></view>");
    let theme = t.load();
    assert!(!theme.is_themed());
    assert!(has_error(&theme, Code::CapabilitiesMissing));
    assert!(theme.system_view.elements.is_empty());
}

#[test]
fn font_sizes_and_aspect_ratios_are_listed_in_the_documented_order_and_bad_entries_are_dropped() {
    let t = Synth::new();
    t.capabilities(
        "<fontSize>x-small</fontSize><fontSize>small</fontSize><fontSize>medium</fontSize><fontSize>huge</fontSize>
        <aspectRatio>4:3_vertical</aspectRatio><aspectRatio>1:1</aspectRatio><aspectRatio>16:10</aspectRatio>
        <aspectRatio>4:3</aspectRatio><aspectRatio>17:9</aspectRatio><aspectRatio>16:10</aspectRatio><aspectRatio>1:1_vertical</aspectRatio>",
    )
    .theme("");
    let caps = capabilities::read(&t.dir());
    assert_eq!(caps.font_sizes, ["medium", "small", "x-small"]);
    assert_eq!(caps.aspect_ratios, ["16:10", "4:3", "4:3_vertical", "1:1"]);
    assert_eq!(caps.diagnostics.iter().filter(|d| d.code == Code::InvalidValue).count(), 3);
    assert_eq!(caps.diagnostics.iter().filter(|d| d.code == Code::Duplicate).count(), 1);
    assert!(caps.diagnostics.iter().all(|d| d.severity == Severity::Warning));
}

#[test]
fn languages_without_en_us_are_not_loaded() {
    let t = Synth::new();
    t.capabilities("<language>sv_SE</language><language>de_DE</language>").theme("");
    let caps = capabilities::read(&t.dir());
    assert!(caps.languages.is_empty());
    assert!(caps.diagnostics.iter().any(|d| d.code == Code::LanguageWithoutEnglish));
    t.capabilities("<language>en_US</language><language>sv_SE</language><language>pt_PR</language>");
    assert_eq!(capabilities::read(&t.dir()).languages, ["en_US", "sv_SE"]);
}

#[test]
fn duplicate_and_reserved_names_are_warned_and_not_loaded() {
    let t = Synth::new();
    t.capabilities(
        r#"<variant name="a"/><variant name="a"/><variant name="all"/>
        <colorScheme name="x"/><colorScheme name="x"/>
        <transitions name="builtin-fade"><systemToSystem>fade</systemToSystem></transitions>"#,
    )
    .theme("");
    let caps = capabilities::read(&t.dir());
    assert_eq!(caps.variants.iter().map(|v| v.name.as_str()).collect::<Vec<_>>(), ["a"]);
    assert_eq!(caps.color_schemes.iter().map(|v| v.name.as_str()).collect::<Vec<_>>(), ["x"]);
    assert!(caps.transitions.is_empty());
    assert_eq!(caps.diagnostics.iter().filter(|d| d.code == Code::Duplicate).count(), 2);
    assert_eq!(caps.diagnostics.iter().filter(|d| d.code == Code::Reserved).count(), 2);
}

#[test]
fn transition_kinds_left_out_are_instant_except_startup_which_follows_its_view() {
    let t = Synth::new();
    t.capabilities(
        r#"<transitions name="p"><label>P</label><systemToSystem>fade</systemToSystem><gamelistToGamelist>slide</gamelistToGamelist></transitions>
        <transitions name="empty"><label>E</label></transitions>
        <suppressTransitionProfiles><entry>builtin-slide</entry><entry>builtin-bogus</entry></suppressTransitionProfiles>"#,
    )
    .theme("");
    let caps = capabilities::read(&t.dir());
    assert_eq!(caps.transitions.len(), 1);
    let p = &caps.transitions[0];
    assert_eq!(p.system_to_system, Animation::Fade);
    assert_eq!(p.system_to_gamelist, Animation::Instant);
    assert_eq!(p.gamelist_to_gamelist, Animation::Slide);
    assert_eq!(p.gamelist_to_system, Animation::Instant);
    assert_eq!(p.startup_to_system, Animation::Fade);
    assert_eq!(p.startup_to_gamelist, Animation::Slide);
    assert!(p.selectable);
    assert_eq!(caps.suppressed_transitions, ["builtin-slide"]);
}

#[test]
fn the_default_variant_is_the_first_selectable_and_a_chosen_one_must_be_declared() {
    let t = Synth::new();
    t.capabilities(
        "<variant name=\"hidden\"><selectable>false</selectable></variant><variant name=\"first\"><selectable>true</selectable></variant>\
         <variant name=\"second\"><selectable>true</selectable></variant>",
    )
    .theme("");
    let caps = capabilities::read(&t.dir());
    let pick = |c| Selection::resolve(&caps, &c).variant.unwrap();
    assert_eq!(pick(ch()), "first");
    assert_eq!(pick(ch().variant("second")), "second");
    assert_eq!(pick(ch().variant("hidden")), "hidden");
    assert_eq!(pick(ch().variant("undeclared")), "first");
}

#[test]
fn scheme_font_size_and_language_fall_back_as_documented_or_chosen() {
    let t = Synth::new();
    t.capabilities(
        r#"<colorScheme name="dark"/><colorScheme name="light"/>
        <fontSize>small</fontSize><fontSize>medium</fontSize><fontSize>large</fontSize>
        <language>en_US</language><language>sv_SE</language>"#,
    )
    .theme("");
    let caps = capabilities::read(&t.dir());
    let auto = Selection::resolve(&caps, &ch().scheme("sepia").font("x-large").language("fr_FR"));
    assert_eq!(auto.color_scheme.as_deref(), Some("dark"));
    assert_eq!(auto.font_size.as_deref(), Some("medium"));
    assert_eq!(auto.language.as_deref(), Some("en_US"));
    let chosen = Selection::resolve(&caps, &ch().scheme("light").font("large").language("sv_SE"));
    assert_eq!(
        (chosen.color_scheme.as_deref(), chosen.font_size.as_deref(), chosen.language.as_deref()),
        (Some("light"), Some("large"), Some("sv_SE"))
    );
}

#[test]
fn automatic_aspect_ratio_is_the_declared_one_nearest_the_screen() {
    let t = Synth::new();
    t.capabilities("<aspectRatio>16:9</aspectRatio><aspectRatio>16:10</aspectRatio><aspectRatio>4:3</aspectRatio><aspectRatio>16:10_vertical</aspectRatio>").theme("");
    let caps = capabilities::read(&t.dir());
    for (w, h, expected) in [
        (1280, 800, "16:10"),
        (1920, 1200, "16:10"),
        (1920, 1080, "16:9"),
        (1024, 768, "4:3"),
        (1280, 1024, "4:3"),
        (800, 1280, "16:10_vertical"),
        (2560, 1080, "16:9"),
    ] {
        assert_eq!(Selection::resolve(&caps, &ch().screen(w, h)).aspect_ratio.as_deref(), Some(expected), "{w}x{h}");
        assert_eq!(Selection::resolve(&caps, &ch().screen(w, h).aspect("4:3")).aspect_ratio.as_deref(), Some("4:3"));
    }
}

#[test]
fn without_declared_aspect_ratios_none_is_selected() {
    let t = Synth::new();
    t.capabilities("").theme("");
    assert_eq!(Selection::resolve(&capabilities::read(&t.dir()), &ch()).aspect_ratio, None);
}

#[test]
fn no_media_is_tried_before_no_videos_and_the_triggers_are_one_step_deep() {
    let t = Synth::new();
    t.capabilities(VARIANTS).theme("");
    let caps = capabilities::read(&t.dir());
    let cases: [(&[&str], &str); 5] = [
        (&["miximage", "screenshot", "cover", "video"], "withVideos"),
        (&["screenshot"], "withoutVideos"),
        (&["video"], "noGameMedia"),
        (&[], "noGameMedia"),
        (&["marquee"], "noGameMedia"),
    ];
    for (present, expected) in cases {
        let result = apply_triggers(&caps, Some("withVideos"), Some(&MediaPresence::of(present)), true, &mut Diagnostics::default());
        assert_eq!(result.as_deref(), Some(expected), "{present:?}");
    }
}

#[test]
fn a_variant_reached_by_a_trigger_does_not_trigger_again() {
    let t = Synth::new();
    t.capabilities(VARIANTS).theme("");
    let caps = capabilities::read(&t.dir());
    let result = apply_triggers(&caps, Some("withVideos"), Some(&MediaPresence::of(&["screenshot"])), true, &mut Diagnostics::default());
    assert_eq!(result.as_deref(), Some("withoutVideos"));
}

#[test]
fn triggers_are_off_when_disabled_or_when_media_is_unknown() {
    let t = Synth::new();
    t.capabilities(VARIANTS).theme("");
    let caps = capabilities::read(&t.dir());
    assert_eq!(
        apply_triggers(&caps, Some("withVideos"), Some(&MediaPresence::default()), false, &mut Diagnostics::default()).as_deref(),
        Some("withVideos")
    );
    assert_eq!(apply_triggers(&caps, Some("withVideos"), None, true, &mut Diagnostics::default()).as_deref(), Some("withVideos"));
}

#[test]
fn triggers_change_the_gamelist_view_only() {
    let t = Synth::new();
    t.capabilities(VARIANTS).theme(
        r#"<variant name="withVideos"><view name="system, gamelist"><text name="which"><text>with</text></text></view></variant>
        <variant name="noGameMedia"><view name="system, gamelist"><text name="which"><text>none</text></text></view></variant>"#,
    );
    let theme = t.load_with(&ch(), &snes(), Some(&MediaPresence::default()));
    assert_eq!(theme.selection.variant.as_deref(), Some("withVideos"));
    assert_eq!(theme.gamelist_variant.as_deref(), Some("noGameMedia"));
    assert_eq!(text(&theme, "system", "which").as_deref(), Some("with"));
    assert_eq!(text(&theme, "gamelist", "which").as_deref(), Some("none"));
}

#[test]
fn transitions_automatic_takes_the_variant_profile_then_the_first_declared_and_a_choice_overrides_both() {
    let t = Synth::new();
    t.capabilities(
        r#"<variant name="a"/><variant name="b"/>
        <transitions name="one"><systemToSystem>slide</systemToSystem></transitions>
        <transitions name="two"><systemToSystem>fade</systemToSystem></transitions>
        <suppressTransitionProfiles><entry>builtin-fade</entry></suppressTransitionProfiles>"#,
    )
    .theme("<variant name=\"b\"><transitions>two</transitions></variant>");
    let name = |c| t.load_choices(&c).transitions.name;
    assert_eq!(name(ch().variant("a")), "one");
    assert_eq!(name(ch().variant("b")), "two");
    assert_eq!(name(ch().variant("b").transitions("one")), "one");
    assert_eq!(name(ch().transitions("builtin-slide")), "builtin-slide");
    assert_eq!(name(ch().variant("a").transitions("builtin-fade")), "one");
}

#[test]
fn with_no_profiles_at_all_transitions_are_instant() {
    let t = Synth::new();
    t.capabilities("<suppressTransitionProfiles><entry>builtin-instant</entry><entry>builtin-slide</entry><entry>builtin-fade</entry></suppressTransitionProfiles>")
        .theme("");
    let profile = t.load().transitions;
    assert_eq!(profile.system_to_gamelist, Animation::Instant);
    assert_eq!(profile.startup_to_system, Animation::Instant);
}
