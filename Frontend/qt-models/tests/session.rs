//! A game through the picture's feed with no window (EmuSen_Qt.md §1.8.4): the test core opened on its frame thread,
//! its pictures handed over, and the feed's two calls made as GamePicture's render thread makes them, its sound played
//! through Endymion's game player on SDL's dummy driver. On the RX 6800.

use emusen_client_fixture::{library, World};
use emusen_crystal_client::{Api, Config, Runtime};
use emusen_kit_qt::picture::{Feed, Picture};
use emusen_platform_client::serenity::{builtin_preset, Device};
use emusen_platform_client::audio::{GameSound, SoundSettings};
use emusen_platform_client::pace::DisplayClock;
use emusen_platform_client::{Lent, Platform};
use emusen_qt_models::picture::PictureFeed;
use emusen_qt_models::session::{shown_aspect, GameIo, Report, Session};
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Vulkan's `VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL`.
const SHADER_READ_ONLY: u32 = 5;

fn with_test_core(world: &World) -> Runtime {
    let player = world.dir("Cores");
    world.player_core(&player);
    let api = Api::load(library()).unwrap();
    let rt = Runtime::new(
        &api,
        &Config { development: false, cores: Some(Vec::new()), player_cores: Some(vec![player.to_str().unwrap().into()]) },
    )
    .unwrap();
    let sha = rt
        .cores()
        .unwrap()
        .into_iter()
        .find(|c| c.directory.as_deref() == Some("player"))
        .and_then(|c| c.sha256)
        .expect("the player's core");
    rt.core_approve(&sha).unwrap();
    rt
}

fn feed_of(picture: &PictureFeed) -> &Feed {
    unsafe { &*(picture.address() as *const Feed) }
}

fn wait_until(what: &str, mut done: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(10);
    while !done() {
        assert!(Instant::now() < until, "{what}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn a_game_s_pictures_are_drawn_in_place_paced_and_paused() {
    // Set before SDL is first loaded, while this binary's one test is its only thread that reads the environment.
    unsafe { std::env::set_var("SDL_AUDIODRIVER", "dummy") };
    let world = World::new("session");
    let runtime = with_test_core(&world);
    let platform = Platform::load(library()).unwrap();
    let device =
        Arc::new(Device::new_for(&platform, std::env::var("EMUSEN_GPU_DEVICE").ok().as_deref(), true, &[], &[]).expect("a Vulkan device"));
    let raw = device.raw();
    assert!(raw.instance != 0 && raw.physical != 0 && raw.device != 0 && raw.queue != 0, "{raw:?}");
    let preset = builtin_preset(&platform, "effect.None.nearest", None, world.dir("presets").to_str().unwrap()).unwrap();
    let picture = PictureFeed::new(device.clone(), preset, world.dir("presets"));
    let handoff = picture.handoff();
    let feed = feed_of(&picture);

    let mut out = Picture::default();
    assert_eq!((feed.draw)(feed.context, 64, 48, &mut out), 0, "nothing to show before a frame");

    platform.lend_sdl(&Lent::open("libSDL3.so.0").unwrap()).unwrap();
    let sound = GameSound::new(&platform, SoundSettings::default()).unwrap();
    let (tell, told) = mpsc::channel();
    let game = world.game("game.tst", &[1, 2, 3, 4]);
    let session = Session::start(
        runtime,
        game.to_str().unwrap().into(),
        None,
        std::ptr::NonNull::new(device.as_ptr().cast_mut()),
        handoff.clone(),
        GameIo { pads: None, sound: Some(sound), sync: true, cheats: None },
        DisplayClock::new(&platform),
        false,
        move |r| {
            let _ = tell.send(r);
        },
    );
    assert!(matches!(told.recv_timeout(Duration::from_secs(10)), Ok(Report::Opened { .. })));
    assert!(matches!(told.recv_timeout(Duration::from_secs(10)), Ok(Report::FirstPicture { .. })));

    assert_eq!((feed.draw)(feed.context, 64, 48, &mut out), 1, "{:?}", picture.problem());
    assert!(out.image != 0 && (out.width, out.height, out.layout) == (64, 48, SHADER_READ_ONLY), "{out:?}");
    assert_eq!((feed.draw)(feed.context, 80, 60, &mut out), 1, "{:?}", picture.problem());
    assert_eq!((out.width, out.height), (80, 60), "a new size remakes the output");

    // Swaps at 60 Hz: the frame thread locks to them, one frame a swap.
    let start = 1_000_000_000i64;
    let before = picture.counts().handed.load(Ordering::Relaxed);
    for n in 0..30 {
        (feed.swapped)(feed.context, start + n * 16_666_667);
        (feed.draw)(feed.context, 80, 60, &mut out);
        std::thread::sleep(Duration::from_millis(4));
    }
    assert_eq!(picture.counts().swaps.load(Ordering::Relaxed), 30);
    // Locked by Serenity's display clock, a frame follows each draw at once rather than a sixtieth of a second later.
    let locked = picture.counts().handed.load(Ordering::Relaxed) - before;
    assert!(locked >= 24, "{locked} frames over 30 swaps 4 ms apart: the frame thread did not lock to the swaps");
    // The test core makes a sixtieth of a second of its rate (32 kHz) each frame; every frame's reaches the player.
    let frames = picture.counts().handed.load(Ordering::Relaxed);
    let sounded = session.sounded();
    assert!(frames > 0 && sounded >= frames * (32000 / 60), "{sounded} stereo frames of sound played over {frames} frames");

    // Sync to display off: the frame thread runs on the game's own clock, whatever the swaps.
    session.set_sync(false);
    std::thread::sleep(Duration::from_millis(50));
    let before = picture.counts().handed.load(Ordering::Relaxed);
    for n in 30..60 {
        (feed.swapped)(feed.context, start + n * 16_666_667);
        (feed.draw)(feed.context, 80, 60, &mut out);
        std::thread::sleep(Duration::from_millis(4));
    }
    let free = picture.counts().handed.load(Ordering::Relaxed) - before;
    assert!(free <= 16, "{free} frames over 30 swaps 4 ms apart: the frame thread followed the swaps with Sync to display off");
    session.set_sync(true);
    let start = start + 30 * 16_666_667;

    // A volume and a device asked of the running game reach its player between two frames; a name no device has leaves
    // the system default playing.
    let mut heard = std::iter::from_fn(|| told.recv_timeout(Duration::from_secs(5)).ok());
    session.set_volume(0.5);
    assert!(heard.find_map(|r| if let Report::Volume(v) = r { Some(v) } else { None }) == Some(0.5));
    session.set_device(Some("No such device".into()));
    assert!(heard.find_map(|r| if let Report::Device(d) = r { Some(d) } else { None }) == Some(None));
    drop(heard);

    session.set_paused(true);
    std::thread::sleep(Duration::from_millis(100));
    let handed = picture.counts().handed.load(Ordering::Relaxed);
    let sounded = session.sounded();
    for n in 30..40 {
        (feed.swapped)(feed.context, start + n * 16_666_667);
    }
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(picture.counts().handed.load(Ordering::Relaxed), handed, "a paused game hands nothing over");
    assert_eq!(session.sounded(), sounded, "a paused game plays no sound");
    session.set_paused(false);
    wait_until("a resumed game hands frames over again", || picture.counts().handed.load(Ordering::Relaxed) > handed);

    drop(session);
    assert!(told.try_iter().any(|r| r == Report::Closed));
    drop(picture);
    assert_eq!(device.findings().unwrap(), Vec::<String>::new());
}

#[test]
fn the_picture_s_shape_is_the_runtime_s_display_aspect_else_the_frame_s_own() {
    let mut info = emusen_crystal_client::frame_info();
    assert_eq!(shown_aspect(&info), 0.0, "no picture, no shape: the picture fills its item");
    (info.width, info.height, info.row_repeat) = (256, 224, 1);
    info.display_aspect = 4.0 / 3.0;
    assert_eq!(shown_aspect(&info), 4.0 / 3.0, "the console's screen, not the frame's 256 by 224");
    info.display_aspect = 0.0;
    assert_eq!(shown_aspect(&info), 256.0 / 224.0, "with none declared, the frame's own shape");
    (info.width, info.height, info.row_repeat) = (512, 224, 2);
    assert_eq!(shown_aspect(&info), 512.0 / 448.0, "each line drawn twice where the frame repeats its rows");
}
