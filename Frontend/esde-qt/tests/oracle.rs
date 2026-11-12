//! The oracle comparison (EmuSen_Qt.md §4.8.6, §4.8.7): each watched theme's system view at the SNES and its gamelist at
//! the first game, at 1280 × 800 and 1920 × 1200, on the oracle's library, grabbed on the GPU tier and held to ES-DE
//! 3.4.1's captures: every element's box within two design units of where ES-DE draws it (the C# scene's spec, or the
//! box measured on the capture in `Testing/esde-baseline/esde-boxes.tsv` where the C#'s is not ES-DE's), and the mean
//! absolute error, the help bar's strip left out, no higher than the last committed measure. Each capture is used only
//! with the SHA-256 that `Testing/esde-oracle/captures.sha256` gives it. The corpus measure of `esde-check` gains the
//! watched columns and is written for `Testing/esde-corpus`; a row that worsens against the last committed measure
//! fails, a watched figure that falls fails until the lower one is committed, and only a run with no failure commits it,
//! or one whose only failures are rises in unknown or unsupported pairs with `EMUSEN_ESDE_CORPUS=ruled:<ruling id>`.

mod common;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use common::Scratch;
use cxx_qt_lib::{QDateTime, QImage};
use emusen_esde::check::{self, Table};
use emusen_qt_test::mockup::{self, Edge};
use emusen_qt_test::tree::Node;
use emusen_qt_test::{Harness, Scene};

const SIZES: [(i32, i32); 2] = [(1280, 800), (1920, 1200)];
/// The themes the media viewer was captured over (Testing/esde-oracle/README.md); the viewer is ES-DE's, not the theme's.
const VIEWER_THEMES: [&str; 2] = ["caralt-es-de", "artflix-revisited-es-de"];
const STATES: [(&str, &str); 2] = [("system", "system"), ("gamelist", "gamelist-1")];

/// Boxes off ES-DE's that the tester has named: (theme, state, the spec's key), at both sizes. Empty since 2026-11-09
/// (EmuSen_Qt.md §4.8.7); an entry no longer seen fails, so the list cannot outlive its findings.
const NAMED_BOXES: [(&str, &str, &str); 0] = [];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Every pixel of a picture as RGB, row by row, read through Qt's own decoder.
fn pixels(path: &Path) -> Option<(i32, i32, Vec<[u8; 3]>)> {
    let image = QImage::from_data(&std::fs::read(path).ok()?, None)?;
    let (w, h) = (image.width(), image.height());
    let mut out = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            let c = image.pixel_color(x, y);
            out.push([c.red() as u8, c.green() as u8, c.blue() as u8]);
        }
    }
    Some((w, h, out))
}

/// The mean absolute error over R, G and B of the rows outside `strip`.
fn mae(a: &[[u8; 3]], b: &[[u8; 3]], w: i32, strip: (i32, i32)) -> f64 {
    let (mut total, mut n) = (0u64, 0u64);
    for (i, (p, q)) in a.iter().zip(b).enumerate() {
        if (strip.0..strip.1).contains(&(i as i32 / w)) {
            continue;
        }
        total += (0..3).map(|c| u64::from(p[c].abs_diff(q[c]))).sum::<u64>();
        n += 3;
    }
    total as f64 / n.max(1) as f64
}

/// The bounds of the pixels between rows `rows` whose brightest channel stands over `over`: x0, y0, x1, y1.
fn ink(p: &[[u8; 3]], w: i32, rows: (i32, i32), over: u8) -> [i32; 4] {
    let mut b = [i32::MAX, i32::MAX, -1, -1];
    for y in rows.0..rows.1 {
        for x in 0..w {
            if p[(y * w + x) as usize].iter().copied().max().unwrap_or(0) > over {
                b = [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)];
            }
        }
    }
    b
}

/// ES-DE's boxes measured on its captures where the C# scene's is not ES-DE's: (theme, state, width, key) → box in design units.
fn esde_boxes() -> BTreeMap<(String, String, i32, String), [f64; 4]> {
    let text = std::fs::read_to_string(repo().join("Testing/esde-baseline/esde-boxes.tsv")).unwrap();
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .skip(1)
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            let n = |i: usize| c[i].parse::<f64>().unwrap_or_else(|e| panic!("esde-boxes.tsv: {l}: {e}"));
            ((c[0].to_owned(), c[1].to_owned(), c[2].parse().unwrap(), c[3].to_owned()), [n(4), n(5), n(6), n(7)])
        })
        .collect()
}

/// ES-DE's random picks in a capture: (theme, state, width) → (the element's spec key, the game it shows), in order.
fn captured_picks() -> BTreeMap<(String, String, i32), Vec<(String, String)>> {
    let text = std::fs::read_to_string(repo().join("Testing/esde-baseline/picks.tsv")).unwrap();
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .skip(1)
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            let picks = c[3].split(',').map(|p| p.split_once('=').map(|(k, g)| (k.to_owned(), g.to_owned())).unwrap()).collect();
            ((c[0].to_owned(), c[1].to_owned(), c[2].parse().unwrap()), picks)
        })
        .collect()
}

/// The first seed whose gameselector picks show ES-DE's captured games, read through the scene's own rule
/// (`selector::game_for`) on the systems the painter builds from the same texts, so the oracle's state is the capture's.
fn seed_for(settings: &str, texts: &(String, String, String), w: i32, h: i32, picks: &[(String, String)]) -> u64 {
    use emusen_esde_qt::{painter, shelves};
    let systems = painter::systems(
        &painter::ThemeCache::default(),
        &shelves::settings(settings).expect("the settings read"),
        shelves::shelves(&texts.0, &texts.1).expect("the shelves read"),
        &shelves::presence(&texts.2).expect("the presence reads"),
        w,
        h,
    );
    let system = &systems[0];
    let view = system.theme.view("system");
    (1..1_000_000)
        .find(|seed| {
            picks.iter().all(|(key, game)| {
                let (kind, name) = key.split_once('_').unwrap();
                view.find(kind, name)
                    .and_then(|e| emusen_esde::scene::selector::game_for(e, view, system, *seed))
                    .is_some_and(|i| system.any_game(i).is_some_and(|g| g.name == *game))
            })
        })
        .unwrap_or_else(|| panic!("no seed picks {picks:?}"))
}

/// The C# baseline's rows: (theme, state, width) → (strip top, strip bottom, MAE).
fn baseline() -> BTreeMap<(String, String, i32), (i32, i32, f64)> {
    let text = std::fs::read_to_string(repo().join("Testing/esde-baseline/mae.tsv")).unwrap();
    text.lines()
        .skip(1)
        .filter(|l| !l.is_empty())
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            ((c[0].to_owned(), c[1].to_owned(), c[2].parse().unwrap()), (c[4].parse().unwrap(), c[5].parse().unwrap(), c[6].parse().unwrap()))
        })
        .collect()
}

/// The captures in use and their SHA-256s, `sha256sum` lines relative to the oracle's folder (Testing/esde-oracle/README.md).
fn manifest() -> BTreeMap<String, String> {
    let text = std::fs::read_to_string(repo().join("Testing/esde-oracle/captures.sha256")).unwrap();
    text.lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let (sum, name) = l.split_once("  ").unwrap_or_else(|| panic!("captures.sha256: sum, two spaces, name: {l}"));
            (name.to_owned(), sum.to_owned())
        })
        .collect()
}

/// A file's SHA-256 in hex, by `sha256sum`, as the manifest names it.
fn sha256(path: &Path) -> Option<String> {
    let out = std::process::Command::new("sha256sum").arg(path).output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    out.status.success().then(|| text.split_whitespace().next().unwrap_or_default().to_owned())
}

/// The capture `name` under `oracle` when the manifest lists it with the file's own sum; refused otherwise.
fn captured(oracle: &Path, sums: &BTreeMap<String, String>, name: &str) -> Result<PathBuf, String> {
    let path = oracle.join(name);
    let listed = sums.get(name).ok_or_else(|| format!("{name}: not in Testing/esde-oracle/captures.sha256"))?;
    match sha256(&path) {
        Some(sum) if sum == *listed => Ok(path),
        Some(sum) => Err(format!("{name}: its SHA-256 {sum} is not the manifest's {listed}")),
        None => Err(format!("{}: not read", path.display())),
    }
}

/// How a run may write the corpus measure: `EMUSEN_ESDE_CORPUS` unset, `1`, or `ruled:<ruling id>` (EmuSen_Qt.md §4.4).
#[derive(Debug, PartialEq)]
enum Recording {
    Off,
    Plain,
    Ruled(String),
}

impl Recording {
    fn of(var: Option<&str>) -> Self {
        match var {
            None => Self::Off,
            Some(v) => v.strip_prefix("ruled:").map_or(Self::Plain, |id| Self::Ruled(id.trim().to_owned())),
        }
    }
}

/// Writes the corpus measure as the committed one: plainly when asked and nothing failed, under a ruling when its rises are
/// the only failures and the ruling is named, its id on the file's first line; whether it wrote.
fn record(committed: &Path, file: &str, measure: &str, rises: &[String], failures: &[String], how: &Recording) -> bool {
    let text = match how {
        Recording::Plain if rises.is_empty() && failures.is_empty() => measure.to_owned(),
        Recording::Ruled(id) if !id.is_empty() && failures.is_empty() => format!("# ruled {id}: {} rises\n{measure}", rises.len()),
        _ => return false,
    };
    std::fs::write(committed.join(file), text).unwrap();
    true
}

/// Whether a worsened line is a corpus rise: a count of unknown or unsupported pairs that grew.
fn is_rise(line: &str) -> bool {
    line.split(": ").nth(1).is_some_and(|c| c.starts_with("unknown ") || c.starts_with("unsupported "))
}

/// The spec's elements in drawing order: (key, kind, name, box in design units).
fn spec_boxes(path: &Path, view: &emusen_esde::loader::View) -> Vec<(String, String, String, [f64; 4])> {
    let m = mockup::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
    m.elements
        .iter()
        .map(|e| {
            let get = |want: Edge| e.edges.iter().find(|(k, _)| *k == want).map_or(0.0, |(_, v)| *v);
            let (kind, name) = e.name.split_once('_').unwrap();
            let turn = view.find(kind, name).map_or((0.0, (0.5, 0.5)), |t| {
                (f64::from(t.float("rotation").unwrap_or(0.0)), t.pair("rotationOrigin").map_or((0.5, 0.5), |o| (f64::from(o.x), f64::from(o.y))))
            });
            (e.name.clone(), kind.to_owned(), name.to_owned(), turned([get(Edge::X), get(Edge::Y), get(Edge::W), get(Edge::H)], turn.0, turn.1))
        })
        .collect()
}

/// A box turned by `degrees` about its `origin` (fractions of it), as the window holds it: the bounds of its corners. The
/// specs hold the C#'s box before its turn, and the item tree the turned item's bounds.
fn turned(b: [f64; 4], degrees: f64, origin: (f64, f64)) -> [f64; 4] {
    if degrees == 0.0 {
        return b;
    }
    let (ox, oy) = (b[0] + origin.0 * b[2], b[1] + origin.1 * b[3]);
    let (sin, cos) = degrees.to_radians().sin_cos();
    let corners = [(b[0], b[1]), (b[0] + b[2], b[1]), (b[0], b[1] + b[3]), (b[0] + b[2], b[1] + b[3])]
        .map(|(x, y)| (ox + (x - ox) * cos - (y - oy) * sin, oy + (x - ox) * sin + (y - oy) * cos));
    let xs = corners.map(|c| c.0);
    let ys = corners.map(|c| c.1);
    let (x0, y0) = (xs.iter().copied().fold(f64::MAX, f64::min), ys.iter().copied().fold(f64::MAX, f64::min));
    [x0, y0, xs.iter().copied().fold(f64::MIN, f64::max) - x0, ys.iter().copied().fold(f64::MIN, f64::max) - y0]
}

/// The delegate of View.qml that draws an element of this kind, so two kinds sharing a name (the loader's `(default)` clock
/// and status) are told apart.
fn delegate(kind: &str) -> &'static str {
    match kind {
        "image" | "video" | "animation" => "PictureElement",
        "text" | "datetime" | "clock" => "TextElement",
        "rating" => "RatingElement",
        "badges" => "BadgesElement",
        "helpsystem" => "HelpElement",
        "gamelistinfo" => "InfoElement",
        "systemstatus" => "StatusElement",
        "carousel" => "CarouselElement",
        "textlist" => "TextListView",
        "grid" => "GridElement",
        _ => "",
    }
}

/// The drawn elements' boxes in design units (window pixels over `unit`), by delegate and name, in drawing order.
fn qt_boxes(nodes: &[Node], unit: f64) -> BTreeMap<(String, String), Vec<[f64; 4]>> {
    let mut out: BTreeMap<(String, String), Vec<[f64; 4]>> = BTreeMap::new();
    for n in nodes.iter().filter(|n| n.visible && n.name.starts_with("element-")) {
        let class = n.cls.split('_').next().unwrap_or_default().to_owned();
        out.entry((class, n.name["element-".len()..].to_owned())).or_default().push([n.x / unit, n.y / unit, n.w / unit, n.h / unit]);
    }
    out
}

/// Each element of the spec, held to Qt's box of the same name and place within `mockup::UNITS`: the ones off (each a
/// line), and those Qt draws that the spec does not hold (noted, not held).
fn rects(spec: &[(String, String, String, [f64; 4])], qt: &BTreeMap<(String, String), Vec<[f64; 4]>>) -> (Vec<(String, String)>, Vec<String>) {
    let mut seen: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut off = vec![];
    for (key, kind, name, want) in spec {
        let at = (delegate(kind).to_owned(), name.clone());
        let k = seen.entry(at.clone()).or_default();
        match qt.get(&at).and_then(|v| v.get(*k)) {
            None => off.push((key.clone(), "not drawn".to_owned())),
            Some(got) => {
                let d = (0..4).map(|i| (got[i] - want[i]).abs()).fold(0.0, f64::max);
                if d > mockup::UNITS {
                    let f = |b: &[f64; 4]| format!("{:.1},{:.1} {:.1}×{:.1}", b[0], b[1], b[2], b[3]);
                    off.push((key.clone(), format!("{} against ES-DE's {}, {d:.1} off", f(got), f(want))));
                }
            }
        }
        *k += 1;
    }
    let noted = qt
        .iter()
        .flat_map(|(at, v)| (seen.get(at).copied().unwrap_or(0)..v.len()).map(move |_| format!("{} {}", at.0, at.1)))
        .collect();
    (off, noted)
}

/// Today's date as the corpus measure's file is named.
fn today() -> String {
    let d = QDateTime::current_date_time().date();
    format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day())
}

/// The rows of a corpus measure by theme, its columns by header name.
fn corpus_rows(text: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut lines = text.lines().filter(|l| !l.starts_with('#') && !l.is_empty());
    let header: Vec<&str> = lines.next().unwrap_or_default().split('\t').collect();
    lines
        .map(|l| {
            let row: BTreeMap<String, String> = header.iter().zip(l.split('\t')).map(|(h, v)| ((*h).to_owned(), v.to_owned())).collect();
            (row["theme"].clone(), row)
        })
        .collect()
}

/// Rows that worsened from `old` to `new`: a theme that stops loading, more unknown or unsupported pairs, an undrawn kind
/// gained, a watched MAE or count of boxes off that rose. A theme new to the corpus is not a worsening.
fn worsened(old: &str, new: &str) -> Vec<String> {
    let (old, new) = (corpus_rows(old), corpus_rows(new));
    let mut out = vec![];
    for (theme, now) in &new {
        let Some(was) = old.get(theme) else { continue };
        let num = |r: &BTreeMap<String, String>, c: &str| r.get(c).and_then(|v| v.parse::<f64>().ok());
        if was.get("loads").map(String::as_str) == Some("yes") && now["loads"] != "yes" {
            out.push(format!("{theme}: no longer loads"));
        }
        for c in ["unknown", "unsupported", "mae_system_1280", "mae_gamelist_1280", "mae_system_1920", "mae_gamelist_1920", "rects_off", "mae_viewer_1280", "mae_viewer_1920", "viewer_off"] {
            if let (Some(a), Some(b)) = (num(was, c), num(now, c)) {
                // MAE moves by up to 0.02 between runs (measured on 2026-11-09); a count by one.
                if b > a + if c.starts_with("mae") { 0.05 } else { 0.5 } {
                    out.push(format!("{theme}: {c} {a} → {b}"));
                }
            }
        }
        let kinds = |r: &BTreeMap<String, String>| r.get("undrawn").map(|v| v.split(',').filter(|k| *k != "-").map(str::to_owned).collect::<Vec<_>>()).unwrap_or_default();
        for k in kinds(now).into_iter().filter(|k| !kinds(was).contains(k)) {
            out.push(format!("{theme}: {k} undrawn"));
        }
    }
    out
}

#[test]
fn the_watched_states_hold_against_es_de_and_the_c_sharp_baseline() {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    assert!(common::oracle_library().join("gamelists").exists(), "the oracle's library: Testing/esde-oracle/library.py");
    let oracle = PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/emusen/esde-oracle");
    let out = common::compare_dir().join("../oracle");
    std::fs::create_dir_all(out.join("qt")).unwrap();
    std::fs::create_dir_all(out.join("difference")).unwrap();
    std::env::set_var("QT_ENABLE_HIGHDPI_SCALING", "0");
    let _kwin = common::start();
    let scratch = Scratch::new("oracle");
    let texts = common::oracle_texts(&common::ORDER);
    let baseline = baseline();
    let esde = esde_boxes();
    let picks = captured_picks();
    let sums = manifest();
    // ES-DE's default face, read from the oracle's AppImage's fonts by path as an installed ES-DE's would be (EmuSen_Qt.md §4.8.7).
    let face = oracle.join("bin/AppDir/usr/share/es-de/resources/fonts").join(emusen_esde::fonts::FILE);
    assert!(face.is_file(), "ES-DE's face at {}: Testing/esde-oracle/README.md", face.display());
    let mut harness = Harness::new();
    let mut failures = vec![];
    let mut table = vec!["theme\tstate\tsize\tqt_mae\tcs_mae\trects_off".to_owned()];
    let mut watched: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut boxes_seen = vec![false; NAMED_BOXES.len()];
    let (w0, h0) = SIZES[0];
    let mut scene: Scene<'_> = harness.load(common::VIEW, w0, h0);
    for (theme, variant, scheme, font) in common::watched() {
        let folder = common::corpus().join(&theme);
        assert!(folder.join("capabilities.xml").exists(), "{theme} in the corpus at {}", folder.display());
        let (choices, picked) = common::choices_of(&variant, &scheme, &font);
        let mut off_count = 0;
        for (view, state) in STATES {
            for (w, h) in SIZES {
                let at = format!("{theme} {state} {w}x{h}");
                let capture = captured(&oracle, &sums, &format!("{theme}/{state}-{w}x{h}.png")).unwrap_or_else(|e| panic!("{at}: {e}"));
                // The clock and the relative dates at the capture's time, as the C# baseline set them (Testing/esde-baseline/README.md).
                let taken = std::fs::metadata(&capture).and_then(|m| m.modified()).unwrap_or_else(|e| panic!("{}: {e}", capture.display()));
                let taken = taken.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
                assert!(scene.set_property("view", "now", &taken.to_string()));
                assert!(scene.set_property("view", "defaultFont", &face.to_string_lossy()));
                // The gameselectors' picks as ES-DE's random ones in this capture, as the clock is its time.
                let seed = picks.get(&(theme.clone(), state.to_owned(), w)).map_or(0, |p| seed_for(&common::install_settings(&folder, &choices, view), &texts, w, h, p));
                assert!(scene.set_property("view", "selectionSeed", &seed.to_string()));
                common::install(&mut scene, &folder, &choices, view, &texts, w, h);
                let problem = scene.property("esdeModel", "problem");
                if !problem.is_empty() {
                    failures.push(format!("{at}: {problem}"));
                }
                if view == "gamelist" {
                    // The first game: the folder Tower Set is entry 0, as ES-DE lists it.
                    scene.pad(common::along(&folder, &picked, view));
                }
                if !common::settle_picture(&mut scene, &scratch.0) {
                    failures.push(format!("{at}: the picture did not settle in 10 s"));
                }
                let name = format!("{theme}-{state}-{w}x{h}");
                let qt_png = out.join("qt").join(format!("{name}.png"));
                assert!(scene.grab(&qt_png.to_string_lossy()));
                let differing = scene.difference(&capture.to_string_lossy(), 8, 1, &[], &out.join("difference").join(format!("{name}.png")).to_string_lossy());
                let tree = scene.tree();

                // The rectangles against the C# scene's.
                let system = common::oracle_snes();
                let theme_view = emusen_esde::loader::load(&folder.to_string_lossy(), &system, &picked, None);
                let spec: Vec<_> = spec_boxes(&repo().join(format!("Testing/esde-baseline/{name}.spec")), theme_view.view(view))
                    .into_iter()
                    .map(|(key, kind, name, b)| {
                        let measured = esde.get(&(theme.clone(), state.to_owned(), w, key.clone())).copied();
                        (key, kind, name, measured.unwrap_or(b))
                    })
                    .collect();
                let unit = f64::from(w) / 1280.0;
                let qt = qt_boxes(&tree.nodes, unit);
                let (off, noted) = rects(&spec, &qt);
                for (key, what) in &off {
                    eprintln!("{at}: off: {key} {what}");
                    match NAMED_BOXES.iter().position(|(t, s, k)| *t == theme && *s == state && k == key) {
                        Some(i) => boxes_seen[i] = true,
                        None => failures.push(format!("{at}: {key} {what}")),
                    }
                }
                off_count += off.len();
                if !noted.is_empty() {
                    eprintln!("{at}: drawn by Qt, not by the C#: {}", noted.join(", "));
                }

                // The picture against ES-DE's, the help bar's strip left out, the C# painter's figure on the same rows beside it.
                let (top, bottom, cs_mae) = baseline[&(theme.clone(), state.to_owned(), w)];
                let (ew, eh, esde) = pixels(&capture).unwrap_or_else(|| panic!("ES-DE's capture {}: Testing/esde-baseline/capture.sh", capture.display()));
                let (qw, qh, qtp) = pixels(&qt_png).unwrap();
                assert_eq!((ew, eh, qw, qh), (w, h, w, h), "{at}: the capture and the grab at the state's size");
                // The ceiling is the last committed measure, held below with the corpus; the C#'s figure is printed beside it.
                let (qt_held, cs_held) = (mae(&esde, &qtp, w, (top, bottom)), cs_mae);
                table.push(format!("{theme}\t{state}\t{w}x{h}\t{qt_held:.2}\t{cs_held:.2}\t{}", off.len()));
                eprintln!("{}  ({} pixels differ by over 8 levels within a pixel)", table.last().unwrap(), differing.unwrap_or(0));
                let column = format!("mae_{view}_{w}");
                watched.entry(theme.clone()).or_default().insert(column, format!("{qt_held:.2}"));
            }
        }
        watched.entry(theme.clone()).or_default().insert("rects_off".to_owned(), off_count.to_string());
    }
    // ES-DE's media viewer on the NES's first game, every item in turn and one right past the last, and X on the SNES's folder (EmuSen_Qt.md §4.10).
    let viewer_texts = common::oracle_texts(&["nes"]);
    for (theme, variant, scheme, font) in common::watched().into_iter().filter(|w| VIEWER_THEMES.contains(&w.0.as_str())) {
        let folder = common::corpus().join(&theme);
        let (choices, _) = common::choices_of(&variant, &scheme, &font);
        let mut off_count = 0;
        for (w, h) in SIZES {
            let unit = f64::from(w) / 1280.0;
            for n in 0..=8 {
                let name = format!("nes-gamelist-0-media-{n}-{w}x{h}");
                let at = format!("{theme} {name}");
                let capture = captured(&oracle, &sums, &format!("{theme}/{name}.png")).unwrap_or_else(|e| panic!("{at}: {e}"));
                assert!(scene.set_property("view", "defaultFont", &face.to_string_lossy()));
                common::install(&mut scene, &folder, &choices, "gamelist", &viewer_texts, w, h);
                scene.pad(emusen_kit_core::input::UiButton::Screensaver);
                for _ in 0..n {
                    scene.pad(emusen_kit_core::input::UiButton::Right);
                }
                // The first item is the video, a still held four seconds and encoded with H.264's noise (Testing/esde-oracle/library.py).
                if !common::settle_within(&mut scene, &scratch.0, if n == 0 { 8 } else { 0 }) {
                    failures.push(format!("{at}: the picture did not settle in 10 s"));
                }
                let qt_png = out.join("qt").join(format!("{theme}-{name}.png"));
                assert!(scene.grab(&qt_png.to_string_lossy()));
                let (_, _, esde) = pixels(&capture).unwrap_or_else(|| panic!("ES-DE's capture {}: Testing/esde-oracle/README.md", capture.display()));
                let (_, _, qtp) = pixels(&qt_png).unwrap();
                let strip = emusen_esde::scene::Viewer::strip(f64::from(h)) as i32;
                for (what, a, b) in [
                    ("help", ink(&esde, w, (0, strip), 100), ink(&qtp, w, (0, strip), 100)),
                    ("item", ink(&esde, w, (strip + 1, h), 10), ink(&qtp, w, (strip + 1, h), 10)),
                ] {
                    let d = (0..4).map(|i| f64::from((a[i] - b[i]).abs()) / unit).fold(0.0, f64::max);
                    if d > mockup::UNITS {
                        off_count += 1;
                        failures.push(format!("{at}: the {what}'s ink {b:?} against ES-DE's {a:?}, {d:.1} off"));
                    }
                }
                let m = mae(&esde, &qtp, w, (0, 0));
                table.push(format!("{theme}\tviewer-{n}\t{w}x{h}\t{m:.2}\t-\t-"));
                eprintln!("{}", table.last().unwrap());
                let column = format!("mae_viewer_{w}");
                let row = watched.entry(theme.clone()).or_default();
                let sum = row.get(&column).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0) + m / 9.0;
                row.insert(column, format!("{sum:.2}"));
            }
            // X on the folder Tower Set, entry 0 of the SNES's gamelist, opens nothing.
            common::install(&mut scene, &folder, &choices, "gamelist", &texts, w, h);
            scene.pad(emusen_kit_core::input::UiButton::Screensaver);
            scene.wait(300);
            if scene.tree().nodes.iter().any(|n| n.visible && n.name == "element-strip") {
                failures.push(format!("{theme} {w}x{h}: X on the folder opened the media viewer"));
            }
        }
        watched.entry(theme.clone()).or_default().insert("viewer_off".to_owned(), off_count.to_string());
    }
    // The viewer's manual on the NES's third game (entry 1) and its settings on the first, in CarAlt (EmuSen_Qt.md §4.13, §4.14).
    {
        use emusen_kit_core::input::UiButton::{Down, PageDown, PageUp, Right, Screensaver, Up};
        let (theme, variant, scheme, font) = common::watched().into_iter().find(|w| w.0 == "caralt-es-de").expect("CarAlt is watched");
        let folder = common::corpus().join(&theme);
        let (choices, _) = common::choices_of(&variant, &scheme, &font);
        let manual: [(&str, &[emusen_kit_core::input::UiButton]); 14] = [
            ("media-0", &[]),
            ("media-1", &[Right]),
            ("media-2", &[Right, Right]),
            ("media-3", &[Right, Right, Right]),
            ("media-4", &[Right, Right, Right, Right]),
            ("manual-0", &[Up]),
            ("manual-1", &[Up, Right]),
            ("manual-2", &[Up, Right, Right]),
            ("manual-3", &[Up, Right, Right, Right]),
            ("manual-0-zoom-1", &[Up, PageDown]),
            ("manual-0-zoom-2", &[Up, PageDown, PageDown]),
            ("manual-0-zoom-2-pan-2", &[Up, PageDown, PageDown, Right, Right]),
            ("manual-0-zoom-2-pan-y-2", &[Up, PageDown, PageDown, Down, Down]),
            ("manual-0-zoom-2-zoom-out-1", &[Up, PageDown, PageDown, PageUp]),
        ];
        let settings: [(&str, usize, &str); 7] = [
            ("types0", 0, "\"MediaViewerShowTypes\":true"),
            ("types4", 4, "\"MediaViewerShowTypes\":true"),
            ("bottom", 0, "\"MediaViewerHelpPrompts\":\"bottom\""),
            ("nohelp", 4, "\"MediaViewerHelpPrompts\":\"disabled\""),
            ("stretch", 0, "\"MediaViewerStretchVideos\":true"),
            ("blur", 0, "\"MediaViewerVideoBlur\":true"),
            ("plain", 0, ""),
        ];
        let states = manual
            .iter()
            .map(|(n, presses)| (format!("nes-gamelist-1-{n}"), 1, presses.to_vec(), "", "top"))
            .chain(settings.iter().map(|(tag, n, more)| {
                let place = if more.contains("bottom") { "bottom" } else if more.contains("disabled") { "off" } else { "top" };
                (format!("nes-gamelist-0-media-{n}-{tag}"), 0, vec![Right; *n], *more, place)
            }));
        for (name0, entry, presses, more, place) in states.collect::<Vec<_>>() {
            // The game with no video was captured at 1280 x 800 alone, its order being the same at every size.
            let only_small = ["media-1", "media-2", "media-3", "media-4"].iter().any(|m| name0.ends_with(m));
            for (w, h) in SIZES.into_iter().filter(|s| !only_small || s.0 == 1280) {
                let unit = f64::from(w) / 1280.0;
                let name = format!("{name0}-{w}x{h}");
                let at = format!("{theme} {name}");
                let capture = captured(&oracle, &sums, &format!("{theme}/{name}.png")).unwrap_or_else(|e| panic!("{at}: {e}"));
                assert!(scene.set_property("view", "defaultFont", &face.to_string_lossy()));
                common::install_with(&mut scene, &folder, &choices, "gamelist", &viewer_texts, w, h, more);
                for _ in 0..entry {
                    scene.pad(Down);
                }
                scene.wait(300);
                scene.pad(Screensaver);
                for b in &presses {
                    scene.pad(*b);
                    scene.wait(100);
                }
                if !common::settle_within(&mut scene, &scratch.0, if name0.contains("media-0") { 8 } else { 2 }) {
                    failures.push(format!("{at}: the picture did not settle in 10 s"));
                }
                let qt_png = out.join("qt").join(format!("{theme}-{name}.png"));
                assert!(scene.grab(&qt_png.to_string_lossy()));
                let (_, _, esde) = pixels(&capture).unwrap_or_else(|| panic!("ES-DE's capture {}: Testing/esde-oracle/README.md", capture.display()));
                let (_, _, qtp) = pixels(&qt_png).unwrap();
                let strip = emusen_esde::scene::Viewer::strip(f64::from(h)) as i32;
                let (help_rows, item_rows) = match place {
                    "bottom" => ((h - strip, h), (0, h - strip - 1)),
                    "off" => ((0, 0), (0, h)),
                    _ => ((0, strip), (strip + 1, h)),
                };
                let mut parts = vec![("item", ink(&esde, w, item_rows, 10), ink(&qtp, w, item_rows, 10))];
                if place != "off" {
                    parts.push(("help", ink(&esde, w, help_rows, 100), ink(&qtp, w, help_rows, 100)));
                }
                for (what, a, b) in parts {
                    let d = (0..4).map(|i| f64::from((a[i] - b[i]).abs()) / unit).fold(0.0, f64::max);
                    if d > mockup::UNITS {
                        failures.push(format!("{at}: the {what}'s ink {b:?} against ES-DE's {a:?}, {d:.1} off"));
                    }
                }
                let m = mae(&esde, &qtp, w, (0, 0));
                table.push(format!("{theme}\t{name0}\t{w}x{h}\t{m:.2}\t-\t-"));
                eprintln!("{}", table.last().unwrap());
            }
        }
    }
    scene.assert_quiet();
    for (i, seen) in boxes_seen.iter().enumerate() {
        if !seen {
            failures.push(format!("named but no longer seen: {:?}", NAMED_BOXES[i]));
        }
    }
    std::fs::write(out.join("oracle.tsv"), table.join("\n") + "\n").unwrap();

    // The corpus measure with the watched columns, held against the last committed one (today's too, when a re-measure).
    let corpus = PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/emusen/bigpicture/corpus/xml");
    let report = check::report(&[corpus], &check::SYSTEMS, Table::Summary);
    let columns = ["mae_system_1280", "mae_gamelist_1280", "mae_system_1920", "mae_gamelist_1920", "rects_off", "mae_viewer_1280", "mae_viewer_1920", "viewer_off"];
    let measure: Vec<String> = report
        .text
        .lines()
        .enumerate()
        .map(|(i, l)| {
            if l.starts_with('#') {
                l.to_owned()
            } else if i == 0 {
                format!("{l}\t{}", columns.join("\t"))
            } else {
                let theme = l.split('\t').next().unwrap();
                let row = watched.get(theme);
                let cells: Vec<String> = columns.iter().map(|c| row.and_then(|r| r.get(*c)).cloned().unwrap_or_else(|| "-".to_owned())).collect();
                format!("{l}\t{}", cells.join("\t"))
            }
        })
        .collect();
    let measure = measure.join("\n") + "\n";
    let committed = repo().join("Testing/esde-corpus");
    let file = format!("{}.tsv", today());
    let how = Recording::of(std::env::var("EMUSEN_ESDE_CORPUS").ok().as_deref());
    let mut rises = vec![];
    let mut earlier: Vec<PathBuf> = std::fs::read_dir(&committed).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "tsv")).collect();
    earlier.sort();
    if let Some(last) = earlier.last() {
        let last_text = std::fs::read_to_string(last).unwrap();
        for w in worsened(&last_text, &measure) {
            let line = format!("against {}: {w}", last.file_name().unwrap().to_string_lossy());
            if is_rise(&w) { rises.push(line) } else { failures.push(line) }
        }
        // The ceiling falls with the figure: a lower one is committed by the run that measures it.
        if how == Recording::Off {
            for w in fell(&last_text, &measure) {
                failures.push(format!("against {}: {w}: commit the lower measure (EMUSEN_ESDE_CORPUS=1)", last.file_name().unwrap().to_string_lossy()));
            }
        }
    }
    if !report.all_load {
        failures.push("a theme of the corpus does not load".to_owned());
    }
    std::fs::write(out.join("corpus.tsv"), &measure).unwrap();
    // The committed measure is re-recorded only by a run that passed every check above, or failed only by rises it was ruled for.
    for r in &rises {
        eprintln!("rise: {r}");
    }
    if !record(&committed, &file, &measure, &rises, &failures, &how) {
        if let Recording::Ruled(id) = &how {
            failures.push(format!("EMUSEN_ESDE_CORPUS=ruled:{id}: nothing written ({} other failures{})", failures.len(), if id.is_empty() { ", no ruling named" } else { "" }));
        }
        failures.extend(rises);
    }
    eprintln!("{}", measure.lines().last().unwrap_or_default());
    eprintln!("{} failures\n{}", failures.len(), failures.join("\n"));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Watched figures that fell from `old` to `new` by more than a run's noise, so the committed ceiling would lag them.
fn fell(old: &str, new: &str) -> Vec<String> {
    let (old, new) = (corpus_rows(old), corpus_rows(new));
    let mut out = vec![];
    for (theme, now) in &new {
        let Some(was) = old.get(theme) else { continue };
        for c in ["mae_system_1280", "mae_gamelist_1280", "mae_system_1920", "mae_gamelist_1920", "mae_viewer_1280", "mae_viewer_1920"] {
            let num = |r: &BTreeMap<String, String>| r.get(c).and_then(|v| v.parse::<f64>().ok());
            if let (Some(a), Some(b)) = (num(was), num(now)) {
                if b < a - 0.05 {
                    out.push(format!("{theme}: {c} {a} → {b}"));
                }
            }
        }
    }
    out
}

#[test]
fn a_row_that_worsens_is_named() {
    let old = "theme\tloads\tunknown\tunsupported\tundrawn\tmae_system_1280\nA\tyes\t0\t5\t-\t1.00\nB\tyes\t0\t2\t-\t-\n";
    let new = "theme\tloads\tunknown\tunsupported\tundrawn\tmae_system_1280\nA\tno\t0\t6\tvideo\t1.20\nB\tyes\t0\t2\t-\t3.00\nC\tno\t1\t9\t-\t-\n";
    assert_eq!(worsened(old, new), ["A: no longer loads", "A: unsupported 5 → 6", "A: mae_system_1280 1 → 1.2", "A: video undrawn"]);
    assert!(worsened(new, new).is_empty());
    assert_eq!(fell(new, old), ["A: mae_system_1280 1.2 → 1"]);
    assert!(fell(old, new).is_empty());
}

#[test]
fn a_failing_run_leaves_the_committed_measure_alone() {
    let scratch = Scratch::new("oracle-record");
    let rise = ["against x.tsv: t: unsupported 8 → 10".to_owned()];
    assert!(!record(&scratch.0, "a.tsv", "m\n", &[], &["a box off".to_owned()], &Recording::Plain));
    assert!(!scratch.0.join("a.tsv").exists(), "a failing run re-recorded the baseline");
    assert!(!record(&scratch.0, "b.tsv", "m\n", &[], &[], &Recording::Off));
    assert!(!record(&scratch.0, "b.tsv", "m\n", &rise, &[], &Recording::Plain), "a plain run wrote over a rise");
    assert!(!scratch.0.join("b.tsv").exists());
    assert!(record(&scratch.0, "c.tsv", "m\n", &[], &[], &Recording::Plain));
    assert_eq!(std::fs::read_to_string(scratch.0.join("c.tsv")).unwrap(), "m\n");
}

#[test]
fn a_ruled_rise_is_recorded_only_alone_and_with_its_ruling() {
    let scratch = Scratch::new("oracle-ruled");
    let rise = ["against x.tsv: t: unsupported 8 → 10".to_owned()];
    let ruled = Recording::of(Some("ruled:R-20261111-87"));
    assert_eq!(ruled, Recording::Ruled("R-20261111-87".to_owned()));
    assert_eq!(Recording::of(Some("1")), Recording::Plain);
    assert_eq!(Recording::of(None), Recording::Off);
    assert!(record(&scratch.0, "rises.tsv", "m\n", &rise, &[], &ruled), "rises only: written");
    assert_eq!(std::fs::read_to_string(scratch.0.join("rises.tsv")).unwrap(), "# ruled R-20261111-87: 1 rises\nm\n");
    assert!(!record(&scratch.0, "boxes.tsv", "m\n", &rise, &["box: a box off".to_owned()], &ruled), "a rise and a box failure: not written");
    assert!(!record(&scratch.0, "noid.tsv", "m\n", &rise, &[], &Recording::of(Some("ruled:"))), "no id: not written");
    assert!(!scratch.0.join("boxes.tsv").exists() && !scratch.0.join("noid.tsv").exists());
    assert!(is_rise("t: unsupported 8 → 10") && is_rise("t: unknown 0 → 1"));
    assert!(!is_rise("t: rects_off 0 → 1") && !is_rise("t: no longer loads") && !is_rise("t: video undrawn"));
    let header = "theme\tloads\tunsupported\nt\tyes\t8\n";
    assert_eq!(corpus_rows(&format!("# ruled R-1: 0 rises\n{header}"))["t"]["unsupported"], "8", "the ruling's line is not read as the header");
}

#[test]
fn a_capture_is_used_only_with_the_manifest_s_sum() {
    let scratch = Scratch::new("oracle-manifest");
    std::fs::create_dir_all(scratch.0.join("t")).unwrap();
    std::fs::write(scratch.0.join("t/s.png"), b"capture").unwrap();
    let sum = sha256(&scratch.0.join("t/s.png")).unwrap();
    assert_eq!(sum, "460ee6aa3a80359181b794cc31a7185addba77626e9f719c10e3c8efb8668a1d");
    let listed = BTreeMap::from([("t/s.png".to_owned(), sum.clone())]);
    assert_eq!(captured(&scratch.0, &listed, "t/s.png"), Ok(scratch.0.join("t/s.png")));
    std::fs::write(scratch.0.join("t/s.png"), b"recaptured").unwrap();
    assert!(captured(&scratch.0, &listed, "t/s.png").unwrap_err().contains("is not the manifest's"));
    assert!(captured(&scratch.0, &listed, "t/other.png").unwrap_err().contains("not in"));
    let sums = manifest();
    assert!(sums.len() >= 86 && sums.values().all(|s| s.len() == 64), "every capture the oracle reads is listed by its SHA-256");
}
