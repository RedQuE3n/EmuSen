//! `FrameController`: kit-core's `Walk` on Qt, and the one place a pad press, a click or a key becomes what the frame does (EmuSen_Kit.md §7.3).

use core::pin::Pin;

use cxx_qt::CxxQtType;

use cxx_qt_lib::{QList, QString, QStringList};
use emusen_kit_core::input::{intent, ElementId, InputEvent, Intent, Key, UiButton, Walk, WalkAction, WalkEvent, WalkForm};

use crate::kinds::{self, Form, PadButton, Role};
use crate::qt_keys;

/// Where a press went: the open sheet, the walk, or declined with the key it becomes for the focused control (EmuSen_Qt.md §1.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Routed {
    Sheet,
    Walk,
    Declined(Option<Key>),
}

/// The order of §1.7: the active sheet or question first, then the frame's walk, then the focused control.
pub fn route(sheet_open: bool, walk: &mut Walk, button: UiButton) -> Routed {
    if sheet_open {
        return Routed::Sheet;
    }
    if walk.press(button) {
        Routed::Walk
    } else {
        Routed::Declined(button.key())
    }
}

/// What the open sheet does with a press: B dismisses it; anything else is announced and reaches its focused control as a key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheetRouted {
    Dismiss,
    Pass(Option<Key>),
}

pub fn sheet_route(button: UiButton) -> SheetRouted {
    if button == UiButton::Back {
        SheetRouted::Dismiss
    } else {
        SheetRouted::Pass(button.key())
    }
}

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("emusen-kit-qt/src/kinds.cxxqt.h");
        #[namespace = "Forms"]
        type Form = crate::kinds::ffi::Form;
        #[namespace = "Roles"]
        type Role = crate::kinds::ffi::Role;
        #[namespace = "Pad"]
        type PadButton = crate::kinds::ffi::PadButton;
        include!("kit_glue.h");
        fn send_key(key: i32, modifiers: i32);
    }

    unsafe extern "C++Qt" {
        include!(<QtQuick/QQuickItem>);
        #[qobject]
        type QQuickItem;
        #[cxx_name = "forceActiveFocus"]
        fn force_active_focus(self: Pin<&mut QQuickItem>);
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(Form, form)]
        #[qproperty(bool, has_detail)]
        #[qproperty(bool, detail_beside)]
        #[qproperty(bool, navigation_shown)]
        #[qproperty(bool, actions_shown)]
        #[qproperty(bool, search_shown)]
        #[qproperty(QStringList, sections)]
        #[qproperty(QString, selected_section)]
        #[qproperty(bool, sheet_open)]
        #[qproperty(Role, focused_role, READ, NOTIFY)]
        #[qproperty(bool, detail_open, READ, NOTIFY)]
        #[qproperty(bool, page_open, READ, NOTIFY)]
        type FrameController = super::FrameControllerRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qsignal]
        fn section_chosen(self: Pin<&mut FrameController>, key: QString);
        #[qsignal]
        fn focus_moved(self: Pin<&mut FrameController>, role: Role);
        #[qsignal]
        fn detail_opened(self: Pin<&mut FrameController>);
        #[qsignal]
        fn detail_closed(self: Pin<&mut FrameController>);
        #[qsignal]
        fn page_closed(self: Pin<&mut FrameController>);
        #[qsignal]
        fn action_invoked(self: Pin<&mut FrameController>, key: QString);
        /// A press while a sheet or question is open: theirs before the walk's, and the focused control's as a key.
        #[qsignal]
        fn sheet_pressed(self: Pin<&mut FrameController>, button: PadButton);
        /// B while a sheet or question is open: it closes.
        #[qsignal]
        fn sheet_dismissed(self: Pin<&mut FrameController>);
        /// B at the top of the walk: the host's signal to close.
        #[qsignal]
        fn back_declined(self: Pin<&mut FrameController>);

        /// A pad press in §1.7's order: the sheet, the walk, then the focused control as a key; true if the first two took it.
        #[qinvokable]
        fn offer(self: Pin<&mut FrameController>, button: PadButton) -> bool;
        /// The walk alone, without the sheet or the key.
        #[qinvokable]
        fn press(self: Pin<&mut FrameController>, button: PadButton) -> bool;
        /// A key the focused control declined: Tab walks the roles, the others are the presses they map to; true if used.
        #[qinvokable]
        fn key(self: Pin<&mut FrameController>, key: i32, modifiers: i32) -> bool;
        /// The pointer rests on a walkable element: the ring moves to it.
        #[qinvokable]
        unsafe fn hover(self: Pin<&mut FrameController>, item: *mut QQuickItem, hovered: bool);
        /// A pointer button on a walkable element: the left is A there, the right and the back button are B.
        #[qinvokable]
        unsafe fn click(self: Pin<&mut FrameController>, item: *mut QQuickItem, button: i32);
        #[qinvokable]
        fn move_focus(self: Pin<&mut FrameController>, by: i32) -> bool;
        #[qinvokable]
        fn focus_role(self: Pin<&mut FrameController>, role: Role) -> bool;
        #[qinvokable]
        fn step_section(self: Pin<&mut FrameController>, by: i32) -> bool;
        #[qinvokable]
        fn open_detail(self: Pin<&mut FrameController>) -> bool;
        #[qinvokable]
        fn close_detail(self: Pin<&mut FrameController>) -> bool;
        #[qinvokable]
        fn open_page(self: Pin<&mut FrameController>);
        #[qinvokable]
        fn close_page(self: Pin<&mut FrameController>) -> bool;
        /// Adds or replaces an action by its key, with the pad button that invokes it (NoRole's counterpart is Kit.Guide for none).
        #[qinvokable]
        fn set_action(self: Pin<&mut FrameController>, key: &QString, button: PadButton, enabled: bool);
        #[qinvokable]
        fn clear_actions(self: Pin<&mut FrameController>);
    }

    impl cxx_qt::Initialize for FrameController {}
}

pub struct FrameControllerRust {
    form: Form,
    has_detail: bool,
    detail_beside: bool,
    navigation_shown: bool,
    actions_shown: bool,
    search_shown: bool,
    sections: QStringList,
    selected_section: QString,
    sheet_open: bool,
    focused_role: Role,
    detail_open: bool,
    page_open: bool,
    walk: Walk,
    actions: Vec<WalkAction>,
    syncing: bool,
}

impl Default for FrameControllerRust {
    fn default() -> Self {
        FrameControllerRust {
            form: Form::BigScreen,
            has_detail: false,
            detail_beside: false,
            navigation_shown: true,
            actions_shown: false,
            search_shown: false,
            sections: QStringList::default(),
            selected_section: QString::default(),
            sheet_open: false,
            focused_role: Role::NoRole,
            detail_open: false,
            page_open: false,
            walk: Walk::new(WalkForm::BigScreen),
            actions: Vec::new(),
            syncing: false,
        }
    }
}

impl cxx_qt::Initialize for qobject::FrameController {
    fn initialize(mut self: Pin<&mut Self>) {
        self.as_mut()
            .on_form_changed(|mut this| {
                let form = kinds::walk_form(this.form);
                this.as_mut().rust_mut().walk.set_form(form);
                this.sync();
            })
            .release();
        self.as_mut().on_has_detail_changed(|this| this.apply_detail()).release();
        self.as_mut().on_detail_beside_changed(|this| this.apply_detail()).release();
        self.as_mut()
            .on_navigation_shown_changed(|mut this| {
                let shown = this.navigation_shown;
                this.as_mut().rust_mut().walk.show_navigation(shown);
                this.sync();
            })
            .release();
        self.as_mut()
            .on_actions_shown_changed(|mut this| {
                let shown = this.actions_shown;
                this.as_mut().rust_mut().walk.show_actions(shown);
                this.sync();
            })
            .release();
        self.as_mut()
            .on_search_shown_changed(|mut this| {
                let shown = this.search_shown;
                this.as_mut().rust_mut().walk.show_search(shown);
                this.sync();
            })
            .release();
        self.as_mut().on_sections_changed(|this| this.apply_sections()).release();
        self.as_mut()
            .on_selected_section_changed(|mut this| {
                if this.syncing {
                    return;
                }
                let key = this.selected_section.to_string();
                this.as_mut().rust_mut().walk.select_section(&key);
                this.sync();
            })
            .release();
    }
}

unsafe fn focus(item: *mut qobject::QQuickItem) {
    if let Some(item) = item.as_mut() {
        Pin::new_unchecked(item).force_active_focus();
    }
}

impl qobject::FrameController {
    pub fn offer(mut self: Pin<&mut Self>, button: PadButton) -> bool {
        let Some(ui) = kinds::ui_button(button) else { return false };
        let sheet_open = self.sheet_open;
        let routed = route(sheet_open, &mut self.as_mut().rust_mut().walk, ui);
        match routed {
            Routed::Sheet => {
                self.as_mut().sheet_offer(ui);
                true
            }
            Routed::Walk => {
                self.sync();
                true
            }
            Routed::Declined(key) => {
                if ui == UiButton::Back {
                    self.as_mut().back_declined();
                }
                self.sync();
                if let Some(code) = key.and_then(qt_keys::qt_key) {
                    qobject::send_key(code, 0);
                }
                false
            }
        }
    }

    /// The sheet's share of a press: B dismisses it; anything else is announced and reaches its focused control as a key.
    fn sheet_offer(mut self: Pin<&mut Self>, button: UiButton) {
        match sheet_route(button) {
            SheetRouted::Dismiss => self.as_mut().sheet_dismissed(),
            SheetRouted::Pass(key) => {
                self.as_mut().sheet_pressed(kinds::pad_button(button));
                if let Some(code) = key.and_then(qt_keys::qt_key) {
                    qobject::send_key(code, 0);
                }
            }
        }
    }

    pub fn press(mut self: Pin<&mut Self>, button: PadButton) -> bool {
        let Some(ui) = kinds::ui_button(button) else { return false };
        let used = self.as_mut().rust_mut().walk.press(ui);
        if !used && ui == UiButton::Back {
            self.as_mut().back_declined();
        }
        self.sync();
        used
    }

    pub fn key(self: Pin<&mut Self>, key: i32, modifiers: i32) -> bool {
        let (key, shift) = qt_keys::kit_key(key, modifiers);
        match intent(InputEvent::Key { key, shift }) {
            Some(Intent::Walk(by)) => self.move_focus(by),
            Some(Intent::Press(button)) => {
                if self.sheet_open {
                    self.sheet_offer(button);
                    true
                } else {
                    self.press(kinds::pad_button(button))
                }
            }
            _ => false,
        }
    }

    pub unsafe fn hover(self: Pin<&mut Self>, item: *mut qobject::QQuickItem, hovered: bool) {
        if hovered && matches!(intent(InputEvent::Hover(ElementId(0))), Some(Intent::Focus(_))) {
            focus(item);
        }
    }

    pub unsafe fn click(mut self: Pin<&mut Self>, item: *mut qobject::QQuickItem, button: i32) {
        let Some(button) = qt_keys::pointer_button(button) else { return };
        match intent(InputEvent::Click { target: ElementId(0), button }) {
            Some(Intent::Activate(_)) => {
                focus(item);
                self.as_mut().offer(kinds::pad_button(UiButton::Accept));
            }
            Some(Intent::Press(button)) => {
                self.as_mut().offer(kinds::pad_button(button));
            }
            _ => {}
        }
    }

    pub fn move_focus(mut self: Pin<&mut Self>, by: i32) -> bool {
        let moved = self.as_mut().rust_mut().walk.move_focus(by);
        self.sync();
        moved
    }

    pub fn focus_role(mut self: Pin<&mut Self>, role: Role) -> bool {
        let Some(role) = kinds::walk_role(role) else { return false };
        let focused = self.as_mut().rust_mut().walk.focus_role(role);
        self.sync();
        focused
    }

    pub fn step_section(mut self: Pin<&mut Self>, by: i32) -> bool {
        let stepped = self.as_mut().rust_mut().walk.step_section(by);
        self.sync();
        stepped
    }

    pub fn open_detail(mut self: Pin<&mut Self>) -> bool {
        let opened = self.as_mut().rust_mut().walk.open_detail();
        self.sync();
        opened
    }

    pub fn close_detail(mut self: Pin<&mut Self>) -> bool {
        let closed = self.as_mut().rust_mut().walk.close_detail();
        self.sync();
        closed
    }

    pub fn open_page(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().walk.open_page();
        self.sync();
    }

    pub fn close_page(mut self: Pin<&mut Self>) -> bool {
        let closed = self.as_mut().rust_mut().walk.close_page();
        self.sync();
        closed
    }

    pub fn set_action(mut self: Pin<&mut Self>, key: &QString, button: PadButton, enabled: bool) {
        let action = WalkAction { key: key.to_string(), button: kinds::ui_button(button), enabled };
        let mut rust = self.as_mut().rust_mut();
        match rust.actions.iter_mut().find(|a| a.key == action.key) {
            Some(existing) => *existing = action,
            None => rust.actions.push(action),
        }
        let actions = rust.actions.clone();
        rust.walk.set_actions(actions);
    }

    pub fn clear_actions(mut self: Pin<&mut Self>) {
        let mut rust = self.as_mut().rust_mut();
        rust.actions.clear();
        rust.walk.set_actions(Vec::new());
    }

    fn apply_detail(mut self: Pin<&mut Self>) {
        let (has, beside) = (self.has_detail, self.detail_beside);
        self.as_mut().rust_mut().walk.set_detail(has, beside);
        self.sync();
    }

    fn apply_sections(mut self: Pin<&mut Self>) {
        let keys: Vec<String> = QList::<QString>::from(&self.sections).iter().map(QString::to_string).collect();
        let selected = self.selected_section.to_string();
        let selected = if selected.is_empty() { None } else { Some(selected.as_str()) };
        self.as_mut().rust_mut().walk.set_sections(keys, selected);
        self.sync();
    }

    /// Mirrors the walk into the read-only properties and turns its events into signals.
    fn sync(mut self: Pin<&mut Self>) {
        let (role, detail_open, page_open, selected) = {
            let walk = &self.walk;
            (kinds::role_of(walk.focused()), walk.is_detail_open(), walk.is_page_open(), walk.selected_section().unwrap_or("").to_owned())
        };
        if self.focused_role != role {
            self.as_mut().rust_mut().focused_role = role;
            self.as_mut().focused_role_changed();
        }
        if self.detail_open != detail_open {
            self.as_mut().rust_mut().detail_open = detail_open;
            self.as_mut().detail_open_changed();
        }
        if self.page_open != page_open {
            self.as_mut().rust_mut().page_open = page_open;
            self.as_mut().page_open_changed();
        }
        let selected = QString::from(selected.as_str());
        if self.selected_section != selected {
            self.as_mut().rust_mut().syncing = true;
            self.as_mut().set_selected_section(selected);
            self.as_mut().rust_mut().syncing = false;
        }
        for event in self.as_mut().rust_mut().walk.take_events() {
            match event {
                WalkEvent::SectionChosen(key) => self.as_mut().section_chosen(QString::from(key.as_str())),
                WalkEvent::Focused(role) => self.as_mut().focus_moved(kinds::role_of(Some(role))),
                WalkEvent::DetailOpened => self.as_mut().detail_opened(),
                WalkEvent::DetailClosed => self.as_mut().detail_closed(),
                WalkEvent::PageClosed => self.as_mut().page_closed(),
                WalkEvent::ActionInvoked(key) => self.as_mut().action_invoked(QString::from(key.as_str())),
            }
        }
    }
}
