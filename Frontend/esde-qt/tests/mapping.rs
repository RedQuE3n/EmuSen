//! The mapping is checked, not asserted (EmuSen_Qt.md §4.8.5): every pair `mapping.tsv` marks drawn is rendered through
//! the bridge and `View.qml` on the GPU tier with two values, and the two pictures must differ; every drawn pair has a
//! case, and a case's pair is drawn or waits for a later family. Translated from the C# SceneMappingTests (EmuSen_BigPicture.md §13.5).

mod common;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use common::{settings, until, Scratch};
use emusen_esde::loader::catalog::Scope;
use emusen_esde::loader::{self, catalog, Severity, ThemeSystem};
use emusen_esde::scene::mapping::{self, Status};
use emusen_kit_core::input::UiButton;
use emusen_qt_test::{Harness, Scene};

const W: i32 = 640;
const H: i32 = 400;
const CASES: &str = include_str!("themes/mapping-cases.tsv");

// What each element needs to be visible at all; a case's own property replaces its entry here.
const BASE: [(&str, &str); 15] = [
    ("image", "pos=0.1 0.1;size=0.5 0.6;path={A}"),
    ("video", "pos=0.1 0.1;size=0.5 0.6;imageType=cover"),
    ("text", "pos=0.05 0.05;size=0.9 0.6;text=Themed words here and more;fontSize=0.1;color=FFFFFF"),
    ("datetime", "pos=0.05 0.05;size=0.9 0.6;metadata=releasedate;fontSize=0.1;color=FFFFFF"),
    ("carousel", "pos=0 0;size=1 1;imageType=cover;maxItemCount=3;itemSize=0.3 0.6"),
    ("textlist", "pos=0 0;size=1 1;fontSize=0.1;primaryColor=FFFFFF"),
    ("grid", "pos=0 0;size=1 1;imageType=cover;itemSize=0.3 0.4;itemScale=1.2;unfocusedItemOpacity=0.5"),
    ("rating", "pos=0.1 0.1;size=0 0.2;filledPath={S1};unfilledPath={S2}"),
    (
        "badges",
        "pos=0 0;size=1 0.5;slots=favorite,completed;lines=1;itemsPerLine=4;itemMargin=0 0;\
         customBadgeIcon:favorite=<customBadgeIcon badge=\"favorite\">{S1}</customBadgeIcon>;\
         customBadgeIcon:completed=<customBadgeIcon badge=\"completed\">{S2}</customBadgeIcon>",
    ),
    ("helpsystem", "pos=0 0;fontSize=0.1;textColor=FFFFFF;iconColor=FFFFFF"),
    ("clock", "pos=0 0;fontSize=0.1;color=FFFFFF"),
    ("systemstatus", "pos=1 0;origin=1 0;height=0.15;entries=all"),
    ("gameselector", "selection=lastplayed;gameCount=3"),
    ("animation", "pos=0.1 0.1;size=0.5 0.6;path={G}"),
    ("gamelistinfo", "pos=0.05 0.05;size=0.9 0.3;color=FFFFFF;fontSize=0.1;backgroundColor=203040"),
];

const COMMON: [&str; 10] = ["pos", "origin", "size", "rotation", "rotationOrigin", "zIndex", "opacity", "visible", "metadataElement", "scope"];

#[derive(Clone, Debug, Default)]
struct Case {
    kind: String,
    property: String,
    a: String,
    b: String,
    view: Option<String>,
    system: usize,
    game: usize,
    remove: Vec<String>,
    context: String,
    at: Option<i32>,
    moves: Option<String>,
    hide: bool,
    beside: String,
}

impl std::fmt::Display for Case {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.kind, self.property)
    }
}

fn base(kind: &str) -> &'static str {
    BASE.iter().find(|(k, _)| *k == kind).map(|(_, b)| *b).unwrap()
}

fn has(kind: &str, property: &str) -> bool {
    catalog().find(kind).and_then(|e| e.find(property)).is_some()
}

fn listed() -> Vec<Case> {
    let opt = |v: &str| (v != "-").then(|| v.to_owned());
    CASES
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            assert_eq!(c.len(), 13, "mapping-cases.tsv: thirteen columns: {l}");
            Case {
                kind: c[0].into(),
                property: c[1].into(),
                a: c[2].into(),
                b: c[3].into(),
                view: opt(c[4]),
                system: opt(c[5]).map_or(1, |v| v.parse().unwrap()),
                game: opt(c[6]).map_or(4, |v| v.parse().unwrap()),
                remove: opt(c[7]).map(|v| v.split(',').map(str::to_owned).collect()).unwrap_or_default(),
                context: opt(c[8]).unwrap_or_default(),
                at: opt(c[9]).map(|v| v.parse().unwrap()),
                moves: opt(c[10]),
                hide: c[11] == "true",
                beside: opt(c[12]).unwrap_or_default(),
            }
        })
        .collect()
}

// The properties every element shares, where its type declares them.
fn common_cases(kind: &str) -> Vec<Case> {
    let c = |p: &str, a: &str, b: &str| Case { kind: kind.into(), property: p.into(), a: a.into(), b: b.into(), system: 1, game: 4, ..Case::default() };
    let size = base(kind).split(';').find_map(|p| p.strip_prefix("size=")).unwrap_or("0.5 0.5");
    let mut out = vec![];
    for p in COMMON.iter().filter(|p| has(kind, p)) {
        out.push(match *p {
            "pos" => c(p, "0.1 0.1", "0.3 0.2"),
            "origin" => Case { context: "pos=0.5 0.5".into(), ..c(p, "0 0", "1 1") },
            "size" => c(p, size, if kind == "rating" { "0 0.3" } else { "0.3 0.3" }),
            "rotation" => c(p, "0", "30"),
            "rotationOrigin" => Case { context: "rotation=30".into(), ..c(p, "0 0", "1 1") },
            "zIndex" => Case { context: "cover".into(), ..c(p, "30", "45") },
            "opacity" => c(p, "1", "0.5"),
            "visible" => c(p, "true", "false"),
            "metadataElement" => Case { hide: true, ..c(p, "false", "true") },
            _ => c(p, "shared", "none"),
        });
    }
    out
}

fn all_cases() -> Vec<Case> {
    let mut cases = listed();
    for (kind, _) in BASE {
        cases.extend(common_cases(kind));
    }
    cases
}

/// The pictures, icons, animations and faces a case names, written once into the scratch folder.
struct Assets {
    dir: PathBuf,
}

impl Assets {
    fn new(dir: &Path) -> Assets {
        let p = |n: &str| dir.join(n);
        common::bmp(&p("a.bmp"), 60, 80, |x, y| [[[255, 140, 0], [0, 128, 128]], [[128, 0, 128], [255, 215, 0]]][(y >= 40) as usize][(x >= 30) as usize]);
        common::halves(&p("b.bmp"), 80, 50, [128, 0, 128], [255, 215, 0]);
        common::halves(&p("screenshot.bmp"), 80, 70, [128, 0, 128], [255, 215, 0]);
        common::halves(&p("marquee.bmp"), 100, 30, [220, 20, 60], [128, 128, 0]);
        common::halves(&p("other.bmp"), 70, 70, [70, 130, 180], [250, 128, 114]);
        let four = [[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 0]];
        common::gif(&p("four.gif"), 8, 6, four, &[10, 20, 30, 40], |f, _, _| f as u8);
        let checker = [[255, 255, 255], [0, 128, 128], [0, 0, 0], [0, 0, 0]];
        common::gif(&p("checker.gif"), 4, 4, checker, &[10, 10], |f, x, y| ((x as usize + y as usize + f) % 2) as u8);
        let svg = |name: &str, body: &str| {
            let text = format!("<svg xmlns='http://www.w3.org/2000/svg' width='20' height='20' viewBox='0 0 20 20'>{body}</svg>");
            std::fs::write(p(name), text).unwrap();
        };
        svg("icon-1.svg", "<circle cx='10' cy='10' r='9' fill='#fff' fill-opacity='0.5'/>");
        svg("icon-2.svg", "<rect x='2' y='2' width='16' height='16' fill='#888'/>");
        svg("icon-3.svg", "<path d='M10 1L19 19H1Z' fill='#fff'/>");
        Assets { dir: dir.to_owned() }
    }

    fn fill(&self, text: &str) -> String {
        let p = |n: &str| self.dir.join(n).display().to_string();
        let fonts = concat!(env!("CARGO_MANIFEST_DIR"), "/../kit-qt/fonts/");
        text.replace("{A}", &p("a.bmp"))
            .replace("{B}", &p("b.bmp"))
            .replace("{X}", "/nonexistent/emusen/scene/missing.png")
            .replace("{G2}", &p("checker.gif"))
            .replace("{G}", &p("four.gif"))
            .replace("{S1}", &p("icon-1.svg"))
            .replace("{S2}", &p("icon-2.svg"))
            .replace("{S3}", &p("icon-3.svg"))
            .replace("{FT}", &format!("{fonts}BarlowCondensed-Light.ttf"))
            .replace("{FB}", &format!("{fonts}Barlow-Medium.ttf"))
    }

    /// `esdeMedia`: every game has every type, each a different picture, as the C# fixtures' `SceneAssets.Media`.
    fn media(&self, files: &[(String, String)]) -> String {
        let p = |n: &str| self.dir.join(n).display().to_string();
        let types = [
            ("cover", p("a.bmp")),
            ("screenshot", p("screenshot.bmp")),
            ("marquee", p("marquee.bmp")),
            ("miximage", p("other.bmp")),
            ("titlescreen", p("other.bmp")),
            ("backcover", p("other.bmp")),
            ("3dbox", p("other.bmp")),
            ("physicalmedia", p("other.bmp")),
            ("fanart", p("other.bmp")),
        ];
        let mut systems: Vec<String> = files.iter().map(|(s, _)| s.clone()).collect();
        systems.dedup();
        let per = |system: &str| {
            let stems: Vec<&str> = files.iter().filter(|(s, _)| s == system).map(|(_, f)| f.rsplit_once('.').map_or(f.as_str(), |(s, _)| s)).collect();
            let kinds: Vec<String> = types
                .iter()
                .map(|(t, path)| format!("\"{t}\":{{{}}}", stems.iter().map(|s| format!("\"{s}\":\"{path}\"")).collect::<Vec<_>>().join(",")))
                .collect();
            format!("\"{system}\":{{{}}}", kinds.join(","))
        };
        format!("{{{}}}", systems.iter().map(|s| per(s)).collect::<Vec<_>>().join(","))
    }
}

const TITLES: [&str; 12] = [
    "Aurora Drift", "Brass Lantern", "Cobalt Harbor", "Dune Relay", "Ember Circuit", "Fable of Tiles", "Granite Choir", "Hollow Comet", "Ivory Signal",
    "Juniper Vault", "Kestrel Run", "Lumen Garden",
];

// The C# SyntheticLibrary's twelve games, as `esdeShelves` rows; `extra` adds fields to game i.
fn games(esde_system: &str, ext: &str, extra: impl Fn(usize) -> String) -> Vec<String> {
    let genres = ["Platform", "Puzzle", "Racing", "Role playing", "Shooter", "Sports"];
    (0..12)
        .map(|i| {
            let t = TITLES[i];
            let mut f = vec![
                format!("\"file\":\"{t} (Synthetic){ext}\""),
                format!("\"name\":\"{t}\""),
                format!("\"esde_system\":\"{esde_system}\""),
                format!("\"description\":\"{t} is a game invented for EmuSen's tests. It has no story and no pictures of its own; every image shown for it is a flat colour with its name written on it. This second sentence is here so that the description is long enough to wrap over several lines and to be cut off where its box ends.\""),
                format!("\"developer\":\"Studio {}\"", (b'A' + (i % 5) as u8) as char),
                format!("\"publisher\":\"Publisher {}\"", (b'P' + (i % 3) as u8) as char),
                format!("\"genre\":\"{}\"", genres[i % 6]),
                format!("\"players\":\"{}\"", 1 + i % 4),
                format!("\"release_date\":\"{}-{:02}-{:02}\"", 1990 + i, 1 + i % 12, 1 + i),
                format!("\"play_count\":{i}"),
                format!("\"favourite\":{}", i % 2 == 0),
                format!("\"completed\":{}", i % 3 == 1),
                format!("\"alt_emulator\":{}", i % 4 == 2),
            ];
            if i % 3 != 0 {
                f.push(format!("\"last_played\":{}", common::unix(2026, 9, 24 - i as i64, 10, 0, 0)));
                f.push(format!("\"play_time\":{}", (37 + 61 * i) * 60));
            }
            if i % 4 != 3 {
                f.push(format!("\"rating\":{}", 0.2 + 0.2 * (i % 5) as f64 - 0.1));
            }
            let more = extra(i);
            if !more.is_empty() {
                f.push(more);
            }
            format!("{{{}}}", f.join(","))
        })
        .collect()
}

/// The nine shelves of the C# case library: five consoles, an automatic collection, a console whose sixth game is a
/// folder, a custom collection, and a console of pads with a folder link; the case's system and game come first.
fn library(system: usize, game: usize, hide: bool) -> (String, Vec<(String, String)>) {
    let none = |_: usize| String::new();
    let mut shelves: Vec<(String, String, String, &str, Vec<String>)> = [("nes", "Nintendo Entertainment System", ".nes"), ("snes", "Super Nintendo", ".sfc"), ("n64", "Nintendo 64", ".z64"), ("gb", "Game Boy", ".gb"), ("gbc", "Game Boy Color", ".gbc")]
        .iter()
        .map(|(n, f, e)| (n.to_string(), f.to_string(), n.to_string(), "regular", games(n, e, none)))
        .collect();
    shelves.push(("all".into(), "All Games".into(), "auto-allgames".into(), "auto", games("nes", ".nes", none)));
    shelves.push(("snes".into(), "Super Nintendo".into(), "snes".into(), "regular", games("snes", ".sfc", |i| if i == 5 { "\"folder\":true".into() } else { String::new() })));
    shelves.push(("mine".into(), "mine".into(), "mine".into(), "custom", games("snes", ".sfc", |i| if i % 2 == 0 { "\"in_collection\":true".into() } else { String::new() })));
    let pads = |i: usize| if i == 3 { "\"folder\":true,\"folder_link\":\"Linked Game.sfc\"".to_owned() } else { "\"controller\":\"gamepad_nintendo_snes\"".to_owned() };
    shelves.push(("pads".into(), "Pads".into(), "snes".into(), "regular", games("snes", ".sfc", pads)));
    if hide {
        let g = &mut shelves[system].4[game];
        g.insert_str(1, "\"hide_metadata\":true,");
    }
    shelves[system].4.rotate_left(game);
    shelves.rotate_left(system);
    let mut files = vec![];
    for (_, _, _, _, rows) in &shelves {
        for r in rows {
            let file = r.split("\"file\":\"").nth(1).and_then(|s| s.split('"').next()).unwrap().to_owned();
            let sys = r.split("\"esde_system\":\"").nth(1).and_then(|s| s.split('"').next()).unwrap().to_owned();
            files.push((sys, file));
        }
    }
    files.sort();
    files.dedup();
    let text: Vec<String> = shelves
        .iter()
        .map(|(n, f, t, k, rows)| format!("{{\"name\":\"{n}\",\"full_name\":\"{f}\",\"theme\":\"{t}\",\"kind\":\"{k}\",\"stars\":true,\"games\":[{}]}}", rows.join(",")))
        .collect();
    (format!("[{}]", text.join(",")), files)
}

fn view_of(c: &Case) -> &str {
    if let Some(v) = &c.view {
        return v;
    }
    let only = catalog().find(&c.kind).and_then(|e| e.find(&c.property)).map(|p| p.only_in);
    if matches!(only, Some(Scope::System)) {
        "system"
    } else {
        "gamelist"
    }
}

// The case's element with one of its values, in its view, as the C# `Element` writes it.
fn element(c: &Case, value: &str, assets: &Assets) -> String {
    let mut props: Vec<(String, String)> = vec![];
    let set = |props: &mut Vec<(String, String)>, key: &str, v: &str| {
        props.retain(|(k, _)| k != key);
        props.push((key.to_owned(), v.to_owned()));
    };
    for part in base(&c.kind).split(';') {
        let (k, v) = part.split_once('=').unwrap();
        set(&mut props, k, v);
    }
    props.retain(|(k, _)| !c.remove.contains(k));
    for part in c.context.split(';').filter(|p| p.contains('=')) {
        let (k, v) = part.split_once('=').unwrap();
        set(&mut props, k, v);
    }
    if value.starts_with('<') {
        props.push((format!("{}:case", c.property), value.to_owned()));
    } else {
        set(&mut props, &c.property, value);
    }
    let body: String = props.iter().map(|(k, v)| if v.starts_with('<') { v.clone() } else { format!("<{k}>{v}</{k}>") }).collect();
    let cover = if c.context.split(';').any(|p| p == "cover") {
        "<image name=\"cover\"><pos>0 0</pos><size>1 1</size><path>{B}</path><zIndex>35</zIndex></image>"
    } else {
        ""
    };
    assets.fill(&format!("<view name=\"{}\">{cover}<{} name=\"x\">{body}</{}>{}</view>", view_of(c), c.kind, c.kind, c.beside))
}

/// The theme's errors as the loader reports them for the case's system, before anything is drawn.
fn errors(theme: &Path) -> Vec<String> {
    let loaded = loader::load(&theme.to_string_lossy(), &ThemeSystem::new("snes", "Super Nintendo", "snes"), &loader::Choices::default(), None);
    loaded.diagnostics.iter().filter(|d| d.severity == Severity::Error).map(|d| d.to_string()).collect()
}

// A direction along the case's primary element, as the painter reads its axis.
fn along(c: &Case, value: &str) -> UiButton {
    let vertical = c.kind == "textlist" || (c.kind == "carousel" && (c.context.contains("type=vertical") || (c.property == "type" && value.starts_with("vertical"))));
    if vertical {
        UiButton::Down
    } else {
        UiButton::Right
    }
}

/// One rendering of a case: the view emptied so the next install starts afresh, the theme and the rotated library
/// installed, the case's move made, and the picture left on the window at the case's time.
fn render(scene: &mut Scene<'_>, c: &Case, value: &str, theme: &Path, assets: &Assets) {
    common::write_theme(theme, "", &element(c, value, assets));
    if scene.property("esdeModel", "themed") == "true" {
        assert!(scene.set_property("view", "shelves", "[]"));
        until(scene, "the view emptied", |s| s.property("esdeModel", "themed") == "false");
    }
    let (shelves, files) = library(c.system, c.game, c.hide);
    // A handheld's status, so systemstatus has indicators to draw.
    assert!(scene.set_property("view", "device", "{\"wifi\":true,\"battery\":80}"));
    assert!(scene.set_property("view", "media", &assets.media(&files)));
    let interface = format!("\"DisplayClock\":true,\"StartupView\":\"{}\"", view_of(c));
    assert!(scene.set_property("view", "now", common::HELD_NOW));
    assert!(scene.set_property("view", "settings", &settings(theme, &interface)));
    assert!(scene.set_property("view", "shelves", &shelves));
    until(scene, "the case's theme installed", |s| s.property("esdeModel", "themed") == "true");
    let started = std::time::Instant::now();
    match c.moves.as_deref() {
        Some("step") => scene.pad(along(c, value)),
        Some("down") => scene.pad(UiButton::Down),
        Some("hold") => scene.pad_down(along(c, value)),
        _ => {}
    }
    match c.at {
        Some(at) => {
            let left = at - started.elapsed().as_millis() as i32;
            scene.wait(left.max(0));
        }
        None => {
            until(scene, "the case at rest", |s| s.property("esdeModel", "moving") == "false");
            scene.wait(150);
        }
    }
}

#[test]
fn every_pair_the_mapping_marks_drawn_has_a_case_and_every_case_is_mapped() {
    let claimed: BTreeSet<(String, String)> =
        mapping::pairs().filter(|(_, _, s)| **s == Status::Drawn).map(|(k, p, _)| (k.to_owned(), p.to_owned())).collect();
    let proved: BTreeSet<(String, String)> = all_cases().into_iter().map(|c| (c.kind, c.property)).collect();
    let unproved: Vec<_> = claimed.difference(&proved).collect();
    let unmapped: Vec<_> = proved.difference(&claimed).filter(|(k, p)| !matches!(mapping::status(k, p), Some(Status::Later(_)))).collect();
    assert!(unproved.is_empty(), "drawn in mapping.tsv with no case: {unproved:?}");
    assert!(unmapped.is_empty(), "a case for a pair mapping.tsv marks neither drawn nor later: {unmapped:?}");
}

#[test]
fn every_case_s_two_themes_load_without_errors() {
    let dir = Scratch::new("mapping-load");
    let assets = Assets::new(&dir.0);
    let mut failures = vec![];
    for (n, c) in all_cases().iter().enumerate() {
        for (side, value) in [("a", &c.a), ("b", &c.b)] {
            let theme = dir.path(&format!("theme-{n}-{side}"));
            common::write_theme(&theme, "", &element(c, value, &assets));
            failures.extend(errors(&theme).into_iter().map(|e| format!("{c} = {value}: {e}")));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_drawn_pair_changes_the_painted_pixels() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let _kwin = common::start();
    let dir = Scratch::new("mapping");
    let assets = Assets::new(&dir.0);
    let only = std::env::var("ESDE_MAPPING_ONLY").ok();
    // A case kept for a pair marked later waits for its family; only the drawn pairs are held.
    let cases: Vec<Case> = all_cases()
        .into_iter()
        .filter(|c| mapping::status(&c.kind, &c.property) == Some(&Status::Drawn))
        .filter(|c| only.as_deref().is_none_or(|o| o.split(',').any(|w| w == c.to_string())))
        .collect();
    let mut harness = Harness::new();
    let mut scene = harness.load(common::VIEW, W, H);
    let mut failures = vec![];
    let picture = dir.path("a.png").to_string_lossy().into_owned();
    for (n, c) in cases.iter().enumerate() {
        let (ta, tb) = (dir.path(&format!("theme-{n}-a")), dir.path(&format!("theme-{n}-b")));
        render(&mut scene, c, &c.a, &ta, &assets);
        let problems: Vec<String> = errors(&ta).into_iter().chain(Some(scene.property("esdeModel", "problem")).filter(|p| !p.is_empty())).collect();
        assert!(scene.grab(&picture));
        if c.moves.as_deref() == Some("hold") {
            scene.pad_up(along(c, &c.a));
        }
        render(&mut scene, c, &c.b, &tb, &assets);
        let problems: Vec<String> = problems.into_iter().chain(errors(&tb)).chain(Some(scene.property("esdeModel", "problem")).filter(|p| !p.is_empty())).collect();
        let differing = scene.difference(&picture, 0, 0, &[], "");
        if c.moves.as_deref() == Some("hold") {
            scene.pad_up(along(c, &c.b));
        }
        if !problems.is_empty() {
            failures.push(format!("{c}: theme errors: {}", problems.join("; ")));
        } else if differing == Some(0) {
            failures.push(format!("{c}: {} and {} paint the same pixels", c.a, c.b));
        }
        let _ = std::fs::remove_dir_all(&ta);
        let _ = std::fs::remove_dir_all(&tb);
    }
    eprintln!("{} cases, {} failing\n{}", cases.len(), failures.len(), failures.join("\n"));
    scene.assert_quiet();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
