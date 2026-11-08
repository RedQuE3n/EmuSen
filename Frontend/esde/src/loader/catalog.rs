//! Every element type and property in THEMES.md's reference section, plus the sound element, as the data file
//! catalog.tsv holds them (EmuSen_BigPicture.md §12.2): read once, never changed.

use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PropertyType {
    Pair,
    Path,
    Boolean,
    Color,
    UInt,
    Float,
    String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Primary,
    Secondary,
    Special,
}

/// How a STRING's text is read: free text, one of a set, a list of a set, or a PATH keyed by an attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Single,
    Enum,
    List,
    Keyed,
}

/// Per-axis rules the reference gives particular pairs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairRule {
    Clamp,
    ZeroAxisAuto,
    MinusOneMatchesOther,
    OneAxisOnly,
}

/// Defaults the reference states in words rather than as a literal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Computed {
    None,
    SystemFullName,
    ElementWidth,
    OneAndAHalfFontSize,
    HelpPosByOrientation,
    HelpFontSizeByOrientation,
    InterpolationByRotation,
    ContainerForDescription,
    DisplayRelativeForLastPlayed,
    ContainerStartDelayByType,
}

/// The view a property is limited to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Any,
    System,
    Gamelist,
}

/// One documented property.
#[derive(Clone, Debug)]
pub struct PropertySpec {
    pub name: String,
    pub kind: PropertyType,
    pub default: Option<String>,
    pub same_as: Option<String>,
    pub computed: Computed,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub max_y: Option<f32>,
    pub rule: PairRule,
    pub shape: Shape,
    pub values: Vec<String>,
    pub max_items: usize,
    pub key_attribute: Option<String>,
    pub only_in: Scope,
    /// Set for the video playback properties, which nothing draws yet (EmuSen_BigPicture.md §10.1, Q4).
    pub deferred: Option<String>,
    /// A range THEMES.md does not state, measured in ES-DE: outside it the default applies, with a warning (§35.2).
    pub valid_range: Option<(u32, u32)>,
}

/// One documented element type.
#[derive(Debug)]
pub struct ElementSpec {
    pub kind: String,
    pub group: Group,
    pub system: bool,
    pub gamelist: bool,
    pub all: bool,
    pub properties: Vec<PropertySpec>,
    by_name: HashMap<String, usize>,
}

impl ElementSpec {
    pub fn find(&self, name: &str) -> Option<&PropertySpec> {
        self.by_name.get(name).map(|&i| &self.properties[i])
    }

    /// Only helpsystem, clock and systemstatus lack one; they draw above everything.
    pub fn default_z_index(&self) -> Option<f32> {
        self.find("zIndex").and_then(|z| z.default.as_deref()).and_then(|d| d.parse().ok())
    }
}

pub struct Catalog {
    pub elements: Vec<ElementSpec>,
    by_kind: HashMap<String, usize>,
}

impl Catalog {
    pub fn find(&self, kind: &str) -> Option<&ElementSpec> {
        self.by_kind.get(kind).map(|&i| &self.elements[i])
    }

    pub fn property_count(&self) -> usize {
        self.elements.iter().map(|e| e.properties.len()).sum()
    }
}

/// The catalogue, read from catalog.tsv on first use.
pub fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| read(include_str!("catalog.tsv")))
}

fn read(text: &str) -> Catalog {
    let mut elements: Vec<ElementSpec> = Vec::new();
    for line in text.lines().filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let f: Vec<&str> = line.split('\t').collect();
        if f[0] == "@" {
            let views: Vec<&str> = f[3].split(',').collect();
            let group = match f[2] {
                "primary" => Group::Primary,
                "secondary" => Group::Secondary,
                _ => Group::Special,
            };
            elements.push(ElementSpec {
                kind: f[1].to_owned(),
                group,
                system: views.contains(&"system"),
                gamelist: views.contains(&"gamelist"),
                all: views.contains(&"all"),
                properties: Vec::new(),
                by_name: HashMap::new(),
            });
            continue;
        }
        assert_eq!(f.len(), 16, "catalog.tsv: {line}");
        let opt = |s: &str| (s != "-").then(|| s.to_owned());
        let num = |s: &str| (s != "-").then(|| s.parse::<f32>().expect("catalog.tsv: a number"));
        let spec = PropertySpec {
            name: f[0].to_owned(),
            kind: match f[1] {
                "pair" => PropertyType::Pair,
                "path" => PropertyType::Path,
                "boolean" => PropertyType::Boolean,
                "color" => PropertyType::Color,
                "uint" => PropertyType::UInt,
                "float" => PropertyType::Float,
                _ => PropertyType::String,
            },
            default: opt(f[2]),
            same_as: opt(f[3]),
            computed: match f[4] {
                "SystemFullName" => Computed::SystemFullName,
                "ElementWidth" => Computed::ElementWidth,
                "OneAndAHalfFontSize" => Computed::OneAndAHalfFontSize,
                "HelpPosByOrientation" => Computed::HelpPosByOrientation,
                "HelpFontSizeByOrientation" => Computed::HelpFontSizeByOrientation,
                "InterpolationByRotation" => Computed::InterpolationByRotation,
                "ContainerForDescription" => Computed::ContainerForDescription,
                "DisplayRelativeForLastPlayed" => Computed::DisplayRelativeForLastPlayed,
                "ContainerStartDelayByType" => Computed::ContainerStartDelayByType,
                _ => Computed::None,
            },
            min: num(f[5]),
            max: num(f[6]),
            max_y: num(f[7]),
            rule: match f[8] {
                "ZeroAxisAuto" => PairRule::ZeroAxisAuto,
                "MinusOneMatchesOther" => PairRule::MinusOneMatchesOther,
                "OneAxisOnly" => PairRule::OneAxisOnly,
                _ => PairRule::Clamp,
            },
            shape: match f[9] {
                "enum" => Shape::Enum,
                "list" => Shape::List,
                "keyed" => Shape::Keyed,
                _ => Shape::Single,
            },
            values: opt(f[10]).map(|v| v.split(' ').map(str::to_owned).collect()).unwrap_or_default(),
            max_items: opt(f[11]).map(|n| n.parse().expect("catalog.tsv: maxItems")).unwrap_or(0),
            key_attribute: opt(f[12]),
            only_in: match f[13] {
                "system" => Scope::System,
                "gamelist" => Scope::Gamelist,
                _ => Scope::Any,
            },
            deferred: opt(f[14]),
            valid_range: opt(f[15]).map(|r| {
                let (lo, hi) = r.split_once('-').expect("catalog.tsv: a range");
                (lo.parse().expect("catalog.tsv: a range"), hi.parse().expect("catalog.tsv: a range"))
            }),
        };
        let element = elements.last_mut().expect("catalog.tsv: a property before any element");
        element.by_name.insert(spec.name.clone(), element.properties.len());
        element.properties.push(spec);
    }
    let by_kind = elements.iter().enumerate().map(|(i, e)| (e.kind.clone(), i)).collect();
    Catalog { elements, by_kind }
}
