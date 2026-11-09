// What the GPU-tier tests of the painter share: the compositor, a written theme, the library's texts, and reads of the item tree.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::Scene;

pub const VIEW: &str = concat!("file://", env!("CARGO_MANIFEST_DIR"), "/tests/qml/View.qml");

/// Vulkan on the RX 6800 in a compositor of the test's own, the validation layer on (EmuSen_Qt.md §8.1, tier 2).
pub fn start() -> Compositor {
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(900);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    std::env::set_var("QSG_RENDER_LOOP", "threaded");
    std::env::set_var("QSG_RHI_DEBUG_LAYER", "1");
    std::env::set_var("VK_LAYER_SYNCVAL_SHADER_ACCESSES_HEURISTIC", "1");
    std::env::set_var("QT_VK_PHYSICAL_DEVICE_INDEX", std::env::var("EMUSEN_GPU_INDEX").unwrap_or_else(|_| "0".into()));
    kwin
}

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// A scratch folder of its own under the temporary folder, removed on drop.
pub struct Scratch(pub PathBuf);

impl Scratch {
    pub fn new(what: &str) -> Scratch {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("esde-qt-{what}-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Scratch(dir)
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.0.join(relative)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// An ES-DE theme folder of the given capabilities and theme.xml body; nothing from a real theme.
pub fn write_theme(dir: &Path, capabilities: &str, body: &str) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("capabilities.xml"), format!("<themeCapabilities>{capabilities}</themeCapabilities>")).unwrap();
    std::fs::write(dir.join("theme.xml"), format!("<theme>{body}</theme>")).unwrap();
}

/// `esdeSettings` for a theme folder: the startup view and the interface's switches the test names.
pub fn settings(theme: &Path, interface: &str) -> String {
    let theme = theme.display().to_string().replace('\\', "\\\\").replace('"', "\\\"");
    format!("{{\"theme\":\"{theme}\",\"interface\":{{{interface}}}}}")
}

pub fn until(scene: &mut Scene<'_>, what: &str, ok: impl Fn(&mut Scene<'_>) -> bool) {
    let mut waited = 0;
    while !ok(scene) {
        assert!(waited < 10_000, "{what} within 10 s");
        scene.wait(10);
        waited += 10;
    }
}

/// The box of the first visible item of this objectName, in window pixels.
pub fn box_of(scene: &mut Scene<'_>, name: &str) -> Option<(f64, f64, f64, f64)> {
    scene.tree().nodes.iter().find(|n| n.name == name && n.visible).map(|n| (n.x, n.y, n.w, n.h))
}

/// The opacity the item of this objectName is drawn at, its parents' counted; 0 where none is drawn.
pub fn opacity_of(scene: &mut Scene<'_>, name: &str) -> f64 {
    scene.tree().nodes.iter().find(|n| n.name == name).map_or(0.0, |n| n.opacity)
}

/// A 24-bit BMP of `w` × `h` coloured by `pixel(x, y)`, a format Qt reads with no plugin.
pub fn bmp(path: &Path, w: u32, h: u32, pixel: impl Fn(u32, u32) -> [u8; 3]) {
    let stride = (w * 3).div_ceil(4) * 4;
    let size = 54 + stride * h;
    let mut out = Vec::with_capacity(size as usize);
    out.extend_from_slice(b"BM");
    for v in [size, 0, 54, 40] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&(h as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    for v in [0u32, stride * h, 2835, 2835, 0, 0] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    for y in (0..h).rev() {
        let start = out.len();
        for x in 0..w {
            let [r, g, b] = pixel(x, y);
            out.extend_from_slice(&[b, g, r]);
        }
        out.resize(start + stride as usize, 0);
    }
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, out).unwrap();
}

/// A picture of two flat halves, left and right, as the C# fixtures' `Halves`.
pub fn halves(path: &Path, w: u32, h: u32, left: [u8; 3], right: [u8; 3]) {
    bmp(path, w, h, |x, _| if x < w / 2 { left } else { right });
}

/// An animated GIF of four colours at most, one frame per delay (hundredths of a second), `pixel(frame, x, y)` an index
/// into `palette`. Each pixel is written as its own LZW code with a clear code after every two, so no table grows.
pub fn gif(path: &Path, w: u16, h: u16, palette: [[u8; 3]; 4], delays: &[u16], pixel: impl Fn(usize, u16, u16) -> u8) {
    let mut out = Vec::new();
    out.extend_from_slice(b"GIF89a");
    out.extend_from_slice(&w.to_le_bytes());
    out.extend_from_slice(&h.to_le_bytes());
    out.extend_from_slice(&[0x81, 0, 0]);
    for c in palette {
        out.extend_from_slice(&c);
    }
    out.extend_from_slice(&[0x21, 0xFF, 11]);
    out.extend_from_slice(b"NETSCAPE2.0");
    out.extend_from_slice(&[3, 1, 0, 0, 0]);
    for (f, delay) in delays.iter().enumerate() {
        out.extend_from_slice(&[0x21, 0xF9, 4, 0x04]);
        out.extend_from_slice(&delay.to_le_bytes());
        out.extend_from_slice(&[0, 0, 0x2C, 0, 0, 0, 0]);
        out.extend_from_slice(&w.to_le_bytes());
        out.extend_from_slice(&h.to_le_bytes());
        out.extend_from_slice(&[0, 2]);
        let (mut bits, mut nbits, mut data) = (0u32, 0u32, Vec::new());
        let mut put = |code: u32, data: &mut Vec<u8>| {
            bits |= code << nbits;
            nbits += 3;
            while nbits >= 8 {
                data.push(bits as u8);
                bits >>= 8;
                nbits -= 8;
            }
        };
        put(4, &mut data);
        let mut run = 0;
        for y in 0..h {
            for x in 0..w {
                put(u32::from(pixel(f, x, y) & 3), &mut data);
                run += 1;
                if run == 2 {
                    put(4, &mut data);
                    run = 0;
                }
            }
        }
        put(5, &mut data);
        if nbits > 0 {
            data.push(bits as u8);
        }
        for chunk in data.chunks(255) {
            out.push(chunk.len() as u8);
            out.extend_from_slice(chunk);
        }
        out.push(0);
    }
    out.push(0x3B);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, out).unwrap();
}

/// Seconds since 1970 of a civil date and time in UTC.
pub fn unix(year: i64, month: i64, day: i64, hour: i64, minute: i64, second: i64) -> i64 {
    let (y, m) = if month <= 2 { (year - 1, month + 9) } else { (year, month - 3) };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + (153 * m + 2) / 5 + day - 1;
    (era * 146_097 + doe - 719_468) * 86_400 + hour * 3600 + minute * 60 + second
}

/// The oracle's synthetic library (Testing/esde-oracle/library.py), the one ES-DE and the C# painter were given.
pub fn oracle_library() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/emusen/esde-oracle/library")
}

fn tag<'a>(block: &'a str, name: &str) -> Option<&'a str> {
    let open = format!("<{name}>");
    let start = block.find(&open)? + open.len();
    Some(&block[start..start + block[start..].find(&format!("</{name}>"))?])
}

fn unescape(text: &str) -> String {
    text.replace("&apos;", "'").replace("&quot;", "\"").replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")
}

fn json_text(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

const ORACLE_TYPES: [(&str, &str); 5] =
    [("covers", "cover"), ("screenshots", "screenshot"), ("titlescreens", "titlescreen"), ("marquees", "marquee"), ("miximages", "miximage")];

/// The oracle library as `esdeShelves`, `esdeMedia` and `esdePresence` in ES-DE's order (`order`, the systems as ES-DE
/// shows them): folders on top, then favourites, then the rest, each by name, as its settings have it (FoldersOnTop,
/// FavoritesFirst). Read from the gamelists library.py writes, whose form is fixed.
pub fn oracle_texts(order: &[&str]) -> (String, String, String) {
    let root = oracle_library();
    let (mut shelves, mut media, mut presence) = (vec![], vec![], vec![]);
    for system in order {
        let full = &full_name(system);
        let xml = std::fs::read_to_string(root.join("gamelists").join(system).join("gamelist.xml")).unwrap();
        let mut rows: Vec<(u8, String, String)> = vec![];
        // ES-DE counts every game of the system, those inside folders too (its captures: 14 games, 7 favourites).
        let every: Vec<&str> = xml.split("<game>").skip(1).filter_map(|b| b.split("</game>").next()).collect();
        let favourites = every.iter().filter(|b| tag(b, "favorite") == Some("true")).count();
        for (open, close, folder) in [("<folder>", "</folder>", true), ("<game>", "</game>", false)] {
            for block in xml.split(open).skip(1).filter_map(|b| b.split(close).next()) {
                let path = unescape(tag(block, "path").unwrap().trim_start_matches("./"));
                if path.contains('/') {
                    continue;
                }
                let name = unescape(tag(block, "name").unwrap_or(&path));
                let mut f = vec![
                    format!("\"file\":{}", json_text(&path)),
                    format!("\"name\":{}", json_text(&name)),
                    format!("\"esde_system\":\"{system}\""),
                    format!("\"folder\":{folder}"),
                ];
                for (from, to) in [("desc", "description"), ("developer", "developer"), ("publisher", "publisher"), ("genre", "genre"), ("players", "players")] {
                    if let Some(v) = tag(block, from) {
                        f.push(format!("\"{to}\":{}", json_text(&unescape(v))));
                    }
                }
                let date = |v: &str| (v[0..4].parse().unwrap(), v[4..6].parse().unwrap(), v[6..8].parse().unwrap(), v[9..11].parse().unwrap(), v[11..13].parse().unwrap(), v[13..15].parse().unwrap());
                if let Some(v) = tag(block, "releasedate") {
                    f.push(format!("\"release_date\":\"{}-{}-{}\"", &v[0..4], &v[4..6], &v[6..8]));
                }
                if let Some(v) = tag(block, "lastplayed") {
                    let (y, mo, d, h, mi, s) = date(v);
                    f.push(format!("\"last_played\":{}", unix(y, mo, d, h, mi, s)));
                }
                if let Some(v) = tag(block, "rating") {
                    f.push(format!("\"rating\":{v}"));
                }
                if let Some(v) = tag(block, "playcount") {
                    f.push(format!("\"play_count\":{v}"));
                }
                if let Some(v) = tag(block, "folderlink") {
                    f.push(format!("\"folder_link\":{}", json_text(&unescape(v).replace("./", ""))));
                }
                let flag = |t: &str| tag(block, t) == Some("true");
                for (from, to) in [("favorite", "favourite"), ("completed", "completed"), ("hidemetadata", "hide_metadata")] {
                    f.push(format!("\"{to}\":{}", flag(from)));
                }
                let rank = if folder { 0 } else if flag("favorite") { 1 } else { 2 };
                rows.push((rank, name.to_lowercase(), format!("{{{}}}", f.join(","))));
            }
        }
        rows.sort();
        let games: Vec<String> = rows.into_iter().map(|(_, _, r)| r).collect();
        shelves.push(format!(
            "{{\"name\":\"{system}\",\"full_name\":{},\"theme\":\"{system}\",\"kind\":\"regular\",\"stars\":true,\"favourites_on_top\":true,\"counts\":{{\"games\":{n},\"favorites\":{favourites},\"shown\":{n}}},\"games\":[{}]}}",
            json_text(full),
            games.join(","),
            n = every.len()
        ));
        let (mut kinds, mut present) = (vec![], vec![]);
        for (folder, kind) in ORACLE_TYPES {
            let dir = root.join("downloaded_media").join(system).join(folder);
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            // Files only: a folder's own media sit in a subfolder of the same name, which is not a picture.
            let mut files: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.is_file()).collect();
            files.sort();
            let named: Vec<String> = files
                .iter()
                .map(|p| format!("{}:{}", json_text(&p.file_stem().unwrap().to_string_lossy()), json_text(&p.to_string_lossy())))
                .collect();
            if !named.is_empty() {
                present.push(format!("\"{kind}\""));
            }
            kinds.push(format!("\"{kind}\":{{{}}}", named.join(",")));
        }
        media.push(format!("\"{system}\":{{{}}}", kinds.join(",")));
        presence.push(format!("\"{system}\":[{}]", present.join(",")));
    }
    (format!("[{}]", shelves.join(",")), format!("{{{}}}", media.join(",")), format!("{{{}}}", presence.join(",")))
}

/// The watched themes with the options the oracle and the C# baseline gave them (Testing/esde-baseline/states.tsv).
pub fn watched() -> Vec<(String, String, Option<String>, String)> {
    let states = include_str!("../../../../Testing/esde-baseline/states.tsv");
    states
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            (c[0].to_owned(), c[1].to_owned(), (c[2] != "-").then(|| c[2].to_owned()), c[3].to_owned())
        })
        .collect()
}

/// The theme corpus the oracle reads its themes from, outside the repository.
pub fn corpus() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/emusen/bigpicture/corpus/whole")
}

/// The folder the side-by-sides are written to for a human look.
pub fn compare_dir() -> PathBuf {
    let dir = PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/emusen/tmp/qt-s4/compare/qt");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Waits until two grabs 300 ms apart are the same picture (pictures decode on a thread of their own); false after 10 s.
pub fn settle_picture(scene: &mut Scene<'_>, scratch: &Path) -> bool {
    let shot = scratch.join("settle.png").to_string_lossy().into_owned();
    for _ in 0..33 {
        until(scene, "the view at rest", |s| s.property("esdeModel", "moving") == "false");
        assert!(scene.grab(&shot));
        scene.wait(300);
        if scene.difference(&shot, 0, 0, &[], "") == Some(0) {
            return true;
        }
    }
    false
}

// The systems in the order ES-DE 3.4.1 shows them on the oracle's library (its capture of Art Book Next's carousel, 2026-11-08), the SNES first.
pub const ORDER: [&str; 5] = ["snes", "n64", "nes", "gb", "gbc"];

/// A system's name as an ES-DE theme reads it (`${system.fullName}`), from its owner, the systems table's data file (EmuSen_Qt.md §4.6).
pub fn full_name(system: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../Platform/systems/data/{system}.json"));
    let row: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    row["esde_full_name"].as_str().unwrap_or_else(|| panic!("{system}: the oracle's systems are all named by ES-DE")).to_owned()
}

/// The oracle's SNES as the loader is given it.
pub fn oracle_snes() -> emusen_esde::loader::ThemeSystem {
    emusen_esde::loader::ThemeSystem::new("snes", &full_name("snes"), "snes")
}

/// A step along a view's primary element, as the painter reads its axis: down a list or a vertical carousel, else right.
pub fn along(theme: &Path, choices: &emusen_esde::loader::Choices, view: &str) -> emusen_kit_core::input::UiButton {
    use emusen_esde::loader;
    let t = loader::load(&theme.to_string_lossy(), &oracle_snes(), choices, None);
    let vertical = match t.view(view).primary() {
        Some(p) if p.kind() == "textlist" => true,
        Some(p) if p.kind() == "carousel" => p.string("type").is_some_and(|t| t.starts_with("vertical")),
        _ => false,
    };
    if vertical {
        emusen_kit_core::input::UiButton::Down
    } else {
        emusen_kit_core::input::UiButton::Right
    }
}

/// One watched theme's options as the painter's `choices` text and as the loader's `Choices`.
pub fn choices_of(variant: &str, scheme: &Option<String>, font: &str) -> (String, emusen_esde::loader::Choices) {
    let mut text = format!("\"Variant\":\"{variant}\",\"FontSize\":\"{font}\"");
    if let Some(s) = scheme {
        text += &format!(",\"ColorScheme\":\"{s}\"");
    }
    let picked = emusen_esde::loader::Choices {
        variant: Some(variant.to_owned()),
        color_scheme: scheme.clone(),
        font_size: Some(font.to_owned()),
        ..emusen_esde::loader::Choices::default()
    };
    (text, picked)
}

/// The painter's settings for a watched theme's view: the theme, its choices, the clock shown and the view to start in.
pub fn install_settings(folder: &Path, choices: &str, view: &str) -> String {
    settings(folder, &format!("\"DisplayClock\":true,\"StartupView\":\"{view}\"")).replacen('{', &format!("{{\"choices\":{{{choices}}},"), 1)
}

/// A watched theme's view installed afresh at `w` × `h` on the oracle's texts, the clock shown; waits until it is themed.
/// The instant a test holds the clock at (EsdeModel's `now`, Unix seconds): 2026-11-06 12:00 UTC, so a minute turning over
/// mid-test changes no clock's words.
pub const HELD_NOW: &str = "1793966400";

pub fn install(scene: &mut Scene<'_>, folder: &Path, choices: &str, view: &str, texts: &(String, String, String), w: i32, h: i32) {
    let text = install_settings(folder, choices, view);
    assert!(scene.set_property("view", "shelves", "[]"));
    until(scene, "the view emptied", |s| s.property("esdeModel", "themed") == "false");
    scene.resize(w, h);
    assert!(scene.set_property("view", "media", &texts.1));
    assert!(scene.set_property("view", "presence", &texts.2));
    assert!(scene.set_property("view", "settings", &text));
    assert!(scene.set_property("view", "shelves", &texts.0));
    until(scene, "the theme loaded", |s| s.property("esdeModel", "themed") == "true");
}
