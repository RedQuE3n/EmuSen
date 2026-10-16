//! `emusen.session`: the session as a whole. `summary` prints the core's own text, and `coretop` refuses where its live
//! dashboard cannot be drawn, as the C# command did in a console window. See EmuSen_DianaSh.md §8.1 and §13.11.

use dianash_module::{Answer, Call, Manifest, Module};
use emusen_session_service::{abi, Session};

/// The commands this module has, with their read-only flags and today's usage lines.
pub const COMMANDS: &[(&str, bool, &str)] = &[
    (
        "coretop",
        false,
        "  coretop [-w]                  live htop-style dashboard of the loaded core's hardware (Ctrl+C to exit - interactive terminal only;\n                                -w opens it in a separate window instead, if this frontend supports one)",
    ),
    ("summary", false, "  summary                       free-text state dump (whatever isn't structured above yet)"),
];

/// The module: one value, no state of its own.
pub struct SessionCommands;

pub static MODULE: SessionCommands = SessionCommands;

fn summary(call: &Call) -> Answer {
    let session = Session::from_service(call.service(abi::SERVICE, abi::MAJOR));
    match session.and_then(|s| s.summary()) {
        Ok(text) => Answer::ok(text),
        Err(refusal) if refusal.is(abi::status::NO_SESSION) => Answer::ok("No target attached."),
        Err(refusal) => Answer::fault(refusal.words),
    }
}

/// `coretop`: a window is the frontend service's to open, and the live dashboard needs a terminal of its own, which a
/// call through the module ABI's buffered output cannot drive; both are refused in the C# command's words.
fn coretop(call: &Call, args: &[String]) -> Answer {
    let session = Session::from_service(call.service(abi::SERVICE, abi::MAJOR)).and_then(|s| s.info().map(|_| s));
    if let Err(refusal) = session {
        return Answer::fail(refusal.words);
    }
    if args.iter().any(|a| a.eq_ignore_ascii_case("-w")) {
        return Answer::fail("coretop: -w is not supported by this frontend.");
    }
    Answer::fail("coretop: needs a real interactive terminal (stdin/stdout is redirected, or this console has none)")
}

impl Module for SessionCommands {
    fn manifest(&self) -> &'static str {
        static MANIFEST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(|| {
            Manifest { id: "emusen.session", version: env!("CARGO_PKG_VERSION"), services: &[(abi::SERVICE, abi::MAJOR, 0, false)], commands: COMMANDS, aliases: &[], jobs: &[] }.json()
        })
    }

    fn run(&self, call: &Call) -> i32 {
        let args = call.args();
        call.answer(match call.command().as_str() {
            "summary" => summary(call),
            "coretop" => coretop(call, &args),
            other => Answer::fault(format!("emusen.session has no command '{other}'")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_manifest_names_both_commands() {
        let manifest = SessionCommands.manifest();
        assert!(manifest.contains("\"name\":\"summary\"") && manifest.contains("\"name\":\"coretop\""));
        assert!(manifest.starts_with("{\"manifest\":1,\"id\":\"emusen.session\""));
    }
}
