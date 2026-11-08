// EmuSen_Kit.md §7.5: every cue is rendered once and kept; a second ask costs no render.

use emusen_kit_core::sound::{Cue, RATE};
use emusen_kit_qt::bellcache::BellCache;
use emusen_kit_qt::kinds::{self, Cue as QmlCue};

#[test]
fn kit_7_5_a_cue_renders_once() {
    let mut cache = BellCache::default();
    let first = cache.cue(Cue::Choose).len();
    assert!(first > 0);
    assert_eq!(cache.renders(), 1);
    assert_eq!(cache.cue(Cue::Choose).len(), first);
    assert_eq!(cache.renders(), 1, "kept, not rendered again");
    cache.cue(Cue::Move);
    cache.quick_select();
    cache.quick_select();
    assert_eq!(cache.renders(), 3);
}

#[test]
fn kit_7_5_durations_are_stereo_frames_at_48_khz() {
    assert_eq!(RATE, 48_000);
    assert_eq!(BellCache::duration_ms(&[0.0; 96_000]), 1000);
    let mut cache = BellCache::default();
    let ms = BellCache::duration_ms(cache.cue(Cue::Launch));
    assert!(ms > 300 && ms < 3000, "{ms} ms");
}

#[test]
fn kit_7_1_every_cue_mirrors_its_kit_core_cue() {
    for (i, cue) in Cue::ALL.iter().enumerate() {
        let qml = QmlCue { repr: i as i32 };
        assert_eq!(kinds::sound_cue(qml), Some(*cue));
        assert_eq!(kinds::cue_of(*cue).repr, qml.repr);
    }
    assert_eq!(kinds::sound_cue(QmlCue { repr: 9 }), None);
}
