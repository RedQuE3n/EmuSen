//! Qt's key, modifier and mouse-button numbers against kit-core's names, the one place the kit knows them (EmuSen_Kit.md §7.3).

use emusen_kit_core::input::{Key, PointerButton};

/// Qt::Key values from qnamespace.h, in kit-core's `Key` order.
const KEYS: [(Key, i32); 13] = [
    (Key::Up, 0x0100_0013),
    (Key::Down, 0x0100_0015),
    (Key::Left, 0x0100_0012),
    (Key::Right, 0x0100_0014),
    (Key::Return, 0x0100_0004),
    (Key::Escape, 0x0100_0000),
    (Key::Tab, 0x0100_0001),
    (Key::Menu, 0x0100_0055),
    (Key::Select, 0x0101_0000),
    (Key::PageUp, 0x0100_0016),
    (Key::PageDown, 0x0100_0017),
    (Key::Home, 0x0100_0010),
    (Key::End, 0x0100_0011),
];

/// Qt::Key_Enter, the keypad's Return.
pub const KEY_ENTER: i32 = 0x0100_0005;
/// Qt::Key_Backtab, what Shift+Tab arrives as.
pub const KEY_BACKTAB: i32 = 0x0100_0002;
/// Qt::ShiftModifier.
pub const SHIFT: i32 = 0x0200_0000;

/// A kit-core key as Qt's number, for the key a declined press becomes.
pub fn qt_key(key: Key) -> Option<i32> {
    KEYS.iter().find(|(k, _)| *k == key).map(|(_, code)| *code)
}

/// Qt's key and modifiers as a kit-core key and whether Shift is held; Backtab is Tab with Shift, Enter is Return.
pub fn kit_key(qt: i32, modifiers: i32) -> (Key, bool) {
    let shift = modifiers & SHIFT != 0;
    match qt {
        KEY_BACKTAB => (Key::Tab, true),
        KEY_ENTER => (Key::Return, shift),
        _ => (KEYS.iter().find(|(_, code)| *code == qt).map(|(k, _)| *k).unwrap_or(Key::Other), shift),
    }
}

/// Qt::MouseButton as kit-core's pointer button: left, right, and the mouse's back button (XButton1).
pub fn pointer_button(qt: i32) -> Option<PointerButton> {
    match qt {
        0x1 => Some(PointerButton::Primary),
        0x2 => Some(PointerButton::Secondary),
        0x8 => Some(PointerButton::Back),
        _ => None,
    }
}
