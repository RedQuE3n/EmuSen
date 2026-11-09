//! The oracle comparison (EmuSen_Qt.md §4.8.6): each watched theme's system view at the SNES and its gamelist at the
//! first game, at 1280 × 800 and 1920 × 1200, on the oracle's library, grabbed on the GPU tier and held to ES-DE 3.4.1's
//! captures through the C# baseline (`Testing/esde-baseline`): the mean absolute error, the help bar's strip left out,
//! no higher than the C# painter's in `mae.tsv`, and every element's box within two design units of the C# scene's. The
//! corpus measure of `esde-check` gains the watched columns and is written for `Testing/esde-corpus`; a row that
//! worsens against the last committed measure fails.

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
const STATES: [(&str, &str); 2] = [("system", "system"), ("gamelist", "gamelist-1")];

/// Boxes off the C#'s, measured on 2026-11-09 and named in EmuSen_Qt.md §4.8.6: (theme, state, the spec's key), at both
/// sizes. An entry no longer seen fails, so the list cannot outlive its findings.
const NAMED_BOXES: [(&str, &str, &str); 22] = [
    ("art-book-next-es-de", "system", "clock_clock"),
    ("art-book-next-es-de", "system", "systemstatus_system-status"),
    ("art-book-next-es-de", "system", "helpsystem_help-system-view"),
    ("art-book-next-es-de", "gamelist-1", "datetime_releasedate"),
    ("art-book-next-es-de", "gamelist-1", "text_players"),
    ("art-book-next-es-de", "gamelist-1", "text_play-time"),
    ("art-book-next-es-de", "gamelist-1", "systemstatus_system-status"),
    ("art-book-next-es-de", "gamelist-1", "helpsystem_help-gamelist-view"),
    ("artflix-revisited-es-de", "system", "systemstatus_(default)"),
    ("artflix-revisited-es-de", "system", "helpsystem_help"),
    ("artflix-revisited-es-de", "gamelist-1", "gamelistinfo_gamelistInfo"),
    ("artflix-revisited-es-de", "gamelist-1", "systemstatus_(default)"),
    ("artflix-revisited-es-de", "gamelist-1", "helpsystem_help"),
    ("caralt-es-de", "system", "image_cover1"),
    ("caralt-es-de", "system", "image_cover2"),
    ("caralt-es-de", "system", "image_cover3"),
    ("caralt-es-de", "system", "helpsystem_help"),
    ("caralt-es-de", "system", "systemstatus_menustatus"),
    ("caralt-es-de", "gamelist-1", "image_marquee"),
    ("caralt-es-de", "gamelist-1", "text_player2"),
    ("caralt-es-de", "gamelist-1", "helpsystem_help"),
    ("caralt-es-de", "gamelist-1", "systemstatus_menustatus"),
];

/// States whose MAE is above the C# painter's, measured on 2026-11-09 and named in EmuSen_Qt.md §4.8.6: (theme, state,
/// width, the measured figure as a ceiling). A state above its ceiling fails, and one back under the C#'s fails too.
const NAMED_MAE: [(&str, &str, i32, f64); 7] = [
    ("art-book-next-es-de", "system", 1280, 2.16),
    ("art-book-next-es-de", "system", 1920, 1.61),
    ("art-book-next-es-de", "gamelist-1", 1280, 3.82),
    ("art-book-next-es-de", "gamelist-1", 1920, 3.53),
    ("artflix-revisited-es-de", "system", 1280, 7.49),
    ("artflix-revisited-es-de", "system", 1920, 7.63),
    ("caralt-es-de", "system", 1280, 26.64),
];

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

/// The mean absolute error over R, G and B of the rows outside `strip`, and outside `masked` boxes when any are given.
fn mae(a: &[[u8; 3]], b: &[[u8; 3]], w: i32, strip: (i32, i32), masked: &[(i32, i32, i32, i32)]) -> f64 {
    let (mut total, mut n) = (0u64, 0u64);
    for (i, (p, q)) in a.iter().zip(b).enumerate() {
        let (x, y) = (i as i32 % w, i as i32 / w);
        if (strip.0..strip.1).contains(&y) || masked.iter().any(|m| x >= m.0 && x < m.0 + m.2 && y >= m.1 && y < m.1 + m.3) {
            continue;
        }
        total += (0..3).map(|c| u64::from(p[c].abs_diff(q[c]))).sum::<u64>();
        n += 3;
    }
    total as f64 / n.max(1) as f64
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

/// The drawn elements' boxes in design units (window pixels over `unit`), by name, in drawing order.
fn qt_boxes(nodes: &[Node], unit: f64) -> BTreeMap<String, Vec<[f64; 4]>> {
    let mut out: BTreeMap<String, Vec<[f64; 4]>> = BTreeMap::new();
    for n in nodes.iter().filter(|n| n.visible && n.name.starts_with("element-")) {
        out.entry(n.name["element-".len()..].to_owned()).or_default().push([n.x / unit, n.y / unit, n.w / unit, n.h / unit]);
    }
    out
}

/// Each element the C# drew, held to Qt's box of the same name and place within `mockup::UNITS`: the ones off (each a
/// line), and those Qt draws that the C# did not (noted, not held).
fn rects(spec: &[(String, String, String, [f64; 4])], qt: &BTreeMap<String, Vec<[f64; 4]>>) -> (Vec<(String, String)>, Vec<String>) {
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    let mut off = vec![];
    for (key, _, name, want) in spec {
        let k = seen.entry(name).or_default();
        match qt.get(name).and_then(|v| v.get(*k)) {
            None => off.push((key.clone(), "not drawn".to_owned())),
            Some(got) => {
                let d = (0..4).map(|i| (got[i] - want[i]).abs()).fold(0.0, f64::max);
                if d > mockup::UNITS {
                    let f = |b: &[f64; 4]| format!("{:.1},{:.1} {:.1}×{:.1}", b[0], b[1], b[2], b[3]);
                    off.push((key.clone(), format!("{} against the C#'s {}, {d:.1} off", f(got), f(want))));
                }
            }
        }
        *k += 1;
    }
    let noted = qt
        .iter()
        .flat_map(|(name, v)| (seen.get(name.as_str()).copied().unwrap_or(0)..v.len()).map(move |_| name.clone()))
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
        for c in ["unknown", "unsupported", "mae_system_1280", "mae_gamelist_1280", "mae_system_1920", "mae_gamelist_1920", "rects_off"] {
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
    let mut harness = Harness::new();
    let mut failures = vec![];
    let mut table = vec!["theme\tstate\tsize\tqt_mae\tcs_mae\tqt_mae_unclocked\tcs_mae_unclocked\trects_off".to_owned()];
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
                let capture = oracle.join(&theme).join(format!("{state}-{w}x{h}.png"));
                let differing = scene.difference(&capture.to_string_lossy(), 8, 1, &[], &out.join("difference").join(format!("{name}.png")).to_string_lossy());
                let tree = scene.tree();

                // The rectangles against the C# scene's.
                let system = emusen_esde::loader::ThemeSystem::new("snes", "Super Nintendo", "snes");
                let theme_view = emusen_esde::loader::load(&folder.to_string_lossy(), &system, &picked, None);
                let spec = spec_boxes(&repo().join(format!("Testing/esde-baseline/{name}.spec")), theme_view.view(view));
                let unit = f64::from(w) / 1280.0;
                let qt = qt_boxes(&tree.nodes, unit);
                let (off, noted) = rects(&spec, &qt);
                for (key, what) in &off {
                    match NAMED_BOXES.iter().position(|(t, s, k)| *t == theme && *s == state && k == key) {
                        Some(i) => boxes_seen[i] = true,
                        None => failures.push(format!("{at}: {key} {what}")),
                    }
                }
                off_count += off.len();
                if !noted.is_empty() {
                    eprintln!("{at}: drawn by Qt, not by the C#: {}", noted.join(", "));
                }

                // The picture against ES-DE's, the help bar's strip left out, beside the C# painter's on the same rows.
                let (top, bottom, cs_mae) = baseline[&(theme.clone(), state.to_owned(), w)];
                let (ew, eh, esde) = pixels(&capture).unwrap_or_else(|| panic!("ES-DE's capture {}: Testing/esde-baseline/capture.sh", capture.display()));
                let (qw, qh, qtp) = pixels(&qt_png).unwrap();
                assert_eq!((ew, eh, qw, qh), (w, h, w, h), "{at}: the capture and the grab at the state's size");
                let qt_mae = mae(&esde, &qtp, w, (top, bottom), &[]);
                // The clocks read the wall clock at their grabs, which no run repeats, so both painters are held without their boxes.
                let clocks: Vec<(i32, i32, i32, i32)> = spec
                    .iter()
                    .filter(|e| e.1 == "clock")
                    .map(|e| e.3)
                    .chain(theme_view.view(view).of_kind("clock").flat_map(|e| qt.get(&e.name).cloned().unwrap_or_default()))
                    .map(|b| ((b[0] * unit).floor() as i32, (b[1] * unit).floor() as i32, (b[2] * unit).ceil() as i32 + 1, (b[3] * unit).ceil() as i32 + 1))
                    .collect();
                let cs_png = common::compare_dir().join("../csharp").join(format!("{name}.png"));
                let cs_unclocked = pixels(&cs_png).map_or(cs_mae, |(_, _, cs)| mae(&esde, &cs, w, (top, bottom), &clocks));
                let qt_unclocked = mae(&esde, &qtp, w, (top, bottom), &clocks);
                let (qt_held, cs_held) = (qt_unclocked, cs_unclocked);
                match NAMED_MAE.iter().find(|(t, s, width, _)| *t == theme && *s == state && *width == w) {
                    Some((.., ceiling)) if qt_held > ceiling + 0.005 => failures.push(format!("{at}: MAE {qt_held:.2} above its named {ceiling:.2}")),
                    Some(_) if qt_held <= cs_held + 0.005 => failures.push(format!("{at}: MAE {qt_held:.2} back under the C#'s {cs_held:.2}: unname it")),
                    Some(_) => {}
                    None if qt_held > cs_held + 0.005 => failures.push(format!("{at}: MAE {qt_held:.2} above the C# painter's {cs_held:.2}")),
                    None => {}
                }
                table.push(format!("{theme}\t{state}\t{w}x{h}\t{qt_mae:.2}\t{cs_mae:.2}\t{qt_unclocked:.2}\t{cs_unclocked:.2}\t{}", off.len()));
                eprintln!("{}  ({} pixels differ by over 8 levels within a pixel)", table.last().unwrap(), differing.unwrap_or(0));
                let column = format!("mae_{view}_{w}");
                watched.entry(theme.clone()).or_default().insert(column, format!("{qt_held:.2}"));
            }
        }
        watched.entry(theme.clone()).or_default().insert("rects_off".to_owned(), off_count.to_string());
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
    let columns = ["mae_system_1280", "mae_gamelist_1280", "mae_system_1920", "mae_gamelist_1920", "rects_off"];
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
    let mut earlier: Vec<PathBuf> = std::fs::read_dir(&committed).unwrap().flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "tsv")).collect();
    earlier.sort();
    if let Some(last) = earlier.last() {
        for w in worsened(&std::fs::read_to_string(last).unwrap(), &measure) {
            failures.push(format!("against {}: {w}", last.file_name().unwrap().to_string_lossy()));
        }
    }
    if !report.all_load {
        failures.push("a theme of the corpus does not load".to_owned());
    }
    std::fs::write(out.join("corpus.tsv"), &measure).unwrap();
    if std::env::var_os("EMUSEN_ESDE_CORPUS").is_some() {
        std::fs::write(committed.join(&file), &measure).unwrap();
    }
    eprintln!("{}", measure.lines().last().unwrap_or_default());
    eprintln!("{} failures\n{}", failures.len(), failures.join("\n"));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_row_that_worsens_is_named() {
    let old = "theme\tloads\tunknown\tunsupported\tundrawn\tmae_system_1280\nA\tyes\t0\t5\t-\t1.00\nB\tyes\t0\t2\t-\t-\n";
    let new = "theme\tloads\tunknown\tunsupported\tundrawn\tmae_system_1280\nA\tno\t0\t6\tvideo\t1.20\nB\tyes\t0\t2\t-\t3.00\nC\tno\t1\t9\t-\t-\n";
    assert_eq!(worsened(old, new), ["A: no longer loads", "A: unsupported 5 → 6", "A: mae_system_1280 1 → 1.2", "A: video undrawn"]);
    assert!(worsened(new, new).is_empty());
}
