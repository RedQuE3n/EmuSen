use super::UiButton;

/// What the Guide chords read this tick: Guide held with one of `GuideChords::BUTTONS` going down, or Guide let go alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuideChord {
    With(UiButton),
    Alone,
}

/// Guide held with Up, Down or the right shoulder, read from the held buttons each tick; a button already down when Guide
/// went down is no chord, and Guide let go with nothing chorded is `Alone` (EmuSen_Kit.md §7.26, Mistress's `GuideChordReader`).
#[derive(Clone, Debug, Default)]
pub struct GuideChords {
    guide: bool,
    used: bool,
    was: [bool; 3],
}

impl GuideChords {
    /// The buttons a chord is made with, the first going down winning a tick.
    pub const BUTTONS: [UiButton; 3] = [UiButton::Up, UiButton::Down, UiButton::PageDown];

    pub fn feed(&mut self, held: impl Fn(UiButton) -> bool) -> Option<GuideChord> {
        if !held(UiButton::Guide) {
            let alone = self.guide && !self.used;
            self.guide = false;
            return alone.then_some(GuideChord::Alone);
        }
        let now = Self::BUTTONS.map(&held);
        let mut chord = None;
        if self.guide {
            chord = (0..now.len()).find(|&i| now[i] && !self.was[i]).map(|i| GuideChord::With(Self::BUTTONS[i]));
        } else {
            self.used = false;
        }
        self.used |= chord.is_some();
        self.guide = true;
        self.was = now;
        chord
    }

    /// Forgets a Guide held across a menu, so letting it go opens nothing.
    pub fn reset(&mut self, guide_held: bool) {
        self.guide = guide_held;
        self.used = true;
    }

    pub fn guide_held(&self) -> bool {
        self.guide
    }
}
