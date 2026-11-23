//! Eclipse's splash on the real program, on the card, held to its board and entering Home, a GPU job (EmuSen_Qt.md §3.5.3).

// The modules the scene imports register through the program's crate, which a test must link.
extern crate emusen_qt;

mod eclipse_board;
#[allow(dead_code)]
mod eclipse_home_oracle;
mod start_held;

use eclipse_board::read;
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

const TABLE: &str = include_str!("eclipse_splash.tsv");
const SOUND: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../themes/eclipse/sounds/splash.wav");

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

/// The frames of the theme's sound file: its data chunk's bytes over 16-bit stereo's four a frame.
fn sound_frames() -> i64 {
    let bytes = std::fs::read(SOUND).unwrap();
    let at = bytes.windows(4).position(|w| w == b"data").expect("the WAV's data chunk");
    i64::from(u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap())) / 4
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
            // Words by the board's rule for their box, as eclipse_home.tsv holds Home's: the name about its line's middle, the counts from its top.
            "wordmark" | "counts" => {
                let down = if row.name == "wordmark" { (y + h / 2.0, row.y + row.h / 2.0) } else { (y, row.y) };
                if off(x + w / 2.0, centre) || off(down.0, down.1) {
                    differences.push(format!("{at} {moment}: {name} at ({}, {}) where the board's column puts it at ({centre}, {})", x + w / 2.0, down.0, down.1));
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
        assert_eq!(scene.property("themeSplash", "time"), scene.property("themeSplash", "length"), "skipped to its last frame");
        assert_eq!(scene.property("splashSound", "ends"), "1", "the sound let go");
        until(&mut scene, "the splash gone after the skip", 3000, |s| s.property("themeSplash", "done") == "true");
        scene.wait(300);
        assert_eq!(scene.property("screens", "shown"), shown, "the press that skipped is not Home's");
        assert_eq!(scene.property("library", "lastPlayed"), eclipse_home_oracle::rom(&world, eclipse_home_oracle::GAMES[0].0, eclipse_home_oracle::GAMES[0].1), "no game started");
        let tree = scene.tree();
        assert!(node(&tree, "disc").is_some_and(|n| n.visible), "Home whole");
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
        assert!(scene.property("splashSoundFile", "source").ends_with("/eclipse/sounds/splash.wav"), "{at}: the theme's own file");
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
        // On to its end: the splash goes, Home whole and its own disc drawn.
        scene.hold_clock(Some(10_200));
        until(&mut scene, "the splash gone", 3000, |s| s.property("themeSplash", "done") == "true");
        scene.wait(200);
        let tree = scene.tree();
        assert!(node(&tree, "splashScreen").is_none_or(|n| !n.visible), "{at}: the splash is gone");
        assert!(node(&tree, "disc").is_some_and(|n| n.visible), "{at}: Home's disc stands in the splash's");
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
