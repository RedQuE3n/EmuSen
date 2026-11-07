use super::{Key, Press, UiButton};

/// An element a pointer can rest on or press: the host's handle, meaningless to the kit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ElementId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerButton {
    /// A click, a tap: the same as A on the element.
    Primary,
    /// A right click: B.
    Secondary,
    /// The mouse's back button: B.
    Back,
}

/// One event from any of the three ways in, so the Qt layer only translates (EmuSen_Qt.md §1.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEvent {
    /// A press the grammar has already made of what the pad holds.
    Pad(Press),
    /// The pointer came to rest on an element.
    Hover(ElementId),
    /// A pointer button went down on an element.
    Click { target: ElementId, button: PointerButton },
    /// The wheel turned: positive notches go forward, negative back.
    Wheel { notches: i32 },
    /// A key went down, with whether Shift was held.
    Key { key: Key, shift: bool },
}

/// What an event asks of the kit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intent {
    /// A press in the pad's grammar, from the pad, a key or a pointer button that means one.
    Press(UiButton),
    /// Move the focus ring to the element, as the walk would, without activating it.
    Focus(ElementId),
    /// Activate the element: the same view-model call as A there.
    Activate(ElementId),
    /// Scroll a list or turn a carousel this many steps, one per notch.
    Step(i32),
    /// Walk the roles forward or back, as Tab and Shift+Tab do.
    Walk(i32),
}

/// The rules of EmuSen_Qt.md §1.7: hover focuses, a click is A, the wheel steps, the back button and a right click are B, Tab walks.
pub fn intent(event: InputEvent) -> Option<Intent> {
    match event {
        InputEvent::Pad(press) => Some(Intent::Press(press.button)),
        InputEvent::Hover(target) => Some(Intent::Focus(target)),
        InputEvent::Click { target, button: PointerButton::Primary } => Some(Intent::Activate(target)),
        InputEvent::Click { button: PointerButton::Secondary | PointerButton::Back, .. } => Some(Intent::Press(UiButton::Back)),
        InputEvent::Wheel { notches: 0 } => None,
        InputEvent::Wheel { notches } => Some(Intent::Step(notches)),
        InputEvent::Key { key: Key::Tab, shift } => Some(Intent::Walk(if shift { -1 } else { 1 })),
        InputEvent::Key { key, .. } => key.button().map(Intent::Press),
    }
}
