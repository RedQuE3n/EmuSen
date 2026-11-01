//! The libretro cores EmuSen hosts (`data/libretro-cores.tsv`): the one list Cores offers for download and the host
//! runs, each with the systems it is placed under and its upstream source (EmuSen_Cores.md §4.7, decided 2026-11-01).

/// One hosted libretro core.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LibretroCore {
    /// The core's file stem, `pcee2` of `pcee2_libretro.so`.
    pub stem: String,
    /// The EmuSen system ids it runs.
    pub systems: Vec<String>,
    /// Where its source is published.
    pub upstream: String,
}

/// The list as this build ships it.
pub const SHIPPED: &str = include_str!("../data/libretro-cores.tsv");

/// The lines of a list, `#` lines and blank lines skipped; a line of another shape is an error naming it.
pub fn parse(text: &str) -> Result<Vec<LibretroCore>, String> {
    let mut cores = Vec::new();
    for (n, line) in text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty() && !l.starts_with('#')) {
        let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
        let [stem, systems, upstream] = fields.as_slice() else { return Err(format!("libretro-cores.tsv line {} has not three fields", n + 1)) };
        let systems: Vec<String> = systems.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
        if stem.is_empty() || systems.is_empty() || !upstream.starts_with("https://") {
            return Err(format!("libretro-cores.tsv line {} needs a stem, a system and an https:// source", n + 1));
        }
        cores.push(LibretroCore { stem: stem.to_string(), systems, upstream: upstream.to_string() });
    }
    Ok(cores)
}

/// The shipped list, read once.
pub fn hosted() -> &'static [LibretroCore] {
    static LIST: std::sync::OnceLock<Vec<LibretroCore>> = std::sync::OnceLock::new();
    LIST.get_or_init(|| parse(SHIPPED).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_list_reads_and_names_pcee2_and_flycast() {
        let list = parse(SHIPPED).unwrap();
        assert_eq!(list.iter().map(|c| c.stem.as_str()).collect::<Vec<_>>(), ["pcee2", "flycast"]);
        assert_eq!(list[1].systems, ["dreamcast", "naomi", "atomiswave"]);
        assert!(parse("x\tps2\n").is_err());
        assert!(parse("x\tps2\thttp://insecure\n").is_err());
    }
}
