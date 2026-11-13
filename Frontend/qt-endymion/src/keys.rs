//! A key pressed on Qt, in the word Endymion's binding files name it by: Avalonia 12's `Key` names, the vocabulary the
//! files were first written in (emusen_platform.h, EMUSEN_ENDYMION_BINDINGS_*; EmuSen_Input.md §5.2). Qt's numbers are
//! qnamespace.h's; this table is the one place the two meet (EmuSen_Qt.md §7.7.H).

/// What a key pressed while a binding listens is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Heard {
    /// A key the files name, by its word.
    Key(&'static str),
    /// Escape: the listening ends and nothing changes.
    Escape,
    /// A modifier alone, which is no binding (Mistress's `IsModifier`), or a key the files have no word for.
    Nothing,
}

/// Qt::KeypadModifier.
const KEYPAD: i32 = 0x2000_0000;

/// Qt's keys with a word of their own, its number first.
const NAMED: &[(i32, &str)] = &[
    (0x0100_0001, "Tab"),
    (0x0100_0002, "Tab"),
    (0x0100_0003, "Back"),
    (0x0100_0004, "Return"),
    (0x0100_0005, "Return"),
    (0x0100_0006, "Insert"),
    (0x0100_0007, "Delete"),
    (0x0100_0008, "Pause"),
    (0x0100_0009, "PrintScreen"),
    (0x0100_0010, "Home"),
    (0x0100_0011, "End"),
    (0x0100_0012, "Left"),
    (0x0100_0013, "Up"),
    (0x0100_0014, "Right"),
    (0x0100_0015, "Down"),
    (0x0100_0016, "PageUp"),
    (0x0100_0017, "PageDown"),
    (0x0100_0024, "CapsLock"),
    (0x0100_0025, "NumLock"),
    (0x0100_0026, "Scroll"),
    (0x20, "Space"),
    // Each punctuation key by its own word, shifted or not.
    (0x2e, "OemPeriod"),
    (0x3e, "OemPeriod"),
    (0x2c, "OemComma"),
    (0x3c, "OemComma"),
    (0x2d, "OemMinus"),
    (0x5f, "OemMinus"),
    (0x3d, "OemPlus"),
    (0x2b, "OemPlus"),
    (0x2f, "OemQuestion"),
    (0x3f, "OemQuestion"),
    (0x3b, "OemSemicolon"),
    (0x3a, "OemSemicolon"),
    (0x27, "OemQuotes"),
    (0x22, "OemQuotes"),
    (0x5b, "OemOpenBrackets"),
    (0x7b, "OemOpenBrackets"),
    (0x5d, "OemCloseBrackets"),
    (0x7d, "OemCloseBrackets"),
    (0x5c, "OemPipe"),
    (0x7c, "OemPipe"),
    (0x60, "OemTilde"),
    (0x7e, "OemTilde"),
    // The digits' shifted signs on a US keyboard, as their digits.
    (0x21, "D1"),
    (0x40, "D2"),
    (0x23, "D3"),
    (0x24, "D4"),
    (0x25, "D5"),
    (0x5e, "D6"),
    (0x26, "D7"),
    (0x2a, "D8"),
    (0x28, "D9"),
    (0x29, "D0"),
];

/// Qt's modifier keys: Shift, Control, Meta, Alt, AltGr.
const MODIFIERS: [i32; 5] = [0x0100_0020, 0x0100_0021, 0x0100_0022, 0x0100_0023, 0x0100_1103];

const LETTERS: [&str; 26] = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"];
const DIGITS: [&str; 10] = ["D0", "D1", "D2", "D3", "D4", "D5", "D6", "D7", "D8", "D9"];
const PAD_DIGITS: [&str; 10] = ["NumPad0", "NumPad1", "NumPad2", "NumPad3", "NumPad4", "NumPad5", "NumPad6", "NumPad7", "NumPad8", "NumPad9"];
const FUNCTIONS: [&str; 24] = [
    "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12", "F13", "F14", "F15", "F16", "F17", "F18", "F19", "F20", "F21", "F22", "F23",
    "F24",
];

/// Escape as the binding files name it (the Exit to Library hotkey's key by default).
pub const ESCAPE: &str = "Escape";

/// A key pressed over a running game, as the files name it, Escape among them; none for a key they have no word for.
pub fn word(key: i32, modifiers: i32) -> Option<&'static str> {
    match heard(key, modifiers) {
        Heard::Key(word) => Some(word),
        Heard::Escape => Some(ESCAPE),
        Heard::Nothing => None,
    }
}

/// A key pressed, by Qt's number and modifiers, as the binding files name it.
pub fn heard(key: i32, modifiers: i32) -> Heard {
    if key == 0x0100_0000 {
        return Heard::Escape;
    }
    if MODIFIERS.contains(&key) {
        return Heard::Nothing;
    }
    if modifiers & KEYPAD != 0 {
        let word = match key {
            0x30..=0x39 => Some(PAD_DIGITS[(key - 0x30) as usize]),
            0x2a => Some("Multiply"),
            0x2b => Some("Add"),
            0x2d => Some("Subtract"),
            0x2e => Some("Decimal"),
            0x2f => Some("Divide"),
            _ => None,
        };
        if let Some(word) = word {
            return Heard::Key(word);
        }
    }
    let word = match key {
        0x41..=0x5a => Some(LETTERS[(key - 0x41) as usize]),
        0x30..=0x39 => Some(DIGITS[(key - 0x30) as usize]),
        0x0100_0030..=0x0100_0047 => Some(FUNCTIONS[(key - 0x0100_0030) as usize]),
        _ => NAMED.iter().find(|(k, _)| *k == key).map(|(_, w)| *w),
    };
    word.map_or(Heard::Nothing, Heard::Key)
}
