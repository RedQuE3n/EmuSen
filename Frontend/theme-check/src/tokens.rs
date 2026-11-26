//! Token overrides held to the kit: a name the `Tokens` singleton has, a value of its type, and the contrast pairs' floors.

use emusen_kit_core::tokens::{BUNDLED, Colour, ColourToken, MAX_TEXT_SCALE, Variant as KitVariant, clamp_text_scale, colour, contrast_pairs, is_bundled};
use emusen_theme_manifest::{Manifest, TokenOverride, TokenValue, Variant};

use crate::Finding;

/// The kit's committed registry (rule Q): its `Tokens` singleton's read-only properties are the tokens and their types.
const REGISTRY: &str = include_str!("../../../Testing/qmltypes/EmuSen.Kit-1.13.qmltypes");

/// A token's type as the registry declares it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
    Colour,
    Number,
    Whole,
    Switch,
    /// A face's family, one the kit bundles (EmuSen_Kit.md §7.117).
    Face,
    /// A dashed line's dash and gap in widths of the line, two positive numbers apart by a space (1.12, EmuSen_Kit.md §7.2).
    Pattern,
}

/// Every token of the registry's `Tokens`, in its order: the read-only properties, with their types.
pub fn registry_tokens() -> Vec<(String, TokenType)> {
    let mut tokens = Vec::new();
    let lines: Vec<&str> = REGISTRY.lines().map(str::trim).collect();
    let Some(start) = lines.windows(2).position(|w| w[0] == "name: \"Tokens\"" && w[1].starts_with("accessSemantics")) else {
        return tokens;
    };
    let mut depth = 1usize;
    let (mut in_property, mut name, mut kind, mut readonly) = (false, String::new(), String::new(), false);
    for line in &lines[start..] {
        if line.ends_with('{') {
            depth += 1;
            in_property = depth == 2 && *line == "Property {";
            (name, kind, readonly) = (String::new(), String::new(), false);
        } else if *line == "}" {
            if depth == 2 && in_property && readonly {
                let typed = match kind.as_str() {
                    "QColor" => Some(TokenType::Colour),
                    "double" => Some(TokenType::Number),
                    "int" => Some(TokenType::Whole),
                    "bool" => Some(TokenType::Switch),
                    "QString" => Some(TokenType::Face),
                    // A list QML reads is a QVariant in the registry, as the kit's figures' are; the one such token is dashPattern.
                    "QVariant" => Some(TokenType::Pattern),
                    _ => None,
                };
                if let Some(t) = typed {
                    tokens.push((name.clone(), t));
                }
            }
            depth -= 1;
            in_property = false;
            if depth == 0 {
                break;
            }
        } else if in_property {
            let quoted = |key: &str| line.strip_prefix(key).map(|v| v.trim_matches('"').to_owned());
            if let Some(v) = quoted("name: ") {
                name = v;
            } else if let Some(v) = quoted("type: ") {
                kind = v;
            } else if *line == "isReadonly: true" {
                readonly = true;
            }
        }
    }
    tokens
}

/// The `Tokens` property that carries each colour token, kit-core's name for it; held to the registry both ways by the tests.
pub fn property_of(token: ColourToken) -> &'static str {
    token.property()
}

fn kit_variant(v: Variant) -> KitVariant {
    match v {
        Variant::Dark => KitVariant::Dark,
        Variant::Light => KitVariant::Light,
    }
}

/// `#RRGGBB` or `#AARRGGBB`, the form the kit's tables are written in.
fn parse_colour(text: &str) -> Option<Colour> {
    Colour::parse(text)
}

fn type_words(t: TokenType) -> String {
    match t {
        TokenType::Face => format!("a face the kit bundles, one of {}", BUNDLED.join(", ")),
        TokenType::Colour => "a colour, #RRGGBB or #AARRGGBB".to_owned(),
        TokenType::Number => "a number".to_owned(),
        TokenType::Whole => "a whole number".to_owned(),
        TokenType::Switch => "true or false".to_owned(),
        TokenType::Pattern => "a dash and a gap in widths of the line, two positive numbers apart by a space".to_owned(),
    }
}

fn holds(t: TokenType, value: &TokenValue) -> bool {
    match (t, value) {
        (TokenType::Colour, TokenValue::Text(s)) => parse_colour(s).is_some(),
        (TokenType::Number, TokenValue::Number(n)) => n.is_finite(),
        (TokenType::Whole, TokenValue::Number(n)) => n.is_finite() && n.fract() == 0.0 && n.abs() <= i32::MAX as f64,
        (TokenType::Switch, TokenValue::Switch(_)) => true,
        (TokenType::Face, TokenValue::Text(s)) => is_bundled(s),
        (TokenType::Pattern, TokenValue::Text(s)) => emusen_kit_core::tokens::dash_pattern(s).is_some(),
        _ => false,
    }
}

fn validate_overrides<'a>(
    overrides: &'a [TokenOverride],
    known: &[(String, TokenType)],
    file: &str,
    findings: &mut Vec<Finding>,
) -> Vec<&'a TokenOverride> {
    let mut typed = Vec::new();
    for o in overrides {
        match known.iter().find(|(name, _)| *name == o.name) {
            None => findings.push(Finding::new(file, o.line, "token", format!("{} is not a token of the kit", o.name))),
            Some((_, t)) if !holds(*t, &o.value) => {
                findings.push(Finding::new(file, o.line, "token", format!("{} is {}", o.name, type_words(*t))));
            }
            Some(_) => typed.push(o),
        }
    }
    typed
}

fn given_in(overrides: &[&TokenOverride], variant: Variant, token: ColourToken) -> Option<(Colour, usize)> {
    overrides.iter().find(|o| o.variant == variant && o.name == property_of(token)).and_then(|o| match &o.value {
        TokenValue::Text(s) => parse_colour(s).map(|c| (c, o.line)),
        _ => None,
    })
}

/// Every override's name and type, the text scales, and each variant's contrast pairs with the overrides applied.
pub fn check(manifest: &Manifest, file: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let known = registry_tokens();
    for size in &manifest.font_sizes {
        if clamp_text_scale(*size) != *size {
            findings.push(Finding::new(
                file,
                manifest.font_sizes_line,
                "schema",
                format!("fontSizes {size} is outside the text scales 1 to {MAX_TEXT_SCALE}"),
            ));
        }
    }
    let typed = validate_overrides(&manifest.tokens, &known, file, &mut findings);
    let desktop_typed = validate_overrides(&manifest.desktop_tokens, &known, file, &mut findings);
    for variant in manifest.variants.iter().copied() {
        let (base_findings, base_failed) = contrast_layer(&typed, None, &[], variant, file);
        findings.extend(base_findings);
        if desktop_typed.iter().any(|o| o.variant == variant && ColourToken::named(&o.name).is_some()) {
            findings.extend(contrast_layer(&typed, Some(&desktop_typed), &base_failed, variant, file).0);
        }
    }
    findings
}

fn contrast_layer(
    base: &[&TokenOverride],
    desktop: Option<&[&TokenOverride]>,
    skip_unchanged: &[(ColourToken, ColourToken)],
    variant: Variant,
    file: &str,
) -> (Vec<Finding>, Vec<(ColourToken, ColourToken)>) {
    let kit = kit_variant(variant);
    let primary = desktop.unwrap_or(base);
    let value = |token: ColourToken| {
        given_in(primary, variant, token)
            .or_else(|| desktop.and_then(|_| given_in(base, variant, token)))
            .map_or(colour(kit, token), |(c, _)| c)
    };
    let surface_primary = given_in(primary, variant, ColourToken::Surface);
    let screen = value(ColourToken::Surface).over(colour(kit, ColourToken::Void));
    let label = if desktop.is_some() { format!("{}, desktop", variant.name()) } else { variant.name().to_owned() };
    let mut findings = Vec::new();
    let mut failed = Vec::new();
    for pair in contrast_pairs().into_iter().filter(|p| p.variant == kit) {
        let (fore_p, back_p) = (given_in(primary, variant, pair.fore), given_in(primary, variant, pair.back));
        let line = match fore_p.or(back_p) {
            Some((_, line)) => line,
            None => match surface_primary.filter(|_| desktop.is_some() && !skip_unchanged.contains(&(pair.fore, pair.back))) {
                Some((_, line)) => line,
                None => continue,
            },
        };
        let back = if pair.back == ColourToken::Surface { screen } else { value(pair.back).over(screen) };
        let ratio = value(pair.fore).over(back).contrast(back);
        if ratio < pair.floor {
            failed.push((pair.fore, pair.back));
            findings.push(Finding::new(
                file,
                line,
                "contrast",
                format!("{} on {} is {ratio:.2}:1, under the {}:1 floor ({label})", property_of(pair.fore), property_of(pair.back), pair.floor),
            ));
        }
    }
    (findings, failed)
}
