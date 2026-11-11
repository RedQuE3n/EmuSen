//! The classes of finding the tester has accepted, each with the ruling that accepts it; the one table every gate reads (EmuSen_Kit.md §7.102).

/// A class of finding a ruling may accept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    RomFolderCutAtStart,
    SpareTenth,
    FadedEdge,
    SceneColour,
    MoonriseCaption,
    CpuConversionLine,
    DimmedAndMoving,
    DataWordsElided,
    BridgeDuplicates,
}

/// One accepted class: the ruling's id in the rulings log and what it covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Accepted {
    pub class: Class,
    pub ruling: &'static str,
    pub covers: &'static str,
}

/// Every class the tester has accepted; a row is added only by a ruling of hers.
pub const ACCEPTED: [Accepted; 9] = [
    Accepted {
        class: Class::RomFolderCutAtStart,
        ruling: "R-20261111-36",
        covers: "the ROM folder's path cut at its start so the folder's name shows, the pane giving it whole",
    },
    Accepted { class: Class::SpareTenth, ruling: "R-20261111-37", covers: "a tenth of the samples behind words failing the contrast rule" },
    Accepted { class: Class::FadedEdge, ruling: "R-20261111-38", covers: "words under their contrast where they cross a scroller's faded edge" },
    Accepted { class: Class::SceneColour, ruling: "R-20261111-39", covers: "colours painted inside a theme's scene, outside the palette rule" },
    Accepted {
        class: Class::MoonriseCaption,
        ruling: "R-20261111-40",
        covers: "Moonrise's caption under the text floor, until a new splash replaces Moonrise",
    },
    Accepted { class: Class::CpuConversionLine, ruling: "R-20261111-41", covers: "Qt Multimedia's CPU-conversion log line on the software backend" },
    Accepted {
        class: Class::DimmedAndMoving,
        ruling: "R-20261111-43",
        covers: "dimmed (declined) places exempt from contrast, and moving words not judged",
    },
    Accepted { class: Class::DataWordsElided, ruling: "R-20261111-44", covers: "data words, such as a game's title, cut with an ellipsis" },
    Accepted { class: Class::BridgeDuplicates, ruling: "R-20261111-46", covers: "the 22 duplicate-code pairs of the cxx-qt bridges in dupes.accept" },
];

/// The row of `table` that accepts `class`, if any.
pub fn accepted_in(table: &'static [Accepted], class: Class) -> Option<&'static Accepted> {
    table.iter().find(|a| a.class == class)
}

/// The row of `ACCEPTED` that accepts `class`, if any.
pub fn accepted(class: Class) -> Option<&'static Accepted> {
    accepted_in(&ACCEPTED, class)
}
