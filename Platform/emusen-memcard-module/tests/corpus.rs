//! memcard's cases of the golden corpus (`memcard.golden`, recorded from the C# command it replaced), run through a
//! shell with this module over the media the C# packs named when they were recorded. See EmuSen_Backend.md §40.11.3.

#[path = "../../dianash/tests/corpus/mod.rs"]
mod corpus;

use emusen_dianash::shell::{Config, Shell};

/// The media the C# shell's packs named when the corpus was recorded, as the engines declare them.
fn recorded_media() -> Vec<(String, String)> {
    [("psx.card", ".mcr"), ("n64.pak", ".mpk"), ("mcd.backup", ".brm"), ("mcd.ramcart", ".crm"), ("ps2.card", ".ps2")].iter().map(|(i, e)| (i.to_string(), e.to_string())).collect()
}

static MEMCARD: emusen_memcard_module::MemcardCommands = emusen_memcard_module::MemcardCommands::over(recorded_media);

fn shell_with_memcard(config: Config) -> Shell {
    let mut shell = Shell::new(config);
    shell.add_module(&MEMCARD).unwrap();
    shell
}

#[test]
fn memcard_prints_what_the_csharp_command_printed() {
    let t = corpus::run_corpus(Some(&["memcard.golden"]), shell_with_memcard);
    assert!(t.failures.is_empty(), "{}", t.failures.join("\n\n"));
    assert_eq!((t.passed, t.needs_host, t.case_of_the_system), (4, 0, 0));
}
