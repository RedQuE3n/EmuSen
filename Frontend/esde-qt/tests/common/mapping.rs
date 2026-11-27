// Shared mapping-case loader, synthetic theme assets, and scene runner for CPU and GPU mapping checks.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use emusen_esde::loader::catalog::Scope;
use emusen_esde::loader::{self, catalog, Severity, ThemeSystem};
use emusen_kit_core::input::UiButton;
use emusen_qt_test::{clips, Scene};

use super::{settings, until, Scratch};

pub const W: i32 = 640;
pub const H: i32 = 400;
const CASES: &str = include_str!("../themes/mapping-cases.tsv");

// Minimum properties each element needs to draw; a case's own property replaces its entry here.
pub const BASE: [(&str, &str); 15] = [
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
pub struct Case {
    pub kind: String,
    pub property: String,
    pub a: String,
    pub b: String,
    pub view: Option<String>,
    pub system: usize,
    pub game: usize,
    pub remove: Vec<String>,
    pub context: String,
    pub at: Option<i32>,
    pub moves: Option<String>,
    pub hide: bool,
    pub beside: String,
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

pub fn listed() -> Vec<Case> {
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

// Properties every element shares, where its type declares them.
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

pub fn all_cases() -> Vec<Case> {
    let mut cases = listed();
    for (kind, _) in BASE {
        cases.extend(common_cases(kind));
    }
    cases
}

/// Pictures, icons, animations, and faces named by mapping cases, written once into the scratch folder.
pub struct Assets {
    pub dir: PathBuf,
}

impl Assets {
    pub fn new(dir: &Path) -> Assets {
        let p = |n: &str| dir.join(n);
        super::bmp(&p("a.bmp"), 60, 80, |x, y| [[[255, 140, 0], [0, 128, 128]], [[128, 0, 128], [255, 215, 0]]][(y >= 40) as usize][(x >= 30) as usize]);
        super::halves(&p("b.bmp"), 80, 50, [128, 0, 128], [255, 215, 0]);
        super::bmp(&p("fine.bmp"), 64, 64, |x, y| if (x + y) % 2 == 0 { [255, 255, 255] } else { [0, 0, 0] });
        super::halves(&p("screenshot.bmp"), 80, 70, [128, 0, 128], [255, 215, 0]);
        super::halves(&p("marquee.bmp"), 100, 30, [220, 20, 60], [128, 128, 0]);
        super::halves(&p("other.bmp"), 70, 70, [70, 130, 180], [250, 128, 114]);
        let four = [[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 0]];
        super::gif(&p("four.gif"), 8, 6, four, &[10, 20, 30, 40], |f, _, _| f as u8);
        let checker = [[255, 255, 255], [0, 128, 128], [0, 0, 0], [0, 0, 0]];
        super::gif(&p("checker.gif"), 4, 4, checker, &[10, 10], |f, x, y| ((x as usize + y as usize + f) % 2) as u8);
        let svg = |name: &str, body: &str| {
            let text = format!("<svg xmlns='http://www.w3.org/2000/svg' width='20' height='20' viewBox='0 0 20 20'>{body}</svg>");
            std::fs::write(p(name), text).unwrap();
        };
        // Clips: red, blue, one second of red, red/blue two-pixel stripes, and red/blue halves.
        let clip = |name: &str, w: usize, frames: u32, colour: fn(u32, usize, usize) -> [u8; 3]| clips::write_frames(&p(name), w, 48, frames, colour);
        clip("red.y4m", 64, 300, |_, _, _| clips::RED);
        clip("blue.y4m", 64, 300, |_, _, _| clips::BLUE);
        clip("short.y4m", 64, clips::RATE, |_, _, _| clips::RED);
        clip("stripes.y4m", 16, 300, |_, x, _| if x % 4 < 2 { clips::RED } else { clips::BLUE });
        clip("halves.y4m", 64, 300, |_, x, _| if x < 32 { clips::RED } else { clips::BLUE });
        svg("icon-1.svg", "<circle cx='10' cy='10' r='9' fill='#fff' fill-opacity='0.5'/>");
        svg("icon-2.svg", "<rect x='2' y='2' width='16' height='16' fill='#888'/>");
        svg("icon-3.svg", "<path d='M10 1L19 19H1Z' fill='#fff'/>");
        // Folder path matching ES-DE gameOverridePath lookup (<system>/<stem>).
        std::fs::create_dir_all(p("overrides/snes")).unwrap();
        svg("overrides/snes/Ember Circuit (Synthetic).svg", "<rect x='0' y='0' width='20' height='10' fill='#0f0'/>");
        Assets { dir: dir.to_owned() }
    }

    pub fn fill(&self, text: &str) -> String {
        let p = |n: &str| self.dir.join(n).display().to_string();
        let fonts = concat!(env!("CARGO_MANIFEST_DIR"), "/../kit-qt/fonts/");
        text.replace("{A}", &p("a.bmp"))
            .replace("{B}", &p("b.bmp"))
            .replace("{C}", &p("fine.bmp"))
            .replace("{O}", &p("overrides"))
            .replace("{X}", "/nonexistent/emusen/scene/missing.png")
            .replace("{G2}", &p("checker.gif"))
            .replace("{G}", &p("four.gif"))
            .replace("{S1}", &p("icon-1.svg"))
            .replace("{S2}", &p("icon-2.svg"))
            .replace("{S3}", &p("icon-3.svg"))
            .replace("{V2}", &p("blue.y4m"))
            .replace("{VS}", &p("short.y4m"))
            .replace("{VC}", &p("stripes.y4m"))
            .replace("{VH}", &p("halves.y4m"))
            .replace("{V}", &p("red.y4m"))
            .replace("{FT}", &format!("{fonts}BarlowCondensed-Light.ttf"))
            .replace("{FB}", &format!("{fonts}Barlow-Medium.ttf"))
    }

    /// `esdeMedia`: every game has every type mapped to a distinct picture.
    pub fn media(&self, files: &[(String, String)]) -> String {
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

// Synthetic twelve-game library per system; `extra` appends per-game JSON fields.
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
                f.push(format!("\"last_played\":{}", super::unix(2026, 9, 24 - i as i64, 10, 0, 0)));
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

/// Nine synthetic shelves matching the C# fixture layout, rotated so the case's system and game come first.
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

// Element XML for a case and value inside its view.
pub fn element(c: &Case, value: &str, assets: &Assets) -> String {
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

// Capabilities transitions fragment requested by a case's move kind.
pub fn capabilities(c: &Case) -> &'static str {
    match c.moves.as_deref() {
        Some("switch") => "<transitions name=\"slide\"><systemToGamelist>slide</systemToGamelist><gamelistToSystem>slide</gamelistToSystem></transitions>",
        Some("fade") => "<transitions name=\"fade\"><systemToSystem>fade</systemToSystem></transitions>",
        _ => "",
    }
}

/// Theme load errors reported before rendering.
pub fn errors(theme: &Path) -> Vec<String> {
    let loaded = loader::load(&theme.to_string_lossy(), &ThemeSystem::new("snes", "Super Nintendo", "snes"), &loader::Choices::default(), None);
    loaded.diagnostics.iter().filter(|d| d.severity == Severity::Error).map(|d| d.to_string()).collect()
}

// Navigation direction along the case's primary element axis.
pub fn along(c: &Case, value: &str) -> UiButton {
    let vertical = c.kind == "textlist" || (c.kind == "carousel" && (c.context.contains("type=vertical") || (c.property == "type" && value.starts_with("vertical"))));
    if vertical {
        UiButton::Down
    } else {
        UiButton::Right
    }
}

// Whether this case requires Vulkan RHI (video playback, Picture/Icon ShaderEffect, or ClipElement layer blur).
pub fn is_gpu_case(c: &Case) -> bool {
    matches!(c.kind.as_str(), "video" | "image" | "animation" | "rating" | "badges" | "carousel" | "grid")
        || c.context.split(';').any(|p| p == "cover")
        || c.property == "fadeAbovePrimary"
        || (c.kind == "systemstatus" && c.property == "customIcon")
        || (c.kind == "textlist" && matches!(c.property.as_str(), "selectorImagePath" | "selectorImageTile"))
}

// Whether this case reads at an explicit clock timestamp after a move.
pub fn held(c: &Case) -> bool {
    c.at.is_some() && c.moves.is_some()
}

fn since_press(scene: &mut Scene<'_>) -> f64 {
    scene.property("viewClock", "tickedMs").parse().unwrap_or(f64::NAN)
}

/// Empties the view, installs one case/value theme and library, performs its move, and settles at its target time.
pub fn render(scene: &mut Scene<'_>, c: &Case, value: &str, theme: &Path, assets: &Assets) {
    super::write_theme(theme, capabilities(c), &element(c, value, assets));
    if scene.property("esdeModel", "themed") == "true" {
        assert!(scene.set_property("view", "shelves", "[]"));
        until(scene, "the view emptied", |s| s.property("esdeModel", "themed") == "false");
    }
    let (shelves, files) = library(c.system, c.game, c.hide);
    // Handheld status so systemstatus elements have indicators to draw.
    assert!(scene.set_property("view", "device", "{\"wifi\":true,\"battery\":80}"));
    assert!(scene.set_property("view", "media", &assets.media(&files)));
    let interface = format!("\"DisplayClock\":true,\"StartupView\":\"{}\"", view_of(c));
    assert!(scene.set_property("view", "now", super::HELD_NOW));
    // Menu-dimmed cases leave the view visible beneath a paused menu card.
    assert!(scene.set_property("view", "menuOpen", if c.moves.as_deref() == Some("menu") { "true" } else { "false" }));
    assert!(scene.set_property("view", "settings", &settings(theme, &interface)));
    assert!(scene.set_property("view", "shelves", &shelves));
    until(scene, "the case's theme installed", |s| s.property("esdeModel", "themed") == "true");
    if held(c) {
        until(scene, "the case at rest before its move", |s| s.property("esdeModel", "moving") == "false");
        scene.hold_clock(Some(0));
    }
    let started = std::time::Instant::now();
    match c.moves.as_deref() {
        Some("step" | "fade") => scene.pad(along(c, value)),
        Some("down") => scene.pad(UiButton::Down),
        Some("hold") => scene.pad_down(along(c, value)),
        Some("switch") => scene.pad(if view_of(c) == "system" { UiButton::Accept } else { UiButton::Back }),
        _ => {}
    }
    match c.at {
        Some(at) if held(c) => {
            // View's clock lags the held clock by a few ms; advance the deficit if still moving.
            let near = i64::from(at) - 32;
            scene.hold_clock(Some(near));
            let moving = |s: &mut Scene<'_>| s.property("esdeModel", "moving") == "true";
            let lacking = if moving(scene) { (f64::from(at) - since_press(scene)).round() as i64 } else { 32 };
            scene.hold_clock(Some(near + lacking));
            let t = since_press(scene);
            assert!(!moving(scene) || (t - f64::from(at)).abs() < 0.5, "{c} = {value}: the scene read at {t} ms after the press, not {at}");
        }
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

/// Renders both values of every case on an already-loaded scene and asserts non-zero pixel difference and clean logs.
pub fn run_cases(scene: &mut Scene<'_>, label: &str, cases: &[Case]) {
    let dir = Scratch::new(label);
    let assets = Assets::new(&dir.0);
    let mut failures = vec![];
    let picture = dir.path("a.png").to_string_lossy().into_owned();
    for (n, c) in cases.iter().enumerate() {
        let (ta, tb) = (dir.path(&format!("theme-{n}-a")), dir.path(&format!("theme-{n}-b")));
        render(scene, c, &c.a, &ta, &assets);
        let problems: Vec<String> = errors(&ta).into_iter().chain(Some(scene.property("esdeModel", "problem")).filter(|p| !p.is_empty())).collect();
        assert!(scene.grab(&picture));
        if c.moves.as_deref() == Some("hold") {
            scene.pad_up(along(c, &c.a));
        }
        if held(c) {
            scene.hold_clock(None);
        }
        render(scene, c, &c.b, &tb, &assets);
        let problems: Vec<String> = problems.into_iter().chain(errors(&tb)).chain(Some(scene.property("esdeModel", "problem")).filter(|p| !p.is_empty())).collect();
        let differing = scene.difference(&picture, 0, 0, &[], "");
        if c.moves.as_deref() == Some("hold") {
            scene.pad_up(along(c, &c.b));
        }
        if held(c) {
            scene.hold_clock(None);
        }
        if !problems.is_empty() {
            failures.push(format!("{c}: theme errors: {}", problems.join("; ")));
        } else if differing == Some(0) {
            failures.push(format!("{c}: {} and {} paint the same pixels", c.a, c.b));
        }
        let _ = std::fs::remove_dir_all(&ta);
        let _ = std::fs::remove_dir_all(&tb);
    }
    eprintln!("{label}: {} cases, {} failing\n{}", cases.len(), failures.len(), failures.join("\n"));
    scene.assert_quiet();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
