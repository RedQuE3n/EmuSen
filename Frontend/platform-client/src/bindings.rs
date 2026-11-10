//! Endymion's binding files through the platform library: keybindings.json, gamepadbindings.json, gamepadprofiles.json
//! and hotkeybindings.json, each read as a program starts with it, saved, and a map's defaults. A model crosses as JSON
//! and stays Endymion's: its words and shapes are emusen_platform.h's, not restated here.

use crate::error::Result;
use crate::out::{json, to_json};
use crate::platform::Platform;
use serde_json::Value;

/// The four files, EMUSEN_ENDYMION_BINDINGS_*.
pub mod consts {
    pub use crate::raw::{
        EMUSEN_ENDYMION_BINDINGS_GAMEPAD as GAMEPAD, EMUSEN_ENDYMION_BINDINGS_HOTKEYS as HOTKEYS, EMUSEN_ENDYMION_BINDINGS_KEYS as KEYS,
        EMUSEN_ENDYMION_BINDINGS_PROFILES as PROFILES,
    };
}

impl Platform {
    /// A binding file's model for the consoles a program knows; a file that would not read is its defaults, and its
    /// diagnostic waits in `diagnostics`.
    pub fn bindings_load(&self, which: u32, consoles: &[&str]) -> Result<Value> {
        let names: Vec<u8> = consoles.iter().flat_map(|c| c.bytes().chain([0])).collect();
        let bytes =
            self.read(|o, l| unsafe { self.lib.emusen_endymion_bindings_load(which, names.as_ptr(), names.len(), o, l) })?.unwrap_or_default();
        json("bindings_load", &bytes)
    }

    /// A model saved as its binding file.
    pub fn bindings_save(&self, which: u32, model: &Value) -> Result<()> {
        let document = to_json(model);
        self.status(unsafe { self.lib.emusen_endymion_bindings_save(which, document.as_ptr(), document.len()) }.into()).map(drop)
    }

    /// The gamepad button each PadButton is bound to for a player of a console, SDL's number or -1, by the profiles' rules;
    /// `game` is the game's path, for a profile it names (since interface 43).
    pub fn bindings_bound(&self, console: &str, player: i32, game: Option<&str>) -> Result<[i32; 16]> {
        let mut out = [-1i32; 16];
        let (g, gl) = crate::out::text(game);
        self.status(unsafe {
            self.lib.emusen_endymion_bindings_bound(console.as_ptr(), console.len(), player, g, gl, out.as_mut_ptr(), out.len())
        })?;
        Ok(out)
    }

    /// Which map a player (1 to 8) of a console plays with, by the profiles' rules (since interface 43): the profile in
    /// its own spelling and the player whose map it is there, as `{"profile","mapPlayer"}`. `profile` names the profile
    /// to look in, or none for the one that applies to `game`; a model not given is read from its file.
    pub fn bindings_choose(
        &self,
        gamepad: Option<&Value>,
        profiles: Option<&Value>,
        console: &str,
        player: i32,
        game: Option<&str>,
        profile: Option<&str>,
    ) -> Result<Value> {
        let (gamepad, profiles) = (gamepad.map(to_json), profiles.map(to_json));
        let (g, gl) = crate::out::text(gamepad.as_deref());
        let (p, pl) = crate::out::text(profiles.as_deref());
        let (game, game_len) = crate::out::text(game);
        let (named, named_len) = crate::out::text(profile);
        let bytes = self
            .read(|o, l| unsafe {
                self.lib.emusen_endymion_bindings_choose(g, gl, p, pl, console.as_ptr(), console.len(), player, game, game_len, named, named_len, o, l)
            })?
            .unwrap_or_default();
        json("bindings_choose", &bytes)
    }

    /// The name a game is known by in a console's profiles, from its path (since interface 47).
    pub fn bindings_game_key(&self, path: &str) -> Result<String> {
        let bytes = self.read(|o, l| unsafe { self.lib.emusen_endymion_bindings_game_key(path.as_ptr(), path.len(), o, l) })?.unwrap_or_default();
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    /// One map at its defaults: a console's keys, a console's gamepad buttons, or the hotkeys.
    pub fn bindings_defaults(&self, which: u32) -> Result<Value> {
        let bytes = self.read(|o, l| unsafe { self.lib.emusen_endymion_bindings_defaults(which, o, l) })?.unwrap_or_default();
        json("bindings_defaults", &bytes)
    }
}
