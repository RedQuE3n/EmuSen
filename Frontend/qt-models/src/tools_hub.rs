//! Eclipse Tools hub view model (#9) routing to Cores, Scraper, Packages, Library Manager, Themes, and Memory cards.

use crate::search_eclipse::{grid_step, handle_grid_key, help_entry, help_pair, nav_sections, step_section_action};
use crate::words::{counted, games, grouped};
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QModelIndex, QString};
use emusen_kit_core::input::UiButton;
use serde_json::{json, Value};
use std::collections::HashSet;

const TOOL_SPECS: [(&str, &str, &str); 6] = [
    ("cores", "Cores", "M7 7h10v10H7zM10 3v4M14 3v4M10 17v4M14 17v4M3 10h4M3 14h4M17 10h4M17 14h4"),
    ("scraper", "Scraper", "M4 5h16v14H4zM4 15l5-5 4 4 3-3 4 4M15 9.500h.01"),
    ("packages", "Packages", "M12 3l8 4.500v9L12 21l-8-4.500v-9zM4 7.500l8 4.500 8-4.500M12 12v9"),
    ("library", "Library Manager", "M5 4h10l4 4v12H5zM15 4v4h4M9 13l2 2 4-4"),
    ("themes", "Themes", "M12 3a9 9 0 1 0 0 18c1.500 0 2-1 2-2s-.800-1.800 0-2.800 4 .300 5.500-1.700A9 9 0 0 0 12 3zM7.500 11h.01M10 7.500h.01M14.500 7.500h.01"),
    ("cards", "Memory cards", "M7 3h10v18H7zM10 3v5M14 3v5M10 14h4"),
];

fn summary_str<'a>(tools: &'a Value, key: &str, field: &str) -> Option<&'a str> {
    tools.get(key).and_then(|v| v.get(field)).and_then(Value::as_str).filter(|s| !s.is_empty())
}

fn find_settings_row(settings: &crate::settings_model::qobject::SettingsModel, target_key: &str) -> Option<i32> {
    let count = settings.row_count(&QModelIndex::default());
    for row in 0..count {
        let idx = settings.index(row, 0, &QModelIndex::default());
        if settings.data(&idx, 6).value::<QString>().is_some_and(|k| k.to_string() == target_key) {
            return Some(row);
        }
    }
    None
}

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("emusen-qt-models/src/library_model.cxxqt.h");
        type LibraryModel = crate::library_model::qobject::LibraryModel;
        include!("emusen-qt-models/src/settings_model.cxxqt.h");
        type SettingsModel = crate::settings_model::qobject::SettingsModel;
        include!("emusen-qt-models/src/screens.cxxqt.h");
        type Screens = crate::screens::qobject::Screens;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(*mut LibraryModel, library)]
        #[qproperty(*mut SettingsModel, settings)]
        #[qproperty(*mut Screens, screens)]
        #[qproperty(i32, focused)]
        #[qproperty(QString, view, READ, NOTIFY)]
        #[qproperty(QString, footer_help, cxx_name = "footerHelp", READ, NOTIFY)]
        type ToolsHubModel = super::ToolsHubModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        fn refresh(self: Pin<&mut ToolsHubModel>);

        #[qinvokable]
        #[cxx_name = "moveFocus"]
        fn move_focus(self: Pin<&mut ToolsHubModel>, dx: i32, dy: i32);

        #[qinvokable]
        #[cxx_name = "openTool"]
        fn open_tool(self: Pin<&mut ToolsHubModel>, index: i32);

        #[qinvokable]
        #[cxx_name = "handleAction"]
        fn handle_action(self: Pin<&mut ToolsHubModel>, key: &QString);

        #[qinvokable]
        #[cxx_name = "handleKey"]
        fn handle_key(self: Pin<&mut ToolsHubModel>, key: i32) -> bool;
    }

    impl cxx_qt::Initialize for ToolsHubModel {}
}

pub struct ToolsHubModelRust {
    library: *mut qobject::LibraryModel,
    settings: *mut qobject::SettingsModel,
    screens: *mut qobject::Screens,
    focused: i32,
    view: QString,
    footer_help: QString,
}

impl Default for ToolsHubModelRust {
    fn default() -> Self {
        ToolsHubModelRust {
            library: std::ptr::null_mut(),
            settings: std::ptr::null_mut(),
            screens: std::ptr::null_mut(),
            focused: 0,
            view: QString::from("{}"),
            footer_help: QString::from("[]"),
        }
    }
}

impl cxx_qt::Initialize for qobject::ToolsHubModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().refresh();
        self.as_mut().on_library_changed(|m| m.refresh()).release();
        self.as_mut().on_settings_changed(|m| m.refresh()).release();
        self.as_mut().on_focused_changed(|m| m.sync_focus()).release();
    }
}

impl qobject::ToolsHubModel {
    pub fn move_focus(mut self: Pin<&mut Self>, dx: i32, dy: i32) {
        let next = grid_step(self.focused, TOOL_SPECS.len(), 3, dx, dy);
        if next >= 0 && next != self.focused {
            self.as_mut().set_focused(next);
        }
    }

    pub fn open_tool(mut self: Pin<&mut Self>, index: i32) {
        let target = if index < 0 { self.focused.clamp(0, TOOL_SPECS.len() as i32 - 1) } else { index.clamp(0, TOOL_SPECS.len() as i32 - 1) };
        if self.focused != target {
            self.as_mut().set_focused(target);
        }
        let Some((key, _, _)) = TOOL_SPECS.get(target as usize).copied() else { return };
        if self.screens.is_null() {
            return;
        }
        let screens = unsafe { Pin::new_unchecked(&mut *self.screens) };
        if key != "cards" {
            screens.act(&QString::from(key));
            return;
        }
        if !self.settings.is_null() {
            let mut settings = unsafe { Pin::new_unchecked(&mut *self.settings) };
            let chosen_sys = {
                let consoles = &settings.view().read.consoles.consoles;
                consoles
                    .iter()
                    .find(|c| c.runs() && c.media.iter().any(|m| !m.cards.is_empty()))
                    .or_else(|| consoles.iter().find(|c| c.runs() && !c.media.is_empty()))
                    .map(|c| c.systems.first().cloned().unwrap_or_else(|| c.console.clone()))
            };
            if let Some(sys) = chosen_sys {
                if settings.as_mut().open_console(&QString::from(sys.as_str())) {
                    if let Some(row) = find_settings_row(&settings, "cards") {
                        settings.as_mut().choose(row);
                    }
                }
            } else {
                let lib_sec = settings.view().sections().iter().position(|s| s.eq_ignore_ascii_case("library")).unwrap_or(0) as i32;
                settings.as_mut().set_section(lib_sec);
                if let Some(row) = find_settings_row(&settings, "saves") {
                    settings.as_mut().choose(row);
                }
            }
        }
        screens.act(&QString::from(crate::home_card::SETTINGS));
    }

    pub fn handle_action(self: Pin<&mut Self>, key: &QString) {
        step_section_action(self.screens, &key.to_string());
    }

    pub fn handle_key(mut self: Pin<&mut Self>, key: i32) -> bool {
        let screens = self.screens;
        let mut this = self.as_mut();
        match crate::search_eclipse::qt_nav_key(key) {
            Some(crate::search_eclipse::NavKey::Accept) => { this.open_tool(-1); true }
            _ => handle_grid_key(key, screens, |dx, dy| this.as_mut().move_focus(dx, dy), || {}),
        }
    }

    fn sync_focus(mut self: Pin<&mut Self>) {
        let focused = self.focused.clamp(0, TOOL_SPECS.len() as i32 - 1);
        let mut parsed: Value = serde_json::from_str(&self.view.to_string()).unwrap_or_else(|_| json!({}));
        parsed["focused"] = json!(focused);
        let view_str = QString::from(parsed.to_string().as_str());
        if self.view != view_str {
            self.as_mut().rust_mut().view = view_str;
            self.as_mut().view_changed();
        }
    }

    pub fn refresh(mut self: Pin<&mut Self>) {
        let lib = unsafe { self.library.as_ref() };
        let settings = unsafe { self.settings.as_ref() };
        let tools_json = lib
            .and_then(|l| serde_json::from_str::<Value>(&l.tool_summaries().to_string()).ok())
            .unwrap_or(Value::Null);

        let cards: Vec<Value> = TOOL_SPECS
            .iter()
            .enumerate()
            .map(|(i, (key, name, icon))| {
                let (line, badge) = self.describe_tool(key, &tools_json, lib, settings);
                json!({
                    "index": i,
                    "key": key,
                    "name": name,
                    "line": line,
                    "badge": badge,
                    "icon": icon,
                    "objectName": format!("toolCard.{i}"),
                    "badgeName": format!("toolBadge.{i}"),
                })
            })
            .collect();

        let help = vec![
            help_entry(UiButton::Accept, "Open", true),
            help_entry(UiButton::Back, "Back", true),
            help_pair(UiButton::PageUp, UiButton::PageDown, "Sections", true),
        ];
        let payload = json!({
            "navSections": nav_sections("tools"),
            "title": "Tools",
            "subtitle": "Each one says what it would do before it does it.",
            "focused": self.focused.clamp(0, TOOL_SPECS.len() as i32 - 1),
            "cards": cards,
            "actionLines": "previous PageUp\nnext PageDown\n",
        });
        let view_str = QString::from(payload.to_string().as_str());
        let help_str = QString::from(Value::Array(help).to_string().as_str());
        if self.view != view_str {
            self.as_mut().rust_mut().view = view_str;
            self.as_mut().view_changed();
        }
        if self.footer_help != help_str {
            self.as_mut().rust_mut().footer_help = help_str;
            self.as_mut().footer_help_changed();
        }
    }

    fn describe_tool(
        &self,
        key: &str,
        tools: &Value,
        lib: Option<&crate::library_model::qobject::LibraryModel>,
        settings: Option<&crate::settings_model::qobject::SettingsModel>,
    ) -> (String, String) {
        let explicit_badge = summary_str(tools, key, "badge").map(str::to_owned);
        let explicit_line = summary_str(tools, key, "sentence").map(str::to_owned);
        let attention = tools.get(key).and_then(|v| v.get("attention")).and_then(Value::as_bool).unwrap_or(false);
        let fallback = crate::home_card::tool_words(key).map(|(_, d)| d).unwrap_or_default();
        match key {
            "cores" => {
                let installed = tools
                    .get(key)
                    .and_then(|v| v.get("installed"))
                    .and_then(Value::as_u64)
                    .map(|n| n as usize)
                    .unwrap_or_else(|| settings.map_or(0, |s| s.view().read.consoles.cores.len()));
                let updates = tools.get(key).and_then(|v| v.get("updates")).and_then(Value::as_u64).unwrap_or(0) as usize;
                let badge = explicit_badge.unwrap_or_else(|| {
                    if updates > 0 {
                        counted(updates, "update", "updates")
                    } else if attention {
                        summary_str(tools, key, "brief").unwrap_or_default().to_owned()
                    } else {
                        String::new()
                    }
                });
                let line = explicit_line.unwrap_or_else(|| {
                    if installed > 0 && updates > 0 {
                        format!("{} installed. {} have a newer build.", counted(installed, "core", "cores"), updates)
                    } else if let Some(brief) = summary_str(tools, key, "brief") {
                        brief.to_owned()
                    } else if installed > 0 {
                        format!("{} installed.", counted(installed, "core", "cores"))
                    } else {
                        fallback.to_owned()
                    }
                });
                (line, badge)
            }
            "scraper" => {
                let line = explicit_line.unwrap_or_else(|| {
                    let Some(lib) = lib else { return fallback.to_owned() };
                    let catalog = &lib.view().catalog;
                    let total = catalog.games.len();
                    if total == 0 {
                        return fallback.to_owned();
                    }
                    let covers: HashSet<&str> = catalog.covers.iter().map(|(p, _)| p.as_str()).collect();
                    let meta_paths: HashSet<&str> = lib
                        .effective_metadata()
                        .iter()
                        .filter(|(_, m)| !m.is_empty())
                        .map(|(p, _)| p.as_str())
                        .collect();
                    let have = catalog.games.iter().filter(|g| covers.contains(g.path.as_str()) || meta_paths.contains(g.path.as_str())).count();
                    format!("{} of {} have art and facts.", grouped(have), games(total))
                });
                (line, explicit_badge.unwrap_or_default())
            }
            "packages" => {
                let line = explicit_line.or_else(|| summary_str(tools, key, "brief").map(str::to_owned)).unwrap_or_else(|| fallback.to_owned());
                (line, explicit_badge.unwrap_or_default())
            }
            "library" => {
                let badge = explicit_badge.unwrap_or_else(|| {
                    if !attention {
                        return String::new();
                    }
                    if let Some(n) = tools.get(key).and_then(|v| v.get("issues")).and_then(Value::as_u64) {
                        return format!("{n} to check");
                    }
                    let brief = summary_str(tools, key, "brief").unwrap_or_default();
                    let lead = brief.split_whitespace().next().and_then(|w| w.parse::<u64>().ok());
                    lead.map_or_else(|| "Needs a check".to_owned(), |n| format!("{n} to check"))
                });
                let line = explicit_line.or_else(|| summary_str(tools, key, "brief").map(str::to_owned)).unwrap_or_else(|| fallback.to_owned());
                (line, badge)
            }
            "themes" => {
                let line = explicit_line.unwrap_or_else(|| {
                    let Some(listing) = settings.and_then(|s| s.view().themes.as_ref()) else {
                        return "Eclipse is on.".to_owned();
                    };
                    let active = listing.rows.iter().find(|r| r.current || r.id == listing.chosen).map_or("Eclipse", |r| r.name.as_str());
                    let count = listing.rows.len();
                    if count > 0 {
                        format!("{active} is on. {} installed.", counted(count, "theme", "themes"))
                    } else {
                        format!("{active} is on.")
                    }
                });
                (line, explicit_badge.unwrap_or_default())
            }
            _ => {
                let line = explicit_line.unwrap_or_else(|| {
                    let Some(s) = settings else { return "Manage console memory cards and game saves.".to_owned() };
                    let card_consoles: Vec<_> = s.view().read.consoles.consoles.iter().filter(|c| c.runs() && !c.media.is_empty()).collect();
                    let cards: usize = card_consoles.iter().map(|c| c.media.iter().map(|m| m.cards.len()).sum::<usize>()).sum();
                    let with_cards = card_consoles.iter().filter(|c| c.media.iter().any(|m| !m.cards.is_empty())).count();
                    if cards > 0 {
                        format!("{} across {}.", counted(cards, "card", "cards"), counted(with_cards, "console", "consoles"))
                    } else if !card_consoles.is_empty() {
                        format!("{} with memory cards.", counted(card_consoles.len(), "console", "consoles"))
                    } else {
                        "Manage console memory cards and game saves.".to_owned()
                    }
                });
                (line, explicit_badge.unwrap_or_default())
            }
        }
    }
}
