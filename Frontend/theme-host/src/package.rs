//! A theme package as the host needs it: `theme.json` read through `emusen-theme-manifest` (the schema's one owner), and what it
//! provides. No Qt; the host hands it the file's text.

use emusen_theme_manifest::{parse, Screen};

/// What the host loads from a package.
#[derive(Clone, Debug, PartialEq)]
pub struct Package {
    pub name: String,
    pub screens: Vec<Screen>,
}

/// `theme.json`'s text held to the schema and the API version; `Err` with one `theme.json:line kind words` per refusing finding.
pub fn read(text: &str) -> Result<Package, String> {
    let words = |f: &emusen_theme_manifest::Finding| format!("theme.json:{} {} {}", f.line, f.kind, f.words);
    let (manifest, findings) = parse(text).map_err(|f| words(&f))?;
    let refusing: Vec<String> = findings.iter().filter(|f| f.refuses()).map(words).collect();
    if !refusing.is_empty() {
        return Err(refusing.join("\n"));
    }
    Ok(Package { name: manifest.name, screens: manifest.screens })
}
