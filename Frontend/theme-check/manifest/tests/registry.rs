// Rule Q: the schema's API version and cue names are held to EmuSen.Theme's committed registry, the module's one surface
// (EmuSen_Qt.md §3.2, §3.3), so a minor added there and not here, or a cue renamed, fails here.

use emusen_theme_manifest::{API, CUES};

const TYPES: &str = include_str!("../../../../Testing/qmltypes/EmuSen.Theme-1.0.qmltypes");
const QMLDIR: &str = include_str!("../../../../Testing/qmltypes/EmuSen.Theme-1.0.qmldir");

/// A `major.minor` as numbers.
fn version(text: &str) -> Option<(u64, u64)> {
    let (major, minor) = text.trim().split_once('.')?;
    Some((major.parse().ok()?, minor.parse().ok()?))
}

#[test]
fn rule_q_the_api_is_the_newest_version_the_module_exports() {
    let exported: Vec<(u64, u64)> = TYPES
        .lines()
        .filter_map(|l| l.trim().strip_prefix("exports: ["))
        .flat_map(|list| list.trim_end_matches(']').split(',').map(str::to_owned).collect::<Vec<_>>())
        .filter_map(|e| e.trim().trim_matches('"').rsplit_once(' ').and_then(|(_, v)| version(v)))
        .collect();
    let listed: Vec<(u64, u64)> = QMLDIR.lines().filter_map(|l| l.split_whitespace().find_map(version)).collect();
    assert!(exported.len() >= 9 && listed.len() >= 11, "the registry was read: {} exports, {} qmldir types", exported.len(), listed.len());
    assert_eq!(exported.iter().chain(&listed).max(), Some(&API), "the manifest's API is the module's newest version");
    assert!(QMLDIR.lines().next() == Some("module EmuSen.Theme"), "the qmldir is EmuSen.Theme's");
}

#[test]
fn rule_q_the_cue_names_are_the_cues_singleton_s_methods_in_order() {
    let lines: Vec<&str> = TYPES.lines().map(str::trim).collect();
    let at = lines.iter().position(|l| *l == "exports: [\"EmuSen.Theme/Cues 1.0\"]").expect("the Cues singleton in the registry");
    let start = lines[..at].iter().rposition(|l| *l == "Component {").expect("its component");
    let end = lines[at..].iter().position(|l| *l == "Component {").map_or(lines.len(), |i| at + i);
    let methods: Vec<&str> = lines[start..end]
        .iter()
        .filter_map(|l| l.strip_prefix("Method { name: \""))
        .filter_map(|r| r.split_once('"').map(|(n, _)| n))
        .collect();
    assert_eq!(methods, CUES, "the cue names a theme may give a file are the Cues the theme can ring");
}
