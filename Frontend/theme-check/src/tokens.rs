//! Token overrides held to the kit: a name the `Tokens` singleton has, a value of its type, and the contrast pairs' floors.

use emusen_kit_core::tokens::{Colour, ColourToken, MAX_TEXT_SCALE, Variant as KitVariant, clamp_text_scale, colour, contrast_pairs};
use emusen_theme_manifest::{Manifest, TokenOverride, TokenValue, Variant};

use crate::Finding;

/// The kit's committed registry (rule Q): its `Tokens` singleton's read-only properties are the tokens and their types.
const REGISTRY: &str = include_str!("../../../Testing/qmltypes/EmuSen.Kit-1.0.qmltypes");

/// A token's type as the registry declares it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenType {
    Colour,
    Number,
    Whole,
    Switch,
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

fn type_words(t: TokenType) -> &'static str {
    match t {
        TokenType::Colour => "a colour, #RRGGBB or #AARRGGBB",
        TokenType::Number => "a number",
        TokenType::Whole => "a whole number",
        TokenType::Switch => "true or false",
    }
}

fn holds(t: TokenType, value: &TokenValue) -> bool {
    match (t, value) {
        (TokenType::Colour, TokenValue::Text(s)) => parse_colour(s).is_some(),
        (TokenType::Number, TokenValue::Number(n)) => n.is_finite(),
        (TokenType::Whole, TokenValue::Number(n)) => n.is_finite() && n.fract() == 0.0 && n.abs() <= i32::MAX as f64,
        (TokenType::Switch, TokenValue::Switch(_)) => true,
        _ => false,
    }
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
    let mut typed = Vec::new();
    for o in &manifest.tokens {
        match known.iter().find(|(name, _)| *name == o.name) {
            None => findings.push(Finding::new(file, o.line, "token", format!("{} is not a token of the kit", o.name))),
            Some((_, t)) if !holds(*t, &o.value) => {
                findings.push(Finding::new(file, o.line, "token", format!("{} is {}", o.name, type_words(*t))));
            }
            Some(_) => typed.push(o),
        }
    }
    for variant in manifest.variants.iter().copied() {
        findings.extend(contrast(&typed, variant, file));
    }
    findings
}

fn contrast(overrides: &[&TokenOverride], variant: Variant, file: &str) -> Vec<Finding> {
    let kit = kit_variant(variant);
    let given = |token: ColourToken| {
        overrides.iter().find(|o| o.variant == variant && o.name == property_of(token)).and_then(|o| match &o.value {
            TokenValue::Text(s) => parse_colour(s).map(|c| (c, o.line)),
            _ => None,
        })
    };
    let value = |token: ColourToken| given(token).map_or(colour(kit, token), |(c, _)| c);
    let screen = value(ColourToken::Surface).over(colour(kit, ColourToken::Void));
    let mut findings = Vec::new();
    for pair in contrast_pairs().into_iter().filter(|p| p.variant == kit) {
        let (fore_given, back_given) = (given(pair.fore), given(pair.back));
        let Some(line) = fore_given.or(back_given).map(|(_, line)| line) else { continue };
        let back = if pair.back == ColourToken::Surface { screen } else { value(pair.back).over(screen) };
        let ratio = value(pair.fore).over(back).contrast(back);
        if ratio < pair.floor {
            let words = format!(
                "{} on {} is {ratio:.2}:1, under the {}:1 floor ({})",
                property_of(pair.fore),
                property_of(pair.back),
                pair.floor,
                variant.name()
            );
            findings.push(Finding::new(file, line, "contrast", words));
        }
    }
    findings
}
