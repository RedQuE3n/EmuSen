//! "Did you mean …?" for a name that matched nothing: the C# `Suggestion`. See EmuSen_Config_Reference.md §6.
//!
//! Lengths and edits are counted in UTF-16 code units and case is .NET's, because the answer is a sentence the
//! player reads and must be the sentence the C# gave.

use crate::dotnet_path::{equals_ignore_case, is_blank, simple_upper};
use crate::dotnet_text::{compare_ignore_case, lower_unit, utf16_len};

/// Optimal string alignment: Levenshtein with a swap of two neighbours counted as one slip, case ignored.
pub fn distance(a: &str, b: &str) -> usize {
    let a: Vec<u16> = a.encode_utf16().map(lower_unit).collect();
    let b: Vec<u16> = b.encode_utf16().map(lower_unit).collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut d = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in d.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in d[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            d[i][j] = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                d[i][j] = d[i][j].min(d[i - 2][j - 2] + cost);
            }
        }
    }
    d[a.len()][b.len()]
}

/// How wrong a short word may be before a guess stops being one.
fn tolerance(length: usize) -> usize {
    match length {
        0..=3 => 1,
        4..=6 => 2,
        _ => 3,
    }
}

fn starts_with_ignore_case(name: &str, prefix: &str) -> bool {
    let mut name = name.chars().map(simple_upper);
    prefix.chars().map(simple_upper).all(|c| name.next() == Some(c))
}

/// The closest candidates, closest first; one the typed text begins wins a tie. None when the text matches a candidate exactly, since the caller then refused it for another reason.
pub fn nearest<'a>(typed: &str, candidates: &[&'a str], max: i32) -> Vec<&'a str> {
    if is_blank(Some(typed)) || max <= 0 || candidates.iter().any(|candidate| equals_ignore_case(candidate, typed)) {
        return Vec::new();
    }
    let tolerance = tolerance(utf16_len(typed));
    let scored: Vec<(&str, usize)> = candidates
        .iter()
        .filter(|candidate| !candidate.is_empty())
        .map(|candidate| (*candidate, distance(typed, candidate)))
        .filter(|(_, distance)| *distance > 0 && *distance <= tolerance)
        .collect();
    let Some(best) = scored.iter().map(|(_, distance)| *distance).min() else {
        return Vec::new();
    };
    let mut joint: Vec<&str> = scored.into_iter().filter(|(_, distance)| *distance == best).map(|(name, _)| name).collect();
    // Both sorts keep the order given among equals, as C#'s OrderBy does.
    joint.sort_by(|a, b| starts_with_ignore_case(b, typed).cmp(&starts_with_ignore_case(a, typed)).then_with(|| compare_ignore_case(a, b)));
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
        assert_eq!(distance("abc", ""), 3);
        assert_eq!(distance("kitten", "sitting"), 3);
        assert_eq!(distance("ca", "abc"), 3);
        assert_eq!(distance("😀", "a"), 2);
    }

    #[test]
    fn only_the_joint_best_are_offered_and_a_short_word_gets_little_room() {
        let commands = ["cheat", "cat", "clear", "cls", "ls", "cp", "mv"];
        assert_eq!(nearest("chaet", &commands, 2), ["cheat"]);
        assert_eq!(nearest("rm", &commands, 2), Vec::<&str>::new());
        assert_eq!(nearest("cheat", &commands, 2), Vec::<&str>::new());
        assert_eq!(nearest("CHEAT", &commands, 2), Vec::<&str>::new());
        assert_eq!(nearest("  ", &commands, 2), Vec::<&str>::new());
        assert_eq!(nearest("chaet", &commands, 0), Vec::<&str>::new());
        assert_eq!(nearest("qwertyuiop", &commands, 2), Vec::<&str>::new());
        // Three letters get one slip, four to six get two, and seven or more get three.
        assert_eq!(nearest("abc", &["axy"], 2), Vec::<&str>::new());
        assert_eq!(nearest("abcd", &["abxy"], 2), ["abxy"]);
        assert_eq!(nearest("abcdef", &["abcxyz"], 2), Vec::<&str>::new());
        assert_eq!(nearest("abcdefg", &["abcdxyz"], 2), ["abcdxyz"]);
    }

    #[test]
    fn a_candidate_the_text_begins_wins_a_tie_and_the_rest_are_in_order() {
        assert_eq!(nearest("cl", &["al", "cls", "bl", "clx"], 4), ["cls", "clx", "al", "bl"]);
        assert_eq!(nearest("cl", &["al", "cls", "bl", "clx"], 2), ["cls", "clx"]);
        assert_eq!(nearest("ab", &["Ac", "ab1", "aB2", "ad"], 3), ["ab1", "aB2", "Ac"]);
    }

    #[test]
    fn the_hint_is_the_end_of_a_sentence() {
        assert_eq!(hint("DPadUpp", &["DPadUp", "DPadDown"], 2), " Did you mean 'DPadUp'?");
        assert_eq!(hint("cl", &["cls", "clx", "al"], 2), " Did you mean 'cls' or 'clx'?");
        assert_eq!(hint("cl", &["cls", "clx", "cly"], 3), " Did you mean 'cls' or 'clx' or 'cly'?");
        assert_eq!(hint("qwertyuiop", &["Up"], 2), "");
    }
}
