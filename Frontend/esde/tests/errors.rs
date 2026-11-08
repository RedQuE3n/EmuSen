// ThemeErrorTests re-expressed: what unthemes a system, what resets a value, what is silent (EmuSen_BigPicture.md §12.3).
mod common;

use common::*;
use emusen_esde::loader::{Code, Severity, Theme, DEFAULT_NAME};

fn load_view(t: &Synth, elements: &str, view: &str) -> Theme {
    t.capabilities("").theme(&format!("<view name=\"{view}\">{elements}</view>"));
    t.load()
}

fn gamelist(elements: &str) -> Theme {
    load_view(&Synth::new(), elements, "gamelist")
}

#[test]
fn malformed_xml_unthemes_the_system() {
    let t = Synth::new();
    t.capabilities("").theme("<view name=\"gamelist\"><text name=\"t\"><text>a</view></text>");
    assert_unthemed(&t.load(), Code::MalformedXml);
}

#[test]
fn malformed_xml_in_an_included_file_unthemes_the_system() {
    let t = Synth::new();
    t.capabilities("").file("colors.xml", "<theme><view name=\"gamelist\"></theme>").theme("<include>./colors.xml</include>");
    assert_unthemed(&t.load(), Code::MalformedXml);
}

#[test]
fn malformed_capabilities_unthemes_every_system() {
    let t = Synth::new();
    t.file("capabilities.xml", "<themeCapabilities><variant name=\"a\"></themeCapabilities>").theme("");
    assert_unthemed(&t.load(), Code::MalformedXml);
}

#[test]
fn a_file_whose_root_is_not_theme_unthemes_the_system() {
    let t = Synth::new();
    t.capabilities("").file("theme.xml", "<themes><view name=\"gamelist\"/></themes>");
    assert_unthemed(&t.load(), Code::WrongRoot);
}

#[test]
fn a_missing_theme_file_unthemes_the_system() {
    let t = Synth::new();
    t.capabilities("");
    assert_unthemed(&t.load(), Code::NoThemeFile);
}

#[test]
fn an_unknown_element_unthemes_the_system() {
    assert_unthemed(&gamelist("<stackpanel name=\"s\"><pos>0 0</pos></stackpanel>"), Code::UnknownElement);
}

#[test]
fn an_unknown_property_unthemes_the_system() {
    assert_unthemed(&gamelist("<image name=\"i\"><logoAlignment>left</logoAlignment></image>"), Code::UnknownProperty);
}

#[test]
fn an_unknown_tag_at_theme_level_unthemes_the_system() {
    let t = Synth::new();
    t.capabilities("").theme("<formatVersion>7</formatVersion>");
    assert_unthemed(&t.load(), Code::UnknownTag);
}

#[test]
fn the_legacy_extra_attribute_unthemes_the_system() {
    assert_unthemed(&gamelist("<image name=\"i\" extra=\"true\"><pos>0 0</pos></image>"), Code::LegacySyntax);
}

#[test]
fn an_unknown_view_unthemes_the_system() {
    assert_unthemed(&load_view(&Synth::new(), "<image name=\"i\"><pos>0 0</pos></image>", "detailed"), Code::UnknownView);
}

#[test]
fn an_element_without_a_name_unthemes_the_system() {
    assert_unthemed(&gamelist("<image><pos>0 0</pos></image>"), Code::MissingName);
}

#[test]
fn a_property_with_no_value_unthemes_the_system() {
    assert_unthemed(&gamelist("<image name=\"i\"><origin></origin></image>"), Code::NoValue);
}

#[test]
fn a_value_in_the_wrong_format_unthemes_the_system() {
    for element in [
        "<image name=\"i\"><pos>0.5</pos></image>",
        "<image name=\"i\"><color>FFF</color></image>",
        "<image name=\"i\"><color>#FF0000</color></image>",
    ] {
        assert_unthemed(&gamelist(element), Code::BadFormat);
    }
}

#[test]
fn an_invalid_value_of_a_valid_format_is_reset_to_its_default_with_a_warning() {
    let theme = gamelist("<badges name=\"b\"><horizontalAlignment>leftr</horizontalAlignment><direction>diagonal</direction></badges>");
    assert!(theme.is_themed());
    let b = theme.gamelist_view.find("badges", "b").unwrap();
    assert_eq!(b.string("horizontalAlignment"), None);
    assert_eq!(b.string("direction"), Some("row"));
    assert_eq!(count_at(&theme, Code::InvalidValue, Severity::Warning), 2);
}

#[test]
fn an_invalid_image_type_stops_the_element_rendering() {
    let theme = gamelist(
        "<video name=\"gamelistVideo\"><imageType>covr</imageType></video><image name=\"kept\"><imageType>cover</imageType></image>",
    );
    assert!(theme.is_themed());
    assert!(theme.gamelist_view.find("video", "gamelistVideo").is_none());
    assert!(theme.gamelist_view.find("image", "kept").is_some());
    assert!(count_at(&theme, Code::InvalidImageType, Severity::Warning) > 0);
}

#[test]
fn a_repeated_image_type_is_ignored() {
    let theme = gamelist("<image name=\"i\"><imageType>cover, screenshot cover</imageType></image>");
    assert!(theme.is_themed());
    assert!(theme.gamelist_view.find("image", "i").unwrap().list("imageType").is_empty());
}

#[test]
fn a_carousel_uses_at_most_two_image_types() {
    let theme = gamelist("<carousel name=\"c\"><imageType>cover screenshot marquee</imageType></carousel>");
    assert_eq!(theme.gamelist_view.find("carousel", "c").unwrap().list("imageType"), ["cover", "screenshot"]);
}

#[test]
fn an_element_in_a_view_it_does_not_support_is_ignored_with_a_warning() {
    let theme = load_view(
        &Synth::new(),
        "<badges name=\"b\"><pos>0 0</pos></badges><gamelistinfo name=\"g\"><pos>0 0</pos></gamelistinfo>",
        "system",
    );
    assert!(theme.is_themed());
    assert!(theme.system_view.elements.iter().all(|e| e.name == DEFAULT_NAME));
    assert_eq!(count(&theme, Code::ViewRestricted), 2);
}

#[test]
fn a_property_limited_to_the_other_view_is_ignored_with_a_warning() {
    let t = Synth::new();
    t.capabilities("").asset("a.png").theme(
        r#"<view name="system, gamelist"><carousel name="c"><staticImage>./a.png</staticImage><imageType>cover</imageType></carousel></view>"#,
    );
    let theme = t.load();
    assert!(theme.is_themed());
    let system = theme.system_view.find("carousel", "c").unwrap();
    let list = theme.gamelist_view.find("carousel", "c").unwrap();
    assert!(system.path("staticImage").is_some());
    assert!(system.list("imageType").is_empty());
    assert!(list.path("staticImage").is_none());
    assert_eq!(list.list("imageType"), ["cover"]);
    assert_eq!(count(&theme, Code::ViewRestricted), 2);
}

#[test]
fn a_second_primary_element_in_a_view_is_ignored_with_a_warning() {
    let theme = gamelist("<textlist name=\"list\"><pos>0 0</pos></textlist><carousel name=\"c\"><pos>0 0</pos></carousel>");
    assert_eq!(theme.gamelist_view.primary().unwrap().name, "list");
    assert!(theme.gamelist_view.find("carousel", "c").is_none());
    assert!(count(&theme, Code::SecondPrimary) > 0);
}

#[test]
fn a_missing_file_written_out_is_a_warning_and_one_from_a_variable_is_a_debug_note() {
    let t = Synth::new();
    t.capabilities("").theme(
        r#"<variables><logos>./logos</logos></variables>
        <view name="gamelist"><image name="i"><path>./none.png</path><default>${logos}/none.svg</default></image></view>"#,
    );
    let theme = t.load();
    assert!(theme.is_themed());
    let image = theme.gamelist_view.find("image", "i").unwrap();
    assert!(!image.path("path").unwrap().exists);
    assert!(image.path("default").unwrap().from_variable);
    assert_eq!(count_at(&theme, Code::PathMissing, Severity::Warning), 1);
    assert_eq!(count_at(&theme, Code::PathMissing, Severity::Debug), 1);
}

#[test]
fn a_keyed_property_needs_its_key_and_ignores_an_unknown_one() {
    assert_unthemed(&gamelist("<badges name=\"b\"><customBadgeIcon>./x.svg</customBadgeIcon></badges>"), Code::MissingName);
    let unknown =
        gamelist("<badges name=\"b\"><customBadgeIcon badge=\"shiny\">./x.svg</customBadgeIcon><customBadgeIcon badge=\"favorite\">./f.svg</customBadgeIcon></badges>");
    assert!(unknown.is_themed());
    let keys: Vec<&String> = unknown.gamelist_view.find("badges", "b").unwrap().keyed("customBadgeIcon").unwrap().keys().collect();
    assert_eq!(keys, ["favorite"]);
}

#[test]
fn a_property_made_empty_by_its_variables_is_ignored_with_a_warning() {
    let t = Synth::new();
    t.capabilities("").theme("<variables><maker/></variables><view name=\"gamelist\"><text name=\"t\"><text>${maker}</text></text></view>");
    let theme = t.load();
    assert!(theme.is_themed());
    assert_eq!(text(&theme, "gamelist", "t"), None);
    assert!(count_at(&theme, Code::NoValue, Severity::Warning) > 0);
}

#[test]
fn an_unused_attribute_is_ignored_with_a_warning() {
    let theme = gamelist("<image name=\"i\" ifSubset=\"x\"><pos>0 0</pos></image>");
    assert!(theme.is_themed());
    assert!(count(&theme, Code::UnusedAttribute) > 0);
}

#[test]
fn a_batocera_edition_theme_does_not_load() {
    let t = Synth::new();
    t.capabilities("").theme(
        r#"<formatVersion>7</formatVersion>
        <subset name="ratio" appliesTo="system"><include name="16/10">./16-10.xml</include></subset>
        <view name="system"><stackpanel name="s"/></view>"#,
    );
    let theme = t.load();
    assert!(!theme.is_themed());
    assert!(has_error(&theme, Code::UnknownElement));
    assert!(theme.errors().filter(|d| d.code == Code::UnknownTag).count() >= 2);
}
