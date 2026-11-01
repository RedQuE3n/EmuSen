//! Which games a hosted libretro core takes: the extensions its own `.info` lists under `supported_extensions`, for the
//! systems its `libretro-cores.tsv` line names, with the named exceptions of `data/libretro-extensions.tsv` applied
//! (EmuSen_LibretroHost.md §4.1a, 2026-11-01).

use crate::libretro_cores::{self, LibretroCore};
use std::path::Path;

/// One named exception: extensions EmuSen adds to or drops from a core's own list, for one system.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exception {
    pub stem: String,
    pub system: String,
    pub add: Vec<String>,
    pub drop: Vec<String>,
    pub reason: String,
}

/// The exceptions as this build ships them.
pub const SHIPPED: &str = include_str!("../data/libretro-extensions.tsv");

/// The `supported_extensions` of a `.info`'s text, lower-cased, without dots; empty where the key is absent or blank.
pub fn info_extensions(info: &str) -> Vec<String> {
    let value = info.lines().map(|l| l.trim_start_matches('\u{FEFF}').trim()).find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (!line.starts_with('#') && key.trim() == "supported_extensions").then_some(value.trim())
    });
    let Some(raw) = value else { return Vec::new() };
    let text = match raw.strip_prefix('"') {
        Some(rest) => rest.find('"').map_or(rest, |end| &rest[..end]),
        None => raw.split('#').next().unwrap_or(""),
    };
    exts(text)
}

fn exts(field: &str) -> Vec<String> {
    if field.trim() == "-" {
        return Vec::new();
    }
    field.split('|').map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase()).filter(|e| !e.is_empty()).collect()
}

/// The lines of an exceptions list, checked against the cores list; a line of another shape is an error naming it.
pub fn parse(text: &str, cores: &[LibretroCore]) -> Result<Vec<Exception>, String> {
    let mut out: Vec<Exception> = Vec::new();
    for (n, line) in text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty() && !l.starts_with('#')) {
        let line_no = n + 1;
        let fields: Vec<&str> = line.split('\t').map(str::trim).collect();
        let [stem, system, add, drop, reason] = fields.as_slice() else {
            return Err(format!("libretro-extensions.tsv line {line_no} has not five fields"));
        };
        let Some(core) = cores.iter().find(|c| c.stem == *stem) else {
            return Err(format!("libretro-extensions.tsv line {line_no}: {stem} is not a hosted core"));
        };
        if !core.systems.iter().any(|s| s == system) {
            return Err(format!("libretro-extensions.tsv line {line_no}: {stem} does not run {system}"));
        }
        let (add, drop) = (exts(add), exts(drop));
        if reason.is_empty() || (add.is_empty() && drop.is_empty()) {
            return Err(format!("libretro-extensions.tsv line {line_no} needs an extension added or dropped and a reason"));
        }
        if out.iter().any(|e| e.stem == *stem && e.system == *system) {
            return Err(format!("libretro-extensions.tsv line {line_no}: {stem} on {system} is listed twice"));
        }
        out.push(Exception { stem: stem.to_string(), system: system.to_string(), add, drop, reason: reason.to_string() });
    }
    Ok(out)
}

/// The shipped exceptions, read once.
pub fn shipped() -> &'static [Exception] {
    static LIST: std::sync::OnceLock<Vec<Exception>> = std::sync::OnceLock::new();
    LIST.get_or_init(|| parse(SHIPPED, libretro_cores::hosted()).unwrap_or_default())
}

fn ends_in(name: &str, ext: &str) -> bool {
    name.len() > ext.len() + 1 && name.ends_with(ext) && name.as_bytes()[name.len() - ext.len() - 1] == b'.'
}

/// Whether a core takes a game file for a system, under the given cores list and exceptions.
pub fn takes_under(cores: &[LibretroCore], exceptions: &[Exception], stem: &str, supported: &[String], system: &str, game: &Path) -> bool {
    if !cores.iter().any(|c| c.stem == stem && c.systems.iter().any(|s| s == system)) {
        return false;
    }
    let Some(name) = game.file_name().and_then(|n| n.to_str()).map(str::to_ascii_lowercase) else { return false };
    let rule = exceptions.iter().find(|e| e.stem == stem && e.system == system);
    let listed = |list: &[String]| list.iter().any(|e| ends_in(&name, &e.trim_start_matches('.').to_ascii_lowercase()));
    if rule.is_some_and(|r| listed(&r.drop)) {
        return false;
    }
    listed(supported) || rule.is_some_and(|r| listed(&r.add))
}

/// Whether the hosted core `stem`, whose `.info` lists `supported`, takes `game` as a game of `system`.
pub fn takes(stem: &str, supported: &[String], system: &str, game: &Path) -> bool {
    takes_under(libretro_cores::hosted(), shipped(), stem, supported, system, game)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PCEE2_INFO: &str = "display_name = \"Sony - PlayStation 2 (PCEE2)\"\nsupported_extensions = \"iso|chd|cso|zso|gz|bin|mdf|nrg|elf|irx\"\nlicense = \"GPLv3\"\n";
    const FLYCAST_INFO: &str = "# Flycast\nsupported_extensions = \"chd|cdi|iso|elf|bin|cue|gdi|lst|zip|dat|7z|m3u\"\nsystemid = \"dreamcast\"\n";

    fn take(info: &str, stem: &str, system: &str, file: &str) -> bool {
        takes(stem, &info_extensions(info), system, Path::new(file))
    }

    #[test]
    fn the_info_value_is_read_quoted_or_bare() {
        assert_eq!(info_extensions(PCEE2_INFO), ["iso", "chd", "cso", "zso", "gz", "bin", "mdf", "nrg", "elf", "irx"]);
        assert_eq!(info_extensions("\u{FEFF}supported_extensions = ISO|.Bin # bare\n"), ["iso", "bin"]);
        assert_eq!(info_extensions("# supported_extensions = \"iso\"\nsupported_extensions=\"a|b\"\nsupported_extensions = \"c\"\n"), ["a", "b"], "a comment skipped, the first key kept");
        assert!(info_extensions("corename = \"x\"\n").is_empty());
        assert!(info_extensions("supported_extensions = \"\"\n").is_empty());
    }

    #[test]
    fn pcee2_takes_its_ten_extensions_for_the_playstation_2() {
        for ext in ["iso", "chd", "cso", "zso", "gz", "bin", "mdf", "nrg", "elf", "irx"] {
            assert!(take(PCEE2_INFO, "pcee2", "ps2", &format!("/games/ps2/Game.{ext}")), "{ext}");
        }
        assert!(take(PCEE2_INFO, "pcee2", "ps2", "Game.ISO"), "case is ignored");
        assert!(take(PCEE2_INFO, "pcee2", "ps2", "Game.iso.gz"));
    }

    #[test]
    fn pcee2_refuses_a_cue_so_a_cue_bin_set_goes_through_its_bin() {
        assert!(!take(PCEE2_INFO, "pcee2", "ps2", "Game (USA).cue"));
        assert!(take(PCEE2_INFO, "pcee2", "ps2", "Game (USA).bin"));
    }

    #[test]
    fn flycast_takes_its_twelve_extensions_for_each_system_of_its_line() {
        for system in ["dreamcast", "naomi", "atomiswave"] {
            for ext in ["chd", "cdi", "iso", "elf", "bin", "cue", "gdi", "lst", "zip", "dat", "7z", "m3u"] {
                assert!(take(FLYCAST_INFO, "flycast", system, &format!("Game.{ext}")), "{system} {ext}");
            }
        }
        assert!(!take(FLYCAST_INFO, "flycast", "dreamcast", "Game.ccd"), "the Dreamcast table lists .ccd, Flycast does not");
    }

    #[test]
    fn a_core_takes_nothing_for_a_system_its_line_does_not_name() {
        assert!(!take(PCEE2_INFO, "pcee2", "psx", "Game.bin"));
        assert!(!take(FLYCAST_INFO, "flycast", "saturn", "Game.cue"));
        assert!(!take(PCEE2_INFO, "unlisted", "ps2", "Game.iso"), "a core not on the list");
    }

    #[test]
    fn a_name_without_the_extension_is_refused() {
        for file in ["iso", ".iso", "Game", "Game.", "Gameiso", "/games/iso/Game"] {
            assert!(!take(PCEE2_INFO, "pcee2", "ps2", file), "{file}");
        }
    }

    #[test]
    fn the_named_exceptions_add_and_drop_for_their_system_only() {
        let cores = libretro_cores::parse(libretro_cores::SHIPPED).unwrap();
        let rules = parse("flycast\tdreamcast\tccd\tzip|7z\tan EmuSen rule\n", &cores).unwrap();
        let ext = info_extensions(FLYCAST_INFO);
        let under = |system: &str, file: &str| takes_under(&cores, &rules, "flycast", &ext, system, Path::new(file));
        assert!(under("dreamcast", "Game.ccd"));
        assert!(!under("dreamcast", "Game.zip") && !under("dreamcast", "Game.7z"));
        assert!(under("dreamcast", "Game.gdi"));
        assert!(under("naomi", "Game.zip") && !under("naomi", "Game.ccd"));
    }

    #[test]
    fn an_exception_of_another_shape_is_refused_with_its_number() {
        let cores = libretro_cores::parse(libretro_cores::SHIPPED).unwrap();
        let p = |s: &str| parse(&format!("# c\n{s}\n"), &cores);
        assert_eq!(p("pcee2\tps2\tcue").unwrap_err(), "libretro-extensions.tsv line 2 has not five fields");
        assert!(p("nobody\tps2\tcue\t-\tr").unwrap_err().contains("not a hosted core"));
        assert!(p("pcee2\tpsx\tcue\t-\tr").unwrap_err().contains("does not run psx"));
        assert!(p("pcee2\tps2\tcue\t-\t").unwrap_err().contains("a reason"));
        assert!(p("pcee2\tps2\t-\t-\tr").unwrap_err().contains("added or dropped"));
        assert!(parse("pcee2\tps2\tcue\t-\tr\npcee2\tps2\t-\tirx\tr\n", &cores).unwrap_err().contains("listed twice"));
    }

    #[test]
    fn the_shipped_exceptions_read_whole() {
        assert_eq!(parse(SHIPPED, libretro_cores::hosted()).unwrap(), shipped());
    }
}
