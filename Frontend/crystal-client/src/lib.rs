//! emusen-crystal-client: the frontend API, emusen_crystal.h 2.x, as safe Rust, over a raw layer bindgen generated
//! from the header (src/raw.rs). The library is loaded by its path and its version checked first; the JSON shapes the
//! header names are typed records; the per-frame calls are thin wrappers over its structs; the session service's table
//! is wrapped once. Nothing here knows Qt or names a runtime type (EmuSen_Qt.md §1.1.1).

mod api;
mod error;
mod events;
mod frame;
mod out;
mod raw;
mod runtime;
mod service;
pub mod shapes;

pub use api::Api;
pub use error::{Error, Result, status_name};
pub use events::{Event, Subscription, kinds};
pub use frame::{FrameInfo, RawInfo, frame_info, raw_info};
pub use runtime::{Config, LentLibrary, Open, Runtime, Scope};
pub use service::{SessionService, flags as service_flags};

/// The header's numbers a caller passes: open's flags, the speeds, the card manager's ops and the session service's ops.
pub mod consts {
    pub use crate::raw::{
        EMUSEN_CRYSTAL_BAD_STRUCT, EMUSEN_CRYSTAL_BUSY, EMUSEN_CRYSTAL_FIRMWARE, EMUSEN_CRYSTAL_IO, EMUSEN_CRYSTAL_NO_ENGINE,
        EMUSEN_CRYSTAL_NOT_APPROVED, EMUSEN_CRYSTAL_STATE_REFUSED, EMUSEN_SVC_ABSENT, EMUSEN_SVC_BAD_ARGUMENT, EMUSEN_SVC_FAULT,
        EMUSEN_SVC_NO_SESSION, EMUSEN_SVC_NO_SUCH_ID, EMUSEN_SVC_NOT_SUPPORTED, EMUSEN_SVC_READ_ONLY,
    };
    pub use crate::raw::{
        EMUSEN_CRYSTAL_CARDS_COPY as CARDS_COPY, EMUSEN_CRYSTAL_CARDS_DELETE as CARDS_DELETE,
        EMUSEN_CRYSTAL_CARDS_DUPLICATE as CARDS_DUPLICATE, EMUSEN_CRYSTAL_CARDS_IMPORT as CARDS_IMPORT,
        EMUSEN_CRYSTAL_CARDS_LIST as CARDS_LIST, EMUSEN_CRYSTAL_CARDS_MOVE as CARDS_MOVE, EMUSEN_CRYSTAL_CARDS_NEW as CARDS_NEW,
        EMUSEN_CRYSTAL_CARDS_OPEN as CARDS_OPEN, EMUSEN_CRYSTAL_CARDS_RENAME as CARDS_RENAME, EMUSEN_CRYSTAL_MAJOR as MAJOR,
        EMUSEN_CRYSTAL_MINOR as MINOR, EMUSEN_CRYSTAL_OPEN_NO_BATTERY as OPEN_NO_BATTERY, EMUSEN_CRYSTAL_OPEN_RING as OPEN_RING,
        EMUSEN_CRYSTAL_OPEN_SINGLE_ROWS as OPEN_SINGLE_ROWS, EMUSEN_CRYSTAL_SPEED_DRAW_NONE as SPEED_DRAW_NONE,
        EMUSEN_CRYSTAL_SPEED_NORMAL as SPEED_NORMAL, EMUSEN_CRYSTAL_SPEED_SLOW as SPEED_SLOW, EMUSEN_CRYSTAL_SPEED_TURBO as SPEED_TURBO,
        EMUSEN_SESSION_CHEAT_ADD as CHEAT_ADD, EMUSEN_SESSION_CHEAT_CLEAR as CHEAT_CLEAR, EMUSEN_SESSION_CHEAT_DB_STATUS as CHEAT_DB_STATUS, EMUSEN_SESSION_CHEAT_DECODE as CHEAT_DECODE,
        EMUSEN_SESSION_CHEAT_DISABLE as CHEAT_DISABLE, EMUSEN_SESSION_CHEAT_ENABLE as CHEAT_ENABLE, EMUSEN_SESSION_CHEAT_IMPORT as CHEAT_IMPORT,
        EMUSEN_SESSION_CHEAT_LIST as CHEAT_LIST, EMUSEN_SESSION_CHEAT_MASTER as CHEAT_MASTER, EMUSEN_SESSION_CHEAT_REMOVE as CHEAT_REMOVE,
        EMUSEN_SESSION_RUN_PAUSE as RUN_PAUSE, EMUSEN_SESSION_RUN_RESUME as RUN_RESUME,
        EMUSEN_SESSION_TELEMETRY_HARDWARE_LOAD as TELEMETRY_HARDWARE_LOAD,
    };
}
