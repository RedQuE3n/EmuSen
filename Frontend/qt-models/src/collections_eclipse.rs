//! Eclipse Collections hub view model (#11) and games.db collection dialog flows.

use crate::game_model::cover_url;
use crate::library::Catalog;
use crate::search_eclipse::{cover_swatch, genre_groups, grid_step, handle_grid_key, help_entry, nav_sections, step_section_action, ECLIPSE_KEYBOARD_LAYOUTS};
use crate::words::games;
use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QStringList};
use emusen_kit_core::input::UiButton;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

const PLUS_ICON: &str = "M12 5v14M5 12h14";

#[derive(Clone, Debug)]
struct CardMeta {
    key: String,
    collection_id: i64,
    name: String,
    count: usize,
    mutable: bool,
}

#[derive(Clone, Debug, Default)]
enum DialogMode {
    #[default]
    None,
    Manage { id: i64, name: String },
    ConfirmDelete { id: i64 },
    Notice,
}

#[derive(Clone, Debug, Default)]
enum TypingMode {
    #[default]
    Create,
    Rename { id: i64 },
}

fn build_covers(paths: &[String], covers: &HashMap<&str, &str>, seed: usize) -> Vec<Value> {
    let found: Vec<String> = paths.iter().filter_map(|p| covers.get(p.as_str()).map(|c| cover_url(c))).take(3).collect();
    let offsets = [0usize, 3, 5];
    (0..3)
        .map(|slot| {
            let url = found.get(slot).map(String::as_str).unwrap_or("");
            cover_swatch(url, seed + offsets[slot])
        })
        .collect()
}

fn ordered_paths(raw: &[String], live: &HashSet<&str>) -> Vec<String> {
    raw.iter().filter(|p| live.contains(p.as_str())).cloned().collect()
}

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("emusen-qt-models/src/library_model.cxxqt.h");
        type LibraryModel = crate::library_model::qobject::LibraryModel;
        include!("emusen-qt-models/src/screens.cxxqt.h");
        type Screens = crate::screens::qobject::Screens;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(*mut LibraryModel, library)]
        #[qproperty(*mut Screens, screens)]
        #[qproperty(i32, focused)]
        #[qproperty(QString, view, READ, NOTIFY)]
        #[qproperty(QString, footer_help, cxx_name = "footerHelp", READ, NOTIFY)]
        #[qproperty(QString, dialog_title, cxx_name = "dialogTitle", READ, NOTIFY)]
        #[qproperty(QString, dialog_message, cxx_name = "dialogMessage", READ, NOTIFY)]
        #[qproperty(QStringList, dialog_choices, cxx_name = "dialogChoices", READ, NOTIFY)]
        #[qproperty(i32, dialog_back_index, cxx_name = "dialogBackIndex", READ, NOTIFY)]
        #[qproperty(bool, dialog_open, cxx_name = "dialogOpen", READ, NOTIFY)]
        #[qproperty(bool, typing_open, cxx_name = "typingOpen", READ, NOTIFY)]
        #[qproperty(QString, typing_title, cxx_name = "typingTitle", READ, NOTIFY)]
        #[qproperty(QString, typing_prompt, cxx_name = "typingPrompt", READ, NOTIFY)]
        #[qproperty(QString, typing_initial, cxx_name = "typingInitial", READ, NOTIFY)]
        #[qproperty(QString, layouts, READ, CONSTANT)]
        type CollectionsEclipseModel = super::CollectionsEclipseModelRust;
    }

    extern "RustQt" {
        #[qinvokable]
        fn refresh(self: Pin<&mut CollectionsEclipseModel>);

        #[qinvokable]
        #[cxx_name = "moveFocus"]
        fn move_focus(self: Pin<&mut CollectionsEclipseModel>, dx: i32, dy: i32);

        #[qinvokable]
        fn activate(self: Pin<&mut CollectionsEclipseModel>) -> QString;

        #[qinvokable]
        fn manage(self: Pin<&mut CollectionsEclipseModel>);

        #[qinvokable]
        #[cxx_name = "dialogChoose"]
        fn dialog_choose(self: Pin<&mut CollectionsEclipseModel>, index: i32);

        #[qinvokable]
        #[cxx_name = "dialogSettled"]
        fn dialog_settled(self: Pin<&mut CollectionsEclipseModel>);

        #[qinvokable]
        #[cxx_name = "finishTyping"]
        fn finish_typing(self: Pin<&mut CollectionsEclipseModel>, kept: bool, text: &QString);

        #[qinvokable]
        #[cxx_name = "showProblem"]
        fn show_problem(self: Pin<&mut CollectionsEclipseModel>, message: &QString);

        #[qinvokable]
        #[cxx_name = "openSlot"]
        fn open_slot(self: Pin<&mut CollectionsEclipseModel>, index: i32);

        #[qinvokable]
        #[cxx_name = "handleAction"]
        fn handle_action(self: Pin<&mut CollectionsEclipseModel>, key: &QString);

        #[qinvokable]
        #[cxx_name = "handleKey"]
        fn handle_key(self: Pin<&mut CollectionsEclipseModel>, key: i32) -> bool;
    }

    impl cxx_qt::Initialize for CollectionsEclipseModel {}
}

pub struct CollectionsEclipseModelRust {
    library: *mut qobject::LibraryModel,
    screens: *mut qobject::Screens,
    focused: i32,
    view: QString,
    footer_help: QString,
    dialog_title: QString,
    dialog_message: QString,
    dialog_choices: QStringList,
    dialog_back_index: i32,
    dialog_open: bool,
    typing_open: bool,
    typing_title: QString,
    typing_prompt: QString,
    typing_initial: QString,
    layouts: QString,
    cards: Vec<CardMeta>,
    dialog_mode: DialogMode,
    typing_mode: TypingMode,
    pending_select_name: Option<String>,
    pending_delete: Option<(i64, String)>,
}

impl Default for CollectionsEclipseModelRust {
    fn default() -> Self {
        CollectionsEclipseModelRust {
            library: std::ptr::null_mut(),
            screens: std::ptr::null_mut(),
            focused: 0,
            view: QString::from("{}"),
            footer_help: QString::from("[]"),
            dialog_title: QString::default(),
            dialog_message: QString::default(),
            dialog_choices: QStringList::default(),
            dialog_back_index: -1,
            dialog_open: false,
            typing_open: false,
            typing_title: QString::default(),
            typing_prompt: QString::default(),
            typing_initial: QString::default(),
            layouts: QString::from(ECLIPSE_KEYBOARD_LAYOUTS),
            cards: Vec::new(),
            dialog_mode: DialogMode::None,
            typing_mode: TypingMode::Create,
            pending_select_name: None,
            pending_delete: None,
        }
    }
}

impl cxx_qt::Initialize for qobject::CollectionsEclipseModel {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut().refresh();
        self.as_mut().on_library_changed(|m| m.refresh()).release();
        self.as_mut().on_focused_changed(|m| m.sync_footer()).release();
    }
}

fn to_qlist(items: &[&str]) -> QStringList {
    let mut list = QStringList::default();
    for item in items {
        list.append(QString::from(*item));
    }
    list
}

fn collect_cards(catalog: &Catalog, metadata: &[(String, crate::esde_shelves::Metadata)]) -> (Vec<(CardMeta, Vec<Value>)>, usize, usize) {
    let live: HashSet<&str> = catalog.games.iter().map(|g| g.path.as_str()).collect();
    let covers: HashMap<&str, &str> = catalog.covers.iter().map(|(p, c)| (p.as_str(), c.as_str())).collect();
    let mut out: Vec<(CardMeta, Vec<Value>)> = Vec::new();
    let mut seed = 0usize;

    let fav_paths = ordered_paths(&catalog.favourites, &live);
    out.push((
        CardMeta { key: crate::library::FAVOURITES.to_owned(), collection_id: -1, name: "Favourites".to_owned(), count: fav_paths.len(), mutable: false },
        build_covers(&fav_paths, &covers, seed),
    ));
    seed += 1;

    for c in &catalog.collections {
        let members = ordered_paths(&c.members, &live);
        out.push((
            CardMeta { key: format!("collection:{}", c.id), collection_id: c.id, name: c.name.clone(), count: members.len(), mutable: true },
            build_covers(&members, &covers, seed),
        ));
        seed += 1;
    }
    let yours_count = out.len();

    let recent_paths = ordered_paths(&catalog.recent, &live);
    out.push((
        CardMeta { key: crate::library::RECENT.to_owned(), collection_id: -1, name: "Recently played".to_owned(), count: recent_paths.len(), mutable: false },
        build_covers(&recent_paths, &covers, seed),
    ));
    seed += 1;

    for (genre, paths) in genre_groups(metadata, &live) {
        out.push((
            CardMeta { key: format!("genre:{genre}"), collection_id: -1, name: genre, count: paths.len(), mutable: false },
            build_covers(&paths, &covers, seed),
        ));
        seed += 1;
    }
    let kept_count = out.len() - yours_count;
    (out, yours_count, kept_count)
}

impl qobject::CollectionsEclipseModel {
    pub fn refresh(mut self: Pin<&mut Self>) {
        let lib = unsafe { self.library.as_ref() };
        let (built, yours_count, kept_count) = if let Some(lib) = lib {
            collect_cards(&lib.view().catalog, lib.effective_metadata())
        } else {
            collect_cards(&Catalog::default(), &[])
        };
        let total = built.len();
        let pending = self.as_mut().rust_mut().pending_select_name.take();
        let mut next_focus = self.focused.clamp(0, total as i32);
        if let Some(name) = pending {
            if let Some(idx) = built.iter().position(|(m, _)| m.mutable && m.name.eq_ignore_ascii_case(&name)) {
                next_focus = idx as i32;
            }
        }
        let mut card_meta = Vec::with_capacity(total);
        let mut cards_json = Vec::with_capacity(total);
        for (i, (meta, covers)) in built.into_iter().enumerate() {
            let group_label = if meta.collection_id >= 0 || meta.key == crate::library::FAVOURITES { "yours" } else { "kept by EmuSen" };
            cards_json.push(json!({
                "index": i,
                "key": meta.key,
                "collectionId": meta.collection_id,
                "name": meta.name,
                "meta": format!("{} · {group_label}", games(meta.count)),
                "mutable": meta.mutable,
                "covers": covers,
                "objectName": format!("collectionCard.{i}"),
            }));
            card_meta.push(meta);
        }
        let payload = json!({
            "navSections": nav_sections("collections"),
            "cards": cards_json,
            "yoursCount": yours_count,
            "keptCount": kept_count,
            "totalCount": total,
            "newCardLabel": "New collection",
            "plusIcon": PLUS_ICON,
            "eraseIcon": crate::search_eclipse::ERASE_ICON,
            "keyPrefix": "collectionKey.",
            "keyRingPrefix": "collectionKeyRing.",
            "keyBgPrefix": "collectionKeyBg.",
            "keyTextPrefix": "collectionKeyText.",
            "actionLines": "previous PageUp\nnext PageDown\nmanage Screensaver\n",
            "typingActionLines": "space Search\ndone Screensaver\n",
        });
        self.as_mut().rust_mut().cards = card_meta;
        if self.focused != next_focus {
            self.as_mut().rust_mut().focused = next_focus;
            self.as_mut().focused_changed();
        }
        let view_str = QString::from(payload.to_string().as_str());
        if self.view != view_str {
            self.as_mut().rust_mut().view = view_str;
            self.as_mut().view_changed();
        }
        self.sync_footer();
    }

    fn sync_footer(mut self: Pin<&mut Self>) {
        let total = self.cards.len();
        let focused = self.focused.clamp(0, total as i32) as usize;
        let on_new = focused >= total;
        let can_manage = self.cards.get(focused).is_some_and(|c| c.mutable);
        let yours_count = self.cards.iter().filter(|c| c.mutable || c.key == crate::library::FAVOURITES).count();
        let kept_count = total.saturating_sub(yours_count);
        let footer_right = if on_new {
            format!("New collection · {yours_count} made by you, {kept_count} kept by EmuSen")
        } else {
            format!("{} of {total} · {yours_count} made by you, {kept_count} kept by EmuSen", focused + 1)
        };
        let help = vec![
            help_entry(UiButton::Accept, if on_new { "Create" } else { "Open" }, true),
            help_entry(UiButton::Screensaver, "Rename or remove", can_manage),
            help_entry(UiButton::Back, "Back", true),
        ];
        let mut parsed: Value = serde_json::from_str(&self.view.to_string()).unwrap_or_else(|_| json!({}));
        parsed["footerRight"] = Value::String(footer_right);
        parsed["focused"] = json!(focused);
        let view_str = QString::from(parsed.to_string().as_str());
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

    pub fn move_focus(mut self: Pin<&mut Self>, dx: i32, dy: i32) {
        let slots = self.cards.len() + 1;
        let next = grid_step(self.focused, slots, 4, dx, dy);
        if next >= 0 && next != self.focused {
            self.as_mut().set_focused(next);
        }
    }

    pub fn activate(mut self: Pin<&mut Self>) -> QString {
        let idx = self.focused.max(0) as usize;
        if idx >= self.cards.len() {
            self.as_mut().rust_mut().typing_mode = TypingMode::Create;
            self.open_typing("New collection", "Give this collection a name.", "");
            return QString::default();
        }
        QString::from(self.cards[idx].key.as_str())
    }

    pub fn manage(mut self: Pin<&mut Self>) {
        let idx = self.focused.max(0) as usize;
        let Some(card) = self.cards.get(idx).cloned() else { return };
        if !card.mutable {
            return;
        }
        let msg = format!("{} in this collection. Renaming or removing it never deletes your games.", games(card.count));
        self.as_mut().rust_mut().dialog_mode = DialogMode::Manage { id: card.collection_id, name: card.name.clone() };
        self.open_dialog(&card.name, &msg, &["Rename", "Remove", "Cancel"], 2);
    }

    pub fn dialog_choose(mut self: Pin<&mut Self>, index: i32) {
        let mode = std::mem::take(&mut self.as_mut().rust_mut().dialog_mode);
        self.as_mut().close_dialog();
        match (mode, index) {
            (DialogMode::Manage { id, name }, 0) => {
                self.as_mut().rust_mut().typing_mode = TypingMode::Rename { id };
                let prompt = format!("Choose a new name for {name}.");
                self.open_typing("Rename collection", &prompt, &name);
            }
            (DialogMode::Manage { id, name }, 1) => {
                self.as_mut().rust_mut().pending_delete = Some((id, name));
            }
            (DialogMode::ConfirmDelete { id }, 0) => {
                if !self.library.is_null() {
                    let lib = unsafe { Pin::new_unchecked(&mut *self.library) };
                    lib.delete_collection(id);
                }
            }
            _ => {}
        }
    }

    pub fn dialog_settled(mut self: Pin<&mut Self>) {
        if let Some((id, name)) = self.as_mut().rust_mut().pending_delete.take() {
            self.as_mut().rust_mut().dialog_mode = DialogMode::ConfirmDelete { id };
            let title = format!("Remove \"{name}\"?");
            self.open_dialog(&title, "Games in this collection stay in your library.", &["Remove", "Keep it"], 1);
        }
    }

    pub fn finish_typing(mut self: Pin<&mut Self>, kept: bool, text: &QString) {
        let mode = std::mem::take(&mut self.as_mut().rust_mut().typing_mode);
        self.as_mut().close_typing();
        let name = text.to_string().trim().to_owned();
        if !kept || name.is_empty() || self.library.is_null() {
            return;
        }
        let lib = unsafe { Pin::new_unchecked(&mut *self.library) };
        match mode {
            TypingMode::Create => {
                self.as_mut().rust_mut().pending_select_name = Some(name.clone());
                lib.create_collection(&QString::from(name.as_str()));
            }
            TypingMode::Rename { id } => {
                self.as_mut().rust_mut().pending_select_name = Some(name.clone());
                lib.rename_collection(id, &QString::from(name.as_str()));
            }
        }
    }

    pub fn show_problem(mut self: Pin<&mut Self>, message: &QString) {
        let msg = message.to_string();
        if msg.is_empty() {
            return;
        }
        self.as_mut().rust_mut().pending_select_name = None;
        self.as_mut().rust_mut().dialog_mode = DialogMode::Notice;
        self.open_dialog("Collections", &msg, &["OK"], 0);
    }

    pub fn open_slot(mut self: Pin<&mut Self>, index: i32) {
        if index >= 0 && index != self.focused {
            self.as_mut().set_focused(index);
        }
        let target = self.as_mut().activate();
        if !target.is_empty() && !self.screens.is_null() {
            unsafe { Pin::new_unchecked(&mut *self.screens) }.open(&target);
        }
    }

    pub fn handle_action(mut self: Pin<&mut Self>, key: &QString) {
        let k = key.to_string();
        if step_section_action(self.screens, &k) {
            return;
        }
        if k == "manage" {
            self.as_mut().manage();
        }
    }

    pub fn handle_key(mut self: Pin<&mut Self>, key: i32) -> bool {
        let screens = self.screens;
        let mut this = self.as_mut();
        match crate::search_eclipse::qt_nav_key(key) {
            Some(crate::search_eclipse::NavKey::Accept) => { this.open_slot(-1); true }
            _ => handle_grid_key(key, screens, |dx, dy| this.as_mut().move_focus(dx, dy), || {}),
        }
    }

    fn open_dialog(mut self: Pin<&mut Self>, title: &str, message: &str, choices: &[&str], back: i32) {
        let mut r = self.as_mut().rust_mut();
        r.dialog_title = QString::from(title);
        r.dialog_message = QString::from(message);
        r.dialog_choices = to_qlist(choices);
        r.dialog_back_index = back;
        r.dialog_open = true;
        self.as_mut().dialog_title_changed();
        self.as_mut().dialog_message_changed();
        self.as_mut().dialog_choices_changed();
        self.as_mut().dialog_back_index_changed();
        self.as_mut().dialog_open_changed();
    }

    fn close_dialog(mut self: Pin<&mut Self>) {
        if self.dialog_open {
            self.as_mut().rust_mut().dialog_open = false;
            self.as_mut().dialog_open_changed();
        }
    }

    fn open_typing(mut self: Pin<&mut Self>, title: &str, prompt: &str, initial: &str) {
        let mut r = self.as_mut().rust_mut();
        r.typing_title = QString::from(title);
        r.typing_prompt = QString::from(prompt);
        r.typing_initial = QString::from(initial);
        r.typing_open = true;
        self.as_mut().typing_title_changed();
        self.as_mut().typing_prompt_changed();
        self.as_mut().typing_initial_changed();
        self.as_mut().typing_open_changed();
    }

    fn close_typing(mut self: Pin<&mut Self>) {
        if self.typing_open {
            self.as_mut().rust_mut().typing_open = false;
            self.as_mut().typing_open_changed();
        }
    }
}
