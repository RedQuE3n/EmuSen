// ThemeEsdeRulesTests and ThemeValueRulesTests re-expressed: Q47, Q70 and Q71, each type read as ES-DE 3.4.1 was measured to read it (EmuSen_BigPicture.md §31, §35).
mod common;

use common::*;
use emusen_esde::loader::capabilities;
use emusen_esde::loader::choices::Selection;
use emusen_esde::loader::values::{esde_uint, full_path};
use emusen_esde::loader::{Code, Element, Severity, Theme, Value};

fn gamelist(elements: &str) -> (Synth, Theme) {
    let t = Synth::new();
    t.capabilities("").theme(&format!("<view name=\"gamelist\">{elements}</view>"));
    let theme = t.load();
    (t, theme)
}

fn only<'a>(theme: &'a Theme, kind: &str, name: &str) -> &'a Element {
    assert!(theme.is_themed(), "{}", shown(theme));
    theme.gamelist_view.find(kind, name).unwrap()
}

fn warned(theme: &Theme) -> bool {
    count_at(theme, Code::LenientValue, Severity::Warning) > 0
}

fn near(a: f32, b: f32, places: i32) -> bool {
    (a - b).abs() < 0.5 * 10f32.powi(-places)
}

#[test]
fn a_float_is_its_leading_number() {
    for (written, expected, warn) in
        [("0.85 0.9", 0.85, true), ("0.85", 0.85, false), ("0.9abc", 0.9, true), (" 0.3", 0.3, false), ("abc", 0.1, true)]
    {
        let (_t, theme) = gamelist(&format!("<badges name=\"b\"><folderLinkSize>{written}</folderLinkSize></badges>"));
        assert!(near(only(&theme, "badges", "b").float("folderLinkSize").unwrap(), expected, 5), "{written}");
        assert_eq!(warned(&theme), warn, "{written}");
    }
}

#[test]
fn a_pair_is_split_at_its_first_space_and_each_side_read_as_a_float() {
    for (written, x, y) in [
        ("w 0.02", 0.0, 0.02),
        ("0.5 w", 0.5, 0.0),
        ("0.75 0.3 0.9", 0.75, 0.3),
        (" 0.02 0.62", 0.0, 0.02),
        ("  0.1 0.8", 0.0, 0.1),
        ("0.6  0.5", 0.6, 0.5),
        ("abc def", 0.0, 0.0),
    ] {
        let (_t, theme) = gamelist(&format!("<gamelistinfo name=\"g\"><size>1 1</size><pos>{written}</pos></gamelistinfo>"));
        let pair = only(&theme, "gamelistinfo", "g").pair("pos").unwrap();
        assert!(near(pair.x, x, 5) && near(pair.y, y, 5), "{written}: {pair:?}");
    }
}

#[test]
fn a_pair_with_no_space_unthemes_the_system_as_in_es_de() {
    for written in ["w0.02", "0.6", "0.6\t0.5", "0.6\n0.5", "0.75,0.6"] {
        let (_t, theme) = gamelist(&format!("<gamelistinfo name=\"g\"><size>{written}</size></gamelistinfo>"));
        assert!(!theme.is_themed());
        let errors: Vec<_> = theme.errors().collect();
        assert_eq!(errors.len(), 1, "{written}: {}", shown(&theme));
        assert_eq!(errors[0].code, Code::BadFormat);
        assert!(errors[0].message.contains("no space"));
    }
}

#[test]
fn a_boolean_is_true_when_its_first_character_is_t_y_or_1() {
    let cases = [
        ("true", true),
        ("TRUE", true),
        ("1", true),
        ("yes", true),
        ("y", true),
        ("t", true),
        ("truex", true),
        ("1.0", true),
        ("true ", true),
        ("false", false),
        ("0", false),
        ("no", false),
        ("flase", false),
        ("False", false),
        ("on", false),
        ("2", false),
        ("-1", false),
        (" true ", false),
        (" 1", false),
    ];
    for (written, expected) in cases {
        let (_t, theme) = gamelist(&format!(
            "<text name=\"v\"><text>v</text><visible>{written}</visible><metadataElement>{written}</metadataElement></text>"
        ));
        let text = only(&theme, "text", "v");
        assert_eq!(text.explicit.get("visible"), Some(&Value::Bool(expected)), "{written:?}");
        assert_eq!(text.explicit.get("metadataElement"), Some(&Value::Bool(expected)), "{written:?}");
    }
}

#[test]
fn a_bare_ampersand_is_kept_as_text_and_references_are_decoded() {
    let t = Synth::new();
    t.file(
        "capabilities.xml",
        "<themeCapabilities><colorScheme name=\"a\"><label>Game & Watch</label></colorScheme>\
         <colorScheme name=\"b\"><label>E &foo; F</label></colorScheme><colorScheme name=\"c\"><label>G &amp; H</label></colorScheme>\
         <colorScheme name=\"d\"><label>I &#74; K</label></colorScheme><colorScheme name=\"e\"><label>L &amp M</label></colorScheme></themeCapabilities>",
    )
    .theme("<view name=\"gamelist\"><text name=\"t\"><text>Q & A</text></text></view>");
    let theme = t.load();
    let labels: Vec<&str> = theme.capabilities.color_schemes.iter().map(|c| c.labels["en_US"].as_str()).collect();
    assert_eq!(labels, ["Game & Watch", "E &foo; F", "G & H", "I J K", "L &amp M"]);
    assert_eq!(only(&theme, "text", "t").string("text"), Some("Q & A"));
    assert_eq!(count_at(&theme, Code::LenientXml, Severity::Warning), 2);
}

#[test]
fn text_outside_the_root_element_is_ignored() {
    let t = Synth::new();
    t.capabilities("")
        .file(
            "before.xml",
            "f<!-- a stray letter -->\n<theme><view name=\"gamelist\"><text name=\"b\"><text>b</text></text></view></theme>",
        )
        .file("after.xml", "<theme><view name=\"gamelist\"><text name=\"a\"><text>a</text></text></view></theme>\nzz")
        .theme("<include>./before.xml</include><include>./after.xml</include>");
    let theme = t.load();
    only(&theme, "text", "b");
    assert!(theme.gamelist_view.find("text", "a").is_some());
}

#[test]
fn mismatched_tags_still_untheme_the_system() {
    let t = Synth::new();
    t.capabilities("")
        .file("broken.xml", "<theme><view name=\"gamelist\"><text name=\"x\"><text>x</text></view></theme>")
        .theme("<include>./broken.xml</include>");
    assert_unthemed(&t.load(), Code::MalformedXml);
}

const PROFILES: &str = "<transitions name=\"slideprof\"><gamelistToGamelist>slide</gamelistToGamelist></transitions>\
    <transitions name=\"fadeprof\"><gamelistToGamelist>fade</gamelistToGamelist></transitions><variant name=\"main\"/>";

#[test]
fn transitions_directly_in_theme_are_ignored_with_a_warning() {
    let t = Synth::new();
    t.capabilities(PROFILES).theme("<transitions>fadeprof</transitions><variant name=\"main\"><view name=\"gamelist\"><text name=\"t\"><text>t</text></text></view></variant>");
    let theme = t.load_choices(&ch().variant("main"));
    assert!(theme.is_themed(), "{}", shown(&theme));
    assert_eq!(theme.transitions.name, "slideprof");
    assert!(count_at(&theme, Code::IgnoredTag, Severity::Warning) > 0);
}

#[test]
fn transitions_at_the_top_of_a_file_included_from_a_variant_apply_to_that_variant() {
    let t = Synth::new();
    t.capabilities(PROFILES).file("v.xml", "<theme><transitions>fadeprof</transitions></theme>").theme(
        "<variant name=\"main\"><include>./v.xml</include><view name=\"gamelist\"><text name=\"t\"><text>t</text></text></view></variant>",
    );
    let theme = t.load_choices(&ch().variant("main"));
    assert!(theme.is_themed(), "{}", shown(&theme));
    assert_eq!(theme.transitions.name, "fadeprof");
    assert_eq!(count(&theme, Code::IgnoredTag), 0);
}

// Where ES-DE drew the colour transparent, only its alpha was measured; the other channels follow the same reading.
#[test]
fn a_colour_of_six_or_eight_characters_is_read_as_hexadecimal_the_c_way() {
    let cases = [
        ("FF0000", "FF0000FF", false),
        ("ff0000", "FF0000FF", false),
        ("FF000080", "FF000080", false),
        ("FF0000FF", "FF0000FF", false),
        ("0000FF80", "0000FF80", false),
        ("0xFF0000", "00FF0000", true),
        ("GG0000", "000000FF", true),
        ("FF00ZZ", "00FF00FF", true),
        ("ffff00zz", "00FFFF00", true),
        ("FF00G0", "00FF00FF", true),
        ("0000GG", "000000FF", true),
        ("GGFF0000", "00000000", true),
        ("0x00FF00", "0000FF00", true),
        ("+0FF0000", "00FF0000", true),
        (" 00FF000", "000FF000", true),
        ("-0000001", "FFFFFFFF", true),
        ("0x0000FF", "000000FF", true),
        ("FF00FF 0", "00FF00FF", true),
        ("0xFFFFFF", "00FFFFFF", true),
        ("0xFF00", "00FF00FF", true),
        (" FF000", "0FF000FF", true),
        ("-00001", "FFFFFFFF", true),
        ("+0FF00", "00FF00FF", true),
        ("0x00FF", "0000FFFF", true),
        ("0X0000", "000000FF", true),
        ("  FF00", "00FF00FF", true),
    ];
    for (written, rgba, warn) in cases {
        let (_t, theme) = gamelist(&format!("<text name=\"t\"><text>t</text><backgroundColor>{written}</backgroundColor></text>"));
        assert_eq!(only(&theme, "text", "t").color("backgroundColor").unwrap().to_string(), rgba, "{written:?}");
        assert_eq!(warned(&theme), warn, "{written:?}");
    }
}

#[test]
fn a_colour_of_any_other_length_unthemes_the_system_as_in_es_de() {
    for written in [
        "#FF0000",
        "F00",
        "FF000",
        "FF00000",
        "4c94ff6",
        "FF0000FF0",
        " FF0000",
        "FF0000 ",
        "red",
        "0",
        "-FF0000",
        "1FFFFFFFF",
        "FFFFFFFFFFFFFFFFFF",
    ] {
        let (_t, theme) = gamelist(&format!("<text name=\"t\"><text>t</text><color>{written}</color></text>"));
        assert!(!theme.is_themed(), "{written:?}");
        let errors: Vec<_> = theme.errors().collect();
        assert_eq!(errors.len(), 1, "{written:?}");
        assert_eq!(errors[0].code, Code::BadFormat);
        assert!(errors[0].message.contains("not 6 or 8 characters long"));
    }
}

#[test]
fn a_whole_number_is_read_the_c_way_with_its_base_from_its_prefix_and_kept_to_32_bits() {
    let cases: [(&str, u32, bool); 34] = [
        ("3", 3, false),
        ("3.5", 3, true),
        ("3.9", 3, true),
        ("2.9", 2, true),
        (" 3", 3, false),
        ("3 ", 3, false),
        ("\t3", 3, false),
        ("+3", 3, true),
        ("-1", 4294967295, true),
        ("abc", 0, true),
        ("3abc", 3, true),
        ("0x3", 3, true),
        ("0xA", 10, true),
        ("0x", 0, true),
        ("1e1", 1, true),
        (".5", 0, true),
        ("3 4", 3, true),
        ("3,5", 3, true),
        ("-0", 0, true),
        ("4294967296", 0, true),
        ("4294967298", 2, true),
        ("-4294967294", 2, true),
        ("4294967295", 4294967295, false),
        ("010", 8, true),
        ("011", 9, true),
        ("012", 10, true),
        ("0010", 8, true),
        ("08", 0, true),
        ("09", 0, true),
        ("0x8", 8, true),
        ("0X9", 9, true),
        (" 010", 8, true),
        ("99999999999", 1215752191, true),
        ("99999999999999999999", 4294967295, true),
    ];
    for (written, expected, warn) in cases {
        let (value, exact) = esde_uint(written);
        assert_eq!((value, !exact), (expected, warn), "{written:?}");
    }
}

#[test]
fn badge_lines_and_items_per_line_keep_1_to_10_and_reset_the_rest_to_the_default_with_a_warning() {
    let cases = [
        ("1", Some(1)),
        ("10", Some(10)),
        ("3.5", Some(3)),
        ("010", Some(8)),
        ("0xA", Some(10)),
        ("1e1", Some(1)),
        ("0", None),
        ("11", None),
        ("-1", None),
        ("abc", None),
        ("08", None),
        ("4294967296", None),
    ];
    for (written, expected) in cases {
        let (_t, theme) = gamelist(&format!("<badges name=\"b\"><itemsPerLine>{written}</itemsPerLine><lines>{written}</lines></badges>"));
        let b = only(&theme, "badges", "b");
        assert_eq!(b.uint("itemsPerLine"), Some(expected.unwrap_or(4)), "{written}");
        assert_eq!(b.uint("lines"), Some(expected.unwrap_or(3)), "{written}");
        assert_eq!(count_at(&theme, Code::InvalidValue, Severity::Warning), if expected.is_none() { 2 } else { 0 }, "{written}");
    }
}

// P173: Canvas and Iconic write 3.5, which ES-DE draws as 3; the probe drew 1.5 and 1.9 as 1.
#[test]
fn a_carousel_count_written_as_a_fraction_keeps_its_whole_part_and_the_system_themed() {
    for (written, expected) in [("3.5", 3), ("4.5", 4), ("3.25", 3), ("1.5", 1), ("1.9", 1), ("abc", 0), ("-1", 20)] {
        let (_t, theme) =
            gamelist(&format!("<carousel name=\"c\"><itemsBeforeCenter>{written}</itemsBeforeCenter><itemsAfterCenter>{written}</itemsAfterCenter></carousel>"));
        let c = only(&theme, "carousel", "c");
        assert_eq!((c.uint("itemsBeforeCenter"), c.uint("itemsAfterCenter")), (Some(expected), Some(expected)), "{written}");
    }
}

#[test]
fn a_float_reads_exponents_and_hexadecimal_and_an_unbounded_infinity_is_0() {
    let cases = [
        ("1e-1", 0.1, false),
        ("5E-2", 0.05, false),
        ("0x1p-2", 0.25, true),
        ("0x0.4p0", 0.25, true),
        ("0x0.8", 0.5, true),
        (".5", 0.5, false),
        ("5e-1", 0.5, false),
        ("+0.5", 0.5, false),
        ("0.1e", 0.1, true),
        ("0.1e+", 0.1, true),
        ("0.1e-", 0.1, true),
        ("0x", 0.0, true),
        ("0.3.4", 0.3, true),
        ("-0x0.1p0", -0.0625, true),
        ("inf", 0.0, true),
        ("nan", 0.0, true),
    ];
    for (written, x, warn) in cases {
        let (_t, theme) = gamelist(&format!("<text name=\"t\"><text>t</text><pos>{written} 0.5</pos></text>"));
        assert!(near(only(&theme, "text", "t").pair("pos").unwrap().x, x, 6), "{written}");
        assert_eq!(warned(&theme), warn, "{written}");
    }
}

#[test]
fn an_infinite_float_is_clamped_to_the_range_and_nan_draws_nothing() {
    let cases = [
        ("1", 1.0),
        ("0", 0.0),
        ("0.5", 0.5),
        ("inf", 1.0),
        ("-inf", 0.0),
        ("nan", 0.0),
        ("infinity", 1.0),
        ("1e999", 1.0),
        ("-1e999", 0.0),
        ("0x1p-1", 0.5),
        ("1e", 1.0),
        ("NAN", 0.0),
        ("INF", 1.0),
        ("1E0", 1.0),
    ];
    for (written, opacity) in cases {
        let (_t, theme) = gamelist(&format!("<text name=\"t\"><text>t</text><opacity>{written}</opacity></text>"));
        assert!(near(only(&theme, "text", "t").float("opacity").unwrap(), opacity, 6), "{written}");
    }
}

#[test]
fn an_enumerated_string_must_match_exactly_and_untrimmed_or_the_default_applies() {
    let cases = [
        ("right", Some("right")),
        ("center", Some("center")),
        ("left", Some("left")),
        ("RIGHT", None),
        (" right", None),
        ("right ", None),
        ("Right", None),
        ("middle", None),
        ("LEFT", None),
        ("Left", None),
    ];
    for (written, expected) in cases {
        let (_t, theme) = gamelist(&format!(
            "<text name=\"t\"><text>t</text><horizontalAlignment>{written}</horizontalAlignment><metadata>{written}</metadata></text>"
        ));
        let t = only(&theme, "text", "t");
        let got = match t.explicit.get("horizontalAlignment") {
            Some(Value::Str(s)) => Some(s.as_str()),
            _ => None,
        };
        assert_eq!(got, expected, "{written:?}");
        assert!(!t.explicit.contains_key("metadata"));
        assert_eq!(count_at(&theme, Code::InvalidValue, Severity::Warning), if expected.is_none() { 2 } else { 1 }, "{written:?}");
    }
}

#[test]
fn a_metadata_name_that_is_not_exact_leaves_the_text_shown() {
    for written in [" name", "Name", "name ", "nosuchfield"] {
        let (_t, theme) = gamelist(&format!("<text name=\"t\"><text>TEXT</text><metadata>{written}</metadata></text>"));
        let t = only(&theme, "text", "t");
        assert_eq!(t.string("metadata"), None);
        assert_eq!(t.string("text"), Some("TEXT"));
    }
}

#[test]
fn text_keeps_its_spaces() {
    let (_t, theme) = gamelist("<text name=\"t\"><text>  PADDED  </text></text>");
    assert_eq!(only(&theme, "text", "t").string("text"), Some("  PADDED  "));
}

#[test]
fn an_unknown_image_type_hides_the_element_and_keeps_the_system_themed() {
    let cases = [
        ("cover,screenshot", true),
        ("cover screenshot", true),
        ("cover, screenshot", true),
        (" cover", true),
        ("cover ", true),
        ("Cover", false),
        ("boxart", false),
        ("cover;screenshot", false),
        ("screenshot,boxart", false),
    ];
    for (written, drawn) in cases {
        let (_t, theme) =
            gamelist(&format!("<image name=\"i\"><imageType>{written}</imageType></image><text name=\"t\"><text>t</text></text>"));
        assert!(theme.is_themed(), "{}", shown(&theme));
        assert_eq!(theme.gamelist_view.find("image", "i").is_some(), drawn, "{written:?}");
        assert!(theme.gamelist_view.find("text", "t").is_some());
    }
}

fn unique(stem: &str) -> String {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    format!("{stem}-{}-{nanos:x}", std::process::id())
}

#[test]
fn a_path_is_the_theme_folder_only_after_dot_slash_and_is_not_trimmed() {
    let png = unique("q70") + ".png";
    let t = Synth::new();
    let leaf = t.root.file_name().unwrap().to_string_lossy().into_owned();
    t.capabilities("").asset(&format!("img/{png}")).asset(&format!("sub/img/{png}"));
    t.theme(&format!(
        "<variables><imgdir>./img/</imgdir></variables><view name=\"gamelist\">\
         <image name=\"dot\"><path>./img/{png}</path></image><image name=\"bare\"><path>img/{png}</path></image>\
         <image name=\"back\"><path>.\\img\\{png}</path></image><image name=\"lead\"><path> ./img/{png}</path></image>\
         <image name=\"trail\"><path>./img/{png} </path></image><image name=\"case\"><path>./IMG/{upper}</path></image>\
         <image name=\"up\"><path>./../{leaf}/img/{png}</path></image><image name=\"upbare\"><path>../{leaf}/img/{png}</path></image>\
         <image name=\"here\"><path>./img/./{png}</path></image></view><include>./sub/inc.xml</include>",
        upper = png.to_uppercase()
    ))
    .theme_at(&format!("<view name=\"gamelist\"><image name=\"inc\"><path>./img/{png}</path></image><image name=\"var\"><path>${{imgdir}}{png}</path></image></view>"), "sub/inc.xml");
    let theme = t.load();
    let p = |name: &str| only(&theme, "image", name).path("path").unwrap().clone();
    for found in ["dot", "back", "up", "here"] {
        assert!(p(found).exists, "{found}");
        assert_eq!(p(found).absolute, t.path_of(&format!("img/{png}")));
    }
    assert_eq!(p("inc").absolute, t.path_of(&format!("sub/img/{png}")));
    assert_eq!(p("var").absolute, t.path_of(&format!("sub/img/{png}")));
    assert_eq!(p("bare").absolute, full_path(&format!("img/{png}")));
    for missing in ["bare", "lead", "trail", "case", "upbare"] {
        assert!(!p(missing).exists, "{missing}");
    }
    assert_eq!(count_at(&theme, Code::PathMissing, Severity::Warning), 5);
}

#[test]
fn an_include_is_read_untrimmed_and_a_bare_name_is_not_the_theme_folder() {
    for (written, loads) in [("{0}", false), (" ./{0} ", false), ("./{0}\n", false), ("./{1}", false), (".\\{0}", true), ("./{0}", true)] {
        let name = unique("inc") + ".xml";
        let text = written.replace("{0}", &name).replace("{1}", &name.to_uppercase());
        let t = Synth::new();
        t.capabilities("")
            .file(&name, "<theme><view name=\"gamelist\"><text name=\"inc\"><text>i</text></text></view></theme>")
            .theme(&format!("<include>{text}</include><view name=\"gamelist\"><text name=\"t\"><text>t</text></text></view>"));
        let theme = t.load();
        assert_eq!(theme.is_themed(), loads, "{written:?}: {}", shown(&theme));
        if loads {
            assert!(theme.gamelist_view.find("text", "inc").is_some());
        } else {
            assert!(has_error(&theme, Code::IncludeMissing));
        }
    }
}

#[test]
fn selectable_is_false_only_when_its_first_character_is_0_f_or_n() {
    let cases = [
        ("true", true),
        ("yes", true),
        ("1", true),
        ("TRUE", true),
        (" true ", true),
        ("t", true),
        ("Y", true),
        ("", true),
        ("  ", true),
        ("x", true),
        ("2", true),
        ("on", true),
        (" false", true),
        (" no", true),
        ("\ntrue\n", true),
        (" yes", true),
        ("\tfalse", true),
        ("flase", false),
        ("0", false),
        ("no", false),
        ("false", false),
        ("N", false),
        ("f", false),
        ("F", false),
        ("False", false),
        ("NO", false),
        ("0.0", false),
        ("FALSE", false),
    ];
    for (written, selectable) in cases {
        let t = Synth::new();
        t.capabilities(&format!(
            "<variant name=\"v\"><selectable>{written}</selectable></variant>\
             <transitions name=\"p\"><selectable>{written}</selectable><systemToGamelist>fade</systemToGamelist></transitions>"
        ))
        .theme("");
        let caps = capabilities::read(&t.dir());
        assert_eq!(caps.variants[0].selectable, selectable, "{written:?}");
        assert_eq!(caps.transitions[0].selectable, selectable, "{written:?}");
    }
}

// Q71: in ES-DE a variant without <selectable> is not offered, while a transitions profile without one is.
#[test]
fn a_variant_without_selectable_is_not_offered_and_a_profile_without_one_is() {
    let t = Synth::new();
    t.capabilities(
        "<variant name=\"aNone\"/><variant name=\"bTrue\"><selectable>true</selectable></variant>\
         <transitions name=\"tNone\"><systemToGamelist>fade</systemToGamelist></transitions>",
    )
    .theme("");
    let caps = capabilities::read(&t.dir());
    assert_eq!(caps.variants.iter().filter(|v| v.selectable).map(|v| v.name.as_str()).collect::<Vec<_>>(), ["bTrue"]);
    assert!(caps.transitions[0].selectable);
    assert_eq!(Selection::resolve(&caps, &ch()).variant.as_deref(), Some("bTrue"));
    assert_eq!(Selection::resolve(&caps, &ch().variant("aNone")).variant.as_deref(), Some("aNone"));
    assert!(caps.diagnostics.is_empty());
}

#[test]
fn with_no_variant_offered_the_first_declared_is_drawn() {
    let t = Synth::new();
    t.capabilities("<variant name=\"cNone\"/><variant name=\"dNone\"/>").theme("");
    let caps = capabilities::read(&t.dir());
    assert!(caps.variants.iter().all(|v| !v.selectable));
    assert_eq!(Selection::resolve(&caps, &ch()).variant.as_deref(), Some("cNone"));
}
