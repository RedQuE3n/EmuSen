// What the fade tests of the real view share (EmuSen_Qt.md §4.8.8): two systems whose pictures differ in width, a theme
// stepping between them by its profile, and one read of the item tree per frame of the held clock.

use emusen_kit_core::input::UiButton;
use emusen_qt_test::{Harness, Scene};

use super::{settings, until, write_theme, Scratch, VIEW};

// The test's window, a size of the test's own design.
const W: i32 = 1280;
const H: i32 = 800;
/// The held clock's frame: `harness_clock_move` steps 16 ms at a time (EmuSen_Kit.md §7.36).
pub const STEP_MS: i64 = 16;
/// The two systems' pictures, drawn at their own widths: the system left's and the arriving one's.
pub const LEFT_WIDTH: f64 = 40.0;
pub const ARRIVING_WIDTH: f64 = 80.0;

/// ES-DE 3.4.1's fade between systems, stated from its source, not from the scene's code.
pub mod es_de {
    /// A carousel's or list's run: 400 ms (es-app/src/views/SystemView.cpp 427) x 1.3, cast to whole ms (476).
    pub const RUN_MS: f64 = 520.0;

    /// The black at a time into the run: t = time / run (es-core/src/animations/AnimationController.cpp 41); t / 0.2
    /// while t < 0.3, whole while t < 0.7, then 1 - (t - 0.6) / 0.3, each clamped to 0..1 (SystemView.cpp 451-457).
    pub fn black(ms: f64) -> f64 {
        let t = (ms / RUN_MS).clamp(0.0, 1.0);
        if t < 0.3 {
            (t / 0.2).clamp(0.0, 1.0)
        } else if t < 0.7 {
            1.0
        } else {
            1.0 - ((t - 0.6) / 0.3).clamp(0.0, 1.0)
        }
    }

    /// Whether the system left still shows: the camera moves to the new one once t > 0.5 (SystemView.cpp 459).
    pub fn shows_left(ms: f64) -> bool {
        ms / RUN_MS <= 0.5
    }
}

/// ES-DE computes the curve in single precision, the scene in double; the two agree well inside this.
pub const PRECISION: f64 = 1e-6;

/// One frame as the view drew it, `ms` after the press on the held clock.
#[derive(Debug)]
pub struct Read {
    pub ms: f64,
    /// The view's own clock since the scene began to move (View.qml's `viewClock`).
    pub ticked: f64,
    pub moving: bool,
    /// The black's opacity, 0 where none is drawn.
    pub black: f64,
    pub below_width: Option<f64>,
    pub above: f64,
    /// The drawn order of the picture below, the black, the primary and the picture above, by their place in the tree.
    pub order: Vec<&'static str>,
}

fn read(scene: &mut Scene<'_>, ms: f64) -> Read {
    let tree = scene.tree();
    let node = |name: &str| tree.nodes.iter().position(|n| n.name == name && n.visible);
    let opacity = |name: &str| node(name).map_or(0.0, |i| tree.nodes[i].opacity);
    let mut order: Vec<(usize, &'static str)> =
        [("element-below", "below"), ("element-veil", "veil"), ("element-systems", "systems"), ("element-above", "above")]
            .into_iter()
            .filter_map(|(n, short)| node(n).map(|i| (i, short)))
            .collect();
    order.sort();
    Read {
        ms,
        ticked: scene.property("viewClock", "tickedMs").parse().unwrap_or(f64::NAN),
        moving: scene.property("esdeModel", "moving") == "true",
        black: opacity("element-veil"),
        below_width: node("element-below").map(|i| tree.nodes[i].w.round()),
        above: opacity("element-above"),
        order: order.into_iter().map(|(_, s)| s).collect(),
    }
}

/// A theme whose systemToSystem is `kind`, its first system shown and at rest, then one press to the next system and
/// `frames` frames of the held clock read after it: the press at 0 ms, a frame each STEP_MS.
pub fn run(kind: &str, fade_above: bool, frames: i64) -> Vec<Read> {
    assert!(emusen_esde_qt::marks::mark_named("star").is_some(), "the module is linked into this test");
    let _kwin = super::start();
    // A held clock is refused under a threaded render loop, which installs its own animation driver (EmuSen_Kit.md §7.36).
    std::env::set_var("QSG_RENDER_LOOP", "basic");
    let dir = Scratch::new(&format!("fade-{kind}"));
    let theme = dir.path("theme");
    let caps = format!("<transitions name=\"p\"><systemToSystem>{kind}</systemToSystem><startupToSystem>instant</startupToSystem></transitions>");
    let above = if fade_above { "<fadeAbovePrimary>true</fadeAbovePrimary>" } else { "" };
    // Below the carousel's zIndex 50 a picture each system names; above it one that fades only by fadeAbovePrimary.
    let body = format!(
        "<view name=\"system\"><carousel name=\"systems\"><pos>0 0.3</pos><size>1 0.4</size><maxItemCount>3</maxItemCount>\
         <text>${{system.fullName}}</text><textColor>FFFFFF</textColor><fontSize>0.05</fontSize>{above}</carousel>\
         <image name=\"below\"><pos>0 0</pos><path>./${{system.theme}}.bmp</path><zIndex>40</zIndex></image>\
         <image name=\"above\"><pos>0.5 0.8</pos><size>0.2 0.2</size><path>./nes.bmp</path><zIndex>60</zIndex></image></view>\
         <view name=\"gamelist\"><textlist name=\"games\"><pos>0.05 0.1</pos><size>0.4 0.8</size></textlist></view>"
    );
    write_theme(&theme, &caps, &body);
    super::bmp(&theme.join("nes.bmp"), LEFT_WIDTH as u32, 40, |_, _| [200, 40, 40]);
    super::bmp(&theme.join("snes.bmp"), ARRIVING_WIDTH as u32, 40, |_, _| [40, 40, 200]);
    let shelves = "[{\"name\":\"nes\",\"full_name\":\"Nintendo\",\"theme\":\"nes\",\"kind\":\"regular\",\"games\":[]},\
                   {\"name\":\"snes\",\"full_name\":\"Super Nintendo\",\"theme\":\"snes\",\"kind\":\"regular\",\"games\":[]}]";
    let mut harness = Harness::new();
    let mut scene = harness.load(VIEW, W, H);
    assert!(scene.set_property("view", "settings", &settings(&theme, "")));
    assert!(scene.set_property("view", "shelves", shelves));
    until(&mut scene, "the theme loaded", |s| s.property("esdeModel", "themed") == "true");
    until(&mut scene, "the view at rest", |s| s.property("esdeModel", "moving") == "false");
    until(&mut scene, "the first system's picture measured", |s| read(s, 0.0).below_width == Some(LEFT_WIDTH));
    scene.hold_clock(Some(0));
    scene.pad(UiButton::Right);
    let mut reads = vec![read(&mut scene, 0.0)];
    for k in 1..=frames {
        scene.hold_clock(Some(k * STEP_MS));
        let ms = (k * STEP_MS) as f64;
        let mut r = read(&mut scene, ms);
        // The painter measures a picture it has not drawn before off the clock; the held clock does not move meanwhile.
        if r.below_width != Some(LEFT_WIDTH) {
            until(&mut scene, "the arriving picture measured", |s| read(s, ms).below_width == Some(ARRIVING_WIDTH));
            r = read(&mut scene, ms);
        }
        reads.push(r);
    }
    scene.hold_clock(None);
    reads
}

/// The view's clock is a NumberAnimation's value, interpolated in double (View.qml), and the painter rounds each step to a
/// whole microsecond (painter.rs `tick`): within half of one, the painter's time is the held time exactly.
pub const CLOCK_ROUNDING_MS: f64 = 0.0005;

/// The view's clock stepped with the held one: the press at its 0, a frame each STEP_MS while the scene moved.
pub fn assert_clock_held(reads: &[Read]) {
    let mut moved = true;
    for r in reads {
        if moved {
            assert!((r.ticked - r.ms).abs() < CLOCK_ROUNDING_MS, "the view's clock at the held {} ms is {}: {reads:#?}", r.ms, r.ticked);
        }
        moved = r.moving;
    }
}
