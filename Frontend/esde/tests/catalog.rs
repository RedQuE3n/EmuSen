// ThemeCatalogTests re-expressed: every documented element type and property, set, typed, defaulted and clamped, on synthetic themes (EmuSen_BigPicture.md §12.2).
mod common;

use std::collections::{BTreeMap, HashSet};

use common::*;
use emusen_esde::loader::catalog::{catalog, Group, PairRule, PropertySpec, PropertyType, Scope, Shape};
use emusen_esde::loader::values::number;
use emusen_esde::loader::{Binding, Color, Pair, Severity, ThemePath, Value};

#[test]
fn the_catalogue_holds_themes_md_s_fifteen_reference_elements_and_sound_with_468_properties() {
    let expected: BTreeMap<&str, usize> = [
        ("carousel", 71),
        ("grid", 64),
        ("textlist", 38),
        ("image", 35),
        ("video", 41),
        ("animation", 22),
        ("badges", 32),
        ("text", 35),
        ("datetime", 25),
        ("gamelistinfo", 15),
        ("rating", 17),
        ("gameselector", 3),
        ("helpsystem", 31),
        ("systemstatus", 19),
        ("clock", 19),
        ("sound", 1),
    ]
    .into_iter()
    .collect();
    let actual: BTreeMap<&str, usize> = catalog().elements.iter().map(|e| (e.kind.as_str(), e.properties.len())).collect();
    assert_eq!(actual, expected);
    assert_eq!(catalog().property_count(), 468);
}

// Spot checks read from THEMES.md by hand, independent of how the catalogue was drafted.
#[test]
fn catalogue_entries_match_themes_md() {
    use PropertyType::*;
    #[allow(clippy::type_complexity)]
    let cases: [(&str, &str, PropertyType, Option<&str>, Option<f32>, Option<f32>); 39] = [
        ("carousel", "pos", Pair, Some("0 0.38378"), None, None),
        ("carousel", "size", Pair, Some("1 0.2324"), Some(0.05), Some(2.0)),
        ("carousel", "maxItemCount", Float, Some("3"), Some(0.5), Some(30.0)),
        ("carousel", "itemScale", Float, Some("1.2"), Some(0.2), Some(3.0)),
        ("carousel", "color", Color, Some("FFFFFFD8"), None, None),
        ("carousel", "unfocusedItemOpacity", Float, Some("0.5"), Some(0.1), Some(1.0)),
        ("carousel", "fontSize", Float, Some("0.085"), Some(0.001), Some(1.5)),
        ("grid", "itemSize", Pair, Some("0.15 0.25"), Some(0.05), Some(1.0)),
        ("grid", "itemScale", Float, Some("1.05"), Some(0.5), Some(2.0)),
        ("grid", "unfocusedItemOpacity", Float, Some("1"), Some(0.1), Some(1.0)),
        ("textlist", "selectorColor", Color, Some("333333FF"), None, None),
        ("textlist", "primaryColor", Color, Some("0000FFFF"), None, None),
        ("textlist", "secondaryColor", Color, Some("00FF00FF"), None, None),
        ("textlist", "lineSpacing", Float, Some("1.5"), Some(0.5), Some(3.0)),
        ("image", "zIndex", Float, Some("30"), None, None),
        ("image", "size", Pair, None, Some(0.001), Some(3.0)),
        ("image", "cornerRadius", Float, Some("0"), Some(0.0), Some(0.5)),
        ("video", "delay", Float, Some("1.5"), Some(0.0), Some(15.0)),
        ("video", "pillarboxThreshold", Pair, Some("0.85 0.90"), Some(0.2), Some(1.0)),
        ("video", "fadeInTime", Float, Some("1"), Some(0.0), Some(8.0)),
        ("animation", "speed", Float, Some("1"), Some(0.2), Some(3.0)),
        ("animation", "zIndex", Float, Some("35"), None, None),
        ("badges", "lines", UInt, Some("3"), None, None),
        ("badges", "itemsPerLine", UInt, Some("4"), None, None),
        ("badges", "itemMargin", Pair, Some("0.01 0.01"), Some(0.0), Some(0.2)),
        ("text", "color", Color, Some("000000FF"), None, None),
        ("text", "containerResetDelay", Float, Some("7"), Some(0.0), Some(20.0)),
        ("datetime", "format", String, Some("%Y-%m-%d"), None, None),
        ("gamelistinfo", "zIndex", Float, Some("45"), None, None),
        ("rating", "size", Pair, Some("0 0.06"), Some(0.01), Some(1.0)),
        ("gameselector", "gameCount", UInt, Some("1"), Some(1.0), Some(30.0)),
        ("helpsystem", "textColor", Color, Some("777777FF"), None, None),
        ("helpsystem", "entrySpacing", Float, Some("0.00833"), Some(0.0), Some(0.04)),
        ("helpsystem", "opacity", Float, Some("1"), Some(0.2), Some(1.0)),
        ("systemstatus", "pos", Pair, Some("0.982 0.016"), None, None),
        ("systemstatus", "origin", Pair, Some("1 0"), Some(0.0), Some(1.0)),
        ("systemstatus", "height", Float, Some("0.035"), Some(0.01), Some(0.5)),
        ("clock", "format", String, Some("%H:%M"), None, None),
        ("clock", "pos", Pair, Some("0.018 0.016"), None, None),
    ];
    for (element, property, kind, default, min, max) in cases {
        let spec = catalog().find(element).unwrap().find(property).unwrap();
        assert_eq!((spec.kind, spec.default.as_deref(), spec.min, spec.max), (kind, default, min, max), "{element}.{property}");
    }
}

#[test]
fn views_groups_and_default_z_index_match_themes_md() {
    let kinds = |f: &dyn Fn(&emusen_esde::loader::catalog::ElementSpec) -> bool| {
        catalog().elements.iter().filter(|e| f(e)).map(|e| e.kind.as_str()).collect::<Vec<_>>()
    };
    assert_eq!(kinds(&|e| e.group == Group::Primary), ["carousel", "grid", "textlist"]);
    assert_eq!(kinds(&|e| e.gamelist && !e.system && !e.all), ["badges", "gamelistinfo"]);
    assert_eq!(kinds(&|e| e.system && !e.gamelist && !e.all), ["gameselector"]);
    let z = |kind: &str| catalog().find(kind).unwrap().default_z_index();
    let expected = [
        ("image", 30.0),
        ("video", 30.0),
        ("animation", 35.0),
        ("badges", 35.0),
        ("text", 40.0),
        ("datetime", 40.0),
        ("gamelistinfo", 45.0),
        ("rating", 45.0),
        ("carousel", 50.0),
        ("grid", 50.0),
        ("textlist", 50.0),
    ];
    for (kind, value) in expected {
        assert_eq!(z(kind), Some(value), "{kind}");
    }
    for kind in ["helpsystem", "clock", "systemstatus"] {
        assert_eq!(z(kind), None);
    }
}

fn f4(v: f32) -> String {
    let text = format!("{v:.4}");
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

fn sample(spec: &PropertySpec, element: &str, t: &Synth) -> (String, Value) {
    let mid = |fraction: f32| match (spec.min, spec.max) {
        (Some(lo), Some(hi)) => lo + (hi - lo) * fraction,
        _ => fraction,
    };
    let round = |v: f32| f4(v).parse::<f32>().unwrap();
    match spec.kind {
        PropertyType::Pair => {
            let pair = if spec.rule == PairRule::OneAxisOnly { Pair::new(0.0, 0.1) } else { Pair::new(round(mid(0.25)), round(mid(0.75))) };
            return (format!("{} {}", f4(pair.x), f4(pair.y)), Value::Pair(pair));
        }
        PropertyType::Path if spec.shape != Shape::Keyed => {
            let relative = format!("assets/{element}-{}.png", spec.name);
            t.asset(&relative);
            return (
                format!("./{relative}"),
                Value::Path(ThemePath {
                    written: format!("./{relative}"),
                    absolute: t.path_of(&relative),
                    from_variable: false,
                    exists: true,
                }),
            );
        }
        PropertyType::Boolean => {
            let b = spec.default.as_deref() != Some("true");
            return (b.to_string(), Value::Bool(b));
        }
        PropertyType::Color => return ("12345678".into(), Value::Color(Color(0x12345678))),
        PropertyType::UInt => {
            let u = spec.min.map(|m| m as u32 + 1).unwrap_or(2);
            return (u.to_string(), Value::UInt(u));
        }
        PropertyType::Float => {
            let f = round(mid(0.25));
            return (f4(f), Value::Float(f));
        }
        _ => {}
    }
    match spec.shape {
        Shape::Enum => {
            let value = spec.values.iter().rev().find(|v| Some(v.as_str()) != spec.default.as_deref()).unwrap().clone();
            (value.clone(), Value::Str(value))
        }
        Shape::List => {
            let item = if spec.values.iter().any(|v| v == "cover") { "cover".to_owned() } else { spec.values[0].clone() };
            (item.clone(), Value::List(vec![item]))
        }
        Shape::Keyed => {
            let key = &spec.values[0];
            let keyed = format!("assets/{element}-{}-{key}.svg", spec.name);
            t.asset(&keyed);
            let path = ThemePath { written: format!("./{keyed}"), absolute: t.path_of(&keyed), from_variable: false, exists: true };
            (format!("./{keyed}"), Value::Keyed([(key.clone(), path)].into_iter().collect()))
        }
        Shape::Single => ("sample text".into(), Value::Str("sample text".into())),
    }
}

fn strip_other_primaries(xml: &str, keep: &str, primaries: &[&str]) -> String {
    let mut xml = xml.to_owned();
    for other in primaries.iter().filter(|p| **p != keep) {
        let open = format!("<{other} name=\"every\">");
        let close = format!("</{other}>");
        while let Some(start) = xml.find(&open) {
            let end = xml[start..].find(&close).unwrap() + start + close.len();
            xml.replace_range(start..end, "");
        }
    }
    xml
}

#[test]
fn every_documented_property_of_every_element_is_read_into_its_type() {
    let t = Synth::new();
    let mut expectations: Vec<(String, String, String, Value)> = vec![];
    let mut views: BTreeMap<&str, String> =
        [("system", String::new()), ("gamelist", String::new()), ("all", String::new())].into_iter().collect();
    for element in &catalog().elements {
        let targets: Vec<&str> = if element.all {
            vec!["all"]
        } else if element.system && element.gamelist {
            vec!["system", "gamelist"]
        } else if element.system {
            vec!["system"]
        } else {
            vec!["gamelist"]
        };
        let mut by_view: BTreeMap<&str, String> = targets.iter().map(|v| (*v, String::new())).collect();
        for property in &element.properties {
            let view = match property.only_in {
                Scope::System => "system",
                Scope::Gamelist => "gamelist",
                Scope::Any => {
                    if targets.contains(&"gamelist") {
                        "gamelist"
                    } else {
                        targets[0]
                    }
                }
            };
            let (text, value) = sample(property, &element.kind, &t);
            let key = property.key_attribute.as_ref().map(|a| format!(" {a}=\"{}\"", property.values[0])).unwrap_or_default();
            by_view.get_mut(view).unwrap().push_str(&format!("<{0}{key}>{text}</{0}>", property.name));
            expectations.push((view.to_owned(), element.kind.clone(), property.name.clone(), value));
        }
        for (view, props) in by_view {
            views.get_mut(view).unwrap().push_str(&format!("<{0} name=\"every\">{props}</{0}>", element.kind));
        }
    }
    t.capabilities("<variant name=\"v\"/>");
    let primaries = ["carousel", "grid", "textlist"];
    let mut seen = HashSet::new();
    for keep in primaries {
        let text: Vec<String> =
            views.iter().map(|(v, body)| format!("<view name=\"{v}\">{}</view>", strip_other_primaries(body, keep, &primaries))).collect();
        t.theme(&text.join("\n"));
        let theme = t.load();
        assert!(theme.is_themed(), "{}", shown(&theme));
        assert!(theme.diagnostics.iter().all(|d| d.severity == Severity::Debug), "{}", shown(&theme));
        for (view, element, property, value) in expectations.iter().filter(|e| !primaries.contains(&e.1.as_str()) || e.1 == keep) {
            let resolved = if view == "all" { theme.sound_view.elements.first() } else { theme.view(view).find(element, "every") };
            let resolved = resolved.unwrap_or_else(|| panic!("{element} missing from the {view} view"));
            let actual = resolved.explicit.get(property.as_str()).unwrap_or_else(|| panic!("{element}.{property} was not read"));
            assert_eq!(actual, value, "{element}.{property}");
            seen.insert((element.clone(), property.clone()));
        }
    }
    assert_eq!(seen.len(), 468);
}

fn normalise(text: &str, kind: PropertyType) -> String {
    match kind {
        PropertyType::Color => {
            if text.len() == 6 {
                text.to_uppercase() + "FF"
            } else {
                text.to_uppercase()
            }
        }
        PropertyType::Float => number(text.parse::<f32>().unwrap() as f64),
        PropertyType::Pair => {
            let (x, y) = text.split_once(' ').unwrap();
            Pair::new(x.parse().unwrap(), y.parse().unwrap()).to_string()
        }
        _ => text.to_owned(),
    }
}

#[test]
fn every_literal_default_is_in_the_effective_values_of_an_element_that_sets_nothing() {
    let mut covered = HashSet::new();
    for primary in ["textlist", "carousel", "grid"] {
        let bare = |pick: &dyn Fn(&emusen_esde::loader::catalog::ElementSpec) -> bool| {
            catalog()
                .elements
                .iter()
                .filter(|e| pick(e) && (e.group != Group::Primary || e.kind == primary))
                .map(|e| format!("<{} name=\"bare\"/>", e.kind))
                .collect::<String>()
        };
        let t = Synth::new();
        t.capabilities("").theme(&format!(
            "<view name=\"gamelist\">{}</view><view name=\"system\">{}</view><view name=\"all\">{}</view>",
            bare(&|e| e.gamelist),
            bare(&|e| e.system),
            bare(&|e| e.all)
        ));
        let theme = t.load();
        assert!(theme.is_themed(), "{}", shown(&theme));
        for (element, view) in
            theme.gamelist_view.elements.iter().map(|e| (e, "gamelist")).chain(theme.system_view.elements.iter().map(|e| (e, "system")))
        {
            for spec in element.spec.properties.iter().filter(|p| {
                p.default.is_some()
                    && match p.only_in {
                        Scope::Any => true,
                        Scope::System => view == "system",
                        Scope::Gamelist => view == "gamelist",
                    }
            }) {
                let value = element.get(&spec.name).unwrap_or_else(|| panic!("{}.{} has no default value", element.kind(), spec.name));
                let shown = match value {
                    Value::List(l) => l.join(" "),
                    other => other.to_string(),
                };
                assert_eq!(shown, normalise(spec.default.as_deref().unwrap(), spec.kind), "{}.{}", element.kind(), spec.name);
                covered.insert((element.kind().to_owned(), spec.name.clone()));
            }
        }
    }
    let literal: HashSet<(String, String)> = catalog()
        .elements
        .iter()
        .flat_map(|e| e.properties.iter().filter(|p| p.default.is_some()).map(|p| (e.kind.clone(), p.name.clone())))
        .collect();
    assert!(literal.difference(&covered).next().is_none(), "{:?}", literal.difference(&covered).collect::<Vec<_>>());
    assert_eq!(covered.len(), 336);
}

#[test]
fn defaults_stated_as_another_property_follow_that_property() {
    let t = Synth::new();
    t.capabilities("").theme(
        r#"<view name="gamelist">
            <textlist name="l"><primaryColor>112233</primaryColor><selectedBackgroundColor>44556677</selectedBackgroundColor></textlist>
            <image name="i"><color>AABBCC</color></image>
            <helpsystem name="h"><pos>0.3 0.4</pos><opacity>0.5</opacity></helpsystem>
            <video name="v"><maxSize>0.5 0.5</maxSize></video>
        </view>"#,
    );
    let theme = t.load();
    let view = &theme.gamelist_view;
    let list = view.find("textlist", "l").unwrap();
    assert_eq!(list.color("selectedColor").unwrap().to_string(), "112233FF");
    assert_eq!(list.color("selectedSecondaryColor").unwrap().to_string(), "112233FF");
    assert_eq!(list.color("selectedSecondaryBackgroundColor").unwrap().to_string(), "44556677");
    assert_eq!(view.find("image", "i").unwrap().color("colorEnd").unwrap().to_string(), "AABBCCFF");
    assert_eq!(view.find("helpsystem", "h").unwrap().pair("posDimmed"), Some(Pair::new(0.3, 0.4)));
    assert_eq!(view.find("helpsystem", "h").unwrap().float("opacityDimmed"), Some(0.5));
    assert_eq!(view.find("video", "v").unwrap().pair("imageMaxSize"), Some(Pair::new(0.5, 0.5)));
}

#[test]
fn defaults_stated_in_words_are_computed() {
    let t = Synth::new();
    t.capabilities("<aspectRatio>16:9</aspectRatio><aspectRatio>16:9_vertical</aspectRatio>").theme(
        r#"<view name="system"><carousel name="c"><pos>0 0</pos></carousel></view>
        <view name="gamelist">
            <textlist name="l"><size>0.4 0.8</size><fontSize>0.04</fontSize></textlist>
            <helpsystem name="h"><scope>view</scope></helpsystem>
            <image name="straight"><rotation>-90</rotation></image>
            <image name="tilted"><rotation>45</rotation></image>
            <text name="description"><metadata>description</metadata></text>
            <text name="name"><metadata>name</metadata></text>
            <text name="scroller"><container>true</container><containerType>horizontal</containerType></text>
            <datetime name="last"><metadata>lastplayed</metadata></datetime>
            <datetime name="released"><metadata>releasedate</metadata></datetime>
        </view>"#,
    );
    let horizontal = t.load_choices(&ch().aspect("16:9"));
    let g = &horizontal.gamelist_view;
    assert_eq!(horizontal.system_view.find("carousel", "c").unwrap().string("text"), Some("Super Nintendo"));
    assert_eq!(g.find("textlist", "l").unwrap().float("selectorWidth"), Some(0.4));
    assert!((g.find("textlist", "l").unwrap().float("selectorHeight").unwrap() - 0.06).abs() < 5e-6);
    assert_eq!(g.find("helpsystem", "h").unwrap().pair("pos"), Some(Pair::new(0.012, 0.9515)));
    assert_eq!(g.find("helpsystem", "h").unwrap().float("fontSize"), Some(0.035));
    assert_eq!(g.find("image", "straight").unwrap().string("interpolation"), Some("nearest"));
    assert_eq!(g.find("image", "tilted").unwrap().string("interpolation"), Some("linear"));
    assert_eq!(g.find("text", "description").unwrap().bool("container"), Some(true));
    assert_eq!(g.find("text", "name").unwrap().bool("container"), Some(false));
    assert_eq!(g.find("text", "description").unwrap().float("containerStartDelay"), Some(4.5));
    assert_eq!(g.find("text", "scroller").unwrap().float("containerStartDelay"), Some(1.5));
    assert_eq!(g.find("datetime", "last").unwrap().bool("displayRelative"), Some(true));
    assert_eq!(g.find("datetime", "released").unwrap().bool("displayRelative"), Some(false));
    let vertical = t.load_choices(&ch().aspect("16:9_vertical"));
    assert_eq!(vertical.gamelist_view.find("helpsystem", "h").unwrap().pair("pos"), Some(Pair::new(0.012, 0.975)));
    assert_eq!(vertical.gamelist_view.find("helpsystem", "h").unwrap().float("fontSize"), Some(0.025));
}

#[test]
fn colours_of_six_digits_are_opaque_and_of_eight_carry_their_alpha() {
    let t = Synth::new();
    t.capabilities("").theme(
        "<view name=\"gamelist\"><text name=\"a\"><color>0a0B0c</color></text><text name=\"b\"><color>0A0B0C80</color></text></view>",
    );
    let theme = t.load();
    let a = theme.gamelist_view.find("text", "a").unwrap().color("color").unwrap();
    let b = theme.gamelist_view.find("text", "b").unwrap().color("color").unwrap();
    assert_eq!((a.r(), a.g(), a.b(), a.a()), (10, 11, 12, 255));
    assert_eq!(b.a(), 0x80);
}

#[test]
fn booleans_accept_true_false_one_and_zero() {
    let t = Synth::new();
    t.capabilities("").theme("<view name=\"gamelist\"><image name=\"a\"><tile>1</tile><mipmap>TRUE</mipmap><visible>0</visible><flipVertical>false</flipVertical></image></view>");
    let theme = t.load();
    let a = theme.gamelist_view.find("image", "a").unwrap();
    assert_eq!(
        (a.bool("tile"), a.bool("mipmap"), a.bool("visible"), a.bool("flipVertical")),
        (Some(true), Some(true), Some(false), Some(false))
    );
}

#[test]
fn numbers_outside_their_range_are_clamped_silently() {
    let t = Synth::new();
    t.capabilities("").theme(
        r#"<view name="gamelist">
            <carousel name="c"><maxItemCount>99</maxItemCount><itemScale>0.01</itemScale><size>5 0.001</size><itemsBeforeCenter>40</itemsBeforeCenter></carousel>
            <image name="i"><opacity>1.5</opacity><brightness>-3</brightness><origin>-1 2</origin><pos>-0.5 1.5</pos></image>
            <rating name="r"><size>0 0.9</size></rating>
            <rating name="rx"><size>3 0</size></rating>
        </view>"#,
    );
    let theme = t.load();
    assert!(theme.is_themed());
    assert!(theme.diagnostics.iter().all(|d| d.severity == Severity::Debug), "{}", shown(&theme));
    let c = theme.gamelist_view.find("carousel", "c").unwrap();
    assert_eq!(c.float("maxItemCount"), Some(30.0));
    assert_eq!(c.float("itemScale"), Some(0.2));
    assert_eq!(c.pair("size"), Some(Pair::new(2.0, 0.05)));
    assert_eq!(c.uint("itemsBeforeCenter"), Some(20));
    let i = theme.gamelist_view.find("image", "i").unwrap();
    assert_eq!((i.float("opacity"), i.float("brightness")), (Some(1.0), Some(-2.0)));
    assert_eq!(i.pair("origin"), Some(Pair::new(0.0, 1.0)));
    assert_eq!(i.pair("pos"), Some(Pair::new(-0.5, 1.5)));
    assert_eq!(theme.gamelist_view.find("rating", "r").unwrap().pair("size"), Some(Pair::new(0.0, 0.5)));
    assert_eq!(theme.gamelist_view.find("rating", "rx").unwrap().pair("size"), Some(Pair::new(1.0, 0.0)));
}

#[test]
fn sizes_with_a_zero_axis_are_automatic_and_both_zero_is_clamped_to_the_minimum() {
    let t = Synth::new();
    t.capabilities("").theme(
        r#"<view name="gamelist">
            <image name="auto"><size>0 0.5</size></image>
            <image name="none"><size>0 0</size></image>
            <video name="v"><size>0 0</size></video>
            <text name="t"><size>0 0</size></text>
        </view>"#,
    );
    let theme = t.load();
    let view = &theme.gamelist_view;
    assert_eq!(view.find("image", "auto").unwrap().pair("size"), Some(Pair::new(0.0, 0.5)));
    assert_eq!(view.find("image", "none").unwrap().pair("size"), Some(Pair::new(0.001, 0.001)));
    assert_eq!(view.find("video", "v").unwrap().pair("size"), Some(Pair::new(0.01, 0.01)));
    assert_eq!(view.find("text", "t").unwrap().pair("size"), Some(Pair::new(0.0, 0.0)));
}

#[test]
fn minus_one_means_match_the_other_axis_where_themes_md_allows_it() {
    let t = Synth::new();
    t.capabilities("").theme(
        r#"<view name="gamelist">
            <grid name="g"><itemSize>-1 0.2</itemSize><itemSpacing>0.02 -1</itemSpacing></grid>
            <badges name="b"><itemMargin>-1 0.05</itemMargin></badges>
        </view>"#,
    );
    let theme = t.load();
    let view = &theme.gamelist_view;
    assert_eq!(view.find("grid", "g").unwrap().pair("itemSize"), Some(Pair::new(-1.0, 0.2)));
    assert_eq!(view.find("grid", "g").unwrap().pair("itemSpacing"), Some(Pair::new(0.02, -1.0)));
    assert_eq!(view.find("badges", "b").unwrap().pair("itemMargin"), Some(Pair::new(-1.0, 0.05)));
}

#[test]
fn lists_split_on_commas_and_whitespace_and_the_image_shortcut_binds_four_media() {
    let t = Synth::new();
    t.capabilities("").theme(
        "<view name=\"gamelist\">
            <badges name=\"b\"><slots>favorite,
                completed\tkidgame , broken</slots></badges>
            <image name=\"i\"><imageType>image</imageType></image>
            <video name=\"v\"><imageType>none</imageType></video>
        </view>",
    );
    let theme = t.load();
    let view = &theme.gamelist_view;
    assert_eq!(view.find("badges", "b").unwrap().list("slots"), ["favorite", "completed", "kidgame", "broken"]);
    let media = Binding { kind: "media".into(), names: ["miximage", "screenshot", "titlescreen", "cover"].map(String::from).to_vec() };
    assert_eq!(view.find("image", "i").unwrap().bindings, [media]);
    assert!(view.find("video", "v").unwrap().bindings[0].names.is_empty());
}

#[test]
fn media_and_data_the_theme_does_not_supply_are_named_bindings() {
    let t = Synth::new();
    t.capabilities("").theme(
        r#"<view name="system">
            <carousel name="c"><pos>0 0</pos></carousel>
            <text name="count"><systemdata>gamecount</systemdata></text>
            <gameselector name="g"><selection>lastplayed</selection></gameselector>
        </view>
        <view name="gamelist">
            <textlist name="l"><pos>0 0</pos></textlist>
            <video name="art"><imageType>cover, screenshot</imageType></video>
            <text name="d"><metadata>description</metadata></text>
            <datetime name="r"><metadata>releasedate</metadata></datetime>
            <rating name="s"><pos>0 0</pos></rating>
            <badges name="b"><slots>favorite</slots></badges>
            <gamelistinfo name="gi"><pos>0 0</pos></gamelistinfo>
            <helpsystem name="h"><entries>a b</entries></helpsystem>
            <clock name="k"><pos>0 0</pos></clock>
            <systemstatus name="st"><entries>battery</entries></systemstatus>
        </view>"#,
    );
    let theme = t.load();
    let show = |view: &str, kind: &str, name: &str| {
        theme.view(view).find(kind, name).unwrap().bindings.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(" ")
    };
    assert_eq!(show("system", "carousel", "c"), "systems");
    assert_eq!(show("system", "text", "count"), "systemdata:gamecount");
    assert_eq!(show("system", "gameselector", "g"), "gameselector:lastplayed");
    assert_eq!(show("gamelist", "textlist", "l"), "games");
    assert_eq!(show("gamelist", "video", "art"), "videoFallbackImage:cover,screenshot");
    assert_eq!(show("gamelist", "text", "d"), "metadata:description");
    assert_eq!(show("gamelist", "datetime", "r"), "metadata:releasedate");
    assert_eq!(show("gamelist", "rating", "s"), "metadata:rating");
    assert_eq!(show("gamelist", "badges", "b"), "badges:favorite");
    assert_eq!(show("gamelist", "gamelistinfo", "gi"), "gamelistinfo");
    assert_eq!(show("gamelist", "helpsystem", "h"), "help:a,b");
    assert_eq!(show("gamelist", "clock", "k"), "clock");
    assert_eq!(show("gamelist", "systemstatus", "st"), "systemstatus:battery");
}

#[test]
fn only_video_playback_properties_are_marked_unsupported() {
    let deferred: Vec<String> = catalog()
        .elements
        .iter()
        .flat_map(|e| e.properties.iter().filter(|p| p.deferred.is_some()).map(move |p| format!("{}.{}", e.kind, p.name)))
        .collect();
    assert_eq!(
        deferred,
        [
            "video.path",
            "video.default",
            "video.iterationCount",
            "video.onIterationsDone",
            "video.audio",
            "video.videoCornerRadius",
            "video.pillarboxes",
            "video.pillarboxThreshold",
            "video.scanlines",
            "video.delay",
            "video.fadeInType",
            "video.fadeInTime"
        ]
    );
}
