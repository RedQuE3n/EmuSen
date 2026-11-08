// The drawn world held to LunaP §219 (the splash), §221 (the sky), §222 (the bodies) and EmuSen_Interface.md §22, plus an independent Mulberry32 oracle.

use std::time::Duration;

use emusen_kit_core::motion::{Curve, Glide, spring};
use emusen_kit_core::tokens::Colour;
use emusen_kit_core::world::{
    CAPTION_SIZE, DRAFT_TURN, GLOW_RADII, GLOW_STRENGTH, HEIGHT_SHARE, MOONRISE_SKY, Paint, SCREENS_SKY, STAR_COLOUR, SeededRandom, Shape,
    SkyGlow, Span, Splash, SplashBody, TITLE_SIZE, Timeline, WorldBody, WorldRing, WorldShape, body_box, colours, ease, facet_inks,
    geometry, glow, glow_bounds, glow_stops, place_stars, round_spacing, size_of, star_points, tallness, twinkle, unit_box, wrap_hue,
};

fn close(a: f64, b: f64, within: f64) -> bool {
    (a - b).abs() <= within
}

fn ms(v: u64) -> Duration {
    Duration::from_millis(v)
}

#[test]
fn lunap_219_3_mulberry32_matches_an_independent_run_of_the_published_generator() {
    let moonrise = SeededRandom::first(0x4d6f_6f6e, 5);
    let expected = [0.687270255061, 0.111584058264, 0.303902204148, 0.915673088282, 0.517944498220];
    for (got, want) in moonrise.iter().zip(expected) {
        assert!(close(*got, want, 1e-9), "{got} vs {want}");
    }
    let bells = SeededRandom::first(0x4265_6c6c, 3);
    for (got, want) in bells.iter().zip([0.478135526413, 0.986048294231, 0.757816098398]) {
        assert!(close(*got, want, 1e-9));
    }
    for (got, want) in SeededRandom::first(1, 3).iter().zip([0.627073940588, 0.002735721180, 0.527447039960]) {
        assert!(close(*got, want, 1e-9));
    }
}

#[test]
fn lunap_219_2_the_moonrise_timeline_is_the_drafts_spans_at_pace_1_55_for_8_s() {
    let t = Timeline::MOONRISE;
    assert_eq!((t.pace, t.length), (1.55, 8.0));
    assert_eq!(t.stars, Span::new(0.0, 0.9));
    assert_eq!(t.stroke, Span::new(0.4, 1.6));
    assert_eq!(t.halo, Span::new(0.9, 2.0));
    assert_eq!(t.fill, Span::new(1.2, 2.0));
    assert_eq!((t.appear, t.appear_stagger), (Span::new(1.15, 1.6), 0.03));
    assert_eq!((t.settle, t.settle_rate, t.settle_stagger, t.orbit_swing), (Span::new(1.2, 2.7), 1.25, 0.02, 2.6));
    assert_eq!((t.sweep, t.flare_width), (Span::new(2.35, 3.65), 34.0));
    assert_eq!(t.line, Span::new(2.4, 3.7));
    assert_eq!(t.words, Span::new(3.45, 4.15));
    assert_eq!((t.star_count, t.seed, t.fade_out), (220, 0x4d6f_6f6e, 0.15));
    assert!(close(t.draft_time(3.1), 2.0, 1e-12));
    assert!(close(t.draft_time(100.0), 8.0 / 1.55, 1e-12), "clamped to the length");
    assert_eq!(t.draft_time(-1.0), 0.0);
    let s = Span::new(1.0, 3.0);
    assert_eq!((s.at(0.5), s.at(2.0), s.at(4.0)), (0.0, 0.5, 1.0));
    assert_eq!(Span::new(2.0, 2.0).at(2.0), 1.0, "an empty span is done at its time");
}

fn thirteen() -> Splash {
    let bodies = (0..13).map(|i| SplashBody { name: format!("world {i}"), hue: i as f64 * 27.0, body: None }).collect();
    Splash::new(Timeline::MOONRISE, bodies)
}

#[test]
fn lunap_219_4_thirteen_bells_at_1280x720_ring_from_3_75_s_over_about_1_8_s() {
    let times = thirteen().strike_times(1280.0, 720.0);
    assert_eq!(times.len(), 13);
    assert!(close(times[0], 3.7524, 0.001), "{}", times[0]);
    assert!(close(times[12], 5.5476, 0.001), "{}", times[12]);
    assert!(close(times[12] - times[0], 1.8, 0.01));
    assert!(times.windows(2).all(|w| w[1] > w[0]), "left to right");
    let one = Splash::new(Timeline::MOONRISE, vec![SplashBody { name: "only".into(), hue: 0.0, body: None }]);
    assert!(close(one.strike_times(1280.0, 720.0)[0], (2.35 + 0.5 * 1.3) * 1.55, 1e-9), "one planet rings at the row's middle");
    assert_eq!(thirteen().strike_times(1280.0, 0.0), vec![0.0; 13], "no size yet: no times");
}

#[test]
fn lunap_219_5_the_spring_ends_0_2_percent_short_and_the_splash_eases_cubic_out() {
    assert_eq!(spring(0.0), 0.0);
    assert!(close(spring(1.0), 0.998, 0.0005), "{}", spring(1.0));
    assert!(close(spring(2.0), spring(1.0), 1e-12), "clamped");
    assert_eq!(ease(0.5), Curve::CubicOut.at(0.5));
}

#[test]
fn lunap_219_2_the_finished_frame_has_every_planet_in_the_row_the_line_full_and_the_words_settled() {
    let splash = thirteen();
    let (w, h) = (1280.0, 720.0);
    let frame = splash.frame(8.0, w, h);
    assert!(frame.finished);
    assert!(close(frame.stars_opacity, 0.9, 1e-12));
    assert_eq!((frame.moon.outline_drawn, frame.moon.fill, frame.moon.halo_opacity), (1.0, 1.0, 1.0));
    assert_eq!(frame.moon.centre, [640.0, 720.0 * 0.36]);
    assert_eq!(frame.moon.radius, 92.0);
    assert_eq!(frame.planets.len(), 13);
    let row_w = Splash::row_width(w, h);
    assert_eq!(row_w, 980.0, "the lesser of 78% of 1280 and 980 units");
    for p in &frame.planets {
        assert_eq!(p.opacity, 1.0);
        let slot = w / 2.0 - row_w / 2.0 + row_w * p.index as f64 / 12.0;
        assert!(close(p.centre[0], slot, 2.0), "planet {} at {} not {slot}", p.index, p.centre[0]);
        assert!(close(p.centre[1], h * 0.66, 2.0), "planet {} at y {}", p.index, p.centre[1]);
        assert!(close(p.radius, (5.0 + (p.index % 4) as f64 * 1.6) * 1.25, 1e-9));
        assert_eq!(p.flare, 0.0, "the sweep has passed");
    }
    assert!(frame.sweep.is_none());
    let line = frame.line.expect("the line is drawn");
    assert!(close(line.from_x, w / 2.0 - row_w / 2.0, 1e-9));
    assert!(close(line.to_x, w / 2.0 + row_w / 2.0, 1e-9));
    assert_eq!((line.y, line.opacity), (h * 0.66, 1.0));
    let words = frame.words.expect("the words are drawn");
    assert_eq!(words.opacity, 1.0);
    assert_eq!(words.title_size, TITLE_SIZE);
    assert_eq!(words.title_spacing, 10.0, "closed from 18 to 10 units");
    assert_eq!(words.title_baseline_y, h * 0.85);
    assert_eq!((words.caption_size, words.caption_spacing, words.caption_baseline_y), (CAPTION_SIZE, 6.0, h * 0.91));
    assert_eq!(splash.frame(9.0, w, h), frame, "the frame holds after the length");
}

#[test]
fn lunap_219_2_the_first_frame_is_dark_and_the_sweep_crosses_the_rows_middle_half_way() {
    let splash = thirteen();
    let (w, h) = (1280.0, 720.0);
    let start = splash.frame(0.0, w, h);
    assert!(!start.finished);
    assert_eq!(start.stars_opacity, 0.0);
    assert_eq!((start.moon.outline_drawn, start.moon.fill, start.moon.halo_opacity), (0.0, 0.0, 0.0));
    assert!(start.planets.is_empty() && start.sweep.is_none() && start.line.is_none() && start.words.is_none());

    let mid = splash.frame(3.0 * 1.55, w, h);
    assert!(close(mid.draft_time, 3.0, 1e-12));
    let sweep = mid.sweep.expect("the light is sweeping");
    assert!(close(sweep.x, w / 2.0, 1e-9), "half way along the sweep is the row's middle");
    assert_eq!(sweep.band, [sweep.x - 90.0, h * 0.66 - 26.0, 180.0, 52.0]);
    let middle = &mid.planets[6];
    assert!(close(middle.flare, 1.0, 0.01), "the middle planet flares as the light passes: {}", middle.flare);
    assert!(close(middle.flare_radius, middle.radius * 4.2, 0.05));
    assert!(close(middle.flare_opacity, 0.55, 0.01));
    assert!(mid.planets[0].flare < 0.01, "the first planet has stopped flaring");
    assert!(mid.moon.shadow_centre[0] > mid.moon.centre[0], "the shadow sits to the right, cutting a crescent");

    let no_size = splash.frame(3.0 * 1.55, 0.0, 0.0);
    assert!(
        no_size.planets.iter().all(|p| p.flare == 0.0 && p.flare_opacity == 0.0 && p.flare_radius.is_finite()),
        "no size yet: no flare, nothing undefined"
    );

    let partly = splash.frame(1.0 * 1.55, w, h);
    assert!(partly.moon.outline_drawn > 0.0 && partly.moon.outline_drawn < 1.0);
    assert_eq!(partly.moon.fill, 0.0, "the fill begins at 1.2");
}

#[test]
fn lunap_222_6_a_bodys_box_is_its_disc_or_2_5_radii_tall_about_the_centre() {
    assert_eq!(body_box(WorldShape::Planet, [100.0, 100.0], 11.0), [87.5, 87.5, 25.0, 25.0], "a disc of radius 44 in a box of 100");
    let shards = body_box(WorldShape::Shards, [100.0, 100.0], 10.0);
    assert!(close(shards[3], 25.0, 1e-9) && close(shards[2], 25.0 / 1.6, 1e-9));
    assert!(close(shards[1], 100.0 - 12.5, 1e-9), "stands with 80 of 160 above the centre");
    assert_eq!(round_spacing(10.1), 10.0);
    assert_eq!(round_spacing(10.13), 10.25);
}

#[test]
fn lunap_222_2_the_sizes_colours_and_glow_of_a_body() {
    assert_eq!(size_of(WorldShape::Planet, false), (58.0, 58.0));
    assert_eq!(size_of(WorldShape::Planet, true), (84.0, 84.0));
    assert_eq!(size_of(WorldShape::Shards, false), (42.0, 68.0));
    assert_eq!(size_of(WorldShape::Gem, true), (60.0, 96.0));
    assert_eq!((tallness(WorldShape::Planet), tallness(WorldShape::Gem)), (1.0, 1.6));
    assert_eq!(unit_box(WorldShape::Shards), (100.0, 160.0));
    let plain = colours(&WorldBody::PLAIN, 120.0);
    assert_eq!(plain, [Colour::hsl(120.0, 0.85, 0.8, 1.0), Colour::hsl(120.0, 0.65, 0.52, 1.0), Colour::hsl(120.0, 0.6, 0.22, 1.0)]);
    let palette = [Colour::hex("#FFF8FB"), Colour::hex("#F3CFE3"), Colour::hex("#A9789F")];
    let pink = WorldBody { palette: Some(palette), craters: 0.28, ..WorldBody::PLAIN };
    assert_eq!(colours(&pink, 330.0), palette, "a palette outranks the hue");
    let rest = facet_inks(200.0, false);
    assert_eq!(rest[0].a, (0.62f64 * 255.0).round() as u8);
    assert!(facet_inks(200.0, true).iter().all(|c| c.a == 255));
    assert_eq!(glow(40.0, true), (Colour::hsl(40.0, 0.9, 0.7, 1.0), 24.0, 0.75));
    assert_eq!(glow(40.0, false), (Colour::hsl(40.0, 0.9, 0.7, 1.0), 9.0, 0.3));
}

fn bounds_of(shape: &Shape) -> [f64; 4] {
    shape.bounds()
}

#[test]
fn lunap_222_2_a_planets_geometry_is_a_disc_and_its_features_each_at_its_opacity() {
    let plain = geometry(&WorldBody::PLAIN, 0.0, false);
    assert_eq!(plain.len(), 2, "a disc and its highlight");
    let disc = bounds_of(&plain[0]);
    assert!(disc.iter().zip([6.0, 6.0, 88.0, 88.0]).all(|(a, b)| close(*a, b, 1e-9)), "radius 44 at 50, 50: {disc:?}");
    let Some(Paint::Radial { centre, radius, stops }) = &plain[0].fill else { panic!("the disc is lit by a radial gradient") };
    assert!(close(centre[0], 37.68, 1e-9) && close(centre[1], 34.16, 1e-9), "36%, 32% of the disc's box");
    assert!(close(*radius, 65.12, 1e-9));
    assert_eq!(stops.iter().map(|s| s.0).collect::<Vec<_>>(), vec![0.0, 0.48, 1.0]);
    let highlight = bounds_of(&plain[1]);
    assert!(close(highlight[0] + highlight[2] / 2.0, 35.0, 1e-9) && close(highlight[1] + highlight[3] / 2.0, 29.0, 1e-9));

    let nes = WorldBody { craters: 0.28, ..WorldBody::PLAIN };
    let shapes = geometry(&nes, 330.0, false);
    assert_eq!(shapes.len(), 5, "three craters join");
    for (shape, (cx, cy, r)) in shapes[1..4].iter().zip([(63.0, 40.0, 7.0), (40.0, 64.0, 5.0), (67.0, 66.0, 3.5)]) {
        let b = bounds_of(shape);
        assert!(close(b[0] + b[2] / 2.0, cx, 1e-9) && close(b[1] + b[3] / 2.0, cy, 1e-9) && close(b[2], 2.0 * r, 1e-9));
        assert_eq!(shape.opacity, 0.28);
        assert_eq!(shape.fill, Some(Paint::Solid(colours(&nes, 330.0)[2])), "craters in the rim colour");
    }

    let snes = WorldBody { bands: 0.5, ..WorldBody::PLAIN };
    let bands = geometry(&snes, 38.0, false);
    assert_eq!(bands.len(), 4);
    assert_eq!((bands[1].stroke.unwrap().width, bands[2].stroke.unwrap().width), (4.0, 3.0));
    assert!(!bands[1].closed && bands[1].opacity == 0.5);
    assert_eq!(bands[1].points[0], [9.0, 44.0]);
    assert_eq!(*bands[1].points.last().unwrap(), [91.0, 44.0]);

    let n64 = WorldBody { cap: 0.9, moons: 2, ..WorldBody::PLAIN };
    let polar = geometry(&n64, 352.0, true);
    assert_eq!(polar.len(), 5, "a cap and two moonlets join");
    assert_eq!(polar[1].fill, Some(Paint::Solid(Colour::hex("#FFF4F6"))));
    assert_eq!(polar[1].opacity, 0.9);
    let first_moon = bounds_of(&polar[3]);
    assert!(close(first_moon[0] + first_moon[2] / 2.0, 93.0, 1e-9) && close(first_moon[2], 11.0, 1e-9));
    let second_moon = bounds_of(&polar[4]);
    assert!(close(second_moon[0] + second_moon[2] / 2.0, 5.0, 1e-9) && close(second_moon[2], 7.6, 1e-9));
    assert_eq!(polar[3].stroke.unwrap().width, 1.0);

    let ringed = WorldBody { ring: WorldRing::Big, ..WorldBody::PLAIN };
    let ring = geometry(&ringed, 60.0, true);
    assert_eq!(ring.len(), 4, "the ring's back before the disc and its front after");
    assert_eq!((ring[0].opacity, ring[0].stroke.unwrap().width), (0.75, 2.4));
    assert!(ring[0].closed && !ring[2].closed);
    assert!(ring[2].points.iter().all(|p| p[1] >= 52.0 - 64.0f64.to_radians().sin() * 20.0), "the front half is the lower half");
    let subtle = geometry(&WorldBody { ring: WorldRing::Subtle, ..WorldBody::PLAIN }, 60.0, false);
    assert_eq!((subtle[0].opacity, subtle[0].stroke.unwrap().width), (0.45, 1.4));
}

#[test]
fn lunap_222_2_shards_and_a_gem_are_six_facets_in_four_inks_with_an_edge_and_two_sparks() {
    for shape in [WorldShape::Shards, WorldShape::Gem] {
        let body = WorldBody::new(shape);
        let rest = geometry(&body, 0.0, false);
        assert_eq!(rest.len(), 8);
        let inks = facet_inks(0.0, false);
        assert!(rest[..6].iter().all(|s| matches!(&s.fill, Some(Paint::Solid(c)) if inks.contains(c))));
        assert!(rest[..6].iter().all(|s| s.stroke.unwrap().width == 1.2 && s.closed));
        assert_eq!((rest[6].stroke.unwrap().width, rest[7].stroke.unwrap().width), (3.0, 2.0));
        assert_eq!(rest[6].stroke.unwrap().colour.a, 115);
        let focused = geometry(&body, 0.0, true);
        assert_eq!(focused[0].stroke.unwrap().colour, Colour::rgba(255, 255, 255, 242));
        let all = rest.iter().map(bounds_of).fold([f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY], |acc, b| {
            [acc[0].min(b[0]), acc[1].min(b[1]), acc[2].max(b[0] + b[2]), acc[3].max(b[1] + b[3])]
        });
        assert!(all[3] <= 160.0 && all[1] >= 0.0 && all[0] >= 0.0 && all[2] <= 100.0, "inside the 100 by 160 box: {all:?}");
    }
}

#[test]
fn lunap_221_1_the_sky_and_the_stars() {
    assert_eq!(MOONRISE_SKY[1], (0.7, Colour::hex("#0B1029")));
    assert_eq!(SCREENS_SKY[1], (0.55, Colour::hex("#0A0D22")));
    assert_eq!(STAR_COLOUR, Colour::hex("#DFE6FF"));
    assert_eq!(HEIGHT_SHARE, 0.92);
    let mut random = SeededRandom::new(Timeline::MOONRISE.seed);
    let stars = place_stars(&mut random, 220);
    assert_eq!(stars.len(), 220);
    for s in &stars {
        assert!((0.0..1.0).contains(&s.x) && (0.0..0.92).contains(&s.y));
        assert!((0.2..1.4).contains(&s.r) && (0.0..DRAFT_TURN).contains(&s.phase) && (0.6..2.2).contains(&s.rate));
    }
    let first = SeededRandom::first(Timeline::MOONRISE.seed, 5);
    assert_eq!(stars[0].x, first[0]);
    assert_eq!(stars[0].y, first[1] * 0.92);
    assert_eq!(stars[0].r, first[2] * 1.2 + 0.2);
    let still = emusen_kit_core::world::Star { x: 0.5, y: 0.25, r: 1.0, phase: 0.0, rate: 1.0 };
    assert!(close(twinkle(&still, 0.0), 0.55, 1e-12));
    assert!(close(twinkle(&still, std::f64::consts::FRAC_PI_2), 1.0, 1e-12));
    let mut out = Vec::new();
    star_points(&[still], [1280.0, 800.0], 1.0, 0.7, 0.85, 0.0, &mut out);
    assert_eq!(out.len(), 1);
    assert_eq!((out[0].x, out[0].y, out[0].radius), (640.0, 200.0, 0.7));
    assert!(close(out[0].opacity, 0.85 * 0.55, 1e-12));
    star_points(&[still], [1280.0, 800.0], 1.0, 0.7, 0.0, 0.0, &mut out);
    assert!(out.is_empty(), "no opacity, no stars");
}

#[test]
fn lunap_221_2_the_glow_eases_the_shorter_way_round_over_pan() {
    assert_eq!(GLOW_RADII, (756.0, 450.0));
    assert_eq!(GLOW_STRENGTH, 0.2);
    assert_eq!(glow_bounds([0.5, 0.5], GLOW_RADII, 1280.0, 800.0), [640.0 - 756.0, 400.0 - 450.0, 1512.0, 900.0]);
    let stops = glow_stops(200.0, 0.2);
    assert_eq!(stops[0].1.a, 51);
    assert_eq!(stops[1].1.a, (0.07f64 * 255.0).round() as u8);
    assert_eq!((stops[2].1.a, stops[3].1.a), (0, 0));
    assert_eq!(wrap_hue(10.0 - 350.0), 20.0, "350 to 10 goes up 20");
    assert_eq!(wrap_hue(-20.0), -20.0);
    assert_eq!(wrap_hue(190.0), -170.0);

    let mut glow = SkyGlow::new();
    assert!(glow.at(ms(0)).is_none(), "no glow until a hue is set");
    glow.set_hue(Some(30.0), ms(0));
    assert!(glow.at(ms(0)).is_none(), "it fades in from nothing");
    let half = glow.at(ms(160)).expect("fading in");
    assert_eq!(half.hue, 30.0, "the first hue is taken at once");
    assert!(close(half.alpha, 0.2 * Curve::CubicOut.at(0.5), 1e-12));
    assert!(!glow.is_settled(ms(319)) && glow.is_settled(ms(320)));
    assert!(close(glow.at(ms(320)).unwrap().alpha, 0.2, 1e-12));
    glow.set_hue(Some(350.0), ms(320));
    let turning = glow.at(ms(480)).unwrap();
    assert!(close(turning.hue, 360.0 - 5.0, 1e-9), "30 to 350 goes down 40, half way eased is -5: {}", turning.hue);
    assert!(close(glow.at(ms(640)).unwrap().hue, 350.0, 1e-9));
    glow.set_centre([0.25, 0.75], ms(640));
    assert_eq!(glow.at(ms(960)).unwrap().centre, [0.25, 0.75]);
    glow.set_hue(None, ms(960));
    assert!(glow.at(ms(1280)).is_none(), "faded out");
    assert_eq!(glow.shown_hue(ms(1280)), None);
}

#[test]
fn lunap_102_1_a_glide_reads_on_the_hosts_clock_and_never_from_wall_time() {
    let still = Glide::still(4.0);
    assert_eq!(still.value_at(ms(0), false), 4.0);
    assert!(still.is_settled(ms(0), false));
    let g = still.toward(8.0, ms(100), ms(200), Curve::Linear);
    assert_eq!(g.value_at(ms(50), false), 4.0, "before the start");
    assert_eq!(g.value_at(ms(200), false), 6.0);
    assert_eq!(g.value_at(ms(300), false), 8.0);
    assert_eq!(g.value_at(ms(150), true), 8.0, "reduced motion is already there");
    assert!(!g.is_settled(ms(299), false) && g.is_settled(ms(300), false) && g.is_settled(ms(0), true));
    let again = g.toward(0.0, ms(200), ms(100), Curve::QuadraticOut);
    assert_eq!(again.from, 6.0, "a fresh glide starts from where the last one is now");
    assert_eq!(again.value_at(ms(250), false), 6.0 * (1.0 - 0.75));
}

#[test]
fn lunap_219_the_shadow_is_drawn_inside_the_moon_and_the_halo_and_face_are_the_drafts_gradients() {
    let splash = thirteen();
    let distance = |p: [f64; 2], c: [f64; 2]| (p[0] - c[0]).hypot(p[1] - c[1]);
    let mut outline = Vec::new();
    for seconds in [2.0, 2.6, 8.0] {
        let moon = splash.frame(seconds, 1280.0, 720.0).moon;
        moon.shadow_outline(48, &mut outline);
        assert_eq!(outline.len(), 98, "two arcs at {seconds} s");
        for &p in &outline {
            assert!(distance(p, moon.centre) <= moon.radius + 1e-9, "inside the moon at {seconds} s: {p:?}");
            assert!(distance(p, moon.shadow_centre) <= moon.shadow_radius + 1e-9, "inside the shadow's disc at {seconds} s");
        }
        assert!(
            outline.iter().filter(|&&p| (distance(p, moon.centre) - moon.radius).abs() < 1e-9).count() >= 49,
            "half the outline is the moon's edge"
        );
    }
    let moon = splash.frame(8.0, 1280.0, 720.0).moon;
    assert_eq!((moon.halo_inner_radius, moon.halo_radius), (92.0 * 0.6, 92.0 * 3.2));
    assert_eq!(moon.outline_width, 2.2);
    assert_eq!(
        (moon.lit_focus, moon.lit_focus_radius, moon.lit_radius),
        ([640.0 - 92.0 * 0.35, 259.2 - 92.0 * 0.3], 92.0 * 0.1, 92.0 * 1.1)
    );
    let mut far = moon;
    far.shadow_centre = [moon.centre[0] + 300.0, moon.centre[1]];
    far.shadow_outline(48, &mut outline);
    assert!(outline.is_empty(), "discs apart: no shadow");
}
