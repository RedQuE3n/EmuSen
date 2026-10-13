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

/// §24.2: "Precise geometry" is the schema's one enhancement, a switch off by default in the Enhancements group, run
/// scope, with its cost, and passes the rules the kit's C3 applies.
#[test]
fn precise_geometry_is_offered_as_an_enhancement_off_by_default() {
    let schema = Diamond::settings_schema();
    let e: Vec<&Setting> = schema.iter().filter(|s| matches!(s.effect, Effect::Enhancement { .. })).collect();
    assert_eq!(e.len(), 1);
    let s = e[0];
    assert!(s.check().is_ok(), "{:?}", s.check());
    let Effect::Enhancement { cost, hardware } = &s.effect else { unreachable!() };
    assert!(matches!(s.kind, SettingKind::Switch));
    assert_eq!((s.key.as_str(), s.label.as_str(), s.default.as_str(), hardware.as_str(), s.scope, s.category.as_deref(), s.restart), (PRECISION_KEY, "Precise geometry", "false", "false", Scope::Run, Some("Enhancements"), false));
    assert!(cost.contains("Vulkan") && cost.contains("save states"));
    assert_eq!(Diamond::CAPABILITIES & (caps::SETTINGS | caps::SETTING_NOTES), caps::SETTINGS | caps::SETTING_NOTES);
}

/// §24.2: turned on and off between frames, the setting changes the picture alone: the machine's state and sound are
/// those of a machine that never had it, and with it on the frame is the renderer's, here a triangle half a pixel lower.
#[test]
fn turned_on_and_off_between_frames_it_changes_only_the_picture() {
    let (mut plain, mut d) = (create(&[]), create(&[]));
    let (mut pa, mut pb) = (vec![0i16; 8192], vec![0i16; 8192]);
    let mut differed = 0;
    for f in 0..12 {
        if f == 3 {
            d.set_settings(&Settings::from_pairs(vec![(PRECISION_KEY.into(), "true".into())])).unwrap();
            if d.pair_ref().is_none() {
                eprintln!("no Vulkan device: {:?}", d.setting_notes());
                return;
            }
        }
        if f == 9 {
            d.set_settings(&Settings::from_pairs(vec![(PRECISION_KEY.into(), "false".into())])).unwrap();
            assert!(d.pair_ref().is_none() && d.setting_notes().is_empty());
        }
        Core::advance(&mut plain, &mut 0).unwrap();
        Core::advance(&mut d, &mut 0).unwrap();
        let (na, nb) = (plain.drain_audio(&mut pa, usize::MAX), d.drain_audio(&mut pb, usize::MAX));
        assert!(pa[..na] == pb[..nb], "frame {f}: the sound differs");
        assert!(accurate(&mut plain) == accurate(&mut d), "frame {f}: the machine differs");
        assert_eq!(d.frame().len(), (d.frame_info().width * d.frame_info().height * 4) as usize);
        if d.frame() != plain.frame() {
            differed += 1;
            assert!((3..9).contains(&f), "frame {f}: the picture differs with the setting off");
        }
    }
    assert!(differed > 0, "the precise triangle drew no pixel otherwise");
}

/// §24.2: where the renderer cannot be made, the setting is accepted, the machine runs as with it off, and the note
/// says why; turned off, the note goes.
#[test]
fn with_no_usable_device_the_setting_is_noted_and_the_console_picture_shown() {
    let (mut plain, mut d) = (create(&[]), create(&[]));
    d.set_precise_on(true, Some("no device has this name"));
    let notes = d.setting_notes();
    assert!(d.pair_ref().is_none() && notes.len() == 1 && notes[0].0 == PRECISION_KEY && notes[0].1.contains("No usable Vulkan"), "{notes:?}");
    for _ in 0..3 {
        Core::advance(&mut plain, &mut 0).unwrap();
        Core::advance(&mut d, &mut 0).unwrap();
    }
    assert!(d.frame() == plain.frame() && accurate(&mut d) == accurate(&mut plain));
    d.set_precise_on(false, None);
    assert!(d.setting_notes().is_empty());
}

/// §24.4: with the setting on the state carries the two blocks, and loads on a machine with it off, which continues
/// as the first; a state made with it off loads with it on.
#[test]
fn a_state_with_the_setting_on_loads_with_it_off_and_the_reverse() {
    let mut on = create(&[(PRECISION_KEY, "true")]);
    if on.pair_ref().is_none() {
        eprintln!("no Vulkan device: {:?}", on.setting_notes());
        return;
    }
    let mut off = create(&[]);
    for _ in 0..5 {
        Core::advance(&mut on, &mut 0).unwrap();
    }
    let s = state(&mut on);
    assert!(s.len() > accurate(&mut on).len() + 4_000_000, "the blocks: {} bytes", s.len());
    off.load_state(&s).unwrap();
    for _ in 0..3 {
        Core::advance(&mut on, &mut 0).unwrap();
        Core::advance(&mut off, &mut 0).unwrap();
    }
    assert!(accurate(&mut on) == state(&mut off));
    let plain = state(&mut off);
    on.load_state(&plain).unwrap();
    Core::advance(&mut on, &mut 0).unwrap();
    Core::advance(&mut off, &mut 0).unwrap();
    assert!(accurate(&mut on) == state(&mut off));
}
