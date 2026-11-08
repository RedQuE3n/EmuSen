//! The words the binding files are written in: the pad's buttons and controls, a gamepad's buttons as SDL3 names them,
//! the keyboard's keys, and the emulator's actions. Each word is read from its name in any case or from its number, and
//! written by its name. See EmuSen_Input.md §5.2.

use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize, Serializer};
use std::marker::PhantomData;

/// A set of words: what one is called, and each name with its number. A number's first name is the one written.
pub trait Vocabulary {
    const WHAT: &'static str;
    const WORDS: &'static [(&'static str, i32)];
}

/// A word of one vocabulary, held by its number, so that two names of one number are one word.
pub struct Word<V>(i32, PhantomData<V>);

impl<V: Vocabulary> Word<V> {
    /// The word of a name, matched without regard to case.
    pub fn named(name: &str) -> Option<Word<V>> {
        V::WORDS.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|&(_, number)| Word(number, PhantomData))
    }

    /// The word of a number, when the vocabulary has one.
    pub fn numbered(number: i64) -> Option<Word<V>> {
        V::WORDS.iter().find(|&&(_, n)| i64::from(n) == number).map(|&(_, n)| Word(n, PhantomData))
    }

    pub fn number(self) -> i32 {
        self.0
    }

    /// The name the word is written by.
    pub fn name(self) -> &'static str {
        V::WORDS.iter().find(|&&(_, n)| n == self.0).map(|&(name, _)| name).expect("a word is one of its vocabulary's")
    }

    /// A text as a word: a name in any case or a number, with the spaces around it ignored; otherwise the words of the refusal.
    pub fn parse(text: &str) -> Result<Word<V>, String> {
        let trimmed = text.trim();
        let found = match trimmed.parse::<i64>() {
            Ok(number) => Word::numbered(number),
            Err(_) => Word::named(trimmed),
        };
        found.ok_or_else(|| {
            let names: Vec<&str> = V::WORDS.iter().map(|&(name, _)| name).collect();
            format!("'{text}' is not a valid {}.{}", V::WHAT, emusen_suggest::hint(text, &names, 2))
        })
    }
}

impl<V> Clone for Word<V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V> Copy for Word<V> {}

impl<V> PartialEq for Word<V> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<V> Eq for Word<V> {}

impl<V> std::hash::Hash for Word<V> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<V: Vocabulary> std::fmt::Debug for Word<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl<V: Vocabulary> Serialize for Word<V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

struct WordVisitor<V>(PhantomData<V>);

impl<V: Vocabulary> Visitor<'_> for WordVisitor<V> {
    type Value = Word<V>;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a {} by name or number", V::WHAT)
    }

    fn visit_str<E: de::Error>(self, text: &str) -> Result<Word<V>, E> {
        Word::parse(text).map_err(E::custom)
    }

    fn visit_i64<E: de::Error>(self, number: i64) -> Result<Word<V>, E> {
        Word::numbered(number).ok_or_else(|| E::custom(format!("{number} is not a valid {}.", V::WHAT)))
    }

    fn visit_u64<E: de::Error>(self, number: u64) -> Result<Word<V>, E> {
        i64::try_from(number).map_err(|_| E::custom(format!("{number} is not a valid {}.", V::WHAT))).and_then(|n| self.visit_i64(n))
    }
}

impl<'de, V: Vocabulary> Deserialize<'de> for Word<V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(WordVisitor(PhantomData))
    }
}

/// `PadControl`'s words (`pad::CONTROLS`): the buttons under their own numbers, then each stick's directions.
const CONTROL_WORDS: &[(&str, i32)] = &[
    ("B", 0), ("Y", 1), ("Select", 2), ("Start", 3), ("Up", 4), ("Down", 5), ("Left", 6), ("Right", 7), ("A", 8), ("X", 9),
    ("L", 10), ("R", 11), ("L2", 12), ("R2", 13), ("L3", 14), ("R3", 15), ("LeftStickUp", 16), ("LeftStickDown", 17),
    ("LeftStickLeft", 18), ("LeftStickRight", 19), ("RightStickUp", 20), ("RightStickDown", 21), ("RightStickLeft", 22),
    ("RightStickRight", 23),
];

/// `PadButton`, Galaxia's generic pad's buttons in libretro's RetroPad order: the controls' first `pad::BUTTONS`.
pub enum PadButtons {}

impl Vocabulary for PadButtons {
    const WHAT: &'static str = "PadButton";
    const WORDS: &'static [(&'static str, i32)] = CONTROL_WORDS.split_at(crate::pad::BUTTONS).0;
}

/// `PadControl`: a button or a stick's direction.
pub enum PadControls {}

impl Vocabulary for PadControls {
    const WHAT: &'static str = "PadControl";
    const WORDS: &'static [(&'static str, i32)] = CONTROL_WORDS;
}

/// A gamepad's buttons by SDL3's `SDL_GamepadButton`, named as the files have always named them.
pub enum GamepadButtons {}

impl Vocabulary for GamepadButtons {
    const WHAT: &'static str = "GamepadButton";
    const WORDS: &'static [(&'static str, i32)] = &[
        ("Invalid", -1), ("South", 0), ("East", 1), ("West", 2), ("North", 3), ("Back", 4), ("Guide", 5), ("Start", 6),
        ("LeftStick", 7), ("RightStick", 8), ("LeftShoulder", 9), ("RightShoulder", 10), ("DPadUp", 11), ("DPadDown", 12),
        ("DPadLeft", 13), ("DPadRight", 14), ("Misc1", 15), ("RightPaddle1", 16), ("LeftPaddle1", 17), ("RightPaddle2", 18),
        ("LeftPaddle2", 19), ("Touchpad", 20), ("Misc2", 21), ("Misc3", 22), ("Misc4", 23), ("Misc5", 24), ("Misc6", 25),
        ("Count", 26),
    ];
}

/// The emulator's actions a key can be bound to.
pub enum HotkeyActions {}

impl Vocabulary for HotkeyActions {
    const WHAT: &'static str = "HotkeyAction";
    const WORDS: &'static [(&'static str, i32)] = &[
        ("FastForward", 0), ("Rewind", 1), ("SaveState", 2), ("LoadState", 3), ("TogglePause", 4), ("ToggleFullscreen", 5),
        ("ExitToLibrary", 6), ("Screenshot", 7), ("ToggleBigPicture", 8), ("FrameAdvance", 9),
    ];
}

/// The keyboard's keys, by the names and numbers of Avalonia 12's `Key`, which the files were first written in.
pub enum Keys {}

impl Vocabulary for Keys {
    const WHAT: &'static str = "Key";
    const WORDS: &'static [(&'static str, i32)] = &[
    ("None", 0), ("Cancel", 1), ("Back", 2), ("Tab", 3), ("LineFeed", 4), ("Clear", 5), ("Return", 6), ("Enter", 6),
    ("Pause", 7), ("CapsLock", 8), ("Capital", 8), ("HangulMode", 9), ("KanaMode", 9), ("JunjaMode", 10),
    ("FinalMode", 11), ("HanjaMode", 12), ("KanjiMode", 12), ("Escape", 13), ("ImeConvert", 14),
    ("ImeNonConvert", 15), ("ImeAccept", 16), ("ImeModeChange", 17), ("Space", 18), ("PageUp", 19), ("Prior", 19),
    ("PageDown", 20), ("Next", 20), ("End", 21), ("Home", 22), ("Left", 23), ("Up", 24), ("Right", 25), ("Down", 26),
    ("Select", 27), ("Print", 28), ("Execute", 29), ("PrintScreen", 30), ("Snapshot", 30), ("Insert", 31),
    ("Delete", 32), ("Help", 33), ("D0", 34), ("D1", 35), ("D2", 36), ("D3", 37), ("D4", 38), ("D5", 39), ("D6", 40),
    ("D7", 41), ("D8", 42), ("D9", 43), ("A", 44), ("B", 45), ("C", 46), ("D", 47), ("E", 48), ("F", 49), ("G", 50),
    ("H", 51), ("I", 52), ("J", 53), ("K", 54), ("L", 55), ("M", 56), ("N", 57), ("O", 58), ("P", 59), ("Q", 60),
    ("R", 61), ("S", 62), ("T", 63), ("U", 64), ("V", 65), ("W", 66), ("X", 67), ("Y", 68), ("Z", 69), ("LWin", 70),
    ("RWin", 71), ("Apps", 72), ("Sleep", 73), ("NumPad0", 74), ("NumPad1", 75), ("NumPad2", 76), ("NumPad3", 77),
    ("NumPad4", 78), ("NumPad5", 79), ("NumPad6", 80), ("NumPad7", 81), ("NumPad8", 82), ("NumPad9", 83),
    ("Multiply", 84), ("Add", 85), ("Separator", 86), ("Subtract", 87), ("Decimal", 88), ("Divide", 89), ("F1", 90),
    ("F2", 91), ("F3", 92), ("F4", 93), ("F5", 94), ("F6", 95), ("F7", 96), ("F8", 97), ("F9", 98), ("F10", 99),
    ("F11", 100), ("F12", 101), ("F13", 102), ("F14", 103), ("F15", 104), ("F16", 105), ("F17", 106), ("F18", 107),
    ("F19", 108), ("F20", 109), ("F21", 110), ("F22", 111), ("F23", 112), ("F24", 113), ("NumLock", 114),
    ("Scroll", 115), ("LeftShift", 116), ("RightShift", 117), ("LeftCtrl", 118), ("RightCtrl", 119), ("LeftAlt", 120),
    ("RightAlt", 121), ("BrowserBack", 122), ("BrowserForward", 123), ("BrowserRefresh", 124), ("BrowserStop", 125),
    ("BrowserSearch", 126), ("BrowserFavorites", 127), ("BrowserHome", 128), ("VolumeMute", 129), ("VolumeDown", 130),
    ("VolumeUp", 131), ("MediaNextTrack", 132), ("MediaPreviousTrack", 133), ("MediaStop", 134),
    ("MediaPlayPause", 135), ("LaunchMail", 136), ("SelectMedia", 137), ("LaunchApplication1", 138),
    ("LaunchApplication2", 139), ("OemSemicolon", 140), ("Oem1", 140), ("OemPlus", 141), ("OemComma", 142),
    ("OemMinus", 143), ("OemPeriod", 144), ("OemQuestion", 145), ("Oem2", 145), ("Oem3", 146), ("OemTilde", 146),
    ("AbntC1", 147), ("AbntC2", 148), ("Oem4", 149), ("OemOpenBrackets", 149), ("OemPipe", 150), ("Oem5", 150),
    ("OemCloseBrackets", 151), ("Oem6", 151), ("OemQuotes", 152), ("Oem7", 152), ("Oem8", 153), ("OemBackslash", 154),
    ("Oem102", 154), ("ImeProcessed", 155), ("System", 156), ("OemAttn", 157), ("DbeAlphanumeric", 157),
    ("OemFinish", 158), ("DbeKatakana", 158), ("OemCopy", 159), ("DbeHiragana", 159), ("OemAuto", 160),
    ("DbeSbcsChar", 160), ("OemEnlw", 161), ("DbeDbcsChar", 161), ("OemBackTab", 162), ("DbeRoman", 162),
    ("DbeNoRoman", 163), ("Attn", 163), ("CrSel", 164), ("DbeEnterWordRegisterMode", 164), ("ExSel", 165),
    ("DbeEnterImeConfigureMode", 165), ("DbeFlushString", 166), ("EraseEof", 166), ("DbeCodeInput", 167),
    ("Play", 167), ("Zoom", 168), ("DbeNoCodeInput", 168), ("NoName", 169), ("DbeDetermineString", 169),
    ("DbeEnterDialogConversionMode", 170), ("Pa1", 170), ("OemClear", 171), ("DeadCharProcessed", 172),
    ("FnLeftArrow", 10001), ("FnRightArrow", 10002), ("FnUpArrow", 10003), ("FnDownArrow", 10004),
    ("MediaHome", 100000), ("MediaChannelList", 100001), ("MediaChannelRaise", 100002), ("MediaChannelLower", 100003),
    ("MediaRecord", 100005), ("MediaRed", 100010), ("MediaGreen", 100011), ("MediaYellow", 100012),
    ("MediaBlue", 100013), ("MediaMenu", 100020), ("MediaMore", 100021), ("MediaOption", 100022),
    ("MediaInfo", 100023), ("MediaSearch", 100024), ("MediaSubtitle", 100025), ("MediaTvGuide", 100026),
    ("MediaPreviousChannel", 100027),
    ];
}
