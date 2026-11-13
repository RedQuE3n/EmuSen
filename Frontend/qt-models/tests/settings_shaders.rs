//! Settings ▸ Display ▸ Shaders and Shader levers, and a console's Shader levers, without Qt (EmuSen_Qt.md §7.7): the
//! shaders a console may draw with as a page of choices, the choice stored in its `ScreenFilter` with the recent list,
//! and the levers of the shader it draws with, as the base reports them, stored in `ShaderParameters`.

use emusen_kit_core::settings::RowKind;
use emusen_qt_models::consoles::Console;
use emusen_qt_models::settings::{self, Model, Outcome, Preview, SettingsSource, SettingsView};
use serde_json::{json, Value};

const ROWS: &str = include_str!("../../../Platform/galaxia/data/frontend_settings.json");

fn f(v: f32) -> u64 {
    u64::from(v.to_bits())
}

fn parameter(id: &str, description: &str, initial: f32, minimum: f32, maximum: f32, step: f32) -> Value {
    json!({"id": id, "description": description, "initial": f(initial), "minimum": f(minimum), "maximum": f(maximum), "step": f(step)})
}

struct Fake {
    graphics: Value,
}

impl SettingsSource for Fake {
    fn rows(&self) -> Result<Value, String> {
        Ok(serde_json::from_str(ROWS).unwrap())
    }
    fn load(&self, model: Model) -> Result<Value, String> {
        Ok(if model == Model::App { json!({}) } else { self.graphics.clone() })
    }
    fn save(&self, _: Model, _: &Value) -> Result<(), String> {
        Ok(())
    }
    fn catalogue(&self) -> Result<Value, String> {
        Ok(json!({
            "filters": [
                {"key": "CRT", "name": "CRT", "consoles": ["NES"], "credit": "a credit", "defaults": {"NES": {"gamma": f(2.2)}},
                 "settings": [
                    {"id": "quality", "description": "Quality", "initial": f(1.0), "minimum": f(0.0), "maximum": f(2.0), "step": f(1.0), "choices": ["Performance", "Balanced", "Accurate"]},
                    {"id": "gamma", "description": "Tube gamma", "initial": f(2.4), "minimum": f(2.0), "maximum": f(2.8), "step": f(0.05), "choices": null}]},
                {"key": "LCD", "name": "Game Boy LCD", "consoles": ["GB"], "settings": []},
                {"key": "effect.None.nearest", "name": "None", "consoles": null, "settings": []}
            ],
            "screens": [{"console": "NES", "kind": "Television"}, {"console": "GB", "kind": "Lcd"}]
        }))
    }
    fn outputs(&self) -> Vec<String> {
        Vec::new()
    }
    fn folders(&self) -> std::collections::BTreeMap<String, String> {
        Default::default()
    }
    fn pack(&self) -> Vec<String> {
        vec!["crt/crt_glow.slangp".into(), "handheld/lcd_grid.slangp".into(), "tint.slangp".into()]
    }
    fn preset_parameters(&self, path: &str) -> Result<Vec<Value>, String> {
        match path {
            "crt/crt_glow.slangp" => Ok(vec![parameter("heading", "--- Glow ---", 0.0, 0.0, 0.0, 0.0), parameter("glow", "Glow strength", 0.5, 0.0, 1.0, 0.0)]),
            "tint.slangp" => Err("tint.slang could not be read".into()),
            _ => Ok(Vec::new()),
        }
    }
}

fn view(graphics: Value) -> SettingsView {
    let mut view = SettingsView::new();
    view.read = settings::read(&Fake { graphics }).unwrap();
    view.read.consoles.consoles = ["NES", "GB"].map(|c| Console { console: c.into(), name: format!("{c} name"), engines: vec!["Engine".into()], ..Console::default() }).to_vec();
    view
}

fn open_display_door(view: &mut SettingsView, id: &str) {
    let display = view.sections().iter().position(|s| s == "Display").unwrap();
    view.choose_section(display);
    let at = index(view, id);
    assert_eq!(view.rows()[at].kind, RowKind::Opens);
    assert_eq!(view.choose(at), Outcome::Opened, "{id} opens a page of Settings");
}

fn index(view: &SettingsView, id: &str) -> usize {
    view.rows().iter().position(|r| r.id == id).unwrap_or_else(|| panic!("no row {id}: {:?}", view.rows().iter().map(|r| r.id.clone()).collect::<Vec<_>>()))
}

fn shown(view: &SettingsView) -> Vec<(String, String, RowKind, bool)> {
    view.rows().into_iter().map(|r| (r.label, r.value, r.kind, r.marked)).collect()
}

#[test]
fn qt_7_7_shaders_lists_a_console_s_recent_built_in_and_pack_shaders_and_stores_the_one_chosen() {
    let mut view = view(json!({"Consoles": {"NES": {"ScreenFilter": "CRT"}}, "RecentShaders": {"NES": ["slang:gone.slangp", "CRT"]}}));
    open_display_door(&mut view, "shaders");
    assert_eq!(view.trail(), ["Display", "Shaders"]);
    // Serenity's screens' consoles, each by its name and the shader it draws with.
    assert_eq!(shown(&view), [("NES name".into(), "CRT".into(), RowKind::Opens, false), ("GB name".into(), "None".into(), RowKind::Opens, false)]);
    assert_eq!(view.pane(0).sentence, "NES name · CRT");
    assert_eq!(view.choose(0), Outcome::Opened);
    assert_eq!(view.trail(), ["Display", "Shaders", "NES name"]);
    assert_eq!(view.help(), emusen_qt_models::help::Screen::SettingsChoices);
    let rows = shown(&view);
    let labels: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
    // A recent shader no longer offered is left out; the GB's LCD is not offered to the NES.
    assert_eq!(labels, ["Recently used", "CRT", "Built into EmuSen", "CRT", "None", "The shader pack", "tint", "crt", "crt glow", "handheld", "lcd grid"]);
    assert_eq!(rows[1], ("CRT".into(), "Default".into(), RowKind::Choice, true), "the one chosen is marked");
    assert!(rows[0].2 == RowKind::Action && view.rows()[0].held, "a heading takes no press");

    assert_eq!(view.pane(index(&view, "shader:slang:crt/crt_glow.slangp")).sentence, "crt/crt_glow.slangp", "a preset's pane names its path in the pack");
    assert_eq!(view.pane(index(&view, "shader:CRT")).sentence, "a credit", "a built-in's its credit");
    // A pack preset chosen: stored by its path in the pack, first of the recent ones, and the settings read again for its levers.
    let glow = index(&view, "shader:slang:crt/crt_glow.slangp");
    let Outcome::All(outcomes) = view.choose(glow) else { panic!("a choice saves and reads again") };
    assert_eq!(outcomes, [Outcome::Saved(Model::Graphics, view.read.graphics.clone()), Outcome::Reread]);
    assert_eq!(view.read.graphics["Consoles"]["NES"]["ScreenFilter"], "slang:crt/crt_glow.slangp");
    assert_eq!(view.read.graphics["RecentShaders"]["NES"], json!(["slang:crt/crt_glow.slangp", "slang:gone.slangp", "CRT"]));
    let marked: Vec<String> = view.rows().into_iter().filter(|r| r.marked).map(|r| r.id).collect();
    assert_eq!(marked, ["recent:slang:crt/crt_glow.slangp", "shader:slang:crt/crt_glow.slangp"], "the mark moves to it");

    // A built-in chosen is stored by its name, and needs no read.
    let none = index(&view, "shader:None");
    assert_eq!(view.choose(none), Outcome::Saved(Model::Graphics, view.read.graphics.clone()));
    assert_eq!(view.read.graphics["Consoles"]["NES"]["ScreenFilter"], "None");
}

#[test]
fn qt_7_7_a_recent_list_keeps_the_newest_five() {
    let mut view = view(json!({"RecentShaders": {"NES": ["CRT", "None", "slang:tint.slangp", "slang:a", "slang:b"]}}));
    open_display_door(&mut view, "shaders");
    view.choose(0);
    view.choose(index(&view, "shader:slang:handheld/lcd_grid.slangp"));
    assert_eq!(view.read.graphics["RecentShaders"]["NES"], json!(["slang:handheld/lcd_grid.slangp", "CRT", "None", "slang:tint.slangp", "slang:a"]));
}

#[test]
fn qt_7_7_a_pack_preset_s_levers_are_its_parameters_each_a_slider_saved_and_set_on_the_picture() {
    let mut view = view(json!({"Consoles": {"NES": {"ScreenFilter": "slang:crt/crt_glow.slangp"}}}));
    open_display_door(&mut view, "levers");
    assert_eq!(view.trail(), ["Display", "Shader levers"]);
    assert_eq!(view.rows()[0].value, "crt glow");
    view.choose(0);
    assert_eq!(view.trail(), ["Display", "Shader levers", "NES name"]);
    // A parameter with no range heads its group; the rest are sliders at the preset's own value.
    assert_eq!(shown(&view), [("--- Glow ---".into(), String::new(), RowKind::Action, false), ("Glow strength".into(), "0.5".into(), RowKind::Slider, false)]);
    assert!(view.rows()[0].held);
    assert_eq!(view.slider(0), None);
    // No step of its own: the kit's hundredth of the range.
    assert_eq!(view.slider(1), Some([0.0, 1.0, 0.01, 0.5]));
    let pane = view.pane(1);
    assert_eq!((pane.sentence.as_str(), pane.preview, pane.caption.as_str()), ("0.5 is the shader's own value.", Preview::Picture, "The last game · live preview"));

    let lever = |value: f32| Outcome::Lever { console: "NES".into(), id: "glow".into(), value };
    let Outcome::All(outcomes) = view.step(1, 1) else { panic!("a step saves and sets the picture") };
    assert_eq!(outcomes, [Outcome::Saved(Model::Graphics, view.read.graphics.clone()), lever(0.51)]);
    assert_eq!(view.read.graphics["ShaderParameters"]["NES"]["slang:crt/crt_glow.slangp"]["glow"], "0.51");
    assert_eq!(view.rows()[1].value, "0.51");
    let Outcome::All(outcomes) = view.slide(1, 7.0) else { panic!("a slide saves") };
    assert_eq!(outcomes[1], lever(1.0), "held inside the range");
    // Back at the shader's own value, the value is forgotten.
    let Outcome::All(outcomes) = view.slide(1, 0.5) else { panic!("a slide saves") };
    assert_eq!(outcomes[1], lever(0.5));
    assert_eq!(view.read.graphics["ShaderParameters"], json!({}), "nothing kept at the shader's own value");
}

#[test]
fn qt_7_7_a_built_in_filter_s_levers_are_its_catalogue_settings_at_the_console_s_own_values() {
    let mut view = view(json!({"Consoles": {"NES": {"ScreenFilter": "CRT"}}, "ShaderParameters": {"NES": {"CRT": {"quality": "2"}}}}));
    open_display_door(&mut view, "levers");
    view.choose(0);
    assert_eq!(shown(&view), [("Quality".into(), "Accurate".into(), RowKind::Steps, false), ("Tube gamma".into(), "2.2".into(), RowKind::Slider, false)]);
    let gamma = view.slider(1).unwrap();
    assert!((gamma[0] - 2.0).abs() < 1e-6 && (gamma[1] - 2.8).abs() < 1e-6 && (gamma[2] - 0.05).abs() < 1e-6 && (gamma[3] - 2.2).abs() < 1e-6, "{gamma:?}");
    assert_eq!(view.pane(1).sentence, "2.2 is the shader's own value.", "the console's own default");
    // A setting of named values steps through them, round.
    let Outcome::All(outcomes) = view.step(0, 1) else { panic!("a step saves") };
    assert_eq!(outcomes[1], Outcome::Lever { console: "NES".into(), id: "quality".into(), value: 0.0 });
    assert_eq!(view.rows()[0].value, "Performance");
    assert_eq!(view.read.graphics["ShaderParameters"]["NES"]["CRT"]["quality"], "0");
}

#[test]
fn qt_7_7_a_shader_without_levers_or_whose_levers_cannot_be_read_says_so() {
    let mut view = view(json!({"Consoles": {"NES": {"ScreenFilter": "slang:tint.slangp"}, "GB": {"ScreenFilter": "Game Boy LCD"}}}));
    open_display_door(&mut view, "levers");
    view.choose(0);
    assert_eq!(shown(&view), [("tint.slang could not be read".into(), String::new(), RowKind::Action, false)]);
    view.back();
    view.choose(1);
    assert_eq!(shown(&view), [("This shader has no levers.".into(), String::new(), RowKind::Action, false)]);
    assert!(view.rows()[0].held);
    assert_eq!(view.choose(0), Outcome::Nothing);
}

#[test]
fn qt_7_7_a_console_s_own_shader_levers_row_opens_its_levers() {
    let mut view = view(json!({"Consoles": {"NES": {"ScreenFilter": "CRT"}}}));
    let consoles = view.sections().iter().position(|s| s == "Consoles").unwrap();
    view.choose_section(consoles);
    view.choose(index(&view, "console:NES"));
    let levers = index(&view, "consolelevers");
    assert!(!view.rows()[levers].held, "the CRT chosen, its levers are offered");
    assert_eq!(view.choose(levers), Outcome::Opened);
    assert_eq!(view.trail(), ["Consoles", "NES name", "Shader levers"]);
    assert_eq!(view.rows().iter().map(|r| r.label.clone()).collect::<Vec<_>>(), ["Quality", "Tube gamma"]);
}

/// The base's own reads, over a pack the test writes: the presets listed by their paths in the pack's folder, a link
/// not followed; a preset's parameters as its passes declare them, its own values over theirs; a pass that cannot be
/// read said in the reader's words.
#[test]
fn qt_7_7_the_pack_and_a_preset_s_parameters_are_read_through_the_platform_library() {
    use emusen_platform_client::galaxia::consts::DIR_SHADERS;
    let _world = emusen_client_fixture::World::new("models-settings-shaders");
    let platform = emusen_platform_client::Platform::load(emusen_client_fixture::library()).unwrap();
    let pack = platform.directory(DIR_SHADERS).unwrap().expect("the pack's folder");
    std::fs::create_dir_all(pack.join("Test/deeper")).unwrap();
    let shader = "#version 450\n#pragma parameter tint \"Tint toward magenta\" 1.0 0.0 1.0 0.05\n#pragma parameter width \"Width\" 2.0 1.0 4.0 0.0\n#pragma stage vertex\nvoid main() {}\n#pragma stage fragment\nvoid main() {}\n";
    std::fs::write(pack.join("Test/tint.slang"), shader).unwrap();
    std::fs::write(pack.join("Test/magenta_tint.slangp"), "shaders = 1\nshader0 = tint.slang\nwidth = 3.0\n").unwrap();
    std::fs::write(pack.join("Test/deeper/Broken.SLANGP"), "shaders = 1\nshader0 = missing.slang\n").unwrap();
    std::fs::write(pack.join("Test/notes.txt"), "not a preset").unwrap();
    std::os::unix::fs::symlink(pack.join("Test"), pack.join("Linked")).unwrap();

    let source = emusen_qt_models::settings_source::ClientSettings::open(&emusen_client_fixture::library().display().to_string()).unwrap();
    assert_eq!(source.pack(), ["Test/deeper/Broken.SLANGP", "Test/magenta_tint.slangp"]);
    let read = source.preset_parameters("Test/magenta_tint.slangp").unwrap();
    assert_eq!(read, [parameter("tint", "Tint toward magenta", 1.0, 0.0, 1.0, 0.05), parameter("width", "Width", 3.0, 1.0, 4.0, 0.0)]);
    let broken = source.preset_parameters("Test/deeper/Broken.SLANGP").unwrap_err();
    assert!(broken.contains("missing.slang"), "the reader's words: {broken}");

    // Read with the settings: the levers of the preset the console draws with.
    let file = platform.file(emusen_platform_client::galaxia::consts::FILE_GRAPHICS).unwrap();
    let graphics = json!({"Consoles": {"NES": {"ScreenFilter": "slang:Test/magenta_tint.slangp"}}});
    platform.model_save(emusen_platform_client::galaxia::consts::MODEL_GRAPHICS_CONFIG, None, file.file_name().unwrap().to_str().unwrap(), &graphics).unwrap();
    let read = settings::read(&source).unwrap();
    assert_eq!(read.pack.len(), 2);
    assert_eq!(read.levers.keys().collect::<Vec<_>>(), ["slang:Test/magenta_tint.slangp"]);
    assert_eq!(read.levers["slang:Test/magenta_tint.slangp"].as_ref().map(Vec::len), Ok(2));
}
