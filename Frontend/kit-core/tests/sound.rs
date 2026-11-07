// The bells and the cues held to LunaP §219.4, §220 and EmuSen_Interface.md §5.6.

use emusen_kit_core::sound::{
    Bell, Chime, Cue, DEFAULT_VOLUME, ON_BY_DEFAULT, QUICK_SELECT, RATE, UNDER_GAMEPLAY_DECIBELS, bell_samples, loudest,
    quick_select_samples, render, render_bells,
};
use emusen_kit_core::world::{Splash, SplashBody, Timeline};

fn close(a: f64, b: f64, within: f64) -> bool {
    (a - b).abs() <= within
}

/// The left channel of a stereo rendering.
fn mono(samples: &[f32]) -> Vec<f32> {
    samples.iter().step_by(2).copied().collect()
}

/// Goertzel's power at a frequency over a window of samples.
fn power(samples: &[f32], from: usize, len: usize, hz: f64) -> f64 {
    let k = 2.0 * std::f64::consts::PI * hz / RATE as f64;
    let coeff = 2.0 * k.cos();
    let (mut s1, mut s2) = (0.0f64, 0.0f64);
    for v in samples.iter().skip(from).take(len) {
        let s0 = *v as f64 + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    s1 * s1 + s2 * s2 - coeff * s1 * s2
}

/// The ladder rung whose pitch is loudest in a window starting 10 ms after a strike.
fn loudest_rung(samples: &[f32], strike_at: f64) -> usize {
    let from = ((strike_at + 0.010) * RATE as f64) as usize;
    let len = (0.060 * RATE as f64) as usize;
    (0..13)
        .max_by(|a, b| {
            power(samples, from, len, Chime::MOONRISE.note_of(*a)).total_cmp(&power(samples, from, len, Chime::MOONRISE.note_of(*b)))
        })
        .unwrap()
}

#[test]
fn lunap_219_4_the_ladder_climbs_g_major_pentatonic_from_g4() {
    let expected = [391.995, 440.0, 493.88, 587.33, 659.25, 783.99, 880.0, 987.77, 1174.66, 1318.51, 1567.98, 1760.0, 1975.53];
    for (rung, hz) in expected.into_iter().enumerate() {
        assert!(close(Chime::MOONRISE.note_of(rung), hz, 0.01), "rung {rung}: {}", Chime::MOONRISE.note_of(rung));
    }
    assert!(close(Chime::MOONRISE.note_of(13), 2349.32, 0.01), "a fourteenth planet rings D7");
}

#[test]
fn lunap_219_4_the_moonrise_chime_is_four_partials_dry_at_the_menus_level() {
    let c = Chime::MOONRISE;
    assert_eq!(c.ladder, &[0, 2, 4, 7, 9]);
    let ratios: Vec<f64> = c.partials.iter().map(|p| p.ratio).collect();
    assert_eq!(ratios, vec![1.0, 2.76, 5.40, 8.93]);
    let amplitudes: Vec<f64> = c.partials.iter().map(|p| p.amplitude).collect();
    assert_eq!(amplitudes, vec![1.0, 0.18, 0.06, 0.02]);
    let decays: Vec<f64> = c.partials.iter().map(|p| p.decay).collect();
    assert_eq!(decays, vec![0.55, 0.22, 0.11, 0.06]);
    assert_eq!((c.attack, c.tail, c.detune, c.detune_seed), (0.018, 0.05, 0.002, 0x4265_6c6c));
    assert_eq!((c.echo, c.feedback), (0.0, 0.0), "dry");
    assert_eq!((c.gain, c.lowpass, c.lowpass_q), (0.2, 2400.0, 0.5));
    assert_eq!(RATE, 48_000);
}

#[test]
fn lunap_220_2_the_cues_strikes() {
    let notes = |cue: Cue| cue.bells().iter().map(|b| b.note).collect::<Vec<_>>();
    let starts = |cue: Cue| cue.bells().iter().map(|b| b.start).collect::<Vec<_>>();
    assert_eq!(Cue::ALL.len(), 9);
    assert_eq!(Cue::Move.bells(), &[Bell::new(8, 0.0, 0.3, 0.12)], "a short D6");
    assert_eq!(Cue::Section.bells(), &[Bell::new(5, 0.0, 0.55, 0.18)], "a G5");
    assert_eq!(notes(Cue::Choose), vec![5, 8], "G5 then D6");
    assert_eq!(notes(Cue::Back), vec![8, 5], "D6 then G5");
    assert_eq!((notes(Cue::Favourite), starts(Cue::Favourite)), (vec![5, 7, 8, 10], vec![0.0, 0.055, 0.11, 0.165]));
    assert_eq!(notes(Cue::Launch), vec![0, 1, 2, 3, 4, 5], "the splash's run from G4 to G5");
    assert_eq!(starts(Cue::Launch), vec![0.0, 0.075, 0.15, 0.225, 0.3, 0.375]);
    assert_eq!(Cue::Launch.bells()[5].decay, 1.2, "the last rings on");
    assert_eq!(notes(Cue::Saved), vec![7, 9], "B5 then E6");
    assert_eq!(notes(Cue::Loaded), vec![9, 7], "the reverse");
    assert_eq!(notes(Cue::Refused), vec![0, 0], "G4 twice");
    assert!(Cue::Refused.bells().iter().all(|b| b.lowpass == 900.0), "damped and dull");
    assert!(Cue::ALL.iter().filter(|c| **c != Cue::Refused).all(|c| c.bells().iter().all(|b| b.lowpass == 2400.0)));
    assert_eq!(QUICK_SELECT.iter().map(|b| b.note).collect::<Vec<_>>(), vec![4, 6], "E5 then A5");
}

#[test]
fn interface_5_6_the_cues_defaults_and_which_play_over_a_game() {
    assert_eq!((DEFAULT_VOLUME, ON_BY_DEFAULT, UNDER_GAMEPLAY_DECIBELS), (0.6, true, -12.0));
    let over: Vec<Cue> = Cue::ALL.into_iter().filter(|c| c.plays_over_gameplay()).collect();
    assert_eq!(over, vec![Cue::Saved, Cue::Loaded]);
}

#[test]
fn lunap_220_5_each_cues_notes_are_present_at_their_strike_times() {
    for cue in Cue::ALL {
        let samples = mono(&cue.samples());
        for bell in cue.bells() {
            assert_eq!(loudest_rung(&samples, bell.start), bell.note, "{cue:?} at {}", bell.start);
        }
    }
    let quick = mono(&quick_select_samples());
    for bell in QUICK_SELECT {
        assert_eq!(loudest_rung(&quick, bell.start), bell.note);
    }
}

#[test]
fn lunap_220_5_two_renders_are_identical_and_an_empty_one_is_silent() {
    assert_eq!(Cue::Launch.samples(), Cue::Launch.samples());
    assert_eq!(render(&Chime::MOONRISE, &[1.0, 1.2, 1.5]), render(&Chime::MOONRISE, &[1.0, 1.2, 1.5]));
    assert!(render_bells(&Chime::MOONRISE, &[]).is_empty());
}

#[test]
fn lunap_220_5_each_cues_loudest_sample_is_within_0_05_and_0_6() {
    for cue in Cue::ALL {
        let peak = loudest(&cue.samples());
        assert!((0.05..=0.6).contains(&peak), "{cue:?} peaks at {peak}");
    }
    let quick = loudest(&quick_select_samples());
    assert!((0.05..=0.6).contains(&quick));
}

#[test]
fn lunap_220_3_one_bell_rings_at_the_requested_rung() {
    for rung in [0, 5, 8, 12] {
        let samples = mono(&bell_samples(rung));
        assert_eq!(loudest_rung(&samples, 0.0), rung);
    }
    assert_eq!(Bell::new(3, 0.1, 0.6, 0.18).lowpass, 2400.0);
    assert_eq!(Bell::dull(0, 0.0, 0.25, 0.26, 900.0).lowpass, 900.0);
}

#[test]
fn lunap_219_4_thirteen_bells_at_1280x720_last_about_1_8_s_from_the_first_strike_at_the_cues_level() {
    let bodies = (0..13).map(|i| SplashBody { name: format!("world {i}"), hue: i as f64 * 27.0, body: None }).collect();
    let splash = Splash::new(Timeline::MOONRISE, bodies);
    let strikes = splash.strike_times(1280.0, 720.0);
    let samples = render(&Chime::MOONRISE, &strikes);
    let seconds = samples.len() as f64 / 2.0 / RATE as f64;
    assert!((1.7..2.6).contains(&seconds), "{seconds} s");
    let left = mono(&samples);
    assert!(left[..48].iter().any(|v| v.abs() > 0.0), "the sound begins at the first strike");
    let peak = loudest(&samples);
    assert!((0.05..=0.6).contains(&peak), "at the menus' level: {peak}");
    for (n, at) in strikes.iter().enumerate() {
        assert_eq!(loudest_rung(&left, at - strikes[0]), n, "planet {n} rings rung {n}");
    }
    assert_eq!(render(&Chime::MOONRISE, &strikes), samples, "the same for the same times");
}
