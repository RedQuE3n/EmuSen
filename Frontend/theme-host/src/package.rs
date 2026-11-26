//! A theme package as the host needs it: `theme.json` read through `emusen-theme-manifest` (the schema's one owner), and what it
//! provides. No Qt; the host hands it the file's text.

use emusen_theme_manifest::{parse, DesktopScreen, Screen, TokenOverride, TokenValue, Variant};

/// What the host loads from a package.
#[derive(Clone, Debug, PartialEq)]
pub struct Package {
    pub name: String,
    pub screens: Vec<Screen>,
    pub desktop_screens: Vec<DesktopScreen>,
    /// The variants the theme is designed for, as `theme.json` lists them.
    pub variants: Vec<Variant>,
    /// The token overrides `theme.json` names, each for a variant, which the host's program sets on the theme's engine and,
    /// for the chosen theme, on its own (decided by the tester, 2026-11-13).
    pub tokens: Vec<TokenOverride>,
    /// Desktop face token overrides layered over base variant tokens when in Desktop mode.
    pub desktop_tokens: Vec<TokenOverride>,
}

/// `theme.json`'s text held to the schema and the API version; `Err` with one `theme.json:line kind words` per refusing finding.
pub fn read(text: &str) -> Result<Package, String> {
    let words = |f: &emusen_theme_manifest::Finding| format!("theme.json:{} {} {}", f.line, f.kind, f.words);
    let (manifest, findings) = parse(text).map_err(|f| words(&f))?;
    let refusing: Vec<String> = findings.iter().filter(|f| f.refuses()).map(words).collect();
    if !refusing.is_empty() {
        return Err(refusing.join("\n"));
    }
    Ok(Package {
        name: manifest.name,
        screens: manifest.screens,
        desktop_screens: manifest.desktop_screens,
        variants: manifest.variants,
        tokens: manifest.tokens,
        desktop_tokens: manifest.desktop_tokens,
    })
}

fn encode_overrides(tokens: &[TokenOverride]) -> Vec<(bool, String, String)> {
    tokens
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

impl Package {
    /// Every token override as the host sets it on a theme's engine: light or not, the `Tokens` property, and its value as
    /// text (a colour as written, a number, `true` or `false`) for the kit to read as the token's kind (EmuSen_Qt.md §3.5.1).
    pub fn overrides(&self) -> Vec<(bool, String, String)> {
        encode_overrides(&self.tokens)
    }

    /// Desktop face token overrides encoded in the same `(is_light, property, value)` format as `overrides`.
    pub fn desktop_overrides(&self) -> Vec<(bool, String, String)> {
        encode_overrides(&self.desktop_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::read;
    use emusen_theme_manifest::{DesktopScreen, Screen};

    #[test]
    fn schema_1_and_schema_2_packages_expose_base_and_desktop_surfaces() {
        let v1 = r##"{"schema":1,"api":"1.0","name":"Legacy","author":"A","licence":"L","variants":["dark"],"screens":["home"],"tokens":{"dark":{"accent":"#4C9AE8"}}}"##;
        let p1 = read(v1).unwrap();
        assert_eq!(p1.screens, vec![Screen::Home]);
        assert!(p1.desktop_screens.is_empty());
        assert_eq!(p1.overrides(), vec![(false, "accent".to_owned(), "#4C9AE8".to_owned())]);
        assert!(p1.desktop_overrides().is_empty());

        let v2 = r##"{"schema":2,"api":"1.22","name":"Dual","author":"A","licence":"L","variants":["dark","light"],"faces":{"bigPicture":["home"],"desktop":["library","settings"]},"tokens":{"dark":{"accent":"#F6E7D8","desktop":{"rowHeight":38.0}},"light":{"desktop":{"tileRadius":12}}}}"##;
        let p2 = read(v2).unwrap();
        assert_eq!(p2.screens, vec![Screen::Home]);
        assert_eq!(p2.desktop_screens, vec![DesktopScreen::Library, DesktopScreen::Settings]);
        assert_eq!(p2.overrides(), vec![(false, "accent".to_owned(), "#F6E7D8".to_owned())]);
        assert_eq!(
            p2.desktop_overrides(),
            vec![(false, "rowHeight".to_owned(), "38".to_owned()), (true, "tileRadius".to_owned(), "12".to_owned())]
        );
    }
}
