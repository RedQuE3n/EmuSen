//! The systems' rules the platform library serves beside the frontend API's systems table: today the bytes OpenVGDB
//! hashed for a system's games (emusen_platform.h, `emusen_systems_openvgdb`; EmuSen_Settings_Reference.md §4.39).

use crate::error::Result;
use crate::platform::Platform;

impl Platform {
    /// The bytes of `file` OpenVGDB hashed for `system` (a header left out, an order made big-endian); none where they
    /// are the file's own.
    pub fn openvgdb_bytes(&self, system: &str, file: &[u8]) -> Result<Option<Vec<u8>>> {
        self.read(|o, l| unsafe { self.lib.emusen_systems_openvgdb(system.as_ptr(), system.len(), file.as_ptr(), file.len(), o, l) })
    }
}
