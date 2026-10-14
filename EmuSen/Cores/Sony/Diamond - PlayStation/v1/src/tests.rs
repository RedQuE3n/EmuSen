use super::*;
use emusen_native::core::sys;

/// A program that projects a triangle by RTPT each frame, its third vertex half a pixel down, and draws it shaded from
/// the GTE's words after clearing the screen, then waits for the vertical blank.
const PROGRAM: &str = "lui t9, 0x4000\nmtc0 t9, SR\nlui a0, 0x1F80\nsw zero, 0x1814(a0)\nlui t0, 0x0300\nsw t0, 0x1814(a0)\n\
    ori t0, zero, 0x1000\nctc2 t0, $0\nctc2 t0, $2\nctc2 t0, $4\nlui t0, 160\nctc2 t0, $24\nlui t0, 120\nctc2 t0, $25\nori t0, zero, 0x100\nctc2 t0, $26\n\
    lui t0, 0xE300\nsw t0, 0x1810(a0)\nlui t0, 0xE407\nori t0, t0, 0xFFFF\nsw t0, 0x1810(a0)\nlui t0, 0xE500\nsw t0, 0x1810(a0)\n\
    loop:\nlui t0, 0x0200\nsw t0, 0x1810(a0)\nsw zero, 0x1810(a0)\nlui t0, 0x00F0\nori t0, t0, 0x0140\nsw t0, 0x1810(a0)\n\
    lui t0, 0x0040\nori t0, t0, 0x0080\nmtc2 t0, $0\nori t0, zero, 0x400\nmtc2 t0, $1\nlui t0, 0xFFC0\nori t0, t0, 0x0040\nmtc2 t0, $2\nori t0, zero, 0x800\nmtc2 t0, $3\n\
    lui t0, 0x0005\nori t0, t0, 0xFF90\nmtc2 t0, $4\nori t0, zero, 0x200\nmtc2 t0, $5\nnop\n.word 0x4A280030\n\
    lui t0, 0x3000\nori t0, t0, 0x00FF\nsw t0, 0x1810(a0)\nswc2 $12, 0x1810(a0)\nlui t0, 0x0000\nori t0, t0, 0xFF00\nsw t0, 0x1810(a0)\nswc2 $13, 0x1810(a0)\nlui t0, 0x00FF\nsw t0, 0x1810(a0)\nswc2 $14, 0x1810(a0)\n\
    wait: lw t0, 0x1070(a0)\nnop\nandi t0, t0, 1\nbeq t0, zero, wait\nnop\nlui t0, 0xFFFF\nori t0, t0, 0xFFFE\nsw t0, 0x1070(a0)\nj loop\nnop";

fn create(settings: &[(&str, &str)]) -> Diamond {
    let words = emusen_mips::assemble(PROGRAM, 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = machine::exe::build(&words);
    let mut pairs: Vec<(String, String)> = Diamond::settings_schema().iter().map(|s| (s.key.clone(), s.default.clone())).collect();
    for (k, v) in settings {
        pairs.iter_mut().find(|p| p.0 == *k).unwrap().1 = v.to_string();
    }
    Diamond::create(&Create { image: &image, settings: Settings::from_pairs(pairs), files: Vec::new(), pixel_formats: 1, host_abi_version: sys::ABI_VERSION }).unwrap()
}

fn state(d: &mut Diamond) -> Vec<u8> {
    d.settle();
    let mut s = vec![0; d.state_size()];
    let n = d.save_state(&mut s).unwrap();
    s.truncate(n);
    s
}

/// The accurate part of a state, before any layer's block (Diamond_Enhancements.md §4).
fn accurate(d: &mut Diamond) -> Vec<u8> {
    let s = state(d);
    let end = d.layout().lines().find(|l| l.ends_with(".Tag")).and_then(|l| l.split_whitespace().next()?.parse().ok()).unwrap_or(s.len());
    s[..end].to_vec()
}

/// §42.1: the four enhancements, each a setting in the Enhancements group, run scope, its default the hardware's, with
/// its cost, passing the rules the kit's C3 applies; "Internal resolution" Mars's label and kind of choice.
#[test]
fn the_four_enhancements_are_offered_at_the_hardwares_values() {
    let schema = Diamond::settings_schema();
    let e: Vec<&Setting> = schema.iter().filter(|s| matches!(s.effect, Effect::Enhancement { .. })).collect();
    let keys: Vec<&str> = e.iter().map(|s| s.key.as_str()).collect();
    assert_eq!(keys, [PRECISION_KEY, RESOLUTION_KEY, COLOUR_KEY, FILTERING_KEY]);
    for s in &e {
        assert!(s.check().is_ok(), "{}: {:?}", s.key, s.check());
        let Effect::Enhancement { cost, hardware } = &s.effect else { unreachable!() };
        assert_eq!((s.default.as_str(), s.scope, s.category.as_deref(), s.restart), (hardware.as_str(), Scope::Run, Some("Enhancements"), false), "{}", s.key);
        assert!(cost.contains("Vulkan") && cost.contains("a frame later"), "{}: {cost}", s.key);
    }
    let labels: Vec<&str> = e.iter().map(|s| s.label.as_str()).collect();
    assert_eq!(labels, ["Precise geometry", "Internal resolution", "True colour", "Texture filtering"]);
    let SettingKind::Choice(scales) = &e[1].kind else { panic!("a choice") };
    assert_eq!(scales.iter().map(|c| c.value.as_str()).collect::<Vec<_>>(), ["1", "2", "3", "4"]);
    assert_eq!((e[0].default.as_str(), e[1].default.as_str(), e[2].default.as_str(), e[3].default.as_str()), ("false", "1", "false", "off"));
    assert_eq!(Diamond::CAPABILITIES & (caps::SETTINGS | caps::SETTING_NOTES), caps::SETTINGS | caps::SETTING_NOTES);
    let video = create(&[]).machine_info().video;
    assert_eq!((video.max_width, video.max_height), (640 * 4, 576 * 4), "the largest picture of the largest scale");
}

/// §42.1: each enhancement, and all four, turned on and off between frames changes the picture alone: the machine's
/// state and sound are those of a machine that never had them; at 4x the frame is four times the display.
#[test]
fn turned_on_and_off_between_frames_they_change_only_the_picture() {
    let all = [(PRECISION_KEY, "true"), (RESOLUTION_KEY, "4"), (COLOUR_KEY, "true"), (FILTERING_KEY, "all")];
    let cases: Vec<Vec<(&str, &str)>> = vec![vec![all[0]], vec![all[1]], vec![all[2]], vec![all[3]], all.to_vec()];
    let off = [(PRECISION_KEY, "false"), (RESOLUTION_KEY, "1"), (COLOUR_KEY, "false"), (FILTERING_KEY, "off")];
    for case in cases {
        let (mut plain, mut d) = (create(&[]), create(&[]));
        let (mut pa, mut pb) = (vec![0i16; 8192], vec![0i16; 8192]);
        let mut differed = 0;
        for f in 0..12 {
            if f == 3 {
                d.set_settings(&Settings::from_pairs(case.iter().map(|&(k, v)| (k.into(), v.into())).collect())).unwrap();
                if d.renderer_ref().is_none() {
                    eprintln!("no Vulkan device: {:?}", d.setting_notes());
                    return;
                }
            }
            if f == 9 {
                d.set_settings(&Settings::from_pairs(off.iter().map(|&(k, v)| (k.into(), v.into())).collect())).unwrap();
                assert!(d.renderer_ref().is_none() && d.setting_notes().is_empty());
            }
            Core::advance(&mut plain, &mut 0).unwrap();
            Core::advance(&mut d, &mut 0).unwrap();
            let (na, nb) = (plain.drain_audio(&mut pa, usize::MAX), d.drain_audio(&mut pb, usize::MAX));
            assert!(pa[..na] == pb[..nb], "{case:?} frame {f}: the sound differs");
            assert!(accurate(&mut plain) == accurate(&mut d), "{case:?} frame {f}: the machine differs");
            let fi = d.frame_info();
            assert_eq!(d.frame().len(), (fi.width * fi.height * 4) as usize);
            if (4..9).contains(&f) && case.contains(&(RESOLUTION_KEY, "4")) {
                assert_eq!((fi.width, fi.height), (4 * plain.frame_info().width, 4 * plain.frame_info().height), "{case:?} frame {f}");
            }
            if d.frame() != plain.frame() {
                differed += 1;
                assert!((3..10).contains(&f), "{case:?} frame {f}: the picture differs with the settings off");
            }
        }
        // The program's triangle is untextured, which filtering alone leaves as the console draws it.
        assert!(differed > 0 || case == [(FILTERING_KEY, "all")], "{case:?}: the picture never differed");
    }
}

/// §24.2, §42.1: where the renderer cannot be made, the settings are accepted, the machine runs as with them off, and
/// each one set has the note saying why; turned off, the notes go.
#[test]
fn with_no_usable_device_the_settings_are_noted_and_the_console_picture_shown() {
    let (mut plain, mut d) = (create(&[]), create(&[]));
    d.choose_on(Chosen { precise: true, options: diamond_renderer::Options { scale: 2, ..Default::default() } }, Some("no device has this name"));
    let notes = d.setting_notes();
    assert!(d.renderer_ref().is_none() && notes.len() == 2 && notes[0].0 == PRECISION_KEY && notes[1].0 == RESOLUTION_KEY && notes[0].1.contains("No usable Vulkan"), "{notes:?}");
    for _ in 0..3 {
        Core::advance(&mut plain, &mut 0).unwrap();
        Core::advance(&mut d, &mut 0).unwrap();
    }
    assert!(d.frame() == plain.frame() && accurate(&mut d) == accurate(&mut plain));
    d.choose_on(Chosen::default(), None);
    assert!(d.setting_notes().is_empty());
}

/// §52: with every enhancement on the state is the machine's, the bytes a machine with them off saves at the same frame,
/// and loads both ways; machine info states the bound of settling pictures, 0 with the machine's own picture.
#[test]
fn a_state_with_the_settings_on_is_the_machines_and_machine_info_states_the_bound() {
    let mut on = create(&[(PRECISION_KEY, "true"), (RESOLUTION_KEY, "2"), (COLOUR_KEY, "true"), (FILTERING_KEY, "3d")]);
    let mut off = create(&[]);
    assert_eq!(Core::picture_settles(&off), 0);
    if on.renderer_ref().is_none() {
        eprintln!("no Vulkan device: {:?}", on.setting_notes());
        assert_eq!(Core::picture_settles(&on), 0, "the console's own picture is shown");
        return;
    }
    assert_eq!(Core::picture_settles(&on), SETTLES_TRUE_COLOUR);
    on.choose(Chosen { options: Options { colour: false, ..on.chosen.options }, ..on.chosen });
    assert_eq!(Core::picture_settles(&on), SETTLES);
    for _ in 0..5 {
        Core::advance(&mut on, &mut 0).unwrap();
        Core::advance(&mut off, &mut 0).unwrap();
    }
    let s = state(&mut on);
    assert!(s == state(&mut off), "{} bytes against {}", s.len(), state(&mut off).len());
    off.load_state(&s).unwrap();
    on.load_state(&state(&mut off)).unwrap();
    for _ in 0..3 {
        Core::advance(&mut on, &mut 0).unwrap();
        Core::advance(&mut off, &mut 0).unwrap();
    }
    assert!(state(&mut on) == state(&mut off));
}
