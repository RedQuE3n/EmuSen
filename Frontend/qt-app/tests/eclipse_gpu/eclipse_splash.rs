//! Eclipse's splash on the real program, on the card, held to its board and entering Home, a GPU job (EmuSen_Qt.md §3.5.3).

// The modules the scene imports register through the program's crate, which a test must link.

use crate::eclipse_board;
#[allow(unused_imports)]
use crate::eclipse_home_oracle;
use crate::start_held;

use cxx_qt_lib::QString;
use eclipse_board::{object, read};
use emusen_client_fixture::{library, World};
use emusen_crystal_client::Config;
use emusen_kit_core::input::UiButton;
use emusen_kit_core::tokens::Colour;
use emusen_qt_models::clients::ClientSource;
use emusen_qt_models::library::{installed_cores, LibrarySource};
use emusen_qt_models::words;
use emusen_qt_test::compositor::Compositor;
use emusen_qt_test::tree::{Node, Tree};
use emusen_qt_test::Scene;
use start_held::{started, until, Start, PLAIN};
use std::path::PathBuf;

const TABLE: &str = include_str!("../eclipse_splash.tsv");
const SOUND: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes/eclipse/sounds/splash.flac");

/// The board's height in pixels, every Eclipse board's (themes/eclipse/parts/Board.qml).
const BOARD_HEIGHT: f64 = 1200.0;
/// A box's edge within this many board pixels of Chrome's, whose transformed boxes are 32-bit floats (a part in 1e7 of 2000 px).
const PLACE: f64 = 0.01;
/// An opacity within half a level of 255, as the Home oracle holds an alpha (eclipse_board.rs, `painted`).
const OPACITY: f64 = 0.5 / 255.0;
/// The board's planets in front of the disc: its cores 0 to 4 (EclipseSplashPlay:162-167).
const FRONT: usize = 5;

/// One row of the board's table.
struct Board {
    moment: f64,
    name: String,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    opacity: f64,
}

fn board() -> Vec<Board> {
    TABLE
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("moment\t") && !l.trim().is_empty())
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            let n = |i: usize| c[i].parse::<f64>().unwrap();
            Board { moment: n(0), name: c[1].to_owned(), x: n(2), y: n(3), w: n(4), h: n(5), opacity: n(6) }
        })
        .collect()
}

/// The frames of the theme's sound file: STREAMINFO's 36-bit total samples, after "fLaC" and the block's 4-byte header (RFC 9639, 8.2).
fn sound_frames() -> i64 {
    let bytes = std::fs::read(SOUND).unwrap();
    assert_eq!(&bytes[..4], b"fLaC", "the theme's sound is FLAC");
    let info = &bytes[8..];
    (i64::from(info[13] & 0x0F) << 32) | i64::from(u32::from_be_bytes(info[14..18].try_into().unwrap()))
}

/// The theme's sound's rate: STREAMINFO's 20-bit sample rate (RFC 9639, 8.2).
fn sound_rate() -> i64 {
    let bytes = std::fs::read(SOUND).unwrap();
    let info = &bytes[8..];
    (i64::from(info[10]) << 12) | (i64::from(info[11]) << 4) | i64::from(info[12] >> 4)
}

/// The splash's time once the hand-over has ended: its length and the ease.
fn ended(scene: &mut Scene<'_>) -> f64 {
    scene.property("themeSplash", "length").parse::<f64>().unwrap() + scene.property("themeSplash", "ease").parse::<f64>().unwrap()
}

/// Home as it is once whole: no hand-over disc, the art label, the streaks and the focus ring at their own opacity.
fn whole(tree: &Tree) -> bool {
    let full = |name: &str| node(tree, name).is_none_or(|n| !n.visible || (n.opacity - 1.0).abs() < 1e-9);
    node(tree, "handoverDisc").is_none_or(|n| !n.visible) && full("artLabel") && full("tileRing")
}

/// CSS's ease-out, cubic-bezier(0, 0, 0.58, 1), at a progress: the curve's parameter found by bisection, its y the eased value (CSS Easing 1 §3.2).
fn ease_out(progress: f64) -> f64 {
    let x = progress.clamp(0.0, 1.0);
    let curve = |s: f64, p2: f64| 3.0 * (1.0 - s) * s * s * p2 + s * s * s;
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..64 {
        let mid = (lo + hi) / 2.0;
        if curve(mid, 0.58) < x { lo = mid } else { hi = mid }
    }
    curve((lo + hi) / 2.0, 1.0)
}

/// A node's descendant of a name, the last of it.
fn under<'a>(tree: &'a Tree, root: &Node, name: &str) -> Option<&'a Node> {
    let index: std::collections::HashMap<&str, &Node> = tree.nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    tree.nodes.iter().rev().filter(|n| n.name == name).find(|n| {
        let mut at = index.get(n.parent.as_str()).copied();
        while let Some(p) = at {
            if p.id == root.id {
                return true;
            }
            at = index.get(p.parent.as_str()).copied();
        }
        false
    })
}

/// What the hand-over moves, in scene pixels and drawn opacity: the shown disc's box, rim and three glows, the art label, the two streaks and the focus ring.
fn measures(tree: &Tree) -> Vec<(String, f64, bool)> {
    let mut out = Vec::new();
    let disc = ["splashCorona", "handoverDisc", "disc"].iter().find_map(|n| tree.nodes.iter().rev().find(|d| d.name == *n && d.visible)).expect("a disc shown");
    let body = under(tree, disc, "coronaDisc").expect("the disc's body");
    let rim = under(tree, disc, "coronaRim").expect("the disc's rim");
    out.extend([("disc x", body.x, true), ("disc y", body.y, true), ("disc w", body.w, true), ("disc h", body.h, true), ("rim", (rim.w - body.w) / 2.0, true), ("rim opacity", rim.opacity, false)].map(|(k, v, size)| (k.to_owned(), v, size)));
    for i in 0..3 {
        let glow = under(tree, disc, &format!("coronaGlow.{i}"));
        for property in ["blur", "spread"] {
            let v = glow.map_or(0.0, |g| {
                let id: u64 = g.id.parse().unwrap();
                emusen_qt_test::ffi::ffi::harness_property(object(id), &QString::from(g.name.as_str()), &QString::from(property)).to_string().parse::<f64>().unwrap() * g.scale
            });
            out.push((format!("glow {i} {property}"), v, true));
        }
    }
    let shown = |n: Option<&Node>| n.filter(|n| n.visible).map_or(0.0, |n| n.opacity);
    let backdrop = node(tree, "backdrop").expect("Home's backdrop");
    out.push(("art label".to_owned(), shown(node(tree, "artLabel")), false));
    out.push(("streak 3".to_owned(), shown(under(tree, backdrop, "backdropField.3")), false));
    out.push(("streak 4".to_owned(), shown(under(tree, backdrop, "backdropField.4")), false));
    out.push(("focus ring".to_owned(), tree.nodes.iter().filter(|n| n.name == "tileRing" && n.visible).map(|n| n.opacity).fold(0.0, f64::max), false));
    out
}

/// The hand-over frame by frame, 1/60 s apart from 9.5 s to the sound's end: no measure moves between two frames by more than its whole change times the ease-out's own step there, none before Home whole (R-20261123-9).
fn handover(scene: &mut Scene<'_>, length: f64, ease: f64, unit: f64, differences: &mut Vec<String>) {
    const RATE: i64 = 60;
    let (first, last) = ((9.5 * RATE as f64) as i64, ((length + ease) * RATE as f64 / 1000.0).ceil() as i64 + 1);
    let mut frames: Vec<(f64, Vec<(String, f64, bool)>)> = Vec::new();
    for f in first..=last {
        scene.hold_clock(Some(f * 1000 / RATE));
        let time: f64 = scene.property("themeSplash", "time").parse().unwrap();
        let tree = scene.tree();
        if time > length && time < length + ease && frames.last().is_some_and(|(t, _)| *t <= length) {
            assert_eq!(read(&tree, "tile.0", "activeFocus"), "true", "Home has the focus as the hand-over begins, at {time} ms");
        }
        frames.push((time, measures(&tree)));
    }
    let eased = |t: f64| if t <= length { 0.0 } else { ease_out((t - length) / ease) };
    let before = frames.iter().rev().find(|(t, _)| *t < length).expect("a frame of the splash").1.clone();
    let after = frames.last().unwrap().1.clone();
    assert!(frames.last().unwrap().0 >= length + ease, "the frames reach the hand-over's end");
    for pair in frames.windows(2) {
        let ((t0, m0), (t1, m1)) = (&pair[0], &pair[1]);
        let step = eased(*t1) - eased(*t0);
        for (i, (name, v1, size)) in m1.iter().enumerate() {
            let (whole_change, v0) = ((after[i].1 - before[i].1).abs(), m0[i].1);
            let slack = if *size { PLACE * unit } else { OPACITY };
            if (v1 - v0).abs() > whole_change * step + slack {
                differences.push(format!("hand-over: {name} moves {:.4} from {t0} to {t1} ms, where the ease allows {:.4}", (v1 - v0).abs(), whole_change * step));
            }
        }
    }
}

/// The last shown node of a name, else the last of it.
fn node<'a>(tree: &'a Tree, name: &str) -> Option<&'a Node> {
    tree.nodes.iter().rev().find(|n| n.name == name && n.visible).or_else(|| tree.nodes.iter().rev().find(|n| n.name == name))
}

/// The program's element for a row of the board's table, by the board's core order for a planet.
fn program_name(row: &str) -> Option<String> {
    let planet = |prefix: &str, base: usize| row.strip_prefix(prefix).and_then(|k| k.parse::<usize>().ok()).map(|k| format!("splashPlanet.{}", base + k));
    match row {
        "home" => Some("splashGround".into()),
        "stars" => Some("splashStars".into()),
        "orbit" => Some("splashOrbit".into()),
        "sun" => Some("splashSun".into()),
        "corona" => Some("splashCorona".into()),
        "moon" => Some("splashMoon".into()),
        "flare" => Some("splashFlare".into()),
        "wordmark" => Some("splashWordmark".into()),
        "rule" => Some("splashRule".into()),
        "counts" => Some("splashCounts".into()),
        "wordmarkBase" => Some("splashWordmark".into()),
        "countsBase" => Some("splashCounts".into()),
        "header" => Some("arrivalHeader".into()),
        "main" => Some("arrivalMain".into()),
        "recent" => Some("arrivalRecent".into()),
        "footer" => Some("help".into()),
        _ => row.strip_prefix("star.").map(|k| format!("splashStar.{k}")).or_else(|| planet("front.", 0)).or_else(|| planet("back.", FRONT)),
    }
}

/// The board's elements at `moment` against the program's, each difference pushed; `cores` planets are drawn.
fn held_at(scene: &mut Scene<'_>, rows: &[Board], moment: f64, cores: usize, at: &str, differences: &mut Vec<String>) {
    let tree = scene.tree();
    let height: f64 = scene.property("program", "height").parse().unwrap();
    // A board pixel in window pixels: the canvas's unit times the board's measure, the window's height over the board's.
    let s = height / BOARD_HEIGHT;
    // The window's middle across, in board pixels, which the board's centred column stands on.
    let centre = scene.property("program", "width").parse::<f64>().unwrap() / s / 2.0;
    for row in rows.iter().filter(|r| r.moment == moment) {
        let Some(name) = program_name(&row.name) else { continue };
        if let Some(k) = name.strip_prefix("splashPlanet.").and_then(|k| k.parse::<usize>().ok()) {
            if k >= cores {
                continue;
            }
        }
        let Some(n) = node(&tree, &name) else {
            differences.push(format!("{at} {moment}: {name} is not drawn"));
            continue;
        };
        let opacity = if row.name == "home" { 1.0 - n.opacity } else { n.opacity };
        let opacity = if n.visible || row.name == "home" { opacity } else { 0.0 };
        if (opacity - row.opacity).abs() > OPACITY {
            differences.push(format!("{at} {moment}: {name} opacity {opacity:.6} where the board's {} is {}", row.name, row.opacity));
        }
        if row.opacity <= OPACITY || ["home", "stars", "footer"].contains(&row.name.as_str()) {
            continue;
        }
        let (x, y, w, h) = (n.x / s, n.y / s, n.w / s, n.h / s);
        let off = |a: f64, b: f64| (a - b).abs() > PLACE;
        match row.name.as_str() {
            // Home's parts arriving: each moved as the board moves it, the header 48 and Recent 790 down, the words 96 in.
            "header" | "recent" => {
                let rest = if row.name == "header" { 48.0 } else { 790.0 };
                if off(y, row.y - rest) {
                    differences.push(format!("{at} {moment}: {name} is {y} down where the board's {} is {}", row.name, row.y - rest));
                }
            }
            "main" => {
                if off(x, row.x - 96.0) {
                    differences.push(format!("{at} {moment}: {name} is {x} across where the board's main is {}", row.x - 96.0));
                }
            }
            // A Line's box is the board's line box, its words centred across the window as the board's column centres them.
            "wordmark" | "counts" => {
                let line = tree.nodes.iter().find(|m| m.name == name && m.visible).unwrap();
                let (ly, lh) = (line.y / s, line.h / s);
                if off(x + w / 2.0, centre) || off(ly, row.y) || off(lh, row.h) {
                    differences.push(format!("{at} {moment}: {name}'s line box {ly:.3} down, {lh:.3} high, its words centred at {} where the board's is {} down, {} high, at {centre}", x + w / 2.0, row.y, row.h));
                }
            }
            // The words' baseline: the Text's top and its baselineOffset in its own units, where the board's CSS puts it.
            "wordmarkBase" | "countsBase" => {
                let base = y + read(&tree, &name, "baselineOffset").parse::<f64>().unwrap() * n.scale / s;
                if off(base, row.y) {
                    differences.push(format!("{at} {moment}: {name}'s baseline {base:.3} down where the board's is {}", row.y));
                }
            }
            // Centred across the window as the board's column is across its page (left: 0; right: 0).
            "rule" => {
                if off(x + w / 2.0, centre) || off(y, row.y) || off(w, row.w) || off(h, row.h) {
                    differences.push(format!("{at} {moment}: {name} at ({x:.3}, {y:.3}) {w:.3} by {h:.3} where the board's centres it at {centre}, {} down, {} by {}", row.y, row.w, row.h));
                }
            }
            _ => {
                if off(x, row.x) || off(y, row.y) || off(w, row.w) || off(h, row.h) {
                    differences.push(format!("{at} {moment}: {name} at ({x:.3}, {y:.3}) {w:.3} by {h:.3} where the board's {} is ({}, {}) {} by {}", row.name, row.x, row.y, row.w, row.h));
                }
            }
        }
    }
}

/// The board's colours, read from what draws them (EclipseSplashPlay:31, 41, 51, 54, 55, 69, 71, 160).
fn colours(scene: &mut Scene<'_>, cores: usize, differences: &mut Vec<String>) {
    let tree = scene.tree();
    let colour = |name: &str, property: &str| Colour::parse(&read(&tree, name, property));
    for (name, property, want) in [
        ("splashGround", "color", "#05060B"),
        ("splashStar.0", "color", "#E9ECF7"),
        ("splashMoon", "color", "#05060B"),
        ("splashCorona", "rimColor", "#F6E7D8"),
        ("splashWordmark", "color", "#F4F1EC"),
        ("splashCounts", "color", "#C9CDDB"),
    ] {
        if colour(name, property) != Colour::parse(want) {
            differences.push(format!("{name}'s {property} is {:?} where the board's is {want}", read(&tree, name, property)));
        }
    }
    let inks = [["#F4F1EC", "#C9CCD8", "#5E6272"], ["#FFEBB5", "#E8B34B", "#7A5416"], ["#FFC2A8", "#D4553F", "#6B1E16"], ["#D6E6FF", "#7FA6D9", "#2C4670"], ["#C8F0DA", "#4FA87A", "#1C4A36"], ["#D9F5C4", "#6FAE5A", "#274A1E"], ["#FFFFFF", "#BFE3F5", "#4A7890"], ["#CFE0FF", "#3F6FE0", "#162C6B"]];
    for i in 0..cores {
        let name = format!("splashPlanet.{i}");
        for (property, want) in ["light", "middle", "shade"].iter().zip(inks[i % inks.len()]) {
            if colour(&name, property) != Colour::parse(want) {
                differences.push(format!("{name}'s {property} is {:?} where the board's is {want}", read(&tree, &name, property)));
            }
        }
    }
}

#[test]
fn qt_3_4_8_eclipse_s_splash_is_its_board_and_enters_home() {
    // A compositor larger than the window, so its decoration leaves the window 1920 by 1200 whole.
    let kwin = Compositor::start(3840, 2160);
    kwin.watch(600);
    std::env::set_var("QT_QUICK_BACKEND", "rhi");
    std::env::set_var("QSG_RHI_BACKEND", "vulkan");
    // A held clock is refused under a threaded render loop, which installs its own animation driver (EmuSen_Kit.md §7.36).
    std::env::set_var("QSG_RENDER_LOOP", "basic");
    std::env::set_var("LC_ALL", "en_US.UTF-8");
    std::env::set_var("TZ", "UTC");
    // The sound into a file, never the desk's speakers: SDL's disk driver (SDL 3, SDL_AUDIO_DISK_OUTPUT_FILE).
    let world = World::new("qt-app-eclipse-splash");
    let heard = world.root.join("splash-heard.raw");
    std::env::set_var("SDL_AUDIO_DRIVER", "disk");
    std::env::set_var("SDL_AUDIO_DISK_OUTPUT_FILE", &heard);
    let mut harness = emusen_qt_test::Harness::new();
    let (_, media) = eclipse_home_oracle::synthetic(&world, &mut harness);
    // The installed cores as the program's runtime knows them, read as its library reads them.
    let source = ClientSource::open(library().to_str().unwrap(), &Config::default()).unwrap();
    let cores = installed_cores(&source.systems().unwrap()).len();
    drop(source);
    assert!(cores > 0, "the runtime knows cores to draw");
    let rows = board();
    // The splash's moments (splash_times_original.json) and 1.0 and 7.0 s, the moon crossing and the disc opening, where no keyframe stands.
    let moments = [1.0, 2.42, 3.3, 5.72, 7.0, 8.36, 9.68];
    let mut differences = Vec::new();

    // A press skips to Home whole, the sound let go softly, and the press is the splash's alone: first, so the device's file holds no other sound.
    {
        let mut scene = started(&mut harness, (1920, 1200), &PLAIN, &media);
        scene.hold_clock(Some(3000));
        // The sound heard a while first, the picture held: the second before the fade is the sound's own.
        scene.wait(2000);
        let shown = scene.property("screens", "shown");
        scene.pad(UiButton::Accept);
        assert_eq!(ended(&mut scene), scene.property("themeSplash", "time").parse::<f64>().unwrap(), "skipped past the hand-over");
        assert_eq!(scene.property("splashSound", "ends"), "1", "the sound let go");
        until(&mut scene, "the splash gone after the skip", 3000, |s| s.property("themeSplash", "done") == "true");
        scene.wait(300);
        assert_eq!(scene.property("screens", "shown"), shown, "the press that skipped is not Home's");
        assert_eq!(scene.property("library", "lastPlayed"), eclipse_home_oracle::rom(&world, eclipse_home_oracle::GAMES[0].0, eclipse_home_oracle::GAMES[0].1), "no game started");
        let tree = scene.tree();
        assert!(node(&tree, "disc").is_some_and(|n| n.visible), "Home whole");
        assert!(whole(&tree), "Home whole at once after a skip, no hand-over");
        // What the device was handed: sound from the start, then a fall to silence within the fade, no step.
        scene.hold_clock(None);
        scene.wait(500);
        let samples = heard_samples(&heard);
        // The device's file holds interleaved stereo at the interface player's rate: a second's samples, and the Fade token's.
        let second = (emusen_platform_client::sounds::RATE * emusen_platform_client::sounds::CHANNELS) as usize;
        let fading = *scene.kit_tokens().expect("the program's tokens").fade() as usize * second / 1000;
        let loud = samples.iter().position(|s| s.abs() > 1e-4).expect("the sound was heard");
        let last = samples.iter().rposition(|s| s.abs() > 1e-4).expect("and it ended");
        assert!(last > loud, "it played");
        // In the fade no step past twice the sound's own largest the second before; a click is some twenty times it (seen 2026-11-17).
        let steps = |from: usize, to: usize| samples[from..to].windows(3).step_by(2).map(|w| (w[2] - w[0]).abs()).fold(0.0f32, f32::max);
        let fade = (last - last % 2).saturating_sub(fading);
        let own = steps(fade.saturating_sub(second).max(loud - loud % 2), fade);
        // The fade's span and the step from its last frame to the silence after it.
        let end = steps(fade, (last - last % 2 + 4).min(samples.len()));
        assert!(end <= 2.0 * own, "the end falls with no click: its largest step {end}, the sound's own {own}");
        // Nothing heard past the fade: at least twice the fade's length of the device's file after it, the sound let go and not ringing on.
        assert!(samples.len() - last > 2 * fading, "the sound rings on past its fade: {} samples, the last heard at {last}", samples.len());
    }

    // The board's own size and the gate's smaller one of the same 16:10 shape, the pair the Home oracle holds.
    for size in [(1920, 1200), (1280, 800)] {
        let at = format!("{}x{}", size.0, size.1);
        let mut scene = started(&mut harness, size, &PLAIN, &media);
        assert_eq!(scene.property("themeSplash", "set"), "true", "{at}: the start read and the sound ready");
        assert_eq!(scene.property("theme", "leads"), "true", "{at}: Eclipse's splash leads the start");
        // The sound: started once, from the theme's file, whole.
        let said = ["frames", "settled", "problem", "on", "platformLibrary", "folder", "source"].map(|p| format!("{p}={}", scene.property("splashSound", p)));
        let file = ["source", "read", "problem", "rate"].map(|p| format!("{p}={}", scene.property("splashSoundFile", p)));
        assert_eq!(scene.property("splashSound", "starts"), "1", "{at}: {said:?} {file:?}");
        assert!(scene.property("splashSoundFile", "source").ends_with("/eclipse/sounds/splash.flac"), "{at}: the theme's own file");
        assert_eq!(scene.property("splashSound", "frames").parse::<i64>().unwrap(), sound_frames(), "{at}: the file whole");
        assert_eq!(scene.property("splashSound", "heldVolume"), scene.property("settingsModel", "navigationVolume"), "{at}: at the Sound volume");
        // The words: the library's six games and the cores.
        assert_eq!(read(&scene.tree(), "splashCounts", "text"), words::splash_counts(6, cores), "{at}");
        for moment in moments {
            // The clock stepped a frame at a time to the moment, then set to it exactly, the board's to the millisecond.
            // Home whole is the splash's end, where it hands over to Home: that moment is held a millisecond before it.
            let at_ms = if moment * 1000.0 >= scene.property("themeSplash", "length").parse::<f64>().unwrap() { moment * 1000.0 - 1.0 } else { moment * 1000.0 };
            scene.hold_clock(Some(at_ms as i64 - 17));
            assert_eq!(scene.property("moonrise", "visible"), "false", "{at}: Moonrise is never seen");
            assert!(scene.set_property("themeSplash", "clock", &format!("{at_ms}")));
            scene.wait(100);
            assert_eq!(scene.property("disc", "visible"), "false", "{at} {moment}: Home's own disc waits for the splash to go");
            held_at(&mut scene, &rows, moment, cores, &at, &mut differences);
        }
        colours(&mut scene, cores, &mut differences);
        // The hand-over runs from Home whole to the sound's last sample (R-20261123-9, R-20261123-24).
        let length: f64 = scene.property("themeSplash", "length").parse().unwrap();
        let ease: f64 = scene.property("themeSplash", "ease").parse().unwrap();
        let (frames, rate) = (sound_frames() as f64, sound_rate() as f64);
        assert!((length + ease - frames * 1000.0 / rate).abs() < 1e-6, "{at}: the hand-over ends with the sound, {length} + {ease} ms");
        if size == (1920, 1200) {
            handover(&mut scene, length, ease, f64::from(size.1) / BOARD_HEIGHT, &mut differences);
        }
        // On to its end: the splash and the hand-over gone, Home whole and its own disc drawn.
        let end = ended(&mut scene);
        scene.hold_clock(Some(end as i64 + 17));
        until(&mut scene, "the splash gone", 3000, |s| s.property("themeSplash", "done") == "true");
        scene.wait(200);
        let tree = scene.tree();
        assert!(node(&tree, "splashScreen").is_none_or(|n| !n.visible), "{at}: the splash is gone");
        assert!(node(&tree, "disc").is_some_and(|n| n.visible), "{at}: Home's disc stands in the splash's");
        assert!(whole(&tree), "{at}: Home whole after the hand-over");
        for part in ["arrivalHeader", "arrivalMain", "arrivalRecent", "help"] {
            assert!(node(&tree, part).is_some_and(|n| n.visible && (n.opacity - 1.0).abs() < 1e-9), "{at}: {part} whole");
        }
        let focus = scene.focus(&tree).map(|i| tree.nodes[i as usize].name.clone()).unwrap_or_default();
        let dbg = [("chosenTheme", "focus"), ("chosenTheme", "activeFocus"), ("theme", "takesFocus"), ("home.theme", "focus"), ("home.theme", "activeFocus"), ("themeSplash", "focus"), ("moonrise", "focus"), ("home", "activeFocus")].map(|(o, p)| format!("{o}.{p}={}", scene.property(o, p)));
        assert_eq!(read(&tree, "tile.0", "activeFocus"), "true", "{at}: Home holds the focus once the splash has gone: {focus} {dbg:?}");
        assert_eq!(scene.property("splashSound", "starts"), "1", "{at}: started once");
        assert_eq!(scene.property("splashSound", "ends"), "0", "{at}: the sound rings on into Home");
        assert_eq!(scene.property("program", "splashShown"), "false");
        scene.hold_clock(None);
    }

    // The splash off: Home at once, whole, and no sound.
    {
        let mut scene = started(&mut harness, (1920, 1200), &Start { splash: false, ..PLAIN }, &media);
        scene.wait(500);
        assert_eq!(scene.property("themeSplash", "visible"), "false");
        assert_eq!(scene.property("moonrise", "visible"), "false");
        assert_eq!(scene.property("splashSound", "starts"), "0", "no sound with the splash off");
        let tree = scene.tree();
        assert!(node(&tree, "disc").is_some_and(|n| n.visible), "Home at once");
        assert!(node(&tree, "arrivalMain").is_some_and(|n| (n.opacity - 1.0).abs() < 1e-9), "Home whole at once");
        scene.hold_clock(None);
    }

    // Reduced motion: the last frame from the first, held for the splash's length, the sound played.
    {
        let mut scene = started(&mut harness, (1920, 1200), &Start { reduced: true, ..PLAIN }, &media);
        assert_eq!(scene.property("themeSplash", "time"), scene.property("themeSplash", "length"), "the last frame from the start");
        assert_eq!(scene.property("splashSound", "starts"), "1", "the sound still played");
        scene.hold_clock(Some(4000));
        assert_eq!(scene.property("themeSplash", "done"), "false", "held for the splash's length");
        scene.hold_clock(Some(10_200));
        until(&mut scene, "Home after the held frame", 3000, |s| s.property("themeSplash", "done") == "true");
        // Still, Home is whole the moment the splash has gone: no hand-over is drawn.
        assert_eq!(scene.property("themeSplash", "time").parse::<f64>().unwrap(), ended(&mut scene), "the time past the hand-over at once");
        let tree = scene.tree();
        assert!(node(&tree, "disc").is_some_and(|n| n.visible) && whole(&tree), "Home whole at once under reduced motion");
        scene.hold_clock(None);
    }
    assert!(differences.is_empty(), "{} differences:\n{}", differences.len(), differences.join("\n"));
    harness.assert_quiet();
    drop(kwin);
}

/// The disk driver's file as 32-bit float samples, the device's format SDL opens for the interface player's stream.
fn heard_samples(path: &PathBuf) -> Vec<f32> {
    std::fs::read(path).unwrap_or_default().chunks_exact(4).map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]])).collect()
}
