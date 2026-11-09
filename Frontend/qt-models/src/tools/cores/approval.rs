//! The approval question, one set of words for Cores and for a game's first launch (EmuSen_Interface.md §15.6;
//! EmuSen_Cores.md §5.2): what the core says it is, where it came from, and its hash, as C#'s `CoreApproval`. A core's
//! descriptors are shown, never interpreted.

use super::words::grouped;
use crate::question::Question;

/// What the question shows of a core.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Asking {
    pub name: String,
    /// Its library's file name, or a libretro core's `<stem>_libretro`.
    pub file: String,
    pub authors: Vec<String>,
    pub licence: String,
    /// The consoles it runs, as a list names them.
    pub consoles: Vec<String>,
    pub sha256: String,
    /// Downloaded from libretro's buildbot, rather than added by the player.
    pub downloaded: bool,
    /// The host it came from and when, for a downloaded core.
    pub from: Option<String>,
    pub when: Option<String>,
    /// What the project found of this build.
    pub tested: Option<String>,
}

/// The question's sentences, without the small print.
pub fn sentences(a: &Asking) -> Vec<String> {
    let by = if a.authors.is_empty() { "authors not said".to_owned() } else { a.authors.join(", ") };
    let under = if a.licence.is_empty() { "a licence not said".to_owned() } else { format!("the {}", a.licence) };
    let runs = if a.consoles.is_empty() { "games of no console EmuSen knows".to_owned() } else { format!("{} games", a.consoles.join(" and ")) };
    let by = format!("By {by}, under {under}; it runs {runs}.");
    if a.downloaded {
        vec![
            format!(
                "{} was downloaded from {} {} and has not been approved. {by}",
                a.name,
                a.from.as_deref().unwrap_or("libretro's buildbot"),
                a.when.as_deref().unwrap_or("earlier")
            ),
            "libretro signs no build, so approving trusts the build the buildbot served that day. The hash below holds EmuSen to exactly that build, and EmuSen's own proxy runs it in a sandbox. It runs with your permissions: approve it only if you trust where it came from. If the file changes, you will be asked again.".to_owned(),
        ]
    } else {
        vec![
            format!("{}, in {}, did not ship with EmuSen and has not been approved. {by}", a.name, a.file),
            "It runs with your full permissions: approve it only if you trust where it came from. If the file changes, you will be asked again.".to_owned(),
        ]
    }
}

/// What the project found of the build, then the hash in groups of eight: the question's small print, in Caption.
pub fn small_print(a: &Asking) -> String {
    let tested = a.tested.as_deref().filter(|t| !t.is_empty()).map(|t| format!("{t}\n\n")).unwrap_or_default();
    format!("{tested}SHA-256\n{}", grouped(&a.sha256))
}

/// The question in the question form: Not Now first and focused, then Approve; B answers Not Now.
pub fn question(a: &Asking) -> Question {
    Question {
        title: format!("Approve {}?", a.name),
        sentences: sentences(a),
        caption: small_print(a),
        choices: vec!["Not Now".to_owned(), "Approve".to_owned()],
        back: Some(0),
    }
}
