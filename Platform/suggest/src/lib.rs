//! "Did you mean …?" for a name that matched nothing: the nearest candidates by optimal string alignment (a swap of
//! two neighbours is one slip), case ignored, and the end of a sentence that names them. See EmuSen_RustPlatform.md
//! §47.7 and EmuSen_Config_Reference.md §6.

/// Optimal string alignment between two names, counted in characters, case ignored.
pub fn distance(a: &str, b: &str) -> usize {
    strsim::osa_distance(&a.to_lowercase(), &b.to_lowercase())
}

/// How wrong a word of this many characters may be before a guess stops being one.
fn tolerance(length: usize) -> usize {
    match length {
        0..=3 => 1,
        4..=6 => 2,
        _ => 3,
    }
}

/// The closest candidates, closest first; one the typed text begins wins a tie, then names in order ignoring case.
/// None when the text matches a candidate exactly, since the caller then refused it for another reason.
pub fn nearest<'a>(typed: &str, candidates: &[&'a str], max: i32) -> Vec<&'a str> {
    let upper = typed.to_uppercase();
    if typed.trim().is_empty() || max <= 0 || candidates.iter().any(|c| c.to_uppercase() == upper) {
        return Vec::new();
    }
    let tolerance = tolerance(typed.chars().count());
    let scored: Vec<(&str, usize)> = candidates.iter().filter(|c| !c.is_empty()).map(|c| (*c, distance(typed, c))).filter(|(_, d)| *d > 0 && *d <= tolerance).collect();
    let Some(best) = scored.iter().map(|(_, d)| *d).min() else { return Vec::new() };
    let mut joint: Vec<&str> = scored.into_iter().filter(|(_, d)| *d == best).map(|(name, _)| name).collect();
    let begins = |name: &str| name.to_uppercase().starts_with(&upper);
    joint.sort_by(|a, b| begins(b).cmp(&begins(a)).then_with(|| a.to_uppercase().cmp(&b.to_uppercase())));
    joint.truncate(max as usize);
    joint
}

/// The suggestion as the end of a sentence, or nothing, so that it can be appended to any message.
pub fn hint(typed: &str, candidates: &[&str], max: i32) -> String {
    let nearest = nearest(typed, candidates, max);
    match nearest.len() {
        0 => String::new(),
        1 => format!(" Did you mean '{}'?", nearest[0]),
        _ => format!(" Did you mean {}?", nearest.iter().map(|name| format!("'{name}'")).collect::<Vec<_>>().join(" or ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_swap_is_one_slip_and_case_is_none() {
        assert_eq!(distance("chaet", "cheat"), 1);
        assert_eq!(distance("CHEAT", "cheat"), 0);
        assert_eq!(distance("", "abc"), 3);
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("ca", "abc"), 3);
        assert_eq!(distance("😀", "a"), 1);
    }

    #[test]
    fn only_the_joint_best_are_offered_and_a_short_word_gets_little_room() {
        let commands = ["cheat", "cat", "clear", "cls", "ls", "cp", "mv"];
        assert_eq!(nearest("chaet", &commands, 2), ["cheat"]);
        assert_eq!(nearest("rm", &commands, 2), Vec::<&str>::new());
        assert_eq!(nearest("CHEAT", &commands, 2), Vec::<&str>::new());
        assert_eq!(nearest("  ", &commands, 2), Vec::<&str>::new());
        assert_eq!(nearest("chaet", &commands, 0), Vec::<&str>::new());
        assert_eq!(nearest("abc", &["axy"], 2), Vec::<&str>::new());
        assert_eq!(nearest("abcd", &["abxy"], 2), ["abxy"]);
        assert_eq!(nearest("abcdefg", &["abcdxyz"], 2), ["abcdxyz"]);
    }

    #[test]
    fn a_candidate_the_text_begins_wins_a_tie_and_the_rest_are_in_order() {
        assert_eq!(nearest("cl", &["al", "cls", "bl", "clx"], 4), ["cls", "clx", "al", "bl"]);
        assert_eq!(nearest("ab", &["Ac", "ab1", "aB2", "ad"], 3), ["ab1", "aB2", "Ac"]);
        assert_eq!(hint("DPadUpp", &["DPadUp", "DPadDown"], 2), " Did you mean 'DPadUp'?");
        assert_eq!(hint("cl", &["cls", "clx", "al"], 2), " Did you mean 'cls' or 'clx'?");
    }
}
