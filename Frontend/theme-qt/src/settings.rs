//! `Settings`, the Settings screen as a theme reads it, read-only (EmuSen_Qt.md §3.2).

use crate::glue::ffi;
use crate::help::kind_of;
use crate::rows::qobject::ThemeRows;
use crate::view::{Cell, Row};
use core::pin::Pin;
use cxx_qt::casting::Upcast;
use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QString, QStringList, QUrl};
use emusen_kit_core::settings::{Row as WalkRow, RowKind, SliderRange};
use emusen_qt_models::settings::{Pane, SettingsView};
use emusen_qt_models::settings_shown::Form;
use emusen_qt_models::settings;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qlist.h");
        type QList_i32 = cxx_qt_lib::QList<i32>;
        type QList_f64 = cxx_qt_lib::QList<f64>;
        include!("emusen-theme-qt/src/rows.cxxqt.h");
        type ThemeRows = crate::rows::qobject::ThemeRows;
    }

    /// From 1.11: how the boards draw a row's value, `Settings.rows`' `form` (`settings_shown::Form`).
    #[qenum(ThemeSettings)]
    enum SettingsForm {
        Choice,
        Switch,
        Door,
        Fact,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element = "Settings"]
        #[qml_singleton]
        #[qproperty(QStringList, sections, READ, NOTIFY = shown)]
        #[qproperty(i32, section, READ, NOTIFY = shown)]
        #[qproperty(QStringList, trail, READ, NOTIFY = shown)]
        #[qproperty(QString, page, READ, NOTIFY = shown)]
        #[qproperty(*mut ThemeRows, rows, READ, CONSTANT)]
        #[qproperty(i32, focused, READ, NOTIFY = shown)]
        #[qproperty(i32, count, READ, NOTIFY = shown)]
        #[qproperty(i32, help, READ, NOTIFY = shown)]
        #[qproperty(QString, sentence, READ, NOTIFY = shown)]
        #[qproperty(QStringList, lines, READ, NOTIFY = shown)]
        #[qproperty(QStringList, muted, READ, NOTIFY = shown)]
        #[qproperty(QList_i32, tones, READ, NOTIFY = shown)]
        #[qproperty(i32, preview, READ, NOTIFY = shown)]
        #[qproperty(QString, caption, READ, NOTIFY = shown)]
        #[qproperty(f64, meter, READ, NOTIFY = shown)]
        #[qproperty(QStringList, samples, READ, NOTIFY = shown)]
        #[qproperty(QList_f64, sample_scales, cxx_name = "sampleScales", READ, NOTIFY = shown)]
        /// From 1.11 (R-20261123-30): each section's key, in `sections`' order, by which a theme picks a section's picture.
        #[qproperty(QStringList, section_keys, cxx_name = "sectionKeys", READ, NOTIFY = shown)]
        /// From 1.11: the screen's title, the page's heading and the About panel's (the focused row's name), the newest screenshot as a file URL for a picture preview, and whether X takes the focused row back to a default.
        /// From 1.12 (R-20261123-39): every level above the page shown, joined as the boards join them; empty on a section.
        #[qproperty(QString, trail_line, cxx_name = "trailLine", READ, NOTIFY = shown)]
        #[qproperty(QString, title, READ, NOTIFY = shown)]
        #[qproperty(QString, heading, READ, NOTIFY = shown)]
        #[qproperty(QString, about_heading, cxx_name = "aboutHeading", READ, NOTIFY = shown)]
        #[qproperty(QString, shot, READ, NOTIFY = shown)]
        #[qproperty(bool, defaultable, READ, NOTIFY = shown)]
        /// From 1.14 (R-20261116-22): a page's aside toggle chips (Scraping's 11 media kinds), their count, and total items (rows + chips).
        #[qproperty(*mut ThemeRows, chips, READ, CONSTANT)]
        #[qproperty(i32, chip_count, cxx_name = "chipCount", READ, NOTIFY = shown)]
        #[qproperty(i32, total, READ, NOTIFY = shown)]
        /// From 1.15 (boards #27, #28, #57, #29): board-exact heading/aside projections, Consoles grid filter/subtitle/glow, core card, and Firmware file list.
        #[qproperty(QString, board_heading, cxx_name = "boardHeading", READ, NOTIFY = shown)]
        #[qproperty(QString, board_about_heading, cxx_name = "boardAboutHeading", READ, NOTIFY = shown)]
        #[qproperty(QString, board_sentence, cxx_name = "boardSentence", READ, NOTIFY = shown)]
        #[qproperty(QStringList, board_muted, cxx_name = "boardMuted", READ, NOTIFY = shown)]
        #[qproperty(QString, consoles_subtitle, cxx_name = "consolesSubtitle", READ, NOTIFY = shown)]
        #[qproperty(bool, consoles_filtered, cxx_name = "consolesFiltered", READ, NOTIFY = shown)]
        #[qproperty(bool, has_consoles_with_games, cxx_name = "hasConsolesWithGames", READ, NOTIFY = shown)]
        #[qproperty(QString, page_glow, cxx_name = "pageGlow", READ, NOTIFY = shown)]
        #[qproperty(i32, card_planet, cxx_name = "cardPlanet", READ, NOTIFY = shown)]
        #[qproperty(QString, card_title, cxx_name = "cardTitle", READ, NOTIFY = shown)]
        #[qproperty(QString, card_subtitle, cxx_name = "cardSubtitle", READ, NOTIFY = shown)]
        #[qproperty(*mut ThemeRows, firmware_files, cxx_name = "firmwareFiles", READ, CONSTANT)]
        #[qproperty(i32, firmware_count, cxx_name = "firmwareCount", READ, NOTIFY = shown)]
        /// From 1.17 (boards #58, #63, #64): whether row short lines wrap at 1.35 line-height and whether Memory cards' help bar is shown.
        #[qproperty(bool, board_wrap_about, cxx_name = "boardWrapAbout", READ, NOTIFY = shown)]
        #[qproperty(bool, cards_help, cxx_name = "cardsHelp", READ, NOTIFY = shown)]
        /// From 1.18 (boards #52-#56): Eclipse secondary aside lines and rounded-box Start glyph flag.
        #[qproperty(QStringList, board_lines, cxx_name = "boardLines", READ, NOTIFY = shown)]
        #[qproperty(bool, box_menu, cxx_name = "boxMenu", READ, NOTIFY = shown)]
        /// From 1.21 (boards #59-#62): board crumb line, inline kind pills, theme preview card and metadata line.
        #[qproperty(QString, board_trail_line, cxx_name = "boardTrailLine", READ, NOTIFY = shown)]
        #[qproperty(QStringList, inline_pills, cxx_name = "inlinePills", READ, NOTIFY = shown)]
        #[qproperty(i32, inline_pill, cxx_name = "inlinePill", READ, NOTIFY = shown)]
        #[qproperty(bool, theme_preview, cxx_name = "themePreview", READ, NOTIFY = shown)]
        #[qproperty(QString, preview_image, cxx_name = "previewImage", READ, NOTIFY = shown)]
        #[qproperty(QString, board_meta, cxx_name = "boardMeta", READ, NOTIFY = shown)]
        /// From 1.23: Desktop Settings filter, modified-only toggle, trail/tool back state, active tool index, and section badges.
        #[qproperty(QString, filter, READ, NOTIFY = shown)]
        #[qproperty(bool, modified_only, cxx_name = "modifiedOnly", READ, NOTIFY = shown)]
        #[qproperty(bool, can_back, cxx_name = "canBack", READ, NOTIFY = shown)]
        #[qproperty(i32, tool_index, cxx_name = "toolIndex", READ, NOTIFY = shown)]
        #[qproperty(QStringList, section_badges, cxx_name = "sectionBadges", READ, NOTIFY = shown)]
        type ThemeSettings = super::ThemeSettingsRust;
    }

    unsafe extern "RustQt" {
        /// What the screen shows changed: one signal for every member, as the host feeds them together.
        #[qsignal]
        fn shown(self: Pin<&mut ThemeSettings>);
    }

    impl cxx_qt::Initialize for ThemeSettings {}
}

/// `Settings.rows`' roles (EmuSen_Qt.md §3.2, `Settings`; 19..28 added in 1.15; 28..32 added in 1.23).
pub const SETTINGS_ROW_ROLES: [&str; 32] = [
    "key", "label", "about", "value", "kind", "isSwitch", "switchOn", "held", "marked", "code", "minimum", "maximum", "step", "amount", "consoleName",
    "system", "games", "engine", "form", "boardLabel", "boardAbout", "boardValue", "boardForm", "planet", "subtitle", "chip", "chipAttention", "runs",
    "isChoice", "isSlider", "isDoor", "isModified",
];

/// `Settings.chips`' roles (from 1.14, R-20261116-22): Scraping's aside toggle chips and their model row index.
pub const SETTINGS_CHIP_ROLES: [&str; 4] = ["key", "label", "switchOn", "row"];

/// `Settings.firmwareFiles`' roles (from 1.15, board #29): each firmware file's label, status line and state number.
pub const SETTINGS_FIRMWARE_ROLES: [&str; 3] = ["label", "status", "state"];

/// Everything the singleton shows, as the host reads it from the program's model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SettingsFacts {
    pub sections: Vec<String>,
    pub section_keys: Vec<String>,
    pub title: String,
    pub heading: String,
    pub about_heading: String,
    pub defaultable: bool,
    pub section: i32,
    pub trail: Vec<String>,
    pub trail_line: String,
    pub page: String,
    pub rows: Vec<Row>,
    pub chips: Vec<Row>,
    pub firmware_files: Vec<Row>,
    pub focused: i32,
    /// The help bar's kind for what is shown (`Help.entries`' kind), -1 where none is the screen's (a bindings page, listening).
    pub help: i32,
    pub pane: Pane,
    pub board_heading: String,
    pub board_about_heading: String,
    pub board_sentence: String,
    pub board_lines: Vec<String>,
    pub board_muted: Vec<String>,
    pub consoles_subtitle: String,
    pub consoles_filtered: bool,
    pub has_consoles_with_games: bool,
    pub page_glow: String,
    pub card_planet: i32,
    pub card_title: String,
    pub card_subtitle: String,
    pub board_wrap_about: bool,
    pub cards_help: bool,
    pub box_menu: bool,
    pub board_trail_line: String,
    pub inline_pills: Vec<String>,
    pub inline_pill: i32,
    pub theme_preview: bool,
    pub preview_image: String,
    pub board_meta: String,
    pub filter: String,
    pub modified_only: bool,
    pub can_back: bool,
    pub tool_index: i32,
    pub section_badges: Vec<String>,
}

/// The facts of a view whose rows were last told as `rows`; an error where those are not the view's rows.
pub fn facts_of(view: &SettingsView, rows: &[settings::Row], focused: i32) -> Result<SettingsFacts, String> {
    let facets = view.facets();
    if facets.len() != rows.len() {
        return Err(format!("{} rows told and {} in the view", rows.len(), facets.len()));
    }
    let board_rows = view.board_rows(rows, &facets);
    let offset = view.firmware_need_count().min(rows.len());
    let chip_start = view.chip_start().unwrap_or(rows.len()).min(rows.len()).max(offset);
    let (range, unset) = (SliderRange::default(), WalkRow::default());
    let cells = rows[offset..chip_start]
        .iter()
        .zip(&facets[offset..chip_start])
        .zip(&board_rows[offset..chip_start])
        .enumerate()
        .map(|(i, ((r, facet), br))| {
            let slider = if r.kind == RowKind::Slider { view.slider(offset + i) } else { None };
            let [minimum, maximum, step, amount] = slider.unwrap_or([range.minimum, range.maximum, range.step, unset.amount]);
            let tile = facet.tile.clone().unwrap_or_default();
            vec![
                Cell::Text(r.id.clone()),
                Cell::Text(r.label.clone()),
                Cell::Text(facet.about.clone()),
                Cell::Text(r.value.clone()),
                Cell::Whole(r.kind.index() as i32),
                Cell::Flag(facet.switch.is_some()),
                Cell::Flag(facet.switch.unwrap_or(false)),
                Cell::Flag(r.held),
                Cell::Flag(r.marked),
                Cell::Flag(r.code),
                Cell::Number(minimum),
                Cell::Number(maximum),
                Cell::Number(step),
                Cell::Number(amount),
                Cell::Text(tile.console),
                Cell::Text(tile.system),
                Cell::Whole(tile.games as i32),
                Cell::Text(tile.engine),
                Cell::Whole(form_of(Form::of(r, facet.switch)).repr),
                Cell::Text(br.label.clone()),
                Cell::Text(br.about.clone()),
                Cell::Text(br.value.clone()),
                Cell::Whole(form_of(br.form).repr),
                Cell::Whole(br.planet),
                Cell::Text(br.subtitle.clone()),
                Cell::Text(br.chip.clone()),
                Cell::Flag(br.chip_attention),
                Cell::Flag(br.runs),
                Cell::Flag(matches!(r.kind, RowKind::Steps)),
                Cell::Flag(matches!(r.kind, RowKind::Slider)),
                Cell::Flag(matches!(r.kind, RowKind::Opens | RowKind::Asks) || (matches!(r.kind, RowKind::Action) && r.value.is_empty())),
                Cell::Flag(view.row_modified(offset + i)),
            ]
        })
        .collect();
    let chips = rows[chip_start..]
        .iter()
        .zip(&facets[chip_start..])
        .enumerate()
        .map(|(offset, (r, facet))| {
            vec![
                Cell::Text(r.id.clone()),
                Cell::Text(r.label.clone()),
                Cell::Flag(facet.switch.unwrap_or(false)),
                Cell::Whole((chip_start + offset) as i32),
            ]
        })
        .collect();
    let model_focused = (focused.max(offset as i32) as usize).min(rows.len().saturating_sub(1));
    let mut pane = view.pane(focused.max(0) as usize);
    let aside = view.board_aside(model_focused, rows, &board_rows, &pane);
    let firmware_files = aside
        .firmware_files
        .into_iter()
        .map(|f| vec![Cell::Text(f.label), Cell::Text(f.status), Cell::Whole(f.state as i32)])
        .collect();
    let about_heading = if pane.preview == settings::Preview::Chips {
        pane.sentence.clear();
        pane.caption.clone()
    } else {
        usize::try_from(focused).ok().and_then(|f| rows.get(f)).map(|r| r.label.clone()).unwrap_or_default()
    };
    Ok(SettingsFacts {
        sections: view.sections(),
        section_keys: view.section_keys(),
        title: view.title(),
        heading: view.heading(),
        about_heading,
        defaultable: (focused >= 0 || offset > 0) && view.defaultable(model_focused),
        section: view.section as i32,
        trail: view.trail(),
        trail_line: view.trail_line(),
        page: view.page_key(),
        rows: cells,
        chips,
        firmware_files,
        focused: if offset > 0 { (focused - offset as i32).max(0) } else { focused },
        help: kind_of(view.help()).map_or(-1, |k| k.repr),
        pane,
        board_heading: aside.heading,
        board_about_heading: aside.about_heading,
        board_sentence: aside.sentence,
        board_lines: aside.lines,
        board_muted: aside.muted,
        consoles_subtitle: aside.consoles_subtitle,
        consoles_filtered: view.consoles_with_games(),
        has_consoles_with_games: view.has_consoles_with_games(),
        page_glow: aside.page_glow,
        card_planet: aside.card_planet,
        card_title: aside.card_title,
        card_subtitle: aside.card_subtitle,
        board_wrap_about: aside.wrap_about,
        cards_help: aside.cards_help,
        box_menu: aside.box_menu,
        board_trail_line: aside.trail_line,
        inline_pills: aside.inline_pills,
        inline_pill: aside.inline_pill,
        theme_preview: aside.theme_preview,
        preview_image: aside.preview_image,
        board_meta: aside.meta,
        filter: String::new(),
        modified_only: false,
        can_back: !view.trail().is_empty(),
        tool_index: -1,
        section_badges: view.section_badges(),
    })
}

/// A form as the theme's enum gives it.
fn form_of(form: Form) -> qobject::SettingsForm {
    match form {
        Form::Choice => qobject::SettingsForm::Choice,
        Form::Switch => qobject::SettingsForm::Switch,
        Form::Door => qobject::SettingsForm::Door,
        Form::Fact => qobject::SettingsForm::Fact,
    }
}

fn list(words: &[String]) -> QStringList {
    let mut out = QList::<QString>::default();
    words.iter().for_each(|w| out.append(QString::from(w.as_str())));
    QStringList::from(&out)
}

pub struct ThemeSettingsRust {
    sections: QStringList,
    section_keys: QStringList,
    title: QString,
    heading: QString,
    about_heading: QString,
    shot: QString,
    defaultable: bool,
    section: i32,
    trail: QStringList,
    trail_line: QString,
    page: QString,
    rows: *mut ThemeRows,
    chips: *mut ThemeRows,
    firmware_files: *mut ThemeRows,
    focused: i32,
    count: i32,
    chip_count: i32,
    firmware_count: i32,
    total: i32,
    help: i32,
    sentence: QString,
    lines: QStringList,
    muted: QStringList,
    tones: QList<i32>,
    preview: i32,
    caption: QString,
    meter: f64,
    samples: QStringList,
    sample_scales: QList<f64>,
    board_heading: QString,
    board_about_heading: QString,
    board_sentence: QString,
    board_lines: QStringList,
    board_muted: QStringList,
    consoles_subtitle: QString,
    consoles_filtered: bool,
    has_consoles_with_games: bool,
    page_glow: QString,
    card_planet: i32,
    card_title: QString,
    card_subtitle: QString,
    board_wrap_about: bool,
    cards_help: bool,
    box_menu: bool,
    board_trail_line: QString,
    inline_pills: QStringList,
    inline_pill: i32,
    theme_preview: bool,
    preview_image: QString,
    board_meta: QString,
    filter: QString,
    modified_only: bool,
    can_back: bool,
    tool_index: i32,
    section_badges: QStringList,
    last: SettingsFacts,
}

impl Default for ThemeSettingsRust {
    fn default() -> Self {
        ThemeSettingsRust {
            sections: QStringList::default(),
            section_keys: QStringList::default(),
            title: QString::default(),
            heading: QString::default(),
            about_heading: QString::default(),
            shot: QString::default(),
            defaultable: false,
            section: 0,
            trail: QStringList::default(),
            trail_line: QString::default(),
            page: QString::default(),
            rows: std::ptr::null_mut(),
            chips: std::ptr::null_mut(),
            firmware_files: std::ptr::null_mut(),
            focused: 0,
            count: 0,
            chip_count: 0,
            firmware_count: 0,
            total: 0,
            help: -1,
            sentence: QString::default(),
            lines: QStringList::default(),
            muted: QStringList::default(),
            tones: QList::default(),
            preview: 0,
            caption: QString::default(),
            meter: 0.0,
            samples: QStringList::default(),
            sample_scales: QList::default(),
            board_heading: QString::default(),
            board_about_heading: QString::default(),
            board_sentence: QString::default(),
            board_lines: QStringList::default(),
            board_muted: QStringList::default(),
            consoles_subtitle: QString::default(),
            consoles_filtered: false,
            has_consoles_with_games: false,
            page_glow: QString::default(),
            card_planet: -3,
            card_title: QString::default(),
            card_subtitle: QString::default(),
            board_wrap_about: false,
            cards_help: false,
            box_menu: false,
            board_trail_line: QString::default(),
            inline_pills: QStringList::default(),
            inline_pill: -1,
            theme_preview: false,
            preview_image: QString::default(),
            board_meta: QString::default(),
            filter: QString::default(),
            modified_only: false,
            can_back: false,
            tool_index: -1,
            section_badges: QStringList::default(),
            last: SettingsFacts { help: -1, card_planet: -3, inline_pill: -1, tool_index: -1, ..SettingsFacts::default() },
        }
    }
}

impl cxx_qt::Initialize for qobject::ThemeSettings {
    fn initialize(mut self: Pin<&mut Self>) {
        let list = ffi::new_rows(self.as_mut().upcast_pin());
        // Safety: the list was just made, a child of this singleton.
        if let Some(rows) = unsafe { list.as_mut() } {
            unsafe { Pin::new_unchecked(rows) }.set_roles(&SETTINGS_ROW_ROLES);
        }
        let chips_list = ffi::new_rows(self.as_mut().upcast_pin());
        // Safety: the chips list was just made, a child of this singleton.
        if let Some(chips) = unsafe { chips_list.as_mut() } {
            unsafe { Pin::new_unchecked(chips) }.set_roles(&SETTINGS_CHIP_ROLES);
        }
        let firmware_list = ffi::new_rows(self.as_mut().upcast_pin());
        // Safety: the firmware list was just made, a child of this singleton.
        if let Some(fw) = unsafe { firmware_list.as_mut() } {
            unsafe { Pin::new_unchecked(fw) }.set_roles(&SETTINGS_FIRMWARE_ROLES);
        }
        let mut me = self.as_mut().rust_mut();
        me.rows = list;
        me.chips = chips_list;
        me.firmware_files = firmware_list;
    }
}

impl qobject::ThemeSettings {
    /// The host's feed; nothing is told where nothing changed.
    pub fn show(mut self: Pin<&mut Self>, facts: &SettingsFacts) {
        if self.last == *facts {
            return;
        }
        // Safety: the lists are the singleton's children, made in `initialize`.
        if let Some(rows) = unsafe { self.rows.as_mut() } {
            unsafe { Pin::new_unchecked(rows) }.replace(facts.rows.clone());
        }
        if let Some(chips) = unsafe { self.chips.as_mut() } {
            unsafe { Pin::new_unchecked(chips) }.replace(facts.chips.clone());
        }
        if let Some(fw) = unsafe { self.firmware_files.as_mut() } {
            unsafe { Pin::new_unchecked(fw) }.replace(facts.firmware_files.clone());
        }
        let pane = &facts.pane;
        let mut tones = QList::<i32>::default();
        pane.tones.iter().for_each(|t| tones.append(*t as i32));
        let mut me = self.as_mut().rust_mut();
        me.sections = list(&facts.sections);
        me.section_keys = list(&facts.section_keys);
        me.title = QString::from(facts.title.as_str());
        me.heading = QString::from(facts.heading.as_str());
        me.about_heading = QString::from(facts.about_heading.as_str());
        me.shot = if pane.shot.is_empty() { QString::default() } else { QUrl::from_local_file(&QString::from(pane.shot.as_str())).to_qstring() };
        me.defaultable = facts.defaultable;
        me.section = facts.section;
        me.trail = list(&facts.trail);
        me.trail_line = QString::from(facts.trail_line.as_str());
        me.page = QString::from(facts.page.as_str());
        me.focused = facts.focused;
        me.count = facts.rows.len() as i32;
        me.chip_count = facts.chips.len() as i32;
        me.firmware_count = facts.firmware_files.len() as i32;
        me.total = (facts.rows.len() + facts.chips.len()) as i32;
        me.help = facts.help;
        me.sentence = QString::from(pane.sentence.as_str());
        me.lines = list(&pane.lines);
        me.muted = list(&pane.muted);
        me.tones = tones;
        me.preview = pane.preview.number();
        me.caption = QString::from(pane.caption.as_str());
        me.meter = pane.meter;
        me.samples = list(&pane.samples.iter().map(|(s, _)| s.clone()).collect::<Vec<_>>());
        let mut scales = QList::<f64>::default();
        pane.samples.iter().for_each(|(_, scale)| scales.append(*scale));
        me.sample_scales = scales;
        me.board_heading = QString::from(facts.board_heading.as_str());
        me.board_about_heading = QString::from(facts.board_about_heading.as_str());
        me.board_sentence = QString::from(facts.board_sentence.as_str());
        me.board_lines = list(&facts.board_lines);
        me.board_muted = list(&facts.board_muted);
        me.consoles_subtitle = QString::from(facts.consoles_subtitle.as_str());
        me.consoles_filtered = facts.consoles_filtered;
        me.has_consoles_with_games = facts.has_consoles_with_games;
        me.page_glow = QString::from(facts.page_glow.as_str());
        me.card_planet = facts.card_planet;
        me.card_title = QString::from(facts.card_title.as_str());
        me.card_subtitle = QString::from(facts.card_subtitle.as_str());
        me.board_wrap_about = facts.board_wrap_about;
        me.cards_help = facts.cards_help;
        me.box_menu = facts.box_menu;
        me.board_trail_line = QString::from(facts.board_trail_line.as_str());
        me.inline_pills = list(&facts.inline_pills);
        me.inline_pill = facts.inline_pill;
        me.theme_preview = facts.theme_preview;
        me.preview_image = QString::from(facts.preview_image.as_str());
        me.board_meta = QString::from(facts.board_meta.as_str());
        me.filter = QString::from(facts.filter.as_str());
        me.modified_only = facts.modified_only;
        me.can_back = facts.can_back;
        me.tool_index = facts.tool_index;
        me.section_badges = list(&facts.section_badges);
        me.last = facts.clone();
        drop(me);
        self.shown();
    }

    /// The facts last shown, for the host's tests.
    pub fn shown_facts(&self) -> &SettingsFacts {
        &self.last
    }
}
