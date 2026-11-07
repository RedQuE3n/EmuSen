//! `emusen.memcard`: `memcard`, the card manager of every console that keeps saves on a medium of its own, over
//! emusen-savemedia. A card is a file in the sandbox; which medium it is goes by its extension among the media the
//! program's engines declare, in the systems table's order. It needs no game. See `man memcard`, EmuSen_SaveMedia.md §6
//! and EmuSen_Backend.md §40.11.3.

use dianash_module::{Answer, Call, Manifest, Module};
use emusen_dotnet::text::{compare_ignore_case, equals_ignore_case};
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_savemedia::{Card, Done, Manager, Media, Medium, Outcome, Thrown};

const USAGE: &str = "  memcard ls [<card>]           every card under Saves, or the saves on one card\n  memcard cp <card>:<save> <card>\n                                copy a save onto another card of its medium\n  memcard mv <card>:<save> <card>\n                                copy a save, read it back, then delete the first\n  memcard rm <card>:<save>      delete a save as the console's own manager does\n  memcard import <card> <from>...\n                                every save of <from> onto <card>, all or none\n  memcard new <card>            a new card, formatted where its medium allows\n  memcard dup <card> <new>      a whole card copied to a new name\n  memcard rename <card> <new>   a card renamed";

const USE: &str = "Usage: memcard ls|cp|mv|rm|import|new|dup|rename ... (see 'man memcard')";

pub const COMMANDS: &[(&str, bool, &str)] = &[("memcard", false, USAGE)];

/// The module, over a source of `(medium id, extension)` pairs: the engines' declarations in a program.
pub struct MemcardCommands {
    declared: fn() -> Vec<(String, String)>,
}

pub static MODULE: MemcardCommands = MemcardCommands { declared: installed };

impl MemcardCommands {
    /// The module over another source of declared media, as a test gives one.
    pub const fn over(declared: fn() -> Vec<(String, String)>) -> MemcardCommands {
        MemcardCommands { declared }
    }
}

/// What the program's engines declare, scanned from its cores directories as the runtime scans them.
fn installed() -> Vec<(String, String)> {
    let development = std::env::var(emusen_core_host::discovery::DEVELOPMENT_VARIABLE).as_deref() == Ok("1");
    let discovery = emusen_core_host::Discovery::scan(&emusen_crystal::runtime::installation_directories(), development);
    emusen_crystal::media::declared(&discovery)
}

/// The known media: each the systems table names and an engine declares, in the table's order, with its extension.
struct Known(Vec<(String, Medium)>);

impl Known {
    fn of(declared: &[(String, String)]) -> Known {
        let rows = emusen_systems::table().map(|t| t.iter().flat_map(|s| s.media.iter()).collect::<Vec<_>>()).unwrap_or_default();
        Known(rows.iter().filter_map(|row| declared.iter().find(|(id, _)| *id == row.id).and_then(|(id, e)| Some((e.clone(), Medium::by_id(id)?)))).collect())
    }

    fn named(&self) -> String {
        self.0.iter().map(|(e, m)| format!("{e} ({})", m.title)).collect::<Vec<_>>().join(", ")
    }
}

impl Media for Known {
    fn medium_of(&self, file: &str) -> Option<Medium> {
        let extension = path::extension(Style::HOST, file);
        self.0.iter().find(|(e, _)| equals_ignore_case(e, extension)).map(|(_, m)| m.clone())
    }
}

fn done(outcome: Done) -> Answer {
    match outcome {
        Ok(o) => said(o),
        Err(Thrown::Io(e)) => Answer::fail(format!("memcard: {e}")),
        Err(Thrown::Range) => Answer::fault("Specified argument was out of the range of valid values."),
    }
}

fn said(o: Outcome) -> Answer {
    if o.done { Answer::ok(o.words) } else { Answer::fail(format!("memcard: {}", o.words)) }
}

/// One run of the command, over the media known when it began.
struct Run<'a> {
    call: &'a Call<'a>,
    known: &'a Known,
    manager: Manager<'a, Known>,
}

impl Run<'_> {
    /// A card path in the sandbox, of a known medium.
    fn card(&self, given: &str) -> Result<String, Answer> {
        let real = self.call.resolve(given).map_err(|words| Answer::fail(format!("memcard: {words}")))?;
        if self.known.medium_of(&real).is_none() {
            return Err(Answer::fail(format!("memcard: {} is no card: the media known are {}", path::file_name(Style::HOST, given), self.known.named())));
        }
        Ok(real)
    }

    /// `<card>:<save>`, split at the last colon, as a save's id holds none and a path rarely does.
    fn card_and_save(&self, given: &str) -> Result<(String, String), Answer> {
        match given.rfind(':') {
            Some(colon) if colon > 0 && colon < given.len() - 1 => Ok((self.card(&given[..colon])?, given[colon + 1..].to_string())),
            _ => Err(Answer::fail(format!("memcard: '{given}' names no save; write <card>:<save>, the save by its name or its number in 'memcard ls'"))),
        }
    }

    fn shown(&self, real: &str) -> String {
        self.call.display(real).trim_start_matches('/').to_string()
    }

    /// One line for a card: its path, its medium, its use, and who holds it.
    fn summary(&self, real: &str, card: &Card) -> String {
        let usage = match (&card.listing, card.medium.module) {
            (Some(l), Some(m)) if l.formatted => format!("{} save(s), {} of {} {}s free", l.saves.len(), l.free, m.capacity, m.unit),
            (Some(l), _) => l.problem.clone().unwrap_or_else(|| "not formatted".to_string()),
            (None, _) => format!("{} bytes, no format module", card.length),
        };
        let held = emusen_galaxia::medium_lock::holder_of(real).map_or(String::new(), |h| format!("  [in use by {h}]"));
        format!("{}  {}  {usage}{held}", self.shown(real), card.medium.title)
    }

    fn open(&self, real: &str) -> Result<Card, Answer> {
        match self.manager.open(real) {
            Ok(Some(card)) => Ok(card),
            Ok(None) => Err(Answer::fail(format!("memcard: {} is no card", path::file_name(Style::HOST, real)))),
            Err(Thrown::Io(e)) => Err(Answer::fail(format!("memcard: {e}"))),
            Err(Thrown::Range) => Err(Answer::fault("Specified argument was out of the range of valid values.")),
        }
    }

    /// Every card under Saves, by console: each console's Cards and Console folders and its own files.
    fn list_all(&self) -> Answer {
        let saves = emusen_galaxia::tree::tree().saves();
        let mut consoles = folders(&saves);
        consoles.sort_by(|a, b| compare_ignore_case(a, b));
        let mut lines: Vec<String> = Vec::new();
        for console in consoles {
            let mut cards: Vec<String> = [emusen_crystal::media::CARDS_FOLDER, emusen_crystal::media::CONSOLE_FOLDER]
                .iter()
                .flat_map(|f| files(&path::combine(Style::HOST, &console, f)))
                .chain(files(&console))
                .filter(|f| self.known.medium_of(f).is_some())
                .collect();
            if cards.is_empty() {
                continue;
            }
            cards.sort_by(|a, b| compare_ignore_case(a, b));
            lines.push(path::file_name(Style::HOST, &console).to_string());
            for card in cards {
                match self.open(&card) {
                    Ok(c) => lines.push(format!("  {}", self.summary(&card, &c))),
                    Err(answer) => return answer,
                }
            }
        }
        if lines.is_empty() { Answer::ok("No cards: nothing under Saves is a file of a medium a system pack names.") } else { Answer::ok(lines.join("\n")) }
    }

    fn list_one(&self, given: &str) -> Answer {
        let real = match self.card(given) {
            Ok(r) => r,
            Err(a) => return a,
        };
        let card = match self.open(&real) {
            Ok(c) => c,
            Err(a) => return a,
        };
        let mut lines = vec![self.summary(&real, &card)];
        if let (Some(listing), Some(module)) = (&card.listing, card.medium.module)
            && listing.formatted
        {
            for (n, s) in listing.saves.iter().enumerate() {
                lines.push(format!("  {:>2}  {:>2} {}(s)  {}  {}{}", n + 1, s.units, module.unit, s.id, s.title, if s.damaged { "  [damaged]" } else { "" }));
            }
        }
        Answer::ok(lines.join("\n"))
    }

    fn transfer(&self, verb: &str, from: &str, to: &str) -> Result<Answer, Answer> {
        let (source, save) = self.card_and_save(from)?;
        let target = self.card(to)?;
        Ok(done(if verb == "cp" { self.manager.copy_save(&source, &save, &target) } else { self.manager.move_save(&source, &save, &target) }))
    }

    fn remove(&self, given: &str) -> Result<Answer, Answer> {
        let (card, save) = self.card_and_save(given)?;
        Ok(done(self.manager.delete_save(&card, &save)))
    }

    fn import(&self, to: &str, from: &[String]) -> Result<Answer, Answer> {
        let target = self.card(to)?;
        let sources = from.iter().map(|f| self.card(f)).collect::<Result<Vec<_>, _>>()?;
        Ok(done(self.manager.import(&target, &sources)))
    }

    fn new_card(&self, given: &str) -> Result<Answer, Answer> {
        Ok(said(self.manager.new_card(&self.card(given)?)))
    }

    fn pair(&self, verb: &str, from: &str, to: &str) -> Result<Answer, Answer> {
        let source = self.card(from)?;
        let target = self.card(to)?;
        Ok(if verb == "dup" { said(self.manager.duplicate(&source, &target)) } else { done(self.manager.rename(&source, &target)) })
    }
}

fn folders(dir: &str) -> Vec<String> {
    entries(dir, true)
}

fn files(dir: &str) -> Vec<String> {
    entries(dir, false)
}

fn entries(dir: &str, folders: bool) -> Vec<String> {
    let Ok(read) = std::fs::read_dir(dir) else { return Vec::new() };
    read.flatten().filter(|e| e.file_type().is_ok_and(|t| t.is_dir() == folders)).map(|e| path::combine(Style::HOST, dir, &e.file_name().to_string_lossy())).collect()
}

impl Module for MemcardCommands {
    fn manifest(&self) -> &'static str {
        static MANIFEST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(|| Manifest { id: "emusen.memcard", version: env!("CARGO_PKG_VERSION"), services: &[], commands: COMMANDS, aliases: &[], jobs: &[], settings: &[], approvals: false }.json())
    }

    fn run(&self, call: &Call) -> i32 {
        if call.command() != "memcard" {
            return call.answer(Answer::fault(format!("emusen.memcard has no command '{}'", call.command())));
        }
        let known = Known::of(&(self.declared)());
        let run = Run { call, known: &known, manager: Manager::new(&known) };
        let args = call.args();
        let words: Vec<&str> = args.iter().map(String::as_str).collect();
        let answer = match words.as_slice() {
            [_, "ls"] => Ok(run.list_all()),
            [_, "ls", card] => Ok(run.list_one(card)),
            [_, verb @ ("cp" | "mv"), from, to] => run.transfer(verb, from, to),
            [_, "rm", card] => run.remove(card),
            [_, "import", to, from @ ..] if !from.is_empty() => run.import(to, &args[3..]),
            [_, "new", card] => run.new_card(card),
            [_, verb @ ("dup" | "rename"), from, to] => run.pair(verb, from, to),
            _ => Ok(Answer::fail(USE)),
        };
        call.answer(answer.unwrap_or_else(|refused| refused))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
        list.iter().map(|(i, e)| (i.to_string(), e.to_string())).collect()
    }

    #[test]
    fn the_known_media_follow_the_table_whatever_order_the_engines_declare_them_in() {
        let known = Known::of(&pairs(&[("ps2.card", ".ps2"), ("psx.card", ".mcr"), ("n64.pak", ".mpk"), ("mcd.backup", ".brm"), ("mcd.ramcart", ".crm")]));
        assert_eq!(known.named(), ".crm (Backup RAM Cartridge), .brm (Internal Backup RAM), .mcr (Memory Card), .mpk (Controller Pak), .ps2 (Memory Card (PS2))");
        assert_eq!(known.medium_of("/a/Card 1.MCR").map(|m| m.id), Some("psx.card".to_string()));
        assert!(known.medium_of("/a/notes.txt").is_none());
    }

    #[test]
    fn a_medium_no_engine_declares_is_not_known() {
        let known = Known::of(&pairs(&[("psx.card", ".mcr"), ("nobody.card", ".xyz")]));
        assert_eq!(known.named(), ".mcr (Memory Card)");
        assert!(known.medium_of("x.mpk").is_none());
    }

    #[test]
    fn the_manifest_names_memcard_and_its_page() {
        let m = MODULE.manifest();
        assert!(m.contains("\"id\":\"emusen.memcard\"") && m.contains("\"name\":\"memcard\"") && m.contains("man/memcard.1"), "{m}");
    }
}
