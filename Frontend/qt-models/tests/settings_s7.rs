//! Unit tests for Settings boards #59 (Shader levers), #60 (Shader parameters), #61 (Themes) and #62 (Scraping account and run).

use emusen_kit_core::settings::RowKind;
use emusen_qt_models::consoles::Console;
use emusen_qt_models::help;
use emusen_qt_models::settings::{self, Model, Outcome, Page, Preview, SettingsSource, SettingsView};
use emusen_qt_models::settings_controllers::Typed;
use emusen_qt_models::settings_scrape::ScrapeState;
use emusen_qt_models::themes::{self, Kind};
use serde_json::{json, Value};

const ROWS: &str = include_str!("../../../Platform/galaxia/data/frontend_settings.json");

fn f(v: f32) -> u64 {
    u64::from(v.to_bits())
}

fn param(id: &str, description: &str, initial: f32, minimum: f32, maximum: f32, step: f32, choices: Option<&[&str]>) -> Value {
    json!({"id": id, "description": description, "initial": f(initial), "minimum": f(minimum), "maximum": f(maximum), "step": f(step), "choices": choices})
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
            "filters": [{
                "key": "CRT", "name": "CRT", "consoles": ["SNES"], "credit": "Serenity", "defaults": {"SNES": {"curvature": f(0.12), "glare": f(0.2)}},
                "settings": [
                    param("quality", "Quality", 1.0, 0.0, 2.0, 1.0, Some(&["Performance", "Balanced", "Accurate"])),
                    param("maskDepth", "Mask depth", 0.6, 0.0, 1.0, 0.05, None),
                    param("signal", "Signal", 1.0, 0.0, 3.0, 1.0, Some(&["Composite", "S-Video", "RGB", "RF"])),
                    param("standard", "Television standard", 2.0, 0.0, 2.0, 1.0, Some(&["NTSC", "PAL", "The game's"])),
                    param("curvature", "Curvature", 0.12, 0.0, 0.5, 0.02, None),
                    param("level", "Picture brightness", 0.0, 0.0, 1.0, 1.0, Some(&["Bright", "The tube's own"])),
                    param("mask", "Mask", 0.0, 0.0, 3.0, 1.0, Some(&["Aperture grille", "Shadow mask", "Slot mask", "Off"])),
                    param("glare", "Glare in the glass", 0.2, 0.0, 1.0, 0.05, None),
                    param("gamma", "Tube gamma", 2.4, 2.0, 2.8, 0.05, None),
                ]
            }],
            "screens": [{"console": "SNES", "kind": "Television"}]
        }))
    }
    fn outputs(&self) -> Vec<String> {
        Vec::new()
    }
    fn folders(&self) -> std::collections::BTreeMap<String, String> {
        Default::default()
    }
    fn pack(&self) -> Vec<String> {
        vec!["crt/CRT_Royale.slangp".into()]
    }
    fn preset_parameters(&self, path: &str) -> Result<Vec<Value>, String> {
        if path == "crt/CRT_Royale.slangp" {
            Ok(vec![
                param("scanline_weight", "Scanline weight", 0.3, 0.0, 1.0, 0.05, None),
                param("mask_strength", "Mask strength", 0.5, 0.0, 1.0, 0.05, None),
                param("curvature", "Curvature", 0.0, 0.0, 1.0, 1.0, None),
                param("brightness_boost", "Brightness boost", 1.0, 0.5, 2.0, 0.05, None),
            ])
        } else {
            Ok(Vec::new())
        }
    }
}

fn view_for(graphics: Value, themed: bool) -> SettingsView {
    let mut v = SettingsView::new();
    v.themed_settings = themed;
    v.read = settings::read(&Fake { graphics }).unwrap();
    v.read.consoles.consoles = vec![Console { console: "SNES".into(), name: "Super Nintendo".into(), engines: vec!["snes9x".into()], ..Console::default() }];
    v
}

fn aside_of(v: &SettingsView, focused: usize) -> emusen_qt_models::settings_shown::BoardAside {
    let rows = v.rows();
    let facets = v.facets();
    let board_rows = v.board_rows(&rows, &facets);
    let pane = v.pane(focused);
    v.board_aside(focused, &rows, &board_rows, &pane)
}

#[test]
fn s7_59_levers_orders_board_seven_first_formats_percent_and_resets_keeping_quality() {
    let g = json!({
        "Consoles": {"SNES": {"ScreenFilter": "CRT"}},
        "ShaderParameters": {"SNES": {"CRT": {"quality": "2", "curvature": "0.24", "maskDepth": "0.8"}}}
    });
    let mut v = view_for(g, true);
    let display = v.sections().iter().position(|s| s == "Display").unwrap();
    v.choose_section(display);
    let levers_idx = v.rows().iter().position(|r| r.id == "levers").unwrap();
    assert_eq!(v.choose(levers_idx), Outcome::Opened);
    assert_eq!(v.choose(0), Outcome::Opened);
    assert_eq!(v.page(), Some(&Page::Levers("SNES".into())));

    let labels: Vec<String> = v.rows().into_iter().map(|r| r.label).collect();
    assert_eq!(
        labels,
        ["Signal", "Television standard", "Curvature", "Picture brightness", "Mask", "Mask depth", "Glare in the glass", "Quality", "Tube gamma", "Reset to the tier's values"]
    );
    let rows = v.rows();
    let board_rows = v.board_rows(&rows, &v.facets());
    assert_eq!(board_rows[2].value, "24%");
    assert_eq!(board_rows[5].value, "80%");
    assert_eq!(board_rows[6].value, "20%");
    assert_eq!(board_rows[8].value, "2.4");

    let aside = aside_of(&v, 2);
    assert_eq!(aside.trail_line, "Display");
    assert_eq!(aside.heading, "Shader levers");
    assert_eq!(v.pane(2).preview, Preview::Picture);
    assert!(aside.sentence.starts_with("Curvature at 24%."));

    // Stepping Curvature live emits Lever; Reset restores levers while preserving quality=2.
    let Outcome::All(step) = v.step(2, 1) else { panic!("expected saved + lever") };
    assert!(matches!(step[1], Outcome::Lever { ref id, .. } if id == "curvature"));
    let reset_idx = v.rows().len() - 1;
    let Outcome::All(reset) = v.choose(reset_idx) else { panic!("expected reset outcomes") };
    assert!(matches!(reset[0], Outcome::Saved(Model::Graphics, _)));
    assert_eq!(v.read.graphics["ShaderParameters"]["SNES"]["CRT"], json!({"quality": "2"}));
    let reset_rows = v.rows();
    assert_eq!(v.board_rows(&reset_rows, &v.facets())[2].value, "12%");
}

#[test]
fn s7_60_shader_params_supports_search_bool_toggle_and_reset_all() {
    let g = json!({
        "Consoles": {"SNES": {"ScreenFilter": "slang:crt/CRT_Royale.slangp"}},
        "ShaderParameters": {"SNES": {"slang:crt/CRT_Royale.slangp": {"scanline_weight": "0.45", "curvature": "1", "brightness_boost": "1.2"}}}
    });
    let mut v = view_for(g, true);
    let display = v.sections().iter().position(|s| s == "Display").unwrap();
    v.choose_section(display);
    let shaders_idx = v.rows().iter().position(|r| r.id == "shaders").unwrap();
    v.choose(shaders_idx);
    v.choose(0);
    let params_idx = v.rows().iter().position(|r| r.id == "shaders:params").unwrap();
    assert_eq!(v.choose(params_idx), Outcome::Opened);
    assert_eq!(v.page(), Some(&Page::ShaderParams("SNES".into())));

    let rows = v.rows();
    assert_eq!(rows[0].label, "Search parameters");
    assert_eq!(rows[0].value, "…");
    assert_eq!(rows[1].label, "Reset all");
    assert_eq!(rows[2].label, "Scanline weight");
    assert_eq!(rows[2].value, "0.45");
    assert_eq!(rows[4].label, "Curvature");
    assert_eq!(rows[4].value, "On");
    assert_eq!(rows[5].value, "1.2");

    // Search filters by name case-insensitively and clears cleanly.
    assert!(matches!(v.choose(0), Outcome::Type { typed: Typed::SearchParameters, .. }));
    assert_eq!(v.typed(&Typed::SearchParameters, "mask"), Outcome::Shown);
    assert_eq!(v.rows().len(), 3);
    assert_eq!(v.rows()[2].label, "Mask strength");
    assert_eq!(v.typed(&Typed::SearchParameters, ""), Outcome::Shown);
    assert_eq!(v.rows().len(), 6);

    // Toggling bool parameter flips On -> Off; Reset all clears stored overrides.
    let Outcome::All(toggled) = v.choose(4) else { panic!("bool toggle saves and emits lever") };
    assert_eq!(toggled[1], Outcome::Lever { console: "SNES".into(), id: "curvature".into(), value: 0.0 });
    assert_eq!(v.rows()[4].value, "Off");
    let Outcome::All(reset) = v.choose(1) else { panic!("reset all saves and emits levers") };
    assert!(matches!(reset[0], Outcome::Saved(Model::Graphics, _)));
    assert_eq!(v.read.graphics["ShaderParameters"], json!({}));
    assert_eq!(v.rows()[2].value, "0.3");
}

#[test]
fn s7_61_themes_page_lists_real_themes_switches_kind_and_emits_use_and_check() {
    let mut v = view_for(json!({}), true);
    let eclipse = include_str!("../../themes/eclipse/theme.json");
    let reference = include_str!("../../themes/emusen/theme.json");
    v.read.app[themes::SETTING] = json!("eclipse");
    v.themes = Some(themes::listing(&v.read.app, None, &[("emusen", reference), ("eclipse", eclipse)]));
    let library = v.sections().iter().position(|s| s == "Library").unwrap();
    v.choose_section(library);
    let themes_idx = v.rows().iter().position(|r| r.id == "themes").unwrap();
    assert_eq!(v.choose(themes_idx), Outcome::Opened);
    assert_eq!(v.page(), Some(&Page::Themes));

    let rows = v.rows();
    assert_eq!(rows.len(), 2);
    assert_eq!((rows[0].label.as_str(), rows[0].value.as_str()), ("EmuSen", "Checked"));
    assert_eq!((rows[1].label.as_str(), rows[1].value.as_str()), ("Eclipse", "In use"));
    v.focused_row = 0;
    assert_eq!(v.help(), help::Screen::Themes);
    v.focused_row = 1;
    assert_eq!(v.help(), help::Screen::ThemesChecked);
    assert_eq!(v.choose(0), Outcome::UseTheme("emusen".into()));
    assert_eq!(v.choose(1), Outcome::Nothing);
    assert_eq!(v.check_focused_theme(1), Outcome::ThemeCheck("eclipse".into()));

    // Shoulder and inline pill switch between EmuSen and ES-DE kinds.
    assert!(v.shoulder(1));
    assert_eq!(v.theme_kind, Kind::Esde);
    assert_eq!(aside_of(&v, 0).inline_pill, 1);
    assert!(v.choose_inline_pill(0));
    assert_eq!(v.theme_kind, Kind::EmuSen);
}

#[test]
fn s7_62_scrape_run_projects_six_rows_two_step_login_and_actions() {
    let mut v = view_for(json!({}), true);
    v.scrape = ScrapeState { queued: 12, ..ScrapeState::default() };
    let library = v.sections().iter().position(|s| s == "Library").unwrap();
    v.choose_section(library);
    let idx = v.rows().iter().position(|r| r.id == "scraperun").unwrap();
    assert_eq!(v.choose(idx), Outcome::All(vec![Outcome::Opened, Outcome::ScrapeRefresh]));
    assert_eq!(v.page(), Some(&Page::ScrapeRun));

    let rows = v.rows();
    let labels: Vec<&str> = rows.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(labels, ["Member account", "Log in", "Resume", "Cancel scraping", "Status", "Scraper"]);
    assert_eq!(rows[0].value, "Not signed in");
    assert_eq!(rows[2].value, "12 games queued");
    assert_eq!(rows[0].kind, RowKind::Opens);
    assert_eq!(aside_of(&v, 0).sentence, "Not signed in: runs use EmuSen's developer credentials alone.");

    // Two-step Log in: username -> password -> ScrapeSignIn.
    assert!(matches!(v.choose(1), Outcome::Type { typed: Typed::ScrapeUser, .. }));
    assert!(matches!(v.typed(&Typed::ScrapeUser, "alice"), Outcome::Type { typed: Typed::ScrapePassword, .. }));
    assert_eq!(v.typed(&Typed::ScrapePassword, "secret"), Outcome::ScrapeSignIn { user: "alice".into(), password: "secret".into() });
    assert_eq!(v.choose(2), Outcome::ScrapeResume);
    assert_eq!(v.choose(3), Outcome::ScrapeCancel);
    assert_eq!(v.choose(4), Outcome::ScrapeRefresh);
    assert_eq!(v.choose(5), Outcome::Door("scraper".into()));

    // Signed-in state flips Log in -> Sign out and Member account -> ScrapeCheck.
    v.scrape.signed_in = true;
    v.scrape.account_name = "alice".into();
    assert_eq!(v.rows()[0].value, "alice");
    assert_eq!(v.rows()[1].label, "Sign out");
    assert_eq!(v.choose(0), Outcome::ScrapeCheck);
    assert_eq!(v.choose(1), Outcome::ScrapeSignOut);
}
