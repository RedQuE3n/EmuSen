//! The libretro cores EmuSen hosts (`data/libretro-cores.tsv`): the one list Cores offers for download and the host
//! runs, each with the systems it is placed under, its upstream source, its licence, the renderer the host gives it, and
//! the systems it is the default engine of though a native core runs them (EmuSen_Cores.md §4.7 and §4.8, decided
//! 2026-11-01).

use serde_json::Value;

/// How a hosted core draws, as the host provides for it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Renderer {
    /// Software pictures (the host's stage 2).
    Software,
    /// The host's Vulkan context, whatever the core's `.info` lists under `required_hw_api` (the host's stage 3).
    Vulkan,
}

impl Renderer {
    pub fn word(self) -> &'static str {
        match self {
            Renderer::Software => "software",
            Renderer::Vulkan => "vulkan",
        }
    }
}

/// One hosted libretro core.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibretroCore {
    /// The core's file stem, `pcee2` of `pcee2_libretro.so`.
    pub stem: String,
    /// The EmuSen system ids it runs.
    pub systems: Vec<String>,
    /// Where its source is published.
    pub upstream: String,
    /// Its licence, as its `.info`'s `license` names it.
    pub licence: String,
    pub renderer: Renderer,
    /// The systems of its own it is the default engine of, ahead of a native core: the named exceptions to the rule
    /// that a console's native core is its default (EmuSen_Cores.md §4.8).
    pub default_for: Vec<String>,
}

/// The list as this build ships it.
pub const SHIPPED: &str = include_str!("../data/libretro-cores.tsv");

fn ids(field: &str) -> Vec<String> {
    if field == "-" {
        return Vec::new();
    }
    field.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect()
}

/// The lines of a list, `#` lines and blank lines skipped; a line of another shape is an error naming it.
pub fn parse(text: &str) -> Result<Vec<LibretroCore>, String> {
    let mut cores: Vec<LibretroCore> = Vec::new();
    for (n, line) in text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty() && !l.starts_with('#')) {
        let line_no = n + 1;
        let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
        let [stem, systems, upstream, licence, renderer, default_for] = fields.as_slice() else {
            return Err(format!("libretro-cores.tsv line {line_no} has not six fields"));
        };
        let systems = ids(systems);
        if stem.is_empty() || systems.is_empty() || !upstream.starts_with("https://") || licence.is_empty() {
            return Err(format!("libretro-cores.tsv line {line_no} needs a stem, a system, an https:// source and a licence"));
        }
        let renderer = match *renderer {
            "software" => Renderer::Software,
            "vulkan" => Renderer::Vulkan,
            other => return Err(format!("libretro-cores.tsv line {line_no}: the renderer {other} is neither software nor vulkan")),
        };
        let default_for = ids(default_for);
        if let Some(bad) = default_for.iter().find(|d| !systems.contains(d)) {
            return Err(format!("libretro-cores.tsv line {line_no}: {stem} is named the default of {bad}, which it does not run"));
        }
        if cores.iter().any(|c| c.stem == *stem) {
            return Err(format!("libretro-cores.tsv line {line_no}: {stem} is listed twice"));
        }
        if let Some((other, system)) = cores.iter().find_map(|c| default_for.iter().find(|d| c.default_for.contains(d)).map(|d| (c.stem.clone(), d.clone()))) {
            return Err(format!("libretro-cores.tsv line {line_no}: {system} has two defaults, {other} and {stem}"));
        }
        cores.push(LibretroCore { stem: stem.to_string(), systems, upstream: upstream.to_string(), licence: licence.to_string(), renderer, default_for });
    }
    Ok(cores)
}

/// The shipped list, read once.
pub fn hosted() -> &'static [LibretroCore] {
    static LIST: std::sync::OnceLock<Vec<LibretroCore>> = std::sync::OnceLock::new();
    LIST.get_or_init(|| parse(SHIPPED).unwrap_or_default())
}

/// The hosted core a system defaults to ahead of its native core, by stem; none where the native core is the default.
pub fn default_for(system: &str) -> Option<&'static str> {
    hosted().iter().find(|c| c.default_for.iter().any(|d| d == system)).map(|c| c.stem.as_str())
}

/// A list as compact JSON, one object a core with the keys stem, systems, upstream, licence, renderer and default_for.
pub fn to_json(cores: &[LibretroCore]) -> String {
    let words = |list: &[String]| Value::Array(list.iter().map(|s| Value::from(s.clone())).collect());
    let rows = cores
        .iter()
        .map(|c| {
            Value::Object(serde_json::Map::from_iter([
                (String::from("stem"), Value::from(c.stem.clone())),
                (String::from("systems"), words(&c.systems)),
                (String::from("upstream"), Value::from(c.upstream.clone())),
                (String::from("licence"), Value::from(c.licence.clone())),
                (String::from("renderer"), Value::from(c.renderer.word())),
                (String::from("default_for"), words(&c.default_for)),
            ]))
        })
        .collect();
    let mut out = String::new();
    out.push_str(&Value::Array(rows).to_string());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_list_reads_whole() {
        let list = parse(SHIPPED).unwrap();
        let stems: Vec<&str> = list.iter().map(|c| c.stem.as_str()).collect();
        assert_eq!(stems, ["pcee2", "flycast", "snes9x", "nestopia", "genesis_plus_gx", "picodrive", "mupen64plus_next", "sameboy", "mednafen_psx_hw", "mednafen_saturn"]);
        assert_eq!(list[1].systems, ["dreamcast", "naomi", "atomiswave"]);
        assert_eq!(list[4].systems, ["md", "mcd", "sms", "sg1000"]);
        assert_eq!(list[6].renderer, Renderer::Vulkan, "mupen64plus_next draws through ParaLLEl-RDP on the host's Vulkan context");
        assert_eq!(list[9].licence, "GPLv2");
    }

    #[test]
    fn every_listed_system_is_in_the_systems_table_or_named_as_arcade() {
        for core in hosted() {
            for system in &core.systems {
                assert!(crate::find(system).is_some() || ["naomi", "atomiswave"].contains(&system.as_str()), "{} runs {system}, no system of the table", core.stem);
            }
        }
    }

    #[test]
    fn only_the_playstation_2_defaults_to_a_hosted_core() {
        let defaults: Vec<(&str, &str)> = crate::table().unwrap().iter().filter_map(|s| default_for(&s.id).map(|d| (s.id.as_str(), d))).collect();
        assert_eq!(defaults, [("ps2", "pcee2")]);
    }

    #[test]
    fn a_line_of_another_shape_is_refused_with_its_number() {
        let line = |s: &str| format!("# c\n{s}\n");
        assert!(parse(&line("x\tps2\thttps://a\tMIT\tsoftware\t-")).is_ok());
        assert_eq!(parse(&line("x\tps2\thttps://a")).unwrap_err(), "libretro-cores.tsv line 2 has not six fields");
        assert!(parse(&line("x\tps2\thttp://insecure\tMIT\tsoftware\t-")).is_err());
        assert!(parse(&line("x\tps2\thttps://a\t\tsoftware\t-")).is_err(), "no licence");
        assert!(parse(&line("x\tps2\thttps://a\tMIT\topengl\t-")).unwrap_err().contains("neither software nor vulkan"));
        assert!(parse(&line("x\tps2\thttps://a\tMIT\tsoftware\tpsx")).unwrap_err().contains("does not run"));
        assert!(parse("x\tps2\thttps://a\tMIT\tsoftware\tps2\ny\tps2\thttps://b\tMIT\tsoftware\tps2\n").unwrap_err().contains("two defaults"));
        assert!(parse("x\tps2\thttps://a\tMIT\tsoftware\t-\nx\tpsx\thttps://b\tMIT\tsoftware\t-\n").unwrap_err().contains("listed twice"));
    }

    #[test]
    fn the_json_names_every_field() {
        let one = parse("pcee2\tps2\thttps://a\tGPLv3\tvulkan\tps2\n").unwrap();
        assert_eq!(to_json(&one), r#"[{"stem":"pcee2","systems":["ps2"],"upstream":"https://a","licence":"GPLv3","renderer":"vulkan","default_for":["ps2"]}]"#);
    }
}
