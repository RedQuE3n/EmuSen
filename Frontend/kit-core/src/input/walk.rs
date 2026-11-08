use super::UiButton;

/// The frame's roles, in the pad's order; Search is reached by its own button and left by B.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Navigation,
    Content,
    Detail,
    Actions,
    Search,
}

impl Role {
    /// The order the walk steps, skipping a role not shown.
    pub const WALK_ORDER: [Role; 4] = [Role::Navigation, Role::Content, Role::Detail, Role::Actions];
}

/// The frame's three forms (LunaP's `AdaptiveForm`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WalkForm {
    Desktop,
    BigScreen,
    Card,
}

/// An action the frame offers, with the pad button that invokes it, if any.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WalkAction {
    pub key: String,
    pub button: Option<UiButton>,
    pub enabled: bool,
}

impl WalkAction {
    /// A screen's actions declared as text, one a line, `key Button` with the button by its name; a line naming no
    /// button is skipped (EmuSen_Kit.md §7.18).
    pub fn from_lines(text: &str) -> Vec<WalkAction> {
        text.lines()
            .filter_map(|line| {
                let mut words = line.split_whitespace();
                let key = words.next()?;
                let button = UiButton::named(words.next()?)?;
                Some(WalkAction { key: key.to_owned(), button: Some(button), enabled: true })
            })
            .collect()
    }
}

/// What a walk did that its host must answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WalkEvent {
    /// A person chose a section; never raised by `set_sections` or `select_section`.
    SectionChosen(String),
    Focused(Role),
    DetailOpened,
    DetailClosed,
    PageClosed,
    ActionInvoked(String),
}

/// The frame's focus walk over roles, sections, a detail and actions, in ES-DE's grammar (LunaP §202.4, EmuSen_Qt.md §1.4).
#[derive(Clone, Debug)]
pub struct Walk {
    form: WalkForm,
    navigation: bool,
    actions_shown: bool,
    search_shown: bool,
    has_detail: bool,
    detail_beside: bool,
    detail_open: bool,
    page_open: bool,
    sections: Vec<String>,
    selected: Option<usize>,
    focused: Option<Role>,
    swap_accept_back: bool,
    actions: Vec<WalkAction>,
    events: Vec<WalkEvent>,
}

impl Walk {
    /// A walk with content and navigation shown, no detail, no sections and no actions.
    pub fn new(form: WalkForm) -> Walk {
        Walk {
            form,
            navigation: true,
            actions_shown: false,
            search_shown: false,
            has_detail: false,
            detail_beside: false,
            detail_open: false,
            page_open: false,
            sections: Vec::new(),
            selected: None,
            focused: None,
            swap_accept_back: false,
            actions: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn form(&self) -> WalkForm {
        self.form
    }

    /// Changes the form, keeping the chosen section and the focused role; on the desktop or with the detail beside, its page closes.
    pub fn set_form(&mut self, form: WalkForm) {
        self.form = form;
        if form == WalkForm::Desktop || self.detail_beside {
            self.detail_open = false;
        }
        if let Some(role) = self.focused
            && !self.is_shown(role)
        {
            self.focused = None;
            self.move_focus(1);
        }
    }

    pub fn set_swap_accept_back(&mut self, swap: bool) {
        self.swap_accept_back = swap;
    }

    /// Whether the frame has a detail, and whether it stands beside the content on the big screen rather than as a page of its own.
    pub fn set_detail(&mut self, has_detail: bool, beside: bool) {
        self.has_detail = has_detail;
        self.detail_beside = beside;
        if !has_detail || beside {
            self.detail_open = false;
        }
    }

    pub fn show_navigation(&mut self, shown: bool) {
        self.navigation = shown;
    }

    pub fn show_actions(&mut self, shown: bool) {
        self.actions_shown = shown;
    }

    pub fn show_search(&mut self, shown: bool) {
        self.search_shown = shown;
    }

    pub fn set_actions(&mut self, actions: Vec<WalkAction>) {
        self.actions = actions;
    }

    /// Whether a role is shown now: the detail on the big screen only while its page is open or it stands beside.
    pub fn is_shown(&self, role: Role) -> bool {
        match role {
            Role::Navigation => self.navigation,
            Role::Content => true,
            Role::Actions => self.actions_shown,
            Role::Search => self.search_shown,
            Role::Detail => {
                self.has_detail && (self.form != WalkForm::BigScreen || self.detail_beside || self.detail_open || self.page_open)
            }
        }
    }

    pub fn focused(&self) -> Option<Role> {
        self.focused
    }

    /// Moves the focus to a role, if it is shown.
    pub fn focus_role(&mut self, role: Role) -> bool {
        if !self.is_shown(role) {
            return false;
        }
        self.focused = Some(role);
        self.events.push(WalkEvent::Focused(role));
        true
    }

    /// The next or previous shown role in the walk's order, wrapping; Tab, Shift+Tab, F6 and Shift+F6 do the same.
    pub fn move_focus(&mut self, by: i32) -> bool {
        let shown: Vec<Role> = Role::WALK_ORDER.into_iter().filter(|r| self.is_shown(*r)).collect();
        if shown.is_empty() {
            return false;
        }
        let n = shown.len() as i32;
        let at = self.focused.and_then(|f| shown.iter().position(|r| *r == f)).map(|i| i as i32);
        let next = match at {
            None if by >= 0 => 0,
            None => n - 1,
            Some(at) => (at + by).rem_euclid(n),
        };
        self.focus_role(shown[next as usize])
    }

    pub fn sections(&self) -> &[String] {
        &self.sections
    }

    pub fn selected_section(&self) -> Option<&str> {
        self.selected.and_then(|i| self.sections.get(i)).map(String::as_str)
    }

    /// Gives the navigation its sections and the chosen one, raising nothing; an unknown choice takes the first.
    pub fn set_sections(&mut self, sections: Vec<String>, selected: Option<&str>) {
        self.selected = match selected.and_then(|key| sections.iter().position(|s| s == key)) {
            Some(i) => Some(i),
            None if sections.is_empty() => None,
            None => Some(0),
        };
        self.sections = sections;
    }

    /// Chooses a section as the application does, raising nothing.
    pub fn select_section(&mut self, key: &str) -> bool {
        match self.sections.iter().position(|s| s == key) {
            Some(i) => {
                self.selected = Some(i);
                true
            }
            None => false,
        }
    }

    /// Steps the chosen section as the shoulders do, wrapping at the ends, and raises `SectionChosen`.
    pub fn step_section(&mut self, by: i32) -> bool {
        if self.sections.is_empty() {
            return false;
        }
        let at = self.selected.unwrap_or(0) as i32;
        let next = (at + by).rem_euclid(self.sections.len() as i32) as usize;
        self.selected = Some(next);
        self.events.push(WalkEvent::SectionChosen(self.sections[next].clone()));
        true
    }

    pub fn is_detail_open(&self) -> bool {
        self.detail_open && self.form == WalkForm::BigScreen
    }

    pub fn is_page_open(&self) -> bool {
        self.page_open
    }

    /// Opens the detail as a page of its own on the big screen and focuses it; declined where it stands beside, is absent or already open.
    pub fn open_detail(&mut self) -> bool {
        if self.form != WalkForm::BigScreen || self.detail_beside || !self.has_detail || self.detail_open {
            return false;
        }
        self.detail_open = true;
        self.events.push(WalkEvent::DetailOpened);
        self.focus_role(Role::Detail);
        true
    }

    /// Closes the detail's page and gives the focus back to the content.
    pub fn close_detail(&mut self) -> bool {
        if !self.is_detail_open() {
            return false;
        }
        self.detail_open = false;
        self.events.push(WalkEvent::DetailClosed);
        self.focus_role(Role::Content);
        true
    }

    /// A row's page opened by the host (a card turned to a page of its own); B closes it first.
    pub fn open_page(&mut self) {
        self.page_open = true;
        self.focus_role(Role::Detail);
    }

    pub fn close_page(&mut self) -> bool {
        if !self.page_open {
            return false;
        }
        self.page_open = false;
        self.events.push(WalkEvent::PageClosed);
        self.focus_role(Role::Content);
        true
    }

    /// A pad button in ES-DE's grammar; false means the walk declined it and it is the host's, such as B at the top closing the window.
    pub fn press(&mut self, button: UiButton) -> bool {
        let button = button.swapped(self.swap_accept_back);
        match button {
            UiButton::PageUp if self.form != WalkForm::Card => return self.step_section(-1),
            UiButton::PageDown if self.form != WalkForm::Card => return self.step_section(1),
            UiButton::Accept
                if self.form == WalkForm::BigScreen
                    && !self.is_detail_open()
                    && !self.page_open
                    && matches!(self.focused, None | Some(Role::Content)) =>
            {
                return self.open_detail();
            }
            UiButton::Back => {
                if self.close_page() || self.close_detail() {
                    return true;
                }
                return match self.focused {
                    Some(Role::Detail | Role::Actions | Role::Search) => self.focus_role(Role::Content),
                    Some(Role::Content) if self.form == WalkForm::Desktop => self.focus_role(Role::Navigation),
                    _ => false,
                };
            }
            _ => {}
        }
        let Some(action) = self.actions.iter().find(|a| a.button == Some(button)) else { return false };
        if !action.enabled {
            return false;
        }
        let key = action.key.clone();
        self.events.push(WalkEvent::ActionInvoked(key));
        true
    }

    /// Everything the walk did since the last call, oldest first.
    pub fn take_events(&mut self) -> Vec<WalkEvent> {
        std::mem::take(&mut self.events)
    }
}
