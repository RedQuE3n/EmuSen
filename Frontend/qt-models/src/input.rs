//! A running game's controller ports, fed from the pads' one reader (EmuSen_Qt.md §1.8.5): each new read taken from the
//! hand-over on the frame thread, routed by Endymion's router and told to the core between frames. No Qt here.

use emusen_crystal_client::shapes::SessionId;
use emusen_crystal_client::Runtime;
use emusen_platform_client::router::{consts, Change, Router};
use emusen_platform_client::Platform;
use emusen_qt_endymion::game::{GamePads, Read, Want};
use std::sync::Arc;

/// The game's side of the pads, owned by the frame thread.
pub struct GameInput {
    pads: Arc<GamePads>,
    router: Router,
    read: Read,
    changes: Vec<Change>,
    owned: bool,
}

impl GameInput {
    pub fn new(pads: Arc<GamePads>, platform: &Arc<Platform>) -> emusen_platform_client::Result<GameInput> {
        Ok(GameInput { pads, router: Router::new(platform)?, read: Read::default(), changes: Vec::new(), owned: false })
    }

    /// A game opened: its console, ports and axes asked of the pads, the router reset with nothing held.
    pub fn start(&mut self, runtime: &Runtime, session: SessionId, game: &str) -> Result<(), String> {
        let info = runtime.session_info(session).map_err(|e| e.to_string())?;
        // A core that declares no port has one, as the C# front end reads it.
        let ports = info.ports.len().max(1);
        self.router.reset(ports as i32, &info.axes).map_err(|e| e.to_string())?;
        self.pads.want(Some(Want { console: info.console, game: Some(game.to_string()), ports, axes: info.axes }));
        Ok(())
    }

    /// Whether the game hears the pads; given back, it hears nothing until every button is let go.
    pub fn set_owned(&mut self, owned: bool) {
        if self.owned != owned {
            self.owned = owned;
            self.pads.set_owned(owned);
        }
    }

    /// Before a frame: a newer read routed, and each change told to the core in the router's order.
    pub fn frame(&mut self, runtime: &Runtime, session: SessionId) -> Result<(), String> {
        if !self.pads.take(&mut self.read) {
            return Ok(());
        }
        let read = &self.read;
        self.router.poll(&read.held[..read.players], &read.axes, 0, read.seated, &mut self.changes).map_err(|e| e.to_string())?;
        for change in &self.changes {
            let port = u32::try_from(change.port).unwrap_or(0);
            let told = match change.kind {
                consts::CONNECTED => runtime.input_connected(session, port, change.on != 0),
                consts::BUTTON => runtime.input_button(session, port, change.which, change.on != 0),
                _ => runtime.input_axis(session, port, change.which, change.value),
            };
            told.map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

impl Drop for GameInput {
    fn drop(&mut self) {
        self.pads.set_owned(false);
        self.pads.want(None);
    }
}
