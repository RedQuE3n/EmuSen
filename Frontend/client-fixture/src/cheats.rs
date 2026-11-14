//! A cheat database's archive on the loopback, built from this commit's `Platform/` as its `cheats_server` example and
//! run beside the test: the files a test writes into a folder, zipped and served whole, or half and then held so a
//! download stands part way until it is stopped, or held the first time and whole after (EmuSen_RustPlatform.md §56). Never the real address.

use crate::World;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;

/// The game the cheat tests give a list, and the database folder its file is under: the SNES's first cheat system in
/// the systems table (Platform/systems/data/snes.json, `cheat_systems`).
pub const GAME: &str = "Cobalt Harbor (Synthetic)";
pub const SNES: &str = "Nintendo - Super Nintendo Entertainment System";

/// The game's file in the database: two Pro Action Replay codes, the first on.
pub const CHT: &str = "cheats = 2\ncheat0_desc = \"Infinite lives\"\ncheat0_code = \"7E0DBE09\"\ncheat0_enable = true\n\
                       cheat1_desc = \"Moon jump\"\ncheat1_code = \"7E0DBF63\"\ncheat1_enable = false\n";

/// [`CHT`] written as [`GAME`]'s file under `world`'s `Served` folder, and the server over that folder.
pub fn served(world: &World) -> CheatsServer {
    std::fs::write(world.dir(&format!("Served/{SNES}")).join(format!("{GAME}.cht")), CHT).unwrap();
    CheatsServer::start(&world.root.join("Served"))
}

/// The server, running while this is held; it ends when this is dropped.
pub struct CheatsServer {
    child: Child,
    /// The archive whole.
    pub url: String,
    /// The archive's first half, the rest held back.
    pub held: String,
    /// Held the first time it is asked, whole after.
    pub first_held: String,
}

fn built() -> &'static Path {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT.get_or_init(|| crate::example("cheats_server"))
}

impl CheatsServer {
    /// The files under `folder` served, its three addresses read.
    pub fn start(folder: &Path) -> CheatsServer {
        let mut child = Command::new(built()).arg(folder).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().expect("the cheats server starts");
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let mut next = || lines.next().and_then(Result::ok).expect("an address from the cheats server");
        let (url, held, first_held) = (next(), next(), next());
        CheatsServer { url, held, first_held, child }
    }
}

impl Drop for CheatsServer {
    fn drop(&mut self) {
        drop(self.child.stdin.take());
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
