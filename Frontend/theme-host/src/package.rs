//! A theme package as the host needs it: `theme.json` read through `emusen-theme-manifest` (the schema's one owner), and what it
//! provides. No Qt; the host hands it the file's text.

use emusen_theme_manifest::{parse, Screen, TokenOverride, TokenValue, Variant};

/// What the host loads from a package.
#[derive(Clone, Debug, PartialEq)]
pub struct Package {
    pub name: String,
    pub screens: Vec<Screen>,
    /// The token overrides `theme.json` names, each for a variant, which the host's program sets on the theme's engine only.
    pub tokens: Vec<TokenOverride>,
}

/// `theme.json`'s text held to the schema and the API version; `Err` with one `theme.json:line kind words` per refusing finding.
pub fn read(text: &str) -> Result<Package, String> {
    let words = |f: &emusen_theme_manifest::Finding| format!("theme.json:{} {} {}", f.line, f.kind, f.words);
    let (manifest, findings) = parse(text).map_err(|f| words(&f))?;
    let refusing: Vec<String> = findings.iter().filter(|f| f.refuses()).map(words).collect();
    if !refusing.is_empty() {
        return Err(refusing.join("\n"));
    }
    Ok(Package { name: manifest.name, screens: manifest.screens, tokens: manifest.tokens })
}

impl Package {
    /// Every token override as the host sets it on a theme's engine: light or not, the `Tokens` property, and its value as
    /// text (a colour as written, a number, `true` or `false`) for the kit to read as the token's kind (EmuSen_Qt.md §3.5.1).
    pub fn overrides(&self) -> Vec<(bool, String, String)> {
        self.tokens
            .iter()
            .map(|o| {
                let value = match &o.value {
                    TokenValue::Text(text) => text.clone(),
                    TokenValue::Number(n) => n.to_string(),
                    TokenValue::Switch(b) => b.to_string(),
                };
                (o.variant == Variant::Light, o.name.clone(), value)
            })
            .collect()
    }
}
