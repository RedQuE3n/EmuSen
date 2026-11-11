//! Notices: one at a time, a second replacing the first, held 2 s; over running gameplay only those raised as fit for it,
//! the rest waiting for a card or a screen (EmuSen_Interface.md §18.6, EmuSen_Kit.md §7.23). The words are the host's.

/// How long a notice stands before it leaves, in milliseconds.
pub const HOLD_MS: u64 = 2000;

/// The status glyph beside the words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Success,
    Attention,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    pub words: String,
    pub tone: Tone,
    /// Shown over running gameplay: Saved, Loaded and the screenshot.
    pub over_game: bool,
}

/// The notices of one window: the one shown, the one waiting for gameplay to stop, and whether gameplay runs.
#[derive(Clone, Debug, Default)]
pub struct Notices {
    shown: Option<Notice>,
    waiting: Option<Notice>,
    game_running: bool,
}

impl Notices {
    pub fn shown(&self) -> Option<&Notice> {
        self.shown.as_ref()
    }

    pub fn waiting(&self) -> Option<&Notice> {
        self.waiting.as_ref()
    }

    /// Raises a notice; true when it is shown now (its hold starts again), false when it waits for gameplay to stop.
    pub fn raise(&mut self, notice: Notice) -> bool {
        if self.game_running && !notice.over_game {
            self.waiting = Some(notice);
            return false;
        }
        self.shown = Some(notice);
        true
    }

    /// The hold ran out.
    pub fn expire(&mut self) {
        self.shown = None;
    }

    /// Gameplay starts or stops; true when the notice shown changed. A notice unfit for gameplay goes back to waiting as
    /// it starts, and the one waiting is shown as it stops.
    pub fn set_game_running(&mut self, running: bool) -> bool {
        if running == self.game_running {
            return false;
        }
        self.game_running = running;
        if running {
            if self.shown.as_ref().is_some_and(|n| !n.over_game) {
                self.waiting = self.shown.take();
                return true;
            }
            return false;
        }
        match self.waiting.take() {
            Some(notice) => {
                self.shown = Some(notice);
                true
            }
            None => false,
        }
    }
}
