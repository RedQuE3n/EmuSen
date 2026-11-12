//! What a module's QML files offer their users, which rule Q holds as it holds the registry (EmuSen_Qt.md §5.3,
//! R-20261111-99): each root object's properties and signals, and every object name in the file. The registry
//! qmltyperegistrar writes lists the C++ types only, so this is the QML files' half of the module's surface.

use std::path::Path;

/// One QML file's surface, one entry a line: `property <words>`, `signal <name>(<parameters>)`, `objectName <name>`.
pub fn surface(text: &str) -> Vec<String> {
    let mut out = vec![];
    let mut depth = 0usize;
    let mut in_comment = false;
    for raw in text.lines() {
        let line = strip(raw, &mut in_comment);
        let trimmed = line.trim();
        if depth == 1 {
            if let Some(entry) = declaration(trimmed) {
                out.push(entry);
            }
        }
        let mut from = 0;
        while let Some(at) = line[from..].find("objectName:").map(|i| i + from) {
            let start = at + "objectName:".len();
            let value: String = raw.get(start..).unwrap_or("").chars().take_while(|c| !matches!(c, ';' | '}')).collect();
            out.push(format!("objectName {}", value.trim()));
            from = start;
        }
        for c in line.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
    }
    out
}

// A line without its comments, each string's contents written as '_' one for one, so braces in them are not counted and
// a place in it is the same place in the line.
fn strip(line: &str, in_comment: &mut bool) -> String {
    let mut out = String::new();
    let mut chars = line.chars().peekable();
    let mut quote: Option<char> = None;
    while let Some(c) = chars.next() {
        if *in_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                *in_comment = false;
            }
            continue;
        }
        if let Some(q) = quote {
            if c == '\\' {
                out.push('_');
                if chars.next().is_some() {
                    out.push('_');
                }
            } else if c == q {
                out.push(c);
                quote = None;
            } else {
                out.push('_');
            }
            continue;
        }
        match c {
            '"' | '\'' | '`' => {
                quote = Some(c);
                out.push(c);
            }
            '/' if chars.peek() == Some(&'/') => break,
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                *in_comment = true;
            }
            _ => out.push(c),
        }
    }
    out
}

// A root-level property or signal declaration as its surface entry: the words up to the name, without the binding.
fn declaration(line: &str) -> Option<String> {
    let words: Vec<&str> = line.split(|c: char| c.is_whitespace()).filter(|w| !w.is_empty()).collect();
    let at = words.iter().position(|w| *w == "property" || w.starts_with("signal"))?;
    if !words[..at].iter().all(|w| matches!(*w, "readonly" | "required" | "default" | "final")) {
        return None;
    }
    if words[at] == "property" {
        let head = line.split(':').next()?.trim();
        return Some(head.split_whitespace().collect::<Vec<_>>().join(" "));
    }
    let rest = line[line.find("signal")? + "signal".len()..].trim();
    let rest = rest.split('{').next()?.trim().trim_end_matches(';');
    Some(format!("signal {}", rest.split_whitespace().collect::<Vec<_>>().join(" ")))
}

/// A module's QML files' surfaces as one text: for each type the qmldir lists (`Name version file`), read from `root`,
/// a line `Name` and its entries indented under it, in the qmldir's order.
pub fn module(qmldir: &str, root: &Path) -> String {
    let mut out = String::new();
    for line in qmldir.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let [name, _version, file] = parts[..] else { continue };
        if !file.ends_with(".qml") {
            continue;
        }
        let text = std::fs::read_to_string(root.join(file)).unwrap_or_else(|e| panic!("{file}: {e}"));
        out += name;
        out += "\n";
        for entry in surface(&text) {
            out += "    ";
            out += &entry;
            out += "\n";
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLIP: &str = "import QtQuick\n// a { comment\nItem {\n    id: root\n    objectName: \"videoPlayback\"\n    property url source\n    readonly property real position: player.position\n    signal played(string path)\n    Text { text: \"{\"; property int inner: 1; objectName: \"label\" }\n    Timer {\n        property int nested: 2\n    }\n}\n";

    #[test]
    fn a_root_s_properties_signals_and_every_object_name_are_its_surface() {
        assert_eq!(
            surface(CLIP),
            [
                "objectName \"videoPlayback\"",
                "property url source",
                "readonly property real position",
                "signal played(string path)",
                "objectName \"label\""
            ]
        );
    }

    #[test]
    fn an_added_property_changes_the_surface() {
        let added = CLIP.replace("    property url source\n", "    property url source\n    property int fade: 600\n");
        assert_ne!(surface(CLIP), surface(&added));
        let bound = CLIP.replace("readonly property real position: player.position", "readonly property real position: 0");
        assert_eq!(surface(CLIP), surface(&bound), "a binding's value is not the surface");
    }
}
