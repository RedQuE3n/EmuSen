/// What a pad asks of the interface, named by what it does rather than by the button (EmuSen_Settings_Reference.md §4.29).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UiButton {
    Up,
    Down,
    Left,
    Right,
    Accept,
    Back,
    /// Start: the menu.
    Menu,
    /// Select: the options, a favourite, the List card.
    Options,
    /// The left shoulder: a page, or the section before.
    PageUp,
    /// The right shoulder: a page, or the section after.
    PageDown,
    /// The left trigger: the first entry.
    First,
    /// The right trigger: the last entry.
    Last,
    Guide,
    Search,
    Random,
    Screensaver,
}

/// A key the keyboard shares with the pad's grammar, and the key a declined press becomes for a stock control (EmuSen_Qt.md §1.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Return,
    Escape,
    Tab,
    Menu,
    Select,
    PageUp,
    PageDown,
    Home,
    End,
    /// Any key the grammar has no meaning for.
    Other,
}

impl UiButton {
    pub const ALL: [UiButton; 16] = [
        UiButton::Up,
        UiButton::Down,
        UiButton::Left,
        UiButton::Right,
        UiButton::Accept,
        UiButton::Back,
        UiButton::Menu,
        UiButton::Options,
        UiButton::PageUp,
        UiButton::PageDown,
        UiButton::First,
        UiButton::Last,
        UiButton::Guide,
        UiButton::Search,
        UiButton::Random,
        UiButton::Screensaver,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    /// The four directions and the two shoulders repeat while held; nothing else does, so a held Accept never answers its own prompt.
    pub const fn repeats(self) -> bool {
        matches!(self, UiButton::Up | UiButton::Down | UiButton::Left | UiButton::Right | UiButton::PageUp | UiButton::PageDown)
    }

    pub const fn is_direction(self) -> bool {
        matches!(self, UiButton::Up | UiButton::Down | UiButton::Left | UiButton::Right)
    }

    /// The button with the player's swap applied: Accept and Back trade places, and so do Search and Screensaver, the
    /// North and West buttons, as the C# front end and ES-DE trade them (decided 2026-11-08); nothing else moves.
    pub const fn swapped(self, swap: bool) -> UiButton {
        match (swap, self) {
            (true, UiButton::Accept) => UiButton::Back,
            (true, UiButton::Back) => UiButton::Accept,
            (true, UiButton::Search) => UiButton::Screensaver,
            (true, UiButton::Screensaver) => UiButton::Search,
            _ => self,
        }
    }

    /// The key a press becomes when every owner declines it, so a stock control answers the pad with no code of its own.
    pub const fn key(self) -> Option<Key> {
        match self {
            UiButton::Up => Some(Key::Up),
            UiButton::Down => Some(Key::Down),
            UiButton::Left => Some(Key::Left),
            UiButton::Right => Some(Key::Right),
            UiButton::Accept => Some(Key::Return),
            UiButton::Back => Some(Key::Escape),
            UiButton::Menu => Some(Key::Menu),
            UiButton::Options => Some(Key::Select),
            _ => None,
        }
    }
}

impl Key {
    /// The press a key means in the grammar; Tab walks instead and is not a press.
    pub const fn button(self) -> Option<UiButton> {
        match self {
            Key::Up => Some(UiButton::Up),
            Key::Down => Some(UiButton::Down),
            Key::Left => Some(UiButton::Left),
            Key::Right => Some(UiButton::Right),
            Key::Return => Some(UiButton::Accept),
            Key::Escape => Some(UiButton::Back),
            Key::Menu => Some(UiButton::Menu),
            Key::Select => Some(UiButton::Options),
            Key::PageUp => Some(UiButton::PageUp),
            Key::PageDown => Some(UiButton::PageDown),
            Key::Home => Some(UiButton::First),
            Key::End => Some(UiButton::Last),
            Key::Tab | Key::Other => None,
        }
    }
}
