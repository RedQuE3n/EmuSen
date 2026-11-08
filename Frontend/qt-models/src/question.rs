//! The questions the program asks, in the question form's parts (EmuSen_Interface.md §15.6, §18.3): the title, the
//! sentences, a caption, the answers with the one B gives. The safe answer comes first and is focused, so a hurried A
//! never does the thing that cannot be undone.

/// A question as the kit's `Question` shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Question {
    pub title: String,
    pub sentences: Vec<String>,
    pub caption: String,
    pub choices: Vec<String>,
    /// The answer B gives; None goes back without answering.
    pub back: Option<usize>,
}

/// A question that does something: No first and focused, then Yes; B answers No.
pub fn confirm(title: &str, sentences: &[&str]) -> Question {
    Question {
        title: title.to_string(),
        sentences: sentences.iter().map(|s| s.to_string()).collect(),
        caption: String::new(),
        choices: vec!["No".to_string(), "Yes".to_string()],
        back: Some(0),
    }
}

/// The Resume question at a game's start: Continue first and focused, Start again; the caption names the setting that
/// stops the asking; B goes back to the library (§18.3, decided 2026-11-01).
pub fn resume(game: &str) -> Question {
    Question {
        title: "Continue where you left off?".to_string(),
        sentences: vec![format!("{game} was left part of the way through. Continue it from there, or start it again.")],
        caption: "Settings ▸ EmuSen ▸ Saves and states ▸ Resume ◂ Ask each time ▸ stops the asking.".to_string(),
        choices: vec!["Continue".to_string(), "Start again".to_string()],
        back: None,
    }
}

/// The index the kit's `Question` takes for B: the answer's, or -1 to go back without one.
pub fn back_index(question: &Question) -> i32 {
    question.back.map_or(-1, |i| i as i32)
}
