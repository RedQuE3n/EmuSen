//! `ToolModel`: one tool's screen on Qt for the kit's `ToolScreen` (EmuSen_Qt.md §1.2.2): `Tool` over the source the
//! `tool` property names, its words, rows and columns as properties and plain-data lists for `RowsModel`, its cards and
//! its question, and the pad's and the pointer's asks as invokables. Jobs are pulled by `poll`, which the screen's timer
//! calls while `busy`; nothing is pushed.

use crate::tools::cores::backend::{Backend, Clients, Scripted};
use crate::tools::cores::{Cores, Host};
use crate::tools::{Source, Tool};
use core::pin::Pin;
use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::{QList, QString, QStringList};
use serde_json::{json, Value};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
    }

    /// The tools a screen shows.
    #[qenum(ToolModel)]
    enum Tool {
        NoTool,
        Cores,
        /// A tool of jobs, named by `jobs` (tools/jobs.rs).
        Jobs,
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(Tool, tool)]
        /// The tool of jobs shown, by its key, while `tool` is Jobs.
        #[qproperty(QString, jobs)]
        #[qproperty(QString, platform_library, cxx_name = "platformLibrary")]
        /// The runtime's cores folder, as the library's; empty for the installation's own.
        #[qproperty(QString, cores)]
        /// A file holding a world in place of the back end's, as the recordings script theirs; empty for the back end.
        #[qproperty(QString, world)]
        /// Whether the screen is shown: it reads everything anew each time this turns on.
        #[qproperty(bool, open)]
        #[qproperty(bool, busy, READ, NOTIFY)]
        /// Why the back end could not be opened; empty when it was.
        #[qproperty(QString, problem, READ, NOTIFY)]
        #[qproperty(QString, title, READ, NOTIFY)]
        #[qproperty(QString, facts, READ, NOTIFY)]
        #[qproperty(QStringList, sections, READ, NOTIFY)]
        #[qproperty(QString, section, READ, NOTIFY)]
        #[qproperty(QString, columns, READ, NOTIFY)]
        #[qproperty(QString, rows, READ, NOTIFY)]
        #[qproperty(i32, current, READ, NOTIFY)]
        #[qproperty(QString, search_text, cxx_name = "searchText", READ, NOTIFY)]
        /// The search box takes words from the docked keyboard.
        #[qproperty(bool, searching, READ, NOTIFY)]
        /// While searching, the keys hold the focus; the shoulders give it to the rows and back.
        #[qproperty(bool, search_keys, cxx_name = "searchKeys", READ, NOTIFY)]
        #[qproperty(QString, search_hint, cxx_name = "searchHint", READ, CONSTANT)]
        #[qproperty(QString, apply_label, cxx_name = "applyLabel", READ, CONSTANT)]
        #[qproperty(QString, pane_title, cxx_name = "paneTitle", READ, NOTIFY)]
        #[qproperty(QString, pane_caption, cxx_name = "paneCaption", READ, NOTIFY)]
        #[qproperty(QString, sentences, READ, NOTIFY)]
        #[qproperty(QString, summary, READ, NOTIFY)]
        #[qproperty(i32, summary_tone, cxx_name = "summaryTone", READ, NOTIFY)]
        #[qproperty(f64, progress, READ, NOTIFY)]
        #[qproperty(bool, apply_enabled, cxx_name = "applyEnabled", READ, NOTIFY)]
        #[qproperty(f64, meter, READ, NOTIFY)]
        #[qproperty(QString, meter_words, cxx_name = "meterWords", READ, NOTIFY)]
        /// The help bar's entries (`button`, `label`, `pair`) and the screen's declared actions.
        #[qproperty(QString, help, READ, NOTIFY)]
        #[qproperty(QString, action_lines, cxx_name = "actionLines", READ, NOTIFY)]
        /// The card over the screen: a row's page or the List card.
        #[qproperty(bool, card_open, cxx_name = "cardOpen", READ, NOTIFY)]
        #[qproperty(QString, card_title, cxx_name = "cardTitle", READ, NOTIFY)]
        #[qproperty(QString, card_breadcrumb, cxx_name = "cardBreadcrumb", READ, NOTIFY)]
        #[qproperty(QString, card_rows, cxx_name = "cardRows", READ, NOTIFY)]
        #[qproperty(i32, card_current, cxx_name = "cardCurrent", READ, NOTIFY)]
        #[qproperty(QStringList, card_pane_lines, cxx_name = "cardPaneLines", READ, NOTIFY)]
        #[qproperty(QStringList, card_pane_muted, cxx_name = "cardPaneMuted", READ, NOTIFY)]
        /// The question over the screen: the plan, or one the tool asks.
        #[qproperty(bool, asking, READ, NOTIFY)]
        #[qproperty(QString, question_title, cxx_name = "questionTitle", READ, NOTIFY)]
        #[qproperty(QStringList, question_sentences, cxx_name = "questionSentences", READ, NOTIFY)]
        #[qproperty(QString, question_caption, cxx_name = "questionCaption", READ, NOTIFY)]
        #[qproperty(QStringList, question_choices, cxx_name = "questionChoices", READ, NOTIFY)]
        #[qproperty(i32, question_back, cxx_name = "questionBack", READ, NOTIFY)]
        type ToolModel = super::ToolModelRust;
    }

    extern "RustQt" {
        /// The list's focus moved to a row.
        #[qinvokable]
        fn focus(self: Pin<&mut ToolModel>, index: i32);
        /// A on a row.
        #[qinvokable]
        fn activate(self: Pin<&mut ToolModel>, index: i32);
        /// The strip's section chosen by its label.
        #[qinvokable]
        #[cxx_name = "chooseSection"]
        fn choose_section(self: Pin<&mut ToolModel>, label: &QString);
        #[qinvokable]
        fn search(self: Pin<&mut ToolModel>, words: &QString);
        /// The screen's declared action by its key: X's page, Select's List card, Start's plan.
        #[qinvokable]
        fn act(self: Pin<&mut ToolModel>, key: &QString);
        /// While searching, the focus went to the keys (true) or the rows (false) by other means than the shoulders: the pointer.
        #[qinvokable]
        #[cxx_name = "keysFocused"]
        fn keys_focused(self: Pin<&mut ToolModel>, keys: bool);
        /// The docked keyboard closed, its words kept.
        #[qinvokable]
        #[cxx_name = "endSearch"]
        fn end_search(self: Pin<&mut ToolModel>);
        /// Start, or a click on Apply: the plan.
        #[qinvokable]
        fn apply(self: Pin<&mut ToolModel>);
        /// The question answered by the choice's index; -1 for none.
        #[qinvokable]
        fn answer(self: Pin<&mut ToolModel>, index: i32);
        #[qinvokable]
        #[cxx_name = "cardFocus"]
        fn card_focus(self: Pin<&mut ToolModel>, index: i32);
        #[qinvokable]
        #[cxx_name = "cardChoose"]
        fn card_choose(self: Pin<&mut ToolModel>, index: i32);
        #[qinvokable]
        #[cxx_name = "cardStep"]
        fn card_step(self: Pin<&mut ToolModel>, index: i32, by: i32);
        #[qinvokable]
        #[cxx_name = "cardBack"]
        fn card_back(self: Pin<&mut ToolModel>);
        /// The running job's events taken, as the screen's timer asks.
        #[qinvokable]
        fn poll(self: Pin<&mut ToolModel>);
        /// A card or the docked keyboard over the screen closed, so the list takes the keys back.
        #[qsignal]
        #[cxx_name = "listFocused"]
        fn list_focused(self: Pin<&mut ToolModel>);
        /// The docked keyboard opened over the screen, so it takes the keys.
        #[qsignal]
        #[cxx_name = "searchOpened"]
        fn search_opened(self: Pin<&mut ToolModel>);
        /// A destination of the host's, by its key (a console's settings, its firmware).
        #[qsignal]
        #[cxx_name = "hostAsked"]
        fn host_asked(self: Pin<&mut ToolModel>, key: QString);
        /// A folder to show in the system's file manager, through the program's one action for it (`Folders`).
        #[qsignal]
        #[cxx_name = "openFolder"]
        fn open_folder(self: Pin<&mut ToolModel>, path: QString);
    }

    impl cxx_qt::Initialize for ToolModel {}
    impl cxx_qt::Threading for ToolModel {}
}

use qobject::Tool as Shown;

pub struct ToolModelRust {
    tool: Shown,
    jobs: QString,
    platform_library: QString,
    cores: QString,
    world: QString,
    open: bool,
    busy: bool,
    problem: QString,
    title: QString,
    facts: QString,
    sections: QStringList,
    section: QString,
    columns: QString,
    rows: QString,
    current: i32,
    search_text: QString,
    searching: bool,
    search_keys: bool,
    search_hint: QString,
    apply_label: QString,
    pane_title: QString,
    pane_caption: QString,
    sentences: QString,
    summary: QString,
    summary_tone: i32,
    progress: f64,
    apply_enabled: bool,
    meter: f64,
    meter_words: QString,
    help: QString,
    action_lines: QString,
    card_open: bool,
    card_title: QString,
    card_breadcrumb: QString,
    card_rows: QString,
    card_current: i32,
    card_pane_lines: QStringList,
    card_pane_muted: QStringList,
    asking: bool,
    question_title: QString,
    question_sentences: QStringList,
    question_caption: QString,
    question_choices: QStringList,
    question_back: i32,
    shown: Option<Tool<Box<dyn Source>>>,
    /// The key of the tool of jobs shown, and the others opened before, kept with their running jobs and last runs.
    shown_jobs: String,
    parked: std::collections::HashMap<String, Tool<Box<dyn Source>>>,
    /// While the properties are put, the list's own moves (a reset's, or following `current`) are not the player's.
    filling: bool,
}

impl Default for ToolModelRust {
    fn default() -> Self {
        ToolModelRust {
            tool: Shown::NoTool,
            jobs: QString::default(),
            platform_library: QString::default(),
            cores: QString::default(),
            world: QString::default(),
            open: false,
            busy: false,
            problem: QString::default(),
            title: QString::default(),
            facts: QString::from("[]"),
            sections: QStringList::default(),
            section: QString::default(),
            columns: QString::from("[]"),
            rows: QString::from("[]"),
            current: -1,
            search_text: QString::default(),
            searching: false,
            search_keys: false,
            search_hint: QString::from(crate::tools::SEARCH_HINT),
            apply_label: QString::from(crate::tools::APPLY_LABEL),
            pane_title: QString::default(),
            pane_caption: QString::default(),
            sentences: QString::from("[]"),
            summary: QString::default(),
            summary_tone: 0,
            progress: -1.0,
            apply_enabled: false,
            meter: 0.0,
            meter_words: QString::default(),
            help: QString::from("[]"),
            action_lines: QString::default(),
            card_open: false,
            card_title: QString::default(),
            card_breadcrumb: QString::default(),
            card_rows: QString::from("[]"),
            card_current: 0,
            card_pane_lines: QStringList::default(),
            card_pane_muted: QStringList::default(),
            asking: false,
            question_title: QString::default(),
            question_sentences: QStringList::default(),
            question_caption: QString::default(),
            question_choices: QStringList::default(),
            question_back: -1,
            shown: None,
            shown_jobs: String::new(),
            parked: std::collections::HashMap::new(),
            filling: false,
        }
    }
}

impl cxx_qt::Initialize for qobject::ToolModel {
    fn initialize(self: Pin<&mut Self>) {
        self.on_open_changed(|model| model.opened()).release();
    }
}

fn list(words: &[String]) -> QStringList {
    let items: QList<QString> = words.iter().map(|w| QString::from(w.as_str())).collect();
    QStringList::from(&items)
}

fn text(value: &Value) -> QString {
    QString::from(value.to_string().as_str())
}

/// The Cores source over the back end, or over the world a file holds.
fn cores_source(platform_library: &str, cores: &str, world: &str) -> Result<Box<dyn Source>, String> {
    let backend: Box<dyn Backend> = if world.is_empty() {
        let platform = emusen_platform_client::Platform::load(platform_library).map_err(|e| e.to_string())?;
        let config = crate::clients::runtime_config(&platform, cores)?;
        let api = emusen_crystal_client::Api::load(platform_library).map_err(|e| e.to_string())?;
        let runtime = crate::clients::runtime(&api, &config)?.0;
        Box::new(Clients { runtime, platform })
    } else {
        Box::new(Scripted::open(world)?)
    };
    Ok(Box::new(Cores::new(backend, Host::default())))
}

/// The tool of jobs of `key`, its jobs started by the platform library's tools, or from the world a file holds.
fn jobs_source(platform_library: &str, world: &str, key: &str) -> Result<Box<dyn Source>, String> {
    let starter: crate::tools::jobs::Starter = if world.is_empty() {
        let platform = emusen_platform_client::Platform::load(platform_library).map_err(|e| e.to_string())?;
        Box::new(move |request| crate::tools::cores::backend::platform_job(&platform, request))
    } else {
        let scripted = Scripted::open(world)?;
        Box::new(move |request| scripted.start(request))
    };
    crate::tools::jobs::Jobs::new(key, starter).map(|j| Box::new(j) as Box<dyn Source>).ok_or_else(|| format!("{key} is no tool of jobs"))
}

macro_rules! put {
    ($model:expr, $field:ident, $value:expr, $signal:ident) => {{
        let value = $value;
        if $model.$field != value {
            $model.as_mut().rust_mut().$field = value;
            $model.as_mut().$signal();
        }
    }};
}

impl qobject::ToolModel {
    fn opened(mut self: Pin<&mut Self>) {
        if !self.open {
            return;
        }
        let jobs = self.jobs.to_string();
        if self.tool == Shown::Jobs && self.shown_jobs != jobs {
            let mut rust = self.as_mut().rust_mut();
            let was = std::mem::replace(&mut rust.shown_jobs, jobs.clone());
            if let Some(tool) = rust.shown.take() {
                rust.parked.insert(was, tool);
            }
            rust.shown = rust.parked.remove(&jobs);
        }
        if self.shown.is_none() {
            let made = match self.tool {
                Shown::Cores => cores_source(&self.platform_library.to_string(), &self.cores.to_string(), &self.world.to_string()),
                Shown::Jobs => jobs_source(&self.platform_library.to_string(), &self.world.to_string(), &jobs),
                _ => Err("no tool is named".to_owned()),
            };
            match made {
                Ok(source) => self.as_mut().rust_mut().shown = Some(Tool::new(source)),
                Err(words) => put!(self, problem, QString::from(words.as_str()), problem_changed),
            }
        }
        if let Some(tool) = self.as_mut().rust_mut().shown.as_mut() {
            tool.refresh();
        }
        self.fill();
    }

    /// A change asked by QML, made after the event that asked, so one press is never answered by the card it closed and
    /// the screen under it, and no signal of a fill calls back into the model while it fills (as `Screens` does).
    fn with(self: Pin<&mut Self>, change: impl FnOnce(&mut Tool<Box<dyn Source>>) -> Option<String> + Send + 'static) {
        let _ = self.qt_thread().queue(move |model| model.change(change));
    }

    fn change(mut self: Pin<&mut Self>, change: impl FnOnce(&mut Tool<Box<dyn Source>>) -> Option<String>) {
        let host = {
            let mut rust = self.as_mut().rust_mut();
            let Some(tool) = rust.shown.as_mut() else { return };
            change(tool)
        };
        self.as_mut().fill();
        match host.as_deref().map(|k| (k, k.strip_prefix(crate::tools::cores::OPEN_FOLDER))) {
            Some((_, Some(path))) => self.as_mut().open_folder(QString::from(path)),
            Some((key, None)) => self.as_mut().host_asked(QString::from(key)),
            None => {}
        }
    }

    pub fn focus(self: Pin<&mut Self>, index: i32) {
        if self.filling {
            return;
        }
        self.with(move |t| {
            t.set_focus(index.max(0) as usize);
            None
        });
    }

    pub fn activate(self: Pin<&mut Self>, index: i32) {
        self.with(move |t| t.activate(index.max(0) as usize));
    }

    pub fn choose_section(self: Pin<&mut Self>, label: &QString) {
        let label = label.to_string();
        self.with(move |t| {
            t.choose_section(&label);
            None
        });
    }

    pub fn search(self: Pin<&mut Self>, words: &QString) {
        let words = words.to_string();
        self.with(move |t| {
            t.set_search(&words);
            None
        });
    }

    pub fn act(self: Pin<&mut Self>, key: &QString) {
        let key = key.to_string();
        self.with(move |t| {
            match key.as_str() {
                crate::tools::MORE => t.more(),
                crate::tools::LIST => t.list(),
                crate::tools::APPLY => t.apply(),
                other => {
                    t.shoulder(other);
                }
            }
            None
        });
    }

    /// At once, not after the event: the focus has already moved, and a later press must find the state it left.
    pub fn keys_focused(self: Pin<&mut Self>, keys: bool) {
        self.change(move |t| {
            t.focus_keys(keys);
            None
        });
    }

    pub fn end_search(self: Pin<&mut Self>) {
        self.with(move |t| {
            t.end_search();
            None
        });
    }

    pub fn apply(self: Pin<&mut Self>) {
        self.with(move |t| {
            t.apply();
            None
        });
    }

    pub fn answer(self: Pin<&mut Self>, index: i32) {
        self.with(move |t| {
            t.answer(index);
            None
        });
    }

    pub fn card_focus(self: Pin<&mut Self>, index: i32) {
        self.with(move |t| {
            t.card_focus(index.max(0) as usize);
            None
        });
    }

    pub fn card_choose(self: Pin<&mut Self>, index: i32) {
        self.with(move |t| t.card_choose(index.max(0) as usize));
    }

    pub fn card_step(self: Pin<&mut Self>, index: i32, by: i32) {
        self.with(move |t| {
            t.card_step(index.max(0) as usize, by);
            None
        });
    }

    pub fn card_back(self: Pin<&mut Self>) {
        self.with(move |t| {
            t.card_back();
            None
        });
    }

    pub fn poll(self: Pin<&mut Self>) {
        self.with(move |t| {
            t.poll();
            None
        });
    }

    /// Every property read again from the tool's description.
    fn fill(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().filling = true;
        self.as_mut().put();
        self.as_mut().rust_mut().filling = false;
    }

    fn put(mut self: Pin<&mut Self>) {
        let Some(tool) = self.shown.as_ref() else { return };
        let source = &tool.source;
        let title = QString::from(source.title());
        let facts = text(&Value::Array(source.facts().iter().map(|f| json!({ "words": f.words, "tone": f.tone as i32 })).collect()));
        let sections = source.sections();
        let section = QString::from(sections.get(tool.section()).map(|s| s.1.as_str()).unwrap_or_default());
        let section_labels = list(&sections.into_iter().map(|s| s.1).collect::<Vec<_>>());
        let titles: Vec<&str> = source.columns().iter().map(|c| c.title).collect();
        let columns = text(&Value::Array(
            source
                .columns()
                .iter()
                .enumerate()
                .map(|(at, c)| json!({ "key": format!("c{at}"), "toneKey": format!("t{at}"), "shortKey": if c.gives_way { format!("s{at}") } else { String::new() }, "title": c.title, "fill": c.fill, "least": c.least, "trailing": c.trailing }))
                .collect(),
        ));
        let rows = text(&Value::Array(
            tool.rows()
                .iter()
                .map(|item| {
                    let heard: Vec<String> = item.cells.iter().zip(&titles).map(|(c, t)| format!("{t}: {}", c.words)).collect();
                    let mut row = json!({ "key": item.key, "name": heard.join(", ") });
                    for (at, cell) in item.cells.iter().enumerate() {
                        row[format!("c{at}")] = json!(cell.words);
                        row[format!("t{at}")] = json!(cell.tone as i32);
                        row[format!("s{at}")] = json!(cell.short.as_deref().unwrap_or(&cell.words));
                    }
                    row
                })
                .collect(),
        ));
        let current = tool.focus().map_or(-1, |at| at as i32);
        let search_text = QString::from(tool.search());
        let searching = tool.searching();
        let search_keys = tool.search_keys();
        let action_lines = QString::from(tool.action_lines().as_str());
        let pane = tool.pane();
        let sentences = text(&Value::Array(pane.sentences.iter().map(|s| json!({ "words": s.words, "tone": s.tone as i32 })).collect()));
        let bar = tool.bar();
        let help = text(&Value::Array(
            tool.help()
                .iter()
                .map(|(button, label, _)| json!({ "button": button.index() as i32, "label": label, "pair": crate::help::pair_of(*button).map_or(-1, |b| b.index() as i32) }))
                .collect(),
        ));
        let card = tool.card();
        let (card_title, card_breadcrumb, card_rows, card_current, pane_lines, pane_muted) = match &card {
            Some((title, breadcrumb, rows, at)) => {
                let shown: Vec<Value> = rows
                    .iter()
                    .map(|r| json!({ "key": r.key, "label": r.label, "value": r.value, "steps": r.steps, "opens": r.opens, "sectionEnd": r.section_end, "dimmed": false }))
                    .collect();
                let pane = rows.get(*at).map(|r| r.pane.lines.clone()).unwrap_or_default();
                (
                    QString::from(title.as_str()),
                    QString::from(breadcrumb.as_str()),
                    text(&Value::Array(shown)),
                    *at as i32,
                    list(&pane.iter().map(|l| l.0.clone()).collect::<Vec<_>>()),
                    list(&pane.iter().map(|l| l.1.clone()).collect::<Vec<_>>()),
                )
            }
            // With no card the focused row is none, so a card's first row is a change the card's list follows.
            None => (QString::default(), QString::default(), QString::from("[]"), -1, QStringList::default(), QStringList::default()),
        };
        let question = tool.question().cloned();
        let busy = source.running();
        let apply_enabled = tool.can_apply();
        put!(self, title, title, title_changed);
        put!(self, facts, facts, facts_changed);
        put!(self, sections, section_labels, sections_changed);
        put!(self, section, section, section_changed);
        put!(self, columns, columns, columns_changed);
        put!(self, rows, rows, rows_changed);
        put!(self, current, current, current_changed);
        put!(self, search_text, search_text, search_text_changed);
        let typed = self.searching && !searching;
        let opened = !self.searching && searching;
        put!(self, searching, searching, searching_changed);
        put!(self, search_keys, search_keys, search_keys_changed);
        put!(self, action_lines, action_lines, action_lines_changed);
        put!(self, pane_title, QString::from(pane.title.as_str()), pane_title_changed);
        put!(self, pane_caption, QString::from(pane.caption.as_str()), pane_caption_changed);
        put!(self, sentences, sentences, sentences_changed);
        put!(self, summary, QString::from(bar.line.words.as_str()), summary_changed);
        put!(self, summary_tone, bar.line.tone as i32, summary_tone_changed);
        put!(self, progress, bar.progress.unwrap_or(-1.0), progress_changed);
        put!(self, apply_enabled, apply_enabled, apply_enabled_changed);
        put!(self, meter, bar.meter, meter_changed);
        put!(self, meter_words, QString::from(bar.meter_words.as_str()), meter_words_changed);
        put!(self, help, help, help_changed);
        put!(self, card_title, card_title, card_title_changed);
        put!(self, card_breadcrumb, card_breadcrumb, card_breadcrumb_changed);
        put!(self, card_rows, card_rows, card_rows_changed);
        put!(self, card_current, card_current, card_current_changed);
        put!(self, card_pane_lines, pane_lines, card_pane_lines_changed);
        put!(self, card_pane_muted, pane_muted, card_pane_muted_changed);
        let closed = self.card_open && card.is_none();
        put!(self, card_open, card.is_some(), card_open_changed);
        // A card closed on the search's opening leaves the focus to the keys.
        if (closed && !search_keys) || typed {
            self.as_mut().list_focused();
        }
        if opened {
            self.as_mut().search_opened();
        }
        match question {
            Some(q) => {
                put!(self, question_title, QString::from(q.title.as_str()), question_title_changed);
                put!(self, question_sentences, list(&q.sentences), question_sentences_changed);
                put!(self, question_caption, QString::from(q.caption.as_str()), question_caption_changed);
                put!(self, question_choices, list(&q.choices), question_choices_changed);
                put!(self, question_back, crate::question::back_index(&q), question_back_changed);
                put!(self, asking, true, asking_changed);
            }
            None => put!(self, asking, false, asking_changed),
        }
        put!(self, busy, busy, busy_changed);
    }
}
