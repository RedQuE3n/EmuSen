//! `NoticePresenter`: formats notice words into Eclipse's heading, body, icon and optional folder action without touching `EmuSen.Kit`.

use core::pin::Pin;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use emusen_kit_core::input::UiButton;

/// Default action label for failure notices with a folder to open (EclipseDialog:29).
pub const OPEN_ITS_FOLDER: &str = "Open its folder";

/// Splits a raw notice string into (heading, body) for Eclipse's two-line notice banner.
pub fn split_words(words: &str) -> (String, String) {
    let trimmed = words.trim();
    if let Some((head, body)) = trimmed.split_once('\n') {
        return (head.trim().to_owned(), body.trim().to_owned());
    }
    if let Some((head, body)) = trimmed.split_once(": ") {
        let head = head.trim();
        let body = body.trim();
        if !head.is_empty() && !body.is_empty() && head.chars().count() <= 80 {
            return (head.to_owned(), body.to_owned());
        }
    }
    (trimmed.to_owned(), String::new())
}

/// Icon key in `marks.rs` for an Eclipse notice by its attention flag.
pub fn icon_for(attention: bool) -> &'static str {
    if attention { "notice.warning" } else { "notice.ok" }
}

/// Consumes a queued folder action only when an attention notice matches its exact words, clearing mismatches.
pub fn take_pending_action(pending_words: &mut String, pending_action: &mut Option<(String, String)>, words: &str, attention: bool) -> Option<(String, String)> {
    if !attention {
        return None;
    }
    let matched = !pending_words.is_empty() && *pending_words == words;
    pending_words.clear();
    if matched {
        pending_action.take()
    } else {
        *pending_action = None;
        None
    }
}

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("emusen-kit-qt/src/kinds.cxxqt.h");
        #[namespace = "Pad"]
        type PadButton = emusen_kit_qt::kinds::ffi::PadButton;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, active)]
        #[qproperty(QString, heading, READ, NOTIFY)]
        #[qproperty(QString, body, READ, NOTIFY)]
        #[qproperty(QString, icon, READ, NOTIFY)]
        #[qproperty(QString, action_label, cxx_name = "actionLabel", READ, NOTIFY)]
        #[qproperty(QString, action_path, cxx_name = "actionPath", READ, NOTIFY)]
        #[qproperty(bool, has_action, cxx_name = "hasAction", READ, NOTIFY)]
        type NoticePresenter = super::NoticePresenterRust;
    }

    extern "RustQt" {
        /// Updates heading, body and icon from `Kit.NoticeModel`'s `words` and `attention`.
        #[qinvokable]
        fn sync(self: Pin<&mut NoticePresenter>, words: &QString, attention: bool);

        /// Attaches an "Open its folder" action scoped to the exact next attention notice matching `words`.
        #[qinvokable]
        #[cxx_name = "setPendingFolder"]
        fn set_pending_folder(self: Pin<&mut NoticePresenter>, words: &QString, folder: &QString);

        /// Sets an explicit heading, body and optional action (label + folder path) for structured notices.
        #[qinvokable]
        fn present(self: Pin<&mut NoticePresenter>, heading: &QString, body: &QString, attention: bool, action_label: &QString, action_path: &QString);

        /// Clears the active action once the notice hides.
        #[qinvokable]
        #[cxx_name = "clearAction"]
        fn clear_action(self: Pin<&mut NoticePresenter>);

        /// Clears the active action when `shown` becomes false.
        #[qinvokable]
        #[cxx_name = "clearWhenHidden"]
        fn clear_when_hidden(self: Pin<&mut NoticePresenter>, shown: bool);

        /// Invokes the active notice action if one is attached.
        #[qinvokable]
        #[cxx_name = "invokeAction"]
        fn invoke_action(self: Pin<&mut NoticePresenter>);

        /// Consumes X (`UiButton::Screensaver`) when an Eclipse notice with an action is active.
        #[qinvokable]
        #[cxx_name = "takePad"]
        fn take_pad(self: Pin<&mut NoticePresenter>, button: PadButton) -> bool;

        /// Emitted with the folder path when the player activates the notice action.
        #[qsignal]
        #[cxx_name = "actionTriggered"]
        fn action_triggered(self: Pin<&mut NoticePresenter>, path: QString);
    }
}

pub struct NoticePresenterRust {
    active: bool,
    heading: QString,
    body: QString,
    icon: QString,
    action_label: QString,
    action_path: QString,
    has_action: bool,
    pending_words: String,
    pending_action: Option<(String, String)>,
    structured: Option<String>,
}

impl Default for NoticePresenterRust {
    fn default() -> Self {
        NoticePresenterRust {
            active: false,
            heading: QString::default(),
            body: QString::default(),
            icon: QString::from(icon_for(false)),
            action_label: QString::default(),
            action_path: QString::default(),
            has_action: false,
            pending_words: String::new(),
            pending_action: None,
            structured: None,
        }
    }
}

impl qobject::NoticePresenter {
    fn apply(mut self: Pin<&mut Self>, heading: String, body: String, attention: bool, action_label: String, action_path: String) {
        let h = QString::from(heading.as_str());
        if self.heading != h {
            self.as_mut().rust_mut().heading = h;
            self.as_mut().heading_changed();
        }
        let b = QString::from(body.as_str());
        if self.body != b {
            self.as_mut().rust_mut().body = b;
            self.as_mut().body_changed();
        }
        let icon = QString::from(icon_for(attention));
        if self.icon != icon {
            self.as_mut().rust_mut().icon = icon;
            self.as_mut().icon_changed();
        }
        let label = QString::from(action_label.as_str());
        if self.action_label != label {
            self.as_mut().rust_mut().action_label = label;
            self.as_mut().action_label_changed();
        }
        let path = QString::from(action_path.as_str());
        if self.action_path != path {
            self.as_mut().rust_mut().action_path = path;
            self.as_mut().action_path_changed();
        }
        let has = !action_label.is_empty() && !action_path.is_empty();
        if self.has_action != has {
            self.as_mut().rust_mut().has_action = has;
            self.has_action_changed();
        }
    }

    pub fn set_pending_folder(mut self: Pin<&mut Self>, words: &QString, folder: &QString) {
        let w = words.to_string();
        let f = folder.to_string();
        if w.is_empty() || f.is_empty() {
            self.as_mut().rust_mut().pending_words.clear();
            self.as_mut().rust_mut().pending_action = None;
            return;
        }
        self.as_mut().rust_mut().pending_words = w;
        self.as_mut().rust_mut().pending_action = Some((OPEN_ITS_FOLDER.to_owned(), f));
    }

    pub fn present(mut self: Pin<&mut Self>, heading: &QString, body: &QString, attention: bool, action_label: &QString, action_path: &QString) {
        let h = heading.to_string();
        let b = body.to_string();
        let al = action_label.to_string();
        let ap = action_path.to_string();
        let joined = if b.is_empty() { h.clone() } else { format!("{h}\n{b}") };
        self.as_mut().rust_mut().structured = Some(joined);
        self.as_mut().rust_mut().pending_words.clear();
        self.as_mut().rust_mut().pending_action = None;
        self.apply(h, b, attention, al, ap);
    }

    pub fn sync(mut self: Pin<&mut Self>, words: &QString, attention: bool) {
        let raw = words.to_string();
        if self.structured.as_deref() == Some(raw.as_str()) {
            self.as_mut().rust_mut().structured = None;
            return;
        }
        self.as_mut().rust_mut().structured = None;
        let (heading, body) = split_words(&raw);
        let action = {
            let state = self.as_mut().rust_mut().get_mut();
            take_pending_action(&mut state.pending_words, &mut state.pending_action, &raw, attention)
        };
        let (al, ap) = action.unwrap_or_default();
        self.apply(heading, body, attention, al, ap);
    }

    pub fn clear_action(mut self: Pin<&mut Self>) {
        if !self.action_label.is_empty() {
            self.as_mut().rust_mut().action_label = QString::default();
            self.as_mut().action_label_changed();
        }
        if !self.action_path.is_empty() {
            self.as_mut().rust_mut().action_path = QString::default();
            self.as_mut().action_path_changed();
        }
        if self.has_action {
            self.as_mut().rust_mut().has_action = false;
            self.has_action_changed();
        }
    }

    pub fn clear_when_hidden(self: Pin<&mut Self>, shown: bool) {
        if !shown {
            self.clear_action();
        }
    }

    pub fn invoke_action(self: Pin<&mut Self>) {
        if !self.has_action {
            return;
        }
        let path = self.action_path.clone();
        self.action_triggered(path);
    }

    pub fn take_pad(self: Pin<&mut Self>, button: qobject::PadButton) -> bool {
        if !self.active || !self.has_action || emusen_kit_qt::kinds::ui_button(button) != Some(UiButton::Screensaver) {
            return false;
        }
        self.invoke_action();
        true
    }
}
