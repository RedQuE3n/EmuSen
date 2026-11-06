//! The runtime's table with no game, `SERVICE_NO_GAME` since the frontend API's 1.12: what a shell with no game is
//! handed, answering as the C# provider over no target did. The cheat database, a `.cht` file's parse and the cheat
//! files are answered; `resume` is raised for the host's loop; every other call has no session (EmuSen_Backend.md
//! §40.7).

use emusen_session_service::abi::{run, status};

use super::args::Raw;
use super::{Answer, cheats, refused};
use crate::runtime::Runtime;
use crate::session::Refused;

/// `Games`' words where the host hands the provider no way to open one.
pub const NO_GAMES_WORDS: &str = "this host opens no games through the session service";

/// `RequireTarget`'s refusal, the answer to every call that needs a game.
pub fn no_session() -> Refused {
    refused(status::NO_SESSION, emusen_session_service::words::NO_SESSION)
}

/// `sessions`: none.
pub fn sessions() -> Vec<u8> {
    b"[]".to_vec()
}

/// `games`: refused, as the provider with no host's opener refused it.
pub fn games() -> Answer<Vec<u8>> {
    Err(refused(status::NOT_SUPPORTED, NO_GAMES_WORDS))
}

/// `run_control`: resume is the host loop's to answer, raised as a request; anything else needs a game.
pub fn run_control(rt: &Runtime, op: u32) -> Answer<i64> {
    if op != run::RESUME {
        return Err(no_session());
    }
    rt.no_game.raise("resume");
    Ok(0)
}

/// `events_drain`'s peek: the requests not yet drained, as a session's table lists them.
pub fn events_peek(rt: &Runtime, kinds: u32) -> Answer<(Vec<u8>, usize)> {
    rt.no_game.peek_runs(kinds)
}

/// The first `count` requests taken, once a buffer has held them all.
pub fn events_take(rt: &Runtime, count: usize) {
    rt.no_game.take_runs(count);
}

/// `cheat_op`: the ops that need no game, a game's refused as having no session.
pub fn cheat_op(rt: &Runtime, o: u32, raw: Raw, room: Option<usize>, read_only: bool) -> Answer<(Option<Vec<u8>>, i64)> {
    if !cheats::game_free(o) {
        return Err(no_session());
    }
    cheats::cheat_op_without_game(rt, o, raw, room, read_only)
}
