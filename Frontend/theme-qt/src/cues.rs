//! `Cues`, the seven navigation sounds a theme may ring, by EmuSen_Interface.md §23's names (EmuSen_Qt.md §3.2): each
//! passed to the host's player, which rings it through Endymion at the player's volume. With no player, silence.

use core::pin::Pin;
use cxx_qt::CxxQtType;

#[cxx_qt::bridge]
pub mod qobject {
    extern "RustQt" {
        #[qobject]
        #[qml_element = "Cues"]
        #[qml_singleton]
        type ThemeCues = super::ThemeCuesRust;
    }

    extern "RustQt" {
        #[qinvokable]
        fn scroll(self: Pin<&mut ThemeCues>);
        #[qinvokable]
        fn systembrowse(self: Pin<&mut ThemeCues>);
        #[qinvokable]
        fn select(self: Pin<&mut ThemeCues>);
        #[qinvokable]
        fn back(self: Pin<&mut ThemeCues>);
        #[qinvokable]
        fn favorite(self: Pin<&mut ThemeCues>);
        #[qinvokable]
        fn launch(self: Pin<&mut ThemeCues>);
        #[qinvokable]
        fn quicksysselect(self: Pin<&mut ThemeCues>);
    }
}

/// A cue, by §23's name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    Scroll,
    SystemBrowse,
    Select,
    Back,
    Favorite,
    Launch,
    QuickSysSelect,
}

impl Cue {
    /// Every cue, in §23's order.
    pub const ALL: [Cue; 7] = [Cue::Scroll, Cue::SystemBrowse, Cue::Select, Cue::Back, Cue::Favorite, Cue::Launch, Cue::QuickSysSelect];

    /// Its name in §23 and in a theme's `theme.json`.
    pub fn name(self) -> &'static str {
        match self {
            Cue::Scroll => "scroll",
            Cue::SystemBrowse => "systembrowse",
            Cue::Select => "select",
            Cue::Back => "back",
            Cue::Favorite => "favorite",
            Cue::Launch => "launch",
            Cue::QuickSysSelect => "quicksysselect",
        }
    }
}

/// The host's side: rings a cue.
pub trait CuePlayer {
    fn ring(&mut self, cue: Cue);
}

#[derive(Default)]
pub struct ThemeCuesRust {
    player: Option<Box<dyn CuePlayer>>,
}

impl qobject::ThemeCues {
    /// The host's player; none is silence.
    pub fn set_player(mut self: Pin<&mut Self>, player: Option<Box<dyn CuePlayer>>) {
        self.as_mut().rust_mut().player = player;
    }

    fn ring(mut self: Pin<&mut Self>, cue: Cue) {
        if let Some(player) = self.as_mut().rust_mut().player.as_mut() {
            player.ring(cue);
        }
    }

    pub fn scroll(self: Pin<&mut Self>) {
        self.ring(Cue::Scroll)
    }

    pub fn systembrowse(self: Pin<&mut Self>) {
        self.ring(Cue::SystemBrowse)
    }

    pub fn select(self: Pin<&mut Self>) {
        self.ring(Cue::Select)
    }

    pub fn back(self: Pin<&mut Self>) {
        self.ring(Cue::Back)
    }

    pub fn favorite(self: Pin<&mut Self>) {
        self.ring(Cue::Favorite)
    }

    pub fn launch(self: Pin<&mut Self>) {
        self.ring(Cue::Launch)
    }

    pub fn quicksysselect(self: Pin<&mut Self>) {
        self.ring(Cue::QuickSysSelect)
    }
}
