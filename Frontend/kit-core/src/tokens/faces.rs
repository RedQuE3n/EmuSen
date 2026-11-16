//! The face tokens: the family each role of text draws in, one of the faces the kit bundles or the platform's (EmuSen_Kit.md §7.117).

/// The families of the faces the kit carries in `kit-qt/fonts/`, as Qt's font database names each file's family; a test loads
/// every file there and holds this list to them.
pub const BUNDLED: [&str; 4] = ["Barlow", "Barlow Condensed", "Sora", "JetBrains Mono"];

/// Whether a family is one the kit bundles, so a theme naming it is drawn in it and never in a fallback.
pub fn is_bundled(family: &str) -> bool {
    BUNDLED.contains(&family)
}

/// What a text is, for the face it draws in; its weight stays the text's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FaceRole {
    /// The screens' words, their strong words by weight.
    Text,
    /// Names, titles, counts and the wordmark, condensed in the reference.
    Label,
    /// The kit's rows, controls, sentences and the glyphs' letters, which named no face before the tokens.
    Interface,
    /// Codes: a version, a folder's path, a cheat's code.
    Code,
}

impl FaceRole {
    pub const ALL: [FaceRole; 4] = [FaceRole::Text, FaceRole::Label, FaceRole::Interface, FaceRole::Code];

    /// The `Tokens` property that carries the role's family (rule Q).
    pub const fn property(self) -> &'static str {
        match self {
            FaceRole::Text => "faceText",
            FaceRole::Label => "faceLabel",
            FaceRole::Interface => "faceInterface",
            FaceRole::Code => "faceCode",
        }
    }

    pub fn of_property(property: &str) -> Option<FaceRole> {
        FaceRole::ALL.into_iter().find(|role| role.property() == property)
    }

    /// The kit's face for the role, what the reference draws: a bundled family, or none for the platform's default font.
    pub const fn kit(self) -> Option<&'static str> {
        match self {
            FaceRole::Text => Some(BUNDLED[0]),
            FaceRole::Label => Some(BUNDLED[1]),
            FaceRole::Interface | FaceRole::Code => None,
        }
    }
}

/// A kind of text whose weight a look sets, where the reference and Eclipse's boards draw it differently (EmuSen_Kit.md §7.117).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WeightRole {
    /// A screen's, a card's, a pane's and a question's title.
    Title,
    /// The pane's sentence about the focused row.
    Sentence,
    /// The EmuSen wordmark.
    Wordmark,
}

impl WeightRole {
    pub const ALL: [WeightRole; 3] = [WeightRole::Title, WeightRole::Sentence, WeightRole::Wordmark];

    /// The `Tokens` property that carries the weight, a CSS weight as `Font.weight` takes it (rule Q).
    pub const fn property(self) -> &'static str {
        match self {
            WeightRole::Title => "weightTitle",
            WeightRole::Sentence => "weightSentence",
            WeightRole::Wordmark => "weightWordmark",
        }
    }

    pub fn of_property(property: &str) -> Option<WeightRole> {
        WeightRole::ALL.into_iter().find(|role| role.property() == property)
    }

    /// The kit's weight, what the reference draws: regular titles and sentences, the light wordmark (a design constant).
    pub const fn kit(self) -> i32 {
        match self {
            WeightRole::Title | WeightRole::Sentence => 400,
            WeightRole::Wordmark => 300,
        }
    }
}
