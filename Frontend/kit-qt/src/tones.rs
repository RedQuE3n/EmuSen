//! `Tones`: the colour a screen's words are drawn in, named by what they say, so a host's model hands a tone and never a
//! colour (EmuSen_Kit.md §7.39).

#[cxx_qt::bridge]
pub mod ffi {
    #[qml_element]
    qnamespace!("Tones");

    /// A token's role for words: the screen's text, muted facts, strong titles, the accent, and the three states.
    #[qenum]
    #[namespace = "Tones"]
    enum Tone {
        Text,
        Muted,
        Strong,
        Accent,
        Success,
        Attention,
        Error,
    }
}
